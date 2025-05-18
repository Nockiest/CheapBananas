use chrono::{NaiveDateTime};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use std::env;
use bigdecimal::BigDecimal;
#[derive(Debug, FromRow, Serialize, Deserialize)]
struct Product {
    id: i32,
    name: String,
    price: f64,
    product_volume: Option<f64>,
    shop_id: Option<i32>,
    date: Option<NaiveDateTime>,
    notes: Option<String>,
    tags: Option<Vec<String>>,
}

struct Shop {
    id: i32,
    name: String,
    products: Vec<Product>,
}
async fn get_products(pool: &PgPool) -> Result<Vec<Product>, sqlx::Error> {
    let products: Vec<Product> = sqlx::query_as::<_, Product>(
        "SELECT id, name, price, product_volume, shop_id, date, notes, tags FROM product",
    )
    .fetch_all(pool)
    .await?;
    Ok(products)
}

async fn add_shop(pool: &PgPool, name: &str) -> Result<i32, sqlx::Error> {
    let shop = sqlx::query!("INSERT INTO shop (name) VALUES ($1) RETURNING id", name)
        .fetch_one(pool)
        .await?;
    Ok(shop.id)
}

async fn add_product(pool: &PgPool, product: &Product) -> Result<i32, sqlx::Error> {
    let product = sqlx::query!(
        "INSERT INTO product (name, price, product_volume, shop_id, date, notes, tags) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id",
        product.name,
        product.price,
        product.product_volume,
        product.shop_id,
        product.date,
        product.notes,
        product.tags.as_deref() // <-- Fix: convert Option<Vec<String>> to Option<&[String]>
    )
    .fetch_one(pool)
    .await?;
    Ok(product.id)
}
// Example main with tokio runtime
#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    dotenv::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url).await?;
    add_shop(&pool, "Test Shop").await?;

    let test_product: Product = Product {
        id: 1,
        name: "Test Product".to_string(),
        price: 10.0,
        product_volume: Some(1.0),
        shop_id: Some(1),
        date: None, // Uncomment and set if needed
        notes: Some("Test notes".to_string()),
        tags: None//Some(vec!["tag1".to_string(), "tag2".to_string()]),
    };

    add_product(&pool, &test_product).await?;

    let products = get_products(&pool).await?;
    for product in products {
        println!("{:?}", product);
    }

    Ok(())
}
