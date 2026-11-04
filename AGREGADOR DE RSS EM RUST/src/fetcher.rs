use reqwest::{Client, StatusCode};
use std::error::Error;

pub async fn fetch_feed_data(
    client: &Client,
    url: &str,
    etag: Option<&str>,
    last_modified: Option<&str>,
) -> Result<(Option<Vec<u8>>, Option<String>, Option<String>, bool), Box<dyn Error + Send + Sync>> {
    let mut req = client.get(url)
        .header("User-Agent", "RustRSSAggregator/1.0");

    if let Some(e) = etag {
        req = req.header("If-None-Match", e);
    }
    if let Some(lm) = last_modified {
        req = req.header("If-Modified-Since", lm);
    }

    let resp = req.send().await?;

    if resp.status() == StatusCode::NOT_MODIFIED {
        return Ok((None, None, None, true)); // 304 Not Modified
    }

    if !resp.status().is_success() {
        return Err(format!("Erro HTTP: {}", resp.status()).into());
    }

    let new_etag = resp.headers().get("etag").and_then(|h| h.to_str().ok()).map(String::from);
    let new_lm = resp.headers().get("last-modified").and_then(|h| h.to_str().ok()).map(String::from);
    
    let bytes = resp.bytes().await?.to_vec();

    Ok((Some(bytes), new_etag, new_lm, false))
}
