use marinara_core::{AppError, AppResult};
use serde_json::Value;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};

pub mod events;
pub mod layout;
pub mod manifest;
pub mod rebuild;
pub mod validate;
pub mod vault;

pub const MEMORY_VERSION: &str = "1";

#[derive(Debug, Clone)]
pub struct MemoryStore {
    layout: layout::MemoryLayout,
    lock: Arc<Mutex<()>>,
}

impl MemoryStore {
    pub fn new(data_dir: impl Into<PathBuf>) -> Self {
        Self {
            layout: layout::MemoryLayout::new(data_dir),
            lock: Arc::new(Mutex::new(())),
        }
    }

    pub fn ensure_layout(&self) -> AppResult<Value> {
        let _guard = self.lock()?;
        layout::ensure_layout(&self.layout)?;
        if !self.layout.manifest_path().exists() {
            manifest::write_manifest(&self.layout, None)?;
        }
        Ok(self.layout.info())
    }

    pub fn list_notes(&self, options: Option<Value>) -> AppResult<Vec<Value>> {
        let _guard = self.lock()?;
        layout::ensure_layout(&self.layout)?;
        vault::list_notes(&self.layout, options.as_ref())
    }

    pub fn get_note(&self, id: &str) -> AppResult<Option<Value>> {
        let _guard = self.lock()?;
        layout::ensure_layout(&self.layout)?;
        vault::get_note(&self.layout, id)
    }

    pub fn create_note(&self, value: Value) -> AppResult<Value> {
        let _guard = self.lock()?;
        layout::ensure_layout(&self.layout)?;
        let note = vault::create_note(&self.layout, value)?;
        manifest::write_manifest(&self.layout, None)?;
        Ok(note)
    }

    pub fn update_note(&self, id: &str, patch: Value) -> AppResult<Value> {
        let _guard = self.lock()?;
        layout::ensure_layout(&self.layout)?;
        let note = vault::update_note(&self.layout, id, patch)?;
        manifest::write_manifest(&self.layout, None)?;
        Ok(note)
    }

    pub fn archive_note(&self, id: &str) -> AppResult<Value> {
        let _guard = self.lock()?;
        layout::ensure_layout(&self.layout)?;
        let note = vault::archive_note(&self.layout, id)?;
        manifest::write_manifest(&self.layout, None)?;
        Ok(note)
    }

    pub fn append_event(&self, value: Value) -> AppResult<Value> {
        let _guard = self.lock()?;
        layout::ensure_layout(&self.layout)?;
        let event = events::append_event(&self.layout, value)?;
        manifest::write_manifest(&self.layout, None)?;
        Ok(event)
    }

    pub fn manifest(&self) -> AppResult<Value> {
        let _guard = self.lock()?;
        layout::ensure_layout(&self.layout)?;
        manifest::write_manifest(&self.layout, None)
    }

    pub fn validate_vault(&self) -> AppResult<Value> {
        let _guard = self.lock()?;
        layout::ensure_layout(&self.layout)?;
        validate::validate_vault(&self.layout)
    }

    pub fn rebuild_indexes(&self, request: Option<Value>) -> AppResult<Value> {
        let _guard = self.lock()?;
        layout::ensure_layout(&self.layout)?;
        rebuild::rebuild_indexes(&self.layout, request.as_ref())
    }

    fn lock(&self) -> AppResult<MutexGuard<'_, ()>> {
        self.lock
            .lock()
            .map_err(|_| AppError::new("lock_error", "Memory storage lock poisoned"))
    }
}
