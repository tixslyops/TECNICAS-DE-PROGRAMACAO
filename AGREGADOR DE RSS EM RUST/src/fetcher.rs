use anyhow::{Context, Result};
use reqwest::{
    header::{ETAG, IF_MODIFIED_SINCE, IF_NONE_MATCH, LAST_MODIFIED},
    Client, StatusCode,
};
use std::time::Duration;
use url::Url;

use crate::models::ParsedFeed;
use crate::parser::parse_feed;

pub enum FetchResult {
    Updated {
        feed: ParsedFeed,
        etag: Option<String>,
        last_modified: Option<String>,
    },
    NotModified,
}

pub struct Fetcher {
    client: Client,
}

impl Fetcher {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(20))
            .user_agent("AgregadorRSS/1.0")
            .build()?;

        Ok(Self { client })
    }

    pub async fn fetch(
        &self,
        url: &str,
        feed_id: i64,
        etag: Option<&str>,
        last_modified: Option<&str>,
    ) -> Result<FetchResult> {
        // Valida a URL antes de realizar a requisição.
        let parsed_url = Url::parse(url).context("URL do feed inválida")?;

        if !matches!(parsed_url.scheme(), "http" | "https") {
            anyhow::bail!("A URL deve utilizar o protocolo HTTP ou HTTPS.");
        }

        let mut request = self.client.get(url);

        if let Some(etag_value) = etag {
            request = request.header(IF_NONE_MATCH, etag_value);
        }

        if let Some(modified_value) = last_modified {
            request = request.header(IF_MODIFIED_SINCE, modified_value);
        }

        let response = request
            .send()
            .await
            .context("Falha ao realizar requisição HTTP")?;

        if response.status() == StatusCode::NOT_MODIFIED {
            return Ok(FetchResult::NotModified);
        }

        let response = response
            .error_for_status()
            .context("Servidor retornou erro HTTP")?;

        let etag = response
            .headers()
            .get(ETAG)
            .and_then(|value| value.to_str().ok())
            .map(String::from);

        let last_modified = response
            .headers()
            .get(LAST_MODIFIED)
            .and_then(|value| value.to_str().ok())
            .map(String::from);

        let content = response
            .bytes()
            .await
            .context("Falha ao ler conteúdo do feed")?;

        let feed = parse_feed(&content, feed_id)
            .context("O conteúdo recebido não é um RSS/Atom válido")?;

        Ok(FetchResult::Updated {
            feed,
            etag,
            last_modified,
        })
    }
}
