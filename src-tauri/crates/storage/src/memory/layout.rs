use marinara_core::AppResult;
use serde::Serialize;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

pub const MEMORY_DIR: &str = "tori-memory";
pub const VAULT_DIR: &str = "vault";
pub const EVENTS_DIR: &str = "events";
pub const EVENTS_LOG: &str = "log.jsonl";
pub const INDEXES_DIR: &str = "indexes";
pub const USAGE_DIR: &str = "usage";
pub const CONFIG_DIR: &str = "config";
pub const MANIFEST_FILE: &str = "manifest.json";
pub const VALIDATION_FILE: &str = "validation.json";
pub const REBUILD_FILE: &str = "rebuild.json";

#[derive(Debug, Clone)]
pub struct MemoryLayout {
    root: PathBuf,
}

impl MemoryLayout {
    pub fn new(data_dir: impl Into<PathBuf>) -> Self {
        Self {
            root: data_dir.into().join(MEMORY_DIR),
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn vault_dir(&self) -> PathBuf {
        self.root.join(VAULT_DIR)
    }

    pub fn events_dir(&self) -> PathBuf {
        self.root.join(EVENTS_DIR)
    }

    pub fn events_path(&self) -> PathBuf {
        self.events_dir().join(EVENTS_LOG)
    }

    pub fn indexes_dir(&self) -> PathBuf {
        self.root.join(INDEXES_DIR)
    }

    pub fn usage_dir(&self) -> PathBuf {
        self.root.join(USAGE_DIR)
    }

    pub fn config_dir(&self) -> PathBuf {
        self.root.join(CONFIG_DIR)
    }

    pub fn manifest_path(&self) -> PathBuf {
        self.indexes_dir().join(MANIFEST_FILE)
    }

    pub fn validation_path(&self) -> PathBuf {
        self.indexes_dir().join(VALIDATION_FILE)
    }

    pub fn rebuild_path(&self) -> PathBuf {
        self.indexes_dir().join(REBUILD_FILE)
    }

    pub fn info(&self) -> Value {
        json!({
            "root": path_string(self.root()),
            "vaultDir": path_string(&self.vault_dir()),
            "eventsPath": path_string(&self.events_path()),
            "indexesDir": path_string(&self.indexes_dir()),
            "usageDir": path_string(&self.usage_dir()),
            "configDir": path_string(&self.config_dir())
        })
    }
}

pub fn ensure_layout(layout: &MemoryLayout) -> AppResult<()> {
    fs::create_dir_all(layout.vault_dir())?;
    fs::create_dir_all(layout.events_dir())?;
    fs::create_dir_all(layout.indexes_dir())?;
    fs::create_dir_all(layout.usage_dir())?;
    fs::create_dir_all(layout.config_dir())?;
    if !layout.events_path().exists() {
        atomic_write_bytes(&layout.events_path(), b"")?;
    }
    Ok(())
}

pub fn read_json_value(path: &Path) -> AppResult<Value> {
    let raw = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&raw)?)
}

pub fn read_json_object(path: &Path) -> AppResult<serde_json::Map<String, Value>> {
    if !path.exists() {
        return Ok(serde_json::Map::new());
    }
    match read_json_value(path)? {
        Value::Object(value) => Ok(value),
        _ => Err(marinara_core::AppError::invalid_input(format!(
            "{} did not contain a JSON object",
            path.display()
        ))),
    }
}

pub fn atomic_write_json<T>(path: &Path, value: &T) -> AppResult<()>
where
    T: Serialize + ?Sized,
{
    let bytes = serde_json::to_vec_pretty(value)?;
    atomic_write_bytes(path, &bytes)
}

pub fn atomic_write_bytes(path: &Path, bytes: &[u8]) -> AppResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = temp_path(path);
    fs::write(&tmp, bytes)?;
    fs::rename(tmp, path)?;
    Ok(())
}

pub fn path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

pub fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn temp_path(path: &Path) -> PathBuf {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| format!("{value}.tmp"))
        .unwrap_or_else(|| "tmp".to_string());
    path.with_extension(extension)
}
