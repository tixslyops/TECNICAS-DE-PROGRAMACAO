use anyhow::{Context, Result};
use feed_rs::parser;
use sha2::{Digest, Sha256};
use std::io::Cursor;

use crate::models::{NewArticle, ParsedFeed};

pub fn parse_feed(content: &[u8], feed_id: i64) -> Result<ParsedFeed> {
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

        let author = entry.authors.first().map(|person| person.name.clone());

        let link = entry.links.first().map(|link| link.href.clone());

        let published = entry
            .published
            .or(entry.updated)
            .map(|date| date.with_timezone(&chrono::Utc));

        let description = entry
            .summary
            .map(|summary| summary.content)
            .or_else(|| entry.content.and_then(|content| content.body));

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

fn generate_guid(link: Option<&str>, title: &str, published: Option<&str>) -> String {
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
        let xml = r#"
            <rss version="2.0">
                <channel>
                    <title>Notícia de teste</title>
                    <link>https://exemplo.com</link>
                    <description>Feed de teste</description>
                    <item>
                        <title>Conteúdo da notícia</title>
                        <description>Descrição do artigo</description>
                        <guid>artigo-1</guid>
                    </item>
                </channel>
            </rss>
        "#;

        let feed = parse_feed(xml.as_bytes(), 1).unwrap();

        assert_eq!(feed.title, "Notícia de teste");
        assert_eq!(feed.articles.len(), 1);
        assert_eq!(feed.articles[0].title, "Conteúdo da notícia");
        assert_eq!(feed.articles[0].guid, "artigo-1");
        assert_eq!(
            feed.articles[0].description.as_deref(),
            Some("Descrição do artigo")
        );
    }

    #[test]
    fn deve_interpretar_atom() {
        let xml = r#"
            <?xml version="1.0" encoding="utf-8"?>
            <feed xmlns="http://www.w3.org/2005/Atom">
                <title>Feed Atom de teste</title>
                <id>https://exemplo.com/feed</id>
                <updated>2026-11-03T10:00:00Z</updated>
                <entry>
                    <title>Artigo Atom</title>
                    <id>https://exemplo.com/artigo-1</id>
                    <updated>2026-11-03T09:00:00Z</updated>
                    <link href="https://exemplo.com/artigo-1" />
                    <summary>Resumo do artigo Atom</summary>
                    <author>
                        <name>Autor de teste</name>
                    </author>
                </entry>
            </feed>
        "#;

        let feed = parse_feed(xml.as_bytes(), 2).unwrap();

        assert_eq!(feed.title, "Feed Atom de teste");
        assert_eq!(feed.articles.len(), 1);

        let article = &feed.articles[0];

        assert_eq!(article.title, "Artigo Atom");
        assert_eq!(article.guid, "https://exemplo.com/artigo-1");
        assert_eq!(
            article.link.as_deref(),
            Some("https://exemplo.com/artigo-1")
        );
        assert_eq!(article.author.as_deref(), Some("Autor de teste"));
        assert_eq!(
            article.description.as_deref(),
            Some("Resumo do artigo Atom")
        );
    }

    #[test]
    fn deve_aceitar_feed_vazio() {
        let xml = r#"
            <rss version="2.0">
                <channel>
                    <title>Feed sem artigos</title>
                    <link>https://exemplo.com</link>
                    <description>Este feed ainda não possui artigos.</description>
                </channel>
            </rss>
        "#;

        let feed = parse_feed(xml.as_bytes(), 3).unwrap();

        assert_eq!(feed.title, "Feed sem artigos");
        assert!(feed.articles.is_empty());
    }

    #[test]
    fn deve_tratar_campos_ausentes() {
        let xml = r#"
            <rss version="2.0">
                <channel>
                    <description>Feed sem título</description>
                    <item>
                        <description>Artigo sem título, autor ou link</description>
                    </item>
                </channel>
            </rss>
        "#;

        let feed = parse_feed(xml.as_bytes(), 4).unwrap();

        assert_eq!(feed.title, "Feed sem título");
        assert_eq!(feed.articles.len(), 1);

        let article = &feed.articles[0];

        assert_eq!(article.title, "Artigo sem título");
        assert!(article.author.is_none());
        assert!(article.link.is_none());
        assert_eq!(
            article.description.as_deref(),
            Some("Artigo sem título, autor ou link")
        );

        // Mesmo sem GUID, o parser deve gerar um identificador.
        assert!(!article.guid.is_empty());
    }

    #[test]
    fn deve_rejeitar_xml_invalido() {
        let xml = b"<rss><channel>";

        assert!(parse_feed(xml, 1).is_err());
    }
}
