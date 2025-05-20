use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;

#[derive(Debug, Type, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
#[sqlx(type_name = "TEXT")]
pub enum Unit {
    #[serde(rename = "ks")]
    #[sqlx(rename = "ks")]
    Ks,
    #[serde(rename = "kg")]
    #[sqlx(rename = "kg")]
    Kg,
    #[serde(rename = "l")]
    #[sqlx(rename = "l")]
    L,
}

use std::fmt;

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Unit::Kg => write!(f, "kg"),
            Unit::Ks => write!(f, "ks"),
            Unit::L => write!(f, "l"),
        }
    }
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Product {
    pub id: Uuid,
    pub name: String,
    pub notes: Option<String>,
    pub tags: Option<Vec<String>>,
}

// ...existing code...

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct ProductEntry {
    pub id: Uuid,
    pub product_id: Uuid,
    pub price: f64,
    pub product_volume: Option<f64>,
    pub unit: Unit,
    pub shop_id: Option<Uuid>,
    pub date: Option<NaiveDateTime>,
    pub notes: Option<String>,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Shop {
    pub id: Uuid,
    pub name: String,
    pub notes: Option<String>,
    // Removed products field for DB compatibility
}

// ...existing code...
