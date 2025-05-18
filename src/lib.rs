pub mod db;
pub mod models;

// Re-export all DB functions and models for integration tests
pub use db::{add_product, add_shop, delete_product, delete_shop, get_products_filtered, update_product, get_products};
pub use models::{Product, Unit, Shop};
