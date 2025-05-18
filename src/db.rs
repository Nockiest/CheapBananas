use sqlx::PgPool;
use uuid::Uuid;
use crate::models::Product;

pub async fn add_shop(pool: &PgPool, name: &str) -> Result<Uuid, sqlx::Error> {
    let shop = sqlx::query!("INSERT INTO shops (name) VALUES ($1) RETURNING id", name)
        .fetch_one(pool)
        .await?;
    Ok(shop.id)
}

pub async fn add_product(pool: &PgPool, product: &Product) -> Result<Uuid, sqlx::Error> {
    let product = sqlx::query!(
        "INSERT INTO products (name, price, product_volume, unit, shop_id, date, notes, tags) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id",
        product.name,
        product.price,
        product.product_volume,
        product.unit.to_string(),
        product.shop_id,
        product.date,
        product.notes,
        product.tags.as_deref()
    )
    .fetch_one(pool)
    .await?;
    Ok(product.id)
}

pub async fn delete_product(pool: &PgPool, product_id: Uuid) -> Result<u64, sqlx::Error> {
    let result = sqlx::query!(
        "DELETE FROM products WHERE id = $1",
        product_id
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

pub async fn delete_shop(pool: &PgPool, shop_id: Uuid) -> Result<u64, sqlx::Error> {
    let result = sqlx::query!(
        "DELETE FROM shops WHERE id = $1",
        shop_id
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

pub async fn get_products_filtered(
    pool: &PgPool,
    name: Option<&str>,
    unit: Option<&str>,
    min_price: Option<f64>,
    max_price: Option<f64>,
) -> Result<Vec<Product>, sqlx::Error> {
    let mut query = String::from("SELECT id, name, price, product_volume, unit, shop_id, date, notes, tags FROM products WHERE 1=1");
    if name.is_some() {
        query.push_str(" AND name = $1");
    }
    if unit.is_some() {
        query.push_str(" AND unit = $2");
    }
    if min_price.is_some() {
        query.push_str(" AND price >= $3");
    }
    if max_price.is_some() {
        query.push_str(" AND price <= $4");
    }
    let products = sqlx::query_as::<_, Product>(&query)
        .bind(name)
        .bind(unit)
        .bind(min_price)
        .bind(max_price)
        .fetch_all(pool)
        .await?;
    Ok(products)
}
