
use sqlx::PgPool;

use sqlx::Executor;
use dotenv::dotenv;
pub async fn setup_db() -> PgPool {
    dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url).await.expect("Failed to connect to DB");
    let _ = pool.execute("DELETE FROM products;").await;
    let _ = pool.execute("DELETE FROM shops;").await;
    pool
}

