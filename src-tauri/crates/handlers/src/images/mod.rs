// Transport-agnostic image-generation handlers shared by the Tauri desktop
// binary and the Axum server (Phase 4c). The provider-specific HTTP wrappers
// live in `providers.rs`; this module is the public surface that
// `commands/storage/images.rs` (Tauri) and `src-server/src/main.rs` (Axum)
// both dispatch into.
//
// What stays Tauri-only: sprite cleanup (needs Python rembg/cleanlab — better
// as a sibling service per the project plan) and font downloads (Phase 3c).

use crate::shared::{get_required, required_string};
use marinara_core::{now_millis, AppError, AppResult};
use marinara_storage::FileStorage;
use serde_json::{json, Value};

pub mod providers;

pub use providers::{
    connection_base_url as image_connection_base_url, generate_image_with_connection,
    generate_image_with_options, image_source, is_openai_gpt_image_model, ImageGenerationOptions,
};

pub fn avatar_generation_prompt_id(name: &str) -> String {
    let slug: String = name
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    format!("avatar-{}", slug.trim_matches('-'))
}

pub fn avatar_generation_prompt(body: &Value) -> String {
    let name = body
        .get("name")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("Character");
    let appearance = body
        .get("appearance")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("distinctive character portrait");
    format!(
        "Portrait avatar of {name}. {appearance}. Centered bust portrait, expressive face, clean background, high detail, polished character art."
    )
}

pub fn image_dimension(body: &Value, key: &str, fallback: u64) -> u64 {
    body.get(key)
        .and_then(Value::as_u64)
        .filter(|value| (128..=2048).contains(value))
        .unwrap_or(fallback)
}

pub fn avatar_generation_preview(body: Value) -> AppResult<Value> {
    let name = body
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or("Character");
    let prompt = avatar_generation_prompt(&body);
    let title_name = if name.trim().is_empty() {
        "Character"
    } else {
        name.trim()
    };
    Ok(json!({
        "items": [{
            "id": avatar_generation_prompt_id(name),
            "kind": "avatar",
            "title": format!("Avatar: {}", title_name),
            "prompt": prompt,
            "width": image_dimension(&body, "width", 768),
            "height": image_dimension(&body, "height", 1024)
        }]
    }))
}

pub fn prompt_override(body: &Value, id: &str) -> Option<String> {
    body.get("promptOverrides")
        .and_then(Value::as_array)
        .and_then(|items| {
            items.iter().find_map(|item| {
                let item_id = item.get("id").and_then(Value::as_str)?;
                let prompt = item.get("prompt").and_then(Value::as_str)?.trim();
                if item_id == id && !prompt.is_empty() {
                    Some(prompt.to_string())
                } else {
                    None
                }
            })
        })
}

pub fn image_generation_options(body: &Value) -> ImageGenerationOptions {
    let negative_prompt = body
        .get("negativePrompt")
        .or_else(|| body.get("negative_prompt"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    let mut reference_images = Vec::new();
    if let Some(value) = body.get("referenceImage").and_then(Value::as_str) {
        if !value.trim().is_empty() {
            reference_images.push(value.trim().to_string());
        }
    }
    if let Some(items) = body.get("referenceImages").and_then(Value::as_array) {
        reference_images.extend(
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
        );
    }
    ImageGenerationOptions {
        negative_prompt,
        reference_images,
        transparent_background: body
            .get("transparentBackground")
            .or_else(|| body.get("transparent_background"))
            .or_else(|| body.get("nativeTransparentPng"))
            .and_then(Value::as_bool)
            .unwrap_or(false),
    }
}

pub async fn avatar_generation(storage: &FileStorage, body: Value) -> AppResult<Value> {
    let connection_id = required_string(&body, "connectionId")?;
    let connection = get_required(storage, "connections", connection_id)?;
    if connection.get("provider").and_then(Value::as_str) != Some("image_generation") {
        return Err(AppError::invalid_input(
            "Selected connection is not an image-generation connection",
        ));
    }
    let name = body
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or("Character");
    let prompt_id = avatar_generation_prompt_id(name);
    let prompt =
        prompt_override(&body, &prompt_id).unwrap_or_else(|| avatar_generation_prompt(&body));
    let width = image_dimension(&body, "width", 768);
    let height = image_dimension(&body, "height", 1024);
    let (base64, mime_type) = generate_image_with_options(
        &connection,
        &prompt,
        width,
        height,
        image_generation_options(&body),
    )
    .await?;
    Ok(json!({
        "image": format!("data:{mime_type};base64,{base64}"),
        "prompt": prompt
    }))
}

pub async fn generate_image(storage: &FileStorage, body: Value) -> AppResult<Value> {
    let connection_id = required_string(&body, "connectionId")?;
    let prompt = required_string(&body, "prompt")?;
    let width = image_dimension(&body, "width", 1024);
    let height = image_dimension(&body, "height", 1024);
    let connection = get_required(storage, "connections", connection_id)?;
    let (base64, mime_type) = generate_image_with_options(
        &connection,
        prompt,
        width,
        height,
        image_generation_options(&body),
    )
    .await?;
    Ok(json!({
        "base64": base64,
        "mimeType": mime_type,
        "image": format!("data:{mime_type};base64,{base64}")
    }))
}

pub async fn test_image_generation(storage: &FileStorage, id: &str) -> AppResult<Value> {
    let connection = get_required(storage, "connections", id)?;
    if connection.get("provider").and_then(Value::as_str) != Some("image_generation") {
        return Err(AppError::invalid_input(
            "Not an image-generation connection",
        ));
    }
    let prompt = "plate of spaghetti with marinara sauce";
    let start = now_millis();
    match generate_image_with_connection(&connection, prompt, 512, 512).await {
        Ok((base64, mime_type)) => Ok(json!({
            "success": true,
            "base64": base64,
            "mimeType": mime_type,
            "latencyMs": (now_millis() - start).max(0),
            "prompt": prompt
        })),
        Err(error) => Ok(json!({
            "success": false,
            "base64": Value::Null,
            "mimeType": Value::Null,
            "latencyMs": (now_millis() - start).max(0),
            "prompt": prompt,
            "error": error.message
        })),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn avatar_prompt_uses_appearance_when_present() {
        let body = json!({ "name": "Mari", "appearance": "long red hair, freckles" });
        let prompt = avatar_generation_prompt(&body);
        assert!(prompt.contains("Mari"));
        assert!(prompt.contains("long red hair"));
    }

    #[test]
    fn avatar_prompt_falls_back_when_name_empty() {
        let body = json!({});
        let prompt = avatar_generation_prompt(&body);
        assert!(prompt.contains("Character"));
        assert!(prompt.contains("distinctive character portrait"));
    }

    #[test]
    fn prompt_id_slugifies_non_ascii_and_trims_dashes() {
        // The Tauri-side helper preserved `trim_matches('-')` semantics; the
        // lift must keep the same id so prompt overrides keep matching.
        // Each non-ascii character maps to a single dash, then trailing
        // runs of dashes get trimmed off — so "Mari Käse 🍝" collapses to
        // "mari-k-se" (three trailing dashes from the umlaut+space+emoji
        // collapse + trim).
        assert_eq!(
            avatar_generation_prompt_id("Mari Käse 🍝"),
            "avatar-mari-k-se"
        );
        assert_eq!(
            avatar_generation_prompt_id("---trim me---"),
            "avatar-trim-me"
        );
    }

    #[test]
    fn image_dimension_rejects_out_of_range() {
        let body = json!({ "width": 100, "height": 4096 });
        // Below 128 and above 2048 both fall back.
        assert_eq!(image_dimension(&body, "width", 1024), 1024);
        assert_eq!(image_dimension(&body, "height", 1024), 1024);
        // In-range values pass through.
        let ok = json!({ "width": 512 });
        assert_eq!(image_dimension(&ok, "width", 999), 512);
    }

    #[test]
    fn image_generation_options_collects_references_and_reads_transparency() {
        let body = json!({
            "negativePrompt": "  blurry  ",
            "referenceImage": "data:image/png;base64,AAA",
            "referenceImages": ["data:image/png;base64,BBB", "   "],
            "nativeTransparentPng": true
        });
        let options = image_generation_options(&body);
        assert_eq!(options.negative_prompt.as_deref(), Some("blurry"));
        assert_eq!(options.reference_images.len(), 2);
        assert!(options.transparent_background);
    }

    #[test]
    fn prompt_override_matches_id_and_skips_empty() {
        let body = json!({
            "promptOverrides": [
                { "id": "avatar-mari", "prompt": "  custom mari portrait  " },
                { "id": "avatar-other", "prompt": "" }
            ]
        });
        assert_eq!(
            prompt_override(&body, "avatar-mari"),
            Some("custom mari portrait".to_string())
        );
        assert_eq!(prompt_override(&body, "avatar-other"), None);
        assert_eq!(prompt_override(&body, "avatar-missing"), None);
    }

    // Follow-up coverage Phase 4c /review flagged: the four `image_source`
    // / `image_connection_base_url` smoke tests that lived in `llm.rs`
    // during 4a-4b need a new home now that the canonical helpers moved
    // here. The helpers themselves are still exercised transitively through
    // `connection_auth_check`, but pinning the inference rules independently
    // catches drift early.

    #[test]
    fn image_source_infers_runpod_from_url() {
        let connection = json!({
            "provider": "image_generation",
            "model": "anything",
            "baseUrl": "https://api.runpod.ai/v2/whatever"
        });
        assert_eq!(image_source(&connection), "runpod_comfyui");
    }

    #[test]
    fn image_source_respects_explicit_service_field() {
        let connection = json!({
            "provider": "image_generation",
            "imageGenerationSource": "stability",
            "model": "irrelevant"
        });
        assert_eq!(image_source(&connection), "stability");
    }

    #[test]
    fn image_connection_base_url_falls_back_to_source_default() {
        let connection = json!({ "provider": "image_generation" });
        assert_eq!(
            image_connection_base_url(&connection, "novelai"),
            "https://image.novelai.net"
        );
    }

    #[test]
    fn image_connection_base_url_strips_trailing_slash_on_override() {
        let connection = json!({
            "provider": "image_generation",
            "baseUrl": "https://example.com/api/"
        });
        assert_eq!(
            image_connection_base_url(&connection, "stability"),
            "https://example.com/api"
        );
    }

    #[test]
    fn avatar_generation_preview_emits_expected_item() {
        let body = json!({ "name": "Mari", "appearance": "red hair", "width": 640, "height": 960 });
        let preview = avatar_generation_preview(body).expect("preview");
        let items = preview
            .get("items")
            .and_then(Value::as_array)
            .expect("items array");
        assert_eq!(items.len(), 1);
        let item = &items[0];
        assert_eq!(item.get("id").and_then(Value::as_str), Some("avatar-mari"));
        assert_eq!(item.get("kind").and_then(Value::as_str), Some("avatar"));
        assert_eq!(item.get("width").and_then(Value::as_u64), Some(640));
        assert_eq!(item.get("height").and_then(Value::as_u64), Some(960));
    }
}
