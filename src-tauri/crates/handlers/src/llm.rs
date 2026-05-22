// Transport-agnostic LLM handlers shared by the Tauri desktop binary and the
// Axum server. Covers the non-streaming surface (Phase 4a): completion calls,
// model listing, and connection auth checks. The streaming variant
// (`llm_stream_channel`) still lives on the Tauri side until Phase 4b lifts it
// onto an SSE-aware Axum route.
//
// Helpers here are intentionally narrow: each function takes a `&FileStorage`
// for connection lookups and reaches the network via `reqwest` or the
// `marinara_llm` crate. URL guards run through `marinara_security`.
//
// The two image-provider helpers (`image_source`, `image_connection_base_url`)
// are duplicated here in their pure-JSON form so `connection_auth_check`'s
// `"image_generation"` branch keeps working without dragging the rest of
// `commands/storage/images/providers.rs` along. Phase 4c will move the full
// image-generation surface onto the server and the canonical helpers will
// follow.

use crate::shared::get_required;
use marinara_core::{AppError, AppResult};
use marinara_security::is_allowed_outbound_url;
use marinara_storage::FileStorage;
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use std::time::Duration;
use tokio::sync::watch;

const DEFAULT_OPENAI_IMAGE_BASE_URL: &str = "https://api.openai.com/v1";
const DEFAULT_STABILITY_BASE_URL: &str = "https://api.stability.ai/v2beta";
const DEFAULT_TOGETHER_BASE_URL: &str = "https://api.together.xyz/v1";
const DEFAULT_NOVELAI_BASE_URL: &str = "https://image.novelai.net";
const DEFAULT_OPENROUTER_BASE_URL: &str = "https://openrouter.ai/api/v1";
const DEFAULT_XAI_BASE_URL: &str = "https://api.x.ai/v1";
const DEFAULT_POLLINATIONS_BASE_URL: &str = "https://image.pollinations.ai";
const DEFAULT_HORDE_BASE_URL: &str = "https://stablehorde.net/api/v2";
const DEFAULT_AUTOMATIC1111_BASE_URL: &str = "http://localhost:7860";
const DEFAULT_COMFYUI_BASE_URL: &str = "http://127.0.0.1:8188";
const DEFAULT_NANOGPT_BASE_URL: &str = "https://nano-gpt.com/api/v1";
const DEFAULT_BLOCKENTROPY_BASE_URL: &str = "https://api.blockentropy.ai";
const DEFAULT_RUNPOD_BASE_URL: &str = "https://api.runpod.ai/v2";

pub fn resolve_llm_connection_for_request(storage: &FileStorage, body: &Value) -> AppResult<Value> {
    if let Some(connection) = body.get("connection").filter(|value| value.is_object()) {
        return Ok(connection.clone());
    }
    if let Some(connection_id) = body
        .get("connectionId")
        .and_then(Value::as_str)
        .filter(|id| !id.is_empty())
    {
        return get_required(storage, "connections", connection_id);
    }
    if body.get("provider").is_some() && body.get("model").is_some() {
        return Ok(body.clone());
    }
    let connections = storage.list("connections")?;
    if let Some(default) = connections
        .iter()
        .find(|connection| {
            connection
                .get("isDefault")
                .and_then(Value::as_bool)
                .unwrap_or(false)
        })
        .cloned()
    {
        return Ok(default);
    }
    connections
        .into_iter()
        .next()
        .ok_or_else(|| AppError::invalid_input("No LLM connection is configured"))
}

pub fn llm_request_from_body(
    storage: &FileStorage,
    body: Value,
) -> AppResult<marinara_llm::LlmRequest> {
    let connection = resolve_llm_connection_for_request(storage, &body)?;
    let messages = body
        .get("messages")
        .and_then(Value::as_array)
        .ok_or_else(|| AppError::invalid_input("messages is required"))?
        .iter()
        .map(|message| {
            Ok(marinara_llm::LlmMessage {
                role: message
                    .get("role")
                    .and_then(Value::as_str)
                    .unwrap_or("user")
                    .to_string(),
                content: message
                    .get("content")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
                name: message
                    .get("name")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                images: message
                    .get("images")
                    .and_then(Value::as_array)
                    .map(|items| {
                        items
                            .iter()
                            .filter_map(Value::as_str)
                            .filter(|value| !value.trim().is_empty())
                            .map(str::to_string)
                            .collect()
                    })
                    .unwrap_or_default(),
                tool_call_id: message
                    .get("tool_call_id")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                tool_calls: message.get("tool_calls").cloned(),
            })
        })
        .collect::<AppResult<Vec<_>>>()?;
    Ok(marinara_llm::LlmRequest {
        connection: llm_connection_from_value(&connection)?,
        messages,
        parameters: body.get("parameters").cloned().unwrap_or_else(|| json!({})),
        tools: body
            .get("tools")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default(),
    })
}

pub async fn llm_complete(storage: &FileStorage, body: Value) -> AppResult<Value> {
    let content = marinara_llm::complete(llm_request_from_body(storage, body)?).await?;
    Ok(Value::String(content))
}

pub async fn llm_models(storage: &FileStorage, connection_id: Option<&str>) -> AppResult<Value> {
    let connection = connection_id
        .and_then(|id| storage.get("connections", id).ok().flatten())
        .or_else(|| {
            storage
                .list("connections")
                .ok()
                .and_then(|rows| rows.into_iter().next())
        });
    let provider = connection
        .as_ref()
        .and_then(|value| value.get("provider"))
        .and_then(Value::as_str)
        .unwrap_or("openai");
    let mut models = match connection.as_ref() {
        Some(connection) => fetch_provider_models(connection)
            .await
            .unwrap_or_else(|_| provider_model_catalog(provider)),
        None => provider_model_catalog(provider),
    };
    if let Some(connection) = connection.as_ref() {
        for key in ["model", "embeddingModel", "imageModel"] {
            if let Some(model) = connection
                .get(key)
                .and_then(Value::as_str)
                .filter(|value| !value.trim().is_empty())
            {
                push_model(&mut models, model, provider);
            }
        }
    }
    Ok(Value::Array(models))
}

pub fn llm_connection_from_value(value: &Value) -> AppResult<marinara_llm::LlmConnection> {
    let provider = value
        .get("provider")
        .and_then(Value::as_str)
        .ok_or_else(|| AppError::invalid_input("Connection provider is required"))?
        .to_string();
    let model = value
        .get("model")
        .and_then(Value::as_str)
        .filter(|model| !model.trim().is_empty())
        .ok_or_else(|| AppError::invalid_input("Connection model is required"))?
        .to_string();
    let api_key = value
        .get("apiKey")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let base_url = value
        .get("baseUrl")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let openrouter_provider = value
        .get("openrouterProvider")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    let enable_caching = match value.get("enableCaching") {
        Some(Value::Bool(value)) => *value,
        Some(Value::String(value)) => value.eq_ignore_ascii_case("true"),
        _ => false,
    };
    let caching_at_depth = value.get("cachingAtDepth").and_then(|value| {
        value
            .as_u64()
            .or_else(|| value.as_str()?.parse::<u64>().ok())
    });
    let max_tokens_override = value
        .get("maxTokensOverride")
        .and_then(|value| {
            value
                .as_u64()
                .or_else(|| value.as_str()?.parse::<u64>().ok())
        })
        .filter(|value| *value > 0);
    Ok(marinara_llm::LlmConnection {
        provider,
        model,
        api_key,
        base_url,
        openrouter_provider,
        enable_caching,
        caching_at_depth,
        max_tokens_override,
    })
}

pub async fn connection_models(storage: &FileStorage, id: &str) -> AppResult<Value> {
    let models = llm_models(storage, Some(id)).await?;
    Ok(json!({ "models": models }))
}

pub async fn connection_test(storage: &FileStorage, id: &str) -> AppResult<Value> {
    connection_auth_check(storage, id).await
}

pub async fn connection_test_message(storage: &FileStorage, id: &str) -> AppResult<Value> {
    let started = std::time::Instant::now();
    let connection = get_required(storage, "connections", id)?;
    let request = marinara_llm::LlmRequest {
        connection: llm_connection_from_value(&connection)?,
        messages: vec![marinara_llm::LlmMessage {
            role: "user".to_string(),
            content: "hi".to_string(),
            name: None,
            images: Vec::new(),
            tool_call_id: None,
            tool_calls: None,
        }],
        parameters: stored_generation_parameters(&connection),
        tools: Vec::new(),
    };
    let response = marinara_llm::complete(request).await?;
    Ok(json!({
        "success": true,
        "response": response,
        "latencyMs": started.elapsed().as_millis()
    }))
}

fn stored_generation_parameters(connection: &Value) -> Value {
    match connection.get("defaultParameters") {
        Some(Value::Object(map)) => Value::Object(map.clone()),
        Some(Value::String(raw)) => serde_json::from_str::<Value>(raw)
            .ok()
            .filter(Value::is_object)
            .unwrap_or_else(|| json!({})),
        _ => json!({}),
    }
}

pub async fn connection_auth_check(storage: &FileStorage, id: &str) -> AppResult<Value> {
    let started = std::time::Instant::now();
    let connection = get_required(storage, "connections", id)?;
    let model_name = connection
        .get("model")
        .and_then(Value::as_str)
        .map(str::to_string);
    match check_connection_without_generation(&connection).await {
        Ok(message) => Ok(json!({
            "success": true,
            "message": message,
            "latencyMs": started.elapsed().as_millis(),
            "modelName": model_name,
        })),
        Err(error) => {
            let mut response = json!({
                "success": false,
                "message": error.message,
                "latencyMs": started.elapsed().as_millis(),
                "modelName": Value::Null,
                "code": error.code,
            });
            if let Some(details) = error.details {
                response["details"] = details;
            }
            Ok(response)
        }
    }
}

async fn check_connection_without_generation(connection: &Value) -> AppResult<String> {
    let provider = connection
        .get("provider")
        .and_then(Value::as_str)
        .unwrap_or("openai");
    match provider {
        "openai_chatgpt" => marinara_llm::check_openai_chatgpt_auth().await,
        "claude_subscription" => marinara_llm::check_claude_subscription_available(),
        "openrouter" => check_openrouter_key(connection).await,
        "image_generation" => check_image_generation_connection(connection).await,
        _ => {
            let models = fetch_provider_models(connection).await?;
            if models.is_empty() {
                Ok("Connection successful.".to_string())
            } else {
                Ok(format!(
                    "Connection successful. {} model{} available.",
                    models.len(),
                    if models.len() == 1 { "" } else { "s" }
                ))
            }
        }
    }
}

async fn check_openrouter_key(connection: &Value) -> AppResult<String> {
    let api_key = connection_api_key(connection)?;
    let base = connection_base_url(connection);
    let url = format!("{}/key", base.trim_end_matches('/'));
    ensure_model_url_allowed(&url)?;
    let client = connection_test_client()?;
    let request = client
        .get(&url)
        .header("accept", "application/json")
        .bearer_auth(api_key)
        .header("HTTP-Referer", "https://marinara.local")
        .header("X-Title", "Marinara Engine");
    let json = send_connection_test_request(request, "OpenRouter").await?;
    let remaining = json
        .pointer("/data/limit_remaining")
        .and_then(Value::as_f64)
        .map(|value| format!(" Limit remaining: {value}."))
        .unwrap_or_default();
    Ok(format!("OpenRouter API key is valid.{remaining}"))
}

async fn check_image_generation_connection(connection: &Value) -> AppResult<String> {
    let source = image_source(connection);
    let base = image_connection_base_url(connection, &source);
    let source = if source.trim().is_empty() {
        "openai"
    } else {
        source.as_str()
    };
    match source {
        "runpod_comfyui" => Ok(
            "RunPod endpoint is configured. Use Test Image to verify generation because RunPod has no lightweight validation endpoint."
                .to_string(),
        ),
        "openrouter" | "gemini_image" => check_openrouter_key_for_base(connection, &base).await,
        "novelai" => {
            check_bearer_get("https://api.novelai.net/user/subscription", connection, "NovelAI")
                .await?;
            Ok("NovelAI API key is valid.".to_string())
        }
        "horde" => {
            let url = build_horde_url(&base, "status/heartbeat");
            ensure_model_url_allowed(&url)?;
            let api_key = connection_api_key_optional(connection);
            let request = connection_test_client()?
                .get(&url)
                .header("accept", "application/json")
                .header(
                    "apikey",
                    if api_key.trim().is_empty() {
                        "0000000000"
                    } else {
                        api_key.trim()
                    },
                )
                .header("Client-Agent", "Marinara-Engine");
            send_connection_test_request(request, "Stable Horde").await?;
            Ok("Stable Horde endpoint is reachable.".to_string())
        }
        "stability" => {
            let url = stability_url(&base, "v1/user/account");
            check_bearer_get(&url, connection, "Stability").await?;
            Ok("Stability API key is valid.".to_string())
        }
        "comfyui" => {
            let url = format!("{base}/system_stats");
            check_optional_bearer_get(&url, connection, "ComfyUI").await?;
            Ok("ComfyUI endpoint is reachable.".to_string())
        }
        "automatic1111" | "drawthings" => {
            let url = format!("{base}/sdapi/v1/options");
            check_optional_bearer_get(&url, connection, "Stable Diffusion Web UI").await?;
            Ok("Stable Diffusion Web UI endpoint is reachable.".to_string())
        }
        "pollinations" => {
            let url = format!("{base}/models");
            check_optional_bearer_get(&url, connection, "Pollinations").await?;
            Ok("Pollinations endpoint is reachable.".to_string())
        }
        _ => {
            let url = format!("{base}/models");
            check_bearer_get(&url, connection, "Image provider").await?;
            Ok("Image provider API key is valid.".to_string())
        }
    }
}

async fn check_openrouter_key_for_base(connection: &Value, base: &str) -> AppResult<String> {
    let api_key = connection_api_key(connection)?;
    let url = format!("{}/key", base.trim_end_matches('/'));
    ensure_model_url_allowed(&url)?;
    let request = connection_test_client()?
        .get(&url)
        .header("accept", "application/json")
        .bearer_auth(api_key)
        .header("HTTP-Referer", "https://marinara.local")
        .header("X-Title", "Marinara Engine");
    send_connection_test_request(request, "OpenRouter").await?;
    Ok("OpenRouter API key is valid.".to_string())
}

async fn check_bearer_get(url: &str, connection: &Value, label: &str) -> AppResult<Value> {
    let api_key = connection_api_key(connection)?;
    ensure_model_url_allowed(url)?;
    let request = connection_test_client()?
        .get(url)
        .header("accept", "application/json")
        .bearer_auth(api_key);
    send_connection_test_request(request, label).await
}

async fn check_optional_bearer_get(url: &str, connection: &Value, label: &str) -> AppResult<Value> {
    ensure_model_url_allowed(url)?;
    let mut request = connection_test_client()?
        .get(url)
        .header("accept", "application/json");
    let api_key = connection_api_key_optional(connection);
    if !api_key.trim().is_empty() {
        request = request.bearer_auth(api_key.trim().to_string());
    }
    send_connection_test_request(request, label).await
}

async fn send_connection_test_request(
    request: reqwest::RequestBuilder,
    label: &str,
) -> AppResult<Value> {
    let response = request
        .send()
        .await
        .map_err(|error| AppError::new("connection_network_error", error.to_string()))?;
    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|error| AppError::new("connection_response_error", error.to_string()))?;
    if !status.is_success() {
        return Err(AppError::new(
            "connection_provider_error",
            format!(
                "{label} returned HTTP {status}: {}",
                sanitize_provider_body(&text)
            ),
        ));
    }
    Ok(serde_json::from_str::<Value>(&text).unwrap_or(Value::Null))
}

fn connection_test_client() -> AppResult<reqwest::Client> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|error| AppError::new("connection_client_error", error.to_string()))
}

fn connection_api_key(connection: &Value) -> AppResult<String> {
    let api_key = connection_api_key_optional(connection);
    if api_key.trim().is_empty() {
        Err(AppError::invalid_input(
            "API key is required for this provider.",
        ))
    } else {
        Ok(api_key)
    }
}

fn connection_api_key_optional(connection: &Value) -> String {
    connection
        .get("apiKey")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string()
}

fn stability_url(base: &str, target_path: &str) -> String {
    let trimmed = base.trim_end_matches('/');
    if let Ok(mut parsed) = reqwest::Url::parse(trimmed) {
        let parts = parsed
            .path()
            .split('/')
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>();
        let version_index = parts
            .iter()
            .position(|part| *part == "v1" || *part == "v2beta");
        let prefix = version_index
            .map(|index| parts[..index].to_vec())
            .unwrap_or(parts);
        let path = prefix
            .into_iter()
            .chain(target_path.split('/').filter(|part| !part.is_empty()))
            .collect::<Vec<_>>()
            .join("/");
        parsed.set_path(&format!("/{path}"));
        parsed.set_query(None);
        parsed.set_fragment(None);
        return parsed.to_string().trim_end_matches('/').to_string();
    }
    format!("{}/{}", trimmed, target_path.trim_start_matches('/'))
}

fn build_horde_url(base: &str, target_path: &str) -> String {
    let trimmed = base.trim_end_matches('/');
    if let Ok(mut parsed) = reqwest::Url::parse(trimmed) {
        let parts = parsed
            .path()
            .split('/')
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>();
        let version_index = parts
            .windows(2)
            .position(|window| window[0] == "api" && window[1] == "v2");
        let mut prefix = version_index
            .map(|index| parts[..index + 2].to_vec())
            .unwrap_or(parts);
        if prefix.is_empty()
            || !prefix
                .windows(2)
                .any(|window| window[0] == "api" && window[1] == "v2")
        {
            prefix.extend(["api", "v2"]);
        }
        let path = prefix
            .into_iter()
            .chain(target_path.split('/').filter(|part| !part.is_empty()))
            .collect::<Vec<_>>()
            .join("/");
        parsed.set_path(&format!("/{path}"));
        parsed.set_query(None);
        parsed.set_fragment(None);
        return parsed.to_string().trim_end_matches('/').to_string();
    }
    format!("{}/api/v2/{}", trimmed, target_path.trim_start_matches('/'))
}

fn provider_model_catalog(provider: &str) -> Vec<Value> {
    let ids: &[&str] = match provider {
        "openai_chatgpt" => &[
            "gpt-5.2",
            "gpt-5.1",
            "gpt-5",
            "gpt-5.3-codex",
            "gpt-5.2-codex",
            "gpt-5.1-codex",
            "gpt-5-codex",
            "gpt-4o",
            "chatgpt-4o-latest",
        ],
        "anthropic" => &[
            "claude-3-5-sonnet-latest",
            "claude-3-5-haiku-latest",
            "claude-3-opus-latest",
        ],
        "claude_subscription" => &[
            "claude-opus-4-7",
            "claude-opus-4-6",
            "claude-sonnet-4-6",
            "claude-opus-4-5",
            "claude-sonnet-4-5",
            "claude-haiku-4-5",
        ],
        "google" | "google_vertex" => &["gemini-1.5-pro", "gemini-1.5-flash", "text-embedding-004"],
        "openrouter" => &[
            "openai/gpt-4o-mini",
            "anthropic/claude-3.5-sonnet",
            "google/gemini-flash-1.5",
        ],
        "ollama" => &["llama3.1", "mistral", "nomic-embed-text"],
        "xai" => &["grok-2-latest", "grok-2-mini-latest"],
        _ => &[
            "gpt-4o",
            "gpt-4o-mini",
            "text-embedding-3-small",
            "text-embedding-3-large",
        ],
    };
    ids.iter()
        .map(|id| json!({ "id": id, "name": id, "provider": provider }))
        .collect()
}

fn push_model(models: &mut Vec<Value>, id: &str, provider: &str) {
    if models
        .iter()
        .any(|model| model.get("id").and_then(Value::as_str) == Some(id))
    {
        return;
    }
    models.insert(0, json!({ "id": id, "name": id, "provider": provider }));
}

async fn fetch_provider_models(connection: &Value) -> AppResult<Vec<Value>> {
    let provider = connection
        .get("provider")
        .and_then(Value::as_str)
        .unwrap_or("openai");
    if provider == "image_generation" {
        return fetch_image_models(connection).await;
    }
    if provider == "openai_chatgpt" || provider == "claude_subscription" {
        return Ok(provider_model_catalog(provider));
    }
    if provider == "ollama" {
        return fetch_ollama_models(connection).await;
    }
    let base = connection_base_url(connection);
    if base.is_empty() {
        return Ok(provider_model_catalog(provider));
    }
    let url = model_endpoint(provider, &base, connection);
    ensure_model_url_allowed(&url)?;
    let mut request = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|error| AppError::new("models_client_error", error.to_string()))?
        .get(url)
        .header("accept", "application/json");
    let api_key = connection
        .get("apiKey")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();
    if provider == "anthropic" {
        request = request
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01");
    } else if !api_key.is_empty() && provider != "google" {
        request = request.bearer_auth(api_key);
    }
    let response = request
        .send()
        .await
        .map_err(|error| AppError::new("models_network_error", error.to_string()))?;
    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|error| AppError::new("models_response_error", error.to_string()))?;
    if !status.is_success() {
        return Err(AppError::new(
            "models_provider_error",
            format!(
                "Provider returned HTTP {status}: {}",
                sanitize_provider_body(&text)
            ),
        ));
    }
    let json = serde_json::from_str::<Value>(&text)
        .map_err(|error| AppError::new("models_json_error", error.to_string()))?;
    Ok(normalize_models_response(provider, &json))
}

async fn fetch_ollama_models(connection: &Value) -> AppResult<Vec<Value>> {
    let base = connection_base_url(connection);
    let url = format!("{base}/api/tags");
    ensure_model_url_allowed(&url)?;
    let json = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|error| AppError::new("models_client_error", error.to_string()))?
        .get(url)
        .send()
        .await
        .map_err(|error| AppError::new("models_network_error", error.to_string()))?
        .json::<Value>()
        .await
        .map_err(|error| AppError::new("models_json_error", error.to_string()))?;
    Ok(json
        .get("models")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|model| model.get("name").and_then(Value::as_str))
        .map(|id| json!({ "id": id, "name": id, "provider": "ollama" }))
        .collect())
}

async fn fetch_image_models(connection: &Value) -> AppResult<Vec<Value>> {
    let source = image_source(connection);
    let base = image_connection_base_url(connection, &source);
    if source == "stability" {
        return Ok(vec![
            json!({ "id": "stable-image-core", "name": "Stable Image Core", "provider": "image_generation" }),
            json!({ "id": "stable-image-ultra", "name": "Stable Image Ultra", "provider": "image_generation" }),
            json!({ "id": "sd3.5-large", "name": "Stable Diffusion 3.5 Large", "provider": "image_generation" }),
            json!({ "id": "sd3.5-medium", "name": "Stable Diffusion 3.5 Medium", "provider": "image_generation" }),
        ]);
    }
    if base.is_empty() {
        return Ok(provider_model_catalog("image_generation"));
    }
    match source.as_str() {
        "comfyui" => {
            fetch_json_models(
                &format!("{base}/object_info/CheckpointLoaderSimple"),
                connection,
                "image_generation",
                |json| {
                    json.get("CheckpointLoaderSimple")
                        .and_then(|value| value.get("input"))
                        .and_then(|value| value.get("required"))
                        .and_then(|value| value.get("ckpt_name"))
                        .and_then(Value::as_array)
                        .and_then(|items| items.first())
                        .and_then(Value::as_array)
                        .into_iter()
                        .flatten()
                        .filter_map(Value::as_str)
                        .map(|id| json!({ "id": id, "name": id, "provider": "image_generation" }))
                        .collect()
                },
            )
            .await
        }
        "automatic1111" | "drawthings" => {
            fetch_json_models(
                &format!("{base}/sdapi/v1/sd-models"),
                connection,
                "image_generation",
                |json| {
                    json.as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(|model| {
                            model
                                .get("title")
                                .or_else(|| model.get("model_name"))
                                .and_then(Value::as_str)
                        })
                        .map(|id| json!({ "id": id, "name": id, "provider": "image_generation" }))
                        .collect()
                },
            )
            .await
        }
        "horde" => {
            let url = format!(
                "{}/api/v2/status/models?type=image",
                base.trim_end_matches('/')
            );
            fetch_json_models(&url, connection, "image_generation", |json| {
                json.as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(|model| {
                        model
                            .get("name")
                            .or_else(|| model.get("id"))
                            .and_then(Value::as_str)
                    })
                    .map(|id| json!({ "id": id, "name": id, "provider": "image_generation" }))
                    .collect()
            })
            .await
        }
        "nanogpt" => {
            fetch_json_models(
                &format!("{base}/image-models"),
                connection,
                "image_generation",
                |json| normalize_openai_data_models(json, "image_generation"),
            )
            .await
        }
        "openrouter" => {
            fetch_json_models(
                &format!("{base}/models?output_modalities=image"),
                connection,
                "image_generation",
                |json| normalize_openai_data_models(json, "image_generation"),
            )
            .await
        }
        _ => Ok(provider_model_catalog("image_generation")),
    }
}

async fn fetch_json_models<F>(
    url: &str,
    connection: &Value,
    provider: &str,
    normalize: F,
) -> AppResult<Vec<Value>>
where
    F: Fn(&Value) -> Vec<Value>,
{
    ensure_model_url_allowed(url)?;
    let mut request = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|error| AppError::new("models_client_error", error.to_string()))?
        .get(url)
        .header("accept", "application/json");
    if let Some(api_key) = connection
        .get("apiKey")
        .and_then(Value::as_str)
        .filter(|key| !key.trim().is_empty())
    {
        request = request.bearer_auth(api_key.trim());
    }
    let response = request
        .send()
        .await
        .map_err(|error| AppError::new("models_network_error", error.to_string()))?;
    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|error| AppError::new("models_response_error", error.to_string()))?;
    if !status.is_success() {
        return Err(AppError::new(
            "models_provider_error",
            format!(
                "{provider} returned HTTP {status}: {}",
                sanitize_provider_body(&text)
            ),
        ));
    }
    let json = serde_json::from_str::<Value>(&text)
        .map_err(|error| AppError::new("models_json_error", error.to_string()))?;
    Ok(normalize(&json))
}

fn normalize_models_response(provider: &str, json: &Value) -> Vec<Value> {
    match provider {
        "google" => json
            .get("models")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter(|model| {
                model
                    .get("supportedGenerationMethods")
                    .and_then(Value::as_array)
                    .is_none_or(|methods| {
                        methods.iter().any(|method| method.as_str() == Some("generateContent"))
                    })
            })
            .filter_map(|model| {
                let id = model
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .trim_start_matches("models/");
                (!id.is_empty()).then(|| {
                    json!({ "id": id, "name": model.get("displayName").and_then(Value::as_str).unwrap_or(id), "provider": provider })
                })
            })
            .collect(),
        "google_vertex" => json
            .get("publisherModels")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|model| {
                let id = model
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .rsplit("/models/")
                    .next()
                    .unwrap_or("");
                (!id.is_empty()).then(|| {
                    json!({ "id": id, "name": model.get("displayName").and_then(Value::as_str).unwrap_or(id), "provider": provider })
                })
            })
            .collect(),
        "anthropic" => json
            .get("data")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|model| model_id(model).map(|id| (id, model)))
            .map(|(id, model)| {
                json!({ "id": id, "name": model.get("display_name").and_then(Value::as_str).unwrap_or(id), "provider": provider })
            })
            .collect(),
        "cohere" => {
            let data_models = normalize_openai_data_models(json, provider);
            if !data_models.is_empty() {
                return data_models;
            }
            json.get("models")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter(|model| {
                    model
                        .get("endpoints")
                        .and_then(Value::as_array)
                        .is_none_or(|items| items.iter().any(|item| item.as_str() == Some("chat")))
                })
                .filter_map(|model| model.get("name").and_then(Value::as_str))
                .map(|id| json!({ "id": id, "name": id, "provider": provider }))
                .collect()
        }
        _ => normalize_openai_data_models(json, provider),
    }
}

fn normalize_openai_data_models(json: &Value, provider: &str) -> Vec<Value> {
    json.get("data")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|model| model_id(model).map(|id| (id, model)))
        .map(|(id, model)| {
            json!({ "id": id, "name": model.get("name").and_then(Value::as_str).unwrap_or(id), "provider": provider })
        })
        .collect()
}

fn model_id(model: &Value) -> Option<&str> {
    model
        .get("id")
        .or_else(|| model.get("name"))
        .and_then(Value::as_str)
        .filter(|id| !id.trim().is_empty())
}

fn model_endpoint(provider: &str, base: &str, connection: &Value) -> String {
    let base = base.trim_end_matches('/');
    match provider {
        "anthropic" if base.ends_with("/v1") => format!("{base}/models"),
        "anthropic" => format!("{base}/v1/models"),
        "google" if base.ends_with("/v1beta") || base.ends_with("/v1") => {
            format!(
                "{base}/models?key={}",
                connection
                    .get("apiKey")
                    .and_then(Value::as_str)
                    .unwrap_or("")
            )
        }
        "google" => format!(
            "{base}/v1beta/models?key={}",
            connection
                .get("apiKey")
                .and_then(Value::as_str)
                .unwrap_or("")
        ),
        "google_vertex" => format!("{base}/models"),
        _ => format!("{base}/models"),
    }
}

fn connection_base_url(connection: &Value) -> String {
    let provider = connection
        .get("provider")
        .and_then(Value::as_str)
        .unwrap_or("openai");
    connection
        .get("baseUrl")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| provider_default_base_url(provider))
        .trim_end_matches('/')
        .to_string()
}

fn provider_default_base_url(provider: &str) -> &'static str {
    match provider {
        "anthropic" => "https://api.anthropic.com",
        "google" | "google_vertex" => "https://generativelanguage.googleapis.com",
        "openrouter" => "https://openrouter.ai/api/v1",
        "xai" => "https://api.x.ai/v1",
        "ollama" => "http://127.0.0.1:11434",
        "mistral" => "https://api.mistral.ai/v1",
        "cohere" => "https://api.cohere.ai/v2",
        "togetherai" => "https://api.together.xyz/v1",
        _ => "https://api.openai.com/v1",
    }
}

fn ensure_model_url_allowed(url: &str) -> AppResult<()> {
    if is_allowed_outbound_url(url, true) {
        Ok(())
    } else {
        Err(AppError::invalid_input(format!(
            "Outbound model URL is not allowed: {url}"
        )))
    }
}

fn sanitize_provider_body(body: &str) -> String {
    if body.contains("<html") || body.contains("<!DOCTYPE") {
        "Provider returned HTML instead of JSON".to_string()
    } else {
        body.chars().take(300).collect()
    }
}

// Pure-JSON twins of `commands/storage/images/providers.rs::image_source` and
// `connection_base_url`. Kept here so `connection_auth_check` can stay
// transport-agnostic; will be removed when Phase 4c moves the image-generation
// surface (which owns the canonical copies) onto the server.
fn image_source(connection: &Value) -> String {
    let explicit = connection
        .get("imageGenerationSource")
        .or_else(|| connection.get("imageService"))
        .and_then(Value::as_str)
        .or_else(|| connection.get("service").and_then(Value::as_str))
        .unwrap_or("")
        .trim();
    let model = connection
        .get("model")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();
    let base_url = connection
        .get("baseUrl")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();
    infer_image_source(if explicit.is_empty() { model } else { explicit }, base_url)
}

fn infer_image_source(model_or_source: &str, base_url: &str) -> String {
    let model = model_or_source.trim().to_ascii_lowercase();
    let url = base_url.trim().to_ascii_lowercase();
    match model.as_str() {
        "openai" | "stability" | "togetherai" | "novelai" | "pollinations" | "horde"
        | "blockentropy" | "openrouter" | "xai" | "comfyui" | "automatic1111"
        | "runpod_comfyui" | "gemini_image" | "nanogpt" => return model,
        "drawthings" => return "automatic1111".to_string(),
        _ => {}
    }
    if url.contains("nano-gpt.com") {
        return "nanogpt".to_string();
    }
    if url.contains("openrouter.ai") {
        return "openrouter".to_string();
    }
    if url.contains("api.x.ai") || url.contains("x.ai") {
        return "xai".to_string();
    }
    if (model.starts_with("grok-") && model.contains("image"))
        || (model.contains("grok") && model.contains("imagine"))
    {
        return "xai".to_string();
    }
    if model.starts_with("dall-e") || model.starts_with("gpt-image") || url.contains("openai.com") {
        return "openai".to_string();
    }
    if model.starts_with("sd3") || url.contains("stability.ai") {
        return "stability".to_string();
    }
    if model.contains("nai-diffusion") || url.contains("novelai.net") {
        return "novelai".to_string();
    }
    if model == "pollinations" || url.contains("pollinations.ai") {
        return "pollinations".to_string();
    }
    if model.contains("black-forest") || model.contains("flux") || url.contains("together.xyz") {
        return "togetherai".to_string();
    }
    if url.contains("stablehorde.net") {
        return "horde".to_string();
    }
    if url.contains("blockentropy") {
        return "blockentropy".to_string();
    }
    if url.contains(":8188") || url.contains("comfyui") {
        return "comfyui".to_string();
    }
    if url.contains("runpod.ai") {
        return "runpod_comfyui".to_string();
    }
    if url.contains(":7860") && !url.contains("drawthings") {
        return "automatic1111".to_string();
    }
    if (model.contains("gemini") && model.contains("image")) || model.contains("imagen") {
        return "gemini_image".to_string();
    }
    "openai".to_string()
}

fn image_connection_base_url(connection: &Value, source: &str) -> String {
    let fallback = match source {
        "stability" => DEFAULT_STABILITY_BASE_URL,
        "togetherai" => DEFAULT_TOGETHER_BASE_URL,
        "novelai" => DEFAULT_NOVELAI_BASE_URL,
        "openrouter" | "gemini_image" => DEFAULT_OPENROUTER_BASE_URL,
        "xai" => DEFAULT_XAI_BASE_URL,
        "pollinations" => DEFAULT_POLLINATIONS_BASE_URL,
        "horde" => DEFAULT_HORDE_BASE_URL,
        "automatic1111" | "drawthings" => DEFAULT_AUTOMATIC1111_BASE_URL,
        "comfyui" => DEFAULT_COMFYUI_BASE_URL,
        "runpod_comfyui" => DEFAULT_RUNPOD_BASE_URL,
        "nanogpt" => DEFAULT_NANOGPT_BASE_URL,
        "blockentropy" => DEFAULT_BLOCKENTROPY_BASE_URL,
        _ => DEFAULT_OPENAI_IMAGE_BASE_URL,
    };
    connection
        .get("baseUrl")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(fallback)
        .trim_end_matches('/')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn storage_with_connections(connections: Vec<Value>) -> (TempDir, FileStorage) {
        let dir = TempDir::new().expect("temp dir");
        let storage = FileStorage::new(dir.path().join("data")).expect("storage");
        for connection in connections {
            storage
                .create("connections", connection)
                .expect("create connection");
        }
        (dir, storage)
    }

    #[test]
    fn resolve_inline_connection_wins() {
        let (_dir, storage) = storage_with_connections(vec![]);
        let inline = json!({
            "provider": "openai",
            "model": "gpt-4o",
            "apiKey": "inline"
        });
        let body = json!({ "connection": inline.clone() });
        let resolved = resolve_llm_connection_for_request(&storage, &body).expect("resolve");
        assert_eq!(resolved, inline);
    }

    #[test]
    fn resolve_by_connection_id() {
        let (_dir, storage) = storage_with_connections(vec![json!({
            "name": "primary",
            "provider": "openai",
            "model": "gpt-4o",
            "apiKey": "stored"
        })]);
        let row = storage.list("connections").unwrap().pop().unwrap();
        let id = row.get("id").and_then(Value::as_str).unwrap().to_string();
        let body = json!({ "connectionId": id });
        let resolved = resolve_llm_connection_for_request(&storage, &body).expect("resolve");
        assert_eq!(
            resolved.get("name").and_then(Value::as_str),
            Some("primary")
        );
    }

    #[test]
    fn resolve_falls_back_to_default_flagged_connection() {
        let (_dir, storage) = storage_with_connections(vec![
            json!({
                "name": "first",
                "provider": "openai",
                "model": "gpt-4o",
                "apiKey": "first"
            }),
            json!({
                "name": "preferred",
                "provider": "openai",
                "model": "gpt-4o",
                "apiKey": "preferred",
                "isDefault": true
            }),
        ]);
        let body = json!({});
        let resolved = resolve_llm_connection_for_request(&storage, &body).expect("resolve");
        assert_eq!(
            resolved.get("name").and_then(Value::as_str),
            Some("preferred")
        );
    }

    #[test]
    fn resolve_falls_back_to_first_when_no_default() {
        let (_dir, storage) = storage_with_connections(vec![json!({
            "name": "only",
            "provider": "openai",
            "model": "gpt-4o",
            "apiKey": "only"
        })]);
        let body = json!({});
        let resolved = resolve_llm_connection_for_request(&storage, &body).expect("resolve");
        assert_eq!(resolved.get("name").and_then(Value::as_str), Some("only"));
    }

    #[test]
    fn resolve_inline_provider_and_model_body_pass_through() {
        let (_dir, storage) = storage_with_connections(vec![]);
        let body = json!({
            "provider": "openai",
            "model": "gpt-4o-mini"
        });
        let resolved = resolve_llm_connection_for_request(&storage, &body).expect("resolve");
        assert_eq!(
            resolved.get("model").and_then(Value::as_str),
            Some("gpt-4o-mini")
        );
    }

    #[test]
    fn resolve_errors_when_no_connections_configured() {
        let (_dir, storage) = storage_with_connections(vec![]);
        let body = json!({});
        let error = resolve_llm_connection_for_request(&storage, &body)
            .expect_err("must require a connection");
        assert_eq!(error.code, "invalid_input");
    }

    #[test]
    fn connection_from_value_requires_provider_and_model() {
        let no_provider = json!({ "model": "gpt-4o" });
        assert_eq!(
            llm_connection_from_value(&no_provider).unwrap_err().code,
            "invalid_input"
        );
        let no_model = json!({ "provider": "openai" });
        assert_eq!(
            llm_connection_from_value(&no_model).unwrap_err().code,
            "invalid_input"
        );
        let empty_model = json!({ "provider": "openai", "model": "   " });
        assert_eq!(
            llm_connection_from_value(&empty_model).unwrap_err().code,
            "invalid_input"
        );
    }

    #[test]
    fn connection_from_value_normalizes_optional_fields() {
        let value = json!({
            "provider": "openrouter",
            "model": "anthropic/claude-3.5-sonnet",
            "apiKey": "sk-abc",
            "baseUrl": "https://example.com",
            "openrouterProvider": "  routed  ",
            "enableCaching": "TRUE",
            "cachingAtDepth": "3",
            "maxTokensOverride": "1024"
        });
        let parsed = llm_connection_from_value(&value).expect("parse");
        assert_eq!(parsed.provider, "openrouter");
        assert_eq!(parsed.model, "anthropic/claude-3.5-sonnet");
        assert_eq!(parsed.api_key, "sk-abc");
        assert_eq!(parsed.base_url, "https://example.com");
        assert_eq!(parsed.openrouter_provider.as_deref(), Some("routed"));
        assert!(parsed.enable_caching);
        assert_eq!(parsed.caching_at_depth, Some(3));
        assert_eq!(parsed.max_tokens_override, Some(1024));
    }

    #[test]
    fn connection_from_value_drops_zero_max_tokens_override() {
        let value = json!({
            "provider": "openai",
            "model": "gpt-4o",
            "maxTokensOverride": 0
        });
        let parsed = llm_connection_from_value(&value).expect("parse");
        assert!(parsed.max_tokens_override.is_none());
    }

    #[test]
    fn request_from_body_requires_messages_array() {
        let (_dir, storage) = storage_with_connections(vec![]);
        let body = json!({
            "connection": { "provider": "openai", "model": "gpt-4o" }
        });
        let error = llm_request_from_body(&storage, body).expect_err("messages is required");
        assert_eq!(error.code, "invalid_input");
    }

    #[test]
    fn request_from_body_normalizes_messages() {
        let (_dir, storage) = storage_with_connections(vec![]);
        let body = json!({
            "connection": { "provider": "openai", "model": "gpt-4o" },
            "messages": [
                { "role": "system", "content": "be helpful" },
                { "content": "hi", "images": ["data:image/png;base64,AAA", "  "] }
            ]
        });
        let request = llm_request_from_body(&storage, body).expect("build request");
        assert_eq!(request.messages.len(), 2);
        assert_eq!(request.messages[0].role, "system");
        assert_eq!(request.messages[1].role, "user"); // default
        assert_eq!(
            request.messages[1].images,
            vec!["data:image/png;base64,AAA"]
        );
    }

    #[test]
    fn ensure_model_url_allowed_rejects_non_http_schemes() {
        // The security guard runs with allow_local=true (matching the Tauri
        // call site) so loopback URLs are fine for self-hosted endpoints like
        // Ollama and ComfyUI. The guard still blocks non-http schemes — that
        // is our regression canary that the lift still goes through
        // marinara-security.
        let bad = ensure_model_url_allowed("file:///etc/passwd");
        assert!(bad.is_err(), "non-http schemes must be rejected");
    }

    #[test]
    fn ensure_model_url_allowed_accepts_known_provider() {
        ensure_model_url_allowed("https://api.openai.com/v1/models").expect("openai allowed");
    }

    #[test]
    fn ensure_model_url_allowed_accepts_localhost_for_self_hosted() {
        ensure_model_url_allowed("http://127.0.0.1:11434/api/tags")
            .expect("ollama loopback allowed under allow_local");
    }

    #[test]
    fn provider_model_catalog_returns_nonempty_for_known_providers() {
        for provider in [
            "openai",
            "openai_chatgpt",
            "anthropic",
            "claude_subscription",
            "google",
            "openrouter",
            "ollama",
            "xai",
        ] {
            let catalog = provider_model_catalog(provider);
            assert!(!catalog.is_empty(), "{provider} catalog empty");
        }
    }

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
            DEFAULT_NOVELAI_BASE_URL
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
    fn registry_cancel_pre_register_is_pending_then_consumed() {
        let registry = LlmStreamRegistry::new();
        // Cancelling an unknown id stashes a pending mark and returns false
        // (no active stream was actually cancelled).
        let cancelled = registry.cancel("stream-1").expect("cancel");
        assert!(!cancelled);
        // Registering after the pending mark surfaces the cancellation up
        // front: the watch starts in the cancelled state.
        let rx = registry.register("stream-1").expect("register");
        assert!(*rx.borrow(), "pre-cancel must surface on register");
        // And the pending mark must be consumed so a fresh register is clean.
        registry.unregister("stream-1");
        let rx2 = registry.register("stream-1").expect("re-register");
        assert!(!*rx2.borrow(), "pending mark must be consumed once");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn registry_cancel_signals_active_watcher() {
        let registry = LlmStreamRegistry::new();
        let mut rx = registry.register("stream-2").expect("register");
        // cancel() while active returns true and triggers the watcher.
        let cancelled = registry.cancel("stream-2").expect("cancel");
        assert!(cancelled);
        // changed() resolves because the watch flipped to true.
        rx.changed().await.expect("watch should fire");
        assert!(*rx.borrow());
    }

    #[test]
    fn registry_unregister_clears_active_and_pending() {
        let registry = LlmStreamRegistry::new();
        registry.register("stream-3").expect("register");
        // Stash a separate pending cancel so we can confirm it goes away too.
        registry.cancel("stream-4").expect("cancel pending");
        registry.unregister("stream-3");
        registry.unregister("stream-4");
        // Re-registering either id must start in a non-cancelled state.
        let rx3 = registry.register("stream-3").expect("re-register 3");
        let rx4 = registry.register("stream-4").expect("re-register 4");
        assert!(!*rx3.borrow());
        assert!(!*rx4.borrow());
    }
}

// ---------------------------------------------------------------------------
// Phase 4b: streaming surface
//
// The cancellation registry that used to live on `src-tauri/src/state.rs`
// moves here so the Axum server can use the same code. Both binaries hold an
// `Arc<LlmStreamRegistry>` on their respective `AppState`. The Tauri command
// continues to ride on `tauri::ipc::Channel` for the emit closure; the Axum
// route adapts the same closure shape to an SSE stream by feeding events
// through an mpsc channel.
// ---------------------------------------------------------------------------

/// Per-process registry of in-flight LLM streams. Each entry is a tokio
/// `watch::Sender<bool>` whose value flips to `true` when the stream is
/// cancelled; the streaming task `tokio::select!`s on the matching receiver
/// so cancellation interrupts the underlying `marinara_llm::stream_events`
/// call mid-flight.
///
/// Cancel-before-register is a real race (the frontend can fire the cancel
/// invoke before the stream invoke even reaches Rust), so cancels for unknown
/// ids stash a "pending" mark and a subsequent register sees the watch start
/// in the cancelled state.
pub struct LlmStreamRegistry {
    inner: Mutex<LlmStreamState>,
}

#[derive(Default)]
struct LlmStreamState {
    active: HashMap<String, watch::Sender<bool>>,
    pending: HashSet<String>,
}

impl LlmStreamRegistry {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(LlmStreamState::default()),
        }
    }

    pub fn register(&self, stream_id: &str) -> AppResult<watch::Receiver<bool>> {
        let mut state = self.lock()?;
        let starts_cancelled = state.pending.remove(stream_id);
        let (tx, rx) = watch::channel(starts_cancelled);
        state.active.insert(stream_id.to_string(), tx);
        Ok(rx)
    }

    pub fn unregister(&self, stream_id: &str) {
        if let Ok(mut state) = self.inner.lock() {
            state.active.remove(stream_id);
            state.pending.remove(stream_id);
        }
    }

    pub fn cancel(&self, stream_id: &str) -> AppResult<bool> {
        let mut state = self.lock()?;
        if let Some(tx) = state.active.get(stream_id) {
            let _ = tx.send(true);
            Ok(true)
        } else {
            state.pending.insert(stream_id.to_string());
            Ok(false)
        }
    }

    fn lock(&self) -> AppResult<std::sync::MutexGuard<'_, LlmStreamState>> {
        self.inner.lock().map_err(|_| {
            AppError::new(
                "llm_stream_cancel_error",
                "LLM stream cancellation registry is unavailable",
            )
        })
    }
}

impl Default for LlmStreamRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Drive a streaming completion. The `emit` closure handles the transport
/// side: the Tauri shim wraps `tauri::ipc::Channel::send`, the Axum SSE route
/// wraps an `mpsc::UnboundedSender<Event>`. Either way the underlying
/// `marinara_llm::stream_events` walks the provider response and forwards
/// `{type, ...}` event payloads through the closure.
///
/// Cancellation surfaces in two ways: a pre-register cancel (caller hit the
/// cancel endpoint before this future ran) returns immediately with no events
/// emitted; an in-flight cancel interrupts the `stream_events` future via
/// `tokio::select!` on the watch receiver. Either path always unregisters
/// the stream before returning so the registry doesn't leak entries.
pub async fn llm_stream<F>(
    storage: &FileStorage,
    registry: &LlmStreamRegistry,
    stream_id: &str,
    body: Value,
    emit: F,
) -> AppResult<()>
where
    F: FnMut(Value) -> AppResult<()> + Send,
{
    let request = llm_request_from_body(storage, body)?;
    let mut cancellation = registry.register(stream_id)?;
    if *cancellation.borrow() {
        registry.unregister(stream_id);
        return Ok(());
    }
    let result = tokio::select! {
        result = marinara_llm::stream_events(request, emit) => result,
        _ = cancellation.changed() => Ok(()),
    };
    registry.unregister(stream_id);
    result
}

pub fn llm_stream_cancel(registry: &LlmStreamRegistry, stream_id: &str) -> AppResult<Value> {
    Ok(json!({ "cancelled": registry.cancel(stream_id)? }))
}
