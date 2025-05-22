use serde_json::Value;

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
