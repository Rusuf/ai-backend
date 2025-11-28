use db_models::MarketTrend;
use sqlx::PgPool;
use std::time::Duration;
use tracing::{info, error};
use rand::Rng;

pub async fn run_market_agent(pool: PgPool) {
    let mut interval = tokio::time::interval(Duration::from_secs(60)); // Run every minute for demo purposes

    loop {
        interval.tick().await;
        info!("ðŸ•µï¸ â€ â™‚ï¸  Market Agent: Scanning external sources for new trends...");

        // Simulate scraping data from external sources
        let fresh_trends = scrape_mock_data();

        for trend in fresh_trends {
            let result = sqlx::query(
                r#"
                INSERT INTO market_trends (category, dish, source, popularity_score, insight, trend_direction, last_updated)
                VALUES ($1, $2, $3, $4, $5, $6, NOW())
                ON CONFLICT (category, dish, source) 
                DO UPDATE SET 
                    popularity_score = EXCLUDED.popularity_score,
                    insight = EXCLUDED.insight,
                    trend_direction = EXCLUDED.trend_direction,
                    last_updated = NOW()
                "#
            )
            .bind(&trend.category)
            .bind(&trend.dish)
            .bind(&trend.source)
            .bind(trend.popularity_score)
            .bind(&trend.insight)
            .bind(&trend.trend_direction)
            .execute(&pool)
            .await;

            if let Err(e) = result {
                error!("Failed to update market trend: {:?}", e);
            }
        }

        
        info!("âœ… Market Agent: Intelligence updated successfully.");
    }
}

fn scrape_mock_data() -> Vec<MarketTrend> {
    let mut rng = rand::thread_rng();
    
    // Simulate dynamic changes
    let glovo_score = rng.gen_range(90..100);
    let smocha_score = rng.gen_range(85..95);
    let seafood_score = rng.gen_range(70..85);
    
    // Occasionally introduce a "Breakout Trend"
    let breakout = if rng.gen_bool(0.3) {
        Some(MarketTrend {
            id: 0, // Ignored on insert
            category: "Viral Social Media".to_string(),
            dish: "Crunchy Korean Fried Chicken".to_string(),
            source: "TikTok Kenya".to_string(),
            popularity_score: rng.gen_range(80..99),
            insight: "Exploding popularity on TikTok due to new Mukbang challenges.".to_string(),
            trend_direction: Some("up".to_string()),
            last_updated: None,
        })
    } else {
        None
    };

    let mut trends = vec![
        MarketTrend {
            id: 0,
            category: "Delivery (Glovo/UberEats)".to_string(),
            dish: "Chicken Tikka Masala".to_string(),
            source: "Glovo Trends 2024".to_string(),
            popularity_score: glovo_score,
            insight: "Consistently top-ordered dinner item across Nairobi.".to_string(),
            trend_direction: Some("stable".to_string()),
            last_updated: None,
        },
        MarketTrend {
            id: 0,
            category: "Street Food".to_string(),
            dish: "Smocha (Smokie + Chapati)".to_string(),
            source: "Nairobi Street Food Index".to_string(),
            popularity_score: smocha_score,
            insight: "Rapidly growing budget lunch option.".to_string(),
            trend_direction: Some("up".to_string()),
            last_updated: None,
        },
        MarketTrend {
            id: 0,
            category: "Fine Dining".to_string(),
            dish: "Ginger Crab Claws".to_string(),
            source: "Tamarind Nairobi".to_string(),
            popularity_score: seafood_score,
            insight: "Premium seafood choice in high-end zones.".to_string(),
            trend_direction: Some(if seafood_score > 80 { "up".to_string() } else { "stable".to_string() }),
            last_updated: None,
        },
    ];

    if let Some(b) = breakout {
        trends.push(b);
    }

    trends
}
