use agregador_rss::database::Database;
use agregador_rss::fetcher::Fetcher;
use agregador_rss::service::FeedService;

use anyhow::{Context, Result};
use dotenvy::dotenv;
use std::env;

mod cli;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let database_url =
        env::var("DATABASE_URL").context("DATABASE_URL não encontrada no arquivo .env")?;

    let database = Database::new(&database_url).context("Erro ao conectar ao banco MySQL")?;

    println!("Conectado ao MySQL com sucesso!");

    let fetcher = Fetcher::new().context("Erro ao inicializar o cliente HTTP")?;

    let service = FeedService::new(database, fetcher);

    cli::run(&service).await?;

    Ok(())
}
