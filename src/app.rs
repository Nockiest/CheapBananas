use crate::{db};
use crate::models::{Product, Shop, ProductEntry, ProductFilter};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{delete, get, post, put},
};
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;
use tower_http::cors::{CorsLayer, Any};
use uuid::Uuid;
use crate::utils::text_utils::sanitize_underscores_to_empty;
// --- Handler logic ---
pub async fn handle_post_request(pool: &PgPool, request: &str) -> (u16, String) {
    match serde_json::from_str::<Product>(request) {
        Ok(product) => {
            println!("[INFO] Received product POST: {:?}", product);
            match db::add_product(pool, &product).await {
                Ok(id) => {
                    println!("[INFO] Product added to DB with id: {}", id);
                    (201, json!({"id": id}).to_string())
                },
                Err(e) => {
                    println!("[ERROR] Failed to add product: {}", e);
                    (500, json!({"error": e.to_string()}).to_string())
                },
            }
        },
        Err(e) => {
            println!("[ERROR] Invalid product JSON: {}", e);
            (400, json!({"error": format!("Invalid JSON: {}", e)}).to_string())
        },
    }
}

pub async fn handle_get_request(pool: &PgPool, id: &str) -> (u16, String) {
    match Uuid::parse_str(id) {
        Ok(uuid) => match db::get_products(pool).await {
            Ok(products) => {
                if let Some(product) = products.into_iter().find(|p| p.id == uuid) {
                    (200, serde_json::to_string(&product).unwrap())
                } else {
                    (404, json!({"error": "Product not found"}).to_string())
                }
            }
            Err(e) => (500, json!({"error": e.to_string()}).to_string()),
        },
        Err(e) => (
            400,
            json!({"error": format!("Invalid UUID: {}", e)}).to_string(),
        ),
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
                Err(e) => {
                    return (
                        400,
                        serde_json::json!({"error": format!("Invalid JSON: {}", e)}).to_string(),
                    );
                }
            };
            let name = update.get("name").and_then(|v| v.as_str());
            let notes = update.get("notes").and_then(|v| v.as_str());
            let tags_vec = update.get("tags").and_then(|v| {
                v.as_array().map(|arr| {
                    arr.iter()
                        .filter_map(|t| t.as_str().map(|s| s.to_string()))
                        .collect::<Vec<String>>()
                })
            });
            let tags = tags_vec.as_ref().map(|v| v.as_slice());
            match db::update_product(pool, uuid, name, None, None, None, None, None, notes, tags)
                .await
            {
                Ok(affected) => {
                    if affected == 0 {
                        (
                            404,
                            serde_json::json!({"error": "Product not found or nothing to update"})
                                .to_string(),
                        )
                    } else {
                        (200, serde_json::json!({"updated": affected}).to_string())
                    }
                }
                Err(e) => (500, serde_json::json!({"error": e.to_string()}).to_string()),
            }
        }
        Err(e) => (
            400,
            serde_json::json!({"error": format!("Invalid UUID: {}", e)}).to_string(),
        ),
    }
}

pub async fn handle_delete_request(pool: &PgPool, id: &str) -> (u16, String) {
    match Uuid::parse_str(id) {
        Ok(uuid) => match db::delete_product(pool, uuid).await {
            Ok(affected) if affected > 0 => (200, json!({"deleted": affected}).to_string()),
            Ok(_) => (404, json!({"error": "Product not found"}).to_string()),
            Err(e) => (500, json!({"error": e.to_string()}).to_string()),
        },
        Err(e) => (
            400,
            json!({"error": format!("Invalid UUID: {}", e)}).to_string(),
        ),
    }
}

pub async fn handle_post_product_entry(pool: &PgPool, request: &str) -> (u16, String) {
    use crate::models::ProductEntry;
    use crate::db;
    use serde_json::Value;
    println!("[INFO] Incoming product entry POST body: {}", request);
    match serde_json::from_str::<Value>(request) {
        Ok(entry_val) => {
            println!("[DEBUG] Parsed JSON for product entry: {:?}", entry_val);
            // Try to resolve product_name to product_id before any further validation
            let product_name = entry_val.get("product_name").and_then(|v| v.as_str());
            let shop_name = entry_val.get("shop_name").and_then(|v| v.as_str());
            if product_name.is_none() {
                println!("[ERROR] Missing product_name in product entry JSON");
                return (400, json!({"error": "Missing product_name in product entry"}).to_string());
            }
            if shop_name.is_none() {
                println!("[ERROR] Missing shop_name in product entry JSON");
                return (400, json!({"error": "Missing shop_name in product entry"}).to_string());
            }
            // If shop_name is provided, check if it exists
            let shop_id_opt = if let Some(shop_name) = shop_name {
                println!("[INFO] Looking up shop by name: {}", shop_name);
                match db::get_shops_filtered(pool, crate::models::ShopFilter { name: Some(shop_name), ..Default::default() }).await {
                    Ok(shops) if !shops.is_empty() => {
                        println!("[INFO] Found shop id {} for name {}", shops[0].id, shop_name);
                        Some(shops[0].id)
                    },
                    Ok(_) => {
                        println!("[ERROR] No shop found with name: {}", shop_name);
                        return (400, json!({"error": format!("No shop found with name: {}", shop_name)}).to_string());
                    },
                    Err(e) => {
                        println!("[ERROR] DB error looking up shop by name: {}", e);
                        return (500, json!({"error": e.to_string()}).to_string());
                    }
                }
            } else { None };
            println!("[INFO] Looking up product by name: {}", product_name.unwrap());
            match db::get_product_by_name(pool, product_name.unwrap()).await {
                Ok(Some(product)) => {
                    let product_id = product.id;
                    println!("[INFO] Found product id {} for name {}", product_id, product_name.unwrap());
                    let mut entry_map = entry_val.as_object().unwrap().clone();
                    entry_map.insert("product_id".to_string(), serde_json::json!(product_id));
                    if let Some(shop_id) = shop_id_opt {
                        entry_map.insert("shop_id".to_string(), serde_json::json!(shop_id));
                    }
                    // Remove product_name (optional, if ProductEntry expects product_id only)
                    // entry_map.remove("product_name");
                    // Now try to deserialize to ProductEntry (this will check formatting)
                    match serde_json::from_value::<ProductEntry>(serde_json::Value::Object(entry_map)) {
                        Ok(entry) => {
                            println!("[INFO] Received product entry POST (fully formed): {:?}", entry);
                            match db::add_product_entry(pool, &entry).await {
                                Ok(id) => {
                                    println!("[INFO] Product entry added to DB with id: {}", id);
                                    (201, json!({"id": id}).to_string())
                                },
                                Err(e) => {
                                    println!("[ERROR] Failed to add product entry: {}", e);
                                    (500, json!({"error": e.to_string()}).to_string())
                                },
                            }
                        },
                        Err(e) => {
                            println!("[ERROR] Invalid product entry JSON after adding product_id: {}", e);
                            (400, json!({"error": format!("Invalid product entry: {}", e)}).to_string())
                        }
                    }
                },
                Ok(None) => {
                    println!("[ERROR] No product found with name: {}", product_name.unwrap());
                    (400, json!({"error": format!("No product found with name: {}", product_name.unwrap())}).to_string())
                },
                Err(e) => {
                    println!("[ERROR] DB error looking up product by name: {}", e);
                    (500, json!({"error": e.to_string()}).to_string())
                }
            }
        },
        Err(e) => {
            println!("[ERROR] Invalid product entry JSON: {}", e);
            (400, json!({"error": format!("Invalid JSON: {}", e)}).to_string())
        },
    }
}

// --- Axum handler wrappers ---x
pub async fn axum_post_product(
    State(pool): State<Arc<PgPool>>,
    Json(payload): Json<serde_json::Value>,
) -> (axum::http::StatusCode, String) {
    // Sanitize incoming JSON payload
    let sanitized_payload = sanitize_underscores_to_empty(payload);

    // Convert sanitized payload to string and pass to handler
    let (code, body) = handle_post_request(&pool, &sanitized_payload.to_string()).await;

    (
        axum::http::StatusCode::from_u16(code)
            .unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
        body,
    )
}
pub async fn axum_get_product(
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<String>,
) -> (axum::http::StatusCode, String) {
    let (code, body) = handle_get_request(&pool, &id).await;
    (
        axum::http::StatusCode::from_u16(code)
            .unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
        body,
    )
}

pub async fn axum_get_all_products(
    State(pool): State<Arc<PgPool>>,
) -> (axum::http::StatusCode, String) {
    let (code, body) = handle_get_all_request(&pool).await;
    (
        axum::http::StatusCode::from_u16(code)
            .unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
        body,
    )
}

pub async fn axum_put_product(
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> (axum::http::StatusCode, String) {
    let (code, body) = handle_put_request(&pool, &id, &payload.to_string()).await;
    (
        axum::http::StatusCode::from_u16(code)
            .unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
        body,
    )
}

pub async fn axum_delete_product(
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<String>,
) -> (axum::http::StatusCode, String) {
    let (code, body) = handle_delete_request(&pool, &id).await;
    (
        axum::http::StatusCode::from_u16(code)
            .unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
        body,
    )
}
pub async fn axum_post_product_entry(
    State(pool): State<Arc<PgPool>>,
    Json(payload): Json<serde_json::Value>,
) -> (axum::http::StatusCode, String) {
    // Sanitize incoming JSON payload
    let sanitized_payload = sanitize_underscores_to_empty(payload);

    // Convert sanitized payload to string and pass to handler
    let (code, body) = handle_post_product_entry(&pool, &sanitized_payload.to_string()).await;

    (
        axum::http::StatusCode::from_u16(code)
            .unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
        body,
    )
}
pub async fn axum_delete_product_entry(
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<String>,
) -> (axum::http::StatusCode, String) {
    match Uuid::parse_str(&id) {
        Ok(uuid) => match db::delete_product_entry(&pool, uuid).await {
            Ok(affected) if affected > 0 => (200, json!({"deleted": affected}).to_string()),
            Ok(_) => (404, json!({"error": "Product entry not found"}).to_string()),
            Err(e) => (500, json!({"error": e.to_string()}).to_string()),
        },
        Err(e) => (
            400,
            json!({"error": format!("Invalid UUID: {}", e)}).to_string(),
        ),
    }
}

pub async fn axum_post_shop(
    State(pool): State<Arc<PgPool>>,
    Json(payload): Json<serde_json::Value>,
) -> (axum::http::StatusCode, String) {
    // Sanitize incoming JSON payload
    let sanitized_payload = sanitize_underscores_to_empty(payload);

    // Add a default ID if not provided
    let mut shop_data = sanitized_payload.as_object().cloned().unwrap_or_default();
    if !shop_data.contains_key("id") {
        shop_data.insert("id".to_string(), serde_json::json!(Uuid::new_v4()));
    }

    // Attempt to deserialize the shop
    let shop: Result<Shop, _> = serde_json::from_value(serde_json::Value::Object(shop_data));
    match shop {
        Ok(shop) => {
            println!("[INFO] Received shop POST: name={}", shop.name);
            match db::add_shop(&pool, &shop).await {
                Ok(id) => {
                    println!("[INFO] Shop added to DB with id: {}", id);
                    (
                        axum::http::StatusCode::CREATED,
                        serde_json::json!({"id": id}).to_string(),
                    )
                },
                Err(e) => {
                    println!("[ERROR] Failed to add shop: {}", e);
                    (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        serde_json::json!({"error": e.to_string()}).to_string(),
                    )
                },
            }
        }
        Err(e) => {
            println!("[ERROR] Invalid shop JSON: {}", e);
            (
                axum::http::StatusCode::BAD_REQUEST,
                serde_json::json!({"error": format!("Invalid shop JSON: {}", e)}).to_string(),
            )
        }
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
    pub product_volume: Option<f64>, // Assuming this is a string for the sake of example
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
        date: params
            .date
            .as_deref()
            .and_then(|d| NaiveDateTime::parse_from_str(d, "%Y-%m-%dT%H:%M:%S").ok()),
        notes: params.notes.as_deref(),
        tag: params.tag.as_deref(),
        product_id: params.product_id,
        product_volume: params.product_volume, // FIX: do not use .as_deref() on Option<f64>
    };
    println!("[DEBUG] Built ProductFilter: {:?}", filter);
    let result = db::get_products_filtered(&pool, filter).await;
    match result {
        Ok(mut products) => {
            // If you want to enrich products with shop info, you need to redesign Product to include shop_id and shop_name fields.
            // For now, just return the products as-is.
            (
                axum::http::StatusCode::OK,
                serde_json::to_string(&products).unwrap(),
            )
        }
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            serde_json::json!({"error": e.to_string()}).to_string(),
        ),
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
        date: params
            .date
            .as_deref()
            .and_then(|d| NaiveDateTime::parse_from_str(d, "%Y-%m-%dT%H:%M:%S").ok()),
        notes: params.notes.as_deref(),
        tag: params.tag.as_deref(),
        product_id: params.product_id,
        product_volume: params.product_volume,
    };
    println!("[DEBUG] Built ProductFilter for entries: {:?}", filter);
    let result = db::get_product_entries_filtered(&pool, filter).await;
    match result {
        Ok(mut entries) => {
            for entry in &mut entries {
                if let Some(shop_id) = entry.shop_id {
                    match db::get_shop_by_id(&pool, shop_id).await {
                        Ok(Some(shop)) => entry.shop_name = Some(shop.name),
                        Ok(None) => entry.shop_name = None, // Shop not found
                        Err(e) => {
                            return (
                                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                                serde_json::json!({"error": e.to_string()}).to_string(),
                        );
                        }
                    }
                }
            }
            (
                axum::http::StatusCode::OK,
                serde_json::to_string(&entries).unwrap(),
            )
        }
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            serde_json::json!({"error": e.to_string()}).to_string(),
        ),
    }
}

#[derive(Debug, Deserialize)]
pub struct ShopFilterQuery {
    pub id: Option<Uuid>,
    pub name: Option<String>,
    pub notes: Option<String>,
}

pub async fn axum_get_shops_filtered(
    State(pool): State<Arc<PgPool>>,
    Query(params): Query<ShopFilterQuery>,
) -> (axum::http::StatusCode, String) {
    let filter = crate::models::ShopFilter {
        id: params.id,
        name: params.name.as_deref(),
        notes: params.notes.as_deref(),
    };
    let result = db::get_shops_filtered(&pool, filter).await;
    match &result {
        Ok(shops) => println!("[DEBUG] Filtered shops count: {}", shops.len()),
        Err(e) => println!("[DEBUG] Error filtering shops: {}", e),
    }
    match result {
        Ok(shops) => (
            axum::http::StatusCode::OK,
            serde_json::to_string(&shops).unwrap(),
        ),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            serde_json::json!({"error": e.to_string()}).to_string(),
        ),
    }
}

// --- Router builder ---
pub fn build_app_router(shared_pool: Arc<PgPool>) -> Router {
    Router::new()
        .route(
            "/products",
            post(axum_post_product).get(axum_get_all_products),
        )
        .route("/products/filter", get(axum_get_products_filtered))
        .route(
            "/products/{id}",
            get(axum_get_product),
        )
        .route("/product-entries", post(axum_post_product_entry))
        .route(
            "/product-entries/{id}",
            delete(axum_delete_product_entry),
        )
        .route("/shops", post(axum_post_shop))
        .route(
            "/product-entries/filter",
            get(axum_get_product_entries_filtered),
        )
        .route("/shops/filter", get(axum_get_shops_filtered))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any)
        )
        .with_state(shared_pool)
}
