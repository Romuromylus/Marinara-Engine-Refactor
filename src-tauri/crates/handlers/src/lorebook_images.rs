// Transport-agnostic lorebook-image handlers. Pre-lift this lived in
// `src-tauri/src/commands/storage/lorebook_images.rs` and used the Tauri
// AppState. The lifted version takes (`storage: &FileStorage`,
// `data_dir: &Path`) like the rest of the Phase 3e handlers.

use crate::media_uploads::{persist_image_upload, remove_managed_record_file, safe_filename};
use crate::shared::{decode_path, percent_encode_component};
use marinara_core::{now_iso, AppError, AppResult};
use marinara_storage::FileStorage;
use serde_json::{json, Value};
use std::path::Path;

const LOREBOOK_IMAGE_PREFIX: &str = "marinara-lorebook-image:";

pub fn update_lorebook_image(
    storage: &FileStorage,
    data_dir: &Path,
    lorebook_id: &str,
    body: Value,
) -> AppResult<Value> {
    let previous = get_required_lorebook(storage, lorebook_id)?;
    let stored = persist_image_upload(data_dir, "lorebooks/images", lorebook_id, &body, "image")?;
    let updated = storage.patch(
        "lorebooks",
        lorebook_id,
        json!({
            "imagePath": format!(
                "{LOREBOOK_IMAGE_PREFIX}{}",
                percent_encode_component(&stored.filename)
            ),
            "imageFilePath": stored.absolute_path,
            "imageFilename": stored.filename,
            "imageUpdatedAt": now_iso()
        }),
    )?;
    remove_lorebook_image_file(data_dir, &previous);
    Ok(updated)
}

pub fn remove_lorebook_image_file(data_dir: &Path, record: &Value) {
    remove_managed_record_file(
        data_dir,
        "lorebooks/images",
        record,
        "imageFilePath",
        "imageFilename",
    )
}

pub fn lorebook_image_file_path(data_dir: &Path, encoded_filename: &str) -> AppResult<Value> {
    let filename = safe_filename(&decode_path(encoded_filename));
    let path = data_dir.join("lorebooks").join("images").join(filename);
    if !path.exists() || !path.is_file() {
        return Err(AppError::not_found("Lorebook image was not found"));
    }
    Ok(json!({ "path": path.to_string_lossy() }))
}

fn get_required_lorebook(storage: &FileStorage, lorebook_id: &str) -> AppResult<Value> {
    storage
        .get("lorebooks", lorebook_id)?
        .ok_or_else(|| AppError::not_found(format!("lorebooks/{lorebook_id} was not found")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{engine::general_purpose, Engine as _};
    use std::fs;
    use tempfile::TempDir;

    fn setup() -> (TempDir, FileStorage) {
        let dir = TempDir::new().expect("temp dir");
        let storage = FileStorage::new(dir.path().join("data")).expect("storage");
        (dir, storage)
    }

    fn upload_body(name: &str, bytes: &[u8]) -> Value {
        json!({
            "image": format!(
                "data:image/png;base64,{}",
                general_purpose::STANDARD.encode(bytes)
            ),
            "filename": name
        })
    }

    #[test]
    fn update_lorebook_image_writes_file_and_patches_record() {
        let (dir, storage) = setup();
        let lorebook = storage
            .create("lorebooks", json!({ "name": "Test" }))
            .expect("create lorebook");
        let id = lorebook["id"].as_str().expect("id").to_string();

        let updated =
            update_lorebook_image(&storage, dir.path(), &id, upload_body("cover.png", b"PNG"))
                .expect("update");
        let image_path = updated["imagePath"].as_str().expect("imagePath");
        assert!(image_path.starts_with(LOREBOOK_IMAGE_PREFIX));
        assert!(image_path.ends_with("cover.png"));
        let abs = updated["imageFilePath"].as_str().expect("imageFilePath");
        assert!(std::path::Path::new(abs).exists());
        assert!(abs.contains("lorebooks"));
    }

    #[test]
    fn lorebook_image_file_path_decodes_and_sanitizes_filename() {
        let (dir, _storage) = setup();
        let images_dir = dir.path().join("lorebooks").join("images");
        fs::create_dir_all(&images_dir).unwrap();
        // `safe_filename` rewrites the decoded space to `_`, matching what
        // `persist_image_upload` would have written when the lorebook image
        // was uploaded.
        let target = images_dir.join("with_space.png");
        fs::write(&target, b"X").unwrap();
        let resolved = lorebook_image_file_path(dir.path(), "with%20space.png").expect("file_path");
        assert!(resolved["path"]
            .as_str()
            .unwrap()
            .ends_with("with_space.png"));
    }

    #[test]
    fn lorebook_image_file_path_returns_not_found_when_missing() {
        let (dir, _storage) = setup();
        let err = lorebook_image_file_path(dir.path(), "missing.png")
            .expect_err("missing file should 404");
        assert_eq!(err.code, "not_found");
    }
}
