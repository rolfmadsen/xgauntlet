use std::fs;
use std::path::PathBuf;
use xgauntlet_core::features::adapters::AntigravityAdapter;

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
        let path = std::env::temp_dir().join(unique);
        fs::create_dir_all(&path).unwrap();
        Self { path }
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[test]
fn test_ac1_pre_invocation_schema_compliance_flat_format() {
    let generated = AntigravityAdapter::generate_hooks_json(None);
    let xg = generated
        .get("xgauntlet")
        .expect("xgauntlet key must exist");

    // PreInvocation must be a flat array of handler objects directly without matcher
    let pre_inv = xg
        .get("PreInvocation")
        .and_then(|v| v.as_array())
        .expect("PreInvocation must be an array");
    assert_eq!(pre_inv.len(), 1, "Must contain exactly one handler");

    let handler = &pre_inv[0];
    assert!(
        handler.get("matcher").is_none(),
        "PreInvocation handler must NOT contain 'matcher' in official schema. Got: {:?}",
        handler
    );
    assert_eq!(
        handler.get("type").and_then(|v| v.as_str()),
        Some("command")
    );
    assert_eq!(
        handler.get("command").and_then(|v| v.as_str()),
        Some("xgauntlet telemetry --format antigravity-hook")
    );

    // In contrast, PreToolUse is grouped and must have matcher
    let pre_tool = xg
        .get("PreToolUse")
        .and_then(|v| v.as_array())
        .expect("PreToolUse must be an array");
    assert_eq!(pre_tool.len(), 1);
    assert!(
        pre_tool[0].get("matcher").is_some(),
        "PreToolUse must contain 'matcher'"
    );
    assert!(
        pre_tool[0].get("hooks").is_some(),
        "PreToolUse must contain 'hooks' wrapper"
    );
}

#[test]
fn test_ac1_pre_invocation_legacy_grouped_normalization() {
    let legacy = serde_json::json!({
        "xgauntlet": {
            "enabled": true,
            "PreInvocation": [
                {
                    "matcher": ".*",
                    "hooks": [
                        {
                            "type": "command",
                            "command": "xgauntlet telemetry --format antigravity-hook"
                        }
                    ]
                }
            ]
        }
    });

    let migrated = AntigravityAdapter::generate_hooks_json(Some(&legacy));
    let xg = migrated.get("xgauntlet").expect("xgauntlet key must exist");

    let pre_inv = xg
        .get("PreInvocation")
        .and_then(|v| v.as_array())
        .expect("PreInvocation must be an array");
    assert_eq!(
        pre_inv.len(),
        1,
        "PreInvocation should normalize legacy entries to 1 flat handler without duplicate"
    );

    let handler = &pre_inv[0];
    assert!(
        handler.get("matcher").is_none(),
        "Migrated PreInvocation must not have matcher"
    );
    assert_eq!(
        handler.get("command").and_then(|v| v.as_str()),
        Some("xgauntlet telemetry --format antigravity-hook")
    );
}

#[test]
fn test_ac2_scaffold_hooks_fails_on_corrupt_json_without_clobbering() {
    let temp = TempDir::new("antigravity_corrupt_json");
    let ws = &temp.path;
    let agents_dir = ws.join(".agents");
    fs::create_dir_all(&agents_dir).unwrap();

    let hooks_path = agents_dir.join("hooks.json");
    let corrupt_content = "{\n  \"xgauntlet\": {\n    broken json syntax without closing\n";
    fs::write(&hooks_path, corrupt_content).unwrap();

    let result = AntigravityAdapter::scaffold_hooks(ws);
    assert!(
        result.is_err(),
        "scaffold_hooks must return Err when hooks.json is corrupt"
    );

    let err = result.unwrap_err();
    assert_eq!(
        err.kind(),
        std::io::ErrorKind::InvalidData,
        "Error kind must be InvalidData"
    );
    assert!(
        err.to_string().contains("JSON") || err.to_string().contains("corrupt"),
        "Error message must provide actionable hint about corrupt JSON: {}",
        err
    );

    // Invariant: original file must remain 100% untouched
    let preserved_content = fs::read_to_string(&hooks_path).unwrap();
    assert_eq!(
        preserved_content, corrupt_content,
        "Corrupted hooks.json must NOT be clobbered or modified"
    );
}

#[test]
fn test_ac3_group_key_isolation_does_not_hijack_foreign_keys() {
    let foreign = serde_json::json!({
        "custom-security-scanner": {
            "enabled": true,
            "PreToolUse": [
                {
                    "matcher": "run_command",
                    "hooks": [
                        {
                            "type": "command",
                            "command": "./scripts/custom-scan.sh"
                        }
                    ]
                }
            ]
        }
    });

    let merged = AntigravityAdapter::generate_hooks_json(Some(&foreign));

    // Foreign key must remain completely untouched
    let custom = merged
        .get("custom-security-scanner")
        .expect("Foreign key must still exist");
    assert!(
        custom.get("PreInvocation").is_none(),
        "Foreign key must not be hijacked with PreInvocation"
    );
    let foreign_ptu = custom.get("PreToolUse").and_then(|v| v.as_array()).unwrap();
    assert_eq!(foreign_ptu.len(), 1);
    assert_eq!(
        foreign_ptu[0]["hooks"][0]["command"].as_str(),
        Some("./scripts/custom-scan.sh")
    );

    // xgauntlet key must be added distinctly
    let xg = merged
        .get("xgauntlet")
        .expect("xgauntlet key must be created cleanly");
    assert!(xg.get("PreInvocation").is_some());
    assert!(xg.get("PreToolUse").is_some());
}

#[test]
fn test_ac4_scaffold_scenarios_comprehensive_suite() {
    let temp = TempDir::new("antigravity_scenarios");
    let ws = &temp.path;

    // Scenario 1: Non-existent file
    let path = AntigravityAdapter::scaffold_hooks(ws).expect("Scenario 1 must succeed");
    assert!(path.is_file());
    let content_scen1: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    let pre_inv = content_scen1["xgauntlet"]["PreInvocation"]
        .as_array()
        .unwrap();
    assert_eq!(pre_inv.len(), 1);
    assert!(pre_inv[0].get("matcher").is_none());

    // Scenario 2: Valid file with foreign plugin -> merges cleanly
    let foreign_plugin_json = serde_json::json!({
        "my-linter": {
            "enabled": true,
            "PreToolUse": [
                {
                    "matcher": "view_file",
                    "hooks": [{"type": "command", "command": "run-linter"}]
                }
            ]
        }
    });
    fs::write(
        &path,
        serde_json::to_string_pretty(&foreign_plugin_json).unwrap(),
    )
    .unwrap();

    let path2 = AntigravityAdapter::scaffold_hooks(ws).expect("Scenario 2 must succeed");
    let content_scen2: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&path2).unwrap()).unwrap();
    assert!(
        content_scen2.get("my-linter").is_some(),
        "my-linter must be preserved"
    );
    assert!(
        content_scen2.get("xgauntlet").is_some(),
        "xgauntlet must be merged cleanly"
    );

    // Scenario 3: Corrupt file -> fails without clobbering
    let corrupt_raw = "{ invalid json content ...";
    fs::write(&path2, corrupt_raw).unwrap();
    let res3 = AntigravityAdapter::scaffold_hooks(ws);
    assert!(res3.is_err(), "Scenario 3 must fail on corrupt JSON");
    assert_eq!(fs::read_to_string(&path2).unwrap(), corrupt_raw);
}
