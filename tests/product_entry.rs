// Tests for ProductEntry-related DB logic

use backend::db::*;
use backend::models::{Product, ProductEntry, Unit, Shop};
use uuid::Uuid;
use chrono::Utc;

mod common;
use common::setup_db;
// ...existing code for ProductEntry tests will be inserted here...

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


 