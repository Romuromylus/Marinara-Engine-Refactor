// Thin Tauri-side dispatcher over marinara_handlers::fonts. All filesystem +
// Google Fonts download logic lives in the handlers crate so the Axum server
// target shares the same code path. The one thing that stays Tauri-only is
// actually popping the system file manager — handler::open_folder returns the
// resolved path, and the `fonts_open_folder` shim in
// `src/commands/storage/commands/assets.rs` layers `tauri_plugin_opener` on
// top, mirroring how `game_assets_open_folder` works.
use super::*;

pub(crate) async fn fonts_call(
    state: &AppState,
    method: &str,
    rest: &[&str],
    body: Value,
) -> AppResult<Value> {
    let assets = &state.fonts;
    match (method, rest) {
        ("GET", []) => marinara_handlers::fonts::list(assets),
        ("GET", ["file", filename]) => marinara_handlers::fonts::file(assets, filename),
        ("POST", ["open-folder"]) => marinara_handlers::fonts::open_folder(assets),
        ("POST", ["google", "download"]) => {
            marinara_handlers::fonts::download_google(assets, body).await
        }
        _ => Err(AppError::new(
            "route_not_found",
            format!("fonts route {method} /{} was not found", rest.join("/")),
        )),
    }
}
