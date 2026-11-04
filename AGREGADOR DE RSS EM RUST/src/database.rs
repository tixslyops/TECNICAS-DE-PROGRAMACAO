use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDateTime, Utc};
use mysql::prelude::*;
use mysql::{params, Pool, Row, Value};

use crate::models::{Article, Feed, NewArticle};

pub struct Database {
    pool: Pool,
}

impl Database {
    pub fn new(database_url: &str) -> Result<Self> {
        let pool =
            Pool::new(database_url).context("Não foi possível criar a conexão com o MySQL")?;

        let db = Self { pool };

        // Verifica se a conexão está funcionando.
        let mut conn = db.pool.get_conn()?;

        conn.query_drop("SELECT 1")
            .context("Não foi possível testar a conexão com o MySQL")?;

        Ok(db)
    }

    // =========================================================
    // FEEDS
    // =========================================================

    pub fn add_feed(&self, title: &str, url: &str) -> Result<()> {
        let mut conn = self.pool.get_conn()?;

        // Verifica se a URL já está cadastrada.
        let existing: Option<i64> = conn.exec_first(
            "SELECT id FROM feeds WHERE url = :url LIMIT 1",
            params! {
                "url" => url,
            },
        )?;

        if existing.is_some() {
            anyhow::bail!("Este feed já está cadastrado.");
        }

        conn.exec_drop(
            "INSERT INTO feeds (title, url) VALUES (:title, :url)",
            params! {
                "title" => title,
                "url" => url,
            },
        )?;

        Ok(())
    }

    pub fn list_feeds(&self) -> Result<Vec<Feed>> {
        let mut conn = self.pool.get_conn()?;

        let rows: Vec<Row> = conn.query(
            "SELECT id, title, url, last_checked, last_success,
                    etag, last_modified, failure_count
             FROM feeds
             ORDER BY id",
        )?;

        let feeds = rows
            .into_iter()
            .map(|row| Feed {
                id: row.get("id").unwrap_or_default(),
                title: row.get("title").unwrap_or_default(),
                url: row.get("url").unwrap_or_default(),
                last_checked: row
                    .get::<Option<NaiveDateTime>, _>("last_checked")
                    .flatten()
                    .map(datetime_from_mysql),
                last_success: row
                    .get::<Option<NaiveDateTime>, _>("last_success")
                    .flatten()
                    .map(datetime_from_mysql),
                etag: row.get("etag").unwrap_or(None),
                last_modified: row.get("last_modified").unwrap_or(None),
                failure_count: row
                    .get::<i32, _>("failure_count")
                    .unwrap_or_default()
                    .max(0) as u32,
            })
            .collect();

        Ok(feeds)
    }

    pub fn remove_feed(&self, id: i64) -> Result<()> {
        let mut conn = self.pool.get_conn()?;

        conn.exec_drop(
            "DELETE FROM feeds WHERE id = :id",
            params! {
                "id" => id,
            },
        )?;

        if conn.affected_rows() == 0 {
            anyhow::bail!("Feed não encontrado.");
        }

        Ok(())
    }

    // =========================================================
    // ARTICLES
    // =========================================================

    pub fn insert_article(&self, article: &NewArticle) -> Result<bool> {
        let mut conn = self.pool.get_conn()?;

        let published = article.published.map(datetime_to_mysql);

        conn.exec_drop(
            "INSERT IGNORE INTO articles
             (feed_id, title, author, link, published,
              description, fetched_at, guid)
             VALUES
             (:feed_id, :title, :author, :link, :published,
              :description, :fetched_at, :guid)",
            params! {
                "feed_id" => article.feed_id,
                "title" => &article.title,
                "author" => &article.author,
                "link" => &article.link,
                "published" => published,
                "description" => &article.description,
                "fetched_at" => Utc::now().naive_utc(),
                "guid" => &article.guid,
            },
        )?;

        // INSERT IGNORE retorna 0 quando o artigo já existe.
        Ok(conn.affected_rows() > 0)
    }

    pub fn list_articles(
        &self,
        unread_only: bool,
        feed_id: Option<i64>,
        search: Option<&str>,
    ) -> Result<Vec<Article>> {
        let mut conn = self.pool.get_conn()?;

        let mut sql = String::from(
            "SELECT id, feed_id, title, author, link, published,
                    description, fetched_at, is_read, is_favorite, guid
             FROM articles
             WHERE 1 = 1",
        );

        let mut values: Vec<Value> = Vec::new();

        if unread_only {
            sql.push_str(" AND is_read = ?");
            values.push(Value::from(0));
        }

        if let Some(id) = feed_id {
            sql.push_str(" AND feed_id = ?");
            values.push(Value::from(id));
        }

        if let Some(term) = search {
            sql.push_str(" AND (title LIKE ? OR description LIKE ?)");

            let pattern = format!("%{}%", term);

            values.push(Value::from(pattern.clone()));
            values.push(Value::from(pattern));
        }

        sql.push_str(" ORDER BY published DESC, id DESC");

        let rows: Vec<Row> = conn.exec(sql, mysql::Params::Positional(values))?;

        let articles = rows
            .into_iter()
            .map(|row| {
                let fetched_at = row
                    .get::<NaiveDateTime, _>("fetched_at")
                    .unwrap_or_else(|| Utc::now().naive_utc());

                Article {
                    id: row.get("id").unwrap_or_default(),
                    feed_id: row.get("feed_id").unwrap_or_default(),
                    title: row.get("title").unwrap_or_default(),
                    author: row.get("author").unwrap_or(None),
                    link: row.get("link").unwrap_or(None),
                    published: row
                        .get::<Option<NaiveDateTime>, _>("published")
                        .flatten()
                        .map(datetime_from_mysql),
                    description: row.get("description").unwrap_or(None),
                    fetched_at: datetime_from_mysql(fetched_at),
                    is_read: row.get::<bool, _>("is_read").unwrap_or(false),
                    is_favorite: row.get::<bool, _>("is_favorite").unwrap_or(false),
                    guid: row.get("guid").unwrap_or_default(),
                }
            })
            .collect();

        Ok(articles)
    }

    // =========================================================
    // STATUS DOS ARTIGOS
    // =========================================================

    pub fn mark_read(&self, id: i64, read: bool) -> Result<()> {
        let mut conn = self.pool.get_conn()?;

        conn.exec_drop(
            "UPDATE articles SET is_read = :read WHERE id = :id",
            params! {
                "read" => read,
                "id" => id,
            },
        )?;

        if conn.affected_rows() == 0 {
            let exists: Option<i64> = conn.exec_first(
                "SELECT id FROM articles WHERE id = :id",
                params! { "id" => id },
            )?;

            if exists.is_none() {
                anyhow::bail!("Artigo não encontrado.");
            }
        }

        Ok(())
    }

    pub fn mark_favorite(&self, id: i64, favorite: bool) -> Result<()> {
        let mut conn = self.pool.get_conn()?;

        conn.exec_drop(
            "UPDATE articles SET is_favorite = :favorite WHERE id = :id",
            params! {
                "favorite" => favorite,
                "id" => id,
            },
        )?;

        if conn.affected_rows() == 0 {
            let exists: Option<i64> = conn.exec_first(
                "SELECT id FROM articles WHERE id = :id",
                params! { "id" => id },
            )?;

            if exists.is_none() {
                anyhow::bail!("Artigo não encontrado.");
            }
        }

        Ok(())
    }

    // =========================================================
    // CONTROLE DE ATUALIZAÇÃO DOS FEEDS
    // =========================================================

    pub fn update_feed_metadata(
        &self,
        id: i64,
        etag: Option<&str>,
        last_modified: Option<&str>,
    ) -> Result<()> {
        let mut conn = self.pool.get_conn()?;

        conn.exec_drop(
            "UPDATE feeds
            SET etag = COALESCE(:etag, etag),
                last_modified = COALESCE(:last_modified, last_modified),
                last_checked = NOW(),
                last_success = NOW(),
                failure_count = 0
            WHERE id = :id",
            params! {
                "id" => id,
                "etag" => etag,
                "last_modified" => last_modified,
            },
        )?;

        Ok(())
    }

    pub fn record_feed_failure(&self, id: i64) -> Result<()> {
        let mut conn = self.pool.get_conn()?;

        conn.exec_drop(
            "UPDATE feeds
             SET last_checked = NOW(),
                 failure_count = failure_count + 1
             WHERE id = :id",
            params! {
                "id" => id,
            },
        )?;

        Ok(())
    }
}

// =============================================================
// CONVERSÃO DE DATAS
// =============================================================

fn datetime_to_mysql(datetime: DateTime<Utc>) -> NaiveDateTime {
    datetime.naive_utc()
}

fn datetime_from_mysql(datetime: NaiveDateTime) -> DateTime<Utc> {
    DateTime::from_naive_utc_and_offset(datetime, Utc)
}
