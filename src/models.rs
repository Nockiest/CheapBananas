use chrono::NaiveDateTime;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use sqlx::FromRow;

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Product {
    pub id: Uuid,
    pub name: String,
    pub price: f64,
    pub product_volume: Option<f64>,
    pub shop_id: Option<Uuid>,
    pub date: Option<NaiveDateTime>,
    pub notes: Option<String>,
    pub tags: Option<Vec<String>>,
}

pub struct Shop {
    pub id: Uuid,
    pub name: String,
    pub products: Vec<Product>,
}