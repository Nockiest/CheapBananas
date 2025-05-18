use chrono::{NaiveDateTime};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use std::env;
use bigdecimal::BigDecimal;
use uuid::Uuid;
#[derive(Debug, FromRow, Serialize, Deserialize)]
struct Product {
    id: Uuid,
    name: String,
    price: f64,
    product_volume: Option<f64>,
    shop_id: Option<Uuid>,
    date: Option<NaiveDateTime>,
    notes: Option<String>,
    tags: Option<Vec<String>>,
}

struct Shop {
    id: Uuid,
    name: String,
    products: Vec<Product>,
}
async fn get_products(pool: &PgPool) -> Result<Vec<Product>, sqlx::Error> {
    let products: Vec<Product> = sqlx::query_as::<_, Product>(
        "SELECT id, name, price, product_volume, shop_id, date, notes, tags FROM products",
    )
    .fetch_all(pool)
    .await?;
    Ok(products)
}

async fn add_shop(pool: &PgPool, name: &str) -> Result<Uuid, sqlx::Error> {
    let shop = sqlx::query!("INSERT INTO shops (name) VALUES ($1) RETURNING id", name)
        .fetch_one(pool)
        .await?;
    Ok(shop.id)
}

async fn add_product(pool: &PgPool, product: &Product) -> Result<Uuid, sqlx::Error> {
    let product = sqlx::query!(
        "INSERT INTO products (name, price, product_volume, shop_id, date, notes, tags) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id",
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
    let shop_id: Uuid = add_shop(&pool, "Test Shop").await?;
    let test_product: Product = Product {
        id: Uuid::new_v4(),
        name: "Test Product".to_string(),
        price: 10.0,
        product_volume: Some(1.0),
        shop_id: Some(shop_id),
        date: None, // Uncomment and set if needed
        notes: Some("Test notes".to_string()),
        tags: Some(vec!["tag1".to_string(), "tag2".to_string()]),
    };

    add_product(&pool, &test_product).await?;

    let products = get_products(&pool).await?;
    for product in products {
        println!("{:?}", product);
    }

    Ok(())
}
