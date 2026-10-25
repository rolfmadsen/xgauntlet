//! Tests for Task 045: Multi-Session Lineage & Evidence Traceability.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use xgauntlet_core::features::checkpoint::{run_checkpoint, CheckpointOptions, CheckpointPhase};
use xgauntlet_core::features::evidence::lineage::{
    LineageError, SessionLineageTrail, GENESIS_DIGEST,
};
use xgauntlet_core::features::evidence::models::VerificationReport;
use xgauntlet_core::features::evidence::report::{
    generate_report_json, generate_report_markdown, load_report_json, load_verification_report,
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
        let path = std::env::temp_dir().join(format!("xgauntlet_lineage_test_{}", unique));
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
fn test_session_lineage_trail_genesis_and_chaining() {
    let mut trail = SessionLineageTrail::new();
    assert!(trail.is_empty());
    assert_eq!(trail.len(), 0);

    // 1. Genesis Entry (SPEC phase)
    let entry1 = trail.append(
        "session-spec-001",
        "System Architect",
        "SPEC",
        "0e5ff17b8a17856d42721c97652ba00c685785c9",
        Some("2026-10-25T14:51:15Z".to_string()),
    );

    assert_eq!(entry1.session_nonce, "session-spec-001");
    assert_eq!(entry1.role, "System Architect");
    assert_eq!(entry1.phase, "SPEC");
    assert_eq!(
        entry1.commit_oid,
        "0e5ff17b8a17856d42721c97652ba00c685785c9"
    );
    assert_eq!(entry1.timestamp, "2026-10-25T14:51:15Z");
    assert_eq!(entry1.digest.len(), 64);
    assert_ne!(entry1.digest, GENESIS_DIGEST);

    assert_eq!(trail.len(), 1);
    assert!(!trail.is_empty());
    assert!(trail.verify_integrity().is_ok());

    // 2. Chained Entry (RED phase in fresh session)
    let entry2 = trail.append(
        "session-tdd-002",
        "TDD Craftsman",
        "RED",
        "4290150d8ddc9000e3cf6874668940505b28bac2",
        Some("2026-10-25T14:55:00Z".to_string()),
    );

    assert_eq!(entry2.session_nonce, "session-tdd-002");
    assert_eq!(entry2.role, "TDD Craftsman");
    assert_eq!(entry2.phase, "RED");
    assert_ne!(entry2.digest, entry1.digest);
    assert_eq!(entry2.digest.len(), 64);

    assert_eq!(trail.len(), 2);
    assert!(trail.verify_integrity().is_ok());

    // 3. Chained Entry (GREEN phase in same session)
    let entry3 = trail.append(
        "session-tdd-002",
        "TDD Craftsman",
        "GREEN",
        "9248f0ed8ddc9000e3cf6874668940505b28bac2",
        Some("2026-10-25T15:10:00Z".to_string()),
    );

    assert_eq!(entry3.phase, "GREEN");
    assert_eq!(trail.len(), 3);
    assert!(trail.verify_integrity().is_ok());
}

#[test]
fn test_session_lineage_tamper_detection() {
    let mut trail = SessionLineageTrail::new();
    trail.append(
        "session-1",
        "System Architect",
        "SPEC",
        "commit111111111111111111111111111111111111",
        Some("2026-10-25T12:00:00Z".to_string()),
    );
    trail.append(
        "session-2",
        "TDD Craftsman",
        "RED",
        "commit222222222222222222222222222222222222",
        Some("2026-10-25T12:30:00Z".to_string()),
    );
    trail.append(
        "session-2",
        "TDD Craftsman",
        "GREEN",
        "commit333333333333333333333333333333333333",
        Some("2026-10-25T13:00:00Z".to_string()),
    );

    assert!(trail.verify_integrity().is_ok());

    // Case A: Tamper with Genesis entry commit OID
    let mut tampered_trail_a = trail.clone();
    tampered_trail_a.entries[0].commit_oid = "hacked111111111111111111111111111111111111".to_string();
    let res_a = tampered_trail_a.verify_integrity();
    assert!(
        matches!(res_a, Err(LineageError::TamperDetected { index: 0, .. })),
        "expected TamperDetected at index 0, got: {res_a:?}"
    );

    // Case B: Tamper with intermediate entry role
    let mut tampered_trail_b = trail.clone();
    tampered_trail_b.entries[1].role = "Malicious Imposter".to_string();
    let res_b = tampered_trail_b.verify_integrity();
    assert!(
        matches!(res_b, Err(LineageError::TamperDetected { index: 1, .. })),
        "expected TamperDetected at index 1, got: {res_b:?}"
    );

    // Case C: Tamper with entry digest
    let mut tampered_trail_c = trail.clone();
    tampered_trail_c.entries[1].digest = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string();
    let res_c = tampered_trail_c.verify_integrity();
    assert!(
        matches!(res_c, Err(LineageError::TamperDetected { .. })),
        "expected TamperDetected on forged digest, got: {res_c:?}"
    );
}

#[test]
fn test_verification_report_backward_compatibility() {
    // Deserialization of older report WITHOUT lineage field must succeed and set lineage to None
    let legacy_json = r#"{
        "$schema": "https://xgauntlet.dev/schemas/v2/verification-report.json",
        "schema_version": "2.0.0",
        "execution_origin": "LOCAL",
        "verdict": "PASSED",
        "task_contract": {
            "task_id": "044"
        },
        "workspace_state": {},
        "execution_metadata": {},
        "checks": [],
        "diagnostics": []
    }"#;

    let report: VerificationReport = load_report_json(legacy_json).expect("legacy report loads cleanly");
    assert!(report.lineage.is_none());

    // Serialization with lineage should include lineage field
    let mut trail = SessionLineageTrail::new();
    trail.append(
        "nonce-test",
        "System Architect",
        "SPEC",
        "commit1234",
        Some("2026-10-25T14:00:00Z".to_string()),
    );
    let mut report_with_lineage = report.clone();
    report_with_lineage.lineage = Some(trail);

    let json_with_lineage = generate_report_json(&report_with_lineage).expect("serializes cleanly");
    assert!(json_with_lineage.contains("\"lineage\""));
    assert!(json_with_lineage.contains("nonce-test"));

    let reloaded: VerificationReport = load_report_json(&json_with_lineage).expect("round-trips");
    let reloaded_trail = reloaded.lineage.expect("lineage is present");
    assert_eq!(reloaded_trail.len(), 1);
    assert_eq!(reloaded_trail.entries[0].session_nonce, "nonce-test");
    assert!(reloaded_trail.verify_integrity().is_ok());
}

#[test]
fn test_evidence_markdown_renders_lineage_table() {
    let mut trail = SessionLineageTrail::new();
    trail.append(
        "sess-alpha",
        "System Architect",
        "SPEC",
        "0e5ff17b8a17856d42721c97652ba00c685785c9",
        Some("2026-10-25T14:51:15Z".to_string()),
    );
    trail.append(
        "sess-beta",
        "TDD Craftsman",
        "RED",
        "4290150d8ddc9000e3cf6874668940505b28bac2",
        Some("2026-10-25T14:55:00Z".to_string()),
    );

    let mut report = VerificationReport::new_local("045");
    report.lineage = Some(trail);

    let markdown = generate_report_markdown(&report, None);
    assert!(
        markdown.contains("### 📜 Multi-Session Audit Lineage"),
        "expected markdown to contain lineage section, got:\n{}",
        markdown
    );
    assert!(
        markdown.contains("| Session | Rolle | Fase | Commit | Tidsstempel | Status |"),
        "expected table header, got:\n{}",
        markdown
    );
    assert!(
        markdown.contains("`sess-alpha`") && markdown.contains("`System Architect`") && markdown.contains("`SPEC`"),
        "expected first entry in markdown table"
    );
    assert!(
        markdown.contains("`sess-beta`") && markdown.contains("`TDD Craftsman`") && markdown.contains("`RED`"),
        "expected second entry in markdown table"
    );
}

#[tokio::test]
async fn test_checkpoint_engine_appends_lineage() {
    let temp = TempDir::new("checkpoint_lineage");
    let ws = temp.path();

    // 1. Initialize git repo
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(ws)
        .status()
        .unwrap();
    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(ws)
        .status()
        .unwrap();
    std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(ws)
        .status()
        .unwrap();

    // 2. Create minimal task and file
    let tasks_dir = ws.join("tasks");
    fs::create_dir_all(&tasks_dir).unwrap();
    let task_content = r#"---
type: Task Package
title: "Task 045: Lineage Test"
description: "Test task"
status: active
generated: { by: test, at: "2026-10-25T00:00:00Z" }
tags: [test]
scope: ["src/**"]
test_refs: ["tests/**"]
---
# Task 045: Lineage Test
**Status**: `ACTIVE`
## 🎯 Formål
Test purpose
## 📋 Acceptance Criteria
- [ ] Criterion 1
## 🚫 Must NOT
- None
"#;
    fs::write(tasks_dir.join("045-lineage-test.md"), task_content).unwrap();

    let src_dir = ws.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(src_dir.join("lib.rs"), "// Initial code").unwrap();

    // 3. Run checkpoint with phase = Spec
    let options = CheckpointOptions::new(CheckpointPhase::Spec, ws)
        .with_task_id("045-lineage-test")
        .with_skip_verify(true);

    let res = run_checkpoint(&options).await.expect("checkpoint succeeds");
    assert!(res.commit_oid.is_some());

    // 4. Verify verification-report.json was updated with lineage trail
    let report = load_verification_report(ws).expect("report exists on disk");
    let lineage = report.lineage.expect("lineage trail was recorded");
    assert_eq!(lineage.len(), 1);
    let entry = &lineage.entries[0];
    assert_eq!(entry.phase, "SPEC");
    assert_eq!(entry.role, "System Architect");
    assert_eq!(Some(&entry.commit_oid), res.commit_oid.as_ref());
    assert!(lineage.verify_integrity().is_ok());
}
