use serde::Serialize;
use sqlx::{MySqlPool, PgPool};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;

pub async fn migrate_sales(mysql_pool: &MySqlPool, pg_pool: &PgPool) -> Result<(), sqlx::Error> {
    println!("ðŸ“ Migrating sales...");

    let receipt_lookup: HashMap<i32, i32> = {
        println!("   - Caching receipt data from PostgreSQL...");
        let mut lookup = HashMap::new();
        let rows = sqlx::query!("SELECT receipt_id, receipt_no FROM receipts")
            .fetch_all(pg_pool)
            .await?;
        for row in rows {
            if let Some(receipt_no) = row.receipt_no {
                lookup.insert(receipt_no, row.receipt_id);
            }
        }
        println!("   - Cached {} receipt records.", lookup.len());
        lookup
    };

    let product_lookup: HashMap<String, i32> = {
        println!("   - Caching product data from PostgreSQL...");
        let mut lookup = HashMap::new();
        let rows = sqlx::query!("SELECT product_id, product_code FROM products")
            .fetch_all(pg_pool)
            .await?;
        for row in rows {
            if let Some(product_code) = row.product_code {
                lookup.insert(product_code, row.product_id);
            }
        }
        println!("   - Cached {} product records.", lookup.len());
        lookup
    };

    #[derive(sqlx::FromRow, Debug, Serialize)]
    struct MySqlSale {
        sale_id: i32,
        receipt_no: i32,
        product_code: String,
        quantity: f32,
        sellingprice: f32,
        totalsales: f32,
    }

    let sales = sqlx::query_as::<_, MySqlSale>(
        "SELECT sale_id, receipt_no, product_code, quantity, sellingprice, totalsales FROM sales",
    )
    .fetch_all(mysql_pool)
    .await?;

    println!("   Found {} sales records in MySQL", sales.len());

    let mut success_count = 0;
    let mut not_found_count = 0;

    let mut log_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("skipped_sales.log")
        .unwrap();

    for sale in sales {
        let receipt_id = receipt_lookup.get(&sale.receipt_no).cloned();
        let product_id = product_lookup.get(&sale.product_code).cloned();

        if let (Some(rid), Some(pid)) = (receipt_id, product_id) {
            let result = sqlx::query(
                "INSERT INTO sales (sale_id, receipt_id, product_id, quantity, selling_price, total_sale)
                 VALUES ($1, $2, $3, $4, $5, $6)
                 ON CONFLICT (sale_id) DO UPDATE SET
                    receipt_id = EXCLUDED.receipt_id,
                    product_id = EXCLUDED.product_id,
                    quantity = EXCLUDED.quantity,
                    selling_price = EXCLUDED.selling_price,
                    total_sale = EXCLUDED.total_sale;",
            )
            .bind(sale.sale_id)
            .bind(rid)
            .bind(pid)
            .bind(sale.quantity)
            .bind(sale.sellingprice)
            .bind(sale.totalsales)
            .execute(pg_pool)
            .await;

            if result.is_ok() {
                success_count += 1;
            }
        } else {
            not_found_count += 1;
            let log_entry = serde_json::to_string(&sale).unwrap();
            writeln!(log_file, "{}", log_entry).unwrap();
        }
    }

    println!("   âœ“ Migrated {} sales successfully.", success_count);
    if not_found_count > 0 {
        println!("   âš  Warning: {} sales were skipped because their corresponding receipt or product was not found in PostgreSQL. See skipped_sales.log for details.", not_found_count);
    }
    Ok(())
}
