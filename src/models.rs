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

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct ProductEntry {
    pub id: Uuid,
    pub product_id: Uuid,
    pub price: f64,
    pub product_volume:f64,
    pub unit: Unit,
    pub shop_name: Option<String>, // Added field to include shop name
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

// Filter struct for get_products_filtered
#[derive(Default, Debug, Deserialize)]
pub struct ProductFilter<'a> {
    // Product fields
    pub id: Option<Uuid>,
    pub name: Option<&'a str>,
    pub notes: Option<&'a str>,
    pub tag: Option<&'a str>,
    // ProductEntry fields
    pub product_id: Option<Uuid>,
    pub min_price: Option<f64>,
    pub max_price: Option<f64>,
    pub product_volume: Option<f64>,
    pub unit: Option<&'a str>,
    pub shop_name: Option<String>,
    pub date: Option<chrono::NaiveDateTime>,
}

#[derive(Default, Debug, Deserialize)]
pub struct ShopFilter<'a> {
    pub id: Option<Uuid>,
    pub name: Option<&'a str>,
    pub notes: Option<&'a str>,
}

pub type ProductEntryFilter<'a> = ProductFilter<'a>;