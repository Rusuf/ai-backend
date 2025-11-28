use anyhow::Result;
use serde::Serialize;
use sqlx::PgPool;

#[derive(Serialize, Debug, sqlx::FromRow)]
pub struct TrendingItem {
    pub name: String,
    pub popularity_score: f64,
}

pub async fn get_trending_recipes(pool: &PgPool) -> Result<Vec<TrendingItem>> {
    // Internal trends from sales velocity
    // Note: We cast SUM(quantity) to double precision to ensure it maps to f64
    let rows = sqlx::query_as::<_, TrendingItem>(
        r#"
        SELECT p.name, CAST(COALESCE(SUM(s.quantity), 0) AS DOUBLE PRECISION) as popularity_score
        FROM sales s
        JOIN products p ON s.product_id = p.product_id
        GROUP BY p.name
        ORDER BY popularity_score DESC
        LIMIT 10
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}
