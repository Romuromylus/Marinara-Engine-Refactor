use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use marinara_core::AppError;
use marinara_storage::FileStorage;
use serde_json::{json, Value};
use std::path::PathBuf;
use tower_http::services::{ServeDir, ServeFile};
use tracing::info;

#[derive(Clone)]
struct AppState {
    storage: FileStorage,
}

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

    let frontend_dir = std::env::var("MARINARA_FRONTEND_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/app/dist"));
    info!("frontend dir: {}", frontend_dir.display());

    let state = AppState { storage };

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
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match command.as_str() {
        "storage_list" => {
            let entity = args
                .get("entity")
                .and_then(Value::as_str)
                .ok_or_else(|| error_response(AppError::invalid_input("entity is required")))?;
            let options = args.get("options").cloned();
            marinara_handlers::storage::list(&state.storage, entity, options)
                .map(Json)
                .map_err(error_response)
        }
        _ => Err((
            StatusCode::NOT_IMPLEMENTED,
            Json(json!({
                "code": "not_yet_implemented",
                "message": format!(
                    "Command '{command}' is not yet exposed on the server target"
                ),
            })),
        )),
    }
}

fn error_response(error: AppError) -> (StatusCode, Json<Value>) {
    let status = match error.code.as_str() {
        "not_found" => StatusCode::NOT_FOUND,
        "invalid_input" => StatusCode::BAD_REQUEST,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };
    let body = serde_json::to_value(&error)
        .unwrap_or_else(|_| json!({ "code": "serialization_error", "message": "unknown" }));
    (status, Json(body))
}
