use agregador_rss::fetcher::{FetchResult, Fetcher};
use mockito::Server;

const RSS_VALIDO: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
    <channel>
        <title>Feed de teste</title>
        <link>https://exemplo.com</link>
        <description>Feed usado nos testes</description>
        <item>
            <title>Artigo de teste</title>
            <link>https://exemplo.com/artigo</link>
            <guid>artigo-1</guid>
            <description>Conteúdo de teste</description>
        </item>
    </channel>
</rss>"#;

#[tokio::test]
async fn deve_buscar_feed_com_sucesso() -> anyhow::Result<()> {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("GET", "/feed")
        .with_status(200)
        .with_header("content-type", "application/rss+xml")
        .with_header("etag", "\"versao-1\"")
        .with_header("last-modified", "Wed, 21 Oct 2015 07:28:00 GMT")
        .with_body(RSS_VALIDO)
        .create_async()
        .await;

    let fetcher = Fetcher::new()?;

    let resultado = fetcher
        .fetch(&format!("{}/feed", server.url()), 1, None, None)
        .await?;

    match resultado {
        FetchResult::Updated {
            feed,
            etag,
            last_modified,
        } => {
            assert_eq!(feed.title, "Feed de teste");
            assert_eq!(feed.articles.len(), 1);
            assert_eq!(feed.articles[0].title, "Artigo de teste");

            assert_eq!(etag.as_deref(), Some("\"versao-1\""));
            assert_eq!(
                last_modified.as_deref(),
                Some("Wed, 21 Oct 2015 07:28:00 GMT")
            );
        }
        FetchResult::NotModified => {
            panic!("Esperava um feed atualizado");
        }
    }

    mock.assert_async().await;

    Ok(())
}

#[tokio::test]
async fn deve_rejeitar_erro_http_404() -> anyhow::Result<()> {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("GET", "/feed")
        .with_status(404)
        .create_async()
        .await;

    let fetcher = Fetcher::new()?;

    let resultado = fetcher
        .fetch(&format!("{}/feed", server.url()), 1, None, None)
        .await;

    assert!(resultado.is_err());

    mock.assert_async().await;

    Ok(())
}

#[tokio::test]
async fn deve_rejeitar_erro_http_500() -> anyhow::Result<()> {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("GET", "/feed")
        .with_status(500)
        .create_async()
        .await;

    let fetcher = Fetcher::new()?;

    let resultado = fetcher
        .fetch(&format!("{}/feed", server.url()), 1, None, None)
        .await;

    assert!(resultado.is_err());

    mock.assert_async().await;

    Ok(())
}

#[tokio::test]
async fn deve_reconhecer_feed_nao_modificado() -> anyhow::Result<()> {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("GET", "/feed")
        .match_header("if-none-match", "\"versao-1\"")
        .with_status(304)
        .create_async()
        .await;

    let fetcher = Fetcher::new()?;

    let resultado = fetcher
        .fetch(
            &format!("{}/feed", server.url()),
            1,
            Some("\"versao-1\""),
            None,
        )
        .await?;

    assert!(matches!(resultado, FetchResult::NotModified));

    mock.assert_async().await;

    Ok(())
}

#[tokio::test]
async fn deve_enviar_cabecalhos_condicionais() -> anyhow::Result<()> {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("GET", "/feed")
        .match_header("if-none-match", "\"versao-1\"")
        .match_header("if-modified-since", "Wed, 21 Oct 2015 07:28:00 GMT")
        .with_status(304)
        .create_async()
        .await;

    let fetcher = Fetcher::new()?;

    let resultado = fetcher
        .fetch(
            &format!("{}/feed", server.url()),
            1,
            Some("\"versao-1\""),
            Some("Wed, 21 Oct 2015 07:28:00 GMT"),
        )
        .await?;

    assert!(matches!(resultado, FetchResult::NotModified));

    mock.assert_async().await;

    Ok(())
}

#[tokio::test]
async fn deve_rejeitar_xml_invalido() -> anyhow::Result<()> {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("GET", "/feed")
        .with_status(200)
        .with_body("<xml>conteúdo inválido")
        .create_async()
        .await;

    let fetcher = Fetcher::new()?;

    let resultado = fetcher
        .fetch(&format!("{}/feed", server.url()), 1, None, None)
        .await;

    assert!(resultado.is_err());

    mock.assert_async().await;

    Ok(())
}
