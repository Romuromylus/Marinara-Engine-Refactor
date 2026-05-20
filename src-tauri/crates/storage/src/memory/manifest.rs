use crate::memory::layout::{atomic_write_json, read_json_object, relative_path, MemoryLayout};
use crate::memory::vault::hex_sha256;
use crate::memory::MEMORY_VERSION;
use marinara_core::{now_iso, AppResult};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

pub fn write_manifest(layout: &MemoryLayout, embedding_model: Option<Value>) -> AppResult<Value> {
    let files = persisted_file_entries(layout)?;
    let vault_hash = vault_hash(&files)?;
    let embedding_model =
        embedding_model.unwrap_or_else(|| existing_embedding_model(layout).unwrap_or(Value::Null));
    let manifest = json!({
        "version": MEMORY_VERSION,
        "embeddingModel": embedding_model,
        "generatedAt": now_iso(),
        "vaultHash": vault_hash,
        "files": files
    });
    atomic_write_json(&layout.manifest_path(), &manifest)?;
    Ok(manifest)
}

pub fn persisted_file_entries(layout: &MemoryLayout) -> AppResult<Vec<Value>> {
    let mut paths = Vec::new();
    collect_files(&layout.vault_dir(), &mut paths)?;
    if layout.events_path().exists() {
        paths.push(layout.events_path());
    }
    paths.sort();
    paths
        .iter()
        .map(|path| file_entry(layout.root(), path))
        .collect::<AppResult<Vec<_>>>()
}

pub fn stored_manifest(layout: &MemoryLayout) -> AppResult<Option<Value>> {
    if !layout.manifest_path().exists() {
        return Ok(None);
    }
    Ok(Some(crate::memory::layout::read_json_value(
        &layout.manifest_path(),
    )?))
}

pub fn computed_vault_hash(layout: &MemoryLayout) -> AppResult<String> {
    vault_hash(&persisted_file_entries(layout)?)
}

fn existing_embedding_model(layout: &MemoryLayout) -> AppResult<Value> {
    Ok(read_json_object(&layout.manifest_path())?
        .get("embeddingModel")
        .cloned()
        .unwrap_or(Value::Null))
}

fn collect_files(dir: &Path, paths: &mut Vec<PathBuf>) -> AppResult<()> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, paths)?;
        } else {
            paths.push(path);
        }
    }
    Ok(())
}

fn file_entry(root: &Path, path: &Path) -> AppResult<Value> {
    let bytes = fs::read(path)?;
    let updated_at = fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .ok()
        .map(system_time_iso)
        .unwrap_or_else(now_iso);
    Ok(json!({
        "path": relative_path(root, path),
        "hash": hex_sha256(&bytes),
        "size": bytes.len(),
        "updatedAt": updated_at
    }))
}

fn vault_hash(files: &[Value]) -> AppResult<String> {
    let bytes = serde_json::to_vec(files)?;
    let digest = Sha256::digest(bytes);
    Ok(digest.iter().map(|byte| format!("{byte:02x}")).collect())
}

fn system_time_iso(time: std::time::SystemTime) -> String {
    let datetime: chrono::DateTime<chrono::Utc> = time.into();
    datetime.to_rfc3339()
}
