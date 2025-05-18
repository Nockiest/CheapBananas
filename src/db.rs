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
pub async fn get_products(pool: &PgPool) -> Result<Vec<Product>, sqlx::Error> {
    let products: Vec<Product> = sqlx::query_as::<_, Product>(
        "SELECT id, name, price, product_volume, unit, shop_id, date, notes, tags FROM products",
    )
    .fetch_all(pool)
    .await?;
    Ok(products)
}


pub async fn update_product(
    pool: &PgPool,
    product_id: Uuid,
    name: Option<&str>,
    price: Option<f64>,
    product_volume: Option<f64>,
    unit: Option<&str>,
    shop_id: Option<Uuid>,
    date: Option<chrono::NaiveDateTime>,
    notes: Option<&str>,
    tags: Option<&[String]>,
) -> Result<u64, sqlx::Error> {
    // Validate price and product_volume
    if let Some(p) = price {
        if !(0.0..=10000.0).contains(&p) {
            return Err(sqlx::Error::Protocol("Price out of bounds".into()));
        }
    }
    if let Some(v) = product_volume {
        if !(0.0..=10000.0).contains(&v) {
            return Err(sqlx::Error::Protocol("Product volume out of bounds".into()));
        }
    }
    // If shop_id is provided, check it exists
    if let Some(sid) = shop_id {
        let shop_exists = sqlx::query_scalar!("SELECT EXISTS(SELECT 1 FROM shops WHERE id = $1)", sid)
            .fetch_one(pool)
            .await?;
        if !shop_exists.unwrap_or(false) {
            return Err(sqlx::Error::Protocol("Shop does not exist".into()));
        }
    }
    // Use QueryBuilder for dynamic SQL
    use sqlx::QueryBuilder;
    let mut builder = QueryBuilder::new("UPDATE products SET ");
    let mut first = true;
    if let Some(n) = name {
        if !first { builder.push(", "); } first = false;
        builder.push("name = ").push_bind(n);
    }
    if let Some(p) = price {
        if !first { builder.push(", "); } first = false;
        builder.push("price = ").push_bind(p);
    }
    if let Some(v) = product_volume {
        if !first { builder.push(", "); } first = false;
        builder.push("product_volume = ").push_bind(v);
    }
    if let Some(u) = unit {
        if !first { builder.push(", "); } first = false;
        builder.push("unit = ").push_bind(u);
    }
    if let Some(sid) = shop_id {
        if !first { builder.push(", "); } first = false;
        builder.push("shop_id = ").push_bind(sid);
    }
    if let Some(d) = date {
        if !first { builder.push(", "); } first = false;
        builder.push("date = ").push_bind(d);
    }
    if let Some(n) = notes {
        if !first { builder.push(", "); } first = false;
        builder.push("notes = ").push_bind(n);
    }
    if let Some(t) = tags {
        if !first { builder.push(", "); } first = false;
        builder.push("tags = ").push_bind(t);
    }
    if first {
        return Ok(0); // Nothing to update
    }
    builder.push(" WHERE id = ").push_bind(product_id);
    let query = builder.build();
    let result = query.execute(pool).await?;
    Ok(result.rows_affected())
}
