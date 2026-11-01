//! Cross-Platform & Concurrency CI Parity Acceptance Suite (Task 054).
//!
//! Deterministically reproduces macOS and Windows CI failure modes locally on Linux
//! during every `cargo test` and `xgauntlet verify` run:
//! 1. Parallel `IsolatedWorktree` collision stress-test (24 barrier-synchronized threads)
//!    simulating macOS 1µs / Windows 100ns `SystemTime` clock resolution.
//! 2. Symlinked workspace & temp-root resolution (`symlink_dir -> private_dir`),
//!    simulating macOS `/var/folders -> /private/var/folders` and `/tmp -> /private/tmp`.
//! 3. Mechanical source-tree invariant audit preventing any future `temp_dir()` path
//!    without an `AtomicU64` monotonic counter or `env::var("HOME")` without `USERPROFILE`.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Barrier};

use serde_json::json;
use xgauntlet_core::features::adapters::antigravity::AntigravityAdapter;
use xgauntlet_core::features::adapters::models::HarnessAdapter;
use xgauntlet_core::features::checkpoint::provenance::{
    verify_inverse_test_failure, TestProvenanceStatus,
};
use xgauntlet_core::features::evidence::lineage::generate_session_nonce;
use xgauntlet_core::features::policy::WorkspaceRelativePath;

static TEST_COUNTER: AtomicU64 = AtomicU64::new(1);

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(prefix: &str) -> Self {
        let count = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "xgauntlet_ci_parity_{prefix}_{}_{count}_{nanos}",
            std::process::id()
        ));
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

fn init_git_repo(ws: &Path) {
    let init = Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(ws)
        .output()
        .unwrap();
    if !init.status.success() {
        let _ = Command::new("git").args(["init"]).current_dir(ws).output();
    }
    for (k, v) in [
        ("user.name", "CI Parity Tester"),
        ("user.email", "ci-parity@xgauntlet.dev"),
        ("commit.gpgsign", "false"),
    ] {
        let _ = Command::new("git")
            .args(["config", k, v])
            .current_dir(ws)
            .output();
    }
}

fn commit_all(ws: &Path, msg: &str) {
    let out_add = Command::new("git")
        .args(["add", "-A"])
        .current_dir(ws)
        .output()
        .unwrap();
    assert!(out_add.status.success(), "git add failed");
    let out_commit = Command::new("git")
        .args(["commit", "-q", "-m", msg])
        .current_dir(ws)
        .output()
        .unwrap();
    assert!(out_commit.status.success(), "git commit failed");
}

#[test]
fn test_concurrent_worktree_inverse_preflight_and_nonce_uniqueness() {
    // 1. Tight-loop nonce uniqueness (catches coarse 1µs macOS / 100ns Windows SystemTime ticks)
    let nonces: HashSet<String> = (0..256).map(|_| generate_session_nonce()).collect();
    assert_eq!(
        nonces.len(),
        256,
        "generate_session_nonce must produce unique nonces even within the same clock tick"
    );

    // 2. 16 barrier-synchronized threads creating isolated git worktrees at the exact same instant
    let thread_count = 16;
    let barrier = Arc::new(Barrier::new(thread_count));
    let mut handles = Vec::with_capacity(thread_count);

    for idx in 0..thread_count {
        let b = Arc::clone(&barrier);
        handles.push(std::thread::spawn(move || {
            let temp = TempDir::new(&format!("parallel_wt_{idx}"));
            let ws = temp.path();
            init_git_repo(ws);

            fs::create_dir_all(ws.join("src")).unwrap();
            fs::create_dir_all(ws.join("tests")).unwrap();
            fs::write(
                ws.join("gauntlet.toml"),
                "[stack]\nname = \"custom\"\n\n[[layers]]\nname = \"test\"\ncommand = [\"git\", \"grep\", \"-q\", \"-F\", \"expected_marker\", \"--\", \"src/lib.rs\"]\n",
            )
            .unwrap();
            fs::write(ws.join("src/lib.rs"), "pub fn val() -> &'static str { \"baseline\" }\n")
                .unwrap();
            commit_all(ws, "chore: baseline");

            // Stage in-flight GREEN state + new test
            fs::write(
                ws.join("src/lib.rs"),
                "pub fn val() -> &'static str { \"expected_marker\" }\n",
            )
            .unwrap();
            fs::write(
                ws.join("tests/parity_test.rs"),
                "#[test]\nfn test_parallel_case() { assert!(true); }\n",
            )
            .unwrap();

            // Synchronize all 16 threads so IsolatedWorktree::create runs at the exact same microsecond!
            b.wait();

            let evidences = verify_inverse_test_failure(ws, &["test_parallel_case".to_string()])
                .unwrap_or_else(|e| {
                    panic!("Thread {idx} failed inverse preflight under concurrency: {e:?}")
                });
            assert_eq!(evidences.len(), 1);
            assert_eq!(
                evidences[0].status,
                TestProvenanceStatus::InversePreflightVerified
            );
        }));
    }

    for h in handles {
        h.join().expect("worker thread must not panic");
    }
}

#[cfg(unix)]
#[test]
fn test_macos_style_symlinked_temp_and_workspace_parity() {
    // On macOS, /var/folders -> /private/var/folders and /tmp -> /private/tmp.
    // A process often holds canonical `/private/...` as workspace while tools pass `/var/...` paths (or vice versa).
    let root = TempDir::new("macos_symlink_sim");
    let private_ws = root.path().join("private").join("var").join("workspace");
    fs::create_dir_all(private_ws.join("src")).unwrap();
    fs::create_dir_all(private_ws.join("tasks")).unwrap();

    let symlink_var = root.path().join("var_alias");
    std::os::unix::fs::symlink(root.path().join("private").join("var"), &symlink_var).unwrap();
    let alias_ws = symlink_var.join("workspace");

    fs::write(private_ws.join("src/lib.rs"), "pub fn ok() {}\n").unwrap();
    fs::write(
        private_ws.join("tasks/001-demo.md"),
        "---\ntype: Task Package\ntitle: \"Demo\"\nstatus: active\nscope: [\"src/**\"]\ntest_refs: [\"tests/demo.rs\"]\n---\n# Demo\n## 🎯 Formål\nTest\n## 📋 Acceptance Criteria\n- [ ] AC1\n## 🚫 Must NOT\n- None\n",
    )
    .unwrap();

    // Case A: Workspace is canonical (`/private/var/workspace`), tool passes symlinked path (`/var_alias/workspace/src/new_file.rs`)
    let symlinked_new_file = alias_ws.join("src/new_file.rs");
    let sanitized_a = WorkspaceRelativePath::sanitize(
        &private_ws,
        &symlinked_new_file.to_string_lossy(),
    )
    .expect(
        "WorkspaceRelativePath::sanitize must accept symlinked alias path inside canonical workspace (macOS /tmp -> /private/tmp parity)",
    );
    assert_eq!(sanitized_a.as_str(), "src/new_file.rs");

    // Case B: Workspace is symlinked (`/var_alias/workspace`), tool passes canonical path (`/private/var/workspace/src/new_file.rs`)
    let canonical_new_file = private_ws
        .canonicalize()
        .unwrap()
        .join("src/new_file.rs");
    let sanitized_b = WorkspaceRelativePath::sanitize(
        &alias_ws,
        &canonical_new_file.to_string_lossy(),
    )
    .expect(
        "WorkspaceRelativePath::sanitize must accept canonical path inside symlinked workspace",
    );
    assert_eq!(sanitized_b.as_str(), "src/new_file.rs");

    // Case C: End-to-end Antigravity PreToolUse write_file through symlinked path against canonical workspace
    let adapter = AntigravityAdapter;
    let payload = json!({
        "hookEventName": "PreToolUse",
        "tool_name": "write_file",
        "parameters": {
            "Path": symlinked_new_file.to_string_lossy()
        }
    });
    let (code, output) = adapter.handle_hook(&private_ws, &payload.to_string());
    assert_eq!(
        code, 0,
        "PreToolUse hook must allow in-scope symlinked path on macOS-style layout: {output}"
    );

    // Case D: IsolatedWorktree invoked via symlinked workspace path
    init_git_repo(&alias_ws);
    fs::write(
        alias_ws.join("gauntlet.toml"),
        "[stack]\nname = \"custom\"\n\n[[layers]]\nname = \"test\"\ncommand = [\"git\", \"grep\", \"-q\", \"-F\", \"symlink_ok\", \"--\", \"src/lib.rs\"]\n",
    )
    .unwrap();
    commit_all(&alias_ws, "chore: baseline");
    fs::write(alias_ws.join("src/lib.rs"), "// symlink_ok\n").unwrap();
    fs::create_dir_all(alias_ws.join("tests")).unwrap();
    fs::write(
        alias_ws.join("tests/sym_test.rs"),
        "#[test]\nfn test_sym() { assert!(true); }\n",
    )
    .unwrap();

    let evidences = verify_inverse_test_failure(&alias_ws, &["test_sym".to_string()])
        .expect("IsolatedWorktree must succeed when workspace is accessed via symlink");
    assert_eq!(evidences.len(), 1);
}

fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                collect_rs_files(&p, out);
            } else if p.extension().is_some_and(|e| e == "rs") {
                out.push(p);
            }
        }
    }
}

#[test]
fn test_codebase_free_of_collision_prone_temp_paths_and_unix_only_env() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let crates_root = manifest_dir.parent().expect("crates/ directory");

    let mut rs_files = Vec::new();
    collect_rs_files(&crates_root.join("xgauntlet-core"), &mut rs_files);
    collect_rs_files(&crates_root.join("xgauntlet-cli"), &mut rs_files);
    collect_rs_files(&crates_root.join("gauntlet-policy-engine"), &mut rs_files);
    rs_files.sort();

    let mut violations = Vec::new();

    for file in rs_files {
        // Skip inspecting this parity test file's own string literals
        if file
            .file_name()
            .is_some_and(|n| n == "cross_platform_ci_parity_test.rs")
        {
            continue;
        }

        let content = fs::read_to_string(&file).unwrap_or_default();
        let rel = file
            .strip_prefix(crates_root.parent().unwrap_or(crates_root))
            .unwrap_or(&file)
            .display()
            .to_string();

        // Invariant 1: Any file constructing ephemeral paths under temp_dir() + .join(format!(...))
        // (except intentional shared dedup caches `*telemetry_dedup*`) MUST use AtomicU64 to prevent
        // parallel thread collisions on coarse macOS (1µs) and Windows (100ns) system clocks.
        let uses_formatted_temp_join =
            content.contains("std::env::temp_dir()") && content.contains(".join(format!(");
        let is_shared_dedup_only =
            uses_formatted_temp_join && content.contains("xgauntlet_telemetry_dedup_");

        if uses_formatted_temp_join && !is_shared_dedup_only && !content.contains("AtomicU64") {
            violations.push(format!(
                "{rel}: constructs temp_dir().join(format!(...)) without an AtomicU64 monotonic counter (will collide across parallel threads on macOS/Windows)"
            ));
        }

        // Invariant 2: Any cross-platform Rust file reading env::var("HOME") MUST also fall back
        // to or evaluate USERPROFILE so Windows PowerShell CI (`pwsh.EXE`, where HOME is unset) works.
        if content.contains("env::var(\"HOME\")") && !content.contains("\"USERPROFILE\"") {
            violations.push(format!(
                "{rel}: calls env::var(\"HOME\") without \"USERPROFILE\" support for Windows PowerShell"
            ));
        }
    }

    assert!(
        violations.is_empty(),
        "Cross-platform CI parity violations detected locally:\n  - {}",
        violations.join("\n  - ")
    );
}
