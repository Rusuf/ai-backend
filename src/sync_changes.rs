use dotenv::dotenv;
use sqlx::{MySqlPool, PgPool};
use std::env;
use std::time::Duration;
use db_models::{Customer, Product, Receipt, Sale};
use std::collections::HashMap;

#[derive(sqlx::FromRow, Debug, Clone)]
struct LogChange {
    id: i32,
    table_name: String,
    primary_key_value: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    println!("ðŸ”„ Starting database synchronization service...");

    let mysql_pool = MySqlPool::connect(&env::var("MYSQL_DSN")?).await?;
    println!("âœ“ Connected to MySQL");

    let pg_pool = PgPool::connect(&env::var("DATABASE_URL")?).await?;
    println!("âœ“ Connected to PostgreSQL\n");

    loop {
        println!("\nðŸ” Checking for database changes...");

        if let Err(e) = process_changes(&mysql_pool, &pg_pool).await {
            eprintln!("âœ— Error processing changes: {}", e);
        }

        println!("ðŸ›Œ Synchronization check complete. Waiting for 30 minutes...");
        tokio::time::sleep(Duration::from_secs(1800)).await;
    }
}

async fn process_changes(mysql_pool: &MySqlPool, pg_pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let changes = sqlx::query_as::<_, LogChange>("SELECT id, table_name, primary_key_value FROM log_table_sync_change WHERE status = 'pending' ORDER BY change_time ASC")
        .fetch_all(mysql_pool)
        .await?;

    if changes.is_empty() {
        println!("   No new changes found.");
        return Ok(());
    }

    println!("   Found {} new changes to process.", changes.len());

    let mut grouped_changes: HashMap<String, Vec<LogChange>> = HashMap::new();
    for change in changes {
        grouped_changes.entry(change.table_name.clone()).or_default().push(change);
    }

    for (table_name, changes) in grouped_changes {
        println!("   - Processing {} changes for table '{}'", changes.len(), table_name);
        let result = match table_name.as_str() {
            "customers" => apply_customer_changes(mysql_pool, pg_pool, &changes).await,
            "products" => apply_product_changes(mysql_pool, pg_pool, &changes).await,
            "receipts" => apply_receipt_changes(mysql_pool, pg_pool, &changes).await,
            "sales" => apply_sale_changes(mysql_pool, pg_pool, &changes).await,
            _ => {
                println!("   - Skipping unsupported table: {}", table_name);
                Ok(())
            }
        };

        if result.is_ok() {
            let ids: Vec<i32> = changes.iter().map(|c| c.id).collect();
            sqlx::query(&format!("UPDATE log_table_sync_change SET status = 'synced', synced_at = NOW() WHERE id IN ({})", ids.iter().map(|i| i.to_string()).collect::<Vec<String>>().join(",")))
                .execute(mysql_pool)
                .await?;
        } else {
            eprintln!("   âœ— Failed to apply changes for table {}: {:?}", table_name, result.err());
        }
    }

    Ok(())
}

async fn apply_customer_changes(mysql_pool: &MySqlPool, pg_pool: &PgPool, changes: &[LogChange]) -> Result<(), Box<dyn std::error::Error>> {
    let customer_pks: Vec<&str> = changes.iter().map(|c| c.primary_key_value.as_str()).collect();
    let customers = sqlx::query_as::<_, Customer>(&format!("SELECT * FROM customers WHERE customer_id IN ({})", customer_pks.join(",")))
        .fetch_all(mysql_pool)
        .await?;

    for customer in customers {
        sqlx::query(
            "INSERT INTO customers (customer_id, name, email, registered_on)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (customer_id) DO UPDATE SET
                name = EXCLUDED.name,
                email = EXCLUDED.email,
                registered_on = EXCLUDED.registered_on",
        )
        .bind(customer.customer_id)
        .bind(customer.name)
        .bind(customer.email)
        .bind(customer.registered_on)
        .execute(pg_pool)
        .await?;
    }
    Ok(())
}

async fn apply_product_changes(mysql_pool: &MySqlPool, pg_pool: &PgPool, changes: &[LogChange]) -> Result<(), Box<dyn std::error::Error>> {
    let product_pks: Vec<&str> = changes.iter().map(|c| c.primary_key_value.as_str()).collect();
    let products = sqlx::query_as::<_, Product>(&format!("SELECT * FROM products WHERE product_id IN ({})", product_pks.join(",")))
        .fetch_all(mysql_pool)
        .await?;

    for product in products {
        sqlx::query(
            "INSERT INTO products (product_id, product_code, name, department, category, selling_price, current_stock)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             ON CONFLICT (product_id) DO UPDATE SET
                product_code = EXCLUDED.product_code,
                name = EXCLUDED.name,
                department = EXCLUDED.department,
                category = EXCLUDED.category,
                selling_price = EXCLUDED.selling_price,
                current_stock = EXCLUDED.current_stock",
        )
        .bind(product.product_id)
        .bind(product.product_code)
        .bind(product.name)
        .bind(product.department)
        .bind(product.category)
        .bind(product.selling_price)
        .bind(product.current_stock)
        .execute(pg_pool)
        .await?;
    }
    Ok(())
}

async fn apply_receipt_changes(mysql_pool: &MySqlPool, pg_pool: &PgPool, changes: &[LogChange]) -> Result<(), Box<dyn std::error::Error>> {
    let receipt_pks: Vec<&str> = changes.iter().map(|c| c.primary_key_value.as_str()).collect();
    let receipts = sqlx::query_as::<_, Receipt>(&format!("SELECT * FROM receipts WHERE receipt_id IN ({})", receipt_pks.join(",")))
        .fetch_all(mysql_pool)
        .await?;

    for receipt in receipts {
        let customer_id: Option<i32> = if let Some(customer_email) = receipt.customer_id {
            let row = sqlx::query!("SELECT customer_id FROM customers WHERE email = $1", customer_email.to_string())
                .fetch_optional(pg_pool)
                .await?;
            row.map(|r| r.customer_id)
        } else {
            None
        };

        sqlx::query(
            "INSERT INTO receipts (receipt_id, receipt_no, transaction_date, customer_id, total_amount, payment_channel)
             VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (receipt_id) DO UPDATE SET
                receipt_no = EXCLUDED.receipt_no,
                transaction_date = EXCLUDED.transaction_date,
                customer_id = EXCLUDED.customer_id,
                total_amount = EXCLUDED.total_amount,
                payment_channel = EXCLUDED.payment_channel",
        )
        .bind(receipt.receipt_id)
        .bind(receipt.receipt_no)
        .bind(receipt.transaction_date)
        .bind(customer_id)
        .bind(receipt.total_amount)
        .bind(receipt.payment_channel)
        .execute(pg_pool)
        .await?;
    }
    Ok(())
}

async fn apply_sale_changes(mysql_pool: &MySqlPool, pg_pool: &PgPool, changes: &[LogChange]) -> Result<(), Box<dyn std::error::Error>> {
    let sale_pks: Vec<&str> = changes.iter().map(|c| c.primary_key_value.as_str()).collect();
    let sales = sqlx::query_as::<_, Sale>(&format!("SELECT * FROM sales WHERE sale_id IN ({})", sale_pks.join(",")))
        .fetch_all(mysql_pool)
        .await?;

    for sale in sales {
        let receipt_id: Option<i32> = {
            let row = sqlx::query!("SELECT receipt_id FROM receipts WHERE receipt_no = $1", sale.receipt_id)
                .fetch_optional(pg_pool)
                .await?;
            row.map(|r| r.receipt_id)
        };

        let product_id: Option<i32> = {
            let row = sqlx::query!("SELECT product_id FROM products WHERE product_code = $1", sale.product_id.to_string())
                .fetch_optional(pg_pool)
                .await?;
            row.map(|r| r.product_id)
        };

        if let (Some(rid), Some(pid)) = (receipt_id, product_id) {
            sqlx::query(
                "INSERT INTO sales (sale_id, receipt_id, product_id, quantity, selling_price, total_sale)
                 VALUES ($1, $2, $3, $4, $5, $6)
                 ON CONFLICT (sale_id) DO UPDATE SET
                    receipt_id = EXCLUDED.receipt_id,
                    product_id = EXCLUDED.product_id,
                    quantity = EXCLUDED.quantity,
                    selling_price = EXCLUDED.selling_price,
                    total_sale = EXCLUDED.total_sale",
            )
            .bind(sale.sale_id)
            .bind(rid)
            .bind(pid)
            .bind(sale.quantity)
            .bind(sale.selling_price)
            .bind(sale.total_sale)
            .execute(pg_pool)
            .await?;
        } else {
            eprintln!("   - Warning: Skipping sale with id {} because receipt or product was not found.", sale.sale_id);
        }
    }
    Ok(())
}
