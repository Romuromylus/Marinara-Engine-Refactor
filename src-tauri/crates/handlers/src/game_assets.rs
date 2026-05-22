// Transport-agnostic game-asset handlers. The Tauri desktop shims and the Axum
// server both delegate here so the game-assets folder tree, manifest, and CRUD
// operations live in one place. Each function takes an `AssetService` rooted at
// `$APPDATA/game-assets` on Tauri or `$MARINARA_DATA_DIR/game-assets` on the
// server — the AssetService owns the traversal guard via marinara-security.
//
// `open_folder` is intentionally a no-op-on-server: it returns the resolved
// path with `opened: false`, leaving the Tauri shim to layer on the
// `tauri_plugin_opener::open_path` call. Anything Tauri-specific stays in the
// Tauri shim file; this crate must compile without the Tauri dep tree.

use marinara_assets::AssetService;
use marinara_core::{AppError, AppResult};
use serde_json::{json, Value};
use std::fs;

pub fn list(assets: &AssetService, path: Option<&str>) -> AppResult<Value> {
    Ok(json!({
        "items": assets.list(path)?,
        "root": assets.root().to_string_lossy()
    }))
}

pub fn manifest(assets: &AssetService) -> AppResult<Value> {
    assets.manifest()
}

pub fn tree(assets: &AssetService) -> AppResult<Value> {
    assets.tree()
}

/// Rebuilds the manifest. Same return shape as the pre-lift command. Doesn't
/// touch disk beyond what `manifest()` already does — the in-process state
/// isn't cached, so "rescan" and "manifest" return identical data; the name
/// is preserved because the FE calls it after upload/delete batches to refresh
/// its view.
pub fn rescan(assets: &AssetService) -> AppResult<Value> {
    let manifest = assets.manifest()?;
    Ok(json!({ "ok": true, "manifest": manifest }))
}

pub fn create_folder(assets: &AssetService, path: &str) -> AppResult<Value> {
    assets.create_folder(path)?;
    Ok(json!({ "path": path }))
}

pub fn delete_folder(assets: &AssetService, path: &str, recursive: bool) -> AppResult<Value> {
    assets.remove(path, recursive)?;
    Ok(json!({ "deleted": true }))
}

pub fn delete_file(assets: &AssetService, path: &str) -> AppResult<Value> {
    assets.remove(path, false)?;
    Ok(json!({ "deleted": true }))
}

pub fn file_path(assets: &AssetService, path: &str) -> AppResult<Value> {
    Ok(json!({ "path": assets.absolute_path_string(path)? }))
}

pub fn read_text(assets: &AssetService, path: &str) -> AppResult<Value> {
    Ok(json!({ "content": assets.read_text(path)? }))
}

pub fn write_text(assets: &AssetService, path: &str, content: &str) -> AppResult<Value> {
    assets.write_text(path, content)?;
    Ok(json!({ "saved": true }))
}

pub fn rename(assets: &AssetService, path: &str, new_name: &str) -> AppResult<Value> {
    assets.rename(path, new_name)
}

pub fn move_one(assets: &AssetService, path: &str, target_folder: &str) -> AppResult<Value> {
    assets.move_to_folder(path, target_folder)
}

pub fn copy_one(assets: &AssetService, path: &str, target_folder: &str) -> AppResult<Value> {
    assets.copy_to_folder(path, target_folder)
}

pub fn move_bulk(assets: &AssetService, paths: &[String], target_folder: &str) -> AppResult<Value> {
    Ok(assets.move_many(paths, target_folder))
}

pub fn copy_bulk(assets: &AssetService, paths: &[String], target_folder: &str) -> AppResult<Value> {
    Ok(assets.copy_many(paths, target_folder))
}

pub fn delete_bulk(assets: &AssetService, paths: &[String]) -> AppResult<Value> {
    Ok(assets.delete_many(paths))
}

pub fn file_info(assets: &AssetService, path: &str) -> AppResult<Value> {
    assets.file_info(path)
}

pub fn folder_description(
    assets: &AssetService,
    path: &str,
    description: &str,
) -> AppResult<Value> {
    assets.set_folder_description(path, description)
}

pub fn upload(assets: &AssetService, body: Value) -> AppResult<Value> {
    let category = body
        .get("category")
        .and_then(Value::as_str)
        .ok_or_else(|| AppError::invalid_input("category is required"))?;
    let subcategory = body.get("subcategory").and_then(Value::as_str);
    let file = body
        .get("file")
        .ok_or_else(|| AppError::invalid_input("file is required"))?;
    assets.write_upload(category, subcategory, file)
}

/// Resolves the target folder path and ensures it exists. The Tauri shim
/// follows up with `tauri_plugin_opener::open_path`; the server has no native
/// shell to open, so it returns `opened: false` and the FE can decide whether
/// to surface a "not available on web" notice or just hide the button.
pub fn open_folder(assets: &AssetService, subfolder: Option<&str>) -> AppResult<Value> {
    let target = subfolder.unwrap_or("");
    let path = assets.absolute_path(target)?;
    if !path.exists() {
        fs::create_dir_all(&path)?;
    }
    Ok(json!({
        "ok": true,
        "path": path.to_string_lossy(),
        "opened": false,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup() -> (TempDir, AssetService) {
        let dir = TempDir::new().expect("temp dir");
        let assets = AssetService::new(dir.path().join("game-assets")).expect("asset service");
        (dir, assets)
    }

    #[test]
    fn create_then_list_includes_folder() {
        let (_dir, assets) = setup();
        create_folder(&assets, "music").expect("create_folder");
        let response = list(&assets, None).expect("list");
        let items = response
            .get("items")
            .and_then(Value::as_array)
            .expect("items array");
        assert_eq!(items.len(), 1);
        assert_eq!(
            items[0].get("name").and_then(Value::as_str),
            Some("music")
        );
    }

    #[test]
    fn write_then_read_text_roundtrips() {
        let (_dir, assets) = setup();
        write_text(&assets, "notes.txt", "hello world").expect("write");
        let response = read_text(&assets, "notes.txt").expect("read");
        assert_eq!(
            response.get("content").and_then(Value::as_str),
            Some("hello world")
        );
    }

    #[test]
    fn write_text_rejects_oversized_file() {
        let (_dir, assets) = setup();
        // The AssetService enforces a 1 MB cap on text writes.
        let huge = "x".repeat(2 * 1024 * 1024);
        assert!(write_text(&assets, "huge.txt", &huge).is_err());
    }

    #[test]
    fn delete_bulk_returns_per_path_results() {
        let (_dir, assets) = setup();
        write_text(&assets, "a.txt", "1").unwrap();
        write_text(&assets, "b.txt", "2").unwrap();
        let response = delete_bulk(&assets, &["a.txt".into(), "b.txt".into()]).expect("delete_bulk");
        let succeeded = response
            .get("succeeded")
            .and_then(Value::as_array)
            .expect("succeeded array");
        assert_eq!(succeeded.len(), 2);
    }

    #[test]
    fn rename_moves_file_to_new_name() {
        let (_dir, assets) = setup();
        write_text(&assets, "old.txt", "x").unwrap();
        rename(&assets, "old.txt", "new.txt").expect("rename");
        let response = list(&assets, None).expect("list");
        let items = response.get("items").and_then(Value::as_array).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].get("name").and_then(Value::as_str), Some("new.txt"));
    }

    #[test]
    fn open_folder_creates_subfolder_and_reports_opened_false() {
        let (_dir, assets) = setup();
        let response = open_folder(&assets, Some("nested/dir")).expect("open_folder");
        assert_eq!(response.get("opened").and_then(Value::as_bool), Some(false));
        let path = response.get("path").and_then(Value::as_str).unwrap();
        assert!(std::path::Path::new(path).exists());
    }

    #[test]
    fn rescan_includes_manifest() {
        let (_dir, assets) = setup();
        write_text(&assets, "a.txt", "hi").unwrap();
        let response = rescan(&assets).expect("rescan");
        assert_eq!(response.get("ok").and_then(Value::as_bool), Some(true));
        let manifest = response
            .get("manifest")
            .and_then(Value::as_object)
            .expect("manifest object");
        assert!(manifest.get("assets").is_some());
    }

    #[test]
    fn upload_rejects_missing_category() {
        let (_dir, assets) = setup();
        let result = upload(
            &assets,
            json!({ "file": { "name": "x.png", "type": "image/png", "base64": "" } }),
        );
        assert!(result.is_err());
    }
}
