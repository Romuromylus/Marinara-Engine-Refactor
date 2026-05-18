use super::*;
use super::shared::*;
use super::http::{http_binary, http_json};

pub(crate) async fn bot_browser_call(method: &str, rest: &[&str], route: &ParsedPath, _body: Value) -> AppResult<Value> {
    match (method, rest) {
        ("GET", ["chub", "search"]) => {
            let q = route.query.get("q").cloned().unwrap_or_default();
            let page = route.query.get("page").map(String::as_str).unwrap_or("1");
            let sort = route.query.get("sort").map(String::as_str).unwrap_or("download_count");
            let nsfw = route.query.get("nsfw").map(String::as_str).unwrap_or("true");
            let mut params = vec![
                ("search".to_string(), q),
                ("first".to_string(), "48".to_string()),
                ("page".to_string(), page.to_string()),
                ("nsfw".to_string(), nsfw.to_string()),
                ("nsfl".to_string(), nsfw.to_string()),
                ("include_forks".to_string(), "true".to_string()),
                ("venus".to_string(), "false".to_string()),
                ("min_tokens".to_string(), route.query.get("min_tokens").cloned().unwrap_or_else(|| "50".to_string())),
            ];
            if sort != "default" {
                params.push(("sort".to_string(), sort.to_string()));
            }
            for key in ["asc", "max_days_ago", "special_mode", "username", "max_tokens", "tags", "excludeTags"] {
                if let Some(value) = route.query.get(key) {
                    let upstream = match key {
                        "tags" => "topics",
                        "excludeTags" => "excludetopics",
                        _ => key,
                    };
                    params.push((upstream.to_string(), value.clone()));
                }
            }
            for key in ["require_images", "require_lore", "require_expressions", "require_alternate_greetings"] {
                if route.query.get(key).map(String::as_str) == Some("true") {
                    params.push((key.to_string(), "true".to_string()));
                }
            }
            let query = params
                .iter()
                .map(|(key, value)| format!("{key}={value}"))
                .collect::<Vec<_>>()
                .join("&");
            http_json(&format!("https://api.chub.ai/search?{query}")).await
        }
        ("GET", ["chub", "character", path @ ..]) if !path.is_empty() => {
            let full_path = path.join("/");
            http_json(&format!(
                "https://api.chub.ai/api/characters/{}?full=true&nocache={}",
                full_path,
                now_millis()
            ))
            .await
        }
        ("GET", ["chub", "avatar", path @ ..]) if !path.is_empty() => {
            let full_path = path.join("/");
            match http_binary(
                &format!("https://avatars.charhub.io/avatars/{full_path}/avatar.webp"),
                "image/webp",
            )
            .await
            {
                Ok(value) => Ok(value),
                Err(_) => http_binary(
                    &format!("https://avatars.charhub.io/avatars/{full_path}/chara_card_v2.png"),
                    "image/png",
                )
                .await,
            }
        }
        ("GET", ["chub", "download", path @ ..]) if !path.is_empty() => {
            let full_path = path.join("/");
            http_binary(
                &format!("https://avatars.charhub.io/avatars/{full_path}/chara_card_v2.png"),
                "image/png",
            )
            .await
        }
        ("GET", ["pygmalion", "session"]) | ("GET", ["chartavern", "session"]) => Ok(json!({ "active": false })),
        ("POST", ["pygmalion", "set-token"]) | ("POST", ["chartavern", "set-cookie"]) => Ok(json!({ "ok": true, "stored": true })),
        ("GET", ["pygmalion", "validate"]) | ("GET", ["chartavern", "validate"]) => Ok(json!({ "valid": false, "reason": "No provider session is active in local Tauri storage." })),
        ("POST", ["pygmalion", "logout"]) | ("POST", ["chartavern", "logout"]) => Ok(json!({ "ok": true })),
        _ => Err(AppError::new(
            "route_not_found",
            format!("bot-browser route {method} /{} is not implemented", rest.join("/")),
        )),
    }
}
