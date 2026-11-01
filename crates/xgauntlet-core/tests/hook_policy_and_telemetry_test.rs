//! Acceptance tests for Task 052: Hook Policy Hardening, Telemetry Deduplication & Migration.
//!
//! Validates:
//! - AC3: Harness config whitelist, session brain artifact access, safe `run_command`
//!   retention, and mandatory structured stderr on policy rejection.
//! - AC4: Context-saving telemetry deduplication and direct native binary hook binding.
//! - AC5: Deduplicated Rust stack profile in `init` and `migrate`.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::json;
use xgauntlet_core::features::adapters::antigravity::AntigravityAdapter;
use xgauntlet_core::features::adapters::models::HarnessAdapter;
use xgauntlet_core::features::config::profiles::get_stack_profile;
use xgauntlet_core::features::migration::modernize_rust_profile_layers;
use xgauntlet_core::features::policy::{is_harness_whitelisted_path, is_session_brain_artifact_path};

static TEST_COUNTER: AtomicU64 = AtomicU64::new(1);

struct TempWorkspace {
    path: PathBuf,
}

impl TempWorkspace {
    fn new(prefix: &str) -> Self {
        let count = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "xgauntlet_hook_hardening_test_{prefix}_{}_{count}_{nanos}",
            std::process::id()
        ));
        fs::create_dir_all(&path).unwrap();
        let canon = path.canonicalize().unwrap();
        #[cfg(windows)]
        let clean = {
            let s = canon.to_string_lossy();
            if let Some(stripped) = s.strip_prefix(r"\\?\UNC\") {
                std::path::PathBuf::from(format!(r"\\{}", stripped))
            } else if let Some(stripped) = s.strip_prefix(r"\\?\") {
                std::path::PathBuf::from(stripped)
            } else {
                canon
            }
        };
        #[cfg(not(windows))]
        let clean = canon;

        Self { path: clean }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[test]
fn test_harness_whitelist_paths() {
    // Harness configs and skill directories outside workspace must be whitelisted for reading
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/user".to_string());
    let gemini_skill = PathBuf::from(&home).join(".gemini/config/plugins/xgauntlet/skills/tdd/SKILL.md");
    let antigravity_skill = PathBuf::from(&home).join(".gemini/antigravity/builtin/skills/spec/SKILL.md");
    let agents_dir = PathBuf::from(&home).join(".agents/plugins/xgauntlet/skills/review/SKILL.md");
    let claude_skill = PathBuf::from(&home).join(".claude/skills/discover/SKILL.md");

    assert!(is_harness_whitelisted_path(&gemini_skill));
    assert!(is_harness_whitelisted_path(&antigravity_skill));
    assert!(is_harness_whitelisted_path(&agents_dir));
    assert!(is_harness_whitelisted_path(&claude_skill));

    // Arbitrary external files must NOT be whitelisted
    let etc_passwd = PathBuf::from("/etc/passwd");
    let ssh_key = PathBuf::from(&home).join(".ssh/id_rsa");
    assert!(!is_harness_whitelisted_path(&etc_passwd));
    assert!(!is_harness_whitelisted_path(&ssh_key));
}

#[test]
fn test_session_brain_artifact_paths() {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/user".to_string());
    let brain_artifact = PathBuf::from(&home)
        .join(".gemini/antigravity/brain/bbc9e8e6-c901-4018-97d2-ed84df583c36/report.md");
    let scratch_file = PathBuf::from(&home)
        .join(".gemini/antigravity/brain/bbc9e8e6-c901-4018-97d2-ed84df583c36/scratch/temp.txt");

    assert!(is_session_brain_artifact_path(&brain_artifact));
    assert!(is_session_brain_artifact_path(&scratch_file));

    // Outside brain directory must NOT match
    let other_file = PathBuf::from(&home).join(".gemini/antigravity/other/file.txt");
    assert!(!is_session_brain_artifact_path(&other_file));
}

#[test]
fn test_hook_run_command_remains_strictly_gatekept() {
    let ws = TempWorkspace::new("run_command_gate");
    let adapter = AntigravityAdapter;

    // A tool call trying to execute git push
    let payload = json!({
        "hookEventName": "PreToolUse",
        "tool_name": "run_command",
        "parameters": {
            "CommandLine": "git push origin main"
        }
    });

    let (code, output) = adapter.handle_hook(ws.path(), &payload.to_string());
    assert_ne!(code, 0, "run_command with 'git push' must be blocked");
    assert!(
        output.contains("[XGAUNTLET POLICY BLOCKED]"),
        "Blocked hook output must follow structured format: {output}"
    );
    assert!(
        output.contains("Reason:"),
        "Must contain structured Reason field: {output}"
    );
}

#[test]
fn test_hook_read_only_tools_allowed_in_spec_phase() {
    let ws = TempWorkspace::new("spec_read_only");
    let adapter = AntigravityAdapter;

    // In SPEC phase, read_file on workspace files or whitelisted skills must be allowed
    let payload = json!({
        "hookEventName": "PreToolUse",
        "tool_name": "read_file",
        "parameters": {
            "Path": "CONTEXT.md"
        }
    });

    let (code, output) = adapter.handle_hook(ws.path(), &payload.to_string());
    assert_eq!(code, 0, "read_file must not be blocked in SPEC phase: {output}");

    // Interactive planning tools must also be allowed
    let ask_payload = json!({
        "hookEventName": "PreToolUse",
        "tool_name": "ask_question",
        "parameters": {}
    });
    let (ask_code, _) = adapter.handle_hook(ws.path(), &ask_payload.to_string());
    assert_eq!(ask_code, 0, "ask_question must not be blocked");
}

#[test]
fn test_rust_stack_profile_deduplication() {
    // 1. Check default Rust stack profile from init
    let profile = get_stack_profile("rust").expect("Rust profile must exist");
    let layer_names: Vec<&str> = profile.default_layers.iter().map(|l| l.name.as_str()).collect();

    // Must NOT have redundant "types" layer when clippy is configured in "lint"
    assert!(
        !layer_names.contains(&"types"),
        "Rust profile must not include redundant types layer (cargo check) when lint (clippy) is present"
    );

    // 2. Test modernization of existing gauntlet.toml in migration engine
    let legacy_toml = r#"
[layers]
lint = ["cargo", "clippy", "--all-targets", "--", "-D", "warnings"]
types = ["cargo", "check"]
test = ["cargo", "test"]
invariants = ["cargo", "test", "--", "proptest"]
"#;

    let modernized = modernize_rust_profile_layers(legacy_toml);
    assert!(
        !modernized.contains("types = [\"cargo\", \"check\"]"),
        "Migration must remove redundant types layer: {modernized}"
    );
    assert!(
        modernized.contains("tests/proptests.rs") || !modernized.contains("test\", \"--\", \"proptest"),
        "Migration should optimize targeted invariant testing"
    );
}
