
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct Feed {
    pub id: i64,
    pub title: String,
    pub url: String,
    pub last_checked: Option<DateTime<Utc>>,
    pub last_success: Option<DateTime<Utc>>,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub failure_count: u32,
}

#[derive(Debug, Clone)]
pub struct Article {
    pub id: i64,
    pub feed_id: i64,
    pub title: String,
    pub author: Option<String>,
    pub link: Option<String>,
    pub published: Option<DateTime<Utc>>,
    pub description: Option<String>,
    pub fetched_at: DateTime<Utc>,
    pub is_read: bool,
    pub is_favorite: bool,
    pub guid: String,
}

#[derive(Debug, Clone)]
pub struct NewArticle {
    pub feed_id: i64,
    pub title: String,
    pub author: Option<String>,
    pub link: Option<String>,
    pub published: Option<DateTime<Utc>>,
    pub description: Option<String>,
    pub guid: String,
}

#[derive(Debug, Clone)]
pub struct ParsedFeed {
    pub title: String,
    pub articles: Vec<NewArticle>,
}
