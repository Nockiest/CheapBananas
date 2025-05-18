mod models;
mod db;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;
use crate::models::{Product, Shop};
use crate::db::{add_shop, add_product, get_products};
use serde_json::json;

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
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect::<Vec<_>>());
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
// Example main with tokio runtime
#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    dotenv::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url).await?;
    let shop_id: Uuid = add_shop(&pool, "Test Shop").await?;
    let test_product: Product = Product {
        id: Uuid::new_v4(),
        name: "Test Product".to_string(),
        price: 10.0,
        product_volume: Some(1.0),
        unit: models::Unit::Kg, // <-- Use enum variant
        shop_id: Some(shop_id),
        date: None, // Uncomment and set if needed
        notes: Some("Test notes".to_string()),
        tags: Some(vec!["tag1".to_string(), "tag2".to_string()]),
    };

    add_product(&pool, &test_product).await?;

    let products = get_products(&pool).await?;
    for product in products {
        println!("{:?}", product);
    }

    Ok(())
}
