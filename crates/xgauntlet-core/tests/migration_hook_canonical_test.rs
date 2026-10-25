//! Acceptance tests for Task 048: Canonical Antigravity Hook in Migration.
//!
//! Verifies:
//! - AC1: Single source of truth for the Antigravity PreInvocation command.
//! - AC2: Migration injects canonical command `xgauntlet telemetry --format antigravity-hook`.
//! - AC3: Migration normalizes defective `--harness antigravity-hook` and `--harness antigravity`
//!   and is idempotent.
//! - AC4: Simultaneous legacy and modern groups are merged without clobbering or data loss.
//! - AC5: `doctor --fix` reports migration failures truthfully instead of swallowing them.

use std::fs;
use std::path::{Path, PathBuf};

use xgauntlet_core::features::doctor::models::{DoctorOptions, DoctorVerdict};
use xgauntlet_core::features::doctor::run_doctor;
use xgauntlet_core::features::migration::{run_migration, MigrationOptions};

static TEST_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(100);

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(prefix: &str) -> Self {
        let count = TEST_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
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
        let path = std::env::temp_dir().join(format!("xgauntlet_mig_canon_test_{}", unique));
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

const EXPECTED_CANONICAL_PRE_INVOCATION: &str = "xgauntlet telemetry --format antigravity-hook";

/// AC2: Migration of hooks.json without telemetry-hook injects precisely the canonical command.
#[test]
fn test_migration_injects_canonical_telemetry_format() {
    let temp = TempDir::new("mig_ac2_inject");
    let ws = temp.path();

    let agents_dir = ws.join(".agents");
    fs::create_dir_all(&agents_dir).unwrap();

    let initial_hooks = r#"{
  "agent-gauntlet-gatekeeper": {
    "enabled": true,
    "PreToolUse": [
      {
        "matcher": "run_command",
        "hooks": [
          { "command": "agent_gauntlet hook antigravity" }
        ]
      }
    ]
  }
}"#;
    fs::write(agents_dir.join("hooks.json"), initial_hooks).unwrap();

    let options = MigrationOptions {
        workspace: ws.to_path_buf(),
        dry_run: false,
    };

    let report = run_migration(&options).expect("Migration must succeed");
    assert!(report.is_success);

    let migrated_raw = fs::read_to_string(agents_dir.join("hooks.json")).unwrap();
    let migrated: serde_json::Value = serde_json::from_str(&migrated_raw).unwrap();

    let pre_inv = migrated
        .get("xgauntlet")
        .and_then(|x| x.get("PreInvocation"))
        .and_then(|p| p.as_array())
        .expect("xgauntlet.PreInvocation must be an array");

    assert_eq!(
        pre_inv.len(),
        1,
        "PreInvocation should contain exactly 1 hook"
    );
    let cmd = pre_inv[0]
        .get("command")
        .and_then(|c| c.as_str())
        .expect("PreInvocation hook must have command string");

    assert_eq!(
        cmd, EXPECTED_CANONICAL_PRE_INVOCATION,
        "Injected PreInvocation command must match canonical format"
    );
}

/// AC3: Migration of hooks.json with --harness antigravity-hook normalizes to canonical format and is idempotent.
#[test]
fn test_migration_normalizes_defective_harness_hook_and_is_idempotent() {
    let temp = TempDir::new("mig_ac3_norm");
    let ws = temp.path();

    let agents_dir = ws.join(".agents");
    fs::create_dir_all(&agents_dir).unwrap();

    let defective_hooks = r#"{
  "xgauntlet": {
    "enabled": true,
    "PreToolUse": [
      {
        "matcher": "run_command",
        "hooks": [
          { "command": "xgauntlet hook antigravity" }
        ]
      }
    ],
    "PreInvocation": [
      {
        "command": "xgauntlet telemetry --harness antigravity-hook"
      }
    ]
  }
}"#;
    fs::write(agents_dir.join("hooks.json"), defective_hooks).unwrap();

    let options = MigrationOptions {
        workspace: ws.to_path_buf(),
        dry_run: false,
    };

    // First migration: must detect defective flag and normalize it
    let report1 = run_migration(&options).expect("First migration must succeed");
    assert!(report1.is_success);
    assert!(
        !report1.files_modified.is_empty(),
        "First migration must report modifications for defective hooks"
    );

    let hooks_item = report1
        .items
        .iter()
        .find(|item| item.file == PathBuf::from(".agents/hooks.json"))
        .expect("Report must include .agents/hooks.json item");

    assert!(
        hooks_item
            .changes
            .iter()
            .any(|c| c.contains("telemetry") && c.contains("antigravity-hook")),
        "Changes must explicitly describe normalizing telemetry hook: {:?}",
        hooks_item.changes
    );

    let migrated_raw = fs::read_to_string(agents_dir.join("hooks.json")).unwrap();
    let migrated: serde_json::Value = serde_json::from_str(&migrated_raw).unwrap();

    let pre_inv = migrated
        .get("xgauntlet")
        .and_then(|x| x.get("PreInvocation"))
        .and_then(|p| p.as_array())
        .expect("xgauntlet.PreInvocation must be an array");

    let cmd = pre_inv[0]
        .get("command")
        .and_then(|c| c.as_str())
        .expect("PreInvocation hook must have command string");

    assert_eq!(
        cmd, EXPECTED_CANONICAL_PRE_INVOCATION,
        "Normalized command must use --format antigravity-hook"
    );

    // Second migration: must be idempotent and report 0 modifications
    let report2 = run_migration(&options).expect("Second migration must succeed");
    assert_eq!(
        report2.files_modified.len(),
        0,
        "Second migration must be idempotent with 0 items modified"
    );
}

/// AC3 (variant): Migration of hooks.json with --harness antigravity normalizes to canonical format.
#[test]
fn test_migration_normalizes_legacy_harness_antigravity() {
    let temp = TempDir::new("mig_ac3_legacy_harness");
    let ws = temp.path();

    let agents_dir = ws.join(".agents");
    fs::create_dir_all(&agents_dir).unwrap();

    let defective_hooks = r#"{
  "xgauntlet": {
    "enabled": true,
    "PreInvocation": [
      {
        "command": "xgauntlet telemetry --harness antigravity"
      }
    ]
  }
}"#;
    fs::write(agents_dir.join("hooks.json"), defective_hooks).unwrap();

    let options = MigrationOptions {
        workspace: ws.to_path_buf(),
        dry_run: false,
    };

    let report = run_migration(&options).expect("Migration must succeed");
    assert!(report.is_success);

    let migrated_raw = fs::read_to_string(agents_dir.join("hooks.json")).unwrap();
    let migrated: serde_json::Value = serde_json::from_str(&migrated_raw).unwrap();

    let pre_inv = migrated
        .get("xgauntlet")
        .and_then(|x| x.get("PreInvocation"))
        .and_then(|p| p.as_array())
        .expect("xgauntlet.PreInvocation must be an array");

    let cmd = pre_inv[0].get("command").and_then(|c| c.as_str()).unwrap();

    assert_eq!(cmd, EXPECTED_CANONICAL_PRE_INVOCATION);
}

/// AC4: Simultaneous presence of agent-gauntlet-gatekeeper and xgauntlet merges hooks without data loss or clobbering.
#[test]
fn test_migration_merges_simultaneous_legacy_and_modern_groups_without_data_loss() {
    let temp = TempDir::new("mig_ac4_merge");
    let ws = temp.path();

    let agents_dir = ws.join(".agents");
    fs::create_dir_all(&agents_dir).unwrap();

    let mixed_hooks = r#"{
  "agent-gauntlet-gatekeeper": {
    "enabled": true,
    "PreToolUse": [
      {
        "matcher": "custom_special_tool",
        "hooks": [
          { "command": "echo custom" }
        ]
      }
    ]
  },
  "xgauntlet": {
    "enabled": true,
    "PreToolUse": [
      {
        "matcher": "run_command",
        "hooks": [
          { "command": "xgauntlet hook antigravity" }
        ]
      }
    ],
    "PreInvocation": [
      {
        "command": "xgauntlet telemetry --format antigravity-hook"
      }
    ]
  }
}"#;
    fs::write(agents_dir.join("hooks.json"), mixed_hooks).unwrap();

    let options = MigrationOptions {
        workspace: ws.to_path_buf(),
        dry_run: false,
    };

    let report = run_migration(&options).expect("Migration must succeed");
    assert!(report.is_success);

    let hooks_item = report
        .items
        .iter()
        .find(|item| item.file == PathBuf::from(".agents/hooks.json"))
        .expect("Report must include .agents/hooks.json item");

    assert!(
        hooks_item
            .changes
            .iter()
            .any(|c| c.to_lowercase().contains("merge")),
        "Changes list must explicitly mention merging legacy and modern hook configurations: {:?}",
        hooks_item.changes
    );

    let migrated_raw = fs::read_to_string(agents_dir.join("hooks.json")).unwrap();
    let migrated: serde_json::Value = serde_json::from_str(&migrated_raw).unwrap();

    assert!(
        migrated.get("agent-gauntlet-gatekeeper").is_none(),
        "Legacy group must be removed after merge"
    );

    let xg = migrated
        .get("xgauntlet")
        .expect("xgauntlet group must exist");
    let pre_tool = xg
        .get("PreToolUse")
        .and_then(|p| p.as_array())
        .expect("PreToolUse must be an array");

    let matchers: Vec<&str> = pre_tool
        .iter()
        .filter_map(|item| item.get("matcher").and_then(|m| m.as_str()))
        .collect();

    assert!(
        matchers.contains(&"custom_special_tool"),
        "Custom tool matcher from legacy group must be preserved"
    );
    assert!(
        matchers.contains(&"run_command"),
        "Standard run_command matcher from xgauntlet group must be preserved"
    );

    let pre_inv = xg
        .get("PreInvocation")
        .and_then(|p| p.as_array())
        .expect("PreInvocation must be an array");
    assert_eq!(
        pre_inv.len(),
        1,
        "PreInvocation must retain exactly 1 entry without clobbering"
    );
}

/// AC5: doctor --fix must report migration failures truthfully instead of silently swallowing them.
#[test]
fn test_doctor_fix_reports_migration_errors_truthfully() {
    let temp = TempDir::new("mig_ac5_doctor_fix");
    let ws = temp.path();

    let agents_dir = ws.join(".agents");
    fs::create_dir_all(&agents_dir).unwrap();

    // Write corrupted JSON that causes run_migration to fail
    fs::write(agents_dir.join("hooks.json"), "{ invalid json: 1234").unwrap();

    let options = DoctorOptions {
        workspace: ws.to_path_buf(),
        fix: true,
        strict: false,
        verbose: false,
        category: None,
    };

    let report = run_doctor(&options).expect("run_doctor should return a report");

    assert_eq!(
        report.verdict,
        DoctorVerdict::Critical,
        "Doctor verdict must be Critical when auto-fix migration fails"
    );
    assert!(
        report.failed_count > 0,
        "Doctor must report at least 1 failed check for migration error"
    );

    let failed_migration_check = report.checks.iter().find(|c| {
        c.message.to_lowercase().contains("migration")
            || c.name.to_lowercase().contains("migration")
            || c.detail
                .as_ref()
                .map(|d| d.to_lowercase().contains("migration"))
                .unwrap_or(false)
    });

    assert!(
        failed_migration_check.is_some(),
        "Doctor checks must contain a failed item reporting the migration error: {:?}",
        report.checks
    );
}

/// AC1: Single source of truth constant in adapters::antigravity is reused.
#[test]
fn test_canonical_pre_invocation_cmd_single_source_of_truth() {
    use xgauntlet_core::features::adapters::antigravity::{
        AntigravityAdapter, ANTIGRAVITY_PRE_INVOCATION_CMD, CANONICAL_PRE_INVOCATION_CMD,
    };

    assert_eq!(
        CANONICAL_PRE_INVOCATION_CMD,
        EXPECTED_CANONICAL_PRE_INVOCATION
    );
    assert_eq!(
        ANTIGRAVITY_PRE_INVOCATION_CMD,
        EXPECTED_CANONICAL_PRE_INVOCATION
    );
    assert_eq!(
        AntigravityAdapter::CANONICAL_PRE_INVOCATION_CMD,
        EXPECTED_CANONICAL_PRE_INVOCATION
    );
}
