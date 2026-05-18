use super::*;

pub(crate) fn encounter_call(rest: &[&str], body: Value) -> AppResult<Value> {
    match rest {
        ["init"] => Ok(json!({ "encounter": { "id": new_id(), "state": "active" }, "input": body })),
        ["action"] => Ok(json!({ "result": { "ok": true }, "state": "active" })),
        ["summary"] => Ok(json!({ "summary": "", "rewards": [] })),
        _ => Ok(json!({ "ok": true })),
    }
}
