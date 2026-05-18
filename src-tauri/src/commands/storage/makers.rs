use super::*;

pub(crate) async fn prompt_reviewer_events(state: &AppState, body: Value) -> AppResult<Vec<Value>> {
    let preset_id = body.get("presetId").and_then(Value::as_str).unwrap_or("");
    let preset = if preset_id.is_empty() {
        Value::Null
    } else {
        state.storage.get("prompts", preset_id)?.unwrap_or(Value::Null)
    };
    let focus_areas = body
        .get("focusAreas")
        .and_then(Value::as_array)
        .map(|items| items.iter().filter_map(Value::as_str).collect::<Vec<_>>())
        .unwrap_or_else(|| vec!["clarity", "consistency", "coverage"]);
    let name = preset.get("name").and_then(Value::as_str).unwrap_or("Prompt preset");
    let description = preset.get("description").and_then(Value::as_str).unwrap_or("");
    let summary = format!(
        "{name} was reviewed locally for {}. {}",
        focus_areas.join(", "),
        if description.is_empty() {
            "No preset description is set."
        } else {
            "The preset includes a description."
        }
    );
    let review = json!({
        "overall_score": 7,
        "summary": summary,
        "sections": focus_areas.into_iter().map(|area| json!({
            "area": area,
            "score": 7,
            "findings": "Local structural review completed without sending prompt data to an external model.",
            "suggestions": ["Use a configured language-model connection for deeper semantic critique.", "Review section ordering and token-heavy examples before production use."]
        })).collect::<Vec<_>>(),
        "token_estimate": description.len() / 4,
        "warnings": [],
        "best_practices": ["Preset can be inspected locally through the Tauri storage layer."]
    });
    Ok(vec![
        json!({ "type": "token", "data": review["summary"].as_str().unwrap_or("") }),
        json!({ "type": "done", "data": review.to_string() }),
    ])
}

pub(crate) async fn maker_events(kind: &str) -> AppResult<Vec<Value>> {
    let payload = if kind == "persona" {
        json!({ "name": "Generated Persona", "description": "", "personality": "", "scenario": "" })
    } else {
        json!({ "name": "Generated Character", "description": "", "personality": "", "scenario": "", "first_mes": "" })
    };
    Ok(vec![
        json!({ "type": "token", "data": payload.to_string() }),
        json!({ "type": "done", "data": payload.to_string() }),
    ])
}
