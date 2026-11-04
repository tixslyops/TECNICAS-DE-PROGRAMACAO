mod tests;
mod models;
mod storage;
mod parser;
mod fetcher;

use storage::Database;
use fetcher::fetch_feed_data;
use parser::parse_feed_content;
use reqwest::Client;
use std::io::{self, Write};
use futures::stream::{self, StreamExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::new("rss_database.sqlite")?;
    let client = Client::new();

    loop {
        println!("\n=== AGREGADOR DE RSS EM RUST ===");
        println!("1. Listar Feeds");
        println!("2. Cadastrar novo Feed");
        println!("3. Atualizar todos os Feeds (Concorrente)");
        println!("4. Listar Artigos");
        println!("5. Marcar Artigo como Lido");
        println!("6. Favoritar/Desfavoritar Artigo");
        println!("7. Pesquisar Artigos");
        println!("0. Sair");
        print!("Escolha uma opção: ");
        io::stdout().flush()?;

        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?;
        let choice = choice.trim();

        match choice {
            "1" => {
                let feeds = db.get_feeds()?;
                println!("\n--- Feeds Cadastrados ---");
                for f in feeds {
                    println!("ID: {} | Título: {} | URL: {} | Falhas: {}", f.id.unwrap(), f.title, f.url, f.failure_count);
                }
            }
            "2" => {
                print!("Digite a URL do Feed: ");
                io::stdout().flush()?;
                let mut url = String::new();
                io::stdin().read_line(&mut url)?;
                let url = url.trim().to_string();

                print!("Digite um título descritivo para o Feed: ");
                io::stdout().flush()?;
                let mut title = String::new();
                io::stdin().read_line(&mut title)?;
                let title = title.trim().to_string();

                match db.add_feed(&url, &title) {
                    Ok(id) => println!("Feed cadastrado com sucesso! ID: {}", id),
                    Err(e) => eprintln!("Erro ao cadastrar feed (pode já existir): {}", e),
                }
            }
            "3" => {
                println!("Atualizando feeds concorrentemente...");
                let feeds = db.get_feeds()?;
                let client_clone = client.clone();
                
                let tasks = streams_update(feeds, client_clone, db_arc_clone(&db));
                tasks.await;
                println!("Atualização concluída!");
            }
            "4" => {
                print!("Apenas não lidos? (s/n): ");
                io::stdout().flush()?;
                let mut opt = String::new();
                io::stdin().read_line(&mut opt)?;
                let unread = opt.trim().eq_ignore_ascii_case("s");

                let articles = db.get_articles(unread, None)?;
                print_articles(&articles);
            }
            "5" => {
                print!("ID do artigo para marcar como lido: ");
                io::stdout().flush()?;
                let mut id_str = String::new();
                io::stdin().read_line(&mut id_str)?;
                if let Ok(id) = id_str.trim().parse::<i64>() {
                    db.mark_as_read(id, true)?;
                    println!("Artigo marcado como lido!");
                }
            }
            "6" => {
                print!("ID do artigo para alternar favorito: ");
                io::stdout().flush()?;
                let mut id_str = String::new();
                io::stdin().read_line(&mut id_str)?;
                if let Ok(id) = id_str.trim().parse::<i64>() {
                    db.toggle_favorite(id)?;
                    println!("Status de favorito alterado!");
                }
            }
            "7" => {
                print!("Termo de pesquisa (título ou conteúdo): ");
                io::stdout().flush()?;
                let mut term = String::new();
                io::stdin().read_line(&mut term)?;
                let term = term.trim();

                let articles = db.get_articles(false, Some(term))?;
                print_articles(&articles);
            }
            "0" => {
                println!("Saindo...");
                break;
            }
            _ => println!("Opção inválida!"),
        }
    }

    Ok(())
}

fn print_articles(articles: &[models::Article]) {
    println!("\n--- Lista de Artigos ({}) ---", articles.len());
    for a in articles {
        let status = if a.is_read { "[Lido]" } else { "[Não Lido]" };
        let fav = if a.is_favorite { "[★]" } else { "" };
        println!("{} {} ID: {} | Título: {} | Link: {}", status, fav, a.id.unwrap(), a.title, a.link);
    }
}

async fn streams_update(feeds: Vec<models::Feed>, client: Client, db: std::sync::Arc<std::sync::Mutex<Database>>) {
    let stream = stream::iter(feeds).map(|feed| {
        let client = client.clone();
        let db = db.clone();
        tokio::spawn(async move {
            let feed_id = feed.id.unwrap();
            match fetch_feed_data(&client, &feed.url, feed.etag.as_deref(), feed.last_modified.as_deref()).await {
                Ok((Some(bytes), new_etag, new_lm, _)) => {
                    match parse_feed_content(&bytes, feed_id) {
                        Ok(articles) => {
                            let db_lock = db.lock().unwrap();
                            for article in articles {
                                let _ = db_lock.insert_article(&article);
                            }
                            let _ = db_lock.update_feed_meta(feed_id, new_etag.as_deref(), new_lm.as_deref(), true);
                            println!("Feed atualizado com sucesso: {}", feed.title);
                        }
                        Err(e) => eprintln!("Erro ao fazer parse do feed {}: {}", feed.title, e),
                    }
                }
                Ok((None, _, _, true)) => {
                    println!("Feed sem modificações (304 Not Modified): {}", feed.title);
                    let db_lock = db.lock().unwrap();
                    let _ = db_lock.update_feed_meta(feed_id, None, None, true);
                }
                Err(e) => {
                    eprintln!("Erro ao buscar feed {}: {}", feed.title, e);
                    let db_lock = db.lock().unwrap();
                    let _ = db_lock.update_feed_meta(feed_id, None, None, false);
                }
                _ => {}
            }
        })
    });

    stream.buffer_unordered(5).collect::<Vec<_>>().await;
}

fn db_arc_clone(_db: &Database) -> std::sync::Arc<std::sync::Mutex<Database>> {
    std::sync::Arc::new(std::sync::Mutex::new(Database::new("rss_database.sqlite").unwrap()))
}