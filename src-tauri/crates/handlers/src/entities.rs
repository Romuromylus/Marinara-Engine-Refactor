use crate::shared::{
    duplicate_record, materialize_message_swipe_fields, normalize_update_patch,
    with_entity_defaults,
};
use marinara_core::AppError;
use marinara_storage::FileStorage;
use serde_json::{json, Value};

pub fn list(
    storage: &FileStorage,
    entity: &str,
    options: Option<Value>,
) -> Result<Value, AppError> {
    let mut rows = match options
        .as_ref()
        .and_then(|value| value.get("filters"))
        .and_then(Value::as_object)
    {
        Some(filters) if !filters.is_empty() => storage.list_where(entity, filters)?,
        _ => storage.list(entity)?,
    };

    let order_by = options
        .as_ref()
        .and_then(|value| value.get("orderBy"))
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty());
    let descending = options
        .as_ref()
        .and_then(|value| value.get("descending"))
        .and_then(Value::as_bool)
        .unwrap_or(false);

    rows.sort_by(|a, b| {
        let ordering = match order_by {
            Some(field) => compare_json_values(a.get(field), b.get(field)),
            None => compare_json_values(
                a.get("sortOrder")
                    .or_else(|| a.get("order"))
                    .or_else(|| a.get("createdAt")),
                b.get("sortOrder")
                    .or_else(|| b.get("order"))
                    .or_else(|| b.get("createdAt")),
            ),
        };
        if descending {
            ordering.reverse()
        } else {
            ordering
        }
    });

    if entity == "messages" {
        apply_message_pagination(&mut rows, options.as_ref());
        for row in &mut rows {
            materialize_message_swipe_fields(row);
        }
        return Ok(Value::Array(rows));
    }

    if let Some(limit) = options
        .as_ref()
        .and_then(|value| value.get("limit"))
        .and_then(Value::as_u64)
        .map(|value| value as usize)
    {
        rows.truncate(limit);
    }

    Ok(Value::Array(rows))
}

pub fn get(storage: &FileStorage, entity: &str, id: &str) -> Result<Value, AppError> {
    let mut value = storage.get(entity, id)?.unwrap_or(Value::Null);
    if entity == "messages" {
        materialize_message_swipe_fields(&mut value);
    }
    Ok(value)
}

pub fn create(storage: &FileStorage, entity: &str, value: Value) -> Result<Value, AppError> {
    storage.create(entity, with_entity_defaults(entity, value))
}

pub fn update(
    storage: &FileStorage,
    entity: &str,
    id: &str,
    patch: Value,
) -> Result<Value, AppError> {
    storage.patch(entity, id, normalize_update_patch(entity, patch)?)
}

pub fn delete(storage: &FileStorage, entity: &str, id: &str) -> Result<Value, AppError> {
    if crate::is_protected_record(entity, id) {
        return Err(AppError::invalid_input(
            "Built-in Professor Mari cannot be deleted",
        ));
    }
    let deleted = storage.delete(entity, id)?;
    Ok(json!({ "deleted": deleted }))
}

pub fn duplicate(storage: &FileStorage, entity: &str, id: &str) -> Result<Value, AppError> {
    duplicate_record(storage, entity, id)
}

fn compare_json_values(left: Option<&Value>, right: Option<&Value>) -> std::cmp::Ordering {
    match (left, right) {
        (Some(Value::Number(a)), Some(Value::Number(b))) => a
            .as_f64()
            .partial_cmp(&b.as_f64())
            .unwrap_or(std::cmp::Ordering::Equal),
        (Some(Value::String(a)), Some(Value::String(b))) => a.cmp(b),
        (Some(Value::Bool(a)), Some(Value::Bool(b))) => a.cmp(b),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        _ => std::cmp::Ordering::Equal,
    }
}

fn apply_message_pagination(rows: &mut Vec<Value>, options: Option<&Value>) {
    rows.sort_by(|a, b| {
        let (a_created_at, a_id) = message_cursor(a);
        let (b_created_at, b_id) = message_cursor(b);
        a_created_at.cmp(b_created_at).then_with(|| a_id.cmp(b_id))
    });

    let before = options
        .and_then(|value| value.get("before"))
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(parse_message_cursor);

    if let Some((before_created_at, before_id)) = before {
        rows.retain(|row| {
            let (created_at, id) = message_cursor(row);
            created_at < before_created_at.as_str()
                || (created_at == before_created_at.as_str()
                    && before_id.as_deref().is_some_and(|cursor_id| id < cursor_id))
        });
    }

    let Some(limit) = options
        .and_then(|value| value.get("limit"))
        .and_then(Value::as_u64)
        .map(|value| value as usize)
    else {
        return;
    };

    if rows.len() > limit {
        let keep_from = rows.len() - limit;
        rows.drain(0..keep_from);
    }
}

fn parse_message_cursor(cursor: &str) -> (String, Option<String>) {
    let mut parts = cursor.splitn(2, '|');
    let created_at = parts.next().unwrap_or_default().to_string();
    let id = parts
        .next()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);
    (created_at, id)
}

fn message_cursor(row: &Value) -> (&str, &str) {
    (
        row.get("createdAt").and_then(Value::as_str).unwrap_or(""),
        row.get("id").and_then(Value::as_str).unwrap_or(""),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use marinara_storage::FileStorage;
    use serde_json::json;
    use tempfile::TempDir;

    fn seed_storage() -> (TempDir, FileStorage) {
        let temp = TempDir::new().expect("tempdir");
        let storage = FileStorage::new(temp.path().join("data")).expect("storage");
        storage
            .create("characters", json!({ "name": "Bravo", "sortOrder": 2 }))
            .expect("seed bravo");
        storage
            .create("characters", json!({ "name": "Alpha", "sortOrder": 1 }))
            .expect("seed alpha");
        (temp, storage)
    }

    #[test]
    fn list_sorts_by_sort_order_then_created_at() {
        let (_temp, storage) = seed_storage();
        let result = list(&storage, "characters", None).expect("list");
        let names: Vec<&str> = result
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|row| row.get("name").and_then(Value::as_str))
            .collect();
        assert_eq!(names, vec!["Alpha", "Bravo"]);
    }

    #[test]
    fn list_descending_reverses_order() {
        let (_temp, storage) = seed_storage();
        let result = list(&storage, "characters", Some(json!({ "descending": true })))
            .expect("list");
        let names: Vec<&str> = result
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|row| row.get("name").and_then(Value::as_str))
            .collect();
        assert_eq!(names, vec!["Bravo", "Alpha"]);
    }

    #[test]
    fn create_applies_chat_defaults() {
        let temp = TempDir::new().expect("tempdir");
        let storage = FileStorage::new(temp.path().join("data")).expect("storage");
        let created = create(&storage, "chats", json!({ "name": "Test" })).expect("create");
        assert!(created.get("metadata").is_some());
        assert!(created.get("gameState").is_some());
        assert_eq!(created["characterIds"], json!([]));
    }

    #[test]
    fn duplicate_appends_copy_suffix() {
        let temp = TempDir::new().expect("tempdir");
        let storage = FileStorage::new(temp.path().join("data")).expect("storage");
        let original = create(&storage, "personas", json!({ "name": "Mari" })).expect("create");
        let original_id = original["id"].as_str().expect("id").to_string();
        let copy = duplicate(&storage, "personas", &original_id).expect("duplicate");
        assert_eq!(copy["name"], "Mari Copy");
        assert_ne!(copy["id"], original["id"]);
    }
}
