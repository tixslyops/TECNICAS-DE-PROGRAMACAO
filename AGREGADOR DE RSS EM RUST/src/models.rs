use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Feed {
    pub id: Option<i64>,
    pub url: String,
    pub title: String,
    pub last_checked: Option<DateTime<Utc>>,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub failure_count: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Article {
    pub id: Option<i64>,
    pub feed_id: i64,
    pub title: String,
    pub author: Option<String>,
    pub link: String,
    pub published_at: Option<DateTime<Utc>>,
    pub description: Option<String>,
    pub is_read: bool,
    pub is_favorite: bool,
    pub fetched_at: DateTime<Utc>,
    pub guid: String,
}
