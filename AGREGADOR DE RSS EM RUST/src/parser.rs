use crate::models::Article;
use chrono::Utc;
use feed_rs::parser;

pub fn parse_feed_content(content: &[u8], feed_id: i64) -> Result<Vec<Article>, Box<dyn std::error::Error>> {
    let feed = parser::parse(content)?;
    let mut articles = Vec::new();

    for entry in feed.entries {
        let title = entry.title.map(|t| t.content).unwrap_or_else(|| "Sem Título".to_string());
        let link = entry.links.first().map(|l| l.href.clone()).unwrap_or_default();
        let author = entry.authors.first().map(|a| a.name.clone());
        let published_at = entry.published.or(entry.updated);
        let description = entry.summary.map(|s| s.content).or_else(|| entry.content.and_then(|c| c.body));
        
        // Estratégia de Deduplicação baseada no GUID ou Link único do artigo
        let guid = if !entry.id.is_empty() {
            entry.id.clone()
        } else {
            link.clone()
        };

        if guid.is_empty() {
            continue;
        }

        articles.push(Article {
            id: None,
            feed_id,
            title,
            author,
            link,
            published_at,
            description,
            is_read: false,
            is_favorite: false,
            fetched_at: Utc::now(),
            guid,
        });
    }

    Ok(articles)
}
