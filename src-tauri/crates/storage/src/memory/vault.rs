use crate::memory::layout::{atomic_write_json, read_json_value, relative_path, MemoryLayout};
use marinara_core::{ensure_object, new_id, now_iso, AppError, AppResult};
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

const NOTE_EXTENSION: &str = "json";

pub fn list_notes(layout: &MemoryLayout, options: Option<&Value>) -> AppResult<Vec<Value>> {
    let mut notes = read_all_notes(layout)?;
    notes.retain(|note| note_matches_options(note, options));
    notes.sort_by(|a, b| {
        b.get("updatedAt")
            .and_then(Value::as_str)
            .cmp(&a.get("updatedAt").and_then(Value::as_str))
    });
    if let Some(limit) = options
        .and_then(|value| value.get("limit"))
        .and_then(Value::as_u64)
        .map(|value| value as usize)
    {
        notes.truncate(limit);
    }
    Ok(notes)
}

pub fn get_note(layout: &MemoryLayout, id: &str) -> AppResult<Option<Value>> {
    let path = note_path(layout, id)?;
    if !path.exists() {
        return Ok(None);
    }
    Ok(Some(read_note_file(&path)?))
}

pub fn create_note(layout: &MemoryLayout, value: Value) -> AppResult<Value> {
    let mut object = ensure_object(value)?;
    let id = object
        .get("id")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(new_id);
    validate_note_id(&id)?;
    let now = now_iso();
    object.insert("id".to_string(), Value::String(id.clone()));
    object
        .entry("type".to_string())
        .or_insert_with(|| Value::String("custom".to_string()));
    object.insert("status".to_string(), Value::String("active".to_string()));
    object
        .entry("modes".to_string())
        .or_insert_with(|| json!([]));
    object
        .entry("tags".to_string())
        .or_insert_with(|| json!([]));
    object
        .entry("links".to_string())
        .or_insert_with(|| json!([]));
    object
        .entry("createdAt".to_string())
        .or_insert_with(|| Value::String(now.clone()));
    object.insert("updatedAt".to_string(), Value::String(now));
    object.insert("version".to_string(), json!(1));
    object.insert("previousHash".to_string(), Value::Null);
    let note = Value::Object(object);
    validate_note_shape(&note)?;
    atomic_write_json(&note_path(layout, &id)?, &note)?;
    Ok(note)
}

pub fn update_note(layout: &MemoryLayout, id: &str, patch: Value) -> AppResult<Value> {
    let path = note_path(layout, id)?;
    if !path.exists() {
        return Err(AppError::not_found(format!(
            "memory note {id} was not found"
        )));
    }
    let mut note = read_note_file(&path)?;
    let previous_hash = hash_value(&note)?;
    let patch = ensure_object(patch)?;
    let object = note
        .as_object_mut()
        .ok_or_else(|| AppError::invalid_input("Stored memory note is not an object"))?;
    for (key, value) in patch {
        object.insert(key, value);
    }
    let next_version = object
        .get("version")
        .and_then(Value::as_u64)
        .unwrap_or(1)
        .saturating_add(1);
    object.insert("updatedAt".to_string(), Value::String(now_iso()));
    object.insert("version".to_string(), json!(next_version));
    object.insert("previousHash".to_string(), Value::String(previous_hash));
    validate_note_shape(&note)?;
    atomic_write_json(&path, &note)?;
    Ok(note)
}

pub fn archive_note(layout: &MemoryLayout, id: &str) -> AppResult<Value> {
    update_note(layout, id, json!({ "status": "archived" }))
}

pub fn read_all_notes(layout: &MemoryLayout) -> AppResult<Vec<Value>> {
    let mut paths = note_paths(layout)?;
    paths.sort();
    paths
        .iter()
        .map(|path| read_note_file(path))
        .collect::<AppResult<Vec<_>>>()
}

pub fn note_paths(layout: &MemoryLayout) -> AppResult<Vec<PathBuf>> {
    let mut paths = Vec::new();
    collect_note_paths(&layout.vault_dir(), &mut paths)?;
    Ok(paths)
}

pub fn note_relative_path(layout: &MemoryLayout, id: &str) -> AppResult<String> {
    Ok(relative_path(layout.root(), &note_path(layout, id)?))
}

pub fn validate_note_shape(note: &Value) -> AppResult<()> {
    let object = note
        .as_object()
        .ok_or_else(|| AppError::invalid_input("Memory note must be an object"))?;
    require_string(object, "id")?;
    require_enum(
        object,
        "type",
        &[
            "fact",
            "preference",
            "summary",
            "relationship",
            "world",
            "quest",
            "scene",
            "custom",
        ],
    )?;
    require_enum(object, "status", &["active", "archived"])?;
    require_string_array(object, "modes", Some(&["chat", "roleplay", "game"]))?;
    validate_scope(object.get("scope"))?;
    require_string_array(object, "tags", None)?;
    validate_links(object.get("links"))?;
    validate_sections(object.get("sections"))?;
    require_string(object, "createdAt")?;
    require_string(object, "updatedAt")?;
    if object.get("version").and_then(Value::as_u64).is_none() {
        return Err(AppError::invalid_input(
            "Memory note version must be a positive integer",
        ));
    }
    if !object
        .get("previousHash")
        .is_some_and(|value| value.is_null() || value.is_string())
    {
        return Err(AppError::invalid_input(
            "Memory note previousHash must be null or a string",
        ));
    }
    Ok(())
}

pub fn hash_value(value: &Value) -> AppResult<String> {
    Ok(hex_sha256(&serde_json::to_vec(value)?))
}

pub fn hex_sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn collect_note_paths(dir: &Path, paths: &mut Vec<PathBuf>) -> AppResult<()> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_note_paths(&path, paths)?;
        } else if path.extension().and_then(|value| value.to_str()) == Some(NOTE_EXTENSION) {
            paths.push(path);
        }
    }
    Ok(())
}

fn read_note_file(path: &Path) -> AppResult<Value> {
    let note = read_json_value(path)?;
    validate_note_shape(&note)?;
    Ok(note)
}

fn note_path(layout: &MemoryLayout, id: &str) -> AppResult<PathBuf> {
    validate_note_id(id)?;
    let prefix = id.chars().take(2).collect::<String>();
    Ok(layout
        .vault_dir()
        .join(prefix)
        .join(format!("{id}.{NOTE_EXTENSION}")))
}

fn validate_note_id(id: &str) -> AppResult<()> {
    if id.trim().is_empty() {
        return Err(AppError::invalid_input("Memory note id is required"));
    }
    let safe = id
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_'));
    if !safe {
        return Err(AppError::invalid_input(
            "Memory note id may contain only letters, numbers, hyphens, and underscores",
        ));
    }
    Ok(())
}

fn note_matches_options(note: &Value, options: Option<&Value>) -> bool {
    let include_archived = options
        .and_then(|value| value.get("includeArchived"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    if !include_archived && note.get("status").and_then(Value::as_str) == Some("archived") {
        return false;
    }
    if let Some(status) = options
        .and_then(|value| value.get("status"))
        .and_then(Value::as_str)
    {
        if note.get("status").and_then(Value::as_str) != Some(status) {
            return false;
        }
    }
    if let Some(types) = options
        .and_then(|value| value.get("types"))
        .and_then(Value::as_array)
    {
        if !string_array_contains(types, note.get("type").and_then(Value::as_str)) {
            return false;
        }
    }
    if let Some(modes) = options
        .and_then(|value| value.get("modes"))
        .and_then(Value::as_array)
    {
        let note_modes = note
            .get("modes")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        if !modes
            .iter()
            .filter_map(Value::as_str)
            .any(|mode| string_array_contains(&note_modes, Some(mode)))
        {
            return false;
        }
    }
    if let Some(tags) = options
        .and_then(|value| value.get("tags"))
        .and_then(Value::as_array)
    {
        let note_tags = note
            .get("tags")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        if !tags
            .iter()
            .filter_map(Value::as_str)
            .all(|tag| string_array_contains(&note_tags, Some(tag)))
        {
            return false;
        }
    }
    if let Some(scope) = options
        .and_then(|value| value.get("scope"))
        .and_then(Value::as_object)
    {
        let note_scope = note.get("scope").and_then(Value::as_object);
        if !matches_scope(note_scope, scope) {
            return false;
        }
    }
    true
}

fn matches_scope(note_scope: Option<&Map<String, Value>>, expected: &Map<String, Value>) -> bool {
    expected.iter().all(|(key, value)| {
        value.is_null() || note_scope.and_then(|scope| scope.get(key)) == Some(value)
    })
}

fn string_array_contains(values: &[Value], expected: Option<&str>) -> bool {
    let Some(expected) = expected else {
        return false;
    };
    values.iter().any(|value| value.as_str() == Some(expected))
}

fn require_string(object: &Map<String, Value>, key: &str) -> AppResult<()> {
    if object.get(key).and_then(Value::as_str).is_some() {
        return Ok(());
    }
    Err(AppError::invalid_input(format!(
        "Memory note {key} must be a string"
    )))
}

fn require_enum(object: &Map<String, Value>, key: &str, allowed: &[&str]) -> AppResult<()> {
    let value = object
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| AppError::invalid_input(format!("Memory note {key} must be a string")))?;
    if allowed.contains(&value) {
        return Ok(());
    }
    Err(AppError::invalid_input(format!(
        "Memory note {key} has unsupported value {value}"
    )))
}

fn require_string_array(
    object: &Map<String, Value>,
    key: &str,
    allowed: Option<&[&str]>,
) -> AppResult<()> {
    let values = object
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| AppError::invalid_input(format!("Memory note {key} must be an array")))?;
    for value in values {
        let Some(value) = value.as_str() else {
            return Err(AppError::invalid_input(format!(
                "Memory note {key} must contain only strings"
            )));
        };
        if let Some(allowed) = allowed {
            if !allowed.contains(&value) {
                return Err(AppError::invalid_input(format!(
                    "Memory note {key} contains unsupported value {value}"
                )));
            }
        }
    }
    Ok(())
}

fn validate_scope(value: Option<&Value>) -> AppResult<()> {
    let scope = value
        .and_then(Value::as_object)
        .ok_or_else(|| AppError::invalid_input("Memory note scope must be an object"))?;
    require_string(scope, "universeId")?;
    require_nullable_string(scope, "conversationId")?;
    require_nullable_string(scope, "roleplayId")?;
    require_nullable_string(scope, "gameId")?;
    require_enum(scope, "visibility", &["shared", "private", "model_only"])?;
    Ok(())
}

fn validate_links(value: Option<&Value>) -> AppResult<()> {
    let links = value
        .and_then(Value::as_array)
        .ok_or_else(|| AppError::invalid_input("Memory note links must be an array"))?;
    for link in links {
        let link = link
            .as_object()
            .ok_or_else(|| AppError::invalid_input("Memory note links must contain objects"))?;
        require_string(link, "noteId")?;
        require_string(link, "relationship")?;
    }
    Ok(())
}

fn validate_sections(value: Option<&Value>) -> AppResult<()> {
    let sections = value
        .and_then(Value::as_object)
        .ok_or_else(|| AppError::invalid_input("Memory note sections must be an object"))?;
    for (name, section) in sections {
        let section = section.as_object().ok_or_else(|| {
            AppError::invalid_input(format!("Memory note section {name} must be an object"))
        })?;
        require_string(section, "text")?;
        require_unit_number(section, "confidence")?;
        require_unit_number(section, "salience")?;
        if section.contains_key("updatedAt") {
            require_string(section, "updatedAt")?;
        }
        if section.contains_key("visibility") {
            require_enum(section, "visibility", &["shared", "private", "model_only"])?;
        }
        if section.contains_key("gates") {
            require_string_array(section, "gates", None)?;
        }
        if section.contains_key("evidence") {
            require_string_array(section, "evidence", None)?;
        }
    }
    Ok(())
}

fn require_nullable_string(object: &Map<String, Value>, key: &str) -> AppResult<()> {
    if object
        .get(key)
        .is_some_and(|value| value.is_null() || value.is_string())
    {
        return Ok(());
    }
    Err(AppError::invalid_input(format!(
        "Memory note {key} must be null or a string"
    )))
}

fn require_unit_number(object: &Map<String, Value>, key: &str) -> AppResult<()> {
    let value = object
        .get(key)
        .and_then(Value::as_f64)
        .ok_or_else(|| AppError::invalid_input(format!("Memory note {key} must be a number")))?;
    if (0.0..=1.0).contains(&value) {
        return Ok(());
    }
    Err(AppError::invalid_input(format!(
        "Memory note {key} must be between 0 and 1"
    )))
}

pub fn filter_note_ids(notes: &[Value], requested: Option<&HashSet<String>>) -> Vec<String> {
    notes
        .iter()
        .filter_map(|note| note.get("id").and_then(Value::as_str))
        .filter(|id| match requested {
            Some(requested) => requested.contains(*id),
            None => true,
        })
        .map(ToOwned::to_owned)
        .collect()
}
