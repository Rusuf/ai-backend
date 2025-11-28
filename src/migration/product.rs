use db_models::Product;
use sqlx::{MySqlPool, PgPool};

#[derive(sqlx::FromRow)]
struct MySqlProduct {
    product_id: i32,
    product_code: String,
    #[sqlx(rename = "productname")]
    name: String,
    department: String,
    category: String,
    #[sqlx(rename = "sellingprice")]
    selling_price: String,
    current_stock: String,
}

pub async fn migrate_products(mysql_pool: &MySqlPool, pg_pool: &PgPool) -> Result<(), sqlx::Error> {
    println!("ðŸ“– Migrating products...");

    let mysql_products = sqlx::query_as::<_, MySqlProduct>("SELECT product_id, product_code, productname, department, category, sellingprice, current_stock FROM products")
        .fetch_all(mysql_pool)
        .await?;

    println!("   Found {} product records in MySQL", mysql_products.len());

    let products: Vec<Product> = mysql_products
        .into_iter()
        .map(|p| {
            let selling_price = p.selling_price.trim().parse::<f32>().unwrap_or(0.0);
            let current_stock = p.current_stock.trim().parse::<f32>().unwrap_or(0.0);
            Product {
                product_id: p.product_id,
                product_code: p.product_code,
                name: p.name,
                department: p.department,
                category: p.category,
                selling_price,
                current_stock,
            }
        })
        .collect();

    let mut success_count = 0;
    for product in &products {
        let result = sqlx::query(
            "INSERT INTO products (product_id, product_code, name, department, category, selling_price, current_stock)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             ON CONFLICT (product_id) DO UPDATE SET
                product_code = EXCLUDED.product_code,
                name = EXCLUDED.name,
                department = EXCLUDED.department,
                category = EXCLUDED.category,
                selling_price = EXCLUDED.selling_price,
                current_stock = EXCLUDED.current_stock;",
        )
        .bind(product.product_id)
        .bind(&product.product_code)
        .bind(&product.name)
        .bind(&product.department)
        .bind(&product.category)
        .bind(product.selling_price)
        .bind(product.current_stock)
        .execute(pg_pool)
        .await;

        if result.is_ok() {
            success_count += 1;
        }
    }

    println!("   âœ“ Migrated {} products", success_count);
    Ok(())
}
