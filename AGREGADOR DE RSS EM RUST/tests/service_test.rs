use agregador_rss::database::Database;
use agregador_rss::fetcher::Fetcher;
use agregador_rss::service::FeedService;
use chrono::Utc;
use mockito::Server;

#[tokio::test]
async fn deve_continuar_atualizando_quando_um_feed_falhar() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")?;
    let identificador = Utc::now().timestamp_nanos_opt().unwrap();

    // Cria um servidor HTTP falso para os testes.
    let mut server = Server::new_async().await;

    let caminho_valido = format!("/valido-{}", identificador);
    let caminho_invalido = format!("/invalido-{}", identificador);

    let xml = r#"
        <rss version="2.0">
            <channel>
                <title>Feed válido</title>
                <link>https://exemplo.com</link>
                <description>Feed utilizado no teste</description>
                <item>
                    <title>Artigo recebido com sucesso</title>
                    <guid>artigo-teste-unico</guid>
                    <description>Conteúdo do artigo</description>
                </item>
            </channel>
        </rss>
    "#;

    // Primeiro feed responde normalmente.
    let _mock_valido = server
        .mock("GET", caminho_valido.as_str())
        .with_status(200)
        .with_header("content-type", "application/rss+xml")
        .with_body(xml)
        .create_async()
        .await;

    // Segundo feed retorna erro HTTP 404.
    let _mock_invalido = server
        .mock("GET", caminho_invalido.as_str())
        .with_status(404)
        .create_async()
        .await;

    let url_valida = format!("{}{}", server.url(), caminho_valido);
    let url_invalida = format!("{}{}", server.url(), caminho_invalido);

    // Cadastra os dois feeds no banco.
    let db = Database::new(&database_url)?;

    db.add_feed("Feed válido de teste", &url_valida)?;
    db.add_feed("Feed inválido de teste", &url_invalida)?;

    let feeds = db.list_feeds()?;

    let feed_valido_id = feeds
        .iter()
        .find(|feed| feed.url == url_valida)
        .expect("Feed válido não encontrado")
        .id;

    let feed_invalido_id = feeds
        .iter()
        .find(|feed| feed.url == url_invalida)
        .expect("Feed inválido não encontrado")
        .id;

    // Executa a atualização de todos os feeds.
    let service = FeedService::new(db, Fetcher::new()?);

    // A atualização geral não deve falhar por causa do 404.
    service.update_all().await?;

    // Consulta o banco novamente para verificar os resultados.
    let db_verificacao = Database::new(&database_url)?;

    let artigos = db_verificacao.list_articles(false, Some(feed_valido_id), None)?;

    // O artigo do feed válido precisa ter sido salvo.
    assert_eq!(artigos.len(), 1);
    assert_eq!(artigos[0].title, "Artigo recebido com sucesso");

    // O feed inválido precisa ter registrado uma falha.
    let feeds_atualizados = db_verificacao.list_feeds()?;

    let feed_invalido = feeds_atualizados
        .iter()
        .find(|feed| feed.id == feed_invalido_id)
        .expect("Feed inválido não encontrado");

    assert_eq!(feed_invalido.failure_count, 1);

    // Limpa os registros criados pelo teste.
    db_verificacao.remove_feed(feed_valido_id)?;
    db_verificacao.remove_feed(feed_invalido_id)?;

    Ok(())
}
