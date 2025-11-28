use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Customer {
    pub customer_id: i32,
    pub name: String,
    pub email: String,
    pub registered_on: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Product {
    pub product_id: i32,
    pub product_code: String,
    #[sqlx(rename = "productname")]
    pub name: String,
    pub department: String,
    pub category: String,
    #[sqlx(rename = "sellingprice")]
    pub selling_price: f32,
    pub current_stock: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Receipt {
    pub receipt_id: i32,
    pub receipt_no: i32,
    #[sqlx(rename = "date")]
    pub transaction_date: Option<NaiveDateTime>,
    #[sqlx(rename = "customer")]
    pub customer_id: Option<i32>,
    #[sqlx(rename = "total_cost_incl")]
    pub total_amount: f32,
    pub payment_channel: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Sale {
    pub sale_id: i32,
    pub receipt_id: i32,
    pub product_id: i32,
    pub quantity: f32,
    #[sqlx(rename = "sellingprice")]
    pub selling_price: f32,
    #[sqlx(rename = "totalsales")]
    pub total_sale: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MarketTrend {
    pub id: i32,
    pub category: String,
    pub dish: String,
    pub source: String,
    pub popularity_score: i32,
    pub insight: String,
    pub trend_direction: Option<String>,
    pub last_updated: Option<NaiveDateTime>,
}

