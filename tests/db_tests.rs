use backend::db::{add_product, add_shop, delete_product, delete_shop, get_products_filtered, update_product};
use backend::models::{Product, Unit};
use sqlx::PgPool;
use uuid::Uuid;
use dotenv::dotenv;
use sqlx::Executor;
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
    let shop_id = add_shop(&pool, "Test Shop").await.expect("Failed to add shop");
    let test_product = Product {
        id: Uuid::new_v4(),
        name: "Test Product".to_string(),
        price: 10.0,
        product_volume: Some(1.0),
        unit: Unit::Kg,
        shop_id: Some(shop_id),
        date: None,
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
async fn test_add_product_with_all_units() {
    let pool = setup_db().await;
    let shop_id = add_shop(&pool, "Test Shop").await.expect("Failed to add shop");
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
        let products = backend::get_products(&pool).await.expect("Failed to get products");
        assert!(products.iter().any(|p| p.id == product_id && p.unit == unit), "Product with unit {unit:?} should be in DB");
    }
}

#[tokio::test]
#[serial_test::serial]
async fn test_delete_product_and_shop() {
    let pool = setup_db().await;
    let shop_id = add_shop(&pool, "Shop to Delete").await.expect("Failed to add shop");
    let test_product = Product {
        id: Uuid::new_v4(),
        name: "Product to Delete".to_string(),
        price: 5.0,
        product_volume: Some(2.0),
        unit: Unit::Kg,
        shop_id: Some(shop_id),
        date: None,
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
async fn test_get_products_filtered() {
    let pool = setup_db().await;
    let shop_id = add_shop(&pool, "Filter Shop").await.expect("Failed to add shop");
    let products = vec![
        Product {
            id: Uuid::new_v4(),
            name: "Apple".to_string(),
            price: 1.0,
            product_volume: Some(1.0),
            unit: Unit::Kg,
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
            unit: Unit::Kg,
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
            unit: Unit::L,
            shop_id: Some(shop_id),
            date: None,
            notes: Some("Whole milk".to_string()),
            tags: Some(vec!["dairy".to_string()]),
        },
    ];
    for p in &products {
        add_product(&pool, p).await.expect("Failed to add product");
    }
    use backend::db::ProductFilter;
    let filtered = get_products_filtered(
        &pool,
        ProductFilter {
            name: Some("Apple"),
            ..Default::default()
        },
    )
    .await
    .expect("Failed to filter by name");
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].name, "Apple");
    let filtered = get_products_filtered(
        &pool,
        ProductFilter {
            unit: Some("l"),
            ..Default::default()
        },
    )
    .await
    .expect("Failed to filter by unit");
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].name, "Milk");
    let filtered = get_products_filtered(
        &pool,
        ProductFilter {
            min_price: Some(1.5),
            max_price: Some(3.0),
            ..Default::default()
        },
    )
    .await
    .expect("Failed to filter by price range");
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
        unit: Unit::Kg,
        shop_id: Some(shop_id),
        date: None,
        notes: Some("Initial notes".to_string()),
        tags: Some(vec!["tag1".to_string()]),
    };
    let product_id = add_product(&pool, &test_product).await.expect("Failed to add product");
    let affected = update_product(&pool, product_id, None, None, None, None, None, None, None, None).await.expect("Update nothing");
    assert_eq!(affected, 0);
    let affected = update_product(
        &pool, product_id,
        Some("UpdatedName"), Some(20.0), Some(2.0), Some("l"), Some(shop_id), None, Some("Updated notes"), Some(&["tag2".to_string()]),
    ).await.expect("Update valid");
    assert_eq!(affected, 1);
    let updated = backend::get_products(&pool).await.expect("Get products").into_iter().find(|p| p.id == product_id).unwrap();
    assert_eq!(updated.name, "UpdatedName");
    assert_eq!(updated.price, 20.0);
    assert_eq!(updated.product_volume, Some(2.0));
    assert_eq!(updated.unit.to_string(), "l");
    assert_eq!(updated.notes.as_deref(), Some("Updated notes"));
    assert_eq!(updated.tags, Some(vec!["tag2".to_string()]));
    let affected = update_product(&pool, product_id, Some(""), None, None, None, None, None, None, None).await.expect("Update empty name");
    assert_eq!(affected, 1);
    let updated = backend::get_products(&pool).await.expect("Get products").into_iter().find(|p| p.id == product_id).unwrap();
    assert_eq!(updated.name, "");
    let err = update_product(&pool, product_id, None, Some(10001.0), None, None, None, None, None, None).await;
    assert!(err.is_err(), "Should error for price out of bounds");
    let err = update_product(&pool, product_id, None, None, Some(10001.0), None, None, None, None, None).await;
    assert!(err.is_err(), "Should error for product_volume out of bounds");
    let fake_shop = Uuid::new_v4();
    let err = update_product(&pool, product_id, None, None, None, None, Some(fake_shop), None, None, None).await;
    assert!(err.is_err(), "Should error for non-existent shop_id");
    let long_name = "a".repeat(255);
    let long_notes = "b".repeat(1000);
    let affected = update_product(&pool, product_id, Some(&long_name), None, None, None, None, None, Some(&long_notes), None).await.expect("Update long fields");
    assert_eq!(affected, 1);
    let updated = backend::get_products(&pool).await.expect("Get products").into_iter().find(|p| p.id == product_id).unwrap();
    assert_eq!(updated.name, long_name);
    assert_eq!(updated.notes.as_deref(), Some(&long_notes[..]));
}
