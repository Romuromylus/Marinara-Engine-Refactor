use super::*;

pub(crate) use marinara_handlers::shared::{
    decode_uploaded_file, normalize_character_data_for_storage, required_string,
    string_array_from_value, with_entity_defaults,
};

pub(crate) struct ParsedPath {
    pub(crate) parts: Vec<String>,
    pub(crate) query: HashMap<String, String>,
}

impl ParsedPath {
    pub(crate) fn new(path: &str) -> Self {
        let (path_part, query_part) = path.split_once('?').unwrap_or((path, ""));
        let parts = path_part
            .trim_matches('/')
            .split('/')
            .filter(|part| !part.is_empty())
            .map(|part| part.to_string())
            .collect();
        let query = query_part
            .split('&')
            .filter_map(|pair| {
                let (key, value) = pair.split_once('=')?;
                Some((key.to_string(), value.to_string()))
            })
            .collect();
        Self { parts, query }
    }
}

pub(crate) fn list_collection(
    state: &AppState,
    collection: &str,
    filter: Option<(&str, &str)>,
) -> AppResult<Value> {
    marinara_handlers::shared::list_collection(&state.storage, collection, filter)
}

pub(crate) fn get_required(state: &AppState, collection: &str, id: &str) -> AppResult<Value> {
    marinara_handlers::shared::get_required(&state.storage, collection, id)
}

pub(crate) fn find_by_field(
    state: &AppState,
    collection: &str,
    field: &str,
    value: &str,
) -> AppResult<Option<Value>> {
    marinara_handlers::shared::find_by_field(&state.storage, collection, field, value)
}

pub(crate) fn upload_gallery_image(
    state: &AppState,
    collection: &str,
    parent_field: &str,
    parent_id: &str,
    body: Value,
) -> AppResult<Value> {
    marinara_handlers::shared::upload_gallery_image(
        &state.storage,
        collection,
        parent_field,
        parent_id,
        body,
    )
}
