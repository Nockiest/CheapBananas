use backend::*;
use sqlx::PgPool;
use uuid::Uuid;
use serde_json::json;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use tower::ServiceExt; // for .oneshot
use axum::body::to_bytes;
use axum::{extract::{Path, State}, Json};
use std::sync::Arc;
// Import the real handler functions from main.rs
// use backend::{handle_post_request, handle_get_request, handle_get_all_request, handle_put_request, handle_delete_request};

async fn axum_post_product(State(pool): State<Arc<PgPool>>, Json(payload): Json<serde_json::Value>) -> (axum::http::StatusCode, String) {
    let (code, body) = handle_post_request(&pool, &payload.to_string()).await;
    (axum::http::StatusCode::from_u16(code).unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR), body)
}
async fn axum_get_product(State(pool): State<Arc<PgPool>>, Path(id): Path<String>) -> (axum::http::StatusCode, String) {
    let (code, body) = handle_get_request(&pool, &id).await;
    (axum::http::StatusCode::from_u16(code).unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR), body)
}
async fn axum_get_all_products(State(pool): State<Arc<PgPool>>) -> (axum::http::StatusCode, String) {
    let (code, body) = handle_get_all_request(&pool).await;
    (axum::http::StatusCode::from_u16(code).unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR), body)
}
async fn axum_put_product(State(pool): State<Arc<PgPool>>, Path(id): Path<String>, Json(payload): Json<serde_json::Value>) -> (axum::http::StatusCode, String) {
    let (code, body) = handle_put_request(&pool, &id, &payload.to_string()).await;
    (axum::http::StatusCode::from_u16(code).unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR), body)
}
async fn axum_delete_product(State(pool): State<Arc<PgPool>>, Path(id): Path<String>) -> (axum::http::StatusCode, String) {
    let (code, body) = handle_delete_request(&pool, &id).await;
    (axum::http::StatusCode::from_u16(code).unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR), body)
}
async fn axum_post_shop(State(pool): State<Arc<PgPool>>, Json(payload): Json<serde_json::Value>) -> (axum::http::StatusCode, String) {
    let name = payload.get("name").and_then(|v| v.as_str());
    match name {
        Some(name) => {
            match backend::db::add_shop(&pool, name).await {
                Ok(id) => (axum::http::StatusCode::CREATED, serde_json::json!({"id": id}).to_string()),
                Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, serde_json::json!({"error": e.to_string()}).to_string()),
            }
        },
        None => (axum::http::StatusCode::BAD_REQUEST, serde_json::json!({"error": "Missing 'name' field"}).to_string()),
    }
}

async fn setup_app() -> Router {
    use sqlx::Executor;
    use dotenv::dotenv;
    dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url).await.expect("Failed to connect to DB");
    let _ = pool.execute("DELETE FROM products;").await;
    let _ = pool.execute("DELETE FROM shops;").await;
    let shared_pool = Arc::new(pool);
    // Inline router definition (duplicate from main.rs)
    axum::Router::new()
        .route("/products", axum::routing::post(axum_post_product).get(axum_get_all_products))
        .route("/products/{id}", axum::routing::get(axum_get_product).put(axum_put_product).delete(axum_delete_product))
        .route("/shops", axum::routing::post(axum_post_shop))
        .with_state(shared_pool)
}

#[tokio::test]
async fn test_post_shop_route() {
    let app = setup_app().await;
    let payload = json!({"name": "Test Shop"});
    let response = app
        .oneshot(Request::post("/shops")
            .header("content-type", "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(v.get("id").is_some());
}

#[tokio::test]
async fn test_post_and_get_product_route() {
    let app = setup_app().await;
    // First, add a shop
    let shop_payload = json!({"name": "Shop for Product"});
    let shop_response = app.clone()
        .oneshot(Request::post("/shops")
            .header("content-type", "application/json")
            .body(Body::from(shop_payload.to_string()))
            .unwrap())
        .await
        .unwrap();
    let shop_body = to_bytes(shop_response.into_body(), 1024 * 1024).await.unwrap();
    let shop_json: serde_json::Value = serde_json::from_slice(&shop_body).unwrap();
    let shop_id = shop_json["id"].as_str().unwrap();
    println!("TEST: Shop response: {}", shop_json);
    // Add a product
    let product_payload = json!({
        "id": Uuid::new_v4(),
        "name": "Banana",
        "price": 1.23,
        "product_volume": 1.0,
        "unit": "kg",
        "shop_id": shop_id,
        "date": null,
        "notes": "Fresh",
        "tags": ["fruit"]
    });
    println!("TEST: Product payload: {}", product_payload);
    let response = app.clone()
        .oneshot(Request::post("/products")
            .header("content-type", "application/json")
            .body(Body::from(product_payload.to_string()))
            .unwrap())
        .await
        .unwrap();
    println!("TEST: POST /products status: {}", response.status());
    let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    println!("TEST: POST /products body: {}", String::from_utf8_lossy(&body));
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    println!("TEST: POST /products response JSON: {}", v);
    assert!(v.get("id").is_some());

    // Get all products
    let response = app
        .oneshot(Request::get("/products")
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();
    println!("TEST: GET /products status: {}", response.status());
    let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    println!("TEST: GET /products body: {}", String::from_utf8_lossy(&body));
    let products: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    println!("TEST: GET /products response JSON: {:?}", products);
    assert!(products.iter().any(|p| p["name"] == "Banana"));
}

#[tokio::test]
async fn test_update_and_delete_product_route() {
    let app = setup_app().await;
    // Add shop
    let shop_payload = json!({"name": "Shop for Update"});
    let shop_response = app.clone()
        .oneshot(Request::post("/shops")
            .header("content-type", "application/json")
            .body(Body::from(shop_payload.to_string()))
            .unwrap())
        .await
        .unwrap();
    let shop_body = to_bytes(shop_response.into_body(), 1024 * 1024).await.unwrap();
    let shop_json: serde_json::Value = serde_json::from_slice(&shop_body).unwrap();
    let shop_id = shop_json["id"].as_str().unwrap();

    // Add product
    let product_id = Uuid::new_v4();
    println!("TEST: Creating product with id: {}", product_id);
    let product_payload = json!({
        "id": product_id,
        "name": "Apple",
        "price": 2.0,
        "product_volume": 1.0,
        "unit": "kg",
        "shop_id": shop_id,
        "date": null,
        "notes": "Red",
        "tags": ["fruit"]
    });
    let response = app.clone()
        .oneshot(Request::post("/products")
            .header("content-type", "application/json")
            .body(Body::from(product_payload.to_string()))
            .unwrap())
        .await
        .unwrap();
    println!("TEST: POST /products status: {}", response.status());
    assert_eq!(response.status(), StatusCode::CREATED);

    // Update product
    let update_payload = json!({"name": "Green Apple", "price": 2.5});
    println!("TEST: Updating product with id: {}", product_id);
    let response = app.clone()
        .oneshot(Request::put(format!("/products/{}", product_id))
            .header("content-type", "application/json")
            .body(Body::from(update_payload.to_string()))
            .unwrap())
        .await
        .unwrap();
    println!("TEST: PUT /products/{{id}} status: {}", response.status());
    let status1 = response.status();
    let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    println!("TEST: PUT /products/{{id}} body: {}", String::from_utf8_lossy(&body));
    assert_eq!(status1, StatusCode::OK);
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["updated"], 1);

    // Delete product
    println!("TEST: Deleting product with id: {}", product_id);
    let response = app
        .oneshot(Request::delete(format!("/products/{}", product_id))
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();
    println!("TEST: DELETE /products/{{id}} status: {}", response.status());
    let status = response.status();
    let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    println!("TEST: DELETE /products/{{id}} body: {}", String::from_utf8_lossy(&body));
    assert_eq!(status, StatusCode::OK);
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["deleted"], 1);
}
