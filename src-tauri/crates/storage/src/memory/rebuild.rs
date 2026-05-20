use crate::memory::layout::MemoryLayout;
use crate::memory::{events, layout::atomic_write_json, manifest, vault};
use marinara_core::AppResult;
use serde_json::{json, Value};
use std::collections::HashSet;

pub fn rebuild_indexes(layout: &MemoryLayout, request: Option<&Value>) -> AppResult<Value> {
    let embedding_model = request
        .and_then(|value| value.get("embeddingModel"))
        .cloned()
        .filter(|value| !value.is_null());
    let scope_options = request
        .and_then(|value| value.get("scope"))
        .map(|scope| json!({ "scope": scope }));
    let requested_note_ids = request
        .and_then(|value| value.get("noteIds"))
        .and_then(Value::as_array)
        .map(|ids| {
            ids.iter()
                .filter_map(Value::as_str)
                .map(ToOwned::to_owned)
                .collect::<HashSet<_>>()
        });
    let notes = vault::list_notes(layout, scope_options.as_ref())?;
    let reindexed_note_ids = vault::filter_note_ids(&notes, requested_note_ids.as_ref());
    let manifest = manifest::write_manifest(layout, embedding_model)?;
    let result = json!({
        "noteCount": reindexed_note_ids.len(),
        "eventCount": events::read_events(layout)?.len(),
        "reindexedNoteIds": reindexed_note_ids,
        "removedNoteIds": [],
        "manifest": manifest,
        "warnings": []
    });
    atomic_write_json(&layout.rebuild_path(), &result)?;
    Ok(result)
}
