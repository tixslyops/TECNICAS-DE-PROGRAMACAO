use mysql::prelude::*;
use mysql::Pool;
use dotenvy::dotenv;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")?;
    let pool = Pool::new(database_url.as_str())?;
    let mut conn = pool.get_conn()?;

    let resultado: Option<String> =
        conn.query_first("SELECT DATABASE()")?;

    println!("Conectado ao banco: {:?}", resultado);

    Ok(())
}