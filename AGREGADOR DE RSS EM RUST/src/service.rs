use anyhow::{anyhow, Context, Result};
use futures::{stream, StreamExt};
use url::Url;

use crate::database::Database;
use crate::fetcher::{FetchResult, Fetcher};
use crate::models::Feed;

pub struct FeedService {
    db: Database,
    fetcher: Fetcher,
}

impl FeedService {
    pub fn new(db: Database, fetcher: Fetcher) -> Self {
        Self { db, fetcher }
    }

    // =========================================================
    // GERENCIAMENTO DE FEEDS
    // =========================================================

    pub fn add_feed(&self, title: &str, url: &str) -> Result<()> {
        let parsed_url = Url::parse(url).context("URL do feed inválida")?;

        if !matches!(parsed_url.scheme(), "http" | "https") {
            anyhow::bail!("A URL deve utilizar o protocolo HTTP ou HTTPS.");
        }

        self.db.add_feed(title, url)
    }

    pub fn list_feeds(&self) -> Result<Vec<Feed>> {
        self.db.list_feeds()
    }

    pub fn remove_feed(&self, id: i64) -> Result<()> {
        self.db.remove_feed(id)
    }

    // =========================================================
    // ATUALIZAR TODOS OS FEEDS
    // =========================================================

    pub async fn update_all(&self) -> Result<()> {
        let feeds = self.db.list_feeds()?;
        let fetcher = &self.fetcher;

        let results = stream::iter(feeds.into_iter())
            .map(|feed| async move {
                let result = fetcher
                    .fetch(
                        &feed.url,
                        feed.id,
                        feed.etag.as_deref(),
                        feed.last_modified.as_deref(),
                    )
                    .await;

                (feed, result)
            })
            .buffer_unordered(5)
            .collect::<Vec<_>>()
            .await;

        for (feed, result) in results {
            match result {
                Ok(FetchResult::Updated {
                    feed: parsed,
                    etag,
                    last_modified,
                }) => {
                    let mut inserted = 0;

                    for article in &parsed.articles {
                        if self.db.insert_article(article)? {
                            inserted += 1;
                        }
                    }

                    self.db.update_feed_metadata(
                        feed.id,
                        etag.as_deref(),
                        last_modified.as_deref(),
                    )?;

                    println!("[OK] {} - {} artigos novos", parsed.title, inserted);
                }

                Ok(FetchResult::NotModified) => {
                    self.db.update_feed_metadata(
                        feed.id,
                        feed.etag.as_deref(),
                        feed.last_modified.as_deref(),
                    )?;

                    println!("[OK] {} - sem alterações", feed.title);
                }

                Err(error) => {
                    self.db.record_feed_failure(feed.id)?;

                    eprintln!("[ERRO] {} - {}", feed.title, error);
                }
            }
        }

        Ok(())
    }

    // =========================================================
    // ATUALIZAR UM FEED ESPECÍFICO
    // =========================================================

    pub async fn update_feed(&self, id: i64) -> Result<()> {
        let feeds = self.db.list_feeds()?;

        let feed = feeds
            .into_iter()
            .find(|feed| feed.id == id)
            .ok_or_else(|| anyhow!("Feed {} não encontrado", id))?;

        let result = self
            .fetcher
            .fetch(
                &feed.url,
                feed.id,
                feed.etag.as_deref(),
                feed.last_modified.as_deref(),
            )
            .await;

        match result {
            Ok(FetchResult::Updated {
                feed: parsed,
                etag,
                last_modified,
            }) => {
                let mut inserted = 0;

                for article in &parsed.articles {
                    if self.db.insert_article(article)? {
                        inserted += 1;
                    }
                }

                self.db
                    .update_feed_metadata(feed.id, etag.as_deref(), last_modified.as_deref())?;

                println!("[OK] {} - {} artigos novos", parsed.title, inserted);
            }

            Ok(FetchResult::NotModified) => {
                self.db.update_feed_metadata(
                    feed.id,
                    feed.etag.as_deref(),
                    feed.last_modified.as_deref(),
                )?;

                println!("[OK] {} - sem alterações", feed.title);
            }

            Err(error) => {
                self.db.record_feed_failure(feed.id)?;

                return Err(anyhow!(
                    "Falha ao atualizar o feed '{}': {}",
                    feed.title,
                    error
                ));
            }
        }

        Ok(())
    }

    // =========================================================
    // LISTAGEM DE ARTIGOS
    // =========================================================

    pub fn list_articles(
        &self,
        unread_only: bool,
        feed_id: Option<i64>,
        search: Option<&str>,
    ) -> Result<()> {
        let articles = self.db.list_articles(unread_only, feed_id, search)?;

        if articles.is_empty() {
            println!("Nenhum artigo encontrado.");
            return Ok(());
        }

        for article in articles {
            let status = if article.is_read { "[LIDO]" } else { "[NOVO]" };

            let favorite = if article.is_favorite {
                " [FAVORITO]"
            } else {
                ""
            };

            println!("{} {}{} - {}", status, article.id, favorite, article.title);

            if let Some(author) = article.author {
                println!("    Autor: {}", author);
            }

            if let Some(link) = article.link {
                println!("    Link: {}", link);
            }

            if let Some(description) = article.description {
                println!("    {}", description);
            }

            println!();
        }

        Ok(())
    }

    // =========================================================
    // MARCAR COMO LIDO / NÃO LIDO
    // =========================================================

    pub fn mark_read(&self, id: i64, read: bool) -> Result<()> {
        self.db.mark_read(id, read)
    }

    // =========================================================
    // FAVORITAR / REMOVER FAVORITO
    // =========================================================

    pub fn mark_favorite(&self, id: i64, favorite: bool) -> Result<()> {
        self.db.mark_favorite(id, favorite)
    }
}
