use axum::{
    extract::{Path, Query, Request, State},
    http::{HeaderValue, Method, StatusCode},
    middleware::{self, Next},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Json, Redirect, Response,
    },
    routing::{get, post},
    Router,
};
use futures_util::stream::Stream;
use marinara_assets::AssetService;
use marinara_core::AppError;
use marinara_handlers::auth::{CSRF_HEADER_NAME, SESSION_COOKIE_NAME};
use marinara_handlers::llm::LlmStreamRegistry;
use marinara_storage::FileStorage;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::convert::Infallible;
use std::path::PathBuf;
use std::sync::Arc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tower_http::services::{ServeDir, ServeFile};
use tracing::{info, warn};

#[derive(Clone)]
struct AppState {
    storage: FileStorage,
    backgrounds: AssetService,
    game_assets: AssetService,
    lorebook_images: AssetService,
    fonts: AssetService,
    data_dir: PathBuf,
    /// Per-process registry of in-flight LLM streams. The Tauri binary holds
    /// the exact same `Arc<LlmStreamRegistry>` on its AppState — Phase 4b
    /// lifted the type into `marinara-handlers` so both transports share it.
    llm_streams: Arc<LlmStreamRegistry>,
    /// Public URL Spotify redirects the browser to after authorization.
    /// Read from `MARINARA_SPOTIFY_REDIRECT_URI`; users must register this
    /// exact value in their Spotify Developer dashboard. Empty means
    /// "fall back to the loopback default" which only the Tauri binary can
    /// service — the server's `/api/spotify/callback` route will reject
    /// callbacks with an explanatory error if not configured.
    spotify_redirect_uri: Option<String>,
    /// When true, the `auth_middleware` enforces cookie + CSRF on every
    /// /api/* route except the explicit exempt list (login, health,
    /// spotify-callback). When false, all requests pass through unchecked.
    /// Phase 6a ships with this default-false so the deploy keeps behaving
    /// like it does today; Phase 6c flips it once the bootstrap admin is
    /// configured.
    auth_enabled: bool,
}

type ApiError = (StatusCode, Json<Value>);
type ApiResult = Result<Json<Value>, ApiError>;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Pre-runtime CLI subcommand: `marinara-server hash-password <plaintext>`
    // prints a PHC argon2id hash and exits without starting the HTTP server.
    // The operator pipes this into the `MARINARA_ADMIN_PASSWORD_HASH` env var
    // so plaintext passwords never travel through EasyPanel's env editor.
    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 2 && args[1] == "hash-password" {
        if args.len() < 3 {
            eprintln!("usage: marinara-server hash-password <plaintext>");
            std::process::exit(2);
        }
        let hash = marinara_handlers::auth::hash_password(&args[2])
            .map_err(|error| Box::<dyn std::error::Error>::from(error.to_string()))?;
        println!("{hash}");
        return Ok(());
    }
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(async_main())
}

async fn async_main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,marinara_server=debug".into()),
        )
        .init();

    let data_dir = std::env::var("MARINARA_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/data"));
    std::fs::create_dir_all(&data_dir)?;
    let storage = FileStorage::new(data_dir.join("data"))?;
    info!("data dir: {}", data_dir.display());

    let backgrounds = AssetService::new(data_dir.join("backgrounds"))?;
    info!("backgrounds dir: {}", backgrounds.root().display());

    let game_assets = AssetService::new(data_dir.join("game-assets"))?;
    info!("game-assets dir: {}", game_assets.root().display());

    let lorebook_images = AssetService::new(data_dir.join("lorebooks").join("images"))?;
    info!("lorebook-images dir: {}", lorebook_images.root().display());

    let fonts = AssetService::new(data_dir.join("fonts"))?;
    info!("fonts dir: {}", fonts.root().display());

    let frontend_dir = std::env::var("MARINARA_FRONTEND_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/app/dist"));
    info!("frontend dir: {}", frontend_dir.display());

    let spotify_redirect_uri = std::env::var("MARINARA_SPOTIFY_REDIRECT_URI")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    if let Some(uri) = spotify_redirect_uri.as_deref() {
        info!("spotify redirect uri: {uri}");
    } else {
        info!(
            "spotify redirect uri not set (MARINARA_SPOTIFY_REDIRECT_URI); browser OAuth disabled"
        );
    }

    let auth_enabled = std::env::var("MARINARA_AUTH_ENABLED")
        .ok()
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes"
            )
        })
        .unwrap_or(false);
    info!("auth enabled: {auth_enabled} (set MARINARA_AUTH_ENABLED=true to enforce login)");

    // Bootstrap the admin row from env vars whenever both are set. Idempotent
    // on repeat boots; rotates the hash if the env var changes (which is the
    // forgotten-password recovery path).
    if let (Ok(username), Ok(hash)) = (
        std::env::var("MARINARA_ADMIN_USERNAME"),
        std::env::var("MARINARA_ADMIN_PASSWORD_HASH"),
    ) {
        let username = username.trim();
        let hash = hash.trim();
        if !username.is_empty() && !hash.is_empty() {
            match marinara_handlers::auth::bootstrap_admin(&storage, username, hash) {
                Ok(_) => info!("admin user '{username}' bootstrapped from env vars"),
                Err(error) => warn!("admin bootstrap failed: {}", error),
            }
        }
    } else if auth_enabled {
        warn!(
            "MARINARA_AUTH_ENABLED=true but MARINARA_ADMIN_USERNAME / \
             MARINARA_ADMIN_PASSWORD_HASH are not set — login will reject all attempts"
        );
    }

    let state = AppState {
        storage,
        backgrounds,
        game_assets,
        lorebook_images,
        fonts,
        data_dir: data_dir.clone(),
        llm_streams: Arc::new(LlmStreamRegistry::new()),
        spotify_redirect_uri,
        auth_enabled,
    };

    let api_router = Router::new()
        .route("/health", get(health))
        .route("/auth/login", post(auth_login))
        .route("/auth/logout", post(auth_logout))
        .route("/auth/me", get(auth_me))
        .route("/invoke/:command", post(invoke_command))
        .route("/stream/llm", post(llm_stream_sse))
        .route("/spotify/callback", get(spotify_browser_callback))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    let static_service = ServeDir::new(&frontend_dir)
        .not_found_service(ServeFile::new(frontend_dir.join("index.html")));

    // User-asset routes. The Tauri build uses custom URL schemes
    // (marinara-background:, marinara-game-asset:) that the desktop binary
    // intercepts via tauri's protocol-asset handler. The browser has no such
    // hook, so we expose the same files under /assets/<bucket>/<file>. The
    // frontend's local-file-api.ts rewrites the `marinara-*:` URLs onto these
    // when `platform.isWeb` is true. ServeDir handles Range, ETag, and the
    // right Content-Type via mime_guess automatically — no custom code path
    // required.
    let backgrounds_router =
        ServeDir::new(state.backgrounds.root()).append_index_html_on_directories(false);
    let game_assets_router =
        ServeDir::new(state.game_assets.root()).append_index_html_on_directories(false);
    let lorebook_images_router =
        ServeDir::new(state.lorebook_images.root()).append_index_html_on_directories(false);
    let fonts_router = ServeDir::new(state.fonts.root()).append_index_html_on_directories(false);

    let app = Router::new()
        .nest("/api", api_router)
        .nest_service("/assets/backgrounds", backgrounds_router)
        .nest_service("/assets/game-assets", game_assets_router)
        .nest_service("/assets/lorebook-images", lorebook_images_router)
        .nest_service("/assets/fonts", fonts_router)
        .fallback_service(static_service)
        .with_state(state);

    let addr = std::env::var("MARINARA_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".into());
    info!("listening on {addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

async fn invoke_command(
    State(state): State<AppState>,
    Path(command): Path<String>,
    Json(args): Json<Value>,
) -> ApiResult {
    let storage = &state.storage;
    let backgrounds = &state.backgrounds;
    let game_assets = &state.game_assets;
    let fonts = &state.fonts;
    let data_dir = state.data_dir.as_path();
    match command.as_str() {
        // ---- entities ---------------------------------------------------------
        "storage_list" => {
            let entity = required_str(&args, "entity")?;
            let options = args.get("options").cloned();
            marinara_handlers::entities::list(storage, entity, options)
                .map(Json)
                .map_err(error_response)
        }
        "storage_get" => {
            let entity = required_str(&args, "entity")?;
            let id = required_str(&args, "id")?;
            marinara_handlers::entities::get(storage, entity, id)
                .map(Json)
                .map_err(error_response)
        }
        "storage_create" => {
            let entity = required_str(&args, "entity")?;
            let value = args.get("value").cloned().unwrap_or(Value::Null);
            marinara_handlers::entities::create(storage, entity, value)
                .map(Json)
                .map_err(error_response)
        }
        "storage_update" => {
            let entity = required_str(&args, "entity")?;
            let id = required_str(&args, "id")?;
            let patch = args.get("patch").cloned().unwrap_or(Value::Null);
            marinara_handlers::entities::update(storage, entity, id, patch)
                .map(Json)
                .map_err(error_response)
        }
        "storage_delete" => {
            let entity = required_str(&args, "entity")?;
            let id = required_str(&args, "id")?;
            marinara_handlers::entities::delete(storage, entity, id)
                .map(Json)
                .map_err(error_response)
        }
        "storage_duplicate" => {
            let entity = required_str(&args, "entity")?;
            let id = required_str(&args, "id")?;
            marinara_handlers::entities::duplicate(storage, entity, id)
                .map(Json)
                .map_err(error_response)
        }

        // ---- chat memories/notes ---------------------------------------------
        "chat_memories_list" => {
            let chat_id = required_str(&args, "chatId")?;
            marinara_handlers::chats::chat_array_field(storage, chat_id, "memories")
                .map(Json)
                .map_err(error_response)
        }
        "chat_memory_delete" => {
            let chat_id = required_str(&args, "chatId")?;
            let memory_id = required_str(&args, "memoryId")?;
            marinara_handlers::chats::delete_chat_array_item(
                storage, chat_id, "memories", memory_id,
            )
            .map(Json)
            .map_err(error_response)
        }
        "chat_memories_clear" => {
            let chat_id = required_str(&args, "chatId")?;
            marinara_handlers::chats::set_chat_array_field(storage, chat_id, "memories", Vec::new())
                .map(Json)
                .map_err(error_response)
        }
        "chat_memories_refresh" => {
            let chat_id = required_str(&args, "chatId")?;
            marinara_handlers::chats::refresh_chat_memories(storage, chat_id)
                .map(Json)
                .map_err(error_response)
        }
        "chat_memories_export" => {
            let chat_id = required_str(&args, "chatId")?;
            marinara_handlers::chats::export_chat_memories(storage, chat_id)
                .map(Json)
                .map_err(error_response)
        }
        "chat_memories_import" => {
            let chat_id = required_str(&args, "chatId")?;
            let body = args.get("body").cloned().unwrap_or(Value::Null);
            marinara_handlers::chats::import_chat_memories(storage, chat_id, body)
                .map(Json)
                .map_err(error_response)
        }
        "chat_notes_list" => {
            let chat_id = required_str(&args, "chatId")?;
            marinara_handlers::chats::chat_array_field(storage, chat_id, "notes")
                .map(Json)
                .map_err(error_response)
        }
        "chat_note_delete" => {
            let chat_id = required_str(&args, "chatId")?;
            let note_id = required_str(&args, "noteId")?;
            marinara_handlers::chats::delete_chat_array_item(storage, chat_id, "notes", note_id)
                .map(Json)
                .map_err(error_response)
        }
        "chat_notes_clear" => {
            let chat_id = required_str(&args, "chatId")?;
            marinara_handlers::chats::set_chat_array_field(storage, chat_id, "notes", Vec::new())
                .map(Json)
                .map_err(error_response)
        }

        // ---- chat groups + branches ------------------------------------------
        "chat_group_delete" => {
            let group_id = required_str(&args, "groupId")?;
            marinara_handlers::chats::delete_chat_group(storage, group_id)
                .map(Json)
                .map_err(error_response)
        }
        "chat_branch" => {
            let chat_id = required_str(&args, "chatId")?;
            let up_to = args
                .get("upToMessageId")
                .and_then(Value::as_str)
                .map(|value| value.to_string());
            marinara_handlers::chats::branch_chat(
                storage,
                chat_id,
                json!({ "upToMessageId": up_to }),
            )
            .map(Json)
            .map_err(error_response)
        }

        // ---- chat autonomous unread ------------------------------------------
        "chat_autonomous_unread_mark" => {
            let chat_id = required_str(&args, "chatId")?;
            let body = args.get("body").cloned().unwrap_or(Value::Null);
            marinara_handlers::chats::mark_autonomous_unread(storage, chat_id, body)
                .map(Json)
                .map_err(error_response)
        }
        "chat_autonomous_unread_clear" => {
            let chat_id = required_str(&args, "chatId")?;
            marinara_handlers::chats::clear_autonomous_unread(storage, chat_id)
                .map(Json)
                .map_err(error_response)
        }

        // ---- chat messages ---------------------------------------------------
        "chat_messages_bulk_delete" => {
            let chat_id = required_str(&args, "chatId")?;
            let message_ids = args.get("messageIds").cloned().unwrap_or_else(|| json!([]));
            marinara_handlers::chats::bulk_delete_messages(
                storage,
                chat_id,
                json!({ "messageIds": message_ids }),
            )
            .map(Json)
            .map_err(error_response)
        }
        "chat_message_swipes" => {
            let chat_id = required_str(&args, "chatId")?;
            let message_id = required_str(&args, "messageId")?;
            marinara_handlers::chats::message_swipes(
                storage,
                "GET",
                chat_id,
                message_id,
                Value::Null,
            )
            .map(Json)
            .map_err(error_response)
        }
        "chat_message_add_swipe" => {
            let chat_id = required_str(&args, "chatId")?;
            let message_id = required_str(&args, "messageId")?;
            let body = args.get("body").cloned().unwrap_or(Value::Null);
            marinara_handlers::chats::message_swipes(storage, "POST", chat_id, message_id, body)
                .map(Json)
                .map_err(error_response)
        }
        "chat_message_set_active_swipe" => {
            let chat_id = required_str(&args, "chatId")?;
            let message_id = required_str(&args, "messageId")?;
            let index = args.get("index").and_then(Value::as_i64).unwrap_or(0);
            marinara_handlers::chats::set_active_swipe(
                storage,
                chat_id,
                message_id,
                json!({ "index": index }),
            )
            .map(Json)
            .map_err(error_response)
        }
        "chat_message_delete_swipe" => {
            let chat_id = required_str(&args, "chatId")?;
            let message_id = required_str(&args, "messageId")?;
            let index = required_str(&args, "index")?;
            marinara_handlers::chats::delete_swipe(storage, chat_id, message_id, index)
                .map(Json)
                .map_err(error_response)
        }
        "chat_connect" => {
            let chat_id = required_str(&args, "chatId")?;
            let target_chat_id = required_str(&args, "targetChatId")?;
            marinara_handlers::chats::connect(storage, chat_id, target_chat_id)
                .map(Json)
                .map_err(error_response)
        }
        "chat_disconnect" => {
            let chat_id = required_str(&args, "chatId")?;
            marinara_handlers::chats::disconnect(storage, chat_id)
                .map(Json)
                .map_err(error_response)
        }

        // ---- agents ----------------------------------------------------------
        "agent_patch_by_type" => {
            let agent_type = required_str(&args, "agentType")?;
            let patch = args.get("patch").cloned().unwrap_or(Value::Null);
            marinara_handlers::agents::patch_agent_type(storage, agent_type, patch)
                .map(Json)
                .map_err(error_response)
        }
        "agent_toggle_by_type" => {
            let agent_type = required_str(&args, "agentType")?;
            marinara_handlers::agents::toggle_agent_type(storage, agent_type)
                .map(Json)
                .map_err(error_response)
        }
        "agent_cadence_status" => {
            let agent_type = required_str(&args, "agentType")?;
            let chat_id = required_str(&args, "chatId")?;
            marinara_handlers::agents::agent_cadence_status(storage, agent_type, chat_id)
                .map(Json)
                .map_err(error_response)
        }
        "agent_memory_get" => {
            let agent_type = required_str(&args, "agentType")?;
            let chat_id = required_str(&args, "chatId")?;
            marinara_handlers::agents::agent_memory(
                storage,
                "GET",
                agent_type,
                chat_id,
                Value::Null,
            )
            .map(Json)
            .map_err(error_response)
        }
        "agent_memory_patch" => {
            let agent_type = required_str(&args, "agentType")?;
            let chat_id = required_str(&args, "chatId")?;
            let patch = args.get("patch").cloned().unwrap_or(Value::Null);
            marinara_handlers::agents::agent_memory(
                storage,
                "PATCH",
                agent_type,
                chat_id,
                json!({ "patch": patch }),
            )
            .map(Json)
            .map_err(error_response)
        }
        "agent_memory_clear" => {
            let agent_type = required_str(&args, "agentType")?;
            let chat_id = required_str(&args, "chatId")?;
            marinara_handlers::agents::agent_memory(
                storage,
                "DELETE",
                agent_type,
                chat_id,
                Value::Null,
            )
            .map(Json)
            .map_err(error_response)
        }
        "agent_runs_clear_for_chat" => {
            let chat_id = required_str(&args, "chatId")?;
            marinara_handlers::agents::clear_agent_runs_and_memory_for_chat(storage, chat_id)
                .map(Json)
                .map_err(error_response)
        }
        "agent_echo_messages_clear" => {
            let chat_id = required_str(&args, "chatId")?;
            marinara_handlers::agents::echo_messages(storage, "DELETE", chat_id)
                .map(Json)
                .map_err(error_response)
        }

        // ---- admin -----------------------------------------------------------
        "admin_expunge_command" => {
            let scopes = args.get("scopes").cloned().unwrap_or_else(|| json!([]));
            let outcome = marinara_handlers::admin::expunge(
                storage,
                json!({ "confirm": true, "scopes": scopes }),
            )
            .map_err(error_response)?;
            Ok(Json(marinara_handlers::admin::expunge_response(&outcome)))
        }
        "admin_clear_all_command" => {
            marinara_handlers::admin::clear_all_storage(storage).map_err(error_response)?;
            Ok(Json(marinara_handlers::admin::clear_all_response()))
        }

        // ---- backgrounds (filesystem under $MARINARA_DATA_DIR/backgrounds) --
        "backgrounds_list" => marinara_handlers::backgrounds::list(storage, backgrounds)
            .map(Json)
            .map_err(error_response),
        "backgrounds_tags" => marinara_handlers::backgrounds::tags(storage)
            .map(Json)
            .map_err(error_response),
        "background_upload" => {
            let body = args.get("body").cloned().unwrap_or_else(|| args.clone());
            marinara_handlers::backgrounds::upload(storage, backgrounds, body)
                .map(Json)
                .map_err(error_response)
        }
        "background_tags_update" => {
            let filename = required_str(&args, "filename")?;
            let tags = args.get("tags").cloned().unwrap_or_else(|| json!([]));
            marinara_handlers::backgrounds::update_tags(storage, filename, json!({ "tags": tags }))
                .map(Json)
                .map_err(error_response)
        }
        "background_rename" => {
            let filename = required_str(&args, "filename")?;
            let name = required_str(&args, "name")?;
            marinara_handlers::backgrounds::rename(
                storage,
                backgrounds,
                filename,
                json!({ "name": name }),
            )
            .map(Json)
            .map_err(error_response)
        }
        "background_delete" => {
            let filename = required_str(&args, "filename")?;
            marinara_handlers::backgrounds::delete(storage, backgrounds, filename)
                .map(Json)
                .map_err(error_response)
        }
        "background_file_path" => {
            let filename = required_str(&args, "filename")?;
            marinara_handlers::backgrounds::file_path(backgrounds, filename)
                .map(Json)
                .map_err(error_response)
        }

        // ---- game-assets (filesystem under $MARINARA_DATA_DIR/game-assets) --
        "game_assets_list" => {
            let path = args.get("path").and_then(Value::as_str);
            marinara_handlers::game_assets::list(game_assets, path)
                .map(Json)
                .map_err(error_response)
        }
        "game_assets_manifest" => marinara_handlers::game_assets::manifest(game_assets)
            .map(Json)
            .map_err(error_response),
        "game_assets_tree" => marinara_handlers::game_assets::tree(game_assets)
            .map(Json)
            .map_err(error_response),
        "game_assets_rescan" => marinara_handlers::game_assets::rescan(game_assets)
            .map(Json)
            .map_err(error_response),
        "game_assets_create_folder" => {
            let path = required_str(&args, "path")?;
            marinara_handlers::game_assets::create_folder(game_assets, path)
                .map(Json)
                .map_err(error_response)
        }
        "game_assets_delete_folder" => {
            let path = required_str(&args, "path")?;
            let recursive = args
                .get("recursive")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            marinara_handlers::game_assets::delete_folder(game_assets, path, recursive)
                .map(Json)
                .map_err(error_response)
        }
        "game_assets_delete_file" => {
            let path = required_str(&args, "path")?;
            marinara_handlers::game_assets::delete_file(game_assets, path)
                .map(Json)
                .map_err(error_response)
        }
        "game_assets_file_path" => {
            let path = required_str(&args, "path")?;
            marinara_handlers::game_assets::file_path(game_assets, path)
                .map(Json)
                .map_err(error_response)
        }
        "game_assets_read_text" => {
            let path = required_str(&args, "path")?;
            marinara_handlers::game_assets::read_text(game_assets, path)
                .map(Json)
                .map_err(error_response)
        }
        "game_assets_write_text" => {
            let path = required_str(&args, "path")?;
            let content = args.get("content").and_then(Value::as_str).unwrap_or("");
            marinara_handlers::game_assets::write_text(game_assets, path, content)
                .map(Json)
                .map_err(error_response)
        }
        "game_assets_rename" => {
            let path = required_str(&args, "path")?;
            let new_name = required_str(&args, "newName")?;
            marinara_handlers::game_assets::rename(game_assets, path, new_name)
                .map(Json)
                .map_err(error_response)
        }
        "game_assets_move" => {
            let path = required_str(&args, "path")?;
            let target = args
                .get("targetFolder")
                .and_then(Value::as_str)
                .unwrap_or("");
            marinara_handlers::game_assets::move_one(game_assets, path, target)
                .map(Json)
                .map_err(error_response)
        }
        "game_assets_copy" => {
            let path = required_str(&args, "path")?;
            let target = args
                .get("targetFolder")
                .and_then(Value::as_str)
                .unwrap_or("");
            marinara_handlers::game_assets::copy_one(game_assets, path, target)
                .map(Json)
                .map_err(error_response)
        }
        "game_assets_move_bulk" => {
            let paths = required_path_array(&args, "paths")?;
            let target = args
                .get("targetFolder")
                .and_then(Value::as_str)
                .unwrap_or("");
            marinara_handlers::game_assets::move_bulk(game_assets, &paths, target)
                .map(Json)
                .map_err(error_response)
        }
        "game_assets_copy_bulk" => {
            let paths = required_path_array(&args, "paths")?;
            let target = args
                .get("targetFolder")
                .and_then(Value::as_str)
                .unwrap_or("");
            marinara_handlers::game_assets::copy_bulk(game_assets, &paths, target)
                .map(Json)
                .map_err(error_response)
        }
        "game_assets_delete_bulk" => {
            let paths = required_path_array(&args, "paths")?;
            marinara_handlers::game_assets::delete_bulk(game_assets, &paths)
                .map(Json)
                .map_err(error_response)
        }
        "game_assets_file_info" => {
            let path = required_str(&args, "path")?;
            marinara_handlers::game_assets::file_info(game_assets, path)
                .map(Json)
                .map_err(error_response)
        }
        "game_assets_folder_description" => {
            let path = required_str(&args, "path")?;
            let description = args
                .get("description")
                .and_then(Value::as_str)
                .unwrap_or("");
            marinara_handlers::game_assets::folder_description(game_assets, path, description)
                .map(Json)
                .map_err(error_response)
        }
        "game_assets_upload" => {
            let body = args.get("body").cloned().unwrap_or_else(|| args.clone());
            marinara_handlers::game_assets::upload(game_assets, body)
                .map(Json)
                .map_err(error_response)
        }
        "game_assets_open_folder" => {
            let subfolder = args.get("subfolder").and_then(Value::as_str);
            marinara_handlers::game_assets::open_folder(game_assets, subfolder)
                .map(Json)
                .map_err(error_response)
        }

        // ---- fonts (filesystem under $MARINARA_DATA_DIR/fonts) -----------------
        // The browser loads font files via the `/assets/fonts/` ServeDir mount
        // configured above; `fonts_file` is a base64 fallback that nobody on
        // the web target should hit in practice but stays wired for symmetry
        // with the Tauri side. `fonts_open_folder` succeeds with `opened: false`
        // — the server has no GUI to pop, and the frontend already treats the
        // call as fire-and-forget (see SettingsPanel.tsx).
        "fonts_list" => marinara_handlers::fonts::list(fonts)
            .map(Json)
            .map_err(error_response),
        "fonts_file" => {
            let filename = required_str(&args, "filename")?;
            marinara_handlers::fonts::file(fonts, filename)
                .map(Json)
                .map_err(error_response)
        }
        "fonts_open_folder" => marinara_handlers::fonts::open_folder(fonts)
            .map(Json)
            .map_err(error_response),
        "fonts_google_download" => marinara_handlers::fonts::download_google(fonts, args.clone())
            .await
            .map(Json)
            .map_err(error_response),

        // ---- imports (marinara envelope, ST card/chat/preset/lorebook, bulk) -
        // The non-streaming Tauri commands all go through a single dispatcher
        // keyed by a scope path. We mirror that here per-command so the
        // /api/invoke/<command> contract on the frontend stays uniform with
        // the rest of the surface. `import_st_bulk_run_events` is the only
        // streaming variant — it stays desktop-only because the Axum side
        // would want SSE rather than tauri::ipc::Channel.
        "import_marinara" => {
            let envelope = args
                .get("envelope")
                .cloned()
                .unwrap_or_else(|| args.clone());
            marinara_handlers::imports::import_call(storage, data_dir, &["marinara"], envelope)
                .map(Json)
                .map_err(error_response)
        }
        "import_marinara_file" => {
            let body = args.get("body").cloned().unwrap_or_else(|| args.clone());
            marinara_handlers::imports::import_call(storage, data_dir, &["marinara-file"], body)
                .map(Json)
                .map_err(error_response)
        }
        "import_st_character" => {
            let body = args.get("body").cloned().unwrap_or_else(|| args.clone());
            marinara_handlers::imports::import_call(storage, data_dir, &["st-character"], body)
                .map(Json)
                .map_err(error_response)
        }
        "import_st_character_batch" => {
            let body = args.get("body").cloned().unwrap_or_else(|| args.clone());
            marinara_handlers::imports::import_call(
                storage,
                data_dir,
                &["st-character", "batch"],
                body,
            )
            .map(Json)
            .map_err(error_response)
        }
        "import_st_character_inspect" => {
            let body = args.get("body").cloned().unwrap_or_else(|| args.clone());
            marinara_handlers::imports::import_call(
                storage,
                data_dir,
                &["st-character", "inspect"],
                body,
            )
            .map(Json)
            .map_err(error_response)
        }
        "import_st_chat" => {
            let body = args.get("body").cloned().unwrap_or_else(|| args.clone());
            marinara_handlers::imports::import_call(storage, data_dir, &["st-chat"], body)
                .map(Json)
                .map_err(error_response)
        }
        "import_st_chat_into_group" => {
            let body = args.get("body").cloned().unwrap_or_else(|| args.clone());
            marinara_handlers::imports::import_call(
                storage,
                data_dir,
                &["st-chat-into-group"],
                body,
            )
            .map(Json)
            .map_err(error_response)
        }
        "import_st_preset" => {
            let payload = args.get("payload").cloned().unwrap_or_else(|| args.clone());
            marinara_handlers::imports::import_call(storage, data_dir, &["st-preset"], payload)
                .map(Json)
                .map_err(error_response)
        }
        "import_st_lorebook" => {
            let payload = args.get("payload").cloned().unwrap_or_else(|| args.clone());
            marinara_handlers::imports::import_call(storage, data_dir, &["st-lorebook"], payload)
                .map(Json)
                .map_err(error_response)
        }
        "import_list_directory" => {
            let path = required_str(&args, "path")?;
            let picker_selected = args
                .get("pickerSelected")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            marinara_handlers::imports::import_call(
                storage,
                data_dir,
                &["list-directory"],
                json!({ "path": path, "pickerSelected": picker_selected }),
            )
            .map(Json)
            .map_err(error_response)
        }
        "import_st_bulk_scan" => {
            let payload = args.get("payload").cloned().unwrap_or_else(|| args.clone());
            marinara_handlers::imports::import_call(
                storage,
                data_dir,
                &["st-bulk", "scan"],
                payload,
            )
            .map(Json)
            .map_err(error_response)
        }
        "import_st_bulk_run" => {
            let payload = args.get("payload").cloned().unwrap_or_else(|| args.clone());
            marinara_handlers::imports::import_call(storage, data_dir, &["st-bulk", "run"], payload)
                .map(Json)
                .map_err(error_response)
        }

        // ---- profile snapshot + import --------------------------------------
        // profile_import_file stays desktop-only: the Tauri command takes an
        // OS path the desktop dialog produced. Web clients reach `import_profile`
        // by uploading the parsed envelope JSON directly.
        "profile_export" => {
            let format = args.get("format").and_then(Value::as_str);
            marinara_handlers::profile::export_profile(storage, data_dir, format)
                .map(Json)
                .map_err(error_response)
        }
        "profile_import" => {
            let envelope = args
                .get("envelope")
                .cloned()
                .unwrap_or_else(|| args.clone());
            marinara_handlers::profile::import_profile(storage, data_dir, envelope)
                .map(Json)
                .map_err(error_response)
        }

        // ---- lorebook images (uploads + per-file path resolver) -------------
        "lorebook_image_upload" => {
            let id = required_str(&args, "id")?;
            let body = args.get("body").cloned().unwrap_or_else(|| args.clone());
            marinara_handlers::lorebook_images::update_lorebook_image(storage, data_dir, id, body)
                .map(Json)
                .map_err(error_response)
        }
        "lorebook_image_file_path" => {
            let filename = required_str(&args, "filename")?;
            marinara_handlers::lorebook_images::lorebook_image_file_path(data_dir, filename)
                .map(Json)
                .map_err(error_response)
        }

        // ---- LLM non-streaming (Phase 4a) -----------------------------------
        // Streaming (`llm_stream_channel`) stays on the Tauri-only invoke path
        // until Phase 4b lifts it to an SSE route. Image-generation connection
        // testing reuses the same handler because the helpers it depends on
        // are pure-JSON shims in `marinara_handlers::llm`; image *generation*
        // itself is deferred to Phase 4c.
        "llm_complete" => {
            let body = args.get("body").cloned().unwrap_or_else(|| args.clone());
            marinara_handlers::llm::llm_complete(storage, body)
                .await
                .map(Json)
                .map_err(error_response)
        }
        "llm_list_models" => {
            let connection_id = args.get("connectionId").and_then(Value::as_str);
            marinara_handlers::llm::llm_models(storage, connection_id)
                .await
                .map(Json)
                .map_err(error_response)
        }
        "connection_models" => {
            let id = required_str(&args, "id")?;
            marinara_handlers::llm::connection_models(storage, id)
                .await
                .map(Json)
                .map_err(error_response)
        }
        "connection_test" => {
            let id = required_str(&args, "id")?;
            marinara_handlers::llm::connection_test(storage, id)
                .await
                .map(Json)
                .map_err(error_response)
        }
        "connection_test_message" => {
            let id = required_str(&args, "id")?;
            marinara_handlers::llm::connection_test_message(storage, id)
                .await
                .map(Json)
                .map_err(error_response)
        }

        // ---- Spotify (Phase 4e) ---------------------------------------------
        // 22 commands wrapping the Spotify Web API. The shared dispatcher
        // takes a ParsedPath (path + query map) the Tauri integration shim
        // constructs from the request URL; on the server side we build one
        // from scratch using the original Tauri command's path string so
        // body-fallback lookups inside the dispatcher keep working.
        //
        // Loopback OAuth callback (localhost:8754 TCP listener) stays
        // Tauri-only. The new HTTPS callback route the web client needs
        // arrives in Phase 4f; until then, `authorize` returns
        // `callbackListenerStarted: false` on the server target — users
        // with a token already saved on the /data volume can still hit the
        // other endpoints.
        "spotify_status" => spotify_invoke(&state, "POST", &["status"], "/spotify", args).await,
        "spotify_authorize" => {
            // Phase 4f: inject the server's HTTPS redirect URI so the lifted
            // `authorize` writes the right `redirect_uri` into both the auth
            // URL and the pending-session row. Without this the browser
            // path would still send users at the loopback default (which
            // only the desktop binary can answer).
            let mut body = args.clone();
            if let Some(uri) = state.spotify_redirect_uri.as_deref() {
                if let Some(object) = body.as_object_mut() {
                    object.insert("redirectUri".to_string(), Value::String(uri.to_string()));
                }
            }
            spotify_invoke(&state, "POST", &["authorize"], "/spotify", body).await
        }
        "spotify_exchange" => {
            let body = args
                .get("body")
                .cloned()
                .or_else(|| {
                    args.get("callbackUrl")
                        .map(|url| json!({ "callbackUrl": url }))
                })
                .unwrap_or_else(|| args.clone());
            spotify_dispatch(&state, "POST", &["exchange"], "/spotify", body).await
        }
        "spotify_disconnect" => {
            spotify_invoke(&state, "POST", &["disconnect"], "/spotify", args).await
        }
        "spotify_player" => spotify_invoke(&state, "GET", &["player"], "/spotify", args).await,
        "spotify_devices" => spotify_invoke(&state, "GET", &["devices"], "/spotify", args).await,
        "spotify_access_token" => {
            spotify_invoke(&state, "GET", &["access-token"], "/spotify", args).await
        }
        "spotify_playlists" => {
            let limit = args.get("limit").and_then(Value::as_u64).unwrap_or(50);
            let agent_id = args.get("agentId").cloned().unwrap_or(Value::Null);
            spotify_dispatch(
                &state,
                "GET",
                &["playlists"],
                &format!("/spotify/playlists?limit={limit}"),
                json!({ "agentId": agent_id }),
            )
            .await
        }
        "spotify_playlist_tracks" => {
            spotify_invoke(&state, "POST", &["playlist-tracks"], "/spotify", args).await
        }
        "spotify_search_tracks" => {
            spotify_invoke(&state, "POST", &["search-tracks"], "/spotify", args).await
        }
        "spotify_play_track" => {
            spotify_invoke(&state, "POST", &["play-track"], "/spotify", args).await
        }
        "spotify_dj_mari_playlist" => {
            spotify_invoke(&state, "POST", &["dj-mari-playlist"], "/spotify", args).await
        }
        "spotify_player_play" => {
            spotify_invoke(&state, "PUT", &["player", "play"], "/spotify", args).await
        }
        "spotify_player_pause" => {
            spotify_invoke(&state, "PUT", &["player", "pause"], "/spotify", args).await
        }
        "spotify_player_next" => {
            spotify_invoke(&state, "POST", &["player", "next"], "/spotify", args).await
        }
        "spotify_player_previous" => {
            spotify_invoke(&state, "POST", &["player", "previous"], "/spotify", args).await
        }
        "spotify_player_transfer" => {
            spotify_invoke(&state, "PUT", &["player", "transfer"], "/spotify", args).await
        }
        "spotify_player_volume" => {
            spotify_invoke(&state, "PUT", &["player", "volume"], "/spotify", args).await
        }
        "spotify_player_shuffle" => {
            spotify_invoke(&state, "PUT", &["player", "shuffle"], "/spotify", args).await
        }
        "spotify_player_repeat" => {
            spotify_invoke(&state, "PUT", &["player", "repeat"], "/spotify", args).await
        }

        // ---- TTS (Phase 4d) --------------------------------------------------
        // All four routes dispatch through `tts_call`, mirroring the Tauri
        // integration command shim. The audio bytes come back as base64
        // inside the JSON envelope (`audioBase64` + `contentType`); the
        // frontend already handles that shape.
        "tts_config" => {
            marinara_handlers::integrations::tts::tts_call(storage, "GET", &["config"], Value::Null)
                .await
                .map(Json)
                .map_err(error_response)
        }
        "tts_update_config" => {
            let config = args.get("config").cloned().unwrap_or(Value::Null);
            marinara_handlers::integrations::tts::tts_call(storage, "PUT", &["config"], config)
                .await
                .map(Json)
                .map_err(error_response)
        }
        "tts_voices" => {
            marinara_handlers::integrations::tts::tts_call(storage, "GET", &["voices"], Value::Null)
                .await
                .map(Json)
                .map_err(error_response)
        }
        "tts_speak" => {
            let input = args.get("input").cloned().unwrap_or_else(|| args.clone());
            marinara_handlers::integrations::tts::tts_call(storage, "POST", &["speak"], input)
                .await
                .map(Json)
                .map_err(error_response)
        }

        // ---- Image generation (Phase 4c) -----------------------------------
        // `image_generate` / `avatar_generation_command` / `connection_test_image`
        // hit the provider HTTP surface (Stability, ComfyUI, NovelAI, OpenAI
        // image, etc.) through marinara_handlers::images. The preview command
        // is sync; everything else is async.
        "image_generate" => {
            let body = args.get("body").cloned().unwrap_or_else(|| args.clone());
            marinara_handlers::images::generate_image(storage, body)
                .await
                .map(Json)
                .map_err(error_response)
        }
        "avatar_generation_command" => {
            let body = args.get("body").cloned().unwrap_or_else(|| args.clone());
            marinara_handlers::images::avatar_generation(storage, body)
                .await
                .map(Json)
                .map_err(error_response)
        }
        "avatar_generation_preview_command" => {
            let body = args.get("body").cloned().unwrap_or_else(|| args.clone());
            marinara_handlers::images::avatar_generation_preview(body)
                .map(Json)
                .map_err(error_response)
        }
        "connection_test_image" => {
            let id = required_str(&args, "id")?;
            marinara_handlers::images::test_image_generation(storage, id)
                .await
                .map(Json)
                .map_err(error_response)
        }

        // ---- LLM stream cancel (Phase 4b) -----------------------------------
        // The actual streaming endpoint is `POST /api/stream/llm` (SSE).
        // Cancel is a plain invoke because it's a fire-and-forget signal —
        // no event stream to return.
        "llm_stream_cancel" => {
            let stream_id = required_str(&args, "streamId")?;
            marinara_handlers::llm::llm_stream_cancel(&state.llm_streams, stream_id)
                .map(Json)
                .map_err(error_response)
        }

        // ---- tracker snapshots -----------------------------------------------
        "tracker_snapshot_latest" => {
            let chat_id = required_str(&args, "chatId")?;
            marinara_handlers::tracker::latest_snapshot(storage, chat_id)
                .map(|value| Json(value.unwrap_or(Value::Null)))
                .map_err(error_response)
        }
        "tracker_snapshot_get" => {
            let chat_id = required_str(&args, "chatId")?;
            let message_id = required_str(&args, "messageId")?;
            let swipe_index = args.get("swipeIndex").and_then(Value::as_i64).unwrap_or(0);
            marinara_handlers::tracker::snapshot_for_target(
                storage,
                chat_id,
                message_id,
                swipe_index,
            )
            .map(|value| Json(value.unwrap_or(Value::Null)))
            .map_err(error_response)
        }
        "tracker_snapshot_save" => {
            let chat_id = required_str(&args, "chatId")?;
            let snapshot = args.get("snapshot").cloned().unwrap_or(Value::Null);
            marinara_handlers::tracker::save_snapshot(storage, chat_id, snapshot)
                .map(Json)
                .map_err(error_response)
        }

        _ => {
            warn!("unimplemented command: {command}");
            Err((
                StatusCode::NOT_IMPLEMENTED,
                Json(json!({
                    "code": "not_yet_implemented",
                    "message": format!(
                        "Command '{command}' is not yet exposed on the server target"
                    ),
                })),
            ))
        }
    }
}

fn required_str<'a>(args: &'a Value, key: &str) -> Result<&'a str, ApiError> {
    args.get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| error_response(AppError::invalid_input(format!("{key} is required"))))
}

fn required_path_array(args: &Value, key: &str) -> Result<Vec<String>, ApiError> {
    args.get(key)
        .and_then(Value::as_array)
        .map(|array| {
            array
                .iter()
                .filter_map(Value::as_str)
                .map(ToOwned::to_owned)
                .collect()
        })
        .ok_or_else(|| {
            error_response(AppError::invalid_input(format!(
                "{key} must be a non-empty string array"
            )))
        })
}

// ---------------------------------------------------------------------------
// Phase 4b: SSE streaming for llm_stream_channel
//
// The Tauri-side command rides on `tauri::ipc::Channel<Value>`; the browser
// gets the same event payloads via Server-Sent Events on this route. We POST
// rather than use EventSource because the request body (messages + tools +
// parameters) is too large for a query string, and EventSource only allows
// GET. The frontend `llm-api.ts` web path uses `fetch()` with a
// `ReadableStream` reader to parse the `data:` lines.
//
// Cancellation: the frontend separately POSTs `/api/invoke/llm_stream_cancel`
// with the same `streamId`. That handler reaches the per-process
// `LlmStreamRegistry` and flips the watch the spawned task is selecting on,
// so the in-flight provider request gets dropped mid-flight.
//
// Errors before the stream task spawns (missing streamId, bad body shape)
// surface as a normal HTTP 4xx/5xx via the IntoResponse impl on the Err arm.
// Errors *during* streaming (provider hangup, JSON parse) get emitted as a
// final `{"type":"error","text":...}` event so the consumer notices and
// closes the stream cleanly.
// ---------------------------------------------------------------------------
async fn llm_stream_sse(
    State(state): State<AppState>,
    Json(args): Json<Value>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, ApiError> {
    let stream_id = required_str(&args, "streamId")?.to_string();
    let request_body = args
        .get("request")
        .cloned()
        .or_else(|| args.get("body").cloned())
        .unwrap_or_else(|| {
            let mut clone = args.clone();
            if let Some(object) = clone.as_object_mut() {
                object.remove("streamId");
            }
            clone
        });

    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Result<Event, Infallible>>();
    let storage = state.storage.clone();
    let registry = state.llm_streams.clone();
    let emit_tx = tx.clone();
    tokio::spawn(async move {
        let result = marinara_handlers::llm::llm_stream(
            &storage,
            &registry,
            &stream_id,
            request_body,
            move |event| {
                let payload = Event::default()
                    .json_data(&event)
                    .map_err(|error| AppError::new("sse_serialize_error", error.to_string()))?;
                emit_tx
                    .send(Ok(payload))
                    .map_err(|error| AppError::new("sse_send_error", error.to_string()))
            },
        )
        .await;
        if let Err(error) = result {
            let payload = json!({
                "type": "error",
                "text": error.message,
                "code": error.code,
            });
            if let Ok(event) = Event::default().json_data(&payload) {
                let _ = tx.send(Ok(event));
            }
        }
        // Dropping `tx` ends the receiver stream and lets Axum flush the
        // final response chunk.
    });

    Ok(Sse::new(UnboundedReceiverStream::new(rx)).keep_alive(KeepAlive::default()))
}

// ---------------------------------------------------------------------------
// Phase 4f: browser-side Spotify OAuth callback
//
// Spotify redirects the user's browser to this route after the user clicks
// "Authorize". We hand the code + state off to the shared `exchange`
// handler (same code path the Tauri loopback listener uses), then 302
// the user back to the SPA root with a status query the frontend can pick
// up on load (`?spotify=connected` or `?spotify=error&message=...`).
//
// This route only works if `MARINARA_SPOTIFY_REDIRECT_URI` is set to its
// exact public URL — Spotify's PKCE flow requires the redirect_uri at the
// /authorize and /token legs to match byte-for-byte. The Tauri target
// keeps using the loopback default and ignores this route entirely.
// ---------------------------------------------------------------------------
async fn spotify_browser_callback(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Redirect {
    if let Some(error) = params.get("error") {
        return redirect_to_spa("error", Some(error));
    }
    let Some(code) = params.get("code").cloned() else {
        return redirect_to_spa("error", Some("missing_code"));
    };
    let Some(auth_state) = params.get("state").cloned() else {
        return redirect_to_spa("error", Some("missing_state"));
    };
    match marinara_handlers::integrations::spotify::exchange(
        &state.storage,
        json!({ "code": code, "state": auth_state }),
    )
    .await
    {
        Ok(_) => redirect_to_spa("connected", None),
        Err(error) => {
            warn!("spotify callback exchange failed: {}", error.message);
            redirect_to_spa("error", Some(&error.message))
        }
    }
}

fn redirect_to_spa(status: &str, message: Option<&str>) -> Redirect {
    let mut query = format!("spotify={}", urlencode(status));
    if let Some(message) = message {
        query.push_str(&format!("&message={}", urlencode(message)));
    }
    Redirect::to(&format!("/?{query}"))
}

fn urlencode(value: &str) -> String {
    // Same RFC 3986 unreserved-set encoding the handlers crate uses for
    // path components. Adequate for the small status/message strings here.
    marinara_handlers::shared::percent_encode_component(value)
}

// ---------------------------------------------------------------------------
// Phase 4e: Spotify dispatch helpers
//
// The lifted Spotify module reads `agentId`, `limit`, etc. from either a
// ParsedPath query map (Tauri integration shim path) or from the request
// body. Most server-side routes just need an empty path (body-only lookup);
// `spotify_playlists` is the lone exception that puts `limit` in the URL.
// `spotify_invoke` covers the common case; `spotify_dispatch` lets the
// caller supply a custom path + body for the exceptions.
// ---------------------------------------------------------------------------
async fn spotify_invoke(
    state: &AppState,
    method: &str,
    rest: &[&str],
    path: &str,
    body: Value,
) -> ApiResult {
    spotify_dispatch(state, method, rest, path, body).await
}

async fn spotify_dispatch(
    state: &AppState,
    method: &str,
    rest: &[&str],
    path: &str,
    body: Value,
) -> ApiResult {
    let route = marinara_handlers::integrations::spotify::ParsedPath::new(path);
    marinara_handlers::integrations::spotify::spotify_call(
        &state.storage,
        method,
        rest,
        &route,
        body,
    )
    .await
    .map(Json)
    .map_err(error_response)
}

fn error_response(error: AppError) -> ApiError {
    let status = match error.code.as_str() {
        "not_found" => StatusCode::NOT_FOUND,
        "invalid_input" => StatusCode::BAD_REQUEST,
        "invalid_credentials" | "unauthorized" => StatusCode::UNAUTHORIZED,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };
    let body = json!({
        "code": error.code,
        "message": error.message,
        "details": error.details,
    });
    (status, Json(body))
}

// --- Phase 6a: cookie-based auth -------------------------------------------

/// Lifetime of the session cookie when the user logs in. Matches Spotify's
/// typical refresh-token lifetime so the SPA's cached state stays warm for a
/// month at a time. The session row also has a `lastSeen` field that gets
/// touched on every authenticated request — a future Phase 6.x can cull
/// idle sessions without needing the cookie itself to expire.
const SESSION_COOKIE_MAX_AGE_SECONDS: u64 = 60 * 60 * 24 * 30;

/// Lift the named cookie value out of a `Cookie` header. Returns `None` if
/// the header is missing, malformed, or doesn't contain the requested name.
fn extract_cookie(headers: &axum::http::HeaderMap, name: &str) -> Option<String> {
    let raw = headers.get("cookie")?.to_str().ok()?;
    let prefix = format!("{name}=");
    for part in raw.split(';') {
        let trimmed = part.trim();
        if let Some(value) = trimmed.strip_prefix(&prefix) {
            return Some(value.to_string());
        }
    }
    None
}

fn session_cookie_value(session_id: &str) -> String {
    format!(
        "{SESSION_COOKIE_NAME}={session_id}; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age={SESSION_COOKIE_MAX_AGE_SECONDS}",
    )
}

fn clear_session_cookie() -> String {
    format!("{SESSION_COOKIE_NAME}=; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age=0")
}

fn unauthorized(code: &str, message: &str) -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(json!({ "code": code, "message": message, "details": null })),
    )
        .into_response()
}

/// Paths that bypass the auth middleware entirely. The middleware is only
/// mounted on `/api/*` routes, so `/login`, `/`, and the asset ServeDir
/// mounts already bypass it by virtue of not being layered. This is the
/// "things inside /api/* that should still be reachable without a session"
/// list.
fn is_auth_exempt_path(path: &str) -> bool {
    matches!(
        path,
        "/api/auth/login" | "/api/health" | "/api/spotify/callback"
    )
}

/// Gate every protected `/api/*` request on a valid session cookie. On
/// mutating methods (POST/PUT/PATCH/DELETE) the request must also carry the
/// matching `X-CSRF-Token` header. The middleware passes through when
/// `MARINARA_AUTH_ENABLED` is unset/false so the Phase 6a deploy keeps
/// behaving like Phase 5 until the operator opts in.
async fn auth_middleware(State(state): State<AppState>, request: Request, next: Next) -> Response {
    if !state.auth_enabled {
        return next.run(request).await;
    }
    let path = request.uri().path().to_string();
    if is_auth_exempt_path(&path) {
        return next.run(request).await;
    }
    let Some(session_id) = extract_cookie(request.headers(), SESSION_COOKIE_NAME) else {
        return unauthorized("unauthorized", "Session cookie missing");
    };
    let resolved = match marinara_handlers::auth::resolve_session(&state.storage, &session_id) {
        Ok(Some(value)) => value,
        Ok(None) => return unauthorized("unauthorized", "Session not found or expired"),
        Err(error) => {
            warn!("session resolve failed: {error}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "code": error.code, "message": error.message })),
            )
                .into_response();
        }
    };
    let method = request.method().clone();
    if method != Method::GET && method != Method::HEAD && method != Method::OPTIONS {
        let provided = request
            .headers()
            .get(CSRF_HEADER_NAME)
            .and_then(|h| h.to_str().ok())
            .unwrap_or("");
        let expected = resolved
            .get("csrfToken")
            .and_then(Value::as_str)
            .unwrap_or("");
        if provided.is_empty() || provided != expected {
            return unauthorized("unauthorized", "CSRF token missing or mismatched");
        }
    }
    next.run(request).await
}

async fn auth_login(State(state): State<AppState>, Json(body): Json<Value>) -> Response {
    let username = body.get("username").and_then(Value::as_str).unwrap_or("");
    let password = body.get("password").and_then(Value::as_str).unwrap_or("");
    let result = match marinara_handlers::auth::login(&state.storage, username, password) {
        Ok(value) => value,
        Err(error) => {
            let (status, body) = error_response(error);
            return (status, body).into_response();
        }
    };
    let session_id = result
        .get("sessionId")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let response_body = json!({
        "user": result.get("user").cloned().unwrap_or(Value::Null),
        "csrfToken": result.get("csrfToken").cloned().unwrap_or(Value::Null),
    });
    let mut response = (StatusCode::OK, Json(response_body)).into_response();
    if let Ok(value) = HeaderValue::from_str(&session_cookie_value(&session_id)) {
        response.headers_mut().append("set-cookie", value);
    }
    response
}

async fn auth_logout(State(state): State<AppState>, request: Request) -> Response {
    let session_id = extract_cookie(request.headers(), SESSION_COOKIE_NAME).unwrap_or_default();
    let _ = marinara_handlers::auth::logout(&state.storage, &session_id);
    let mut response = (StatusCode::OK, Json(json!({ "ok": true }))).into_response();
    if let Ok(value) = HeaderValue::from_str(&clear_session_cookie()) {
        response.headers_mut().append("set-cookie", value);
    }
    response
}

async fn auth_me(State(state): State<AppState>, request: Request) -> Response {
    // When auth is disabled the SPA still calls /auth/me on boot to decide
    // whether to show the login form. Return a sentinel "auth not enforced"
    // payload so the frontend can short-circuit to the chat UI without
    // surfacing a fake user identity.
    if !state.auth_enabled {
        return (
            StatusCode::OK,
            Json(json!({ "enforced": false, "user": null, "csrfToken": null })),
        )
            .into_response();
    }
    let Some(session_id) = extract_cookie(request.headers(), SESSION_COOKIE_NAME) else {
        return unauthorized("unauthorized", "Session cookie missing");
    };
    match marinara_handlers::auth::resolve_session(&state.storage, &session_id) {
        Ok(Some(resolved)) => {
            let mut payload = json!({ "enforced": true });
            if let Some(map) = payload.as_object_mut() {
                map.insert(
                    "user".to_string(),
                    resolved.get("user").cloned().unwrap_or(Value::Null),
                );
                map.insert(
                    "csrfToken".to_string(),
                    resolved.get("csrfToken").cloned().unwrap_or(Value::Null),
                );
            }
            (StatusCode::OK, Json(payload)).into_response()
        }
        Ok(None) => unauthorized("unauthorized", "Session not found or expired"),
        Err(error) => {
            warn!("auth_me resolve failed: {error}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "code": error.code, "message": error.message })),
            )
                .into_response()
        }
    }
}
