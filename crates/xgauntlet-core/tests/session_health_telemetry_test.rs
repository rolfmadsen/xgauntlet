//! Tests for Task 044: Session Health, Turn & Token Telemetry.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use xgauntlet_core::features::adapters::antigravity::AntigravityAdapter;
use xgauntlet_core::features::tasks::models::TaskStatus;
use xgauntlet_core::features::tasks::telemetry::{CriteriaProgress, GitTelemetry, TaskTelemetry};
use xgauntlet_core::features::telemetry::{
    estimate_tokens_from_chars, estimate_tokens_from_str, record_session_turn,
    reset_session_health, SessionHealth, SessionHealthStatus, SessionTracker,
};

static TEST_COUNTER: AtomicU64 = AtomicU64::new(1);

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(prefix: &str) -> Self {
        let count = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let unique = format!(
            "{}_{}_{}_{}",
            prefix,
            std::process::id(),
            count,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let path = std::env::temp_dir().join(format!("xgauntlet_telemetry_test_{}", unique));
        fs::create_dir_all(&path).unwrap();
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[test]
fn test_session_health_status_transitions() {
    assert_eq!(
        SessionHealthStatus::from_turns(0),
        SessionHealthStatus::Optimal
    );
    assert_eq!(
        SessionHealthStatus::from_turns(1),
        SessionHealthStatus::Optimal
    );
    assert_eq!(
        SessionHealthStatus::from_turns(10),
        SessionHealthStatus::Optimal
    );
    assert_eq!(
        SessionHealthStatus::from_turns(11),
        SessionHealthStatus::Aging
    );
    assert_eq!(
        SessionHealthStatus::from_turns(18),
        SessionHealthStatus::Aging
    );
    assert_eq!(
        SessionHealthStatus::from_turns(19),
        SessionHealthStatus::Saturated
    );
    assert_eq!(
        SessionHealthStatus::from_turns(100),
        SessionHealthStatus::Saturated
    );

    assert_eq!(SessionHealthStatus::Optimal.as_str(), "Optimal");
    assert_eq!(SessionHealthStatus::Aging.as_str(), "Aging");
    assert_eq!(SessionHealthStatus::Saturated.as_str(), "Saturated");

    assert_eq!(SessionHealthStatus::Optimal.badge(), "🟢 Optimal");
    assert_eq!(SessionHealthStatus::Aging.badge(), "🟡 Aging");
    assert_eq!(SessionHealthStatus::Saturated.badge(), "🔴 Saturated");
}

#[test]
fn test_token_estimation_deterministic() {
    assert_eq!(estimate_tokens_from_chars(0), 0);
    assert_eq!(estimate_tokens_from_str(""), 0);

    // ~1 token ≈ 3.8 chars. 38 characters -> 10 tokens
    let text_38 = "12345678901234567890123456789012345678";
    assert_eq!(text_38.len(), 38);
    assert_eq!(estimate_tokens_from_chars(38), 10);
    assert_eq!(estimate_tokens_from_str(text_38), 10);

    // 380 chars -> 100 tokens
    let text_380 = "a".repeat(380);
    assert_eq!(estimate_tokens_from_str(&text_380), 100);
}

#[test]
fn test_session_health_summary_formatting() {
    let optimal = SessionHealth::new(4, 15_200, 4_000);
    assert_eq!(optimal.status, SessionHealthStatus::Optimal);
    let summary_optimal = optimal.format_hud_summary();
    assert!(
        summary_optimal.contains("🟢 Optimal") && summary_optimal.contains("4 turns"),
        "expected optimal summary, got: {}",
        summary_optimal
    );

    let aging = SessionHealth::new(14, 150_000, 39_473);
    assert_eq!(aging.status, SessionHealthStatus::Aging);
    let summary_aging = aging.format_hud_summary();
    assert!(
        summary_aging.contains("🟡 Aging")
            && summary_aging.contains("14 turns")
            && summary_aging.contains("Handoff"),
        "expected aging summary with handoff notice, got: {}",
        summary_aging
    );

    let saturated = SessionHealth::new(21, 300_000, 78_947);
    assert_eq!(saturated.status, SessionHealthStatus::Saturated);
    let summary_saturated = saturated.format_hud_summary();
    assert!(
        summary_saturated.contains("🔴 Saturated")
            && summary_saturated.contains("21 turns")
            && summary_saturated.contains("Handoff"),
        "expected saturated summary, got: {}",
        summary_saturated
    );
}

#[test]
fn test_session_tracker_persistence() {
    let temp = TempDir::new("tracker");
    let root = temp.path();

    // Initial state: no turns recorded
    let initial = SessionTracker::load(root, "044").unwrap();
    assert!(initial.is_none());

    // Record turn 1: 1,500 bytes payload
    let h1 = record_session_turn(root, "044", 1_500, None).unwrap();
    assert_eq!(h1.turns_count, 1);
    assert_eq!(h1.payload_bytes, 1_500);
    assert_eq!(h1.status, SessionHealthStatus::Optimal);

    // Verify file created at .xgauntlet/session_state.json
    let session_file = root.join(".xgauntlet/session_state.json");
    assert!(session_file.exists());
    let raw = fs::read_to_string(&session_file).unwrap();
    assert!(raw.contains("\"task_id\": \"044\""));

    // Record turn 2: 2,500 bytes payload
    let h2 = record_session_turn(root, "044", 2_500, None).unwrap();
    assert_eq!(h2.turns_count, 2);
    assert_eq!(h2.payload_bytes, 4_000);

    // Reset session health
    reset_session_health(root, "044").unwrap();
    let after_reset = SessionTracker::load(root, "044").unwrap();
    assert!(after_reset.is_none());
}

#[test]
fn test_task_telemetry_integration_and_rendering() {
    let git = GitTelemetry {
        branch: "main".to_string(),
        head_oid: "d548d5a".to_string(),
        dirty_count: 0,
        is_clean: true,
    };
    let criteria = CriteriaProgress {
        total: 6,
        completed: 0,
        pending: 6,
        percentage: 0,
        bar: "[□□□□□]".to_string(),
    };
    let health = SessionHealth::new(14, 60_000, 15_789);
    let telemetry = TaskTelemetry {
        task_id: "044-session-health-turn-and-token-telemetry".to_string(),
        title: "Session Health, Turn & Token Telemetry".to_string(),
        status: TaskStatus::Active,
        intent: Some("🚀 NEW FEATURE".to_string()),
        criteria,
        git,
        file_path: "tasks/044-session-health-turn-and-token-telemetry.md".to_string(),
        scope: Some("crates/*".to_string()),
        invariants: Some("14/14 PASS".to_string()),
        evidence: Some("pending".to_string()),
        phase: Some("SPEC".to_string()),
        session_health: Some(health),
    };

    // Verify HUD includes Session:
    let hud = AntigravityAdapter::render_blockquote_hud(&telemetry, None);
    assert!(
        hud.contains("Session: 🟡 Aging"),
        "expected HUD to contain session health badge, got:\n{}",
        hud
    );

    // Verify ephemeral telemetry includes Session info
    let ephem = AntigravityAdapter::format_ephemeral_telemetry(&telemetry);
    assert!(
        ephem.contains("Session: 🟡 Aging (14 turns"),
        "expected ephemeral telemetry to contain session health, got:\n{}",
        ephem
    );

    // Verify JSON serialization includes session_health field
    let json_val = serde_json::to_value(&telemetry).unwrap();
    assert!(json_val.get("session_health").is_some());
    assert_eq!(json_val["session_health"]["status"], "Aging");
}
