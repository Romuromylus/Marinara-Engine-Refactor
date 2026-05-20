use crate::memory::layout::MemoryLayout;
use marinara_core::{ensure_object, now_iso, AppResult};
use serde_json::Value;
use std::fs;
use std::io::Write;
use std::path::Path;

pub fn append_event(layout: &MemoryLayout, value: Value) -> AppResult<Value> {
    let mut object = ensure_object(value)?;
    object
        .entry("ts".to_string())
        .or_insert_with(|| Value::String(now_iso()));
    let event = Value::Object(object);
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(layout.events_path())?;
    let mut line = serde_json::to_vec(&event)?;
    line.push(b'\n');
    file.write_all(&line)?;
    file.sync_all()?;
    Ok(event)
}

pub fn read_events(layout: &MemoryLayout) -> AppResult<Vec<Value>> {
    read_event_file(&layout.events_path())
}

pub fn read_event_file(path: &Path) -> AppResult<Vec<Value>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(path)?;
    raw.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).map_err(Into::into))
        .collect()
}
