use marinara_core::AppResult;
use serde_json::{json, Value};

pub fn review_import(path_or_token: &str) -> AppResult<Value> {
    Ok(json!({
        "path": path_or_token,
        "items": [],
        "warnings": ["Import review UI is wired; concrete import parsers are migrated separately."]
    }))
}
