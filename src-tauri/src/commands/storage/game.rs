use super::*;
use super::shared::*;
use super::chats::patch_chat_object_field;

pub(crate) fn default_game_map() -> Value {
    json!({
        "id": new_id(),
        "type": "grid",
        "name": "Starting Area",
        "description": "",
        "width": 3,
        "height": 3,
        "cells": [],
        "partyPosition": { "x": 1, "y": 1 }
    })
}

pub(crate) fn game_summary(session_number: i64) -> Value {
    json!({
        "sessionNumber": session_number,
        "summary": "",
        "resumePoint": "",
        "partyDynamics": "",
        "partyState": "",
        "keyDiscoveries": [],
        "characterMoments": [],
        "littleDetails": [],
        "statsSnapshot": {},
        "npcUpdates": [],
        "nextSessionRequest": Value::Null,
        "timestamp": now_iso()
    })
}

pub(crate) fn chat_metadata_patch(state: &AppState, chat_id: &str, patch: Value) -> AppResult<Value> {
    patch_chat_object_field(state, chat_id, "metadata", patch)
}

pub(crate) fn chats_for_game(state: &AppState, game_id: &str) -> AppResult<Value> {
    let rows = state
        .storage
        .list("chats")?
        .into_iter()
        .filter(|chat| metadata_map(chat).get("gameId").and_then(Value::as_str) == Some(game_id))
        .collect::<Vec<_>>();
    Ok(Value::Array(rows))
}

pub(crate) fn dice_result(notation: &str) -> Value {
    let sides = notation
        .split('d')
        .nth(1)
        .and_then(|tail| tail.split(|ch| ch == '+' || ch == '-').next())
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(20)
        .max(1);
    let roll = ((now_millis() % sides as u128) + 1) as i64;
    json!({ "notation": notation, "rolls": [roll], "modifier": 0, "total": roll })
}

pub(crate) fn game_call(state: &AppState, method: &str, rest: &[&str], body: Value) -> AppResult<Value> {
    match (method, rest) {
        ("POST", ["create"]) => {
            let game_id = new_id();
            let chat = if let Some(chat_id) = body.get("chatId").and_then(Value::as_str).filter(|id| !id.is_empty()) {
                chat_metadata_patch(
                    state,
                    chat_id,
                    json!({ "gameId": game_id, "gameSessionNumber": 1, "gameSessionStatus": "setup", "gameSetupConfig": body.get("setupConfig").cloned().unwrap_or(Value::Null) }),
                )?
            } else {
                state.storage.create(
                    "chats",
                    json!({
                        "name": body.get("name").and_then(Value::as_str).unwrap_or("New Game"),
                        "mode": "game",
                        "characterIds": [],
                        "connectionId": body.get("connectionId").cloned().unwrap_or(Value::Null),
                        "metadata": {
                            "gameId": game_id,
                            "gameSessionNumber": 1,
                            "gameSessionStatus": "setup",
                            "gameSetupConfig": body.get("setupConfig").cloned().unwrap_or(Value::Null)
                        }
                    }),
                )?
            };
            Ok(json!({ "sessionChat": chat, "gameId": game_id }))
        }
        ("POST", ["setup"]) => {
            let chat_id = required_string(&body, "chatId")?;
            chat_metadata_patch(
                state,
                chat_id,
                json!({
                    "gameSetupConfig": body.get("setupConfig").or_else(|| body.get("preferences")).cloned().unwrap_or(Value::Null),
                    "gameSessionStatus": "ready",
                    "gameBlueprint": body.get("setup").cloned().unwrap_or_else(|| json!({}))
                }),
            )?;
            Ok(json!({ "setup": body, "worldOverview": Value::Null }))
        }
        ("POST", ["start"]) => {
            let chat_id = required_string(&body, "chatId")?;
            chat_metadata_patch(state, chat_id, json!({ "gameSessionStatus": "active", "gameActiveState": "exploration" }))?;
            Ok(json!({ "status": "active", "alreadyStarted": false }))
        }
        ("POST", ["session", "start"]) => {
            let game_id = required_string(&body, "gameId")?;
            let count = match chats_for_game(state, game_id)? {
                Value::Array(rows) => rows.len() as i64,
                _ => 0,
            };
            let session_number = count + 1;
            let chat = state.storage.create(
                "chats",
                json!({
                    "name": format!("Game Session {session_number}"),
                    "mode": "game",
                    "characterIds": [],
                    "connectionId": body.get("connectionId").cloned().unwrap_or(Value::Null),
                    "metadata": {
                        "gameId": game_id,
                        "gameSessionNumber": session_number,
                        "gameSessionStatus": "active",
                        "gameActiveState": "exploration"
                    }
                }),
            )?;
            Ok(json!({ "sessionChat": chat, "sessionNumber": session_number, "recap": "" }))
        }
        ("POST", ["session", "conclude"]) | ("POST", ["session", "regenerate-conclusion"]) => {
            let chat_id = required_string(&body, "chatId")?;
            let chat = get_required(state, "chats", chat_id)?;
            let session_number = metadata_map(&chat)
                .get("gameSessionNumber")
                .and_then(Value::as_i64)
                .unwrap_or(1);
            let summary = game_summary(session_number);
            chat_metadata_patch(state, chat_id, json!({ "gameSessionStatus": "concluded", "gamePreviousSessionSummaries": [summary.clone()] }))?;
            Ok(json!({ "summary": summary }))
        }
        ("POST", ["session", "regenerate-lorebook"]) => Ok(json!({ "sessionNumber": body.get("sessionNumber").cloned().unwrap_or_else(|| json!(1)), "lorebookId": Value::Null, "entryCount": 0 })),
        ("POST", ["session", "update-campaign-progression"]) => {
            let chat_id = required_string(&body, "chatId")?;
            let chat = get_required(state, "chats", chat_id)?;
            let game_id = metadata_map(&chat).get("gameId").cloned().unwrap_or(Value::Null);
            Ok(json!({ "sessionChat": chat, "gameId": game_id, "campaignProgression": { "storyArc": Value::Null, "plotTwists": [], "partyArcs": [] } }))
        }
        ("POST", ["party", "recruit"]) => {
            let chat_id = required_string(&body, "chatId")?;
            Ok(json!({ "sessionChat": get_required(state, "chats", chat_id)?, "added": false, "characterName": body.get("characterName").and_then(Value::as_str).unwrap_or("Character"), "cardCreated": false }))
        }
        ("POST", ["party", "card", "regenerate"]) => {
            let chat_id = required_string(&body, "chatId")?;
            Ok(json!({ "sessionChat": get_required(state, "chats", chat_id)?, "characterName": body.get("characterName").and_then(Value::as_str).unwrap_or("Character"), "gameCard": {} }))
        }
        ("POST", ["party", "remove"]) => {
            let chat_id = required_string(&body, "chatId")?;
            Ok(json!({ "sessionChat": get_required(state, "chats", chat_id)?, "removed": false, "characterName": body.get("characterName").and_then(Value::as_str).unwrap_or("Character") }))
        }
        ("POST", ["dice", "roll"]) => Ok(json!({ "result": dice_result(body.get("notation").and_then(Value::as_str).unwrap_or("1d20")) })),
        ("POST", ["skill-check"]) => {
            let dc = body.get("dc").and_then(Value::as_i64).unwrap_or(10);
            let roll = body.get("preRolledD20").and_then(Value::as_i64).unwrap_or(10);
            Ok(json!({ "result": { "skill": body.get("skill").and_then(Value::as_str).unwrap_or("skill"), "dc": dc, "rolls": [roll], "usedRoll": roll, "modifier": 0, "total": roll, "success": roll >= dc, "criticalSuccess": roll == 20, "criticalFailure": roll == 1, "rollMode": "normal" } }))
        }
        ("POST", ["state", "transition"]) => {
            let chat_id = required_string(&body, "chatId")?;
            let chat = get_required(state, "chats", chat_id)?;
            let previous = metadata_map(&chat).get("gameActiveState").cloned().unwrap_or_else(|| Value::String("exploration".to_string()));
            let new_state = body.get("newState").cloned().unwrap_or_else(|| Value::String("exploration".to_string()));
            chat_metadata_patch(state, chat_id, json!({ "gameActiveState": new_state.clone() }))?;
            Ok(json!({ "previousState": previous, "newState": new_state }))
        }
        ("POST", ["map", "generate"]) => Ok(json!({ "map": default_game_map(), "maps": [default_game_map()], "activeGameMapId": Value::Null })),
        ("POST", ["map", "move"]) => {
            let mut map = default_game_map();
            if let Some(object) = map.as_object_mut() {
                object.insert("partyPosition".to_string(), body.get("position").cloned().unwrap_or_else(|| json!({ "x": 1, "y": 1 })));
            }
            let active_id = map.get("id").cloned().unwrap_or(Value::Null);
            Ok(json!({ "map": map.clone(), "maps": [map], "activeGameMapId": active_id }))
        }
        ("PUT", [chat_id, "widgets"]) => {
            chat_metadata_patch(state, chat_id, json!({ "gameWidgetState": body.get("widgets").cloned().unwrap_or_else(|| json!([])) }))?;
            Ok(json!({ "ok": true }))
        }
        ("GET", [game_id, "sessions"]) => chats_for_game(state, game_id),
        ("GET", [chat_id, "journal"]) => Ok(json!({ "journal": metadata_map(&get_required(state, "chats", chat_id)?).get("gameJournal").cloned().unwrap_or_else(|| json!([])), "recap": "" })),
        ("GET", [chat_id, "checkpoints"]) => list_collection(state, "game-checkpoints", Some(("chatId", *chat_id))),
        ("POST", ["checkpoint"]) => state.storage.create("game-checkpoints", json!({
            "chatId": body.get("chatId").cloned().unwrap_or(Value::Null),
            "snapshotId": new_id(),
            "messageId": "",
            "label": body.get("label").and_then(Value::as_str).unwrap_or("Checkpoint"),
            "triggerType": body.get("triggerType").and_then(Value::as_str).unwrap_or("manual"),
            "location": Value::Null,
            "gameState": Value::Null,
            "weather": Value::Null,
            "timeOfDay": Value::Null,
            "turnNumber": Value::Null
        })).map(|record| json!({ "id": record.get("id").cloned().unwrap_or(Value::Null) })),
        ("POST", ["checkpoint", "load"]) => Ok(json!({ "ok": true, "messageId": "" })),
        ("DELETE", ["checkpoint", id]) => {
            let deleted = state.storage.delete("game-checkpoints", id)?;
            Ok(json!({ "ok": deleted }))
        }
        ("POST", ["combat", "round"]) => Ok(json!({ "result": { "round": body.get("round").cloned().unwrap_or_else(|| json!(1)), "log": [] }, "combatants": body.get("combatants").cloned().unwrap_or_else(|| json!([])) })),
        ("POST", ["combat", "loot"]) | ("POST", ["loot", "generate"]) => Ok(json!({ "drops": [] })),
        ("POST", ["time", "advance"]) => Ok(json!({ "time": {}, "formatted": "Now" })),
        ("POST", ["weather", "update"]) => Ok(json!({ "changed": false, "weather": { "type": "clear", "temperature": 20 } })),
        ("POST", ["encounter", "roll"]) => Ok(json!({ "encounter": { "triggered": false, "type": Value::Null, "hint": "" }, "enemyCount": 0 })),
        ("POST", ["reputation", "update"]) => Ok(json!({ "npcs": [], "changes": [] })),
        ("POST", ["journal", "entry"]) => Ok(json!({ "journal": body })),
        ("POST", ["generate-assets", "preview"]) => Ok(json!({ "items": [] })),
        ("POST", ["generate-assets"]) => Ok(json!({ "assets": [], "generated": [] })),
        ("POST", ["spotify", "candidates"]) => Ok(json!({ "tracks": [], "candidates": [] })),
        ("POST", ["spotify", "play"]) => Ok(json!({ "played": false })),
        ("POST", ["party-turn"]) => Ok(json!({ "message": Value::Null, "events": [] })),
        _ => Ok(json!({ "ok": true })),
    }
}
