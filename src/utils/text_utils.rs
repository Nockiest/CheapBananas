use serde_json::{Value};

/// Recursively replaces any string value that is only underscores (e.g. "_", "__") with an empty string in a serde_json::Value
pub fn sanitize_underscores_to_empty(mut value: Value) -> Value {
    match &mut value {
        Value::String(s) => {
            if s.chars().all(|c| c == '_') {
                *s = String::new();
            }
        }
        Value::Array(arr) => {
            for v in arr.iter_mut() {
                *v = sanitize_underscores_to_empty(v.take());
            }
        }
        Value::Object(map) => {
            for (_k, v) in map.iter_mut() {
                *v = sanitize_underscores_to_empty(v.take());
            }
        }
        _ => {}
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_sanitize_simple_values() {
        let input = json!({
            "a": "_",
            "b": "hello",
            "c": ["__", "world", {"d": "___"}],
            "e": {
                "f": "value",
                "g": "_"
            }
        });

        let expected = json!({
            "a": "",
            "b": "hello",
            "c": [
                "",
                "world",
                {
                    "d": ""
                }
            ],
            "e": {
                "f": "value",
                "g": ""
            }
        });

        let result = sanitize_underscores_to_empty(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_no_underscores() {
        let input = json!({
            "x": "normal",
            "y": ["abc", {"z": "123"}]
        });

        let expected = input.clone(); // Should be unchanged
        let result = sanitize_underscores_to_empty(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_nested_empty_underscores() {
        let input = json!([
            "_",
            {
                "a": ["__", {"b": "___"}]
            }
        ]);

        let expected = json!([
            "",
            {
                "a": ["", {"b": ""}]
            }
        ]);

        let result = sanitize_underscores_to_empty(input);
        assert_eq!(result, expected);
    }
}

// async fn setup_db() -> PgPool {
//     dotenv().ok();
//     let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
//     let pool = PgPool::connect(&database_url).await.expect("Failed to connect to DB");
//     let _ = pool.execute("DELETE FROM products;").await;
//     let _ = pool.execute("DELETE FROM shops;").await;
//     pool
// }