use marinara_core::AppResult;
use serde_json::{json, Value};

pub fn call_integration(integration: &str, operation: &str, _payload: Value) -> AppResult<Value> {
    match (integration, operation) {
        ("haptic", "status") => Ok(json!({ "connected": false, "devices": [] })),
        ("haptic", _) => Ok(json!({ "ok": false, "connected": false, "devices": [] })),
        ("spotify", "status") | ("spotify", "player") => Ok(json!({
            "connected": false,
            "isPlaying": false,
            "item": null,
            "device": null,
            "playback": null
        })),
        ("spotify", "devices") => Ok(json!({ "devices": [], "activeDeviceId": null })),
        ("spotify", _) => Ok(json!({ "ok": false, "connected": false, "devices": [], "playback": null })),
        _ => Ok(json!({ "ok": false })),
    }
}
