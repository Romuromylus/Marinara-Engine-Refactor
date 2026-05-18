use super::*;
use super::shared::*;

pub(crate) fn backup_snapshot(state: &AppState) -> AppResult<Value> {
    let mut collections = Map::new();
    for collection in BACKUP_COLLECTIONS {
        collections.insert((*collection).to_string(), Value::Array(state.storage.list(collection)?));
    }
    Ok(json!({
        "version": 1,
        "exportedAt": now_iso(),
        "runtime": "tauri",
        "collections": collections
    }))
}

pub(crate) fn backup_call(state: &AppState, method: &str, rest: &[&str], body: Value) -> AppResult<Value> {
    match (method, rest) {
        ("GET", []) => Ok(json!({ "backups": [], "profileAvailable": true })),
        ("GET", ["export-profile"]) | ("POST", ["download"]) => backup_snapshot(state),
        ("POST", ["import-profile"]) => {
            let collections = body
                .get("collections")
                .and_then(Value::as_object)
                .ok_or_else(|| AppError::invalid_input("Backup payload must include a collections object"))?;
            let mut imported = Map::new();
            for (collection, rows) in collections {
                if !BACKUP_COLLECTIONS.contains(&collection.as_str()) {
                    continue;
                }
                let rows = rows
                    .as_array()
                    .ok_or_else(|| AppError::invalid_input(format!("{collection} must be an array")))?
                    .clone();
                let count = rows.len();
                state.storage.replace_all(collection, rows)?;
                imported.insert(collection.clone(), json!(count));
            }
            Ok(json!({ "success": true, "imported": imported }))
        }
        _ => Err(AppError::new("route_not_found", format!("backup route {method} /{} is not implemented", rest.join("/")))),
    }
}
