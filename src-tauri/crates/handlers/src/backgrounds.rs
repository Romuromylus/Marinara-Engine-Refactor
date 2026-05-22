// Transport-agnostic background-image handlers. The Tauri shims in
// `src-tauri/src/commands/storage/backgrounds.rs` and the Axum server in
// `src-server/src/main.rs` both call into these functions, so the filesystem
// + metadata logic only exists in one place.
//
// Each function takes a `FileStorage` (for the `background-metadata`
// collection) and an `AssetService` rooted at the per-binary backgrounds
// directory (`$APPDATA/backgrounds` on Tauri, `$MARINARA_DATA_DIR/backgrounds`
// on the server). The shared `AssetService` provides the traversal guard via
// `marinara-security`, so handlers never touch raw paths from user input.

use crate::shared::{
    decode_path, decode_uploaded_file, required_string, string_array_from_value,
};
use marinara_assets::AssetService;
use marinara_core::{ensure_object, now_iso, AppError, AppResult};
use marinara_storage::FileStorage;
use serde_json::{json, Map, Value};
use std::fs;
use std::path::{Path, PathBuf};

const BACKGROUND_METADATA_COLLECTION: &str = "background-metadata";
const BACKGROUND_URL_PREFIX: &str = "marinara-background:";
const ALLOWED_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "webp", "gif", "avif"];

pub fn list(storage: &FileStorage, assets: &AssetService) -> AppResult<Value> {
    let meta_rows = storage.list(BACKGROUND_METADATA_COLLECTION)?;
    let mut rows = Vec::new();
    for item in assets.list(None)? {
        if item.get("type").and_then(Value::as_str) == Some("folder")
            || item.get("isDirectory").and_then(Value::as_bool) == Some(true)
        {
            continue;
        }
        let filename = item
            .get("name")
            .or_else(|| item.get("path"))
            .and_then(Value::as_str)
            .unwrap_or("background")
            .to_string();
        let meta = meta_rows
            .iter()
            .find(|row| row.get("filename").and_then(Value::as_str) == Some(filename.as_str()))
            .cloned()
            .unwrap_or_else(|| {
                json!({
                    "filename": filename,
                    "originalName": filename,
                    "tags": [],
                    "source": "user"
                })
            });
        rows.push(background_item(assets, &filename, &meta)?);
    }
    Ok(Value::Array(rows))
}

pub fn tags(storage: &FileStorage) -> AppResult<Value> {
    let mut tags = std::collections::BTreeSet::new();
    for row in storage.list(BACKGROUND_METADATA_COLLECTION)? {
        for tag in string_array_from_value(row.get("tags")) {
            tags.insert(tag);
        }
    }
    Ok(Value::Array(tags.into_iter().map(Value::String).collect()))
}

pub fn upload(storage: &FileStorage, assets: &AssetService, body: Value) -> AppResult<Value> {
    let (original_name, content_type, bytes) = decode_uploaded_file(&body)?;
    let filename = write_background_file(assets, &original_name, &bytes)?;
    let meta = upsert_background_meta(
        storage,
        &filename,
        json!({
            "filename": filename,
            "originalName": original_name,
            "contentType": content_type,
            "tags": [],
            "source": "user"
        }),
    )?;
    Ok(json!({
        "success": true,
        "filename": filename,
        "url": format!("{BACKGROUND_URL_PREFIX}{filename}"),
        "originalName": meta
            .get("originalName")
            .and_then(Value::as_str)
            .unwrap_or(&filename),
        "tags": [],
        "item": background_item(assets, &filename, &meta)?
    }))
}

pub fn update_tags(storage: &FileStorage, id: &str, body: Value) -> AppResult<Value> {
    let filename = decode_path(id);
    let tags = body.get("tags").cloned().unwrap_or_else(|| json!([]));
    let meta = upsert_background_meta(
        storage,
        &filename,
        json!({
            "filename": filename,
            "tags": tags,
            "source": "user"
        }),
    )?;
    Ok(json!({
        "tags": meta.get("tags").cloned().unwrap_or_else(|| json!([])),
        "item": background_item_from_meta(&filename, &meta)
    }))
}

pub fn rename(
    storage: &FileStorage,
    assets: &AssetService,
    id: &str,
    body: Value,
) -> AppResult<Value> {
    let old_filename = decode_path(id);
    let requested = required_string(&body, "name")?;
    let new_filename = safe_background_filename(requested);
    if new_filename.is_empty() {
        return Err(AppError::invalid_input("Background name is invalid"));
    }
    let old_path = assets.absolute_path(&old_filename)?;
    let new_path = unique_background_path(assets.absolute_path(&new_filename)?)?;
    fs::rename(&old_path, &new_path)?;
    let actual_name = new_path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or(new_filename);
    let old_meta = find_background_meta(storage, &old_filename)?
        .unwrap_or_else(|| json!({ "filename": old_filename, "tags": [] }));
    if let Some(id) = old_meta.get("id").and_then(Value::as_str) {
        let _ = storage.delete(BACKGROUND_METADATA_COLLECTION, id);
    }
    let meta = upsert_background_meta(
        storage,
        &actual_name,
        json!({
            "filename": actual_name,
            "originalName": old_meta
                .get("originalName")
                .cloned()
                .unwrap_or_else(|| json!(actual_name)),
            "tags": old_meta.get("tags").cloned().unwrap_or_else(|| json!([])),
            "source": "user"
        }),
    )?;
    Ok(json!({
        "success": true,
        "oldFilename": old_filename,
        "filename": actual_name,
        "url": format!("{BACKGROUND_URL_PREFIX}{actual_name}"),
        "item": background_item(assets, &actual_name, &meta)?
    }))
}

pub fn delete(storage: &FileStorage, assets: &AssetService, id: &str) -> AppResult<Value> {
    let filename = decode_path(id);
    assets.remove(&filename, false)?;
    if let Some(meta) = find_background_meta(storage, &filename)? {
        if let Some(meta_id) = meta.get("id").and_then(Value::as_str) {
            let _ = storage.delete(BACKGROUND_METADATA_COLLECTION, meta_id);
        }
    }
    Ok(json!({ "deleted": true, "filename": filename }))
}

pub fn file_path(assets: &AssetService, encoded_filename: &str) -> AppResult<Value> {
    Ok(json!({
        "path": assets.absolute_path_string(&decode_path(encoded_filename))?
    }))
}

fn background_item(assets: &AssetService, filename: &str, meta: &Value) -> AppResult<Value> {
    let path = assets.absolute_path(filename)?;
    let metadata = fs::metadata(&path)?;
    Ok(json!({
        "id": filename,
        "filename": filename,
        "name": filename,
        "path": filename,
        "absolutePath": path.to_string_lossy(),
        "url": format!("{BACKGROUND_URL_PREFIX}{filename}"),
        "originalName": meta.get("originalName").and_then(Value::as_str).unwrap_or(filename),
        "tags": meta.get("tags").cloned().unwrap_or_else(|| json!([])),
        "source": meta.get("source").and_then(Value::as_str).unwrap_or("user"),
        "type": "file",
        "isDirectory": false,
        "size": metadata.len(),
        "modified": now_iso()
    }))
}

// Used when we only need a metadata-flavored view (no fresh fs::metadata read).
fn background_item_from_meta(filename: &str, meta: &Value) -> Value {
    json!({
        "id": filename,
        "filename": filename,
        "name": filename,
        "url": format!("{BACKGROUND_URL_PREFIX}{filename}"),
        "originalName": meta.get("originalName").and_then(Value::as_str).unwrap_or(filename),
        "tags": meta.get("tags").cloned().unwrap_or_else(|| json!([])),
        "source": meta.get("source").and_then(Value::as_str).unwrap_or("user"),
    })
}

fn write_background_file(
    assets: &AssetService,
    original_name: &str,
    bytes: &[u8],
) -> AppResult<String> {
    let filename = safe_background_filename(original_name);
    if filename.is_empty() {
        return Err(AppError::invalid_input("Background filename is invalid"));
    }
    let path = unique_background_path(assets.absolute_path(&filename)?)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, bytes)?;
    Ok(path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or(filename))
}

fn upsert_background_meta(
    storage: &FileStorage,
    filename: &str,
    value: Value,
) -> AppResult<Value> {
    if let Some(existing) = find_background_meta(storage, filename)? {
        if let Some(id) = existing
            .get("id")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
        {
            let mut patch = ensure_object(value)?;
            for (key, old_value) in ensure_object(existing)? {
                patch.entry(key).or_insert(old_value);
            }
            patch.insert("filename".to_string(), Value::String(filename.to_string()));
            return storage.patch(BACKGROUND_METADATA_COLLECTION, &id, Value::Object(patch));
        }
    }
    storage.create(BACKGROUND_METADATA_COLLECTION, value)
}

fn find_background_meta(storage: &FileStorage, filename: &str) -> AppResult<Option<Value>> {
    let mut filters = Map::new();
    filters.insert("filename".to_string(), Value::String(filename.to_string()));
    Ok(storage
        .list_where(BACKGROUND_METADATA_COLLECTION, &filters)?
        .into_iter()
        .next())
}

fn safe_background_filename(name: &str) -> String {
    let path = Path::new(name);
    let stem = path
        .file_stem()
        .map(|stem| stem.to_string_lossy().to_string())
        .unwrap_or_else(|| "background".to_string())
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | ' ') {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim()
        .trim_matches('.')
        .to_string();
    let ext = path
        .extension()
        .map(|ext| ext.to_string_lossy().to_ascii_lowercase())
        .filter(|ext| ALLOWED_EXTENSIONS.contains(&ext.as_str()))
        .unwrap_or_else(|| "png".to_string());
    format!(
        "{}.{}",
        if stem.is_empty() {
            "background"
        } else {
            stem.as_str()
        },
        ext
    )
}

fn unique_background_path(path: PathBuf) -> AppResult<PathBuf> {
    if !path.exists() {
        return Ok(path);
    }
    let parent = path.parent().unwrap_or_else(|| Path::new(""));
    let stem = path
        .file_stem()
        .map(|stem| stem.to_string_lossy().to_string())
        .unwrap_or_else(|| "background".to_string());
    let ext = path
        .extension()
        .map(|ext| format!(".{}", ext.to_string_lossy()))
        .unwrap_or_default();
    for index in 1..10_000 {
        let candidate = parent.join(format!("{stem}-{index}{ext}"));
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err(AppError::invalid_input(
        "Could not find an available background filename",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine as _;
    use tempfile::TempDir;

    fn setup() -> (TempDir, FileStorage, AssetService) {
        let dir = TempDir::new().expect("temp dir");
        let storage = FileStorage::new(dir.path().join("data")).expect("storage");
        let assets =
            AssetService::new(dir.path().join("backgrounds")).expect("backgrounds asset service");
        (dir, storage, assets)
    }

    fn upload_body(name: &str, bytes: &[u8]) -> Value {
        json!({
            "file": {
                "name": name,
                "type": "image/png",
                "base64": base64::engine::general_purpose::STANDARD.encode(bytes),
            }
        })
    }

    #[test]
    fn upload_then_list_includes_uploaded_file() {
        let (_dir, storage, assets) = setup();
        let response =
            upload(&storage, &assets, upload_body("seascape.png", b"PNGDATA")).expect("upload");
        let filename = response
            .get("filename")
            .and_then(Value::as_str)
            .expect("filename")
            .to_string();
        let listing = list(&storage, &assets).expect("list");
        let array = listing.as_array().expect("array");
        assert_eq!(array.len(), 1);
        assert_eq!(
            array[0].get("filename").and_then(Value::as_str),
            Some(filename.as_str())
        );
    }

    #[test]
    fn upload_sanitizes_filename() {
        let (_dir, storage, assets) = setup();
        let response = upload(
            &storage,
            &assets,
            upload_body("../../etc/passwd.png", b"X"),
        )
        .expect("upload");
        let filename = response
            .get("filename")
            .and_then(Value::as_str)
            .expect("filename");
        assert!(!filename.contains(".."));
        assert!(!filename.contains('/'));
        assert!(filename.ends_with(".png"));
    }

    #[test]
    fn upload_replaces_disallowed_extension_with_png() {
        let (_dir, storage, assets) = setup();
        let response =
            upload(&storage, &assets, upload_body("doc.exe", b"X")).expect("upload");
        let filename = response.get("filename").and_then(Value::as_str).expect("filename");
        assert!(filename.ends_with(".png"));
    }

    #[test]
    fn update_tags_writes_metadata() {
        let (_dir, storage, assets) = setup();
        let uploaded = upload(&storage, &assets, upload_body("sunset.png", b"X")).expect("upload");
        let filename = uploaded
            .get("filename")
            .and_then(Value::as_str)
            .expect("filename")
            .to_string();
        update_tags(
            &storage,
            &filename,
            json!({ "tags": ["warm", "evening"] }),
        )
        .expect("tags");
        let all_tags = tags(&storage).expect("tags list");
        let tag_array = all_tags.as_array().expect("array");
        let tag_strings: Vec<&str> = tag_array.iter().filter_map(Value::as_str).collect();
        assert!(tag_strings.contains(&"warm"));
        assert!(tag_strings.contains(&"evening"));
    }

    #[test]
    fn rename_moves_file_and_metadata() {
        let (_dir, storage, assets) = setup();
        let uploaded = upload(&storage, &assets, upload_body("a.png", b"X")).expect("upload");
        let old = uploaded
            .get("filename")
            .and_then(Value::as_str)
            .expect("filename")
            .to_string();
        let response = rename(&storage, &assets, &old, json!({ "name": "renamed.png" }))
            .expect("rename");
        let new = response
            .get("filename")
            .and_then(Value::as_str)
            .expect("filename");
        assert_eq!(new, "renamed.png");
        let listing = list(&storage, &assets).expect("list");
        let array = listing.as_array().expect("array");
        assert_eq!(array.len(), 1);
        assert_eq!(
            array[0].get("filename").and_then(Value::as_str),
            Some("renamed.png")
        );
    }

    #[test]
    fn delete_removes_file_and_metadata() {
        let (_dir, storage, assets) = setup();
        let uploaded = upload(&storage, &assets, upload_body("d.png", b"X")).expect("upload");
        let filename = uploaded
            .get("filename")
            .and_then(Value::as_str)
            .expect("filename")
            .to_string();
        delete(&storage, &assets, &filename).expect("delete");
        let listing = list(&storage, &assets).expect("list");
        assert!(listing.as_array().unwrap().is_empty());
        let metadata_rows = storage.list(BACKGROUND_METADATA_COLLECTION).unwrap();
        assert!(metadata_rows.is_empty());
    }

    #[test]
    fn file_path_decodes_percent_encoding() {
        let (_dir, _storage, assets) = setup();
        // Create a real file the AssetService can resolve.
        let real_name = "with space.png";
        fs::write(assets.root().join(real_name), b"X").unwrap();
        let response = file_path(&assets, "with%20space.png").expect("file_path");
        let path = response
            .get("path")
            .and_then(Value::as_str)
            .expect("path");
        assert!(path.ends_with("with space.png"));
    }
}
