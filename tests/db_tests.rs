use backend::db::{add_product, add_shop, delete_product, delete_shop, update_product};
use backend::models::{Product, ProductEntry, Unit, Shop};
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
    let shop = Shop {
        id: Uuid::new_v4(),
        name: "Test Shop".to_string(),
        notes: None,
    };
    let shop_id = add_shop(&pool, &shop).await.expect("Failed to add shop");
    assert!(!shop_id.is_nil(), "Shop ID should not be nil");
}

#[tokio::test]
#[serial_test::serial]
async fn test_add_product() {
    let pool = setup_db().await;
    let shop = Shop {
        id: Uuid::new_v4(),
        name: "Test Shop".to_string(),
        notes: None,
    };
    let _shop_id = add_shop(&pool, &shop).await.expect("Failed to add shop");
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
    let shop = Shop {
        id: Uuid::new_v4(),
        name: "Shop to Delete".to_string(),
        notes: None,
    };
    let shop_id = add_shop(&pool, &shop).await.expect("Failed to add shop");
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
    let shop = Shop {
        id: Uuid::new_v4(),
        name: "Update Shop".to_string(),
        notes: None,
    };
    let _shop_id = add_shop(&pool, &shop).await.expect("Failed to add shop");
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
    let shop = Shop {
        id: Uuid::new_v4(),
        name: "Entry Shop".to_string(),
        notes: None,
    };
    let shop_id = add_shop(&pool, &shop).await.expect("Failed to add shop");
    // Add a shop and a product first
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
    // Insert entry
    let entry_id = backend::add_product_entry(&pool, &entry).await.expect("Failed to add product entry");
    assert_eq!(entry_id, entry.id);
    // Optionally, fetch and check (if get_product_entries exists)
    // let entries = backend::get_product_entries(&pool).await.expect("Failed to get entries");
    // assert!(entries.iter().any(|e| e.id == entry_id));
}
#[tokio::test]
#[serial_test::serial]
async fn test_get_products_filtered_endpoint() {
    use axum::body::Body;
    use axum::http::{Request};
    use tower::ServiceExt;
    use std::sync::Arc;
    use backend::app::build_app_router;

    let pool = setup_db().await;
    let shared_pool = Arc::new(pool.clone());
    let app = build_app_router(shared_pool.clone());

    // Add products
    let shop = Shop {
        id: Uuid::new_v4(),
        name: "Filter Shop".to_string(),
        notes: None,
    };
    let _ = add_shop(&pool, &shop).await.expect("Failed to add shop");
    let products = vec![
        Product {
            id: Uuid::new_v4(),
            name: "Apple".to_string(),
            notes: Some("Fresh apples".to_string()),
            tags: Some(vec!["fruit".to_string()]),
        },
        Product {
            id: Uuid::new_v4(),
            name: "Banana".to_string(),
            notes: Some("Yellow bananas".to_string()),
            tags: Some(vec!["fruit".to_string()]),
        },
        Product {
            id: Uuid::new_v4(),
            name: "Milk".to_string(),
            notes: Some("Whole milk".to_string()),
            tags: Some(vec!["dairy".to_string()]),
        },
    ];
    for p in &products {
        add_product(&pool, p).await.expect("Failed to add product");
    }

    // Filter by name
    let response = app
        .clone()
        .oneshot(Request::builder()
            .uri("/products/filter?name=Apple")
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();
    println!("[TEST] Response status for name=Apple: {:?}", response.status());
    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    println!("[TEST] Response body for name=Apple: {}", String::from_utf8_lossy(&body));
    let filtered: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    println!("[TEST] Filtered products for name=Apple: {:?}", filtered);
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0]["name"], "Apple");

    // Filter by tag
    let response = app
        .oneshot(Request::builder()
            .uri("/products/filter?tag=fruit")
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();
    println!("[TEST] Response status for tag=fruit: {:?}", response.status());
    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    println!("[TEST] Response body for tag=fruit: {}", String::from_utf8_lossy(&body));
    let filtered: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    println!("[TEST] Filtered products for tag=fruit: {:?}", filtered);
    assert_eq!(filtered.len(), 2);
    let names: Vec<_> = filtered.iter().map(|p| p["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"Apple"));
    assert!(names.contains(&"Banana"));
}

#[tokio::test]
#[serial_test::serial]
async fn test_get_products_filtered_all_fields() {
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;
    use std::sync::Arc;
    use backend::app::build_app_router;

    let pool = setup_db().await;
    let shared_pool = Arc::new(pool.clone());
    let app = build_app_router(shared_pool.clone());

    // Add a shop and a product entry for future filter support
    let shop = Shop {
        id: Uuid::new_v4(),
        name: "AllFields Shop".to_string(),
        notes: None,
    };
    let shop_id = add_shop(&pool, &shop).await.expect("Failed to add shop");
    let product = Product {
        id: Uuid::new_v4(),
        name: "TestProductAllFields".to_string(),
        notes: Some("Notes for all fields".to_string()),
        tags: Some(vec!["allfields".to_string()]),
    };
    add_product(&pool, &product).await.expect("Failed to add product");
    // Filtering by name (supported)
    let response = app
        .clone()
        .oneshot(Request::builder()
            .uri("/products/filter?name=TestProductAllFields")
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();
    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let filtered: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0]["name"], "TestProductAllFields");
    // Filtering by notes (supported)
    let response = app
        .clone()
        .oneshot(Request::builder()
            .uri("/products/filter?notes=Notes%20for%20all%20fields")
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();
    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let filtered: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0]["notes"], "Notes for all fields");
    // Filtering by tag (supported)
    let response = app
        .clone()
        .oneshot(Request::builder()
            .uri("/products/filter?tag=allfields")
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();
    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let filtered: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0]["name"], "TestProductAllFields");
    // Filtering by unit, shop_id, min_price, max_price, date (not supported in current schema)
    // These should return all products or be ignored by the filter logic
    let response = app
        .clone()
        .oneshot(Request::builder()
            .uri("/products/filter?unit=kg&shop_id=".to_owned() + &shop_id.to_string() + "&min_price=1.0&max_price=10.0&date=2024-01-01")
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();
    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let filtered: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    // Since these fields are not supported, the filter should not restrict results
    assert!(filtered.iter().any(|p| p["name"] == "TestProductAllFields"));
    // When schema supports these fields, update this test to check for correct filtering
}

#[tokio::test]
#[serial_test::serial]
async fn test_get_product_entries_filtered_endpoint() {
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;
    use std::sync::Arc;
    use backend::app::build_app_router;
    use backend::models::Unit;
    use chrono::Utc;

    let pool = setup_db().await;
    let shared_pool = Arc::new(pool.clone());
    let app = build_app_router(shared_pool.clone());

    // Add a shop and a product
    let shop = Shop {
        id: Uuid::new_v4(),
        name: "EntryFilter Shop".to_string(),
        notes: None,
    };
    let shop_id = add_shop(&pool, &shop).await.expect("Failed to add shop");
    let product = Product {
        id: Uuid::new_v4(),
        name: "EntryFilterProduct".to_string(),
        notes: Some("Entry filter notes".to_string()),
        tags: Some(vec!["entryfilter".to_string()]),
    };
    let product_id = add_product(&pool, &product).await.expect("Failed to add product");
    // Add product entries
    let entry1 = ProductEntry {
        id: Uuid::new_v4(),
        product_id,
        price: 5.0,
        product_volume: Some(1.0),
        unit: Unit::Kg,
        shop_id: Some(shop_id),
        date: Some(Utc::now().naive_utc()),
        notes: Some("Fresh batch".to_string()),
    };
    let entry2 = ProductEntry {
        id: Uuid::new_v4(),
        product_id,
        price: 10.0,
        product_volume: Some(2.0),
        unit: Unit::Kg,
        shop_id: Some(shop_id),
        date: Some(Utc::now().naive_utc()),
        notes: Some("Old batch".to_string()),
    };
    backend::add_product_entry(&pool, &entry1).await.expect("Failed to add entry1");
    backend::add_product_entry(&pool, &entry2).await.expect("Failed to add entry2");

    // Filter by product_id
    let response = app
        .clone()
        .oneshot(Request::builder()
            .uri(&format!("/product-entries/filter?product_id={}", product_id))
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();
    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let filtered: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert_eq!(filtered.len(), 2);
    // Filter by min_price
    let response = app
        .clone()
        .oneshot(Request::builder()
            .uri(&format!("/product-entries/filter?product_id={}&min_price=6.0", product_id))
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();
    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let filtered: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0]["price"], 10.0);
    // Filter by notes
    let response = app
        .clone()
        .oneshot(Request::builder()
            .uri(&format!("/product-entries/filter?product_id={}&notes=Fresh%20batch", product_id))
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();
    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let filtered: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0]["notes"], "Fresh batch");
}

#[tokio::test]
#[serial_test::serial]
async fn test_get_shops_filtered_endpoint() {
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;
    use std::sync::Arc;
    use backend::app::build_app_router;

    let pool = setup_db().await;
    let shared_pool = Arc::new(pool.clone());
    let app = build_app_router(shared_pool.clone());

    // Add shops
    let shop1 = Shop {
        id: Uuid::new_v4(),
        name: "Tesco".to_string(),
        notes: None,
    };
    let _shop1_id = add_shop(&pool, &shop1).await.expect("Failed to add shop");
    let shop2 = Shop {
        id: Uuid::new_v4(),
        name: "Lidl".to_string(),
        notes: None,
    };
    let shop2_id = add_shop(&pool, &shop2).await.expect("Failed to add shop");
    let shop3 = Shop {
        id: Uuid::new_v4(),
        name: "Tesco Express".to_string(),
        notes: None,
    };
    let shop3_id = add_shop(&pool, &shop3).await.expect("Failed to add shop");
    // Update notes for shop3
    sqlx::query!("UPDATE shops SET notes = $1 WHERE id = $2", Some("small"), shop3_id)
        .execute(&pool).await.expect("Failed to update notes");

    // Filter by name
    println!("[TEST] Requesting /shops/filter?name=Tesco");
    let response = app
        .clone()
        .oneshot(Request::builder()
            .uri("/shops/filter?name=Tesco")
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();
    println!("[TEST] Response status for name=Tesco: {:?}", response.status());
    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    println!("[TEST] Response body for name=Tesco: {}", String::from_utf8_lossy(&body));
    let filtered: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert!(filtered.iter().any(|s| s["name"] == "Tesco"));
    // assert!(filtered.iter().any(|s| s["name"] == "Tesco Express"));

    // Filter by id
    println!("[TEST] Requesting /shops/filter?id={}", shop2_id);
    let response = app
        .clone()
        .oneshot(Request::builder()
            .uri(&format!("/shops/filter?id={}", shop2_id))
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();
    println!("[TEST] Response status for id={}: {:?}", shop2_id, response.status());
    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    println!("[TEST] Response body for id={}: {}", shop2_id, String::from_utf8_lossy(&body));
    let filtered: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0]["name"], "Lidl");

    // Filter by notes
    println!("[TEST] Requesting /shops/filter?notes=small");
    let response = app
        .oneshot(Request::builder()
            .uri("/shops/filter?notes=small")
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();
    println!("[TEST] Response status for notes=small: {:?}", response.status());
    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    println!("[TEST] Response body for notes=small: {}", String::from_utf8_lossy(&body));
    let filtered: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0]["name"], "Tesco Express");

    // --- Product volume must be positive edge case ---
    // Add a shop and a product
    let shop = Shop {
        id: Uuid::new_v4(),
        name: "Volume Shop".to_string(),
        notes: None,
    };
    let shop_id = add_shop(&pool, &shop).await.expect("Failed to add shop");
    let product = Product {
        id: Uuid::new_v4(),
        name: "Volume Product".to_string(),
        notes: Some("Test volume".to_string()),
        tags: None,
    };
    let product_id = add_product(&pool, &product).await.expect("Failed to add product");
    // Try to add a product entry with negative volume
    let entry = ProductEntry {
        id: Uuid::new_v4(),
        product_id,
        price: 1.0,
        product_volume: Some(-5.0),
        unit: Unit::Kg,
        shop_id: Some(shop_id),
        date: Some(Utc::now().naive_utc()),
        notes: Some("Negative volume".to_string()),
    };
    let result = backend::add_product_entry(&pool, &entry).await;
    assert!(result.is_err(), "Should not allow negative product volume");
}

#[tokio::test]
#[serial_test::serial]
async fn test_sanitize_underscores_to_empty_on_required_fields() {
    use backend::db::sanitize_underscores_to_empty;
    use serde_json::json;
    use backend::models::{Product, ProductEntry};
    let pool = setup_db().await;
    let shop = Shop {
        id: Uuid::new_v4(),
        name: "TestShop".to_string(),
        notes: None,
    };
    let shop_id = add_shop(&pool, &shop).await.expect("Failed to add shop");
    // Try to add a product with name = "_"
    let product_val = json!({
        "id": Uuid::new_v4(),
        "name": "_",
        "notes": "Some notes",
        "tags": ["tag1"]
    });
    println!("[TEST] Trying to add product with name = '_'");
    let sanitized = sanitize_underscores_to_empty(product_val);
    println!("[TEST] Sanitized product value: {}", sanitized);
    let product: Product = serde_json::from_value(sanitized).unwrap();
    let res = add_product(&pool, &product).await;
    println!("[TEST] Result for product with name = '_': {:?}", res);
    assert!(res.is_err(), "Should not allow product with name as only underscores");

    // Add a valid product to get a product_id for the entry
    let valid_product = Product {
        id: Uuid::new_v4(),
        name: "Valid Product".to_string(),
        notes: Some("Valid notes".to_string()),
        tags: Some(vec!["tag2".to_string()]),
    };
    let product_id = add_product(&pool, &valid_product).await.expect("Failed to add valid product");

    // Try to add a product entry with product_volume = "_" (should be None/empty)
    println!("[TEST] Trying to add product entry with product_volume = '_'");
    let entry_val = json!({
        "id": Uuid::new_v4(),
        "product_id": product_id,
        "price": 1.0,
        "product_volume": "_",
        "unit": "kg",
        "shop_id": shop_id,
        "date": null,
        "notes": "_"
    });
    let sanitized_entry = sanitize_underscores_to_empty(entry_val);
    println!("[TEST] Sanitized product entry value: {}", sanitized_entry);
    let entry: Result<ProductEntry, _> = serde_json::from_value(sanitized_entry);
    println!("[TEST] Deserialization result for product entry with product_volume = '_': {:?}", entry);
    assert!(entry.is_ok(), "Sanitization should allow deserialization with product_volume as None");
    // Now test required field 'unit' as '_'
    println!("[TEST] Trying to add product entry with unit = '_'");
    let entry_val2 = json!({
        "id": Uuid::new_v4(),
        "product_id": product_id,
        "price": 1.0,
        "product_volume": 1.0,
        "unit": "_",
        "shop_id": shop_id,
        "date": null,
        "notes": "Some notes"
    });
    let sanitized_entry2 = sanitize_underscores_to_empty(entry_val2);
    println!("[TEST] Sanitized product entry value (unit = '_'): {}", sanitized_entry2);
    let entry2: Result<ProductEntry, _> = serde_json::from_value(sanitized_entry2);
    println!("[TEST] Deserialization result for product entry with unit = '_': {:?}", entry2);
    assert!(entry2.is_err(), "Should not allow product entry with unit as only underscores");
}
