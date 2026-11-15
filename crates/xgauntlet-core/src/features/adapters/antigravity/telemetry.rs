//! Antigravity PreInvocation telemetry orchestration and state-aware deduplication.

use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use super::hud::{
    format_pre_invocation_system_message, format_snapshot_system_message, render_compact_chip,
    render_start_hud, structural_hud_fingerprint,
};

static DEDUP_TMP_COUNTER: AtomicU64 = AtomicU64::new(1);

fn is_test_runtime() -> bool {
    cfg!(test)
        || std::env::var("XGAUNTLET_TEST_MODE").is_ok()
        || std::env::current_exe()
            .map(|p| {
                let s = p.to_string_lossy();
                s.contains("/deps/") || s.contains("test")
            })
            .unwrap_or(false)
}

fn current_username() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "default".to_string())
}

fn scope_hash(workspace: Option<&Path>, session_id: Option<&str>) -> String {
    let ws_key = workspace
        .map(|p| {
            p.canonicalize()
                .unwrap_or_else(|_| p.to_path_buf())
                .to_string_lossy()
                .into_owned()
        })
        .unwrap_or_default();
    let sess_key = session_id.unwrap_or("default").trim();
    let full = format!("{ws_key}\0{sess_key}");
    let hex = format!("{:x}", Sha256::digest(full.as_bytes()));
    hex.chars().take(16).collect()
}

fn scoped_dedup_cache_file(
    cache_dir: &Path,
    workspace: Option<&Path>,
    session_id: Option<&str>,
) -> PathBuf {
    let hash = scope_hash(workspace, session_id);
    cache_dir.join(format!(
        "xgauntlet_telemetry_dedup_{}_{}.json",
        current_username(),
        hash
    ))
}

fn dedup_cache_file() -> PathBuf {
    scoped_dedup_cache_file(&std::env::temp_dir(), None, None)
}

fn read_dedup_cache(cache_file: &Path) -> Option<Value> {
    let content = std::fs::read_to_string(cache_file).ok()?;
    serde_json::from_str::<Value>(&content).ok()
}

fn now_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

/// Extracts the semantic state key used for 120s PreInvocation deduplication.
///
/// Includes `Task | Phase | Verdict` (line 2) AND any `Session: <badge>` status
/// transition (`Optimal` -> `Aging` -> `Saturated`) so session-health warnings are
/// never suppressed when the phase remains unchanged.
pub fn extract_telemetry_state_key(message: &str) -> String {
    let base_line = message.lines().nth(1).unwrap_or(message).trim();
    let session_status = message
        .lines()
        .find_map(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with("Session:") {
                Some(
                    trimmed
                        .split('(')
                        .next()
                        .unwrap_or(trimmed)
                        .trim()
                        .to_string(),
                )
            } else {
                None
            }
        })
        .unwrap_or_default();

    if session_status.is_empty() {
        base_line.to_string()
    } else {
        format!("{base_line} | {session_status}")
    }
}

fn check_rapid_duplicate_in_file(cache_file: &Path) -> bool {
    let now = now_millis();

    if let Some(val) = read_dedup_cache(cache_file) {
        let last_ts = val
            .get("timestamp_ms")
            .and_then(|t| t.as_u64())
            .unwrap_or(0) as u128;
        if now.saturating_sub(last_ts) < 1500 {
            return true;
        }
    }

    false
}

/// Checks whether another PreInvocation hook fired within the last 1500ms
/// (e.g., duplicate global + workspace hook entries), preventing double-counting.
pub fn is_rapid_duplicate_pre_invocation() -> bool {
    if is_test_runtime() {
        return false;
    }

    check_rapid_duplicate_in_file(&dedup_cache_file())
}

/// Checks whether another PreInvocation hook fired within 1500ms for the exact workspace + session pair.
pub fn is_rapid_duplicate_for_session(workspace: &Path, session_id: &str) -> bool {
    if is_test_runtime() {
        return false;
    }

    let cache_file =
        scoped_dedup_cache_file(&std::env::temp_dir(), Some(workspace), Some(session_id));
    check_rapid_duplicate_in_file(&cache_file)
}

fn suppress_and_update_in_file(cache_file: &Path, message: &str) -> bool {
    let now = now_millis();
    let current_digest = format!("{:x}", Sha256::digest(message.as_bytes()));
    let current_state_key = extract_telemetry_state_key(message);

    if let Some(val) = read_dedup_cache(cache_file) {
        let last_digest = val.get("digest").and_then(|d| d.as_str()).unwrap_or("");
        let last_state_key = val.get("state_key").and_then(|s| s.as_str()).unwrap_or("");
        let last_ts = val
            .get("timestamp_ms")
            .and_then(|t| t.as_u64())
            .unwrap_or(0) as u128;

        let is_same_state = last_digest == current_digest
            || (!last_state_key.is_empty() && last_state_key == current_state_key);

        if is_same_state && now.saturating_sub(last_ts) < 120_000 {
            return true;
        }
    }

    let payload = serde_json::json!({
        "digest": current_digest,
        "state_key": current_state_key,
        "timestamp_ms": now,
    });
    write_json_atomic(cache_file, &payload);
    false
}

/// Suppresses duplicate or semantically unchanged PreInvocation injections within 120s.
pub fn should_suppress_duplicate_telemetry(message: &str) -> bool {
    if is_test_runtime() {
        return false;
    }

    suppress_and_update_in_file(&dedup_cache_file(), message)
}

/// Workspace- and session-scoped telemetry deduplication using an explicit cache directory.
pub fn should_suppress_duplicate_telemetry_in_dir(
    cache_dir: &Path,
    workspace: &Path,
    session_id: &str,
    message: &str,
) -> bool {
    let cache_file = scoped_dedup_cache_file(cache_dir, Some(workspace), Some(session_id));
    suppress_and_update_in_file(&cache_file, message)
}

/// Suppresses duplicate PreInvocation injections scoped strictly to `(workspace, session_id)`.
pub fn should_suppress_duplicate_telemetry_for_session(
    workspace: &Path,
    session_id: &str,
    message: &str,
) -> bool {
    if is_test_runtime() {
        return false;
    }

    should_suppress_duplicate_telemetry_in_dir(
        &std::env::temp_dir(),
        workspace,
        session_id,
        message,
    )
}

fn cadence_cache_file(cache_dir: &Path, workspace: &Path, session_id: &str) -> PathBuf {
    let hash = scope_hash(Some(workspace), Some(session_id));
    cache_dir.join(format!(
        "xgauntlet_hud_cadence_{}_{}.json",
        current_username(),
        hash
    ))
}

fn write_json_atomic(target: &Path, payload: &Value) {
    let user = current_username();
    let seq = DEDUP_TMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let tmp_dir = target
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(std::env::temp_dir);
    let tmp_file = tmp_dir.join(format!(
        "xgauntlet_cadence_{user}.{}.{seq}.tmp",
        std::process::id()
    ));
    if std::fs::write(&tmp_file, payload.to_string()).is_ok() {
        let _ = std::fs::rename(&tmp_file, target);
        let _ = std::fs::remove_file(&tmp_file);
    }
}

fn extract_invocation_num(stdin_json: Option<&Value>) -> Option<u64> {
    stdin_json
        .and_then(|v| v.get("invocationNum"))
        .and_then(|n| n.as_u64())
}

fn is_rapid_duplicate_step(cache_file: &Path, invocation_num: Option<u64>) -> bool {
    if is_test_runtime() {
        return false;
    }
    let Some(val) = read_dedup_cache(cache_file) else {
        return false;
    };
    let last_ts = val
        .get("timestamp_ms")
        .and_then(|t| t.as_u64())
        .unwrap_or(0) as u128;
    if now_millis().saturating_sub(last_ts) >= 1500 {
        return false;
    }
    let last_inv = val.get("last_invocation_num").and_then(|n| n.as_u64());
    last_inv == invocation_num
}

/// Synchronizes session telemetry using an explicit HUD cadence cache directory.
pub fn prepare_pre_invocation_payload_with_cache_dir(
    workspace: &Path,
    explicit_task: Option<&str>,
    stdin_json: Option<&Value>,
    raw_stdin: &str,
    cache_dir: &Path,
) -> Result<Value, crate::features::tasks::TaskError> {
    let session_id = crate::features::telemetry::extract_session_id(stdin_json);
    let invocation_num = extract_invocation_num(stdin_json);
    let is_user_prompt_turn = invocation_num.map(|n| n <= 1).unwrap_or(true);
    let cache_file = cadence_cache_file(cache_dir, workspace, &session_id);

    if is_rapid_duplicate_step(&cache_file, invocation_num) {
        return Ok(serde_json::json!({ "injectSteps": [] }));
    }

    crate::features::telemetry::sync_active_session_for_pre_invocation(
        workspace, stdin_json, raw_stdin,
    );

    let snapshot = crate::features::tasks::inspect_workspace_cockpit(
        workspace,
        explicit_task,
        Some(&session_id),
    )?;
    let current_fp = structural_hud_fingerprint(&snapshot);
    let sys_msg = format_snapshot_system_message(&snapshot);

    let prev = read_dedup_cache(&cache_file);
    let prev_user_fp = prev
        .as_ref()
        .and_then(|v| v.get("user_hud_fp"))
        .and_then(|s| s.as_str())
        .map(str::to_string);
    let prev_model_fp = prev
        .as_ref()
        .and_then(|v| v.get("model_fp"))
        .and_then(|s| s.as_str())
        .map(str::to_string);
    let last_ts = prev
        .as_ref()
        .and_then(|v| v.get("timestamp_ms"))
        .and_then(|t| t.as_u64())
        .unwrap_or(0) as u128;

    let now = now_millis();
    let model_state_changed = prev_model_fp.as_deref() != Some(current_fp.as_str());
    let user_hud_changed = prev_user_fp.as_deref() != Some(current_fp.as_str());

    if !is_user_prompt_turn {
        if !model_state_changed && !is_test_runtime() && now.saturating_sub(last_ts) < 120_000 {
            return Ok(serde_json::json!({ "injectSteps": [] }));
        }

        let updated = serde_json::json!({
            "user_hud_fp": prev_user_fp,
            "model_fp": current_fp,
            "timestamp_ms": now,
            "last_invocation_num": invocation_num,
        });
        write_json_atomic(&cache_file, &updated);
        return Ok(format_pre_invocation_system_message(&sys_msg, None));
    }

    let user_md = if user_hud_changed {
        render_start_hud(&snapshot)
    } else {
        render_compact_chip(&snapshot)
    };

    let updated = serde_json::json!({
        "user_hud_fp": current_fp,
        "model_fp": current_fp,
        "timestamp_ms": now,
        "last_invocation_num": invocation_num,
    });
    write_json_atomic(&cache_file, &updated);

    Ok(format_pre_invocation_system_message(
        &sys_msg,
        Some(&user_md),
    ))
}

/// Synchronizes active task session telemetry and builds the canonical Antigravity `PreInvocation` JSON payload.
pub fn prepare_pre_invocation_payload(
    workspace: &Path,
    explicit_task: Option<&str>,
    stdin_json: Option<&Value>,
    raw_stdin: &str,
) -> Result<Value, crate::features::tasks::TaskError> {
    prepare_pre_invocation_payload_with_cache_dir(
        workspace,
        explicit_task,
        stdin_json,
        raw_stdin,
        &std::env::temp_dir(),
    )
}
