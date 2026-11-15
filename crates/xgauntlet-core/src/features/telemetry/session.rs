//! Session Health, Turn & Token Telemetry engine.
//!
//! Provides deterministic session telemetry (turns count, accumulated payload bytes,
//! and estimated token saturation) without requiring external tokenizer network calls
//! or background daemons.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

pub const SESSION_STATE_FILE: &str = ".xgauntlet/session_state.json";
pub const SESSION_STATE_DIR: &str = ".xgauntlet/sessions";

static ATOMIC_WRITE_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Categorization of session health based on interaction turns and cognitive load.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionHealthStatus {
    /// 1–10 turns: High reasoning fidelity and minimal context drift.
    Optimal,
    /// 11–18 turns: Maturing context window, approaching attention dilution threshold.
    Aging,
    /// >18 turns: Saturated context, recommended session handoff to avoid context rot.
    Saturated,
}

impl SessionHealthStatus {
    /// Determines health status deterministically from interaction turn count.
    pub fn from_turns(turns: usize) -> Self {
        match turns {
            0..=10 => SessionHealthStatus::Optimal,
            11..=18 => SessionHealthStatus::Aging,
            _ => SessionHealthStatus::Saturated,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            SessionHealthStatus::Optimal => "Optimal",
            SessionHealthStatus::Aging => "Aging",
            SessionHealthStatus::Saturated => "Saturated",
        }
    }

    pub fn badge(&self) -> &'static str {
        match self {
            SessionHealthStatus::Optimal => "🟢 Optimal",
            SessionHealthStatus::Aging => "🟡 Aging",
            SessionHealthStatus::Saturated => "🔴 Saturated",
        }
    }
}

/// Estimates token consumption deterministically based on character count
/// using the ~1 token ≈ 3.8 characters BPE heuristic.
pub fn estimate_tokens_from_chars(char_count: usize) -> usize {
    if char_count == 0 {
        0
    } else {
        ((char_count as f64) / 3.8).round() as usize
    }
}

/// Estimates token consumption deterministically from a string slice.
pub fn estimate_tokens_from_str(s: &str) -> usize {
    estimate_tokens_from_chars(s.chars().count())
}

/// Formats a token count into a compact human-readable representation (e.g., "15k tok" or "450 tok").
pub fn format_compact_tokens(tokens: usize) -> String {
    if tokens >= 1_000 {
        format!("~{}k tok", tokens / 1_000)
    } else {
        format!("{tokens} tok")
    }
}

/// Real-time session health telemetry record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionHealth {
    pub turns_count: usize,
    pub payload_bytes: usize,
    pub estimated_tokens: usize,
    pub status: SessionHealthStatus,
}

impl SessionHealth {
    pub fn new(turns_count: usize, payload_bytes: usize, estimated_tokens: usize) -> Self {
        let status = SessionHealthStatus::from_turns(turns_count);
        Self {
            turns_count,
            payload_bytes,
            estimated_tokens,
            status,
        }
    }

    pub fn calculate(
        turns_count: usize,
        payload_bytes: usize,
        explicit_tokens: Option<usize>,
    ) -> Self {
        let estimated_tokens =
            explicit_tokens.unwrap_or_else(|| estimate_tokens_from_chars(payload_bytes));
        Self::new(turns_count, payload_bytes, estimated_tokens)
    }

    /// Formats a high-density summary for the Dynamic Cockpit HUD.
    pub fn format_hud_summary(&self) -> String {
        let turn_label = if self.turns_count == 1 {
            "turn"
        } else {
            "turns"
        };
        let tok_label = format_compact_tokens(self.estimated_tokens);

        match self.status {
            SessionHealthStatus::Optimal => {
                format!(
                    "{} ({} {} • {})",
                    self.status.badge(),
                    self.turns_count,
                    turn_label,
                    tok_label
                )
            }
            SessionHealthStatus::Aging | SessionHealthStatus::Saturated => {
                format!(
                    "{} ({} {} • {}) • Forbered Handoff",
                    self.status.badge(),
                    self.turns_count,
                    turn_label,
                    tok_label
                )
            }
        }
    }
}

/// Persisted session tracking state in `.xgauntlet/session_state.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionTrackerState {
    pub task_id: String,
    pub turns_count: usize,
    pub payload_bytes: usize,
    pub estimated_tokens: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct MultiSessionStore {
    #[serde(default)]
    active_session_id: String,
    #[serde(default)]
    sessions: BTreeMap<String, SessionTrackerState>,
}

/// Extracts a stable session identifier from a hook payload, falling back to `"default"`.
pub fn extract_session_id(payload: Option<&serde_json::Value>) -> String {
    if let Some(val) = payload {
        for key in [
            "session_id",
            "sessionId",
            "conversation_id",
            "conversationId",
        ] {
            if let Some(s) = val.get(key).and_then(|v| v.as_str()) {
                let trimmed = s.trim();
                if !trimmed.is_empty() {
                    return trimmed.to_string();
                }
            }
        }
    }
    "default".to_string()
}

fn read_session_store(workspace: &Path) -> Result<Option<MultiSessionStore>, std::io::Error> {
    let state_path = workspace.join(SESSION_STATE_FILE);
    if !state_path.is_file() {
        return Ok(None);
    }

    let content = fs::read_to_string(&state_path)?;
    if let Ok(store) = serde_json::from_str::<MultiSessionStore>(&content) {
        if !store.sessions.is_empty() || !store.active_session_id.is_empty() {
            return Ok(Some(store));
        }
    }

    if let Ok(legacy) = serde_json::from_str::<SessionTrackerState>(&content) {
        let mut sessions = BTreeMap::new();
        sessions.insert("default".to_string(), legacy);
        return Ok(Some(MultiSessionStore {
            active_session_id: "default".to_string(),
            sessions,
        }));
    }

    Ok(None)
}

fn write_bytes_atomic(target: &Path, content: &[u8]) -> Result<(), std::io::Error> {
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    let seq = ATOMIC_WRITE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let tmp_name = format!(
        ".{}.tmp.{}.{}",
        target
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("state"),
        std::process::id(),
        seq
    );
    let tmp_path = target.with_file_name(tmp_name);
    fs::write(&tmp_path, content)?;
    fs::rename(&tmp_path, target)?;
    Ok(())
}

fn explicit_session_file(workspace: &Path, session_id: &str) -> PathBuf {
    let hex = format!("{:x}", Sha256::digest(session_id.trim().as_bytes()));
    let prefix: String = hex.chars().take(16).collect();
    workspace
        .join(SESSION_STATE_DIR)
        .join(format!("{prefix}.json"))
}

fn task_ids_match(stored_task_id: &str, requested_task_id: &str) -> bool {
    if stored_task_id == requested_task_id {
        return true;
    }
    let short_target = crate::features::checkpoint::extract_short_task_id(requested_task_id);
    let short_stored = crate::features::checkpoint::extract_short_task_id(stored_task_id);
    !short_target.is_empty() && short_target == short_stored
}

fn write_session_store(workspace: &Path, store: &MultiSessionStore) -> Result<(), std::io::Error> {
    let state_path = workspace.join(SESSION_STATE_FILE);
    let json = serde_json::to_string_pretty(store).map_err(std::io::Error::other)?;
    write_bytes_atomic(&state_path, json.as_bytes())
}

/// File-based session state tracker.
pub struct SessionTracker;

impl SessionTracker {
    /// Loads active session health for an explicit session identifier, or falls back to the most recent session in `.xgauntlet/session_state.json`.
    pub fn load_for_session(
        workspace: &Path,
        task_id: &str,
        session_id: Option<&str>,
    ) -> Result<Option<SessionHealth>, std::io::Error> {
        let explicit_key = session_id
            .map(str::trim)
            .filter(|s| !s.is_empty() && *s != "default");

        if let Some(key) = explicit_key {
            let sess_path = explicit_session_file(workspace, key);
            if sess_path.is_file() {
                if let Ok(content) = fs::read_to_string(&sess_path) {
                    if let Ok(state) = serde_json::from_str::<SessionTrackerState>(&content) {
                        return if task_ids_match(&state.task_id, task_id) {
                            Ok(Some(SessionHealth::new(
                                state.turns_count,
                                state.payload_bytes,
                                state.estimated_tokens,
                            )))
                        } else {
                            Ok(None)
                        };
                    }
                }
            }
        }

        let Some(store) = read_session_store(workspace)? else {
            return Ok(None);
        };

        let fallback_key = if store.active_session_id.is_empty() {
            "default"
        } else {
            store.active_session_id.as_str()
        };
        let key = session_id
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or(fallback_key);

        let Some(state) = store.sessions.get(key) else {
            return Ok(None);
        };

        if task_ids_match(&state.task_id, task_id) {
            Ok(Some(SessionHealth::new(
                state.turns_count,
                state.payload_bytes,
                state.estimated_tokens,
            )))
        } else {
            Ok(None)
        }
    }

    /// Loads active session health if the recorded state matches the requested task_id.
    pub fn load(workspace: &Path, task_id: &str) -> Result<Option<SessionHealth>, std::io::Error> {
        Self::load_for_session(workspace, task_id, None)
    }

    /// Persists session tracking state for a specific session identifier in `.xgauntlet/sessions/<hash>.json`
    /// or `.xgauntlet/session_state.json` for default/legacy single-session callers.
    pub fn save_for_session(
        workspace: &Path,
        session_id: &str,
        state: &SessionTrackerState,
    ) -> Result<(), std::io::Error> {
        let trimmed = session_id.trim();
        if !trimmed.is_empty() && trimmed != "default" {
            let sess_path = explicit_session_file(workspace, trimmed);
            let json = serde_json::to_string_pretty(state).map_err(std::io::Error::other)?;
            return write_bytes_atomic(&sess_path, json.as_bytes());
        }

        let mut store = read_session_store(workspace)?.unwrap_or_default();
        store.active_session_id = "default".to_string();
        store.sessions.insert("default".to_string(), state.clone());
        write_session_store(workspace, &store)
    }

    /// Persists session tracking state to `.xgauntlet/session_state.json`.
    pub fn save(workspace: &Path, state: &SessionTrackerState) -> Result<(), std::io::Error> {
        Self::save_for_session(workspace, "default", state)
    }
}

/// Records a turn interaction for a specific session and updates `.xgauntlet/session_state.json`.
pub fn record_session_turn_for_session(
    workspace: &Path,
    task_id: &str,
    session_id: &str,
    additional_payload_bytes: usize,
    explicit_tokens: Option<usize>,
) -> Result<SessionHealth, std::io::Error> {
    let existing = SessionTracker::load_for_session(workspace, task_id, Some(session_id))?;
    let (turns_count, payload_bytes, estimated_tokens) = match existing {
        Some(h) => {
            let next_turns = h.turns_count + 1;
            let next_payload = h.payload_bytes + additional_payload_bytes;
            let next_tokens = match explicit_tokens {
                Some(tokens) => h.estimated_tokens + tokens,
                None => estimate_tokens_from_chars(next_payload),
            };
            (next_turns, next_payload, next_tokens)
        }
        None => {
            let next_turns = 1;
            let next_payload = additional_payload_bytes;
            let next_tokens =
                explicit_tokens.unwrap_or_else(|| estimate_tokens_from_chars(next_payload));
            (next_turns, next_payload, next_tokens)
        }
    };

    let state = SessionTrackerState {
        task_id: task_id.to_string(),
        turns_count,
        payload_bytes,
        estimated_tokens,
    };
    SessionTracker::save_for_session(workspace, session_id, &state)?;

    Ok(SessionHealth::new(
        turns_count,
        payload_bytes,
        estimated_tokens,
    ))
}

/// Records a turn interaction and updates session telemetry in `.xgauntlet/session_state.json`.
pub fn record_session_turn(
    workspace: &Path,
    task_id: &str,
    additional_payload_bytes: usize,
    explicit_tokens: Option<usize>,
) -> Result<SessionHealth, std::io::Error> {
    record_session_turn_for_session(
        workspace,
        task_id,
        "default",
        additional_payload_bytes,
        explicit_tokens,
    )
}

/// Resets session health state for the given task without wiping sessions bound to other tasks.
pub fn reset_session_health(workspace: &Path, task_id: &str) -> Result<(), std::io::Error> {
    let state_path = workspace.join(SESSION_STATE_FILE);
    if state_path.is_file() {
        if let Ok(Some(mut store)) = read_session_store(workspace) {
            store
                .sessions
                .retain(|_, s| !task_ids_match(&s.task_id, task_id));
            if store.sessions.is_empty() {
                fs::remove_file(&state_path)?;
            } else {
                write_session_store(workspace, &store)?;
            }
        } else {
            fs::remove_file(&state_path)?;
        }
    }

    let sessions_dir = workspace.join(SESSION_STATE_DIR);
    if sessions_dir.is_dir() {
        for entry in fs::read_dir(&sessions_dir)?.flatten() {
            let p = entry.path();
            if p.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            if let Ok(content) = fs::read_to_string(&p) {
                if let Ok(state) = serde_json::from_str::<SessionTrackerState>(&content) {
                    if task_ids_match(&state.task_id, task_id) {
                        let _ = fs::remove_file(&p);
                    }
                }
            }
        }
    }
    Ok(())
}

/// Records a monotonic session snapshot for a specific session identifier without cross-session interference.
pub fn record_session_snapshot_for_session(
    workspace: &Path,
    task_id: &str,
    session_id: &str,
    snapshot_turns: Option<usize>,
    snapshot_payload_bytes: Option<usize>,
    snapshot_tokens: Option<usize>,
) -> Result<SessionHealth, std::io::Error> {
    let existing = SessionTracker::load_for_session(workspace, task_id, Some(session_id))?;
    let turns_count = match (existing.as_ref(), snapshot_turns) {
        (Some(e), Some(t)) => e.turns_count.max(t),
        (None, Some(t)) => t.max(1),
        (Some(e), None) => e.turns_count + 1,
        (None, None) => 1,
    };
    let payload_bytes = match (existing.as_ref(), snapshot_payload_bytes) {
        (Some(e), Some(b)) => e.payload_bytes.max(b),
        (None, Some(b)) => b,
        (Some(e), None) => e.payload_bytes,
        (None, None) => 0,
    };
    let estimated_tokens = match (existing.as_ref(), snapshot_tokens) {
        (Some(e), Some(tok)) => e.estimated_tokens.max(tok),
        (None, Some(tok)) => tok,
        (Some(e), None) => e.estimated_tokens,
        (None, None) => estimate_tokens_from_chars(payload_bytes),
    };

    let state = SessionTrackerState {
        task_id: task_id.to_string(),
        turns_count,
        payload_bytes,
        estimated_tokens,
    };
    SessionTracker::save_for_session(workspace, session_id, &state)?;

    Ok(SessionHealth::new(
        turns_count,
        payload_bytes,
        estimated_tokens,
    ))
}

/// Records a monotonic session snapshot without quadratic accumulation across repeated hooks.
pub fn record_session_snapshot(
    workspace: &Path,
    task_id: &str,
    snapshot_turns: Option<usize>,
    snapshot_payload_bytes: Option<usize>,
    snapshot_tokens: Option<usize>,
) -> Result<SessionHealth, std::io::Error> {
    record_session_snapshot_for_session(
        workspace,
        task_id,
        "default",
        snapshot_turns,
        snapshot_payload_bytes,
        snapshot_tokens,
    )
}

fn json_value_char_len(val: &serde_json::Value) -> usize {
    match val {
        serde_json::Value::String(s) => s.chars().count(),
        serde_json::Value::Null => 0,
        other => other.to_string().chars().count(),
    }
}

fn is_model_transcript_step(val: &serde_json::Value) -> bool {
    let type_match = val
        .get("type")
        .and_then(|v| v.as_str())
        .map(|s| {
            s.eq_ignore_ascii_case("PLANNER_RESPONSE")
                || s.eq_ignore_ascii_case("assistant")
                || s.eq_ignore_ascii_case("model")
        })
        .unwrap_or(false);
    let role_match = val
        .get("role")
        .and_then(|v| v.as_str())
        .map(|s| s.eq_ignore_ascii_case("assistant") || s.eq_ignore_ascii_case("model"))
        .unwrap_or(false);
    let source_match = val
        .get("source")
        .and_then(|v| v.as_str())
        .map(|s| s.eq_ignore_ascii_case("MODEL"))
        .unwrap_or(false);
    type_match || role_match || source_match
}

fn extract_step_chars(val: &serde_json::Value) -> usize {
    let mut total = 0usize;
    for key in ["content", "thinking", "tool_calls"] {
        if let Some(field) = val.get(key) {
            total += json_value_char_len(field);
        }
    }
    total
}

/// Parses a local JSONL transcript file into `(turns_count, total_chars, estimated_tokens)`.
fn parse_transcript_snapshot(transcript_path: &Path) -> Option<(usize, usize, usize)> {
    if !transcript_path.is_file() {
        return None;
    }
    let content = fs::read_to_string(transcript_path).ok()?;
    let mut model_turns = 0usize;
    let mut model_chars = 0usize;
    let mut total_json_lines = 0usize;
    let mut all_chars = 0usize;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(trimmed) {
            total_json_lines += 1;
            let step_chars = extract_step_chars(&val);
            all_chars += step_chars;
            if is_model_transcript_step(&val) {
                model_turns += 1;
                model_chars += step_chars;
            }
        }
    }

    if model_turns > 0 {
        Some((
            model_turns,
            model_chars,
            estimate_tokens_from_chars(model_chars),
        ))
    } else if total_json_lines > 0 {
        Some((
            total_json_lines,
            all_chars,
            estimate_tokens_from_chars(all_chars),
        ))
    } else {
        None
    }
}

fn extract_explicit_snapshot_tokens(payload: &serde_json::Value) -> Option<usize> {
    payload
        .get("total_tokens")
        .or_else(|| payload.get("token_count"))
        .or_else(|| payload.get("tokens"))
        .or_else(|| payload.get("usage").and_then(|u| u.get("total_tokens")))
        .and_then(|v| v.as_u64())
        .map(|v| v as usize)
}

fn extract_transcript_path(payload: &serde_json::Value) -> Option<std::path::PathBuf> {
    payload
        .get("transcript_path")
        .or_else(|| payload.get("transcriptPath"))
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(std::path::PathBuf::from)
}

/// Records or enriches session health from a hook payload using snapshot semantics when
/// `transcript_path` or explicit token counts are available, falling back to payload bytes.
pub fn record_session_from_hook_payload(
    workspace: &Path,
    task_id: &str,
    payload: Option<&serde_json::Value>,
    raw_payload: &str,
) -> Result<SessionHealth, std::io::Error> {
    let session_id = extract_session_id(payload);
    if let Some(val) = payload {
        let transcript_snap = extract_transcript_path(val)
            .as_deref()
            .and_then(parse_transcript_snapshot);
        let explicit_tokens = extract_explicit_snapshot_tokens(val);

        if transcript_snap.is_some() || explicit_tokens.is_some() {
            let (snap_turns, snap_chars, snap_tokens) = match transcript_snap {
                Some((t, chars, tok)) => (Some(t), Some(chars.max(raw_payload.len())), Some(tok)),
                None => (None, Some(raw_payload.len()), None),
            };
            let effective_tokens = explicit_tokens.or(snap_tokens);
            return record_session_snapshot_for_session(
                workspace,
                task_id,
                &session_id,
                snap_turns,
                snap_chars,
                effective_tokens,
            );
        }
    }

    record_session_turn_for_session(workspace, task_id, &session_id, raw_payload.len(), None)
}

/// Synchronizes session telemetry on `PreInvocation` only when an active task exists.
pub fn sync_active_session_for_pre_invocation(
    workspace: &Path,
    payload: Option<&serde_json::Value>,
    raw_payload: &str,
) -> Option<SessionHealth> {
    let active_task_id = crate::features::tasks::resolve_active_task_id(workspace)?;
    record_session_from_hook_payload(workspace, &active_task_id, payload, raw_payload).ok()
}
