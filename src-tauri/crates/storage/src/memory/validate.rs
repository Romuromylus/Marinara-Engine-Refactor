use crate::memory::layout::MemoryLayout;
use crate::memory::{events, layout::atomic_write_json, layout::relative_path, manifest, vault};
use marinara_core::AppResult;
use serde_json::{json, Value};

pub fn validate_vault(layout: &MemoryLayout) -> AppResult<Value> {
    let mut issues = Vec::new();
    let mut note_count = 0usize;
    for path in vault::note_paths(layout)? {
        match crate::memory::layout::read_json_value(&path).and_then(|note| {
            vault::validate_note_shape(&note)?;
            Ok(note)
        }) {
            Ok(_) => note_count += 1,
            Err(error) => issues.push(issue(
                relative_path(layout.root(), &path),
                error.to_string(),
                "error",
            )),
        }
    }
    let event_count = match events::read_events(layout) {
        Ok(events) => events.len(),
        Err(error) => {
            issues.push(issue("events/log.jsonl", error.to_string(), "error"));
            0
        }
    };
    let file_entries = manifest::persisted_file_entries(layout)?;
    let computed_hash = manifest::computed_vault_hash(layout)?;
    let stale_indexes = match manifest::stored_manifest(layout)? {
        Some(Value::Object(manifest)) => {
            if manifest.get("vaultHash").and_then(Value::as_str) == Some(computed_hash.as_str()) {
                Vec::new()
            } else {
                vec!["indexes/manifest.json".to_string()]
            }
        }
        Some(_) => {
            issues.push(issue(
                "indexes/manifest.json",
                "Memory manifest is not a JSON object",
                "error",
            ));
            vec!["indexes/manifest.json".to_string()]
        }
        None => vec!["indexes/manifest.json".to_string()],
    };
    let report = json!({
        "ok": issues.iter().all(|issue| issue.get("severity").and_then(Value::as_str) != Some("error")),
        "issues": issues,
        "staleIndexes": stale_indexes,
        "counts": {
            "notes": note_count,
            "events": event_count,
            "files": file_entries.len()
        }
    });
    atomic_write_json(&layout.validation_path(), &report)?;
    Ok(report)
}

fn issue(path: impl Into<String>, message: impl Into<String>, severity: &str) -> Value {
    json!({
        "path": path.into(),
        "message": message.into(),
        "severity": severity
    })
}
