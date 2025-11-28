use anyhow::Result;
use db_models::MarketTrend;
use sqlx::PgPool;

pub async fn get_market_intelligence(pool: &PgPool) -> Result<Vec<MarketTrend>> {
    let trends = sqlx::query_as::<_, MarketTrend>(
        "SELECT id, category, dish, source, popularity_score, insight, trend_direction, last_updated FROM market_trends ORDER BY popularity_score DESC"
    )
    .fetch_all(pool)
    .await?;

    Ok(trends)
}

