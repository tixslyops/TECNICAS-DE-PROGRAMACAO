use anyhow::Result;
use clap::{Parser, Subcommand};

use agregador_rss::service::FeedService;

#[derive(Parser)]
#[command(
    name = "agregador_rss",
    version,
    about = "Agregador de notícias RSS e Atom"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Cadastra um novo feed
    Add { title: String, url: String },

    /// Remove um feed pelo ID
    Remove { id: i64 },

    /// Lista todos os feeds cadastrados
    Feeds,

    /// Atualiza todos os feeds
    Update,

    /// Atualiza um feed específico
    UpdateId { id: i64 },

    /// Lista artigos
    Articles {
        #[arg(long)]
        unread: bool,

        #[arg(long)]
        feed: Option<i64>,
    },

    /// Lista somente artigos não lidos
    Unread,

    /// Marca um artigo como lido
    Read { id: i64 },

    /// Marca um artigo como não lido
    UnreadId { id: i64 },

    /// Adiciona um artigo aos favoritos
    Favorite { id: i64 },

    /// Remove um artigo dos favoritos
    Unfavorite { id: i64 },

    /// Pesquisa artigos pelo título ou descrição
    Search { query: String },
}

pub async fn run(service: &FeedService) -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Add { title, url } => {
            service.add_feed(&title, &url)?;
            println!("Feed cadastrado com sucesso!");
        }

        Commands::Remove { id } => {
            service.remove_feed(id)?;
            println!("Feed removido com sucesso!");
        }

        Commands::Feeds => {
            let feeds = service.list_feeds()?;

            if feeds.is_empty() {
                println!("Nenhum feed cadastrado.");
            } else {
                for feed in feeds {
                    println!("ID: {}", feed.id);
                    println!("Título: {}", feed.title);
                    println!("URL: {}", feed.url);
                    println!("Falhas: {}", feed.failure_count);
                    println!("Última atualização: {:?}", feed.last_checked);
                    println!("------------------------------");
                }
            }
        }

        Commands::Update => {
            service.update_all().await?;
        }

        Commands::UpdateId { id } => {
            service.update_feed(id).await?;
        }

        Commands::Articles { unread, feed } => {
            service.list_articles(unread, feed, None)?;
        }

        Commands::Unread => {
            service.list_articles(true, None, None)?;
        }

        Commands::Read { id } => {
            service.mark_read(id, true)?;
            println!("Artigo marcado como lido.");
        }

        Commands::UnreadId { id } => {
            service.mark_read(id, false)?;
            println!("Artigo marcado como não lido.");
        }

        Commands::Favorite { id } => {
            service.mark_favorite(id, true)?;
            println!("Artigo adicionado aos favoritos.");
        }

        Commands::Unfavorite { id } => {
            service.mark_favorite(id, false)?;
            println!("Artigo removido dos favoritos.");
        }

        Commands::Search { query } => {
            service.list_articles(false, None, Some(&query))?;
        }
    }

    Ok(())
}
