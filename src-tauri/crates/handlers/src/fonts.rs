// Transport-agnostic font handlers. The Tauri shim in
// `src-tauri/src/commands/storage/fonts.rs` and the Axum server in
// `src-server/src/main.rs` both call into these functions, so the filesystem +
// Google Fonts download logic only exists in one place.
//
// Each function takes an `AssetService` rooted at the per-binary fonts
// directory (`$APPDATA/fonts` on Tauri, `$MARINARA_DATA_DIR/fonts` on the
// server). `list` and `download_google` emit `marinara-font:<filename>` URLs;
// the frontend's `local-file-api.ts` rewrites those to `/assets/fonts/<file>`
// on the web target and back to a Tauri asset path on the desktop binary.

use crate::shared::percent_encode_component;
use base64::{engine::general_purpose, Engine as _};
use marinara_assets::AssetService;
use marinara_core::{AppError, AppResult};
use serde_json::{json, Map, Value};
use std::fs;
use std::path::Path;
use std::time::Duration;

const FONT_EXTS: &[&str] = &["ttf", "otf", "woff", "woff2"];
const FONT_URL_PREFIX: &str = "marinara-font:";

pub fn list(assets: &AssetService) -> AppResult<Value> {
    let root = assets.root();
    let metadata = read_font_metadata(root);
    let mut fonts = Vec::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let filename = entry.file_name().to_string_lossy().to_string();
        let ext = path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if !FONT_EXTS.contains(&ext.as_str()) {
            continue;
        }
        let meta = metadata
            .get(&filename)
            .cloned()
            .unwrap_or_else(|| json!({}));
        fonts.push(json!({
            "filename": filename,
            "family": meta
                .get("family")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
                .unwrap_or_else(|| font_display_name(&filename)),
            "url": font_url(&filename),
            "absolutePath": path.to_string_lossy(),
            "weight": meta
                .get("weight")
                .and_then(Value::as_str)
                .unwrap_or_else(|| infer_font_weight(&filename)),
            "style": meta
                .get("style")
                .and_then(Value::as_str)
                .unwrap_or_else(|| infer_font_style(&filename)),
            "unicodeRange": meta.get("unicodeRange").cloned().unwrap_or(Value::Null)
        }));
    }
    fonts.sort_by(|a, b| {
        let af = a.get("family").and_then(Value::as_str).unwrap_or("");
        let bf = b.get("family").and_then(Value::as_str).unwrap_or("");
        af.cmp(bf)
    });
    Ok(Value::Array(fonts))
}

pub fn file(assets: &AssetService, filename: &str) -> AppResult<Value> {
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return Err(AppError::invalid_input("Invalid font filename"));
    }
    let path = assets.absolute_path(filename)?;
    let ext = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if !FONT_EXTS.contains(&ext.as_str()) {
        return Err(AppError::invalid_input("Not a supported font file"));
    }
    if !path.exists() {
        return Err(AppError::not_found("Font file not found"));
    }
    let content_type = match ext.as_str() {
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        _ => "application/octet-stream",
    };
    Ok(json!({
        "base64": general_purpose::STANDARD.encode(fs::read(path)?),
        "contentType": content_type,
        "filename": filename
    }))
}

/// Resolve the fonts directory path. The Tauri shim layers
/// `tauri_plugin_opener::open_path` on top of this to actually pop the
/// system file manager; the server has nowhere to open it, so the returned
/// payload includes `opened: false` and the frontend treats it as a no-op.
pub fn open_folder(assets: &AssetService) -> AppResult<Value> {
    let root = assets.root();
    Ok(json!({
        "ok": true,
        "path": root.to_string_lossy(),
        "opened": false
    }))
}

pub async fn download_google(assets: &AssetService, body: Value) -> AppResult<Value> {
    let family = body
        .get("family")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| AppError::invalid_input("Font family name is required"))?;
    if family.len() > 100
        || !family
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == ' ')
    {
        return Err(AppError::invalid_input(
            "Invalid font family name. Use only letters, numbers, and spaces.",
        ));
    }

    let root = assets.root().to_path_buf();
    let faces = fetch_google_font_faces(family).await?;
    let mut metadata = read_font_metadata(&root);
    let safe_name = family.replace(' ', "");
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|error| AppError::new("font_client_error", error.to_string()))?;
    let mut files = Vec::new();
    for (index, face) in faces.iter().enumerate() {
        let suffix = if faces.len() == 1 {
            String::new()
        } else {
            format!("-{:03}", index + 1)
        };
        let filename = format!("{safe_name}-Regular{suffix}.woff2");
        let response = client
            .get(face.url.as_str())
            .send()
            .await
            .map_err(|error| AppError::new("font_download_failed", error.to_string()))?;
        if !response.status().is_success() {
            return Err(AppError::new(
                "font_download_failed",
                format!("Google Fonts returned {}", response.status()),
            ));
        }
        let bytes = response
            .bytes()
            .await
            .map_err(|error| AppError::new("font_download_failed", error.to_string()))?;
        if bytes.len() > 10 * 1024 * 1024 || bytes.get(0..4) != Some(b"wOF2") {
            return Err(AppError::new(
                "font_download_failed",
                "Downloaded file was not a valid woff2 font",
            ));
        }
        fs::write(root.join(&filename), &bytes)?;
        metadata.insert(
            filename.clone(),
            json!({
                "family": family,
                "weight": face.weight,
                "style": face.style,
                "unicodeRange": face.unicode_range,
                "source": "google"
            }),
        );
        files.push(json!({
            "filename": filename,
            "family": family,
            "url": font_url(&filename),
            "weight": face.weight,
            "style": face.style,
            "unicodeRange": face.unicode_range
        }));
    }
    write_font_metadata(&root, &metadata)?;
    Ok(json!({
        "filename": files
            .first()
            .and_then(|file| file.get("filename"))
            .cloned()
            .unwrap_or_else(|| json!(format!("{safe_name}-Regular.woff2"))),
        "family": family,
        "url": files
            .first()
            .and_then(|file| file.get("url"))
            .cloned()
            .unwrap_or(Value::Null),
        "files": files
    }))
}

fn font_url(filename: &str) -> String {
    format!("{FONT_URL_PREFIX}{}", percent_encode_component(filename))
}

#[derive(Clone, Debug)]
struct FontFace {
    url: String,
    weight: String,
    style: String,
    unicode_range: Value,
}

async fn fetch_google_font_faces(family: &str) -> AppResult<Vec<FontFace>> {
    let url = format!(
        "https://fonts.googleapis.com/css2?family={}:wght@400&display=swap",
        percent_encode_component(family)
    );
    let css = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|error| AppError::new("font_client_error", error.to_string()))?
        .get(url)
        .header(
            reqwest::header::USER_AGENT,
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/120 Safari/537.36",
        )
        .send()
        .await
        .map_err(|error| AppError::new("font_lookup_failed", error.to_string()))?;
    if !css.status().is_success() {
        return Err(AppError::new(
            "font_lookup_failed",
            format!("Google Fonts returned {}", css.status()),
        ));
    }
    parse_google_font_faces(
        &css.text()
            .await
            .map_err(|error| AppError::new("font_lookup_failed", error.to_string()))?,
    )
}

fn parse_google_font_faces(css: &str) -> AppResult<Vec<FontFace>> {
    let mut faces = Vec::new();
    for block in css.split("@font-face").skip(1) {
        let Some(start) = block.find('{') else {
            continue;
        };
        let Some(end) = block[start + 1..].find('}') else {
            continue;
        };
        let body = &block[start + 1..start + 1 + end];
        let Some(url_start) = body.find("https://fonts.gstatic.com/") else {
            continue;
        };
        let url_tail = &body[url_start..];
        let url_end = url_tail
            .find(|ch: char| ch == ')' || ch == '"' || ch == '\'' || ch.is_whitespace())
            .unwrap_or(url_tail.len());
        let url = url_tail[..url_end].to_string();
        faces.push(FontFace {
            url,
            weight: css_descriptor(body, "font-weight").unwrap_or_else(|| "400".to_string()),
            style: css_descriptor(body, "font-style").unwrap_or_else(|| "normal".to_string()),
            unicode_range: css_descriptor(body, "unicode-range")
                .map(Value::String)
                .unwrap_or(Value::Null),
        });
    }
    if faces.is_empty() {
        return Err(AppError::not_found("Font not found on Google Fonts"));
    }
    Ok(faces)
}

fn css_descriptor(body: &str, key: &str) -> Option<String> {
    let start = body.find(key)?;
    let rest = &body[start + key.len()..];
    let colon = rest.find(':')?;
    let value = &rest[colon + 1..];
    let semicolon = value.find(';')?;
    Some(
        value[..semicolon]
            .trim()
            .trim_matches('"')
            .trim_matches('\'')
            .to_string(),
    )
}

fn read_font_metadata(root: &Path) -> Map<String, Value> {
    let path = root.join("font-metadata.json");
    fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str::<Value>(&raw).ok())
        .and_then(|value| value.as_object().cloned())
        .unwrap_or_default()
}

fn write_font_metadata(root: &Path, metadata: &Map<String, Value>) -> AppResult<()> {
    fs::write(
        root.join("font-metadata.json"),
        serde_json::to_vec_pretty(metadata)?,
    )?;
    Ok(())
}

fn font_display_name(filename: &str) -> String {
    Path::new(filename)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or(filename)
        .replace(['-', '_'], " ")
        .split_whitespace()
        .filter(|part| {
            !matches!(
                part.to_ascii_lowercase().as_str(),
                "regular" | "bold" | "italic" | "light" | "medium" | "semibold" | "black" | "thin"
            )
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn infer_font_weight(filename: &str) -> &'static str {
    let lower = filename.to_ascii_lowercase();
    if lower.contains("thin") {
        "100"
    } else if lower.contains("light") {
        "300"
    } else if lower.contains("medium") {
        "500"
    } else if lower.contains("semibold") {
        "600"
    } else if lower.contains("bold") {
        "700"
    } else if lower.contains("black") {
        "900"
    } else {
        "400"
    }
}

fn infer_font_style(filename: &str) -> &'static str {
    if filename.to_ascii_lowercase().contains("italic") {
        "italic"
    } else {
        "normal"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup() -> (TempDir, AssetService) {
        let dir = TempDir::new().expect("temp dir");
        let assets = AssetService::new(dir.path().join("fonts")).expect("fonts asset service");
        (dir, assets)
    }

    fn write_dummy_font(assets: &AssetService, filename: &str, contents: &[u8]) {
        fs::write(assets.root().join(filename), contents).expect("write font");
    }

    #[test]
    fn list_returns_sorted_by_family_and_skips_non_fonts() {
        let (_dir, assets) = setup();
        write_dummy_font(&assets, "Zeta-Regular.ttf", b"TTF");
        write_dummy_font(&assets, "Alpha-Bold.otf", b"OTF");
        write_dummy_font(&assets, "readme.txt", b"hi");
        fs::create_dir_all(assets.root().join("nested")).expect("dir");
        let listing = list(&assets).expect("list");
        let array = listing.as_array().expect("array");
        assert_eq!(array.len(), 2, "non-font file and folder should be skipped");
        assert_eq!(
            array[0].get("family").and_then(Value::as_str),
            Some("Alpha")
        );
        assert_eq!(array[1].get("family").and_then(Value::as_str), Some("Zeta"));
    }

    #[test]
    fn list_emits_marinara_font_url_with_encoded_filename() {
        let (_dir, assets) = setup();
        write_dummy_font(&assets, "With Space.woff2", b"wOF2");
        let listing = list(&assets).expect("list");
        let array = listing.as_array().expect("array");
        let url = array[0].get("url").and_then(Value::as_str).expect("url");
        assert_eq!(url, "marinara-font:With%20Space.woff2");
    }

    #[test]
    fn list_prefers_persisted_metadata_over_inferred() {
        let (_dir, assets) = setup();
        write_dummy_font(&assets, "MyFont-Bold.ttf", b"TTF");
        let mut meta = Map::new();
        meta.insert(
            "MyFont-Bold.ttf".to_string(),
            json!({
                "family": "My Custom Family",
                "weight": "700",
                "style": "italic",
                "unicodeRange": "U+0000-00FF"
            }),
        );
        write_font_metadata(assets.root(), &meta).expect("write metadata");
        let listing = list(&assets).expect("list");
        let entry = &listing.as_array().unwrap()[0];
        assert_eq!(
            entry.get("family").and_then(Value::as_str),
            Some("My Custom Family")
        );
        assert_eq!(entry.get("weight").and_then(Value::as_str), Some("700"));
        assert_eq!(entry.get("style").and_then(Value::as_str), Some("italic"));
        assert_eq!(
            entry.get("unicodeRange").and_then(Value::as_str),
            Some("U+0000-00FF")
        );
    }

    #[test]
    fn file_returns_base64_with_correct_content_type() {
        let (_dir, assets) = setup();
        let bytes = b"woff2-bytes";
        write_dummy_font(&assets, "demo.woff2", bytes);
        let response = file(&assets, "demo.woff2").expect("file");
        assert_eq!(
            response.get("contentType").and_then(Value::as_str),
            Some("font/woff2")
        );
        let encoded = response
            .get("base64")
            .and_then(Value::as_str)
            .expect("base64");
        let decoded = general_purpose::STANDARD.decode(encoded).expect("decode");
        assert_eq!(decoded, bytes);
    }

    #[test]
    fn file_rejects_path_traversal() {
        let (_dir, assets) = setup();
        let err = file(&assets, "../etc/passwd").expect_err("traversal");
        assert_eq!(err.code, "invalid_input");
    }

    #[test]
    fn file_rejects_disallowed_extension() {
        let (_dir, assets) = setup();
        write_dummy_font(&assets, "demo.exe", b"X");
        let err = file(&assets, "demo.exe").expect_err("ext");
        assert_eq!(err.code, "invalid_input");
    }

    #[test]
    fn file_not_found_when_missing() {
        let (_dir, assets) = setup();
        let err = file(&assets, "ghost.ttf").expect_err("missing");
        assert_eq!(err.code, "not_found");
    }

    #[test]
    fn open_folder_returns_root_path_and_opened_false() {
        let (_dir, assets) = setup();
        let response = open_folder(&assets).expect("open_folder");
        assert_eq!(response.get("opened").and_then(Value::as_bool), Some(false));
        let path = response.get("path").and_then(Value::as_str).expect("path");
        assert!(path.ends_with("fonts"));
    }

    #[tokio::test]
    async fn download_google_rejects_empty_family() {
        let (_dir, assets) = setup();
        let err = download_google(&assets, json!({ "family": "" }))
            .await
            .expect_err("empty");
        assert_eq!(err.code, "invalid_input");
    }

    #[tokio::test]
    async fn download_google_rejects_special_characters() {
        let (_dir, assets) = setup();
        let err = download_google(&assets, json!({ "family": "Bad!Family" }))
            .await
            .expect_err("special");
        assert_eq!(err.code, "invalid_input");
    }

    #[tokio::test]
    async fn download_google_rejects_overlong_family() {
        let (_dir, assets) = setup();
        let long = "A".repeat(101);
        let err = download_google(&assets, json!({ "family": long }))
            .await
            .expect_err("overlong");
        assert_eq!(err.code, "invalid_input");
    }

    #[test]
    fn parse_google_font_faces_extracts_url_weight_style() {
        let css = r#"
            /* latin */
            @font-face {
              font-family: 'Inter';
              font-style: normal;
              font-weight: 400;
              src: url(https://fonts.gstatic.com/s/inter/v1/example.woff2) format('woff2');
              unicode-range: U+0000-00FF, U+0131;
            }
            "#;
        let faces = parse_google_font_faces(css).expect("parse");
        assert_eq!(faces.len(), 1);
        assert_eq!(
            faces[0].url,
            "https://fonts.gstatic.com/s/inter/v1/example.woff2"
        );
        assert_eq!(faces[0].weight, "400");
        assert_eq!(faces[0].style, "normal");
        assert_eq!(faces[0].unicode_range.as_str(), Some("U+0000-00FF, U+0131"));
    }

    #[test]
    fn parse_google_font_faces_errors_when_empty() {
        let err = parse_google_font_faces("").expect_err("empty css");
        assert_eq!(err.code, "not_found");
    }

    #[test]
    fn infer_weight_and_style_from_filename() {
        assert_eq!(infer_font_weight("MyFont-Bold.ttf"), "700");
        assert_eq!(infer_font_weight("MyFont-Thin.ttf"), "100");
        assert_eq!(infer_font_weight("MyFont-Regular.ttf"), "400");
        assert_eq!(infer_font_style("MyFont-Italic.ttf"), "italic");
        assert_eq!(infer_font_style("MyFont-Regular.ttf"), "normal");
    }

    #[test]
    fn font_display_name_strips_weight_words() {
        assert_eq!(font_display_name("My-Font-Bold.ttf"), "My Font");
        assert_eq!(font_display_name("Comic_Sans_Italic.otf"), "Comic Sans");
    }
}
