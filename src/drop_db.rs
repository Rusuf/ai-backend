use dotenv::dotenv;
use sqlx::PgPool;
use std::env;
use std::fs;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    print!(
        "Are you sure you want to clear the database? This will drop all tables and data. [y/N]: "
    );
    io::stdout().flush()?;
    let mut confirmation = String::new();
    io::stdin().read_line(&mut confirmation)?;

    if confirmation.trim().to_lowercase() != "y" {
        println!("Database clear cancelled.");
        return Ok(());
    }

    let pool = PgPool::connect(&env::var("DATABASE_URL")?).await?;

    let init_sql = fs::read_to_string("init.sql")?;

    for query in init_sql.split(';') {
        let query = query.trim();
        if !query.is_empty() {
            sqlx::query(query).execute(&pool).await?;
        }
    }

    println!("Database cleared successfully.");

    Ok(())
}