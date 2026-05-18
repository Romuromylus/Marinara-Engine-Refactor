use crate::state::AppState;
use marinara_core::AppResult;
use tauri::AppHandle;

pub fn build_state(app: &AppHandle) -> AppResult<AppState> {
    AppState::new(app)
}
