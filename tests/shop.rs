// Tests for Shop-related DB logic
mod common;
use common::setup_db;
use backend::{db::*, Unit, Product};
use backend::models::{ProductEntry, Shop};
use uuid::Uuid;
use chrono::Utc;
// ...existing code for Shop tests will be inserted here...
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
        product_volume: -5.0,
        unit: Unit::Kg,
        shop_name: None,
        date: Some(Utc::now().naive_utc()),
        notes: Some("Negative volume".to_string()),
    };
    let result = backend::add_product_entry(&pool, &entry).await;
    assert!(result.is_err(), "Should not allow negative product volume");
}
