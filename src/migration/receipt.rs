
use serde::Serialize;
use sqlx::{MySqlPool, PgPool, types::chrono::NaiveDateTime};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;

#[derive(sqlx::FromRow, Serialize)]
struct MySqlReceipt {
    receipt_id: i32,
    receipt_no: i32,
    #[sqlx(rename = "date")]
    transaction_date: Option<NaiveDateTime>,
    customer: Option<String>, // This is the email
    #[sqlx(rename = "total_cost_incl")]
    total_amount: String,
    payment_channel: String,
}

pub async fn migrate_receipts(mysql_pool: &MySqlPool, pg_pool: &PgPool) -> Result<(), sqlx::Error> {
    println!("ðŸ§¾ Migrating receipts...");

    let customer_lookup: HashMap<String, i32> = {
        println!("   - Caching customer data from PostgreSQL...");
        let mut lookup = HashMap::new();
        let rows = sqlx::query!("SELECT customer_id, email FROM customers")
            .fetch_all(pg_pool)
            .await?;
        for row in rows {
            if let Some(email) = row.email {
                lookup.insert(email, row.customer_id);
            }
        }
        println!("   - Cached {} customer records.", lookup.len());
        lookup
    };

    let mysql_receipts = sqlx::query_as::<_, MySqlReceipt>("SELECT receipt_id, receipt_no, date, customer, total_cost_incl, payment_channel FROM receipts")
        .fetch_all(mysql_pool)
        .await?;

    println!("   Found {} receipt records in MySQL", mysql_receipts.len());

    let mut success_count = 0;
    let mut not_found_count = 0;

    let mut log_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("skipped_receipts.log")
        .unwrap();

    for mysql_receipt in mysql_receipts {
        let customer_id: Option<i32> = mysql_receipt.customer.as_ref().and_then(|email| customer_lookup.get(email).cloned());
        let total_amount = mysql_receipt.total_amount.trim().parse::<f32>().unwrap_or(0.0);

        if let Some(cid) = customer_id {
            let result = sqlx::query(
                "INSERT INTO receipts (receipt_id, receipt_no, transaction_date, customer_id, total_amount, payment_channel)
                 VALUES ($1, $2, $3, $4, $5, $6)
                 ON CONFLICT (receipt_id) DO UPDATE SET
                    receipt_no = EXCLUDED.receipt_no,
                    transaction_date = EXCLUDED.transaction_date,
                    customer_id = EXCLUDED.customer_id,
                    total_amount = EXCLUDED.total_amount,
                    payment_channel = EXCLUDED.payment_channel;",
            )
            .bind(mysql_receipt.receipt_id)
            .bind(mysql_receipt.receipt_no)
            .bind(mysql_receipt.transaction_date)
            .bind(cid)
            .bind(total_amount)
            .bind(mysql_receipt.payment_channel)
            .execute(pg_pool)
            .await;

            if result.is_ok() {
                success_count += 1;
            }
        } else {
            not_found_count += 1;
            let log_entry = serde_json::to_string(&mysql_receipt).unwrap();
            writeln!(log_file, "{}", log_entry).unwrap();
        }
    }

    println!("   âœ“ Migrated {} receipts successfully.", success_count);
    if not_found_count > 0 {
        println!("   âš  Warning: {} receipts were skipped because their corresponding customer was not found in PostgreSQL. See skipped_receipts.log for details.", not_found_count);
    }
    Ok(())
}