use agregador_rss::database::Database;
use agregador_rss::models::NewArticle;
use chrono::Utc;

#[test]
fn testa_persistencia_filtros_e_regras_dos_artigos() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")?;
    let db = Database::new(&database_url)?;

    let identificador = Utc::now().timestamp_nanos_opt().unwrap();

    // Cria dois feeds para testar o filtro por feed.
    let url1 = format!("https://teste.invalid/feed/{}", identificador);
    let url2 = format!("https://teste.invalid/feed/{}-2", identificador);

    db.add_feed("Feed de teste 1", &url1)?;
    db.add_feed("Feed de teste 2", &url2)?;

    let feeds = db.list_feeds()?;

    let feed1_id = feeds
        .iter()
        .find(|feed| feed.url == url1)
        .expect("Primeiro feed não encontrado")
        .id;

    let feed2_id = feeds
        .iter()
        .find(|feed| feed.url == url2)
        .expect("Segundo feed não encontrado")
        .id;

    // Cria três artigos, dois no primeiro feed e um no segundo.
    let artigo1 = NewArticle {
        feed_id: feed1_id,
        title: "Notícia sobre tecnologia".to_string(),
        author: Some("Autor 1".to_string()),
        link: Some("https://teste.invalid/artigo1".to_string()),
        published: None,
        description: Some("Conteúdo sobre programação Rust".to_string()),
        guid: format!("guid-1-{}", identificador),
    };

    let artigo2 = NewArticle {
        feed_id: feed1_id,
        title: "Notícia sobre esportes".to_string(),
        author: Some("Autor 2".to_string()),
        link: Some("https://teste.invalid/artigo2".to_string()),
        published: None,
        description: Some("Resultado do campeonato".to_string()),
        guid: format!("guid-2-{}", identificador),
    };

    let artigo3 = NewArticle {
        feed_id: feed2_id,
        title: "Notícia sobre programação".to_string(),
        author: Some("Autor 3".to_string()),
        link: Some("https://teste.invalid/artigo3".to_string()),
        published: None,
        description: Some("Desenvolvimento de software".to_string()),
        guid: format!("guid-3-{}", identificador),
    };

    // Testa inserção e deduplicação.
    assert!(db.insert_article(&artigo1)?);
    assert!(!db.insert_article(&artigo1)?);

    assert!(db.insert_article(&artigo2)?);
    assert!(db.insert_article(&artigo3)?);

    // Testa listagem geral.
    let todos = db.list_articles(false, None, None)?;
    assert!(todos.iter().any(|a| a.guid == artigo1.guid));
    assert!(todos.iter().any(|a| a.guid == artigo2.guid));
    assert!(todos.iter().any(|a| a.guid == artigo3.guid));

    // Testa filtro por feed.
    let artigos_feed1 = db.list_articles(false, Some(feed1_id), None)?;

    assert_eq!(artigos_feed1.len(), 2);
    assert!(artigos_feed1.iter().all(|a| a.feed_id == feed1_id));

    // Testa pesquisa pelo título.
    let pesquisa_titulo = db.list_articles(false, None, Some("tecnologia"))?;

    assert!(pesquisa_titulo.iter().any(|a| a.guid == artigo1.guid));

    // Testa pesquisa pela descrição.
    let pesquisa_conteudo = db.list_articles(false, None, Some("Rust"))?;

    assert!(pesquisa_conteudo.iter().any(|a| a.guid == artigo1.guid));

    // Testa combinação de pesquisa com filtro por feed.
    let pesquisa_filtrada = db.list_articles(false, Some(feed1_id), Some("esportes"))?;

    assert_eq!(pesquisa_filtrada.len(), 1);
    assert_eq!(pesquisa_filtrada[0].guid, artigo2.guid);

    // Recupera o ID do primeiro artigo.
    let artigos = db.list_articles(false, Some(feed1_id), None)?;

    let artigo1_id = artigos
        .iter()
        .find(|a| a.guid == artigo1.guid)
        .expect("Artigo 1 não encontrado")
        .id;

    // Testa marcação como lido.
    db.mark_read(artigo1_id, true)?;

    let nao_lidos = db.list_articles(true, Some(feed1_id), None)?;

    assert!(!nao_lidos.iter().any(|a| a.id == artigo1_id));

    // Testa marcação novamente como não lido.
    db.mark_read(artigo1_id, false)?;

    let nao_lidos = db.list_articles(true, Some(feed1_id), None)?;

    assert!(nao_lidos.iter().any(|a| a.id == artigo1_id));

    // Testa adicionar favorito.
    db.mark_favorite(artigo1_id, true)?;

    let artigos = db.list_articles(false, Some(feed1_id), None)?;

    let artigo_favoritado = artigos
        .iter()
        .find(|a| a.id == artigo1_id)
        .expect("Artigo não encontrado");

    assert!(artigo_favoritado.is_favorite);

    // Testa remover favorito.
    db.mark_favorite(artigo1_id, false)?;

    let artigos = db.list_articles(false, Some(feed1_id), None)?;

    let artigo_atualizado = artigos
        .iter()
        .find(|a| a.id == artigo1_id)
        .expect("Artigo não encontrado");

    assert!(!artigo_atualizado.is_favorite);

    // Testa tratamento de artigo inexistente.
    assert!(db.mark_read(-1, true).is_err());
    assert!(db.mark_favorite(-1, true).is_err());

    // Testa a preservação dos metadados HTTP.
    db.update_feed_metadata(
        feed1_id,
        Some("\"etag-teste-123\""),
        Some("Wed, 21 Oct 2015 07:28:00 GMT"),
    )?;

    // Atualiza sem receber novos metadados do servidor.
    db.update_feed_metadata(feed1_id, None, None)?;

    // Verifica se os valores anteriores foram preservados.
    let feeds_atualizados = db.list_feeds()?;

    let feed_atualizado = feeds_atualizados
        .iter()
        .find(|feed| feed.id == feed1_id)
        .expect("Feed de teste não encontrado");

    assert_eq!(feed_atualizado.etag.as_deref(), Some("\"etag-teste-123\""));

    assert_eq!(
        feed_atualizado.last_modified.as_deref(),
        Some("Wed, 21 Oct 2015 07:28:00 GMT")
    );

    // Remove os feeds e seus respectivos artigos.
    db.remove_feed(feed1_id)?;
    db.remove_feed(feed2_id)?;

    println!("Testes de banco de dados, filtros e artigos aprovados!");

    Ok(())
}
