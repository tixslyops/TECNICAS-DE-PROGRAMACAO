#[cfg(test)]
mod tests {
    use crate::models::Article;
    use crate::parser::parse_feed_content;
    use crate::storage::Database;
    use chrono::Utc;

    // 1. Teste de Persistência e Manipulação de Feeds/Artigos
    #[test]
    fn test_database_persistence() {
        let db_path = "test_rss.sqlite";
        let _ = std::fs::remove_file(db_path); // Limpa base anterior se existir

        let db = Database::new(db_path).expect("Falha ao criar banco de testes");
        
        let feed_id = db.add_feed("https://exemplo.com/rss", "Feed Teste").unwrap();
        assert!(feed_id > 0);

        let article = Article {
            id: None,
            feed_id,
            title: "Artigo de Teste".to_string(),
            author: Some("Autor Teste".to_string()),
            link: "https://exemplo.com/artigo-1".to_string(),
            published_at: Some(Utc::now()),
            description: Some("Descrição do artigo de teste".to_string()),
            is_read: false,
            is_favorite: false,
            fetched_at: Utc::now(),
            guid: "guid-12345".to_string(),
        };

        // Testa inserção
        let inserted = db.insert_article(&article).unwrap();
        assert!(inserted, "O artigo deveria ser inserido com sucesso.");

        // Testa deduplicação (inserir o mesmo GUID não deve duplicar)
        let inserted_again = db.insert_article(&article).unwrap();
        assert!(!inserted_again, "O artigo duplicado (mesmo GUID) não deveria ser inserido.");

        // Testa marcação de lido
        let articles = db.get_articles(false, None).unwrap();
        assert_eq!(articles.len(), 1);
        let art_id = articles[0].id.unwrap();

        db.mark_as_read(art_id, true).unwrap();
        let unread_articles = db.get_articles(true, None).unwrap();
        assert_eq!(unread_articles.len(), 0, "Não deveria haver artigos não lidos.");

        let _ = std::fs::remove_file(db_path);
    }

    // 2. Teste do Parser de RSS/Atom e Tratamento de Conteúdo Vazio/Malformado
    #[test]
    fn test_parser_valid_rss() {
        let rss_xml = br#"<?xml version="1.0" encoding="UTF-8" ?>
        <rss version="2.0">
            <channel>
                <title>Site Teste</title>
                <link>https://site-a.com</link>
                <item>
                    <title>Noticia Urgente</title>
                    <link>https://site-a.com/noticia-1</link>
                    <guid>guid-noticia-1</guid>
                    <description>Resumo da noticia importante.</description>
                </item>
            </channel>
        </rss>"#;

        let articles = parse_feed_content(rss_xml, 1).unwrap();
        assert_eq!(articles.len(), 1);
        assert_eq!(articles[0].title, "Noticia Urgente");
        assert_eq!(articles[0].guid, "guid-noticia-1");
    }

    #[test]
    fn test_parser_malformed_xml() {
        let bad_xml = b"<xml><unclosed_tag>";
        let result = parse_feed_content(bad_xml, 1);
        assert!(result.is_err(), "O parser deve retornar erro para XML malformado sem quebrar a aplicação.");
    }
}