use chrono::NaiveDateTime;
use linfa::prelude::*;
use linfa_linear::LinearRegression;
use ndarray::{Array1, Array2};
use serde::Serialize;
use sqlx::PgPool;
use std::collections::HashMap;

#[derive(Serialize, Debug)]
pub struct StockOptimization {
    pub product_id: i32,
    pub product_name: String,
    pub predicted_demand: f64,
}

pub async fn get_stock_optimization(
    pool: &PgPool,
) -> Result<Vec<StockOptimization>, sqlx::Error> {
    let sales_data = get_sales_data(pool).await?;

    let mut optimizations = Vec::new();

    for (product_id, records) in &sales_data {
        if records.len() < 2 {
            continue; // Not enough data to make a prediction
        }

        let (timestamps, quantities): (Vec<f64>, Vec<f64>) = records
            .iter()
            .map(|(ts, q)| (ts.and_utc().timestamp() as f64, *q as f64))
            .unzip();

        let timestamps = Array2::from_shape_vec((timestamps.len(), 1), timestamps).unwrap();
        let quantities = Array1::from_vec(quantities);

        let dataset = Dataset::new(timestamps, quantities);

        let lin_reg = LinearRegression::new();
        let model = lin_reg.fit(&dataset).unwrap();

        let last_timestamp = records.last().unwrap().0.and_utc().timestamp() as f64;
        let next_timestamp = last_timestamp + (24.0 * 3600.0); // Predict for the next day

        let next_timestamp_array = Array2::from_elem((1, 1), next_timestamp);
        
        let predicted_demand = model.predict(&next_timestamp_array).get(0).cloned().unwrap_or(0.0);

        let product_name = get_product_name(pool, *product_id).await?.unwrap_or_default();

        optimizations.push(StockOptimization {
            product_id: *product_id,
            product_name,
            predicted_demand,
        });
    }

    Ok(optimizations)
}

async fn get_sales_data(
    pool: &PgPool,
) -> Result<HashMap<i32, Vec<(NaiveDateTime, f32)>>, sqlx::Error> {
    let rows = sqlx::query!(
        "SELECT p.product_id, s.quantity, r.transaction_date
         FROM sales s
         JOIN products p ON s.product_id = p.product_id
         JOIN receipts r ON s.receipt_id = r.receipt_id
         WHERE r.transaction_date IS NOT NULL AND s.quantity IS NOT NULL AND p.product_id IS NOT NULL"
    )
    .fetch_all(pool)
    .await?;

    let mut sales_data: HashMap<i32, Vec<(NaiveDateTime, f32)>> = HashMap::new();
    for row in rows {
        if let (Some(quantity), Some(transaction_date)) = (row.quantity, row.transaction_date) {
            sales_data
                .entry(row.product_id)
                .or_default()
                .push((transaction_date, quantity));
        }
    }

    Ok(sales_data)
}

async fn get_product_name(pool: &PgPool, product_id: i32) -> Result<Option<String>, sqlx::Error> {
    let row = sqlx::query!("SELECT name FROM products WHERE product_id = $1", product_id)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(|r| r.name).flatten())


}
