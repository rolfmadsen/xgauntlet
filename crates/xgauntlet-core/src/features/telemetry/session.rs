//! Session Health, Turn & Token Telemetry engine.
//!
//! Provides deterministic session telemetry (turns count, accumulated payload bytes,
//! and estimated token saturation) without requiring external tokenizer network calls
//! or background daemons.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

pub const SESSION_STATE_FILE: &str = ".xgauntlet/session_state.json";

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
            SessionHealthStatus::Aging => {
                format!(
                    "{} ({} {} • {}) • Forbered Handoff",
                    self.status.badge(),
                    self.turns_count,
                    turn_label,
                    tok_label
                )
            }
            SessionHealthStatus::Saturated => {
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

/// File-based session state tracker.
pub struct SessionTracker;

impl SessionTracker {
    /// Loads active session health if the recorded state matches the requested task_id.
    pub fn load(workspace: &Path, task_id: &str) -> Result<Option<SessionHealth>, std::io::Error> {
        let state_path = workspace.join(SESSION_STATE_FILE);
        if !state_path.is_file() {
            return Ok(None);
        }

        let content = fs::read_to_string(&state_path)?;
        let state: SessionTrackerState = match serde_json::from_str(&content) {
            Ok(s) => s,
            Err(_) => return Ok(None),
        };

        let short_target = crate::features::checkpoint::extract_short_task_id(task_id);
        let short_stored = crate::features::checkpoint::extract_short_task_id(&state.task_id);

        if state.task_id == task_id || (!short_target.is_empty() && short_target == short_stored) {
            Ok(Some(SessionHealth::new(
                state.turns_count,
                state.payload_bytes,
                state.estimated_tokens,
            )))
        } else {
            Ok(None)
        }
    }

    /// Persists session tracking state to `.xgauntlet/session_state.json`.
    pub fn save(workspace: &Path, state: &SessionTrackerState) -> Result<(), std::io::Error> {
        let state_path = workspace.join(SESSION_STATE_FILE);
        if let Some(parent) = state_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(state).map_err(std::io::Error::other)?;
        fs::write(state_path, json)?;
        Ok(())
    }
}

/// Records a turn interaction and updates session telemetry in `.xgauntlet/session_state.json`.
pub fn record_session_turn(
    workspace: &Path,
    task_id: &str,
    additional_payload_bytes: usize,
    explicit_tokens: Option<usize>,
) -> Result<SessionHealth, std::io::Error> {
    let existing = SessionTracker::load(workspace, task_id)?;
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
    SessionTracker::save(workspace, &state)?;

    Ok(SessionHealth::new(
        turns_count,
        payload_bytes,
        estimated_tokens,
    ))
}

/// Resets session health state for the active task.
pub fn reset_session_health(workspace: &Path, _task_id: &str) -> Result<(), std::io::Error> {
    let state_path = workspace.join(SESSION_STATE_FILE);
    if state_path.exists() {
        fs::remove_file(state_path)?;
    }
    Ok(())
}
