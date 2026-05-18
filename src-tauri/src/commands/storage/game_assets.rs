use super::*;

pub(crate) fn game_assets_manifest(state: &AppState) -> AppResult<Value> {
    state.game_assets.manifest()
}

pub(crate) fn game_assets_tree(state: &AppState) -> AppResult<Value> {
    state.game_assets.tree()
}

pub(crate) fn game_assets_upload(state: &AppState, body: Value) -> AppResult<Value> {
    let category = body
        .get("category")
        .and_then(Value::as_str)
        .ok_or_else(|| AppError::invalid_input("category is required"))?;
    let subcategory = body.get("subcategory").and_then(Value::as_str);
    let file = body
        .get("file")
        .ok_or_else(|| AppError::invalid_input("file is required"))?;
    state.game_assets.write_upload(category, subcategory, file)
}
