use sqlx::PgPool;
use std::sync::Arc;
use axum::{extract::{Path, State, Query}, Json, Router, routing::{get, post, put, delete}};
use crate::db;
use crate::models::{Product, ProductEntry, ProductFilter};
use serde_json::json;
use uuid::Uuid;
use serde::Deserialize;

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
    match Uuid::parse_str(id) {
        Ok(uuid) => {
            let update: serde_json::Value = match serde_json::from_str(request) {
                Ok(val) => val,
                Err(e) => return (400, serde_json::json!({"error": format!("Invalid JSON: {}", e)}).to_string()),
            };
            let name = update.get("name").and_then(|v| v.as_str());
            let notes = update.get("notes").and_then(|v| v.as_str());
            let tags_vec = update.get("tags").and_then(|v| v.as_array().map(|arr| arr.iter().filter_map(|t| t.as_str().map(|s| s.to_string())).collect::<Vec<String>>()));
            let tags = tags_vec.as_ref().map(|v| v.as_slice());
            match db::update_product(
                pool, uuid,
                name,
                None, None, None, None, None,
                notes,
                tags,
            ).await {
                Ok(affected) => {
                    if affected == 0 {
                        (404, serde_json::json!({"error": "Product not found or nothing to update"}).to_string())
                    } else {
                        (200, serde_json::json!({"updated": affected}).to_string())
                    }
                },
                Err(e) => (500, serde_json::json!({"error": e.to_string()}).to_string()),
            }
        },
        Err(e) => (400, serde_json::json!({"error": format!("Invalid UUID: {}", e)}).to_string()),
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

pub async fn handle_post_product_entry(pool: &PgPool, request: &str) -> (u16, String) {
    match serde_json::from_str::<ProductEntry>(request) {
        Ok(entry) => {
            match db::add_product_entry(pool, &entry).await {
                Ok(id) => (201, json!({"id": id}).to_string()),
                Err(e) => (500, json!({"error": e.to_string()}).to_string()),
            }
        },
        Err(e) => (400, json!({"error": format!("Invalid JSON: {}", e)}).to_string()),
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

pub async fn axum_post_product_entry(
    State(pool): State<Arc<PgPool>>,
    Json(payload): Json<serde_json::Value>,
) -> (axum::http::StatusCode, String) {
    let (code, body) = handle_post_product_entry(&pool, &payload.to_string()).await;
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

#[derive(Debug, Deserialize)]
pub struct ProductFilterQuery {
    pub id: Option<Uuid>,
    pub name: Option<String>,
    pub unit: Option<String>,
    pub min_price: Option<f64>,
    pub max_price: Option<f64>,
    pub shop_id: Option<Uuid>,
    pub date: Option<String>,
    pub notes: Option<String>,
    pub tag: Option<String>,
    pub product_id: Option<Uuid>,
}

pub async fn axum_get_products_filtered(
    State(pool): State<Arc<PgPool>>,
    Query(params): Query<ProductFilterQuery>,
) -> (axum::http::StatusCode, String) {
    use chrono::NaiveDateTime;
    println!("[DEBUG] Received filter params: {:?}", params);
    let filter = ProductFilter {
        id: params.id,
        name: params.name.as_deref(),
        unit: params.unit.as_deref(),
        min_price: params.min_price,
        max_price: params.max_price,
        shop_id: params.shop_id,
        date: params.date.as_deref().and_then(|d| NaiveDateTime::parse_from_str(d, "%Y-%m-%dT%H:%M:%S").ok()),
        notes: params.notes.as_deref(),
        tag: params.tag.as_deref(),
        product_id: params.product_id,
    };
    println!("[DEBUG] Built ProductFilter: {:?}", filter);
    let result = db::get_products_filtered(&pool, filter).await;
    match &result {
        Ok(products) => println!("[DEBUG] Filtered products count: {}", products.len()),
        Err(e) => println!("[DEBUG] Error filtering products: {}", e),
    }
    match result {
        Ok(products) => (axum::http::StatusCode::OK, serde_json::to_string(&products).unwrap()),
        Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, serde_json::json!({"error": e.to_string()}).to_string()),
    }
}

pub async fn axum_get_product_entries_filtered(
    State(pool): State<Arc<PgPool>>,
    Query(params): Query<ProductFilterQuery>,
) -> (axum::http::StatusCode, String) {
    use chrono::NaiveDateTime;
    println!("[DEBUG] Received product entry filter params: {:?}", params);
    let filter = ProductFilter {
        id: params.id,
        name: params.name.as_deref(),
        unit: params.unit.as_deref(),
        min_price: params.min_price,
        max_price: params.max_price,
        shop_id: params.shop_id,
        date: params.date.as_deref().and_then(|d| NaiveDateTime::parse_from_str(d, "%Y-%m-%dT%H:%M:%S").ok()),
        notes: params.notes.as_deref(),
        tag: params.tag.as_deref(),
        product_id: params.product_id,
    };
    println!("[DEBUG] Built ProductFilter for entries: {:?}", filter);
    let result = db::get_product_entries_filtered(&pool, filter).await;
    match &result {
        Ok(entries) => println!("[DEBUG] Filtered product entries: {:?}", entries),
        Err(e) => println!("[DEBUG] Error filtering product entries: {}", e),
    }
    match result {
        Ok(entries) => (axum::http::StatusCode::OK, serde_json::to_string(&entries).unwrap()),
        Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, serde_json::json!({"error": e.to_string()}).to_string()),
    }
}

// --- Router builder ---
pub fn build_app_router(shared_pool: Arc<PgPool>) -> Router {
    Router::new()
        .route("/products", post(axum_post_product).get(axum_get_all_products))
        .route("/products/filter", get(axum_get_products_filtered))
        .route("/products/{id}", get(axum_get_product).put(axum_put_product).delete(axum_delete_product))
        .route("/product-entries", post(axum_post_product_entry))
        .route("/shops", post(axum_post_shop))
        .route("/product-entries/filter", get(axum_get_product_entries_filtered))
        .with_state(shared_pool)
}
