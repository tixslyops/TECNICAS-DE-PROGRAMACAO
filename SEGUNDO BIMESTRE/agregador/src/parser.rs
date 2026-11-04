use crate::models::Article;
use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use std::io::BufReader;

pub struct ParsedFeed {
    pub title: Option<String>,
    pub articles: Vec<Article>,
}

pub fn parse_feed_content(feed_id: i64, xml_content: &[u8]) -> anyhow::Result<ParsedFeed> {
    let reader = BufReader::new(xml_content);
    let feed = feed_rs::parser::parse(reader)?;

    let mut articles = Vec::new();

    for entry in feed.entries {
        let title = entry
            .title
            .map(|t| t.content)
            .unwrap_or_else(|| "Sem Título".to_string());

        let link = entry
            .links
            .first()
            .map(|l| l.href.clone())
            .unwrap_or_default();

        let author = entry
            .authors
            .first()
            .map(|a| a.name.clone());

        let description = entry
            .summary
            .map(|s| s.content)
            .or_else(|| entry.content.and_then(|c| c.body));

        let pub_date = entry.published.or(entry.updated).map(|dt| dt.with_timezone(&Utc));

        let raw_guid = entry
            .id
            .clone();
        let unique_str = if !raw_guid.is_empty() {
            raw_guid
        } else if !link.is_empty() {
            link.clone()
        } else {
            format!("{}{:?}", title, pub_date)
        };

        let mut hasher = Sha256::new();
        hasher.update(unique_str.as_bytes());
        let guid_hash = format!("{:x}", hasher.finalize());

        articles.push(Article {
            id: 0,
            feed_id,
            guid_hash,
            title,
            author,
            link,
            pub_date,
            description,
            is_read: false,
            is_favorite: false,
            fetched_at: Utc::now(),
        });
    }

    Ok(ParsedFeed {
        title: feed.title.map(|t| t.content),
        articles,
    })
}