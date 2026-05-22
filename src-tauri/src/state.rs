use marinara_assets::AssetService;
use marinara_core::{AppError, AppResult};
use marinara_handlers::llm::LlmStreamRegistry;
use marinara_storage::FileStorage;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Manager};

use crate::seed_defaults::seed_bundled_defaults;

#[derive(Clone)]
pub struct AppState {
    pub storage: FileStorage,
    pub game_assets: AssetService,
    pub backgrounds: AssetService,
    pub fonts: AssetService,
    pub data_dir: PathBuf,
    /// Phase 4b lifted the registry into `marinara-handlers` so the Axum
    /// server target can share the same cancellation surface. The Tauri
    /// AppState just holds the same `Arc<LlmStreamRegistry>` the server does.
    pub llm_streams: Arc<LlmStreamRegistry>,
}

impl AppState {
    pub fn new(app: &AppHandle) -> AppResult<Self> {
        let data_dir = app
            .path()
            .app_data_dir()
            .map_err(|error| AppError::new("data_dir_error", error.to_string()))?;
        std::fs::create_dir_all(&data_dir)?;
        let storage = FileStorage::new(data_dir.join("data"))?;
        let game_assets = AssetService::new(data_dir.join("game-assets"))?;
        let backgrounds = AssetService::new(data_dir.join("backgrounds"))?;
        let fonts = AssetService::new(data_dir.join("fonts"))?;
        let mut default_data_roots = Vec::new();
        if let Ok(resource_dir) = app.path().resource_dir() {
            default_data_roots.push(resource_dir.join("resources").join("default-data"));
        }
        default_data_roots.push(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("resources")
                .join("default-data"),
        );

        for default_data in default_data_roots {
            if !default_data.exists() {
                continue;
            }
            seed_bundled_defaults(&storage, &default_data)?;
            game_assets.seed_missing_from(&default_data.join("game-assets"))?;
            backgrounds.seed_missing_from(&default_data.join("backgrounds"))?;
        }
        Ok(Self {
            storage,
            game_assets,
            backgrounds,
            fonts,
            data_dir,
            llm_streams: Arc::new(LlmStreamRegistry::new()),
        })
    }
}
