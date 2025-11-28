use dotenv::dotenv;
use sqlx::postgres::PgConnectOptions;
use sqlx::{ConnectOptions, Connection};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let mut connect_options: PgConnectOptions = database_url.parse()?;
    let db_name = connect_options.get_database().unwrap_or("").to_string();

    if db_name.is_empty() {
        eprintln!("Could not determine database name from DATABASE_URL.");
        return Ok(());
    }

    // Connect to the default 'postgres' database to create the target database
    connect_options = connect_options.database("postgres");

    let mut connection = sqlx::PgConnection::connect_with(&connect_options).await?;

    println!("Creating database '{}'...", db_name);

    let query = format!("CREATE DATABASE \"{}\"", db_name);
    sqlx::query(&query).execute(&mut connection).await?;

    println!("Database '{}' created successfully.", db_name);

    Ok(())
}

