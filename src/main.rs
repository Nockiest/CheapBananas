mod models;
mod db;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;
use crate::models::{Product, Shop};
use crate::db::{add_shop, add_product, get_products};
use serde_json::json;
use axum::{
    extract::{Path, State},
    routing::{get, post, put, delete},
    Router, Json,
};
use std::sync::Arc;
use tokio::net::TcpListener;
use axum::serve;

//handle post request
async fn handle_post_request(pool: &PgPool, request: &str) -> (u16, String) {
    // Expecting JSON body for new product
    match serde_json::from_str::<Product>(request) {
        Ok(product) => {
            match add_product(pool, &product).await {
                Ok(id) => (201, json!({"id": id}).to_string()),
                Err(e) => (500, json!({"error": e.to_string()}).to_string()),
            }
        },
        Err(e) => (400, json!({"error": format!("Invalid JSON: {}", e)}).to_string()),
    }
}

//handle get request (by product id)
async fn handle_get_request(pool: &PgPool, id: &str) -> (u16, String) {
    use uuid::Uuid;
    match Uuid::parse_str(id) {
        Ok(uuid) => {
            match get_products(pool).await {
                Ok(products) => {
                    if let Some(product) = products.into_iter().find(|p| p.id == uuid) {
                        (200, serde_json::to_string(&product).unwrap())
                    } else {
                        (404, json!({"error": "Product not found"}).to_string())
                    }
                },
                Err(e) => (500, json!({"error": e.to_string()}).to_string()),
            }
        },
        Err(e) => (400, json!({"error": format!("Invalid UUID: {}", e)}).to_string()),
    }
}

//handle get all request
async fn handle_get_all_request(pool: &PgPool) -> (u16, String) {
    match get_products(pool).await {
        Ok(products) => (200, serde_json::to_string(&products).unwrap()),
        Err(e) => (500, json!({"error": e.to_string()}).to_string()),
    }
}

//handle put request (update product)
async fn handle_put_request(pool: &PgPool, id: &str, request: &str) -> (u16, String) {
    use uuid::Uuid;
    use chrono::NaiveDateTime;
    match Uuid::parse_str(id) {
        Ok(uuid) => {
            let update: serde_json::Value = match serde_json::from_str(request) {
                Ok(val) => val,
                Err(e) => return (400, json!({"error": format!("Invalid JSON: {}", e)}).to_string()),
            };
            // Extract fields, pass as Option<_> to update_product
            let name = update.get("name").and_then(|v| v.as_str());
            let price = update.get("price").and_then(|v| v.as_f64());
            let product_volume = update.get("product_volume").and_then(|v| v.as_f64());
            let unit = update.get("unit").and_then(|v| v.as_str());
            let shop_id = update.get("shop_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok());
            let date = update.get("date")
                .and_then(|v| v.as_str())
                .and_then(|s| NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S").ok());
            let notes = update.get("notes").and_then(|v| v.as_str());
            let tags_vec = update.get("tags")
                .and_then(|v| v.as_array()
                    .and_then(|arr| {
                        let collected: Vec<String> = arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
                        if collected.is_empty() { None } else { Some(collected) }
                    })
                );
            let tags = tags_vec.as_deref();
            match db::update_product(
                pool, uuid,
                name, price, product_volume, unit, shop_id, date, notes, tags
            ).await {
                Ok(affected) if affected > 0 => (200, json!({"updated": affected}).to_string()),
                Ok(_) => (404, json!({"error": "Product not found"}).to_string()),
                Err(e) => (500, json!({"error": e.to_string()}).to_string()),
            }
        },
        Err(e) => (400, json!({"error": format!("Invalid UUID: {}", e)}).to_string()),
    }
}

//handle delete request
async fn handle_delete_request(pool: &PgPool, id: &str) -> (u16, String) {
    use uuid::Uuid;
    match Uuid::parse_str(id) {
        Ok(uuid) => {
            match db::delete_product(pool, uuid).await {
                Ok(affected) if affected > 0 => (200, json!({"deleted": affected}).to_string()),
                Ok(_) => (404, json!({"error": "Product not found"}).to_string()),
                Err(e) => (500, json!({"error": e.to_string()}).to_string()),
            }
        },
        Err(e) => (400, json!({"error": format!("Invalid UUID: {}", e)}).to_string()),
    }
}
#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    dotenv::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url).await?;
    let shared_pool = Arc::new(pool);

    let app = Router::new()
        .route("/products", post(axum_post_product).get(axum_get_all_products))
        // .route("/products/:id", get(axum_get_product).put(axum_put_product).delete(axum_delete_product))
        .with_state(shared_pool.clone())
        .without_v07_checks();

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    serve(listener, app).await.unwrap();
    Ok(())
}

// Axum handler wrappers
async fn axum_post_product(
    State(pool): State<Arc<PgPool>>,
    Json(payload): Json<serde_json::Value>,
) -> (axum::http::StatusCode, String) {
    let (code, body) = handle_post_request(&pool, &payload.to_string()).await;
    (axum::http::StatusCode::from_u16(code).unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR), body)
}

async fn axum_get_product(
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<String>,
) -> (axum::http::StatusCode, String) {
    let (code, body) = handle_get_request(&pool, &id).await;
    (axum::http::StatusCode::from_u16(code).unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR), body)
}

async fn axum_get_all_products(
    State(pool): State<Arc<PgPool>>,
) -> (axum::http::StatusCode, String) {
    let (code, body) = handle_get_all_request(&pool).await;
    (axum::http::StatusCode::from_u16(code).unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR), body)
}

async fn axum_put_product(
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> (axum::http::StatusCode, String) {
    let (code, body) = handle_put_request(&pool, &id, &payload.to_string()).await;
    (axum::http::StatusCode::from_u16(code).unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR), body)
}

async fn axum_delete_product(
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<String>,
) -> (axum::http::StatusCode, String) {
    let (code, body) = handle_delete_request(&pool, &id).await;
    (axum::http::StatusCode::from_u16(code).unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR), body)
}
