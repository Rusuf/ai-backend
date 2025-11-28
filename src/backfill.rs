use dotenv::dotenv;
use sqlx::{MySqlPool, PgPool};
use std::env;

mod migration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    println!("ðŸ”„ Starting MySQL to PostgreSQL migration...\n");

    // --- Connect to databases using sqlx ---
    let mysql_pool = MySqlPool::connect(&env::var("MYSQL_DSN")?).await?;
    println!("âœ“ Connected to MySQL");

    let pg_pool = PgPool::connect(&env::var("DATABASE_URL")?).await?;
    println!("âœ“ Connected to PostgreSQL\n");

    // --- Create tables in PostgreSQL ---
    create_pg_tables(&pg_pool).await?;

    // --- Migrate data ---
    migration::customer::migrate_customers(&mysql_pool, &pg_pool).await?;
    migration::product::migrate_products(&mysql_pool, &pg_pool).await?;
    migration::receipt::migrate_receipts(&mysql_pool, &pg_pool).await?;
    migration::sale::migrate_sales(&mysql_pool, &pg_pool).await?;

    println!("\nâœ… Migration completed successfully!");
    Ok(())
}

async fn create_pg_tables(pg_pool: &PgPool) -> Result<(), sqlx::Error> {
    println!("ðŸ›  Creating tables in PostgreSQL...");
    let queries = vec![
        "DROP TABLE IF EXISTS sales;",
        "DROP TABLE IF EXISTS receipts;",
        "DROP TABLE IF EXISTS products;",
        "DROP TABLE IF EXISTS customers;",
        "CREATE TABLE products (
            product_id INTEGER PRIMARY KEY,
            product_code VARCHAR(100) UNIQUE,
            name VARCHAR(100),
            department VARCHAR(100),
            category VARCHAR(100),
            selling_price REAL,
            current_stock REAL
        );",
        "CREATE TABLE customers (
            customer_id INTEGER PRIMARY KEY,
            name VARCHAR(100),
            email VARCHAR(100),
            registered_on TIMESTAMP
        );",
        "CREATE TABLE receipts (
            receipt_id INTEGER PRIMARY KEY,
            receipt_no INTEGER,
            transaction_date TIMESTAMP,
            customer_id INTEGER REFERENCES customers(customer_id),
            total_amount REAL,
            payment_channel VARCHAR(100)
        );",
        "CREATE TABLE sales (
            sale_id INTEGER PRIMARY KEY,
            receipt_id INTEGER REFERENCES receipts(receipt_id),
            product_id INTEGER REFERENCES products(product_id),
            quantity REAL,
            selling_price REAL,
            total_sale REAL
        );",
    ];

    for query in queries {
        sqlx::query(query).execute(pg_pool).await?;
    }

    println!("âœ“ PostgreSQL tables ready\n");
    Ok(())
}