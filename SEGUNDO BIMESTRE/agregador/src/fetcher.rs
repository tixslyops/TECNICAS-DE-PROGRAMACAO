use crate::models::{Artigo, Feed};
use chrono::Utc;
use feed_rs::parser;
use reqwest::header::{IF_MODIFIED_SINCE, IF_NONE_MATCH};
use reqwest::Client;
use sha2::{Digest, Sha256};
use std::time::Duration;

pub struct Fetcher {
    client: Client,
}

impl Fetcher {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_default();
        Fetcher { client }
    }

    pub async fn buscar_feed(&self, feed: &Feed) -> Result<(Vec<Artigo>, Option<String>, Option<String>), String> {
        let mut req = self.client.get(&feed.url);

        if let Some(ref etag) = feed.etag {
            req = req.header(IF_NONE_MATCH, etag);
        }
        if let Some(ref last_modified) = feed.last_modified {
            req = req.header(IF_MODIFIED_SINCE, last_modified);
        }

        let response = req.send().await.map_err(|e| format!("Erro de rede/HTTP: {}", e))?;

        if response.status() == reqwest::StatusCode::NOT_MODIFIED {
            return Ok((Vec::new(), feed.etag.clone(), feed.last_modified.clone()));
        }

        if !response.status().is_success() {
            return Err(format!("Status HTTP de erro: {}", response.status()));
        }

        let new_etag = response
            .headers()
            .get("ETag")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let new_last_modified = response
            .headers()
            .get("Last-Modified")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let content = response.bytes().await.map_err(|e| format!("Erro ao ler o corpo: {}", e))?;

        let parsed_feed = parser::parse(&content[..]).map_err(|e| format!("Erro ao processar RSS/Atom XML: {}", e))?;

        let mut artigos = Vec::new();
        for entry in parsed_feed.entries {
            let titulo = entry.title.map(|t| t.content).unwrap_or_else(|| "Sem título".to_string());
            let link = entry.links.first().map(|l| l.href.clone()).unwrap_or_default();
            let autor = entry.authors.first().map(|a| a.name.clone());
            let descricao = entry.summary.map(|s| s.content);

            let data_publicacao = entry.published.or(entry.updated).map(|dt| dt.into());

            let mut hasher = Sha256::new();
            hasher.update(format!("{}{}", link, titulo).as_bytes());
            let guid_hash = format!("{:x}", hasher.finalize());

            artigos.push(Artigo {
                id: 0,
                feed_id: feed.id,
                guid_hash,
                titulo,
                autor,
                link,
                descricao,
                data_publicacao,
                data_obtencao: Utc::now(),
                lido: false,
                favorito: false,
            });
        }

        Ok((artigos, new_etag, new_last_modified))
    }
}