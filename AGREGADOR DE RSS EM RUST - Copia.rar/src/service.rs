
use anyhow::Result;
use futures::{stream, StreamExt};

use crate::database::Database;
use crate::fetcher::{Fetcher, FetchResult};
use crate::models::Feed;

pub struct FeedService {
    db: Database,
    fetcher: Fetcher,
}

impl FeedService {
    pub fn new(db: Database, fetcher: Fetcher) -> Self {
        Self { db, fetcher }
    }

    pub fn add_feed(&self, title: &str, url: &str) -> Result<()> {
        self.db.add_feed(title, url)
    }

    pub fn list_feeds(&self) -> Result<Vec<Feed>> {
        self.db.list_feeds()
    }

    pub fn remove_feed(&self, id: i64) -> Result<()> {
        self.db.remove_feed(id)
    }

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

                    println!(
                        "[OK] {} - {} artigos novos",
                        parsed.title,
                        inserted
                    );
                }

                Ok(FetchResult::NotModified) => {
                    self.db.update_feed_metadata(
                        feed.id,
                        feed.etag.as_deref(),
                        feed.last_modified.as_deref(),
                    )?;

                    println!(
                        "[OK] {} - sem alterações",
                        feed.title
                    );
                }

                Err(error) => {
                    self.db.record_feed_failure(feed.id)?;

                    eprintln!(
                        "[ERRO] {} - {}",
                        feed.title,
                        error
                    );
                }
            }
        }

        Ok(())
    }

    pub async fn update_feed(&self, id: i64) -> Result<()> {
        let feeds = self.db.list_feeds()?;

        let feed = feeds
            .into_iter()
            .find(|feed| feed.id == id)
            .ok_or_else(|| {
                anyhow::anyhow!("Feed {} não encontrado", id)
            })?;

        match self.fetcher
            .fetch(
                &feed.url,
                feed.id,
                feed.etag.as_deref(),
                feed.last_modified.as_deref(),
            )
            .await?
        {
            FetchResult::Updated {
                feed: parsed,
                etag,
                last_modified,
            } => {
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

                println!(
                    "{} - {} artigos novos",
                    parsed.title,
                    inserted
                );
            }

            FetchResult::NotModified => {
                self.db.update_feed_metadata(
                    feed.id,
                    feed.etag.as_deref(),
                    feed.last_modified.as_deref(),
                )?;

                println!("Feed sem alterações");
            }
        }

        Ok(())
    }

    pub fn list_articles(
        &self,
        unread_only: bool,
        feed_id: Option<i64>,
        search: Option<&str>,
    ) -> Result<()> {
        let articles = self.db.list_articles(
            unread_only,
            feed_id,
            search,
        )?;

        for article in articles {
            let status = if article.is_read {
                "[LIDO]"
            } else {
                "[NOVO]"
            };

            let favorite = if article.is_favorite {
                " ★"
            } else {
                ""
            };

            println!(
                "{} {}{} - {}",
                status,
                article.id,
                favorite,
                article.title
            );

            if let Some(link) = article.link {
                println!("    {}", link);
            }

            println!();
        }

        Ok(())
    }

    pub fn mark_read(&self, id: i64, read: bool) -> Result<()> {
        self.db.mark_read(id, read)
    }

    pub fn mark_favorite(
        &self,
        id: i64,
        favorite: bool,
    ) -> Result<()> {
        self.db.mark_favorite(id, favorite)
    }
}
