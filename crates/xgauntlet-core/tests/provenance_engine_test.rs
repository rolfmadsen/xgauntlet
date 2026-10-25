//! Acceptance tests for Task 032: Enforce RED-First Test Provenance & Born-Green Detection.
//!
//! Tests the extraction of test identifiers across languages, the detection of
//! born-green tests in git diffs between checkpoints, inverse preflight verification
//! against baseline production code, and fail-closed gatekeeping in CheckpointPhase::Green.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use xgauntlet_core::features::checkpoint::provenance::{
    detect_born_green_tests, extract_test_identifiers, verify_inverse_test_failure,
    TestProvenanceStatus,
};
use xgauntlet_core::features::checkpoint::{
    run_checkpoint, CheckpointError, CheckpointOptions, CheckpointPhase,
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
        let path = std::env::temp_dir().join(format!("xgauntlet_prov_test_{}", unique));
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

fn init_git_repo(path: &Path) {
    let _ = Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(path)
        .output()
        .or_else(|_| {
            Command::new("git")
                .args(["init"])
                .current_dir(path)
                .output()
        });

    let _ = Command::new("git")
        .args(["config", "user.name", "Test Craftsman"])
        .current_dir(path)
        .output();
    let _ = Command::new("git")
        .args(["config", "user.email", "craftsman@xgauntlet.dev"])
        .current_dir(path)
        .output();
}

fn git_commit_all(path: &Path, msg: &str) -> String {
    let _ = Command::new("git")
        .args(["add", "-A"])
        .current_dir(path)
        .output()
        .unwrap();

    let _ = Command::new("git")
        .args(["commit", "-m", msg])
        .current_dir(path)
        .output()
        .unwrap();

    let out = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(path)
        .output()
        .unwrap();

    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

#[test]
fn test_extract_test_identifiers_polyglot() {
    // 1. Rust syntax
    let rust_code = r#"
        #[test]
        fn test_synchronous_case() {
            assert!(true);
        }

        #[tokio::test]
        async fn test_asynchronous_worker() {
            assert!(true);
        }

        #[test] fn test_inline_definition() {}
    "#;
    let rust_ids = extract_test_identifiers(Path::new("tests/suite.rs"), rust_code);
    assert_eq!(
        rust_ids,
        vec![
            "test_synchronous_case".to_string(),
            "test_asynchronous_worker".to_string(),
            "test_inline_definition".to_string(),
        ]
    );

    // 2. TypeScript / JavaScript syntax
    let ts_code = r#"
        test("validates user credentials", async () => {
            expect(true).toBe(true);
        });

        it('handles boundary timeout', () => {
            expect(1).toBe(1);
        });
    "#;
    let ts_ids = extract_test_identifiers(Path::new("tests/suite.test.ts"), ts_code);
    assert_eq!(
        ts_ids,
        vec![
            "validates user credentials".to_string(),
            "handles boundary timeout".to_string(),
        ]
    );

    // 3. Python syntax
    let py_code = r#"
def test_authentication_success():
    assert True

def test_token_expiration_fails():
    assert False
    "#;
    let py_ids = extract_test_identifiers(Path::new("tests/test_auth.py"), py_code);
    assert_eq!(
        py_ids,
        vec![
            "test_authentication_success".to_string(),
            "test_token_expiration_fails".to_string(),
        ]
    );
}

#[test]
fn test_detect_born_green_tests_in_git_diff() {
    let temp = TempDir::new("detect_born_green");
    let ws = temp.path();
    init_git_repo(ws);

    let tests_dir = ws.join("tests");
    fs::create_dir_all(&tests_dir).unwrap();
    let test_file = tests_dir.join("feature_test.rs");

    // Commit 1: Initial RED commit with only test_initial_requirement
    fs::write(
        &test_file,
        r#"
#[test]
fn test_initial_requirement() {
    assert_eq!(2 + 2, 4);
}
"#,
    )
    .unwrap();
    let red_commit_oid = git_commit_all(ws, "test(032): add failing test [RED]");

    // Now, during GREEN phase: a developer adds a second test directly
    fs::write(
        &test_file,
        r#"
#[test]
fn test_initial_requirement() {
    assert_eq!(2 + 2, 4);
}

#[test]
fn test_sneaky_born_green_requirement() {
    assert_eq!(3 * 3, 9);
}
"#,
    )
    .unwrap();

    let born_green = detect_born_green_tests(ws, &red_commit_oid).expect("should inspect diff");
    assert_eq!(born_green, vec!["test_sneaky_born_green_requirement"]);
}

#[test]
fn test_inverse_preflight_detects_tautological_test() {
    let temp = TempDir::new("inverse_tautological");
    let ws = temp.path();
    init_git_repo(ws);

    let src_dir = ws.join("src");
    let tests_dir = ws.join("tests");
    fs::create_dir_all(&src_dir).unwrap();
    fs::create_dir_all(&tests_dir).unwrap();

    // Configure test layer via gauntlet.toml
    fs::write(
        ws.join("gauntlet.toml"),
        r#"
[stack]
name = "custom"

[[layers]]
name = "test"
command = ["grep", "pub fn calculate() -> i32 { 42 }", "src/lib.rs"]
"#,
    )
    .unwrap();

    // Baseline: simple math module
    fs::write(src_dir.join("lib.rs"), "pub fn calculate() -> i32 { 42 }\n").unwrap();
    fs::write(
        tests_dir.join("calc_test.rs"),
        "use dummy::calculate;\n#[test]\nfn test_existing() { assert_eq!(calculate(), 42); }\n",
    )
    .unwrap();
    let _ = git_commit_all(ws, "chore: baseline");

    // Introduce production change + tautological test that passes regardless of production changes
    fs::write(
        src_dir.join("lib.rs"),
        "pub fn calculate() -> i32 { 42 }\npub fn new_feature() -> bool { true }\n",
    )
    .unwrap();
    fs::write(
        tests_dir.join("calc_test.rs"),
        "use dummy::calculate;\n#[test]\nfn test_existing() { assert_eq!(calculate(), 42); }\n#[test]\nfn test_tautology() { assert_eq!(calculate(), 42); }\n",
    )
    .unwrap();

    // Inverse preflight against baseline must reject test_tautology because it passes against baseline!
    let res = verify_inverse_test_failure(ws, &["test_tautology".to_string()]);
    match res {
        Err(CheckpointError::TautologicalTestDetected { test_name }) => {
            assert_eq!(test_name, "test_tautology");
        }
        other => panic!("expected TautologicalTestDetected, got: {:?}", other),
    }
}

#[test]
fn test_inverse_preflight_verifies_legitimate_test() {
    let temp = TempDir::new("inverse_legitimate");
    let ws = temp.path();
    init_git_repo(ws);

    let src_dir = ws.join("src");
    let tests_dir = ws.join("tests");
    fs::create_dir_all(&src_dir).unwrap();
    fs::create_dir_all(&tests_dir).unwrap();

    // Configure test layer via gauntlet.toml
    fs::write(
        ws.join("gauntlet.toml"),
        r#"
[stack]
name = "custom"

[[layers]]
name = "test"
command = ["grep", "hello world", "src/lib.rs"]
"#,
    )
    .unwrap();

    // Baseline: greet returns "hello"
    fs::write(
        src_dir.join("lib.rs"),
        "pub fn greet() -> &'static str { \"hello\" }\n",
    )
    .unwrap();
    let _ = git_commit_all(ws, "chore: baseline");

    // GREEN modification: greet returns "hello world"
    fs::write(
        src_dir.join("lib.rs"),
        "pub fn greet() -> &'static str { \"hello world\" }\n",
    )
    .unwrap();
    // New test expecting "hello world" (which would fail on baseline "hello")
    fs::write(
        tests_dir.join("greet_test.rs"),
        "#[test]\nfn test_greet_world() { assert_eq!(dummy::greet(), \"hello world\"); }\n",
    )
    .unwrap();

    let evidences = verify_inverse_test_failure(ws, &["test_greet_world".to_string()])
        .expect("legitimate test should pass inverse preflight");

    assert_eq!(evidences.len(), 1);
    assert_eq!(evidences[0].test_identifier, "test_greet_world");
    assert_eq!(
        evidences[0].status,
        TestProvenanceStatus::InversePreflightVerified
    );
    assert!(!evidences[0].failure_output.is_empty());
}

#[tokio::test]
async fn test_checkpoint_green_fails_closed_on_born_green_with_strict_provenance() {
    let temp = TempDir::new("checkpoint_strict_provenance");
    let ws = temp.path();
    init_git_repo(ws);

    let tasks_dir = ws.join("tasks");
    let tests_dir = ws.join("tests");
    fs::create_dir_all(&tasks_dir).unwrap();
    fs::create_dir_all(&tests_dir).unwrap();

    // Setup active task 032
    fs::write(
        tasks_dir.join("032-test-provenance.md"),
        r#"---
type: Task Package
title: "Task 032: Test Provenance"
status: active
scope: ["src/**", "tests/**"]
test_refs: ["tests/provenance_test.rs"]
---
# Task 032
"#,
    )
    .unwrap();

    // RED phase: initial test
    fs::write(
        tests_dir.join("provenance_test.rs"),
        "#[test]\nfn test_initial() { assert_eq!(1, 2); }\n",
    )
    .unwrap();

    let _ = git_commit_all(ws, "test(032): add failing test [RED]");

    // Now during GREEN: add second test
    fs::write(
        tests_dir.join("provenance_test.rs"),
        "#[test]\nfn test_initial() { assert_eq!(1, 1); }\n#[test]\nfn test_born_green() { assert_eq!(2, 2); }\n",
    )
    .unwrap();

    let opts = CheckpointOptions::new(CheckpointPhase::Green, ws)
        .with_task_id("032-test-provenance")
        .with_strict_provenance(true);

    let res = run_checkpoint(&opts).await;
    match res {
        Err(CheckpointError::BornGreenTestDetected { tests }) => {
            assert_eq!(tests, vec!["test_born_green".to_string()]);
        }
        other => panic!("expected BornGreenTestDetected error, got: {:?}", other),
    }
}

#[tokio::test]
async fn test_collect_task_test_provenance_and_verification_report() {
    let temp = TempDir::new("provenance_report");
    let ws = temp.path();
    init_git_repo(ws);

    let tests_dir = ws.join("tests");
    fs::create_dir_all(&tests_dir).unwrap();

    fs::write(
        tests_dir.join("sample_test.rs"),
        "#[test]\nfn test_alpha() { assert_eq!(1, 2); }\n",
    )
    .unwrap();

    let _ = git_commit_all(ws, "test(032): initial red test [RED]");

    let prov =
        xgauntlet_core::features::checkpoint::provenance::collect_task_test_provenance(ws, "032")
            .expect("should collect provenance");

    assert_eq!(prov.len(), 1);
    assert_eq!(prov[0].test_identifier, "test_alpha");
    assert_eq!(prov[0].status, TestProvenanceStatus::RedCheckpointVerified);
    assert!(!prov[0].failure_digest.is_empty());

    let mut report = xgauntlet_core::features::evidence::VerificationReport::new_local("032");
    report.test_provenance = Some(prov);

    let md = xgauntlet_core::features::evidence::generate_report_markdown(&report, None);
    assert!(md.contains("### 🧪 Test Provenance & Born-Green Verification"));
    assert!(md.contains("test_alpha"));
    assert!(md.contains("RedCheckpointVerified"));
}
