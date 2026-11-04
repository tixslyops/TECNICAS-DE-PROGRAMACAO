use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct Feed {
    pub id: i64,
    pub url: String,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub last_checked: Option<DateTime<Utc>>,
    pub failure_count: i32,
}

#[derive(Debug, Clone)]
pub struct Artigo {
    pub id: i64,
    pub feed_id: i64,
    pub guid_hash: String,
    pub titulo: String,
    pub autor: Option<String>,
    pub link: String,
    pub descricao: Option<String>,
    pub data_publicacao: Option<DateTime<Utc>>,
    pub data_obtencao: DateTime<Utc>,
    pub lido: bool,
    pub favorito: bool,
}