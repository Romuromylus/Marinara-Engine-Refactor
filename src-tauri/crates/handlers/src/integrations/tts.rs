// Lifted into `marinara-handlers` from
// `src-tauri/src/commands/storage/integrations/tts.rs` during Phase 4d so
// the Axum server target can drive OpenAI / ElevenLabs / PocketTTS / NanoGPT
// text-to-speech. The public surface is a single dispatcher (`tts_call`) the
// Tauri integration_call shim and the server's invoke route both reach into;
// inside, four routes handle config GET/PUT, voices listing, and the
// audio-bytes-back `speak` endpoint.

use crate::shared::{percent_encode_component, required_string};
use base64::{engine::general_purpose, Engine as _};
use marinara_core::{AppError, AppResult};
use marinara_storage::FileStorage;
use serde_json::{json, Value};
use std::time::Duration;

const TTS_SETTINGS_KEY: &str = "tts";
const TTS_API_KEY_MASK: &str = "\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}";
const MAX_TTS_AUDIO_BYTES: usize = 20 * 1024 * 1024;

const OPENAI_FALLBACK_VOICES: &[&str] = &[
    "alloy", "ash", "coral", "echo", "fable", "nova", "onyx", "sage", "shimmer",
];

const POCKET_TTS_VOICES: &[&str] = &[
    "alba",
    "anna",
    "azelma",
    "bill_boerst",
    "caro_davy",
    "charles",
    "cosette",
    "eponine",
    "eve",
    "fantine",
    "george",
    "jane",
    "jean",
    "javert",
    "marius",
    "mary",
    "michael",
    "paul",
    "peter_yearsley",
    "stuart_bell",
    "vera",
];

const NANOGPT_ELEVENLABS_VOICES: &[&str] = &[
    "Adam",
    "Alice",
    "Antoni",
    "Aria",
    "Arnold",
    "Bella",
    "Bill",
    "Brian",
    "Callum",
    "Charlie",
    "Charlotte",
    "Chris",
    "Daniel",
    "Domi",
    "Dorothy",
    "Drew",
    "Elli",
    "Emily",
    "Eric",
    "Ethan",
    "Fin",
    "Freya",
    "George",
    "Gigi",
    "Giovanni",
    "Grace",
    "James",
    "Jeremy",
    "Jessica",
    "Joseph",
    "Josh",
    "Laura",
    "Liam",
    "Lily",
    "Matilda",
    "Matthew",
    "Michael",
    "Nicole",
    "Rachel",
    "River",
    "Roger",
    "Ryan",
    "Sam",
    "Sarah",
    "Thomas",
    "Will",
];

pub async fn tts_call(
    storage: &FileStorage,
    method: &str,
    rest: &[&str],
    body: Value,
) -> AppResult<Value> {
    match (method, rest) {
        ("GET", ["config"]) => get_config(storage),
        ("PUT", ["config"]) => put_config(storage, body),
        ("GET", ["voices"]) => voices(storage).await,
        ("POST", ["speak"]) => speak(storage, body).await,
        _ => Err(AppError::new(
            "route_not_found",
            format!("Unknown tts route: {method} /{}", rest.join("/")),
        )),
    }
}

fn default_config() -> Value {
    json!({
        "enabled": false,
        "source": "openai",
        "baseUrl": "https://api.openai.com/v1",
        "apiKey": "",
        "voice": "alloy",
        "model": "tts-1",
        "speed": 1.0,
        "elevenLabsStability": 0.5,
        "elevenLabsLanguageCode": "",
        "voiceMode": "single",
        "voiceAssignments": [],
        "npcDefaultVoicesEnabled": false,
        "npcDefaultMaleVoices": [],
        "npcDefaultFemaleVoices": [],
        "autoplayRP": false,
        "autoplayConvo": false,
        "autoplayGame": false,
        "dialogueOnly": false,
        "dialogueScope": "all",
        "dialogueCharacterName": ""
    })
}

fn config_with_defaults(mut config: Value) -> Value {
    let defaults = default_config();
    let Some(object) = config.as_object_mut() else {
        return defaults;
    };
    for (key, value) in defaults.as_object().expect("default config is an object") {
        object.entry(key.clone()).or_insert_with(|| value.clone());
    }
    config
}

fn load_config(storage: &FileStorage) -> AppResult<Value> {
    let value = storage
        .get("app-settings", TTS_SETTINGS_KEY)?
        .and_then(|entry| entry.get("value").cloned().or(Some(entry)))
        .unwrap_or_else(default_config);
    Ok(config_with_defaults(value))
}

fn get_config(storage: &FileStorage) -> AppResult<Value> {
    let mut config = load_config(storage)?;
    if config
        .get("apiKey")
        .and_then(Value::as_str)
        .is_some_and(|key| !key.is_empty())
    {
        config["apiKey"] = Value::String(TTS_API_KEY_MASK.to_string());
    }
    Ok(config)
}

fn put_config(storage: &FileStorage, body: Value) -> AppResult<Value> {
    let mut config = config_with_defaults(body);
    if config.get("apiKey").and_then(Value::as_str) == Some(TTS_API_KEY_MASK) {
        let existing = load_config(storage)?;
        config["apiKey"] = existing
            .get("apiKey")
            .cloned()
            .unwrap_or_else(|| Value::String(String::new()));
    }
    storage.upsert_with_id("app-settings", TTS_SETTINGS_KEY, json!({ "value": config }))?;
    Ok(Value::Null)
}

async fn voices(storage: &FileStorage) -> AppResult<Value> {
    let config = load_config(storage)?;
    let source = config
        .get("source")
        .and_then(Value::as_str)
        .unwrap_or("openai");
    if !config
        .get("enabled")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        return Ok(fallback_voices(source));
    }
    let base = configured_base_url(&config);
    if source == "elevenlabs" && is_nano_gpt_base_url(&base) {
        return Ok(voice_options_response(
            source,
            NANOGPT_ELEVENLABS_VOICES,
            Some("NanoGPT ElevenLabs"),
            true,
        ));
    }
    if source == "elevenlabs" {
        return elevenlabs_voices(&config, &base)
            .await
            .or_else(|_| Ok(fallback_voices(source)));
    }
    if source == "pockettts" {
        return Ok(fallback_voices(source));
    }
    let api_key = config.get("apiKey").and_then(Value::as_str).unwrap_or("");
    let response = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|error| AppError::new("tts_client_error", error.to_string()))?
        .get(format!("{base}/audio/voices"))
        .headers(openai_headers(api_key)?)
        .send()
        .await;
    match response {
        Ok(response) if response.status().is_success() => {
            let data = response
                .json::<Value>()
                .await
                .unwrap_or_else(|_| Value::Null);
            let parsed = parse_voice_options(&data);
            if parsed.is_empty() {
                Ok(fallback_voices(source))
            } else {
                Ok(
                    json!({ "voices": parsed.iter().filter_map(|voice| voice.get("id").cloned()).collect::<Vec<_>>(), "voiceOptions": parsed, "fromProvider": true, "source": source }),
                )
            }
        }
        _ => Ok(fallback_voices(source)),
    }
}

async fn speak(storage: &FileStorage, body: Value) -> AppResult<Value> {
    let text = required_string(&body, "text")?;
    if text.chars().count() > 4096 {
        return Err(AppError::invalid_input("TTS text is too long"));
    }
    let config = load_config(storage)?;
    if !config
        .get("enabled")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        return Err(AppError::invalid_input("TTS is not enabled"));
    }
    let source = config
        .get("source")
        .and_then(Value::as_str)
        .unwrap_or("openai");
    let base = configured_base_url(&config);
    let api_key = config.get("apiKey").and_then(Value::as_str).unwrap_or("");
    let configured_voice = config
        .get("voice")
        .and_then(Value::as_str)
        .unwrap_or("alloy");
    let voice = body
        .get("voice")
        .and_then(Value::as_str)
        .filter(|voice| !voice.trim().is_empty())
        .unwrap_or(configured_voice);
    if source == "elevenlabs" && api_key.is_empty() {
        return Err(AppError::invalid_input(
            "ElevenLabs API key is not configured",
        ));
    }
    if source == "elevenlabs" && voice.trim().is_empty() {
        return Err(AppError::invalid_input("ElevenLabs voice is not selected"));
    }

    let model = normalized_model(
        source,
        &base,
        config.get("model").and_then(Value::as_str).unwrap_or(""),
    );
    let speed = config
        .get("speed")
        .and_then(Value::as_f64)
        .unwrap_or(1.0)
        .clamp(0.25, 4.0);
    let tone = body.get("tone").and_then(Value::as_str);
    let speaker = body.get("speaker").and_then(Value::as_str);
    let use_nano_gpt = is_nano_gpt_base_url(&base);
    let url = if use_nano_gpt {
        format!("{}/audio/speech", nano_gpt_v1_base_url(&base))
    } else if source == "pockettts" {
        format!("{base}/tts")
    } else if source == "elevenlabs" {
        format!(
            "{}/v1/text-to-speech/{}?output_format=mp3_44100_128",
            elevenlabs_api_root(&base),
            percent_encode_component(voice)
        )
    } else {
        format!("{base}/audio/speech")
    };

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .map_err(|error| AppError::new("tts_client_error", error.to_string()))?;
    let provider_text = if source == "elevenlabs" {
        build_elevenlabs_text_input(text, tone)
    } else {
        text.to_string()
    };
    let request = if source == "pockettts" {
        let form = reqwest::multipart::Form::new()
            .text("text", provider_text)
            .text("voice_url", voice.to_string());
        client
            .post(url)
            .headers(optional_bearer_headers(api_key)?)
            .multipart(form)
    } else if source == "elevenlabs" && !use_nano_gpt {
        let language = config
            .get("elevenLabsLanguageCode")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim();
        let mut payload = json!({
            "text": provider_text,
            "model_id": model,
            "voice_settings": {
                "stability": config.get("elevenLabsStability").and_then(Value::as_f64).unwrap_or(0.5),
                "speed": speed
            }
        });
        if !language.is_empty() {
            payload["language_code"] = json!(language);
        }
        client
            .post(url)
            .headers(elevenlabs_headers(api_key)?)
            .json(&payload)
    } else {
        let instructions = build_speech_instructions(speaker, tone);
        let mut payload = json!({
            "model": model,
            "input": provider_text,
            "voice": if voice.trim().is_empty() { "alloy" } else { voice },
            "speed": speed,
            "response_format": "mp3"
        });
        if let Some(instructions) = instructions {
            payload["instructions"] = json!(instructions);
        }
        let headers = if use_nano_gpt {
            nano_gpt_headers(api_key)?
        } else {
            openai_headers(api_key)?
        };
        client.post(url).headers(headers).json(&payload)
    };

    let response = request
        .send()
        .await
        .map_err(|error| AppError::new("tts_provider_unreachable", error.to_string()))?;
    let status = response.status();
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("audio/mpeg")
        .to_string();
    let bytes = response
        .bytes()
        .await
        .map_err(|error| AppError::new("tts_response_error", error.to_string()))?;
    if !status.is_success() {
        let detail = String::from_utf8_lossy(&bytes)
            .chars()
            .take(500)
            .collect::<String>();
        return Err(AppError::with_details(
            "tts_provider_error",
            format!("TTS provider returned HTTP {status}"),
            json!({ "detail": detail }),
        ));
    }
    if !is_allowed_audio_content_type(&content_type) {
        let detail = String::from_utf8_lossy(&bytes)
            .chars()
            .take(500)
            .collect::<String>();
        return Err(AppError::with_details(
            "tts_provider_error",
            "TTS provider returned a non-audio response",
            json!({ "contentType": content_type, "detail": detail }),
        ));
    }
    if bytes.len() > MAX_TTS_AUDIO_BYTES {
        return Err(AppError::invalid_input("TTS audio response is too large"));
    }
    Ok(json!({
        "audioBase64": general_purpose::STANDARD.encode(bytes),
        "contentType": if content_type.starts_with("audio/") { content_type } else { "audio/mpeg".to_string() }
    }))
}

fn configured_base_url(config: &Value) -> String {
    let source = config
        .get("source")
        .and_then(Value::as_str)
        .unwrap_or("openai");
    let configured = config
        .get("baseUrl")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .trim_end_matches('/');
    if !configured.is_empty() {
        return configured.to_string();
    }
    match source {
        "elevenlabs" => "https://api.elevenlabs.io".to_string(),
        "pockettts" => "http://localhost:8000".to_string(),
        _ => "https://api.openai.com/v1".to_string(),
    }
}

fn normalized_model(source: &str, base: &str, configured: &str) -> String {
    let fallback = match source {
        "elevenlabs" => "eleven_multilingual_v2",
        "pockettts" => "pocket-tts",
        _ => "tts-1",
    };
    let model = configured.trim();
    let model = if model.is_empty() { fallback } else { model };
    if is_nano_gpt_base_url(base) {
        match model.to_ascii_lowercase().as_str() {
            "eleven_v3" | "elevenlabs-v3" | "elevenlabs_v3" | "elevenlabs_tts_v3" => {
                "Elevenlabs-V3".to_string()
            }
            "eleven_turbo_v2_5" | "eleven_flash_v2_5" => "Elevenlabs-Turbo-V2.5".to_string(),
            _ => model.to_string(),
        }
    } else if source == "elevenlabs" {
        match model.to_ascii_lowercase().as_str() {
            "tts_v3" | "elevenlabs_v3" | "elevenlabs_tts_v3" => "eleven_v3".to_string(),
            _ => model.to_string(),
        }
    } else {
        model.to_string()
    }
}

fn fallback_voices(source: &str) -> Value {
    match source {
        "elevenlabs" => voice_options_response(source, &[], None, false),
        "pockettts" => {
            voice_options_response(source, POCKET_TTS_VOICES, Some("PocketTTS built-in"), false)
        }
        _ => voice_options_response(source, OPENAI_FALLBACK_VOICES, None, false),
    }
}

fn voice_options_response(
    source: &str,
    voices: &[&str],
    category: Option<&str>,
    from_provider: bool,
) -> Value {
    let options = voices
        .iter()
        .map(|voice| {
            let mut item = json!({ "id": voice, "name": voice });
            if let Some(category) = category {
                item["category"] = json!(category);
            }
            item
        })
        .collect::<Vec<_>>();
    Ok::<Value, AppError>(json!({
        "voices": voices,
        "voiceOptions": options,
        "fromProvider": from_provider,
        "source": source
    }))
    .expect("voice response is valid")
}

async fn elevenlabs_voices(config: &Value, base: &str) -> AppResult<Value> {
    let api_key = config.get("apiKey").and_then(Value::as_str).unwrap_or("");
    if api_key.is_empty() {
        return Ok(fallback_voices("elevenlabs"));
    }
    let url = format!(
        "{}/v2/voices?page_size=100&include_total_count=false",
        elevenlabs_api_root(base)
    );
    let response = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|error| AppError::new("tts_client_error", error.to_string()))?
        .get(url)
        .headers(elevenlabs_headers(api_key)?)
        .send()
        .await
        .map_err(|error| AppError::new("tts_provider_unreachable", error.to_string()))?;
    if !response.status().is_success() {
        return Ok(fallback_voices("elevenlabs"));
    }
    let data = response
        .json::<Value>()
        .await
        .unwrap_or_else(|_| Value::Null);
    let parsed = parse_voice_options(&data);
    if parsed.is_empty() {
        Ok(fallback_voices("elevenlabs"))
    } else {
        Ok(
            json!({ "voices": parsed.iter().filter_map(|voice| voice.get("id").cloned()).collect::<Vec<_>>(), "voiceOptions": parsed, "fromProvider": true, "source": "elevenlabs" }),
        )
    }
}

fn parse_voice_options(data: &Value) -> Vec<Value> {
    let list = data
        .as_array()
        .cloned()
        .or_else(|| data.get("voices").and_then(Value::as_array).cloned())
        .or_else(|| data.get("data").and_then(Value::as_array).cloned())
        .unwrap_or_default();
    list.into_iter()
        .filter_map(|item| {
            if let Some(raw) = item.as_str().filter(|value| !value.trim().is_empty()) {
                return Some(json!({ "id": raw, "name": raw }));
            }
            let id = item
                .get("voice_id")
                .or_else(|| item.get("voiceId"))
                .or_else(|| item.get("id"))
                .or_else(|| item.get("name"))
                .and_then(Value::as_str)?
                .to_string();
            let name = item.get("name").and_then(Value::as_str).unwrap_or(&id).to_string();
            Some(json!({
                "id": id,
                "name": name,
                "description": item.get("description").cloned().unwrap_or(Value::Null),
                "previewUrl": item.get("preview_url").or_else(|| item.get("previewUrl")).cloned().unwrap_or(Value::Null),
                "category": item.get("category").cloned().unwrap_or(Value::Null),
                "labels": item.get("labels").cloned().unwrap_or(Value::Null)
            }))
        })
        .collect()
}

fn elevenlabs_api_root(base: &str) -> String {
    strip_version_suffix(base)
}

fn nano_gpt_v1_base_url(base: &str) -> String {
    let root = strip_version_suffix(base);
    if root.ends_with("/v1") {
        root
    } else {
        format!("{root}/v1")
    }
}

fn strip_version_suffix(base: &str) -> String {
    let trimmed = base.trim_end_matches('/');
    let lower = trimmed.to_ascii_lowercase();
    if lower.ends_with("/v1") || lower.ends_with("/v2") || lower.ends_with("/v3") {
        trimmed
            .rsplit_once('/')
            .map(|(root, _)| root.to_string())
            .unwrap_or_else(|| trimmed.to_string())
    } else {
        trimmed.to_string()
    }
}

fn is_nano_gpt_base_url(base: &str) -> bool {
    base.to_ascii_lowercase().contains("nano-gpt.com")
}

fn build_speech_instructions(speaker: Option<&str>, tone: Option<&str>) -> Option<String> {
    let mut parts = Vec::new();
    if let Some(speaker) = speaker.map(str::trim).filter(|value| !value.is_empty()) {
        parts.push(format!("Voice the line as {speaker}."));
    }
    if let Some(tone) = tone.map(str::trim).filter(|value| !value.is_empty()) {
        let article = if tone
            .chars()
            .next()
            .is_some_and(|ch| "aeiouAEIOU".contains(ch))
        {
            "an"
        } else {
            "a"
        };
        parts.push(format!("Use {article} {tone} tone."));
    }
    if parts.is_empty() {
        None
    } else {
        parts.push(
            "Do not read speaker names, brackets, markup, or stage directions aloud.".to_string(),
        );
        Some(parts.join(" "))
    }
}

fn build_elevenlabs_text_input(text: &str, tone: Option<&str>) -> String {
    let tags = tone
        .unwrap_or("")
        .split(',')
        .filter_map(|part| {
            let tag = part
                .trim()
                .trim_matches(|ch| matches!(ch, '[' | ']' | '"' | '\''))
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
            if tag.is_empty() || tag.contains('<') || tag.contains('>') {
                return None;
            }
            let normalized = tag.to_ascii_lowercase();
            if matches!(
                normalized.as_str(),
                "main" | "side" | "extra" | "action" | "thought"
            ) || normalized.starts_with("whisper")
            {
                return None;
            }
            Some(format!("[{tag}]"))
        })
        .collect::<Vec<_>>();
    if tags.is_empty() {
        text.to_string()
    } else {
        format!("{} {text}", tags.join(" "))
    }
}

fn is_allowed_audio_content_type(content_type: &str) -> bool {
    let normalized = content_type.to_ascii_lowercase();
    normalized.contains("audio/") || normalized.contains("application/octet-stream")
}

fn openai_headers(api_key: &str) -> AppResult<reqwest::header::HeaderMap> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::CONTENT_TYPE,
        reqwest::header::HeaderValue::from_static("application/json"),
    );
    if !api_key.trim().is_empty() {
        headers.insert(
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderValue::from_str(&format!("Bearer {}", api_key.trim()))
                .map_err(|error| AppError::invalid_input(error.to_string()))?,
        );
    }
    Ok(headers)
}

fn elevenlabs_headers(api_key: &str) -> AppResult<reqwest::header::HeaderMap> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::CONTENT_TYPE,
        reqwest::header::HeaderValue::from_static("application/json"),
    );
    if !api_key.trim().is_empty() {
        headers.insert(
            "xi-api-key",
            reqwest::header::HeaderValue::from_str(api_key.trim())
                .map_err(|error| AppError::invalid_input(error.to_string()))?,
        );
    }
    Ok(headers)
}

fn nano_gpt_headers(api_key: &str) -> AppResult<reqwest::header::HeaderMap> {
    let mut headers = openai_headers(api_key)?;
    if !api_key.trim().is_empty() {
        headers.insert(
            "x-api-key",
            reqwest::header::HeaderValue::from_str(api_key.trim())
                .map_err(|error| AppError::invalid_input(error.to_string()))?,
        );
    }
    Ok(headers)
}

fn optional_bearer_headers(api_key: &str) -> AppResult<reqwest::header::HeaderMap> {
    let mut headers = reqwest::header::HeaderMap::new();
    if !api_key.trim().is_empty() {
        headers.insert(
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderValue::from_str(&format!("Bearer {}", api_key.trim()))
                .map_err(|error| AppError::invalid_input(error.to_string()))?,
        );
    }
    Ok(headers)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configured_base_url_falls_back_per_source() {
        let cfg = json!({ "source": "openai" });
        assert_eq!(configured_base_url(&cfg), "https://api.openai.com/v1");
        let cfg = json!({ "source": "elevenlabs" });
        assert_eq!(configured_base_url(&cfg), "https://api.elevenlabs.io");
        let cfg = json!({ "source": "pockettts" });
        assert_eq!(configured_base_url(&cfg), "http://localhost:8000");
    }

    #[test]
    fn configured_base_url_strips_trailing_slash() {
        let cfg = json!({ "source": "openai", "baseUrl": "https://override.example.com/api/" });
        assert_eq!(
            configured_base_url(&cfg),
            "https://override.example.com/api"
        );
    }

    #[test]
    fn normalized_model_remaps_nano_gpt_aliases() {
        let nano = "https://nano-gpt.com/api/v1";
        assert_eq!(
            normalized_model("elevenlabs", nano, "eleven_v3"),
            "Elevenlabs-V3"
        );
        assert_eq!(
            normalized_model("elevenlabs", nano, "eleven_turbo_v2_5"),
            "Elevenlabs-Turbo-V2.5"
        );
        // Native ElevenLabs (non-NanoGPT) preserves the modern alias.
        assert_eq!(
            normalized_model("elevenlabs", "https://api.elevenlabs.io", "tts_v3"),
            "eleven_v3"
        );
        // Empty model falls back to source default.
        assert_eq!(normalized_model("openai", "", ""), "tts-1");
    }

    #[test]
    fn strip_version_suffix_drops_trailing_v1() {
        assert_eq!(strip_version_suffix("https://api.x/v1"), "https://api.x");
        assert_eq!(strip_version_suffix("https://api.x/v2/"), "https://api.x");
        // No version segment to strip.
        assert_eq!(strip_version_suffix("https://api.x/v9"), "https://api.x/v9");
        assert_eq!(strip_version_suffix("https://api.x/"), "https://api.x");
    }

    #[test]
    fn fallback_voices_shape_per_source() {
        let openai = fallback_voices("openai");
        let voices = openai
            .get("voices")
            .and_then(Value::as_array)
            .expect("voices");
        assert!(!voices.is_empty());
        assert_eq!(openai.get("source").and_then(Value::as_str), Some("openai"));
        assert_eq!(
            openai.get("fromProvider").and_then(Value::as_bool),
            Some(false)
        );

        let pocket = fallback_voices("pockettts");
        assert_eq!(
            pocket.get("source").and_then(Value::as_str),
            Some("pockettts")
        );

        // ElevenLabs fallback returns an empty list (provider lookup hasn't
        // happened yet) but the envelope shape stays the same.
        let eleven = fallback_voices("elevenlabs");
        assert!(eleven
            .get("voices")
            .and_then(Value::as_array)
            .expect("voices")
            .is_empty());
    }

    #[test]
    fn parse_voice_options_handles_string_and_object_entries() {
        let data = json!({
            "voices": [
                "alloy",
                { "voice_id": "abc123", "name": "Custom", "category": "premium" },
                "  ",
                { "id": "fallback-id" }
            ]
        });
        let parsed = parse_voice_options(&data);
        assert_eq!(parsed.len(), 3);
        assert_eq!(parsed[0].get("id").and_then(Value::as_str), Some("alloy"));
        assert_eq!(parsed[1].get("id").and_then(Value::as_str), Some("abc123"));
        assert_eq!(
            parsed[1].get("name").and_then(Value::as_str),
            Some("Custom")
        );
        assert_eq!(
            parsed[1].get("category").and_then(Value::as_str),
            Some("premium")
        );
        assert_eq!(
            parsed[2].get("id").and_then(Value::as_str),
            Some("fallback-id")
        );
    }

    #[test]
    fn build_speech_instructions_combines_speaker_and_tone() {
        assert_eq!(build_speech_instructions(None, None), None);
        let cheerful = build_speech_instructions(None, Some("excited"));
        assert!(cheerful.as_deref().unwrap().contains("an excited tone"));
        let combined = build_speech_instructions(Some("Mari"), Some("sad"));
        let text = combined.expect("combined");
        assert!(text.contains("Voice the line as Mari"));
        assert!(text.contains("a sad tone"));
        assert!(text.contains("Do not read speaker names"));
    }

    #[test]
    fn build_elevenlabs_text_input_strips_internal_tags() {
        // "main" and "whisper*" never reach the provider — they're internal
        // markers that would confuse the audio model.
        let out = build_elevenlabs_text_input("hello", Some("main, whisper-quiet, sultry"));
        assert!(out.starts_with("[sultry]"), "got: {out}");
        assert!(out.ends_with("hello"));

        // No tone → text passes through unchanged.
        assert_eq!(build_elevenlabs_text_input("hi", None), "hi");
    }

    #[test]
    fn is_allowed_audio_content_type_accepts_audio_and_octet_stream() {
        assert!(is_allowed_audio_content_type("audio/mpeg"));
        assert!(is_allowed_audio_content_type("AUDIO/wav"));
        assert!(is_allowed_audio_content_type("application/octet-stream"));
        assert!(!is_allowed_audio_content_type("text/html"));
        assert!(!is_allowed_audio_content_type("application/json"));
    }
}
