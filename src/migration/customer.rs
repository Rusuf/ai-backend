use db_models::Customer;
use sqlx::{MySqlPool, PgPool};

pub async fn migrate_customers(mysql_pool: &MySqlPool, pg_pool: &PgPool) -> Result<(), sqlx::Error> {
    println!("ðŸ‘¥ Migrating customers...");

    let customers = sqlx::query_as::<_, Customer>("SELECT customer_id, name, email, registered_on FROM customers")
        .fetch_all(mysql_pool)
        .await?;

    println!("   Found {} customer records in MySQL", customers.len());

    let mut success_count = 0;
    for customer in customers {
        let result = sqlx::query(
            "INSERT INTO customers (customer_id, name, email, registered_on)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (customer_id) DO UPDATE SET
                name = EXCLUDED.name,
                email = EXCLUDED.email,
                registered_on = EXCLUDED.registered_on;",
        )
        .bind(customer.customer_id)
        .bind(customer.name)
        .bind(customer.email)
        .bind(customer.registered_on)
        .execute(pg_pool)
        .await;

        if result.is_ok() {
            success_count += 1;
        }
    }

    println!("   âœ“ Migrated {} customers", success_count);
    Ok(())
}