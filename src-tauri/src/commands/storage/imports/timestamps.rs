use serde_json::{json, Value};

fn parse_trusted_timestamp(value: Option<&Value>) -> Option<String> {
    match value? {
        Value::Number(number) => {
            let raw = number.as_f64()?;
            if !raw.is_finite() {
                return None;
            }
            let millis = if raw < 1_000_000_000_000.0 {
                raw * 1000.0
            } else {
                raw
            };
            chrono::DateTime::<chrono::Utc>::from_timestamp_millis(millis.round() as i64)
                .map(|time| time.to_rfc3339())
        }
        Value::String(raw) => {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                return None;
            }
            if trimmed.chars().all(|ch| ch.is_ascii_digit()) && trimmed.len() >= 10 {
                if let Ok(number) = trimmed.parse::<f64>() {
                    let millis = if trimmed.len() <= 10 {
                        number * 1000.0
                    } else {
                        number
                    };
                    return chrono::DateTime::<chrono::Utc>::from_timestamp_millis(
                        millis.round() as i64
                    )
                    .map(|time| time.to_rfc3339());
                }
            }
            chrono::DateTime::parse_from_rfc3339(trimmed)
                .map(|time| time.with_timezone(&chrono::Utc).to_rfc3339())
                .ok()
                .or_else(|| Some(trimmed.to_string()))
        }
        _ => None,
    }
}

pub(super) fn timestamp_overrides_from_value(value: Option<&Value>) -> Option<(String, String)> {
    let value = value?;
    match value {
        Value::String(raw) => {
            if let Ok(parsed) = serde_json::from_str::<Value>(raw) {
                timestamp_overrides_from_value(Some(&parsed))
            } else {
                parse_trusted_timestamp(Some(value)).map(|timestamp| (timestamp.clone(), timestamp))
            }
        }
        Value::Object(object) => {
            let created = parse_trusted_timestamp(object.get("createdAt"));
            let updated = parse_trusted_timestamp(object.get("updatedAt"));
            match (created, updated) {
                (Some(created), Some(updated)) => Some((created, updated)),
                (Some(created), None) => Some((created.clone(), created)),
                (None, Some(updated)) => Some((updated.clone(), updated)),
                (None, None) => None,
            }
        }
        _ => parse_trusted_timestamp(Some(value)).map(|timestamp| (timestamp.clone(), timestamp)),
    }
}

fn timestamp_overrides_from_body_and_payload(
    body: &Value,
    payload: &Value,
) -> Option<(String, String)> {
    timestamp_overrides_from_value(
        body.get("timestampOverrides")
            .or_else(|| body.get("__timestampOverrides")),
    )
    .or_else(|| {
        let created = body.get("createdAt");
        let updated = body.get("updatedAt");
        timestamp_overrides_from_value(Some(&json!({
            "createdAt": created.cloned().unwrap_or(Value::Null),
            "updatedAt": updated.cloned().unwrap_or(Value::Null)
        })))
    })
    .or_else(|| {
        payload
            .get("metadata")
            .and_then(|metadata| metadata.get("timestamps"))
            .and_then(|timestamps| timestamp_overrides_from_value(Some(timestamps)))
    })
}

pub(super) fn apply_timestamp_overrides(record: &mut Value, body: &Value, payload: &Value) {
    let Some((created_at, updated_at)) = timestamp_overrides_from_body_and_payload(body, payload)
    else {
        return;
    };
    if let Some(object) = record.as_object_mut() {
        object.insert("createdAt".to_string(), Value::String(created_at));
        object.insert("updatedAt".to_string(), Value::String(updated_at));
    }
}
