
use sqlx::{mysql::MySqlPool, PgPool};
use std::env;
use std::time::Duration;

async fn get_watermark(pg_pool: &PgPool) -> anyhow::Result<Option<chrono::NaiveDateTime>> {
    let row: Option<(Option<chrono::NaiveDateTime>,)> = sqlx::query_as("SELECT watermark FROM ingest_state WHERE id = 'main'")
        .fetch_optional(pg_pool)
        .await?;
    Ok(row.and_then(|r| r.0))
}

async fn set_watermark(pg_pool: &PgPool, watermark: chrono::NaiveDateTime) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO ingest_state (id, watermark) VALUES ($1, $2) ON CONFLICT (id) DO UPDATE SET watermark = $2",
    )
    .bind("main")
    .bind(watermark)
    .execute(pg_pool)
    .await?;
    Ok(())
}

async fn process_changes(pg_pool: &PgPool, mysql_pool: &MySqlPool) -> anyhow::Result<()> {
    let watermark = get_watermark(pg_pool).await?.unwrap_or(chrono::NaiveDate::from_ymd_opt(2000, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap());

    let changes = sqlx::query!(
        r#"
        SELECT id, table_name, primary_key_value, change_type, change_time
        FROM log_table_sync_change
        WHERE change_time > ? AND status = 'pending'
        ORDER BY change_time ASC
        LIMIT 1000
        "#,
        watermark
    )
    .fetch_all(mysql_pool)
    .await?;

    for change in changes {
        let pk = &change.primary_key_value;
        match change.table_name.as_str() {
            "sales" => {
                // Process sales changes
            }
            "products" => {
                // Process products changes
            }
            "receipts" => {
                // Process receipts changes
            }
            _ => {}
        }

        if let Some(change_time) = change.change_time {
            set_watermark(pg_pool, change_time).await?;
        }
    }

    Ok(())
}

pub async fn start_ingest() -> anyhow::Result<()> {
    println!("Starting incremental ingest...");

    let pg_pool = PgPool::connect(&env::var("PG_DSN")?).await?;
    let mysql_pool = MySqlPool::connect(&env::var("MYSQL_DSN")?).await?;
    let poll_interval_secs: u64 = env::var("INGEST_POLL_SECS").unwrap_or("5".to_string()).parse()?;

    loop {
        if let Err(e) = process_changes(&pg_pool, &mysql_pool).await {
            eprintln!("Error processing changes: {}", e);
        }
        tokio::time::sleep(Duration::from_secs(poll_interval_secs)).await;
    }
}
