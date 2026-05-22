// Transport-agnostic image-upload helpers shared by the Tauri shims and the
// Axum server. Pre-lift these lived in `src-tauri/src/commands/storage/media_uploads.rs`
// and took the Tauri `AppState`. The lifted versions take an explicit
// `data_dir: &Path` so the same code runs from either binary.

use base64::{engine::general_purpose, Engine as _};
use marinara_core::{new_id, now_millis, AppError, AppResult};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

pub struct StoredImageUpload {
    pub data_url: String,
    pub absolute_path: String,
    pub filename: String,
}

pub fn persist_image_upload(
    data_dir: &Path,
    folder: &str,
    id: &str,
    body: &Value,
    field_name: &str,
) -> AppResult<StoredImageUpload> {
    let image = body
        .get(field_name)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| AppError::invalid_input(format!("{field_name} is required")))?;
    let (mime, bytes) = decode_image_payload(image, field_name)?;
    let ext = extension_for_image_mime(&mime)
        .or_else(|| {
            body.get("filename")
                .and_then(Value::as_str)
                .and_then(extension_from_filename)
        })
        .unwrap_or("png");
    let filename = body
        .get("filename")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(safe_filename)
        .unwrap_or_else(|| format!("{}-{}.{}", safe_filename(id), now_millis(), ext));
    let dir = data_dir.join(folder);
    fs::create_dir_all(&dir)?;
    let target = unique_file_path(&dir.join(&filename))?;
    fs::write(&target, &bytes)?;
    Ok(StoredImageUpload {
        data_url: format!(
            "data:{mime};base64,{}",
            general_purpose::STANDARD.encode(bytes)
        ),
        absolute_path: target.to_string_lossy().to_string(),
        filename: target
            .file_name()
            .map(|value| value.to_string_lossy().to_string())
            .unwrap_or(filename),
    })
}

pub fn remove_managed_record_file(
    data_dir: &Path,
    folder: &str,
    record: &Value,
    path_key: &str,
    filename_key: &str,
) {
    let Ok(Some(path)) = managed_record_file_path(data_dir, folder, record, path_key, filename_key)
    else {
        return;
    };
    if path.exists() && path.is_file() {
        if let Err(error) = fs::remove_file(&path) {
            let record_id = record
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or("<unknown>");
            eprintln!(
                "warn: failed to remove managed file for {folder}/{record_id} at {}: {error}",
                path.display()
            );
        }
    }
}

fn managed_record_file_path(
    data_dir: &Path,
    folder: &str,
    record: &Value,
    path_key: &str,
    filename_key: &str,
) -> AppResult<Option<PathBuf>> {
    let managed_dir = data_dir.join(folder);
    let candidate = record
        .get(filename_key)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(|filename| managed_dir.join(safe_filename(filename)))
        .or_else(|| {
            record
                .get(path_key)
                .and_then(Value::as_str)
                .filter(|value| !value.trim().is_empty())
                .map(PathBuf::from)
        });
    let Some(candidate) = candidate else {
        return Ok(None);
    };
    if !candidate.exists() {
        return Ok(None);
    }
    if !is_path_inside_dir(&candidate, &managed_dir)? {
        return Ok(None);
    }
    Ok(Some(candidate))
}

fn is_path_inside_dir(path: &Path, dir: &Path) -> AppResult<bool> {
    let dir = match fs::canonicalize(dir) {
        Ok(dir) => dir,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(AppError::from(error)),
    };
    let path = match fs::canonicalize(path) {
        Ok(path) => path,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(AppError::from(error)),
    };
    Ok(path.starts_with(dir))
}

pub fn decode_image_payload(value: &str, field_name: &str) -> AppResult<(String, Vec<u8>)> {
    if let Some((header, payload)) = value.split_once(',') {
        if header.starts_with("data:") {
            let mime = header
                .trim_start_matches("data:")
                .split(';')
                .next()
                .filter(|value| !value.trim().is_empty())
                .unwrap_or("image/png")
                .to_string();
            let bytes = general_purpose::STANDARD.decode(payload).map_err(|error| {
                AppError::invalid_input(format!("Invalid {field_name} data: {error}"))
            })?;
            return Ok((mime, bytes));
        }
    }
    let bytes = general_purpose::STANDARD
        .decode(value)
        .map_err(|error| AppError::invalid_input(format!("Invalid {field_name} data: {error}")))?;
    Ok(("image/png".to_string(), bytes))
}

pub fn extension_for_image_mime(mime: &str) -> Option<&'static str> {
    match mime.to_ascii_lowercase().as_str() {
        "image/jpeg" | "image/jpg" => Some("jpg"),
        "image/webp" => Some("webp"),
        "image/gif" => Some("gif"),
        "image/avif" => Some("avif"),
        "image/png" => Some("png"),
        "image/svg+xml" => Some("svg"),
        _ => None,
    }
}

pub fn extension_from_filename(filename: &str) -> Option<&'static str> {
    match Path::new(filename)
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "jpg" | "jpeg" => Some("jpg"),
        "webp" => Some("webp"),
        "gif" => Some("gif"),
        "avif" => Some("avif"),
        "png" => Some("png"),
        "svg" => Some("svg"),
        _ => None,
    }
}

pub fn safe_filename(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string();
    if sanitized.is_empty() {
        new_id()
    } else {
        sanitized
    }
}

pub fn unique_file_path(target: &Path) -> AppResult<PathBuf> {
    if !target.exists() {
        return Ok(target.to_path_buf());
    }
    let parent = target.parent().unwrap_or_else(|| Path::new(""));
    let stem = target
        .file_stem()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(new_id);
    let ext = target
        .extension()
        .map(|value| format!(".{}", value.to_string_lossy()))
        .unwrap_or_default();
    for index in 1..10_000 {
        let candidate = parent.join(format!("{stem}-{index}{ext}"));
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err(AppError::invalid_input("Could not allocate image filename"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::TempDir;

    #[test]
    fn safe_filename_replaces_disallowed_chars() {
        assert_eq!(safe_filename("foo/bar"), "foo_bar");
        // Slashes become underscores; dots (which are alphanumeric-adjacent in
        // the allowlist) are retained but the trim-on-underscore drops the
        // leading/trailing separator we just introduced.
        assert_eq!(safe_filename("../etc/passwd"), ".._etc_passwd");
        assert_eq!(safe_filename("safe-name_1.png"), "safe-name_1.png");
        // Empty / pathologically scrubbed names fall through to a generated id.
        let scrubbed = safe_filename("/////");
        assert!(!scrubbed.is_empty());
        assert!(!scrubbed.contains('/'));
    }

    #[test]
    fn decode_image_payload_accepts_data_url() {
        let data_url = format!(
            "data:image/jpeg;base64,{}",
            general_purpose::STANDARD.encode(b"FAKEJPG")
        );
        let (mime, bytes) = decode_image_payload(&data_url, "avatar").expect("decode");
        assert_eq!(mime, "image/jpeg");
        assert_eq!(bytes, b"FAKEJPG");
    }

    #[test]
    fn decode_image_payload_accepts_raw_base64() {
        let raw = general_purpose::STANDARD.encode(b"FAKEPNG");
        let (mime, bytes) = decode_image_payload(&raw, "avatar").expect("decode");
        assert_eq!(mime, "image/png");
        assert_eq!(bytes, b"FAKEPNG");
    }

    #[test]
    fn unique_file_path_allocates_suffix_when_taken() {
        let dir = TempDir::new().expect("temp dir");
        let target = dir.path().join("photo.png");
        fs::write(&target, b"X").expect("seed file");
        let next = unique_file_path(&target).expect("path");
        assert_eq!(
            next.file_name().and_then(|name| name.to_str()),
            Some("photo-1.png")
        );
    }

    #[test]
    fn persist_image_upload_writes_to_data_dir() {
        let dir = TempDir::new().expect("temp dir");
        let body = json!({
            "image": format!(
                "data:image/png;base64,{}",
                general_purpose::STANDARD.encode(b"PNG")
            ),
            "filename": "portrait.png"
        });
        let stored =
            persist_image_upload(dir.path(), "avatars", "char-1", &body, "image").expect("persist");
        assert_eq!(stored.filename, "portrait.png");
        let absolute = PathBuf::from(stored.absolute_path);
        assert!(absolute.exists(), "file should be written to disk");
        assert!(
            absolute.starts_with(dir.path().join("avatars")),
            "should be under avatars subdir"
        );
    }
}
