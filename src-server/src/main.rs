use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use marinara_assets::AssetService;
use marinara_core::AppError;
use marinara_storage::FileStorage;
use serde_json::{json, Value};
use std::path::PathBuf;
use tower_http::services::{ServeDir, ServeFile};
use tracing::{info, warn};

#[derive(Clone)]
struct AppState {
    storage: FileStorage,
    backgrounds: AssetService,
    game_assets: AssetService,
}

type ApiError = (StatusCode, Json<Value>);
type ApiResult = Result<Json<Value>, ApiError>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    let frontend_dir = std::env::var("MARINARA_FRONTEND_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/app/dist"));
    info!("frontend dir: {}", frontend_dir.display());

    let state = AppState {
        storage,
        backgrounds,
        game_assets,
    };

    let api_router = Router::new()
        .route("/health", get(health))
        .route("/invoke/:command", post(invoke_command));

    let static_service = ServeDir::new(&frontend_dir)
        .not_found_service(ServeFile::new(frontend_dir.join("index.html")));

    let app = Router::new()
        .nest("/api", api_router)
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
            let message_ids = args
                .get("messageIds")
                .cloned()
                .unwrap_or_else(|| json!([]));
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

fn error_response(error: AppError) -> ApiError {
    let status = match error.code.as_str() {
        "not_found" => StatusCode::NOT_FOUND,
        "invalid_input" => StatusCode::BAD_REQUEST,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };
    let body = json!({
        "code": error.code,
        "message": error.message,
        "details": error.details,
    });
    (status, Json(body))
}
