use backend::db::{add_product, add_shop, delete_product, delete_shop, update_product};
use backend::models::{Product, ProductEntry, Unit};
use sqlx::PgPool;
use uuid::Uuid;
use dotenv::dotenv;
use sqlx::Executor;
use chrono::Utc;
async fn setup_db() -> PgPool {
    dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url).await.expect("Failed to connect to DB");
    let _ = pool.execute("DELETE FROM products;").await;
    let _ = pool.execute("DELETE FROM shops;").await;
    pool
}

#[tokio::test]
#[serial_test::serial]
async fn test_add_shop() {
    let pool = setup_db().await;
    let shop_id = add_shop(&pool, "Test Shop").await.expect("Failed to add shop");
    assert!(!shop_id.is_nil(), "Shop ID should not be nil");
}

#[tokio::test]
#[serial_test::serial]
async fn test_add_product() {
    let pool = setup_db().await;
    let _shop_id = add_shop(&pool, "Test Shop").await.expect("Failed to add shop");
    let test_product = Product {
        id: Uuid::new_v4(),
        name: "Test Product".to_string(),
        notes: Some("Test notes".to_string()),
        tags: Some(vec!["tag1".to_string(), "tag2".to_string()]),
    };
    let product_id = add_product(&pool, &test_product).await.expect("Failed to add product");
    assert!(!product_id.is_nil(), "Product ID should not be nil");
    let products = backend::get_products(&pool).await.expect("Failed to get products");
    assert!(products.iter().any(|p| p.id == product_id), "Product should be in DB");
}

#[tokio::test]
#[serial_test::serial]
async fn test_delete_product_and_shop() {
    let pool = setup_db().await;
    let shop_id = add_shop(&pool, "Shop to Delete").await.expect("Failed to add shop");
    let test_product = Product {
        id: Uuid::new_v4(),
        name: "Product to Delete".to_string(),
        notes: Some("To be deleted".to_string()),
        tags: Some(vec!["delete".to_string()]),
    };
    let product_id: Uuid = add_product(&pool, &test_product).await.expect("Failed to add product");
    let deleted: u64 = delete_product(&pool, product_id).await.expect("Failed to delete product");
    assert_eq!(deleted, 1, "Should delete one product");
    let products = backend::get_products(&pool).await.expect("Failed to get products");
    assert!(!products.iter().any(|p| p.id == product_id), "Product should be deleted");
    let deleted_shop = delete_shop(&pool, shop_id).await.expect("Failed to delete shop");
    assert_eq!(deleted_shop, 1, "Should delete one shop");
}

#[tokio::test]
#[serial_test::serial]
async fn test_update_product_edge_cases() {
    let pool = setup_db().await;
    let _shop_id = add_shop(&pool, "Update Shop").await.expect("Failed to add shop");
    let test_product = Product {
        id: Uuid::new_v4(),
        name: "EdgeCaseProduct".to_string(),
        notes: Some("Initial notes".to_string()),
        tags: Some(vec!["tag1".to_string()]),
    };
    let product_id = add_product(&pool, &test_product).await.expect("Failed to add product");
    // Update nothing
    let affected = update_product(&pool, product_id, None, None, None, None, None, None, None, None).await.expect("Update nothing");
    assert_eq!(affected, 0);
    // Update name, notes, tags
    let affected = update_product(
        &pool, product_id,
        Some("UpdatedName"), None, None, None, None, None, Some("Updated notes"), Some(&["tag2".to_string()]),
    ).await.expect("Update valid");
    assert_eq!(affected, 1);
    let updated = backend::get_products(&pool).await.expect("Get products").into_iter().find(|p| p.id == product_id).unwrap();
    assert_eq!(updated.name, "UpdatedName");
    assert_eq!(updated.notes.as_deref(), Some("Updated notes"));
    assert_eq!(updated.tags, Some(vec!["tag2".to_string()]));
    // Update name to empty string
    let affected = update_product(&pool, product_id, Some(""), None, None, None, None, None, None, None).await.expect("Update empty name");
    assert_eq!(affected, 1);
    let updated = backend::get_products(&pool).await.expect("Get products").into_iter().find(|p| p.id == product_id).unwrap();
    assert_eq!(updated.name, "");
    // Update with long fields
    let long_name = "a".repeat(255);
    let long_notes = "b".repeat(1000);
    let affected = update_product(&pool, product_id, Some(&long_name), None, None, None, None, None, Some(&long_notes), None).await.expect("Update long fields");
    assert_eq!(affected, 1);
    let updated = backend::get_products(&pool).await.expect("Get products").into_iter().find(|p| p.id == product_id).unwrap();
    assert_eq!(updated.name, long_name);
    assert_eq!(updated.notes.as_deref(), Some(&long_notes[..]));
}

#[tokio::test]
#[serial_test::serial]
async fn test_add_product_entry() {
    let pool = setup_db().await;
    // Add a shop and a product first
    let shop_id = add_shop(&pool, "Entry Shop").await.expect("Failed to add shop");
    let product = Product {
        id: Uuid::new_v4(),
        name: "Entry Product".to_string(),
        notes: Some("Entry notes".to_string()),
        tags: Some(vec!["entry".to_string()]),
    };
    let product_id = add_product(&pool, &product).await.expect("Failed to add product");
    // Create ProductEntry
    let entry = ProductEntry {
        id: Uuid::new_v4(),
        product_id,
        price: 2.99,
        product_volume: Some(1.0),
        unit: Unit::Kg,
        shop_id: Some(shop_id),
        date: Some(Utc::now().naive_utc()),
        notes: Some("Entry for test".to_string()),
    };
    // Insert entry (assumes add_product_entry exists)
    let entry_id = backend::add_product_entry(&pool, &entry).await.expect("Failed to add product entry");
    assert_eq!(entry_id, entry.id);
    // Optionally, fetch and check
    // let entries = backend::get_product_entries(&pool).await.expect("Failed to get entries");
    // assert!(entries.iter().any(|e| e.id == entry_id));
}
