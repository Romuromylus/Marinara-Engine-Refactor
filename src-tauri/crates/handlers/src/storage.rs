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

pub fn materialize_message_swipe_fields(message: &mut Value) {
    let Some(object) = message.as_object_mut() else {
        return;
    };
    let Some((swipe_count, active_index, active_content)) = object
        .get("swipes")
        .and_then(Value::as_array)
        .map(|swipes| {
            let swipe_count = swipes.len();
            if swipe_count == 0 {
                return (0, 0, None);
            }

            let requested_index = object
                .get("activeSwipeIndex")
                .and_then(Value::as_u64)
                .map(|value| value as usize)
                .unwrap_or(0);
            let active_index = requested_index.min(swipe_count.saturating_sub(1));
            let active_content = swipes
                .get(active_index)
                .and_then(|swipe| swipe.get("content"))
                .and_then(Value::as_str)
                .map(ToOwned::to_owned);
            (swipe_count, active_index, active_content)
        })
    else {
        return;
    };
    object.insert("swipeCount".to_string(), json!(swipe_count));
    if swipe_count == 0 {
        object.insert("activeSwipeIndex".to_string(), json!(0));
        return;
    }

    object.insert("activeSwipeIndex".to_string(), json!(active_index));
    if let Some(content) = active_content {
        object.insert("content".to_string(), Value::String(content));
    }
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
        let result = list(
            &storage,
            "characters",
            Some(json!({ "descending": true })),
        )
        .expect("list");
        let names: Vec<&str> = result
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|row| row.get("name").and_then(Value::as_str))
            .collect();
        assert_eq!(names, vec!["Bravo", "Alpha"]);
    }
}
