
use anyhow::{Context, Result};
use feed_rs::parser;
use sha2::{Digest, Sha256};
use std::io::Cursor;

use crate::models::{NewArticle, ParsedFeed};

pub fn parse_feed(
    content: &[u8],
    feed_id: i64,
) -> Result<ParsedFeed> {
    let feed = parser::parse(Cursor::new(content))
        .context("Não foi possível interpretar o XML do feed")?;

    let feed_title = feed
        .title
        .map(|title| title.content)
        .unwrap_or_else(|| "Feed sem título".to_string());

    let mut articles = Vec::new();

    for entry in feed.entries {
        let title = entry
            .title
            .map(|title| title.content)
            .unwrap_or_else(|| "Artigo sem título".to_string());

        let author = entry.authors
            .first()
            .map(|person| person.name.clone());

        let link = entry.links
            .first()
            .map(|link| link.href.clone());

        let published = entry
            .published
            .or(entry.updated)
            .map(|date| date.with_timezone(&chrono::Utc));

        let description = entry
            .summary
            .map(|summary| summary.content)
            .or_else(|| {
                entry.content
                    .and_then(|content| content.body)
            });

        let guid = if !entry.id.trim().is_empty() {
            entry.id
        } else {
            generate_guid(
                link.as_deref(),
                &title,
                published.as_ref().map(|date| date.to_rfc3339()).as_deref(),
            )
        };

        articles.push(NewArticle {
            feed_id,
            title,
            author,
            link,
            published,
            description,
            guid,
        });
    }

    Ok(ParsedFeed {
        title: feed_title,
        articles,
    })
}

fn generate_guid(
    link: Option<&str>,
    title: &str,
    published: Option<&str>,
) -> String {
    let mut hasher = Sha256::new();

    hasher.update(link.unwrap_or(""));
    hasher.update(title);
    hasher.update(published.unwrap_or(""));

    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deve_interpretar_rss() {
        let xml = br#"
        <rss version="2.0">
            <channel>
                <title>Meu Feed</title>
                <item>
                    <title>Notícia de teste</title>
                    <link>https://example.com/noticia</link>
                    <guid>noticia-1</guid>
                    <description>Conteúdo da notícia</description>
                </item>
            </channel>
        </rss>
        "#;

        let result = parse_feed(xml, 1);

        assert!(result.is_ok());

        let parsed = result.unwrap();

        assert_eq!(parsed.title, "Meu Feed");
        assert_eq!(parsed.articles.len(), 1);
        assert_eq!(
            parsed.articles[0].title,
            "Notícia de teste"
        );
    }

    #[test]
    fn deve_rejeitar_xml_invalido() {
        let xml = b"<rss><channel>";

        assert!(parse_feed(xml, 1).is_err());
    }
}
