mod models;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;
use crate::models::{Product, Shop};
async fn get_products(pool: &PgPool) -> Result<Vec<Product>, sqlx::Error> {
    let products: Vec<Product> = sqlx::query_as::<_, Product>(
        "SELECT id, name, price, product_volume, unit, shop_id, date, notes, tags FROM products",
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
        "INSERT INTO products (name, price, product_volume, unit, shop_id, date, notes, tags) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id",
        product.name,
        product.price,
        product.product_volume,
    product.unit.to_string(), // <-- Fix: convert enum to i32
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
        unit: models::Unit::Kg, // <-- Use enum variant
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

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::Executor;
    use uuid::Uuid;
    use dotenv::dotenv;
    use std::env;

    async fn setup_db() -> PgPool {
        dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&database_url).await.expect("Failed to connect to DB");
        // Optionally, clean up tables before each test
        let _ = pool.execute("DELETE FROM products;").await;
        let _ = pool.execute("DELETE FROM shops;").await;
        pool
    }

    #[tokio::test]
    async fn test_add_shop() {
        let pool = setup_db().await;
        let shop_id = add_shop(&pool, "Test Shop").await.expect("Failed to add shop");
        assert!(!shop_id.is_nil(), "Shop ID should not be nil");
    }

    #[tokio::test]
    async fn test_add_product() {
        let pool = setup_db().await;
        let shop_id = add_shop(&pool, "Test Shop").await.expect("Failed to add shop");
        let test_product = Product {
            id: Uuid::new_v4(),
            name: "Test Product".to_string(),
            price: 10.0,
            product_volume: Some(1.0),
            unit: crate::models::Unit::Kg, // <-- Use enum variant
            shop_id: Some(shop_id),
            date: None,
            notes: Some("Test notes".to_string()),
            tags: Some(vec!["tag1".to_string(), "tag2".to_string()]),
        };
        let product_id = add_product(&pool, &test_product).await.expect("Failed to add product");
        assert!(!product_id.is_nil(), "Product ID should not be nil");
        // Optionally, fetch products and check
        let products = get_products(&pool).await.expect("Failed to get products");
        assert!(products.iter().any(|p| p.id == product_id), "Product should be in DB");
    }

    #[tokio::test]
    async fn test_add_product_with_all_units() {
        let pool = setup_db().await;
        let shop_id = add_shop(&pool, "Test Shop").await.expect("Failed to add shop");
        use crate::models::Unit;
        let units = [Unit::Kg, Unit::Ks, Unit::L];
        for &unit in &units {
            let test_product = Product {
                id: Uuid::new_v4(),
                name: format!("Test Product {unit:?}"),
                price: 10.0,
                product_volume: Some(1.0),
                unit,
                shop_id: Some(shop_id),
                date: None,
                notes: Some(format!("Test notes for {unit:?}")),
                tags: Some(vec!["tag1".to_string(), "tag2".to_string()]),
            };
            let product_id = add_product(&pool, &test_product).await.expect("Failed to add product");
            assert!(!product_id.is_nil(), "Product ID should not be nil");
            let products = get_products(&pool).await.expect("Failed to get products");
            assert!(products.iter().any(|p| p.id == product_id && p.unit == unit), "Product with unit {unit:?} should be in DB");
        }
    }
}
