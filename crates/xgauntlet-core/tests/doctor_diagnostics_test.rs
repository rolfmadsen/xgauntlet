//! Integration tests for Doctor depth inspection and auto-remediation (Task 026 AC1 & AC3).

use std::fs;
use std::path::{Path, PathBuf};

use xgauntlet_core::features::doctor::{
    run_doctor, DoctorCategory, DoctorCheckStatus, DoctorOptions,
};

static TEST_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

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
        let path = std::env::temp_dir().join(format!("xgauntlet_doc_diag_{}", unique));
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
fn test_doctor_warns_on_legacy_hooks_namespace_and_missing_preinvocation() {
    let temp = TempDir::new("doc_legacy_hooks");
    let ws = temp.path();

    let agents_dir = ws.join(".agents");
    fs::create_dir_all(&agents_dir).unwrap();
    fs::write(agents_dir.join("AGENTS.md"), "# Guidelines").unwrap();

    // Legacy hooks without PreInvocation and using obsolete namespace
    let legacy_hooks = r#"{
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
    fs::write(agents_dir.join("hooks.json"), legacy_hooks).unwrap();

    let options = DoctorOptions {
        workspace: ws.to_path_buf(),
        verbose: true,
        strict: false,
        category: Some(DoctorCategory::Governance),
        fix: false,
    };

    let report = run_doctor(&options).expect("Doctor check should succeed");

    let harness_check = report
        .checks
        .iter()
        .find(|c| c.name == "governance_agent_harness")
        .expect("governance_agent_harness check must be present");

    assert_eq!(
        harness_check.status,
        DoctorCheckStatus::Warn,
        "Doctor must issue a WARN for legacy hooks namespace and missing PreInvocation"
    );

    let remediation = harness_check.remediation.as_deref().unwrap_or("");
    assert!(
        remediation.contains("xgauntlet migrate") || remediation.contains("xgauntlet doctor --fix"),
        "Remediation must advise running 'xgauntlet migrate' or 'xgauntlet doctor --fix'. Got: {remediation}"
    );
}

#[test]
fn test_doctor_warns_on_legacy_evidence_file_in_gauntlet_toml() {
    let temp = TempDir::new("doc_legacy_evidence");
    let ws = temp.path();

    let gauntlet_toml = r#"
stack = "rust"
save_evidence = true
evidence_file = "evidence.json"

[[layers]]
name = "unit"
command = ["cargo", "test"]
"#;
    fs::write(ws.join("gauntlet.toml"), gauntlet_toml).unwrap();

    let options = DoctorOptions {
        workspace: ws.to_path_buf(),
        verbose: true,
        strict: false,
        category: Some(DoctorCategory::Governance),
        fix: false,
    };

    let report = run_doctor(&options).expect("Doctor check should succeed");

    let toml_check = report
        .checks
        .iter()
        .find(|c| c.name == "governance_gauntlet_toml")
        .expect("governance_gauntlet_toml check must be present");

    assert_eq!(
        toml_check.status,
        DoctorCheckStatus::Warn,
        "Doctor must issue a WARN when gauntlet.toml references legacy evidence_file = 'evidence.json'"
    );

    let remediation = toml_check.remediation.as_deref().unwrap_or("");
    assert!(
        remediation.contains("xgauntlet migrate") || remediation.contains("xgauntlet doctor --fix"),
        "Remediation must advise running 'xgauntlet migrate' or 'xgauntlet doctor --fix'. Got: {remediation}"
    );
}

#[test]
fn test_doctor_passes_when_hooks_and_evidence_are_modern() {
    let temp = TempDir::new("doc_modern_gov");
    let ws = temp.path();

    let agents_dir = ws.join(".agents");
    fs::create_dir_all(&agents_dir).unwrap();
    fs::create_dir_all(ws.join("tasks")).unwrap();
    fs::write(
        ws.join("CONTEXT.md"),
        "# Glossary\n\n### Term\nIs a concept.\n",
    )
    .unwrap();
    fs::write(agents_dir.join("AGENTS.md"), "# Guidelines").unwrap();

    let modern_hooks = r#"{
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
    fs::write(agents_dir.join("hooks.json"), modern_hooks).unwrap();

    let modern_toml = r#"
stack = "rust"
save_evidence = true
evidence_file = "verification-report.json"

[[layers]]
name = "unit"
command = ["cargo", "test"]
"#;
    fs::write(ws.join("gauntlet.toml"), modern_toml).unwrap();

    let options = DoctorOptions {
        workspace: ws.to_path_buf(),
        verbose: true,
        strict: false,
        category: Some(DoctorCategory::Governance),
        fix: false,
    };

    let report = run_doctor(&options).expect("Doctor check should succeed");

    let harness_check = report
        .checks
        .iter()
        .find(|c| c.name == "governance_agent_harness")
        .expect("governance_agent_harness check must be present");
    assert_eq!(
        harness_check.status,
        DoctorCheckStatus::Pass,
        "Modern hooks must pass doctor check"
    );

    let toml_check = report
        .checks
        .iter()
        .find(|c| c.name == "governance_gauntlet_toml")
        .expect("governance_gauntlet_toml check must be present");
    assert_eq!(
        toml_check.status,
        DoctorCheckStatus::Pass,
        "Modern gauntlet.toml must pass doctor check"
    );
}

#[test]
fn test_doctor_fix_flag_auto_remediates_governance_issues() {
    let temp = TempDir::new("doc_auto_fix");
    let ws = temp.path();

    let agents_dir = ws.join(".agents");
    fs::create_dir_all(&agents_dir).unwrap();
    fs::write(agents_dir.join("AGENTS.md"), "# Guidelines").unwrap();

    let legacy_hooks = r#"{
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
    fs::write(agents_dir.join("hooks.json"), legacy_hooks).unwrap();

    let legacy_toml = r#"
stack = "rust"
save_evidence = true
evidence_file = "evidence.json"

[[layers]]
name = "unit"
command = ["cargo", "test"]
"#;
    fs::write(ws.join("gauntlet.toml"), legacy_toml).unwrap();

    let options = DoctorOptions {
        workspace: ws.to_path_buf(),
        verbose: true,
        strict: false,
        category: Some(DoctorCategory::Governance),
        fix: true,
    };

    let report = run_doctor(&options).expect("Doctor with --fix must succeed");
    assert!(
        report.is_healthy(),
        "Doctor with fix should remediate warnings"
    );

    // Verify hooks.json on disk was modernized
    let hooks_content = fs::read_to_string(agents_dir.join("hooks.json")).unwrap();
    assert!(
        hooks_content.contains("\"xgauntlet\""),
        "hooks.json must be modernized to 'xgauntlet' key"
    );
    assert!(
        !hooks_content.contains("\"agent-gauntlet-gatekeeper\""),
        "legacy namespace must be removed"
    );
    assert!(
        hooks_content.contains("PreInvocation"),
        "PreInvocation telemetry hook must be injected"
    );

    // Verify gauntlet.toml on disk was modernized
    let toml_content = fs::read_to_string(ws.join("gauntlet.toml")).unwrap();
    assert!(
        toml_content.contains("verification-report.json"),
        "gauntlet.toml must be modernized to 'verification-report.json'"
    );
}
