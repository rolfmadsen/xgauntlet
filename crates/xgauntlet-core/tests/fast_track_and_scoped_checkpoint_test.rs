//! Acceptance tests for Task 052: Scoped Test Execution & Fast-Track Bugfix Protocol.
//!
//! Validates:
//! - AC1: `test_refs`-bound scoped pre-flight in `xgauntlet checkpoint` (red/green/refactor)
//!   and the Rust target resolver.
//! - AC2: Fast-Track Bugfix Protocol qualification and atomic Worktree Inverse Proof.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use xgauntlet_core::features::checkpoint::preflight::{
    resolve_scoped_test_command, RustTestTarget,
};
use xgauntlet_core::features::checkpoint::provenance::{
    evaluate_fast_track_inverse_proof, is_fast_track_eligible,
};
use xgauntlet_core::features::tasks::models::{TaskPackageInfo, TaskStatus};

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
            "xgauntlet_fast_track_test_{prefix}_{}_{count}_{nanos}",
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

fn init_git_repo(ws: &Path) {
    let init = Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(ws)
        .output()
        .unwrap();
    if !init.status.success() {
        let _ = Command::new("git")
            .args(["init"])
            .current_dir(ws)
            .output();
    }
    let _ = Command::new("git")
        .args(["config", "user.name", "Test Committer"])
        .current_dir(ws)
        .output();
    let _ = Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(ws)
        .output();
}

#[test]
fn test_rust_target_resolver_standalone_integration_test() {
    let ws = TempWorkspace::new("standalone_test");
    let tests_dir = ws.path().join("tests");
    fs::create_dir_all(&tests_dir).unwrap();
    let test_file = tests_dir.join("order_acceptance_test.rs");
    fs::write(&test_file, "// test").unwrap();

    let target = resolve_scoped_test_command(ws.path(), "tests/order_acceptance_test.rs");
    assert_eq!(
        target,
        RustTestTarget::IntegrationTest {
            target_name: "order_acceptance_test".to_string(),
            filter: None,
            command: vec![
                "cargo".to_string(),
                "test".to_string(),
                "--test".to_string(),
                "order_acceptance_test".to_string()
            ],
        }
    );
}

#[test]
fn test_rust_target_resolver_harness_submodule() {
    let ws = TempWorkspace::new("harness_test");
    let tests_dir = ws.path().join("tests");
    fs::create_dir_all(tests_dir.join("features")).unwrap();
    // Harness root exists as tests/features.rs
    fs::write(tests_dir.join("features.rs"), "mod invoice_acceptance_test;").unwrap();
    let submod_file = tests_dir.join("features").join("invoice_acceptance_test.rs");
    fs::write(&submod_file, "// submod").unwrap();

    let target =
        resolve_scoped_test_command(ws.path(), "tests/features/invoice_acceptance_test.rs");
    assert_eq!(
        target,
        RustTestTarget::HarnessSubmodule {
            harness_name: "features".to_string(),
            submodule: "invoice_acceptance_test".to_string(),
            command: vec![
                "cargo".to_string(),
                "test".to_string(),
                "--test".to_string(),
                "features".to_string(),
                "invoice_acceptance_test".to_string()
            ],
        }
    );
}

#[test]
fn test_rust_target_resolver_lib_unit_test() {
    let ws = TempWorkspace::new("lib_test");
    let src_dir = ws.path().join("src").join("features").join("auth");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(src_dir.join("validator.rs"), "// unit test").unwrap();

    let target = resolve_scoped_test_command(ws.path(), "src/features/auth/validator.rs");
    assert_eq!(
        target,
        RustTestTarget::LibModule {
            module_path: "features::auth::validator".to_string(),
            command: vec![
                "cargo".to_string(),
                "test".to_string(),
                "--lib".to_string(),
                "features::auth::validator".to_string()
            ],
        }
    );
}

#[test]
fn test_fast_track_eligibility_criteria() {
    let mut task = TaskPackageInfo {
        task_id: "099".to_string(),
        title: "Fix null pointer in parser".to_string(),
        status: TaskStatus::Active,
        intent: Some("🐛 BUG FIX".to_string()),
        purpose: "Fix panic".to_string(),
        acceptance_criteria: vec!["AC1".to_string()],
        unresolved_criteria: vec!["AC1".to_string()],
        must_not: vec!["Must not regress".to_string()],
        scope: vec!["src/parser.rs".to_string()],
        test_refs: vec!["tests/parser_test.rs".to_string()],
        metadata: None,
    };

    // 1. Qualified: intent=Bug, scope.len() <= 3, no ADR/gauntlet changed
    let changed_files = vec!["src/parser.rs".to_string(), "tests/parser_test.rs".to_string()];
    assert!(is_fast_track_eligible(&task, &changed_files));

    // 2. Disqualified: scope.len() > 3
    task.scope = vec![
        "src/a.rs".to_string(),
        "src/b.rs".to_string(),
        "src/c.rs".to_string(),
        "src/d.rs".to_string(),
    ];
    assert!(!is_fast_track_eligible(&task, &changed_files));

    // 3. Disqualified: intent=Feature
    task.scope = vec!["src/parser.rs".to_string()];
    task.intent = Some("🚀 NEW FEATURE".to_string());
    assert!(!is_fast_track_eligible(&task, &changed_files));

    // 4. Disqualified: touched docs/adr/
    task.intent = Some("🐛 BUG FIX".to_string());
    let adr_changes = vec![
        "src/parser.rs".to_string(),
        "docs/adr/0011-fast-track.md".to_string(),
    ];
    assert!(!is_fast_track_eligible(&task, &adr_changes));

    // 5. Disqualified: touched gauntlet.toml
    let gauntlet_changes = vec!["gauntlet.toml".to_string()];
    assert!(!is_fast_track_eligible(&task, &gauntlet_changes));
}

#[test]
fn test_fast_track_worktree_inverse_proof_born_green_rejection() {
    let ws = TempWorkspace::new("worktree_born_green");
    init_git_repo(ws.path());

    // Commit baseline code
    let src_dir = ws.path().join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(src_dir.join("lib.rs"), "pub fn answer() -> u32 { 42 }").unwrap();

    let tests_dir = ws.path().join("tests");
    fs::create_dir_all(&tests_dir).unwrap();

    fs::write(
        ws.path().join("Cargo.toml"),
        r#"[package]
name = "mock_repo"
version = "0.1.0"
edition = "2021"

[lib]
path = "src/lib.rs"
"#,
    )
    .unwrap();

    let _ = Command::new("git").args(["add", "."]).current_dir(ws.path()).output();
    let _ = Command::new("git").args(["commit", "-m", "chore: baseline"]).current_dir(ws.path()).output();

    // Now introduce a born-green test (already passes against HEAD)
    let test_file = tests_dir.join("dummy_test.rs");
    fs::write(&test_file, r#"
#[test]
fn test_always_passes() {
    assert_eq!(mock_repo::answer(), 42);
}
"#).unwrap();

    let test_refs = vec!["tests/dummy_test.rs".to_string()];
    let res = evaluate_fast_track_inverse_proof(ws.path(), &test_refs);

    // Must be rejected as born-green
    assert!(res.is_err(), "Born-green test must be rejected by inverse proof");
    let err_msg = format!("{:?}", res.err().unwrap());
    assert!(
        err_msg.contains("BORN_GREEN") || err_msg.contains("Tautological"),
        "Error message should mention born-green / tautological: {err_msg}"
    );
}
