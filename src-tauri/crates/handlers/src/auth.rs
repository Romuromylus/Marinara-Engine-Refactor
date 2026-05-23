// Transport-agnostic auth handlers. The Axum server target wires these into
// `/api/auth/login`, `/api/auth/logout`, and `/api/auth/me`, plus a middleware
// that gates every other /api/* path on a valid session cookie + CSRF header.
// The Tauri desktop binary never calls into this module — desktop is
// single-local-user and has no network surface to protect.
//
// Phase 6a (single-user, self-hosted) intentionally keeps the surface tiny:
// one admin user seeded from env vars on boot, opaque session ids, no
// password-reset flow, no signup. Multi-user is a deliberately-deferred
// Phase 6.x.

use argon2::Argon2;
use marinara_core::{now_iso, AppError, AppResult};
use marinara_storage::FileStorage;
use password_hash::{
    rand_core::{OsRng, RngCore},
    PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
};
use serde_json::{json, Map, Value};

pub const USERS_COLLECTION: &str = "users";
pub const SESSIONS_COLLECTION: &str = "sessions";

/// Name of the cookie the server sets on successful login. The Axum
/// middleware looks for exactly this key in the incoming `Cookie` header.
pub const SESSION_COOKIE_NAME: &str = "marinara_session";

/// Header the SPA must echo back on mutating requests (POST/PATCH/PUT/DELETE).
/// Case-insensitive per HTTP, but stored lowercase so callers don't have to
/// know which form Axum gives them.
pub const CSRF_HEADER_NAME: &str = "x-csrf-token";

// --- Password hashing -------------------------------------------------------

/// Hash a plaintext password with argon2id, returning the PHC string
/// representation (`$argon2id$v=19$m=...$<salt>$<hash>`). The matching
/// `marinara-server hash-password <plaintext>` CLI subcommand emits this
/// exact string; the operator pastes it into `MARINARA_ADMIN_PASSWORD_HASH`.
pub fn hash_password(plaintext: &str) -> AppResult<String> {
    if plaintext.is_empty() {
        return Err(AppError::invalid_input("Password must not be empty"));
    }
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(plaintext.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|error| AppError::new("hash_failed", error.to_string()))
}

/// Verify a plaintext password against a PHC hash string. Returns true on
/// match, false on a clean mismatch, error only on a malformed hash so the
/// caller can distinguish "wrong password" from "broken config".
pub fn verify_password(hash: &str, plaintext: &str) -> AppResult<bool> {
    let parsed = PasswordHash::new(hash)
        .map_err(|error| AppError::new("hash_invalid", error.to_string()))?;
    Ok(Argon2::default()
        .verify_password(plaintext.as_bytes(), &parsed)
        .is_ok())
}

// --- Admin bootstrap --------------------------------------------------------

/// Seed or update the admin user from env-var-derived inputs. Called once at
/// server boot when both `MARINARA_ADMIN_USERNAME` and
/// `MARINARA_ADMIN_PASSWORD_HASH` are set. If the row exists with the same
/// hash, this is a no-op (idempotent boots). If the row exists with a
/// different hash, the hash is rotated — env vars are the source of truth so
/// the operator can recover from a forgotten password by re-setting the env.
/// If the row does not exist, one is inserted.
pub fn bootstrap_admin(
    storage: &FileStorage,
    username: &str,
    password_hash: &str,
) -> AppResult<Value> {
    // Validate hash format before touching storage so a misconfigured env var
    // can never clobber a working user row with garbage.
    PasswordHash::new(password_hash)
        .map_err(|error| AppError::new("hash_invalid", error.to_string()))?;
    let trimmed = username.trim();
    if trimmed.is_empty() {
        return Err(AppError::invalid_input("Admin username must not be empty"));
    }

    let mut filters = Map::new();
    filters.insert("username".to_string(), Value::String(trimmed.to_string()));
    if let Some(existing) = storage
        .list_where(USERS_COLLECTION, &filters)?
        .into_iter()
        .next()
    {
        let existing_hash = existing
            .get("passwordHash")
            .and_then(Value::as_str)
            .unwrap_or("");
        if existing_hash == password_hash {
            return Ok(existing);
        }
        let id = existing
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                AppError::new(
                    "admin_bootstrap_failed",
                    "Existing admin row missing id field",
                )
            })?
            .to_string();
        return storage.patch(
            USERS_COLLECTION,
            &id,
            json!({ "passwordHash": password_hash, "updatedAt": now_iso() }),
        );
    }
    storage.create(
        USERS_COLLECTION,
        json!({
            "username": trimmed,
            "passwordHash": password_hash,
            "role": "admin",
            "createdAt": now_iso(),
            "updatedAt": now_iso()
        }),
    )
}

// --- Session lifecycle ------------------------------------------------------

/// Drop the `passwordHash` and other server-only fields before returning a
/// user record over the wire.
fn user_view(user: &Value) -> Value {
    json!({
        "id": user.get("id").and_then(Value::as_str).unwrap_or(""),
        "username": user.get("username").and_then(Value::as_str).unwrap_or(""),
        "role": user.get("role").and_then(Value::as_str).unwrap_or("user"),
    })
}

/// Verify credentials and create a session row. Returns
/// `{sessionId, csrfToken, user}`. The caller writes `sessionId` into a
/// `Set-Cookie` header and surfaces `csrfToken` to the SPA via the response
/// body — never via a cookie, since the SPA must read it to echo it back.
pub fn login(storage: &FileStorage, username: &str, password: &str) -> AppResult<Value> {
    let username = username.trim();
    if username.is_empty() || password.is_empty() {
        return Err(AppError::new(
            "invalid_credentials",
            "Invalid username or password",
        ));
    }
    let mut filters = Map::new();
    filters.insert("username".to_string(), Value::String(username.to_string()));
    let user = storage
        .list_where(USERS_COLLECTION, &filters)?
        .into_iter()
        .next()
        .ok_or_else(|| AppError::new("invalid_credentials", "Invalid username or password"))?;
    let stored_hash = user
        .get("passwordHash")
        .and_then(Value::as_str)
        .ok_or_else(|| AppError::new("invalid_credentials", "Invalid username or password"))?;
    if !verify_password(stored_hash, password)? {
        return Err(AppError::new(
            "invalid_credentials",
            "Invalid username or password",
        ));
    }
    let session_id = random_hex(32);
    let csrf_token = random_hex(32);
    let user_id = user
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let now = now_iso();
    storage.upsert_with_id(
        SESSIONS_COLLECTION,
        &session_id,
        json!({
            "id": session_id,
            "userId": user_id,
            "csrfToken": csrf_token,
            "createdAt": now,
            "lastSeen": now,
        }),
    )?;
    Ok(json!({
        "sessionId": session_id,
        "csrfToken": csrf_token,
        "user": user_view(&user),
    }))
}

/// Look up a session by cookie value, refresh its `lastSeen`, and return the
/// session payload + user view + CSRF token. Returns `None` if the session
/// id is unknown or if the session points at a since-deleted user (in which
/// case the orphaned session row is also removed). The auth middleware uses
/// this on every protected request.
pub fn resolve_session(storage: &FileStorage, session_id: &str) -> AppResult<Option<Value>> {
    if session_id.is_empty() {
        return Ok(None);
    }
    let Some(session) = storage.get(SESSIONS_COLLECTION, session_id)? else {
        return Ok(None);
    };
    let user_id = session.get("userId").and_then(Value::as_str).unwrap_or("");
    let Some(user) = storage.get(USERS_COLLECTION, user_id)? else {
        // Orphaned session — user was deleted out from under it. Clean up.
        let _ = storage.delete(SESSIONS_COLLECTION, session_id);
        return Ok(None);
    };
    storage.patch(
        SESSIONS_COLLECTION,
        session_id,
        json!({ "lastSeen": now_iso() }),
    )?;
    Ok(Some(json!({
        "user": user_view(&user),
        "csrfToken": session.get("csrfToken").and_then(Value::as_str).unwrap_or(""),
    })))
}

/// Delete a session row by id. Idempotent: returns `{ok: true, removed:
/// false}` for an unknown id rather than erroring, because the SPA may
/// double-fire logout if the cookie expired client-side first.
pub fn logout(storage: &FileStorage, session_id: &str) -> AppResult<Value> {
    let removed = storage.delete(SESSIONS_COLLECTION, session_id)?;
    Ok(json!({ "ok": true, "removed": removed }))
}

fn random_hex(byte_count: usize) -> String {
    let mut bytes = vec![0u8; byte_count];
    OsRng.fill_bytes(&mut bytes);
    let mut out = String::with_capacity(byte_count * 2);
    for byte in bytes {
        out.push_str(&format!("{:02x}", byte));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup() -> (TempDir, FileStorage) {
        let dir = TempDir::new().expect("temp dir");
        let storage = FileStorage::new(dir.path().join("data")).expect("storage");
        (dir, storage)
    }

    #[test]
    fn hash_then_verify_round_trips() {
        let hash = hash_password("correct horse battery staple").expect("hash");
        assert!(verify_password(&hash, "correct horse battery staple").expect("verify"));
        assert!(!verify_password(&hash, "wrong password").expect("verify"));
    }

    #[test]
    fn hash_password_rejects_empty() {
        let err = hash_password("").expect_err("empty");
        assert_eq!(err.code, "invalid_input");
    }

    #[test]
    fn verify_password_errors_on_malformed_hash() {
        let err = verify_password("not-a-real-phc-string", "anything").expect_err("malformed");
        assert_eq!(err.code, "hash_invalid");
    }

    #[test]
    fn bootstrap_admin_creates_row_when_missing() {
        let (_dir, storage) = setup();
        let hash = hash_password("hunter2").unwrap();
        let row = bootstrap_admin(&storage, "jerome", &hash).expect("bootstrap");
        assert_eq!(row.get("username").and_then(Value::as_str), Some("jerome"));
        assert_eq!(row.get("role").and_then(Value::as_str), Some("admin"));
        assert_eq!(
            row.get("passwordHash").and_then(Value::as_str),
            Some(hash.as_str())
        );
    }

    #[test]
    fn bootstrap_admin_is_noop_on_matching_hash() {
        let (_dir, storage) = setup();
        let hash = hash_password("hunter2").unwrap();
        let first = bootstrap_admin(&storage, "jerome", &hash).unwrap();
        let second = bootstrap_admin(&storage, "jerome", &hash).unwrap();
        assert_eq!(
            first.get("id").and_then(Value::as_str),
            second.get("id").and_then(Value::as_str)
        );
        // Single row in collection after two bootstrap calls with same hash.
        assert_eq!(storage.list(USERS_COLLECTION).unwrap().len(), 1);
    }

    #[test]
    fn bootstrap_admin_rotates_hash_when_env_changes() {
        let (_dir, storage) = setup();
        let old_hash = hash_password("hunter2").unwrap();
        let first = bootstrap_admin(&storage, "jerome", &old_hash).unwrap();
        let new_hash = hash_password("better-password").unwrap();
        let second = bootstrap_admin(&storage, "jerome", &new_hash).unwrap();
        // Same row id, new hash.
        assert_eq!(
            first.get("id").and_then(Value::as_str),
            second.get("id").and_then(Value::as_str)
        );
        assert_eq!(
            second.get("passwordHash").and_then(Value::as_str),
            Some(new_hash.as_str())
        );
        // Old password no longer verifies against the persisted row.
        let persisted = storage
            .get(
                USERS_COLLECTION,
                second.get("id").and_then(Value::as_str).unwrap(),
            )
            .unwrap()
            .unwrap();
        let persisted_hash = persisted
            .get("passwordHash")
            .and_then(Value::as_str)
            .unwrap();
        assert!(verify_password(persisted_hash, "better-password").unwrap());
        assert!(!verify_password(persisted_hash, "hunter2").unwrap());
    }

    #[test]
    fn bootstrap_admin_rejects_invalid_hash_format() {
        let (_dir, storage) = setup();
        let err = bootstrap_admin(&storage, "jerome", "not-a-phc-string").expect_err("bad hash");
        assert_eq!(err.code, "hash_invalid");
        // No row was created — the validation happens before storage touches.
        assert!(storage.list(USERS_COLLECTION).unwrap().is_empty());
    }

    #[test]
    fn login_success_returns_session_and_csrf() {
        let (_dir, storage) = setup();
        let hash = hash_password("hunter2").unwrap();
        bootstrap_admin(&storage, "jerome", &hash).unwrap();
        let response = login(&storage, "jerome", "hunter2").expect("login");
        let session_id = response.get("sessionId").and_then(Value::as_str).unwrap();
        let csrf = response.get("csrfToken").and_then(Value::as_str).unwrap();
        assert_eq!(session_id.len(), 64); // 32 bytes hex-encoded
        assert_eq!(csrf.len(), 64);
        let user = response.get("user").and_then(Value::as_object).unwrap();
        assert_eq!(user.get("username").and_then(Value::as_str), Some("jerome"));
        // passwordHash MUST NOT be in the response.
        assert!(user.get("passwordHash").is_none());
    }

    #[test]
    fn login_rejects_wrong_password() {
        let (_dir, storage) = setup();
        let hash = hash_password("hunter2").unwrap();
        bootstrap_admin(&storage, "jerome", &hash).unwrap();
        let err = login(&storage, "jerome", "wrong").expect_err("login");
        assert_eq!(err.code, "invalid_credentials");
    }

    #[test]
    fn login_rejects_unknown_user() {
        let (_dir, storage) = setup();
        let err = login(&storage, "ghost", "anything").expect_err("login");
        assert_eq!(err.code, "invalid_credentials");
    }

    #[test]
    fn login_rejects_empty_inputs() {
        let (_dir, storage) = setup();
        assert_eq!(
            login(&storage, "", "anything").unwrap_err().code,
            "invalid_credentials"
        );
        assert_eq!(
            login(&storage, "jerome", "").unwrap_err().code,
            "invalid_credentials"
        );
    }

    #[test]
    fn resolve_session_returns_user_and_csrf() {
        let (_dir, storage) = setup();
        let hash = hash_password("hunter2").unwrap();
        bootstrap_admin(&storage, "jerome", &hash).unwrap();
        let response = login(&storage, "jerome", "hunter2").unwrap();
        let session_id = response
            .get("sessionId")
            .and_then(Value::as_str)
            .unwrap()
            .to_string();
        let csrf = response
            .get("csrfToken")
            .and_then(Value::as_str)
            .unwrap()
            .to_string();
        let resolved = resolve_session(&storage, &session_id).unwrap().unwrap();
        assert_eq!(
            resolved.get("csrfToken").and_then(Value::as_str),
            Some(csrf.as_str())
        );
        assert_eq!(
            resolved
                .get("user")
                .and_then(|user| user.get("username"))
                .and_then(Value::as_str),
            Some("jerome")
        );
    }

    #[test]
    fn resolve_session_returns_none_for_unknown_id() {
        let (_dir, storage) = setup();
        assert!(resolve_session(&storage, "does-not-exist")
            .unwrap()
            .is_none());
        assert!(resolve_session(&storage, "").unwrap().is_none());
    }

    #[test]
    fn resolve_session_cleans_up_orphaned_session() {
        let (_dir, storage) = setup();
        let hash = hash_password("hunter2").unwrap();
        let user = bootstrap_admin(&storage, "jerome", &hash).unwrap();
        let response = login(&storage, "jerome", "hunter2").unwrap();
        let session_id = response
            .get("sessionId")
            .and_then(Value::as_str)
            .unwrap()
            .to_string();
        // Delete the user underneath the session.
        storage
            .delete(
                USERS_COLLECTION,
                user.get("id").and_then(Value::as_str).unwrap(),
            )
            .unwrap();
        assert!(resolve_session(&storage, &session_id).unwrap().is_none());
        // Session row should have been removed as collateral.
        assert!(storage
            .get(SESSIONS_COLLECTION, &session_id)
            .unwrap()
            .is_none());
    }

    #[test]
    fn logout_removes_session_row() {
        let (_dir, storage) = setup();
        let hash = hash_password("hunter2").unwrap();
        bootstrap_admin(&storage, "jerome", &hash).unwrap();
        let response = login(&storage, "jerome", "hunter2").unwrap();
        let session_id = response
            .get("sessionId")
            .and_then(Value::as_str)
            .unwrap()
            .to_string();
        let result = logout(&storage, &session_id).unwrap();
        assert_eq!(result.get("removed").and_then(Value::as_bool), Some(true));
        assert!(resolve_session(&storage, &session_id).unwrap().is_none());
    }

    #[test]
    fn logout_is_idempotent_on_unknown_id() {
        let (_dir, storage) = setup();
        let result = logout(&storage, "does-not-exist").unwrap();
        assert_eq!(result.get("removed").and_then(Value::as_bool), Some(false));
        assert_eq!(result.get("ok").and_then(Value::as_bool), Some(true));
    }

    #[test]
    fn random_hex_is_byte_count_times_two_chars() {
        assert_eq!(random_hex(32).len(), 64);
        assert_eq!(random_hex(16).len(), 32);
        // Two consecutive draws should not collide (vanishingly improbable).
        assert_ne!(random_hex(32), random_hex(32));
    }
}
