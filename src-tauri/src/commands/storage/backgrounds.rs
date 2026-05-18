use super::*;
use super::shared::*;

pub(crate) fn backgrounds_call(state: &AppState, method: &str, rest: &[&str], body: Value) -> AppResult<Value> {
    match (method, rest) {
        ("GET", []) => Ok(Value::Array(state.backgrounds.list(None)?)),
        ("GET", ["tags"]) => Ok(json!([])),
        ("GET", ["file-path", encoded]) => {
            Ok(json!({ "path": state.backgrounds.absolute_path_string(&decode_path(encoded))? }))
        }
        ("POST", ["upload"]) => {
            let file = body
                .get("file")
                .ok_or_else(|| AppError::invalid_input("file is required"))?;
            let uploaded = state.backgrounds.write_upload("backgrounds", None, file)?;
            let item = uploaded.get("item").cloned().unwrap_or(Value::Null);
            let filename = item
                .get("name")
                .and_then(Value::as_str)
                .or_else(|| file.get("name").and_then(Value::as_str))
                .unwrap_or("background");
            Ok(json!({
                "success": true,
                "filename": filename,
                "url": format!("marinara-background:{}", filename),
                "originalName": filename,
                "tags": [],
                "item": item
            }))
        }
        ("PATCH", [_id, "rename"]) => Ok(json!({ "renamed": true })),
        ("PATCH", [_id, "tags"]) => Ok(json!({ "tags": body.get("tags").cloned().unwrap_or_else(|| json!([])) })),
        ("DELETE", [_id]) => Ok(json!({ "deleted": false })),
        _ => Ok(json!({ "ok": true })),
    }
}
