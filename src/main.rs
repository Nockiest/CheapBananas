mod models;
mod db;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;
use crate::models::{Product, Shop};
use crate::db::{add_shop, add_product};

async fn get_products(pool: &PgPool) -> Result<Vec<Product>, sqlx::Error> {
    let products: Vec<Product> = sqlx::query_as::<_, Product>(
        "SELECT id, name, price, product_volume, unit, shop_id, date, notes, tags FROM products",
    )
    .fetch_all(pool)
    .await?;
    Ok(products)
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
#[serial_test::serial]
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

    #[tokio::test]
    async fn test_delete_product_and_shop() {
        let pool = setup_db().await;
        println!("Database setup complete.");

        // Add a shop
        let shop_id = add_shop(&pool, "Shop to Delete").await.expect("Failed to add shop");
        println!("Added shop with id: {}", shop_id);

        // Add a product
        let test_product = Product {
            id: Uuid::new_v4(),
            name: "Product to Delete".to_string(),
            price: 5.0,
            product_volume: Some(2.0),
            unit: crate::models::Unit::Kg,
            shop_id: Some(shop_id),
            date: None,
            notes: Some("To be deleted".to_string()),
            tags: Some(vec!["delete".to_string()]),
        };
        println!("Created test product: {:?}", test_product);

        let product_id: Uuid = add_product(&pool, &test_product).await.expect("Failed to add product");
        println!("Added product with id: {}", product_id);

        // Delete the product
        let deleted: u64 = crate::db::delete_product(&pool, product_id).await.expect("Failed to delete product");
        println!("Deleted {} product(s) with id: {}", deleted, product_id);

        let products = get_products(&pool).await.expect("Failed to get products");
        println!("Products after deletion: {:?}", products);

        assert!(!products.iter().any(|p| p.id == product_id), "Product should be deleted");

        // Delete the shop
        let deleted_shop = crate::db::delete_shop(&pool, shop_id).await.expect("Failed to delete shop");
        println!("Deleted {} shop(s) with id: {}", deleted_shop, shop_id);

        // Optionally, check shop is gone (if you have a get_shops function)
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_products_filtered() {
        let pool = setup_db().await;
        let shop_id = add_shop(&pool, "Filter Shop").await.expect("Failed to add shop");
        // Insert products with different names, units, and prices
        let products = vec![
            Product {
                id: Uuid::new_v4(),
                name: "Apple".to_string(),
                price: 1.0,
                product_volume: Some(1.0),
                unit: crate::models::Unit::Kg,
                shop_id: Some(shop_id),
                date: None,
                notes: Some("Fresh apples".to_string()),
                tags: Some(vec!["fruit".to_string()]),
            },
            Product {
                id: Uuid::new_v4(),
                name: "Banana".to_string(),
                price: 2.0,
                product_volume: Some(1.0),
                unit: crate::models::Unit::Kg,
                shop_id: Some(shop_id),
                date: None,
                notes: Some("Yellow bananas".to_string()),
                tags: Some(vec!["fruit".to_string()]),
            },
            Product {
                id: Uuid::new_v4(),
                name: "Milk".to_string(),
                price: 3.0,
                product_volume: Some(1.0),
                unit: crate::models::Unit::L,
                shop_id: Some(shop_id),
                date: None,
                notes: Some("Whole milk".to_string()),
                tags: Some(vec!["dairy".to_string()]),
            },
        ];
        for p in &products {
            add_product(&pool, p).await.expect("Failed to add product");
        }
        // Filter by name
        let filtered = crate::db::get_products_filtered(&pool, Some("Apple"), None, None, None)
            .await.expect("Failed to filter by name");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "Apple");
        // Filter by unit
        let filtered = crate::db::get_products_filtered(&pool, None, Some("l"), None, None)
            .await.expect("Failed to filter by unit");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "Milk");
        // Filter by price range
        let filtered = crate::db::get_products_filtered(&pool, None, None, Some(1.5), Some(3.0))
            .await.expect("Failed to filter by price range");
        assert_eq!(filtered.len(), 2);
        let names: Vec<_> = filtered.iter().map(|p| p.name.as_str()).collect();
        assert!(names.contains(&"Banana"));
        assert!(names.contains(&"Milk"));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_update_product_edge_cases() {
        let pool = setup_db().await;
        let shop_id = add_shop(&pool, "Update Shop").await.expect("Failed to add shop");
        let test_product = Product {
            id: Uuid::new_v4(),
            name: "EdgeCaseProduct".to_string(),
            price: 10.0,
            product_volume: Some(1.0),
            unit: crate::models::Unit::Kg,
            shop_id: Some(shop_id),
            date: None,
            notes: Some("Initial notes".to_string()),
            tags: Some(vec!["tag1".to_string()]),
        };
        let product_id = add_product(&pool, &test_product).await.expect("Failed to add product");

        // 1. Update nothing (should return 0 rows affected)
        let affected = crate::db::update_product(&pool, product_id, None, None, None, None, None, None, None, None).await.expect("Update nothing");
        assert_eq!(affected, 0);

        // 2. Update with valid data
        let affected = crate::db::update_product(
            &pool, product_id,
            Some("UpdatedName"), Some(20.0), Some(2.0), Some("l"), Some(shop_id), None, Some("Updated notes"), Some(&["tag2".to_string()]),
        ).await.expect("Update valid");
        assert_eq!(affected, 1);
        let updated = get_products(&pool).await.expect("Get products").into_iter().find(|p| p.id == product_id).unwrap();
        assert_eq!(updated.name, "UpdatedName");
        assert_eq!(updated.price, 20.0);
        assert_eq!(updated.product_volume, Some(2.0));
        assert_eq!(updated.unit.to_string(), "l");
        assert_eq!(updated.notes.as_deref(), Some("Updated notes"));
        assert_eq!(updated.tags, Some(vec!["tag2".to_string()]));

        // 3. Update with empty string for name (should succeed)
        let affected = crate::db::update_product(&pool, product_id, Some(""), None, None, None, None, None, None, None).await.expect("Update empty name");
        assert_eq!(affected, 1);
        let updated = get_products(&pool).await.expect("Get products").into_iter().find(|p| p.id == product_id).unwrap();
        assert_eq!(updated.name, "");

        // 4. Update with too high price (should error)
        let err = crate::db::update_product(&pool, product_id, None, Some(10001.0), None, None, None, None, None, None).await;
        assert!(err.is_err(), "Should error for price out of bounds");

        // 5. Update with too high product_volume (should error)
        let err = crate::db::update_product(&pool, product_id, None, None, Some(10001.0), None, None, None, None, None).await;
        assert!(err.is_err(), "Should error for product_volume out of bounds");

        // 6. Update with non-existent shop_id (should error)
        let fake_shop = Uuid::new_v4();
        let err = crate::db::update_product(&pool, product_id, None, None, None, None, Some(fake_shop), None, None, None).await;
        assert!(err.is_err(), "Should error for non-existent shop_id");

        // 7. Update with long name and notes (should succeed)
        let long_name = "a".repeat(255);
        let long_notes = "b".repeat(1000);
        let affected = crate::db::update_product(&pool, product_id, Some(&long_name), None, None, None, None, None, Some(&long_notes), None).await.expect("Update long fields");
        assert_eq!(affected, 1);
        let updated = get_products(&pool).await.expect("Get products").into_iter().find(|p| p.id == product_id).unwrap();
        assert_eq!(updated.name, long_name);
        assert_eq!(updated.notes.as_deref(), Some(&long_notes[..]));
    }
}
