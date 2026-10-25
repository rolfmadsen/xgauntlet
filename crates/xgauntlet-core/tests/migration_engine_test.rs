//! Comprehensive integration tests for the in-repo migration engine (Task 026 AC2, AC4, AC5).

use std::fs;
use std::path::{Path, PathBuf};

use xgauntlet_core::features::adapters::antigravity::AntigravityAdapter;
use xgauntlet_core::features::adapters::HarnessAdapter;
use xgauntlet_core::features::migration::{run_migration, MigrationOptions};

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
        let path = std::env::temp_dir().join(format!("xgauntlet_mig_test_{}", unique));
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
fn test_migration_engine_dry_run_does_not_modify_disk() {
    let temp = TempDir::new("mig_dry_run");
    let ws = temp.path();

    let agents_dir = ws.join(".agents");
    fs::create_dir_all(&agents_dir).unwrap();
    let original_hooks = r#"{
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
    fs::write(agents_dir.join("hooks.json"), original_hooks).unwrap();

    let original_toml = r#"stack = "rust"
save_evidence = true
evidence_file = "evidence.json"

[[layers]]
name = "unit"
command = ["cargo", "test"]
"#;
    fs::write(ws.join("gauntlet.toml"), original_toml).unwrap();

    let options = MigrationOptions {
        workspace: ws.to_path_buf(),
        dry_run: true,
    };

    let report = run_migration(&options).expect("Migration dry run must succeed");
    assert!(report.is_success);
    assert!(
        !report.items.is_empty(),
        "Expected planned migration items in dry run report"
    );
    assert!(
        report.files_modified.is_empty(),
        "Dry run must NOT report any files modified"
    );

    // Verify files on disk remain strictly unmodified
    let current_hooks = fs::read_to_string(agents_dir.join("hooks.json")).unwrap();
    assert_eq!(
        current_hooks, original_hooks,
        "hooks.json must remain byte-for-byte identical in dry run"
    );

    let current_toml = fs::read_to_string(ws.join("gauntlet.toml")).unwrap();
    assert_eq!(
        current_toml, original_toml,
        "gauntlet.toml must remain byte-for-byte identical in dry run"
    );
}

#[test]
fn test_migration_engine_modernizes_hooks_preserving_custom_matchers() {
    let temp = TempDir::new("mig_hooks_custom");
    let ws = temp.path();

    let agents_dir = ws.join(".agents");
    fs::create_dir_all(&agents_dir).unwrap();

    let initial_hooks = r#"{
  "agent-gauntlet-gatekeeper": {
    "enabled": true,
    "PreToolUse": [
      {
        "matcher": "custom_internal_tool",
        "hooks": [
          { "command": "sh /scripts/check_internal.sh" }
        ]
      },
      {
        "matcher": "run_command|write_to_file",
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

    let migrated = fs::read_to_string(agents_dir.join("hooks.json")).unwrap();
    let v: serde_json::Value =
        serde_json::from_str(&migrated).expect("Migrated hooks must be valid JSON");

    // 1. Must rename agent-gauntlet-gatekeeper to xgauntlet
    assert!(
        v.get("xgauntlet").is_some(),
        "Expected 'xgauntlet' namespace"
    );
    assert!(
        v.get("agent-gauntlet-gatekeeper").is_none(),
        "Legacy namespace must be replaced"
    );

    let xgauntlet_group = v.get("xgauntlet").unwrap();

    // 2. PreToolUse custom matchers must be preserved
    let pre_tool = xgauntlet_group
        .get("PreToolUse")
        .and_then(|p| p.as_array())
        .unwrap();
    let has_custom = pre_tool
        .iter()
        .any(|item| item.get("matcher").and_then(|m| m.as_str()) == Some("custom_internal_tool"));
    assert!(has_custom, "User's custom matcher must be preserved");

    // 3. PreInvocation telemetry hook must be injected
    let pre_inv = xgauntlet_group
        .get("PreInvocation")
        .and_then(|p| p.as_array())
        .unwrap();
    let has_telemetry = pre_inv.iter().any(|item| {
        let cmd = item.get("command").and_then(|c| c.as_str()).unwrap_or("");
        cmd.contains("xgauntlet telemetry")
    });
    assert!(
        has_telemetry,
        "PreInvocation telemetry hook must be injected"
    );
}

#[test]
fn test_migration_engine_modernizes_gauntlet_toml_preserving_custom_layers() {
    let temp = TempDir::new("mig_toml_layers");
    let ws = temp.path();

    let initial_toml = r#"stack = "rust"
save_evidence = true
evidence_file = "evidence.json"

[[layers]]
name = "custom_mutation_gauntlet"
command = ["cargo", "mutants", "--test", "custom"]
timeout_seconds = 180.0
optional = false
"#;
    fs::write(ws.join("gauntlet.toml"), initial_toml).unwrap();

    let options = MigrationOptions {
        workspace: ws.to_path_buf(),
        dry_run: false,
    };

    let report = run_migration(&options).expect("Migration must succeed");
    assert!(report.is_success);

    let migrated_toml = fs::read_to_string(ws.join("gauntlet.toml")).unwrap();
    assert!(
        migrated_toml.contains("verification-report.json"),
        "evidence_file must be modernized to 'verification-report.json'"
    );
    assert!(
        !migrated_toml.contains("evidence_file = \"evidence.json\""),
        "legacy evidence.json must be replaced"
    );
    assert!(
        migrated_toml.contains("custom_mutation_gauntlet"),
        "Custom user layer name must be preserved"
    );
    assert!(
        migrated_toml.contains("timeout_seconds = 180"),
        "Custom layer options must be preserved"
    );
}

#[test]
fn test_migration_engine_scaffolds_missing_adr_template() {
    let temp = TempDir::new("mig_adr_template");
    let ws = temp.path();

    let adr_dir = ws.join("docs").join("adr");
    fs::create_dir_all(&adr_dir).unwrap();
    fs::write(adr_dir.join("0001-initial.md"), "# ADR 0001").unwrap();

    let options = MigrationOptions {
        workspace: ws.to_path_buf(),
        dry_run: false,
    };

    let report = run_migration(&options).expect("Migration must succeed");
    assert!(report.is_success);

    let template_path = adr_dir.join("template.md");
    assert!(
        template_path.is_file(),
        "Missing docs/adr/template.md must be scaffolded"
    );
    let template_content = fs::read_to_string(template_path).unwrap();
    assert!(
        template_content.contains("Context & Problem Statement")
            || template_content.contains("Beslutning")
            || template_content.contains("Decision"),
        "Scaffolded template must contain standard ADR sections"
    );
}

#[test]
fn test_runtime_backward_compatibility_with_legacy_hooks_namespace() {
    let temp = TempDir::new("hook_legacy_compat");
    let ws = temp.path();

    let agents_dir = ws.join(".agents");
    fs::create_dir_all(&agents_dir).unwrap();
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

    let adapter = AntigravityAdapter::new();
    let test_payload = r#"{
      "hookEventName": "PreInvocation"
    }"#;

    let (exit_code, output) = adapter.handle_hook(ws, test_payload);
    assert_eq!(exit_code, 0, "Hook must succeed under legacy namespace");
    assert!(
        output.contains("injectSteps") || output.contains("decision"),
        "Output should be valid harness hook response"
    );
}
