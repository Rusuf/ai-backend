use crate::ml::apriori;
use serde::Serialize;
use sqlx::PgPool;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize)]
pub struct Recommendation {
    pub product_id: i32,
    pub score: f64,
}

pub async fn train_and_cache_recommendations(
    pool: &PgPool,
) -> Result<HashMap<i32, Vec<Recommendation>>, sqlx::Error> {
    println!("Starting recommendation model training...");

    let transactions = get_transactions(pool).await?;
    if transactions.is_empty() {
        println!("No transactions found to train recommendation model.");
        return Ok(HashMap::new());
    }

    // Use the external apriori module
    let rules = apriori::apriori(&transactions, 0.01, 0.1);
    println!("Apriori algorithm generated {} rules.", rules.len());

    let mut recommendation_map: HashMap<i32, Vec<Recommendation>> = HashMap::new();
    for rule in rules {
        if rule.lhs.len() == 1 {
            let antecedent = rule.lhs[0];
            for consequent in &rule.rhs {
                let recommendation = Recommendation {
                    product_id: *consequent,
                    score: rule.confidence,
                };
                recommendation_map
                    .entry(antecedent)
                    .or_default()
                    .push(recommendation);
            }
        }
    }

    for recommendations in recommendation_map.values_mut() {
        recommendations.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    }

    println!("Recommendation model training complete.");
    Ok(recommendation_map)
}

pub fn get_recommendations_from_cache(
    cache: &HashMap<i32, Vec<Recommendation>>,
    product_id: i32,
) -> Vec<Recommendation> {
    cache.get(&product_id).cloned().unwrap_or_default()
}

async fn get_transactions(pool: &PgPool) -> Result<Vec<Vec<i32>>, sqlx::Error> {
    // Ensure we only get valid sales with both receipt and product IDs
    let rows = sqlx::query!("SELECT receipt_id, product_id FROM sales WHERE receipt_id IS NOT NULL AND product_id IS NOT NULL")
        .fetch_all(pool)
        .await?;

    let mut transactions_map: HashMap<i32, Vec<i32>> = HashMap::new();
    for row in rows {
        if let (Some(receipt_id), Some(product_id)) = (row.receipt_id, row.product_id) {
            transactions_map
                .entry(receipt_id)
                .or_default()
                .push(product_id);
        }
    }

    Ok(transactions_map.into_values().collect())
}