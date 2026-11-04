
use anyhow::Result;
use rusqlite::{params, Connection};
use crate::models::{Article, Feed, NewArticle};
use chrono::{DateTime, Utc};

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;

        conn.execute_batch(
            "
            PRAGMA foreign_keys = ON;

            CREATE TABLE IF NOT EXISTS feeds (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                url TEXT NOT NULL UNIQUE,
                last_checked TEXT,
                last_success TEXT,
                etag TEXT,
                last_modified TEXT,
                failure_count INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS articles (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                feed_id INTEGER NOT NULL,
                title TEXT NOT NULL,
                author TEXT,
                link TEXT,
                published TEXT,
                description TEXT,
                fetched_at TEXT NOT NULL,
                is_read INTEGER NOT NULL DEFAULT 0,
                is_favorite INTEGER NOT NULL DEFAULT 0,
                guid TEXT NOT NULL,
                FOREIGN KEY(feed_id) REFERENCES feeds(id)
                    ON DELETE CASCADE,
                UNIQUE(feed_id, guid)
            );

            CREATE INDEX IF NOT EXISTS idx_articles_feed
            ON articles(feed_id);

            CREATE INDEX IF NOT EXISTS idx_articles_read
            ON articles(is_read);
            "
        )?;

        Ok(Self { conn })
    }

    pub fn add_feed(&self, title: &str, url: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO feeds (title, url) VALUES (?1, ?2)",
            params![title, url],
        )?;

        Ok(())
    }

    pub fn list_feeds(&self) -> Result<Vec<Feed>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, url, last_checked,
                    last_success, etag, last_modified,
                    failure_count
             FROM feeds
             ORDER BY id"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(Feed {
                id: row.get(0)?,
                title: row.get(1)?,
                url: row.get(2)?,
                last_checked: parse_datetime(row.get(3)?),
                last_success: parse_datetime(row.get(4)?),
                etag: row.get(5)?,
                last_modified: row.get(6)?,
                failure_count: row.get(7)?,
            })
        })?;

        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn remove_feed(&self, id: i64) -> Result<()> {
        self.conn.execute(
            "DELETE FROM feeds WHERE id = ?1",
            params![id],
        )?;

        Ok(())
    }

    pub fn insert_article(&self, article: &NewArticle) -> Result<bool> {
        let result = self.conn.execute(
            "INSERT OR IGNORE INTO articles
            (feed_id, title, author, link, published,
             description, fetched_at, guid)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                article.feed_id,
                article.title,
                article.author,
                article.link,
                article.published.map(|d| d.to_rfc3339()),
                article.description,
                Utc::now().to_rfc3339(),
                article.guid,
            ],
        )?;

        Ok(result > 0)
    }

    pub fn list_articles(
        &self,
        unread_only: bool,
        feed_id: Option<i64>,
        search: Option<&str>,
    ) -> Result<Vec<Article>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, feed_id, title, author, link,
                    published, description, fetched_at,
                    is_read, is_favorite, guid
             FROM articles
             WHERE (?1 = 0 OR is_read = 0)
               AND (?2 IS NULL OR feed_id = ?2)
               AND (?3 IS NULL OR
                    title LIKE '%' || ?3 || '%' OR
                    description LIKE '%' || ?3 || '%')
             ORDER BY published DESC, id DESC"
        )?;

        let rows = stmt.query_map(
            params![
                unread_only as i32,
                feed_id,
                search
            ],
            |row| {
                Ok(Article {
                    id: row.get(0)?,
                    feed_id: row.get(1)?,
                    title: row.get(2)?,
                    author: row.get(3)?,
                    link: row.get(4)?,
                    published: parse_datetime(row.get(5)?),
                    description: row.get(6)?,
                    fetched_at: parse_datetime(row.get(7)?)
                        .unwrap_or_else(|| Some(Utc::now()))
                        .unwrap(),
                    is_read: row.get::<_, i32>(8)? != 0,
                    is_favorite: row.get::<_, i32>(9)? != 0,
                    guid: row.get(10)?,
                })
            },
        )?;

        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn mark_read(&self, id: i64, read: bool) -> Result<()> {
        self.conn.execute(
            "UPDATE articles SET is_read = ?1 WHERE id = ?2",
            params![read as i32, id],
        )?;

        Ok(())
    }

    pub fn mark_favorite(&self, id: i64, favorite: bool) -> Result<()> {
        self.conn.execute(
            "UPDATE articles SET is_favorite = ?1 WHERE id = ?2",
            params![favorite as i32, id],
        )?;

        Ok(())
    }

    pub fn update_feed_metadata(
        &self,
        id: i64,
        etag: Option<&str>,
        last_modified: Option<&str>,
    ) -> Result<()> {
        self.conn.execute(
            "UPDATE feeds
             SET etag = ?1, last_modified = ?2,
                 last_checked = ?3, last_success = ?3,
                 failure_count = 0
             WHERE id = ?4",
            params![
                etag,
                last_modified,
                Utc::now().to_rfc3339(),
                id
            ],
        )?;

        Ok(())
    }

    pub fn record_feed_failure(&self, id: i64) -> Result<()> {
        self.conn.execute(
            "UPDATE feeds
             SET last_checked = ?1,
                 failure_count = failure_count + 1
             WHERE id = ?2",
            params![Utc::now().to_rfc3339(), id],
        )?;

        Ok(())
    }
}

fn parse_datetime(
    value: Option<String>,
) -> Option<DateTime<Utc>> {
    value.and_then(|text| {
        DateTime::parse_from_rfc3339(&text)
            .ok()
            .map(|date| date.with_timezone(&Utc))
    })
}
