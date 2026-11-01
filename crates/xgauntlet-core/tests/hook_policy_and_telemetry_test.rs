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
use xgauntlet_core::features::migration::modernize_rust_profile_layers_with_workspace;
use xgauntlet_core::features::policy::{
    is_harness_whitelisted_path, is_harness_whitelisted_path_with_homes,
    is_session_brain_artifact_path, is_session_brain_artifact_path_with_homes,
};

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

fn resolve_test_home() -> String {
    std::env::var("HOME")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .or_else(|| {
            std::env::var("USERPROFILE")
                .ok()
                .filter(|v| !v.trim().is_empty())
        })
        .unwrap_or_else(|| "/home/user".to_string())
}

#[test]
fn test_harness_whitelist_paths() {
    // Harness configs and skill directories outside workspace must be whitelisted for reading
    let home = resolve_test_home();
    let gemini_skill =
        PathBuf::from(&home).join(".gemini/config/plugins/xgauntlet/skills/tdd/SKILL.md");
    let antigravity_skill =
        PathBuf::from(&home).join(".gemini/antigravity/builtin/skills/spec/SKILL.md");
    let agents_dir = PathBuf::from(&home).join(".agents/plugins/xgauntlet/skills/review/SKILL.md");
    let claude_skill = PathBuf::from(&home).join(".claude/skills/discover/SKILL.md");

    assert!(is_harness_whitelisted_path(&gemini_skill));
    assert!(is_harness_whitelisted_path(&antigravity_skill));
    assert!(is_harness_whitelisted_path(&agents_dir));
    assert!(is_harness_whitelisted_path(&claude_skill));

    // Path traversal with '..' must NOT be whitelisted
    let traversal_skill =
        PathBuf::from(&home).join(".gemini/config/plugins/xgauntlet/skills/tdd/../../evil/file");
    assert!(!is_harness_whitelisted_path(&traversal_skill));

    // Unanchored paths outside $HOME must NOT be whitelisted
    let unanchored_skill =
        PathBuf::from("/tmp/evil/.gemini/config/plugins/xgauntlet/skills/tdd/SKILL.md");
    assert!(!is_harness_whitelisted_path(&unanchored_skill));

    // Arbitrary external files must NOT be whitelisted
    let etc_passwd = PathBuf::from("/etc/passwd");
    let ssh_key = PathBuf::from(&home).join(".ssh/id_rsa");
    assert!(!is_harness_whitelisted_path(&etc_passwd));
    assert!(!is_harness_whitelisted_path(&ssh_key));
}

#[test]
fn test_session_brain_artifact_paths() {
    let home = resolve_test_home();
    let brain_artifact = PathBuf::from(&home)
        .join(".gemini/antigravity/brain/bbc9e8e6-c901-4018-97d2-ed84df583c36/report.md");
    let scratch_file = PathBuf::from(&home)
        .join(".gemini/antigravity/brain/bbc9e8e6-c901-4018-97d2-ed84df583c36/scratch/temp.txt");

    assert!(is_session_brain_artifact_path(&brain_artifact));
    assert!(is_session_brain_artifact_path(&scratch_file));

    // Path traversal with '..' must NOT be allowed
    let traversal_brain = PathBuf::from(&home)
        .join(".gemini/antigravity/brain/bbc9e8e6-c901-4018-97d2-ed84df583c36/../../evil/file");
    assert!(!is_session_brain_artifact_path(&traversal_brain));

    // Unanchored path outside $HOME must NOT be allowed
    let unanchored_brain = PathBuf::from("/tmp/evil/.gemini/antigravity/brain/session/file.md");
    assert!(!is_session_brain_artifact_path(&unanchored_brain));

    // Outside brain directory must NOT match
    let other_file = PathBuf::from(&home).join(".gemini/antigravity/other/file.txt");
    assert!(!is_session_brain_artifact_path(&other_file));
}

#[test]
fn test_windows_userprofile_and_cross_platform_path_normalization() {
    // 1. Windows PowerShell environment: HOME unset, USERPROFILE = C:\Users\runneradmin
    let win_homes = [r"C:\Users\runneradmin"];

    assert!(is_harness_whitelisted_path_with_homes(
        Path::new(r"C:\Users\runneradmin\.gemini\config\plugins\xgauntlet\skills\tdd\SKILL.md"),
        &win_homes
    ));
    assert!(is_harness_whitelisted_path_with_homes(
        Path::new("C:/Users/runneradmin/.gemini/config/plugins/xgauntlet/skills/tdd/SKILL.md"),
        &win_homes
    ));
    assert!(is_harness_whitelisted_path_with_homes(
        Path::new(r"c:\Users\runneradmin\.gemini\antigravity\builtin\skills\spec\SKILL.md"),
        &win_homes
    ));
    assert!(is_harness_whitelisted_path_with_homes(
        Path::new(r"\\?\C:\Users\runneradmin\.agents\plugins\xgauntlet\skills\review\SKILL.md"),
        &win_homes
    ));

    assert!(is_session_brain_artifact_path_with_homes(
        Path::new(
            r"C:\Users\runneradmin\.gemini\antigravity\brain\bbc9e8e6-c901-4018-97d2-ed84df583c36\report.md"
        ),
        &win_homes
    ));
    assert!(is_session_brain_artifact_path_with_homes(
        Path::new(
            "c:/Users/runneradmin/.gemini/antigravity/brain/bbc9e8e6-c901-4018-97d2-ed84df583c36/scratch/temp.txt"
        ),
        &win_homes
    ));

    // 2. Dual HOME + USERPROFILE (e.g. Git Bash or empty HOME="") must evaluate all valid roots
    let mixed_homes = ["", "/c/Users/runneradmin", r"C:\Users\runneradmin"];
    assert!(is_harness_whitelisted_path_with_homes(
        Path::new(r"C:\Users\runneradmin\.gemini\config\plugins\xgauntlet\skills\tdd\SKILL.md"),
        &mixed_homes
    ));
    assert!(is_session_brain_artifact_path_with_homes(
        Path::new(r"C:\Users\runneradmin\.gemini\antigravity\brain\sess-1\artifact.md"),
        &mixed_homes
    ));
    assert!(!is_harness_whitelisted_path_with_homes(
        Path::new(".gemini/config/plugins/xgauntlet/skills/tdd/SKILL.md"),
        &mixed_homes
    ));
    assert!(!is_harness_whitelisted_path_with_homes(
        Path::new(r"D:\evil\.gemini\config\plugins\xgauntlet\skills\tdd\SKILL.md"),
        &mixed_homes
    ));

    // 3. Empty or filesystem-root anchors MUST be rejected fail-closed
    let invalid_root_homes = ["", "/", "\\", "C:\\", "C:/", "c:\\"];
    assert!(!is_harness_whitelisted_path_with_homes(
        Path::new("/.gemini/config/plugins/xgauntlet/skills/tdd/SKILL.md"),
        &invalid_root_homes
    ));
    assert!(!is_harness_whitelisted_path_with_homes(
        Path::new(r"C:\.gemini\config\plugins\xgauntlet\skills\tdd\SKILL.md"),
        &invalid_root_homes
    ));
    assert!(!is_session_brain_artifact_path_with_homes(
        Path::new(r"C:\.gemini\antigravity\brain\sess-1\artifact.md"),
        &invalid_root_homes
    ));

    // 4. Prefix boundary check: C:\Users\runner must NOT match C:\Users\runneradmin
    let short_homes = [r"C:\Users\runner", "/home/rolf"];
    assert!(!is_harness_whitelisted_path_with_homes(
        Path::new(r"C:\Users\runneradmin\.gemini\config\plugins\xgauntlet\skills\tdd\SKILL.md"),
        &short_homes
    ));
    assert!(!is_harness_whitelisted_path_with_homes(
        Path::new("/home/rolfmadsen/.gemini/config/plugins/xgauntlet/skills/tdd/SKILL.md"),
        &short_homes
    ));
}

#[test]
fn test_backslash_traversal_rejected_across_platforms() {
    // Relative agent paths with Windows backslashes and '..' must be rejected on ALL OSes
    assert!(!is_harness_whitelisted_path(Path::new(
        r".agents\..\..\etc\passwd"
    )));
    assert!(!is_harness_whitelisted_path(Path::new(
        r".claude\..\.ssh\id_rsa"
    )));
    assert!(!is_harness_whitelisted_path_with_homes(
        Path::new(r"C:\Users\runneradmin\.gemini\config\plugins\..\..\.ssh\id_rsa"),
        &[r"C:\Users\runneradmin"]
    ));
    assert!(!is_session_brain_artifact_path_with_homes(
        Path::new(r"C:\Users\runneradmin\.gemini\antigravity\brain\sess\..\..\..\.ssh\id_rsa"),
        &[r"C:\Users\runneradmin"]
    ));

    // End-to-end hook evaluation must also reject backslash '..' traversal
    let ws = TempWorkspace::new("backslash_traversal_guard");
    let adapter = AntigravityAdapter;
    let payload = json!({
        "hookEventName": "PreToolUse",
        "tool_name": "read_file",
        "parameters": {
            "Path": r".agents\..\..\etc\passwd"
        }
    });
    let (code, output) = adapter.handle_hook(ws.path(), &payload.to_string());
    assert_ne!(
        code, 0,
        "Backslash '..' traversal must be blocked in PreToolUse hook: {output}"
    );
    assert!(
        output.contains("[XGAUNTLET POLICY BLOCKED]"),
        "Expected structured denial for backslash traversal: {output}"
    );
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
    assert!(
        output.contains("git push") || output.contains("push"),
        "Denial reason must reference the actual command 'git push', not empty command: {output}"
    );
}

#[test]
fn test_hook_parameters_outside_workspace_denied_with_path() {
    let ws = TempWorkspace::new("params_outside_ws");
    let adapter = AntigravityAdapter;

    let payload = json!({
        "hookEventName": "PreToolUse",
        "tool_name": "read_file",
        "parameters": {
            "Path": "/etc/shadow"
        }
    });

    let (code, output) = adapter.handle_hook(ws.path(), &payload.to_string());
    assert_ne!(code, 0, "read_file on /etc/shadow must be blocked");
    assert!(
        output.contains("/etc/shadow")
            || output.contains("shadow")
            || output.contains("containment"),
        "Must evaluate the actual path from parameters: {output}"
    );
}

#[test]
fn test_structured_stderr_on_claude_code_and_malformed_stdin() {
    let ws = TempWorkspace::new("stderr_contract");
    use xgauntlet_core::features::adapters::claude_code::ClaudeCodeAdapter;

    let adapter = ClaudeCodeAdapter;
    let payload = json!({
        "hook_event_name": "PreToolUse",
        "tool_name": "Bash",
        "tool_input": {
            "command": "git push origin main"
        }
    });

    let (code, output) = adapter.handle_hook(ws.path(), &payload.to_string());
    assert_ne!(code, 0);
    assert!(
        output.contains("[XGAUNTLET POLICY BLOCKED]"),
        "ClaudeCodeAdapter must output structured stderr on denial: {output}"
    );

    let agy = AntigravityAdapter;
    let (code_empty, out_empty) = agy.handle_hook(ws.path(), "");
    assert_ne!(code_empty, 0);
    assert!(
        out_empty.contains("[XGAUNTLET POLICY BLOCKED]"),
        "AntigravityAdapter must output structured stderr on empty/invalid stdin: {out_empty}"
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
    assert_eq!(
        code, 0,
        "read_file must not be blocked in SPEC phase: {output}"
    );

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
    let layer_names: Vec<&str> = profile
        .default_layers
        .iter()
        .map(|l| l.name.as_str())
        .collect();

    // Must NOT have redundant "types" layer when clippy is configured in "lint"
    assert!(
        !layer_names.contains(&"types"),
        "Rust profile must not include redundant types layer (cargo check) when lint (clippy) is present"
    );

    // 2. Test modernization with proptests.rs in workspace
    let ws = TempWorkspace::new("proptests_migration");
    let tests_dir = ws.path().join("tests");
    fs::create_dir_all(&tests_dir).unwrap();
    fs::write(tests_dir.join("proptests.rs"), "// proptest harness").unwrap();

    let legacy_toml = r#"
[layers]
lint = ["cargo", "clippy", "--all-targets", "--", "-D", "warnings"]
types = ["cargo", "check"]
test = ["cargo", "test"]
invariants = ["cargo", "test", "--", "proptest"]
"#;

    let modernized_with_file =
        modernize_rust_profile_layers_with_workspace(legacy_toml, Some(ws.path()));
    assert!(
        !modernized_with_file.contains("types = [\"cargo\", \"check\"]"),
        "Migration must remove redundant types layer: {modernized_with_file}"
    );
    assert!(
        modernized_with_file.contains("tests/proptests.rs") || modernized_with_file.contains("--test\", \"proptests"),
        "Migration should rewrite targeted invariant testing when tests/proptests.rs exists: {modernized_with_file}"
    );

    // 3. Test modernization without proptests.rs in workspace preserves invariants
    let ws_empty = TempWorkspace::new("empty_migration");
    let modernized_without_file =
        modernize_rust_profile_layers_with_workspace(legacy_toml, Some(ws_empty.path()));
    assert!(
        modernized_without_file.contains("[\"cargo\", \"test\", \"--\", \"proptest\"]"),
        "Migration must NOT rewrite to --test proptests if file does not exist: {modernized_without_file}"
    );
}

#[test]
fn test_hook_blocks_command_injection_and_unscoped_git_commit_without_task() {
    // Workspace with NO active task (simulating SPEC or post-DONE state)
    let ws = TempWorkspace::new("cmd_injection_guard");
    let adapter = AntigravityAdapter;

    let forbidden_commands = [
        ("git status | sh", "pipe to shell"),
        ("git status || rm -r crates", "logical OR chain"),
        ("git status\nrm -r src", "newline injection"),
        (
            "git diff > crates/xgauntlet-core/src/lib.rs",
            "shell output redirection",
        ),
        (
            "git add . && git commit -m \"unscoped bypass\"",
            "raw unscoped git commit without active task",
        ),
        (
            "xgauntlet checkpoint -m \"$(rm -r src)\"",
            "command substitution in checkpoint",
        ),
    ];

    for (cmd, label) in forbidden_commands {
        let payload = json!({
            "hookEventName": "PreToolUse",
            "tool_name": "run_command",
            "parameters": {
                "CommandLine": cmd
            }
        });
        let (code, output) = adapter.handle_hook(ws.path(), &payload.to_string());
        assert_ne!(
            code, 0,
            "ExecuteCommand '{cmd}' ({label}) MUST be denied without active task, got: {output}"
        );
        assert!(
            output.contains("[XGAUNTLET POLICY BLOCKED]"),
            "Expected structured denial for '{cmd}' ({label}), got: {output}"
        );
    }

    // Conversely, single unchained xgauntlet governance commands without shell metachars are allowed
    for safe_cmd in [
        "xgauntlet check-release --strict",
        "xgauntlet checkpoint --phase done -t 052",
        "xgauntlet handoff",
    ] {
        let payload = json!({
            "hookEventName": "PreToolUse",
            "tool_name": "run_command",
            "parameters": {
                "CommandLine": safe_cmd
            }
        });
        let (code, output) = adapter.handle_hook(ws.path(), &payload.to_string());
        assert_eq!(
            code, 0,
            "Safe unchained governance command '{safe_cmd}' should be permitted: {output}"
        );
    }
}
