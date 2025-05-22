use backend::db::{add_product, add_shop, delete_product, delete_shop, update_product};
use backend::models::{Product, ProductEntry, Unit, Shop};
use uuid::Uuid;
use chrono::Utc;
mod common;
use common::setup_db;


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
async fn test_sanitize_underscores_to_empty_on_required_fields() {
   
use backend::utils::text_utils::sanitize_underscores_to_empty; 
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
