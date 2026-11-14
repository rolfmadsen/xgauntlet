//! Antigravity PreInvocation telemetry orchestration and state-aware deduplication.

use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use super::AntigravityAdapter;

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

fn dedup_cache_file() -> PathBuf {
    std::env::temp_dir().join(format!(
        "xgauntlet_telemetry_dedup_{}.json",
        current_username()
    ))
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

/// Checks whether another PreInvocation hook fired within the last 1500ms
/// (e.g., duplicate global + workspace hook entries), preventing double-counting.
pub fn is_rapid_duplicate_pre_invocation() -> bool {
    if is_test_runtime() {
        return false;
    }

    let cache_file = dedup_cache_file();
    let now = now_millis();

    if let Some(val) = read_dedup_cache(&cache_file) {
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

/// Suppresses duplicate or semantically unchanged PreInvocation injections within 120s.
pub fn should_suppress_duplicate_telemetry(message: &str) -> bool {
    if is_test_runtime() {
        return false;
    }

    let cache_file = dedup_cache_file();
    let now = now_millis();
    let current_digest = format!("{:x}", Sha256::digest(message.as_bytes()));
    let current_state_key = extract_telemetry_state_key(message);

    if let Some(val) = read_dedup_cache(&cache_file) {
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

    let user = current_username();
    let payload = serde_json::json!({
        "digest": current_digest,
        "state_key": current_state_key,
        "timestamp_ms": now,
    });
    let seq = DEDUP_TMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let tmp_file = std::env::temp_dir().join(format!(
        "xgauntlet_telemetry_dedup_{user}.{}.{seq}.tmp",
        std::process::id()
    ));
    if std::fs::write(&tmp_file, payload.to_string()).is_ok() {
        let _ = std::fs::rename(&tmp_file, &cache_file);
        let _ = std::fs::remove_file(&tmp_file);
    }
    false
}

/// Synchronizes active task session telemetry (guarding against <1500ms rapid duplicates)
/// and builds the canonical Antigravity `PreInvocation` JSON payload.
pub fn prepare_pre_invocation_payload(
    workspace: &Path,
    explicit_task: Option<&str>,
    stdin_json: Option<&Value>,
    raw_stdin: &str,
) -> Result<Value, crate::features::tasks::TaskError> {
    if !is_rapid_duplicate_pre_invocation() {
        crate::features::telemetry::sync_active_session_for_pre_invocation(
            workspace, stdin_json, raw_stdin,
        );
    }

    let telemetry = crate::features::tasks::inspect_task_telemetry(workspace, explicit_task)?;
    let ephemeral = AntigravityAdapter::format_ephemeral_telemetry(&telemetry);
    Ok(AntigravityAdapter::format_pre_invocation_payload(
        &ephemeral,
    ))
}
