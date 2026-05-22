// Thin Tauri-side dispatcher over marinara_handlers::backgrounds. All filesystem
// + metadata logic lives in the handlers crate so the Axum server target shares
// the same code path.
use super::*;

pub(crate) fn backgrounds_call(
    state: &AppState,
    method: &str,
    rest: &[&str],
    body: Value,
) -> AppResult<Value> {
    let storage = &state.storage;
    let assets = &state.backgrounds;
    match (method, rest) {
        ("GET", []) => marinara_handlers::backgrounds::list(storage, assets),
        ("GET", ["tags"]) => marinara_handlers::backgrounds::tags(storage),
        ("GET", ["file-path", encoded]) => {
            marinara_handlers::backgrounds::file_path(assets, encoded)
        }
        ("POST", ["upload"]) => marinara_handlers::backgrounds::upload(storage, assets, body),
        ("PATCH", [id, "rename"]) => {
            marinara_handlers::backgrounds::rename(storage, assets, id, body)
        }
        ("PATCH", [id, "tags"]) => marinara_handlers::backgrounds::update_tags(storage, id, body),
        ("DELETE", [id]) => marinara_handlers::backgrounds::delete(storage, assets, id),
        _ => Err(AppError::new(
            "route_not_found",
            format!("Unknown backgrounds route: {method} /{}", rest.join("/")),
        )),
    }
}
