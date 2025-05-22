// Tests for ProductEntry-related DB logic

use backend::db::*;
use backend::models::{Product, ProductEntry, Unit, Shop};
use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;
use backend::utils::text_utils::sanitize_underscores_to_empty;

// ...existing code for ProductEntry tests will be inserted here...

#[tokio::test]
#[serial_test::serial]
async fn test_add_product_entry() {
    // ...existing code for test_add_product_entry...
}

#[tokio::test]
#[serial_test::serial]
async fn test_get_product_entries_filtered_endpoint() {
    // ...existing code for test_get_product_entries_filtered_endpoint...
}

#[tokio::test]
#[serial_test::serial]
async fn test_sanitize_underscores_to_empty_on_required_fields() {
    // ...existing code for test_sanitize_underscores_to_empty_on_required_fields...
}
