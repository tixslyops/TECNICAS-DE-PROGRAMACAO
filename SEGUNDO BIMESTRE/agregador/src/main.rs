mod db;
mod fetcher;
mod models;

use db::Database;
use fetcher::Fetcher;
use std::io::{self, Write};
use std::sync::Arc;
use tokio::sync::Semaphore;

#[tokio::main]
async fn main() {
    let db = Arc::new(Database::new("rss_aggregator.db").expect("Falha ao inicializar base de dados"));
    let fetcher = Arc::new(Fetcher::new());

    loop {
        println!("\n=== AGREGADOR RSS / ATOM ===");
        println!("1. Cadastrar Feed");
        println!("2. Listar Feeds");
        println!("3. Remover Feed");
        println!("4. Atualizar Feeds (Concorrente)");
        println!("5. Ver Artigos");
        println!("6. Marcar Artigo como Lido");
        println!("7. Sair");
        print!("Escolha uma opção: ");
        io::stdout().flush().unwrap();

        let mut opcao = String::new();
        io::stdin().read_line(&mut opcao).unwrap();

        match opcao.trim() {
            "1" => {
                print!("URL do Feed: ");
                io::stdout().flush().unwrap();
                let mut url = String::new();
                io::stdin().read_line(&mut url).unwrap();
                let url = url.trim();

                if !url.is_empty() {
                    match db.adicionar_feed(url) {
                        Ok(id) => println!("Feed cadastrado com ID: {}", id),
                        Err(e) => println!("Erro ao cadastrar feed: {}", e),
                    }
                }
            }
            "2" => match db.listar_feeds() {
                Ok(feeds) => {
                    println!("\n--- FEEDS CADASTRADOS ---");
                    for f in feeds {
                        println!(
                            "ID: {} | URL: {} | Erros: {} | Última Checagem: {:?}",
                            f.id, f.url, f.failure_count, f.last_checked
                        );
                    }
                }
                Err(e) => println!("Erro ao listar feeds: {}", e),
            },
            "3" => {
                print!("ID do Feed a remover: ");
                io::stdout().flush().unwrap();
                let mut id_str = String::new();
                io::stdin().read_line(&mut id_str).unwrap();
                if let Ok(id) = id_str.trim().parse::<i64>() {
                    let _ = db.remover_feed(id);
                    println!("Feed removido!");
                }
            }
            "4" => {
                println!("\nAtualizando feeds concorrentemente...");
                let feeds = db.listar_feeds().unwrap_or_default();

                let semaforo = Arc::new(Semaphore::new(5));
                let mut handles = vec![];

                for feed in feeds {
                    let db_clone = Arc::clone(&db);
                    let fetcher_clone = Arc::clone(&fetcher);
                    let sem_clone = Arc::clone(&semaforo);

                    let handle = tokio::spawn(async move {
                        let _permit = sem_clone.acquire().await.unwrap();
                        match fetcher_clone.buscar_feed(&feed).await {
                            Ok((artigos, etag, last_mod)) => {
                                let mut inseridos = 0;
                                for artigo in artigos {
                                    if let Ok(true) = db_clone.salvar_artigo(&artigo) {
                                        inseridos += 1;
                                    }
                                }
                                let _ = db_clone.atualizar_status_feed(feed.id, etag, last_mod, false);
                                println!("Feed {}: {} novos artigos adicionados.", feed.url, inseridos);
                            }
                            Err(e) => {
                                println!("Erro ao buscar feed {}: {}", feed.url, e);
                                let _ = db_clone.atualizar_status_feed(feed.id, None, None, true);
                            }
                        }
                    });
                    handles.push(handle);
                }

                for handle in handles {
                    let _ = handle.await;
                }
                println!("Atualização concluída.");
            }
            "5" => {
                print!("Filtrar apenas não lidos? (s/n): ");
                io::stdout().flush().unwrap();
                let mut resp = String::new();
                io::stdin().read_line(&mut resp).unwrap();
                let apenas_nao_lidos = resp.trim().eq_ignore_ascii_case("s");

                print!("Termo de pesquisa (ou Enter para ignorar): ");
                io::stdout().flush().unwrap();
                let mut termo = String::new();
                io::stdin().read_line(&termo).unwrap();
                let termo_clean = termo.trim();
                let filtro = if termo_clean.is_empty() { None } else { Some(termo_clean) };

                match db.listar_artigos(apenas_nao_lidos, filtro) {
                    Ok(artigos) => {
                        println!("\n--- ARTIGOS ({}) ---", artigos.len());
                        for a in artigos {
                            let status = if a.lido { "Lido" } else { "NOVO" };
                            println!(
                                "[ID: {}] [{}] {}\nLink: {}\nData: {:?}\n",
                                a.id, status, a.titulo, a.link, a.data_publicacao
                            );
                        }
                    }
                    Err(e) => println!("Erro ao listar artigos: {}", e),
                }
            }
            "6" => {
                print!("ID do Artigo: ");
                io::stdout().flush().unwrap();
                let mut id_str = String::new();
                io::stdin().read_line(&mut id_str).unwrap();
                if let Ok(id) = id_str.trim().parse::<i64>() {
                    let _ = db.marcar_lido(id, true);
                    println!("Artigo marcado como lido!");
                }
            }
            "7" => break,
            _ => println!("Opção inválida!"),
        }
    }
}