use marinara_core::AppResult;
use serde_json::{json, Value};

pub fn check_updates() -> AppResult<Value> {
    Ok(json!({
        "available": false,
        "currentVersion": env!("CARGO_PKG_VERSION"),
        "latestVersion": null,
        "notes": null
    }))
}
