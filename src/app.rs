use sqlx::PgPool;
use std::sync::Arc;
use axum::{extract::{Path, State}, Json, Router, routing::{get, post, put, delete}};
use crate::db;
use crate::models::Product;
use serde_json::json;
use uuid::Uuid;

// --- Handler logic ---
pub async fn handle_post_request(pool: &PgPool, request: &str) -> (u16, String) {
    match serde_json::from_str::<Product>(request) {
        Ok(product) => {
            match db::add_product(pool, &product).await {
                Ok(id) => (201, json!({"id": id}).to_string()),
                Err(e) => (500, json!({"error": e.to_string()}).to_string()),
            }
        },
        Err(e) => (400, json!({"error": format!("Invalid JSON: {}", e)}).to_string()),
    }
}

pub async fn handle_get_request(pool: &PgPool, id: &str) -> (u16, String) {
    match Uuid::parse_str(id) {
        Ok(uuid) => {
            match db::get_products(pool).await {
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

pub async fn handle_get_all_request(pool: &PgPool) -> (u16, String) {
    match db::get_products(pool).await {
        Ok(products) => (200, serde_json::to_string(&products).unwrap()),
        Err(e) => (500, json!({"error": e.to_string()}).to_string()),
    }
}

pub async fn handle_put_request(pool: &PgPool, id: &str, request: &str) -> (u16, String) {
    use chrono::NaiveDateTime;
    match Uuid::parse_str(id) {
        Ok(uuid) => {
            let update: serde_json::Value = match serde_json::from_str(request) {
                Ok(val) => val,
                Err(e) => return (400, json!({"error": format!("Invalid JSON: {}", e)}).to_string()),
            };
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

pub async fn handle_delete_request(pool: &PgPool, id: &str) -> (u16, String) {
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

// --- Axum handler wrappers ---
pub async fn axum_post_product(
    State(pool): State<Arc<PgPool>>,
    Json(payload): Json<serde_json::Value>,
) -> (axum::http::StatusCode, String) {
    let (code, body) = handle_post_request(&pool, &payload.to_string()).await;
    (axum::http::StatusCode::from_u16(code).unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR), body)
}

pub async fn axum_get_product(
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<String>,
) -> (axum::http::StatusCode, String) {
    let (code, body) = handle_get_request(&pool, &id).await;
    (axum::http::StatusCode::from_u16(code).unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR), body)
}

pub async fn axum_get_all_products(
    State(pool): State<Arc<PgPool>>,
) -> (axum::http::StatusCode, String) {
    let (code, body) = handle_get_all_request(&pool).await;
    (axum::http::StatusCode::from_u16(code).unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR), body)
}

pub async fn axum_put_product(
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> (axum::http::StatusCode, String) {
    let (code, body) = handle_put_request(&pool, &id, &payload.to_string()).await;
    (axum::http::StatusCode::from_u16(code).unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR), body)
}

pub async fn axum_delete_product(
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<String>,
) -> (axum::http::StatusCode, String) {
    let (code, body) = handle_delete_request(&pool, &id).await;
    (axum::http::StatusCode::from_u16(code).unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR), body)
}

pub async fn axum_post_shop(
    State(pool): State<Arc<PgPool>>,
    Json(payload): Json<serde_json::Value>,
) -> (axum::http::StatusCode, String) {
    let name = payload.get("name").and_then(|v| v.as_str());
    match name {
        Some(name) => {
            match db::add_shop(&pool, name).await {
                Ok(id) => (axum::http::StatusCode::CREATED, serde_json::json!({"id": id}).to_string()),
                Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, serde_json::json!({"error": e.to_string()}).to_string()),
            }
        },
        None => (axum::http::StatusCode::BAD_REQUEST, serde_json::json!({"error": "Missing 'name' field"}).to_string()),
    }
}

// --- Router builder ---
pub fn build_app_router(shared_pool: Arc<PgPool>) -> Router {
    Router::new()
        .route("/products", post(axum_post_product).get(axum_get_all_products))
        .route("/products/{id}", get(axum_get_product).put(axum_put_product).delete(axum_delete_product))
        .route("/shops", post(axum_post_shop))
        .with_state(shared_pool)
}
