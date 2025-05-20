use sqlx::PgPool;
use uuid::Uuid;
use crate::models::{Product, ProductEntry};

pub async fn add_shop(pool: &PgPool, name: &str) -> Result<Uuid, sqlx::Error> {
    let shop = sqlx::query!("INSERT INTO shops (name) VALUES ($1) RETURNING id", name)
        .fetch_one(pool)
        .await?;
    Ok(shop.id)
}

pub async fn add_product(pool: &PgPool, product: &Product) -> Result<Uuid, sqlx::Error> {
    let product = sqlx::query!(
        "INSERT INTO products (id, name, notes, tags) VALUES ($1, $2, $3, $4) RETURNING id",
        product.id,
        product.name,
        product.notes,
        product.tags.as_deref()
    )
    .fetch_one(pool)
    .await?;
    Ok(product.id)
}

pub async fn add_product_entry(pool: &PgPool, entry: &ProductEntry) -> Result<Uuid, sqlx::Error> {
    let row = sqlx::query!(
        "INSERT INTO product_entries (id, product_id, price, product_volume, unit, shop_id, date, notes) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id",
        entry.id,
        entry.product_id,
        entry.price,
        entry.product_volume,
        entry.unit as _,
        entry.shop_id,
        entry.date,
        entry.notes
    )
    .fetch_one(pool)
    .await?;
    Ok(row.id)
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

// Filter struct for get_products_filtered
#[derive(Default)]
pub struct ProductFilter<'a> {
    pub name: Option<&'a str>,
    pub unit: Option<&'a str>,
    pub min_price: Option<f64>,
    pub max_price: Option<f64>,
    pub shop_id: Option<Uuid>,
    pub date: Option<chrono::NaiveDateTime>,
    pub notes: Option<&'a str>,
    pub tag: Option<&'a str>,
}

pub async fn get_products_filtered(
    pool: &PgPool,
    filter: ProductFilter<'_>,
) -> Result<Vec<Product>, sqlx::Error> {
    use sqlx::QueryBuilder;
    let mut builder = QueryBuilder::new(
        "SELECT id, name, price, product_volume, unit, shop_id, date, notes, tags FROM products WHERE 1=1"
    );
    if let Some(n) = filter.name {
        builder.push(" AND name = ").push_bind(n);
    }
    if let Some(u) = filter.unit {
        builder.push(" AND unit = ").push_bind(u);
    }
    if let Some(min) = filter.min_price {
        builder.push(" AND price >= ").push_bind(min);
    }
    if let Some(max) = filter.max_price {
        builder.push(" AND price <= ").push_bind(max);
    }
    if let Some(sid) = filter.shop_id {
        builder.push(" AND shop_id = ").push_bind(sid);
    }
    if let Some(d) = filter.date {
        builder.push(" AND date = ").push_bind(d);
    }
    if let Some(n) = filter.notes {
        builder.push(" AND notes = ").push_bind(n);
    }
    if let Some(t) = filter.tag {
        builder.push(" AND $1 = ANY(tags)").push_bind(t);
    }
    let query = builder.build_query_as::<Product>();
    let products = query.fetch_all(pool).await?;
    Ok(products)
}

pub async fn get_products(pool: &PgPool) -> Result<Vec<Product>, sqlx::Error> {
    let products: Vec<Product> = sqlx::query_as::<_, Product>(
        "SELECT id, name, notes, tags FROM products",
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
