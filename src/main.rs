mod models;
mod db;
mod app;
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

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    dotenv::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url).await?;
    let shared_pool = Arc::new(pool);

    let app = app::build_app_router(shared_pool.clone());

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    serve(listener, app).await.unwrap();
    Ok(())
}
