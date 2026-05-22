// Transport-agnostic profile snapshot + import handlers. Pre-lift these lived
// in `src-tauri/src/commands/storage/profile.rs` and `profile/{assets,legacy,
// zip_import}.rs` and used the Tauri AppState. The lifted versions take an
// explicit (`storage: &FileStorage`, `data_dir: &Path`) pair, with the
// asset-restoration helpers in `assets` dropping `storage` entirely since
// they only touch the filesystem.

pub mod assets;
pub mod legacy;
pub mod zip_import;

use self::assets::{profile_assets, restore_profile_assets};
use self::legacy::import_legacy_profile_tables;
use self::zip_import::import_profile_zip;
use marinara_core::{now_iso, AppError, AppResult};
use marinara_storage::FileStorage;
use serde_json::{json, Map, Value};
use std::fs::File;
use std::path::{Path, PathBuf};

const PROFILE_COLLECTIONS: &[&str] = &[
    "characters",
    "character-groups",
    "character-versions",
    "personas",
    "persona-groups",
    "lorebooks",
    "lorebook-entries",
    "lorebook-folders",
    "prompts",
    "prompt-groups",
    "prompt-sections",
    "prompt-variables",
    "chat-presets",
    "agents",
    "agent-runs",
    "agent-memory",
    "themes",
    "extensions",
    "connections",
    "connection-folders",
    "chats",
    "chat-folders",
    "messages",
    "custom-tools",
    "regex-scripts",
    "app-settings",
    "gallery",
    "character-gallery",
    "background-metadata",
    "sprites",
    "knowledge-sources",
    "game-state-snapshots",
    "game-checkpoints",
];

pub fn profile_snapshot(storage: &FileStorage, data_dir: &Path) -> AppResult<Value> {
    Ok(json!({
        "type": "marinara_profile",
        "version": 1,
        "exportedAt": now_iso(),
        "runtime": "tauri",
        "data": {
            "collections": profile_collections(storage)?,
            "assets": profile_assets(data_dir)?,
        }
    }))
}

pub fn import_profile_file_path(
    storage: &FileStorage,
    data_dir: &Path,
    value: &str,
) -> AppResult<Value> {
    let path = PathBuf::from(value.trim());
    if path.as_os_str().is_empty() {
        return Err(AppError::invalid_input("Profile file path is required"));
    }
    if !path.is_file() {
        return Err(AppError::invalid_input("Profile import path is not a file"));
    }
    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.to_ascii_lowercase())
        .as_deref()
    {
        Some("json") => import_profile(
            storage,
            data_dir,
            serde_json::from_reader(File::open(path)?)?,
        ),
        Some("zip") => import_profile_zip(storage, data_dir, &path),
        _ => Err(AppError::invalid_input(
            "Profile import must be a .json or .zip file",
        )),
    }
}

pub fn profile_call(
    storage: &FileStorage,
    data_dir: &Path,
    method: &str,
    rest: &[&str],
    format: Option<&str>,
    body: Value,
) -> AppResult<Value> {
    match (method, rest) {
        ("GET", ["export"]) => export_profile(storage, data_dir, format),
        ("POST", ["import"]) => import_profile(storage, data_dir, body),
        _ => Err(AppError::new(
            "route_not_found",
            format!("Unknown profile route: {method} /{}", rest.join("/")),
        )),
    }
}

pub fn export_profile(
    storage: &FileStorage,
    data_dir: &Path,
    format: Option<&str>,
) -> AppResult<Value> {
    match format {
        Some("native") | None => profile_snapshot(storage, data_dir),
        Some(_) => Err(AppError::invalid_input(
            "Only native Marinara profile JSON export is supported.",
        )),
    }
}

pub fn import_profile(storage: &FileStorage, data_dir: &Path, body: Value) -> AppResult<Value> {
    let data = body
        .get("data")
        .and_then(Value::as_object)
        .filter(|_| body.get("type").and_then(Value::as_str) == Some("marinara_profile"))
        .ok_or_else(|| AppError::invalid_input("Invalid Marinara profile export"))?;
    if let Some(collections) = data.get("collections").and_then(Value::as_object) {
        return import_profile_collections(storage, data_dir, data, collections);
    }
    let tables = data
        .get("fileStorage")
        .and_then(|value| value.get("tables"))
        .and_then(Value::as_object)
        .ok_or_else(|| {
            AppError::invalid_input(
                "Profile export must contain data.collections or data.fileStorage.tables",
            )
        })?;
    import_legacy_profile_tables(storage, data_dir, data, tables)
}

fn import_profile_collections(
    storage: &FileStorage,
    data_dir: &Path,
    data: &Map<String, Value>,
    collections: &Map<String, Value>,
) -> AppResult<Value> {
    let restored_assets = restore_profile_assets(data_dir, data.get("assets"))?;
    import_profile_collections_with_restored_assets(storage, data_dir, collections, restored_assets)
}

pub fn import_profile_collections_with_restored_assets(
    storage: &FileStorage,
    _data_dir: &Path,
    collections: &Map<String, Value>,
    restored_assets: usize,
) -> AppResult<Value> {
    let mut imported = Map::new();
    for collection in PROFILE_COLLECTIONS {
        let rows = collections
            .get(*collection)
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        storage.replace_all(collection, rows.clone())?;
        imported.insert((*collection).to_string(), json!(rows.len()));
    }
    imported.insert("files".to_string(), json!(restored_assets));
    insert_profile_import_aliases(&mut imported);
    Ok(json!({ "success": true, "imported": imported }))
}

fn insert_profile_import_aliases(imported: &mut Map<String, Value>) {
    if let Some(value) = imported.get("prompts").cloned() {
        imported.insert("presets".to_string(), value);
    }
}

fn profile_collections(storage: &FileStorage) -> AppResult<Map<String, Value>> {
    let mut collections = Map::new();
    for collection in PROFILE_COLLECTIONS {
        collections.insert(
            (*collection).to_string(),
            Value::Array(storage.list(collection)?),
        );
    }
    Ok(collections)
}
