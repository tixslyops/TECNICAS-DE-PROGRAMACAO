use rusqlite::{params, Connection, Result};
use crate::models::{Feed, Article};
use chrono::{DateTime, Utc};

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS feeds (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                url TEXT UNIQUE NOT NULL,
                title TEXT NOT NULL,
                last_checked TEXT,
                etag TEXT,
                last_modified TEXT,
                failure_count INTEGER DEFAULT 0
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS articles (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                feed_id INTEGER,
                title TEXT,
                author TEXT,
                link TEXT,
                published_at TEXT,
                description TEXT,
                is_read BOOLEAN,
                is_favorite BOOLEAN,
                fetched_at TEXT,
                guid TEXT UNIQUE,
                FOREIGN KEY(feed_id) REFERENCES feeds(id) ON DELETE CASCADE
            )",
            [],
        )?;

        Ok(Database { conn })
    }

    pub fn add_feed(&self, url: &str, title: &str) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO feeds (url, title, failure_count) VALUES (?, ?, 0)",
            params![url, title],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_feeds(&self) -> Result<Vec<Feed>> {
        let mut stmt = self.conn.prepare("SELECT id, url, title, last_checked, etag, last_modified, failure_count FROM feeds")?;
        let feed_iter = stmt.query_map([], |row| {
            let last_checked_str: Option<String> = row.get(3)?;
            let last_checked = last_checked_str.and_then(|s| s.parse::<DateTime<Utc>>().ok());
            Ok(Feed {
                id: Some(row.get(0)?),
                url: row.get(1)?,
                title: row.get(2)?,
                last_checked,
                etag: row.get(4)?,
                last_modified: row.get(5)?,
                failure_count: row.get(6)?,
            })
        })?;

        let mut feeds = Vec::new();
        for feed in feed_iter {
            feeds.push(feed?);
        }
        Ok(feeds)
    }

    pub fn update_feed_meta(&self, feed_id: i64, etag: Option<&str>, last_modified: Option<&str>, success: bool) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        if success {
            self.conn.execute(
                "UPDATE feeds SET last_checked = ?, etag = ?, last_modified = ?, failure_count = 0 WHERE id = ?",
                params![now, etag, last_modified, feed_id],
            )?;
        } else {
            self.conn.execute(
                "UPDATE feeds SET last_checked = ?, failure_count = failure_count + 1 WHERE id = ?",
                params![now, feed_id],
            )?;
        }
        Ok(())
    }

    pub fn insert_article(&self, article: &Article) -> Result<bool> {
        let count = self.conn.execute(
            "INSERT OR IGNORE INTO articles (feed_id, title, author, link, published_at, description, is_read, is_favorite, fetched_at, guid)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                article.feed_id,
                article.title,
                article.author,
                article.link,
                article.published_at.map(|dt| dt.to_rfc3339()),
                article.description,
                article.is_read,
                article.is_favorite,
                article.fetched_at.to_rfc3339(),
                article.guid
            ],
        )?;
        Ok(count > 0)
    }

    pub fn get_articles(&self, unread_only: bool, search: Option<&str>) -> Result<Vec<Article>> {
        let mut query = String::from("SELECT id, feed_id, title, author, link, published_at, description, is_read, is_favorite, fetched_at, guid FROM articles WHERE 1=1");
        
        if unread_only {
            query.push_str(" AND is_read = 0");
        }

        if search.is_some() {
            query.push_str(" AND (title LIKE ?1 OR description LIKE ?1)");
        }

        query.push_str(" ORDER BY datetime(fetched_at) DESC");

        let mut stmt = self.conn.prepare(&query)?;
        
        let mut articles = Vec::new();
        
        if let Some(term) = search {
            let pattern = format!("%{}%", term);
            let rows = stmt.query_map(params![pattern], |row| Self::map_article(row))?;
            for article in rows {
                articles.push(article?);
            }
        } else {
            let rows = stmt.query_map([], |row| Self::map_article(row))?;
            for article in rows {
                articles.push(article?);
            }
        }

        Ok(articles)
    }

    fn map_article(row: &rusqlite::Row) -> Result<Article> {
        let pub_str: Option<String> = row.get(5)?;
        let fetched_str: String = row.get(9)?;
        Ok(Article {
            id: Some(row.get(0)?),
            feed_id: row.get(1)?,
            title: row.get(2)?,
            author: row.get(3)?,
            link: row.get(4)?,
            published_at: pub_str.and_then(|s| s.parse::<DateTime<Utc>>().ok()),
            description: row.get(6)?,
            is_read: row.get(7)?,
            is_favorite: row.get(8)?,
            fetched_at: fetched_str.parse::<DateTime<Utc>>().unwrap_or_else(|_| Utc::now()),
            guid: row.get(10)?,
        })
    }

    pub fn mark_as_read(&self, article_id: i64, read: bool) -> Result<()> {
        self.conn.execute("UPDATE articles SET is_read = ? WHERE id = ?", params![read, article_id])?;
        Ok(())
    }

    pub fn toggle_favorite(&self, article_id: i64) -> Result<()> {
        self.conn.execute("UPDATE articles SET is_favorite = NOT is_favorite WHERE id = ?", params![article_id])?;
        Ok(())
    }
}