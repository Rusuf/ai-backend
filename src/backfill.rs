
use sqlx::{MySqlPool, PgPool};
use std::env;

pub async fn backfill() -> anyhow::Result<()> {
    println!("Starting backfill...");

    let pg_pool = PgPool::connect(&env::var("PG_DSN")?).await?;
    let mysql_pool = MySqlPool::connect(&env::var("MYSQL_DSN")?).await?;

    // --- Backfill Products ---
    let products = sqlx::query!(
        r#"
        SELECT product_id, product_code, productname, category, department, unit, buyingprice, sellingprice, current_stock, last_updated
        FROM products
        WHERE last_updated >= NOW() - INTERVAL 30 DAY
        "#
    )
    .fetch_all(&mysql_pool)
    .await?;

    for p in products {
        sqlx::query(
            r#"
            INSERT INTO products (product_id, code, name, category, department, unit, buy_price, sell_price, current_stock, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (product_id) DO UPDATE SET
                code = EXCLUDED.code,
                name = EXCLUDED.name,
                category = EXCLUDED.category,
                department = EXCLUDED.department,
                unit = EXCLUDED.unit,
                buy_price = EXCLUDED.buy_price,
                sell_price = EXCLUDED.sell_price,
                current_stock = EXCLUDED.current_stock,
                updated_at = EXCLUDED.updated_at;
            "#,
        )
        .bind(String::from_utf8(p.product_id.clone()).unwrap_or_default())
        .bind(p.product_code.as_deref().unwrap_or_default())
        .bind(p.productname.as_deref().unwrap_or_default())
        .bind(p.category.as_deref().unwrap_or_default())
        .bind(p.department.as_deref().unwrap_or_default())
        .bind(p.unit.as_deref().unwrap_or_default())
        .bind(p.buyingprice.unwrap_or_default())
        .bind(p.sellingprice.unwrap_or_default())
        .bind(p.current_stock.unwrap_or_default())
        .bind(p.last_updated)
        .execute(&pg_pool)
        .await?;
    }

    println!("Products backfill complete.");

    // --- Backfill Receipts ---
    let receipts = sqlx::query!(
        r#"
        SELECT receipt_id, receipt_no, date, payment_channel, customer
        FROM receipts
        WHERE date >= NOW() - INTERVAL 30 DAY
        "#
    )
    .fetch_all(&mysql_pool)
    .await?;

    for r in receipts {
        sqlx::query(
            r#"
            INSERT INTO receipts (receipt_id, receipt_no, at, payment_channel, customer)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (receipt_id) DO UPDATE SET
                receipt_no = EXCLUDED.receipt_no,
                at = EXCLUDED.at,
                payment_channel = EXCLUDED.payment_channel,
                customer = EXCLUDED.customer;
            "#,
        )
        .bind(r.receipt_id.unwrap_or_default())
        .bind(r.receipt_no.unwrap_or_default())
        .bind(r.date)
        .bind(r.payment_channel.as_deref().unwrap_or_default())
        .bind(r.customer.as_deref().unwrap_or_default())
        .execute(&pg_pool)
        .await?;
    }

    println!("Receipts backfill complete.");

    // --- Backfill Sales ---
        let sales = sqlx::query!(
        r#"
        SELECT sale_id, thedate, receipt_no, product_code, productname, quantity, sellingprice, totalsales, customer
        FROM sales
        WHERE thedate >= NOW() - INTERVAL 30 DAY
        "#
    )
    .fetch_all(&mysql_pool)
    .await?;

    for s in sales {
        sqlx::query(
            r#"
            INSERT INTO sales (sale_id, at, receipt_no, product_code, product_name, qty, price, total, customer)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (sale_id) DO UPDATE SET
                at = EXCLUDED.at,
                receipt_no = EXCLUDED.receipt_no,
                product_code = EXCLUDED.product_code,
                product_name = EXCLUDED.product_name,
                qty = EXCLUDED.qty,
                price = EXCLUDED.price,
                total = EXCLUDED.total,
                customer = EXCLUDED.customer;
            "#,
        )
        .bind(s.sale_id.unwrap_or_default())
        .bind(s.thedate)
        .bind(s.receipt_no.unwrap_or_default())
        .bind(s.product_code.as_deref().unwrap_or_default())
        .bind(s.productname.as_deref().unwrap_or_default())
        .bind(s.quantity.unwrap_or_default())
        .bind(s.sellingprice.unwrap_or_default())
        .bind(s.totalsales.unwrap_or_default())
        .bind(s.customer.as_deref().unwrap_or_default())
        .execute(&pg_pool)
        .await?;
    }

    println!("Sales backfill complete.");

    println!("Backfill finished successfully!");

    Ok(())
}
