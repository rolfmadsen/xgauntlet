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

use xgauntlet_core::features::checkpoint::engine::run_checkpoint;
use xgauntlet_core::features::checkpoint::models::{CheckpointOptions, CheckpointPhase};
use xgauntlet_core::features::checkpoint::preflight::{
    resolve_scoped_test_command, run_preflight_check, RustTestTarget,
};
use xgauntlet_core::features::checkpoint::provenance::{
    is_fast_track_eligible, TestProvenanceStatus,
};
use xgauntlet_core::features::gauntlet::{execute_gauntlet_pipeline, GauntletOptions};
use xgauntlet_core::features::tasks::check_task_specification;
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
        let _ = Command::new("git").args(["init"]).current_dir(ws).output();
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
            package: None,
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
    fs::write(
        tests_dir.join("features.rs"),
        "mod invoice_acceptance_test;",
    )
    .unwrap();
    let submod_file = tests_dir
        .join("features")
        .join("invoice_acceptance_test.rs");
    fs::write(&submod_file, "// submod").unwrap();

    let target =
        resolve_scoped_test_command(ws.path(), "tests/features/invoice_acceptance_test.rs");
    assert_eq!(
        target,
        RustTestTarget::HarnessSubmodule {
            package: None,
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
            package: None,
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
fn test_rust_target_resolver_multi_crate_workspace() {
    let ws = TempWorkspace::new("multi_crate_ws");
    let crate_dir = ws.path().join("crates").join("xgauntlet-core");
    let tests_dir = crate_dir.join("tests");
    let src_dir = crate_dir.join("src").join("features").join("auth");

    fs::create_dir_all(&tests_dir).unwrap();
    fs::create_dir_all(&src_dir).unwrap();

    fs::write(
        crate_dir.join("Cargo.toml"),
        "[package]\nname = \"xgauntlet-core\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::write(tests_dir.join("security_test.rs"), "// integration test").unwrap();
    fs::write(src_dir.join("token.rs"), "// lib module").unwrap();

    // 1. Integration test in crate subdirectory must include -p <crate-name>
    let target1 =
        resolve_scoped_test_command(ws.path(), "crates/xgauntlet-core/tests/security_test.rs");
    assert_eq!(
        target1,
        RustTestTarget::IntegrationTest {
            package: Some("xgauntlet-core".to_string()),
            target_name: "security_test".to_string(),
            filter: None,
            command: vec![
                "cargo".to_string(),
                "test".to_string(),
                "-p".to_string(),
                "xgauntlet-core".to_string(),
                "--test".to_string(),
                "security_test".to_string(),
            ],
        }
    );

    // 2. Lib module in crate subdirectory must include -p <crate-name>
    let target2 = resolve_scoped_test_command(
        ws.path(),
        "crates/xgauntlet-core/src/features/auth/token.rs",
    );
    assert_eq!(
        target2,
        RustTestTarget::LibModule {
            package: Some("xgauntlet-core".to_string()),
            module_path: "features::auth::token".to_string(),
            command: vec![
                "cargo".to_string(),
                "test".to_string(),
                "-p".to_string(),
                "xgauntlet-core".to_string(),
                "--lib".to_string(),
                "features::auth::token".to_string(),
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
        domain: None,
        feature: None,
        spec_ref: None,
        metadata: None,
    };

    // 1. Qualified: intent=Bug, scope.len() <= 3, no ADR/gauntlet changed
    let changed_files = vec![
        "src/parser.rs".to_string(),
        "tests/parser_test.rs".to_string(),
    ];
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

    // 6. Disqualified: wildcard globs in scope (* or **) even when scope.len() <= 3
    task.scope = vec!["src/**".to_string()];
    assert!(
        !is_fast_track_eligible(&task, &changed_files),
        "Wildcard glob 'src/**' must disqualify Fast-Track eligibility"
    );
    task.scope = vec!["src/*.rs".to_string()];
    assert!(
        !is_fast_track_eligible(&task, &changed_files),
        "Wildcard glob 'src/*.rs' must disqualify Fast-Track eligibility"
    );
}

#[tokio::test]
async fn test_fast_track_done_without_red_commit_fails_closed() {
    let ws = TempWorkspace::new("fast_track_requires_red");
    init_git_repo(ws.path());

    let src_dir = ws.path().join("src");
    let tests_dir = ws.path().join("tests");
    let tasks_dir = ws.path().join("tasks");
    fs::create_dir_all(&src_dir).unwrap();
    fs::create_dir_all(&tests_dir).unwrap();
    fs::create_dir_all(&tasks_dir).unwrap();

    fs::write(ws.path().join(".gitignore"), "target/\n").unwrap();
    fs::write(src_dir.join("lib.rs"), "pub fn compute() -> i32 { 10 }").unwrap();
    fs::write(
        ws.path().join("Cargo.toml"),
        "[package]\nname = \"math_pkg\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n",
    )
    .unwrap();
    fs::write(
        ws.path().join("gauntlet.toml"),
        "[project]\nname = \"math_pkg\"\nversion = \"0.1.0\"\n\n[[layers]]\nname = \"unit\"\ncommand = [\"true\"]\n",
    )
    .unwrap();

    let _ = Command::new("git")
        .args(["add", "."])
        .current_dir(ws.path())
        .output();
    let _ = Command::new("git")
        .args(["commit", "-m", "chore: baseline"])
        .current_dir(ws.path())
        .output();

    // Deliver fix + test without any preceding RED commit
    fs::write(src_dir.join("lib.rs"), "pub fn compute() -> i32 { 20 }").unwrap();
    fs::write(
        tests_dir.join("calc_test.rs"),
        "#[test]\nfn test_fixed_compute() { assert_eq!(math_pkg::compute(), 20); }\n",
    )
    .unwrap();
    fs::write(
        tasks_dir.join("043-bugfix.md"),
        r#"---
type: Task Package
title: "Task 043: Bugfix"
status: active
intent: bug
scope:
  - "src/lib.rs"
test_refs:
  - "tests/calc_test.rs"
---
# Task 043: Bugfix
**Status**: `ACTIVE`
**Intent**: `🐛 BUG FIX`
## 🎯 Formål
Fix compute return value.
## 📋 Acceptance Criteria
- [x] Fix calculation bug
## 🚫 Must NOT
- Must not break existing contracts
"#,
    )
    .unwrap();

    // 1. checkpoint -p done preflight MUST fail closed with RED guidance
    let opts = CheckpointOptions::new(CheckpointPhase::Done, ws.path()).with_task_id("043");
    let preflight_res = run_preflight_check(&opts).await;
    assert!(
        preflight_res.is_err(),
        "Fast-Track DONE without preceding RED commit must fail closed"
    );
    let err_msg = format!("{:?}", preflight_res.err().unwrap()).to_ascii_lowercase();
    assert!(
        err_msg.contains("red"),
        "Error message must guide user to run checkpoint -p red first: {err_msg}"
    );

    // 2. execute_gauntlet_pipeline MUST also fail closed with RED guidance (no worktree inverse proof)
    let outcome = execute_gauntlet_pipeline(
        ws.path(),
        &GauntletOptions {
            task_id: Some("043".to_string()),
            save_evidence: false,
            layers: None,
        },
    )
    .await
    .unwrap();
    assert!(
        !outcome.success,
        "Pipeline must reject Fast-Track verification without preceding RED commit"
    );
    assert!(
        outcome
            .report
            .diagnostics
            .iter()
            .any(|d| d.message.to_ascii_lowercase().contains("red")),
        "Diagnostics must guide user to run checkpoint -p red: {:?}",
        outcome.report.diagnostics
    );
}

#[tokio::test]
async fn test_phase_machine_blocks_skipping_phases_for_non_fast_track() {
    let ws = TempWorkspace::new("phase_skip_blocked");
    init_git_repo(ws.path());

    // Task with intent: feature (NOT fast-track eligible)
    let tasks_dir = ws.path().join("tasks");
    fs::create_dir_all(&tasks_dir).unwrap();
    fs::write(
        tasks_dir.join("042-new-feature.md"),
        r#"---
type: Task Package
title: "Task 042: New Feature"
status: active
scope:
  - "src/feature.rs"
test_refs:
  - "tests/feature_test.rs"
---
# Task 042: New Feature
**Status**: `ACTIVE`
**Intent**: `🚀 NEW FEATURE`
## 📋 Acceptance Criteria
- [x] Feature works
"#,
    )
    .unwrap();

    let opts = CheckpointOptions {
        workspace: ws.path().to_path_buf(),
        phase: CheckpointPhase::Done,
        task_id: Some("042".to_string()),
        message: None,
        allow_empty: false,
        skip_verify: false,
        strict_provenance: false,
        allow_born_green: false,
    };

    // Calling done directly without RED/GREEN for a FEATURE must fail closed
    let res = run_preflight_check(&opts).await;
    assert!(
        res.is_err(),
        "Skipping TDD phases for feature task must fail closed"
    );
    let err_msg = format!("{:?}", res.err().unwrap());
    assert!(
        err_msg.to_ascii_lowercase().contains("fast-track")
            || err_msg.to_ascii_lowercase().contains("phase"),
        "Expected error mentioning fast-track or phase skip: {err_msg}"
    );
}

#[tokio::test]
async fn test_fast_track_two_step_red_to_done_pipeline() {
    let ws = TempWorkspace::new("fast_track_red_to_done");
    init_git_repo(ws.path());

    // 1. Commit baseline
    let src_dir = ws.path().join("src");
    let tests_dir = ws.path().join("tests");
    let tasks_dir = ws.path().join("tasks");
    fs::create_dir_all(&src_dir).unwrap();
    fs::create_dir_all(&tests_dir).unwrap();
    fs::create_dir_all(&tasks_dir).unwrap();

    fs::write(ws.path().join(".gitignore"), "target/\n").unwrap();
    fs::write(src_dir.join("lib.rs"), "pub fn compute() -> i32 { 10 }").unwrap();
    fs::write(
        ws.path().join("Cargo.toml"),
        r#"[package]
name = "math_pkg"
version = "0.1.0"
edition = "2021"

[lib]
path = "src/lib.rs"
"#,
    )
    .unwrap();

    fs::write(
        ws.path().join("gauntlet.toml"),
        r#"[project]
name = "math_pkg"
version = "0.1.0"

[[layers]]
name = "unit"
command = ["cargo", "test"]
"#,
    )
    .unwrap();

    let _ = Command::new("cargo")
        .args(["check"])
        .current_dir(ws.path())
        .output();
    let _ = Command::new("git")
        .args(["add", "."])
        .current_dir(ws.path())
        .output();
    let _ = Command::new("git")
        .args(["commit", "-m", "chore: baseline"])
        .current_dir(ws.path())
        .output();

    // 2. Write failing regression test and create RED checkpoint (step 1 of 2-step Fast-Track)
    fs::write(
        tests_dir.join("calc_test.rs"),
        r#"
#[test]
fn test_fixed_compute() {
    assert_eq!(math_pkg::compute(), 20);
}
"#,
    )
    .unwrap();

    fs::write(
        tasks_dir.join("043-bugfix.md"),
        r#"---
type: Task Package
title: "Task 043: Bugfix"
status: active
intent: bug
scope:
  - "src/lib.rs"
test_refs:
  - "tests/calc_test.rs"
---
# Task 043: Bugfix
**Status**: `ACTIVE`
**Intent**: `🐛 BUG FIX`
## 🎯 Formål
Fix calculation bug.
## 📋 Acceptance Criteria
- [x] Fix calculation bug
## 🚫 Must NOT
- Must not regress
"#,
    )
    .unwrap();

    let red_res = run_checkpoint(&CheckpointOptions::new(CheckpointPhase::Red, ws.path()))
        .await
        .expect("RED checkpoint must pass when regression test fails against baseline");
    assert_eq!(red_res.phase, CheckpointPhase::Red);

    // 3. Apply fix directly in scope (no GREEN/REFACTOR commits needed)
    fs::write(src_dir.join("lib.rs"), "pub fn compute() -> i32 { 20 }").unwrap();

    // 4. Pipeline verify (and DONE checkpoint) must succeed with fast_track_red_to_done proof_mode
    let outcome = execute_gauntlet_pipeline(
        ws.path(),
        &GauntletOptions {
            task_id: Some("043".to_string()),
            save_evidence: false,
            layers: None,
        },
    )
    .await
    .unwrap();

    assert_eq!(
        outcome.report.verdict,
        xgauntlet_core::VerificationVerdict::Passed
    );
    assert_eq!(
        outcome.report.proof_mode.as_deref(),
        Some("fast_track_red_to_done"),
        "Pipeline must record proof_mode = fast_track_red_to_done for 2-step Fast-Track"
    );
    assert!(
        outcome
            .report
            .test_provenance
            .as_ref()
            .is_some_and(|list| list
                .iter()
                .any(|p| p.status == TestProvenanceStatus::RedCheckpointVerified)),
        "Provenance must contain RedCheckpointVerified entry"
    );
}

#[test]
fn test_small_bugfix_exempt_from_feature_spec_mapping() {
    let ws = TempWorkspace::new("small_bugfix_no_spec_block");
    fs::create_dir_all(ws.path().join("tasks")).unwrap();
    fs::create_dir_all(ws.path().join("specs")).unwrap();
    fs::write(
        ws.path().join("CONTEXT.md"),
        "**Task**:\nAn executable unit of work.\n_Avoid_: Ticket.\n",
    )
    .unwrap();
    fs::write(
        ws.path().join("gauntlet.toml"),
        "stack = \"rust\"\n[architecture]\nfeature_root = \"src/features\"\nisolate_features = true\n",
    )
    .unwrap();

    let write_task = |name: &str, scope_yaml: &str, extra_frontmatter: &str| -> PathBuf {
        let p = ws.path().join("tasks").join(name);
        fs::write(
            &p,
            format!(
                "---\ntype: Task Package\ntitle: \"Task: Bugfix\"\ndescription: \"Fix\"\nstatus: active\nintent: bug\n{extra_frontmatter}generated: {{ by: process:xgauntlet, at: \"2026-11-14T10:00:00Z\" }}\ntags: [bug]\n{scope_yaml}\ntest_refs: [\"tests/bug_test.rs\"]\n---\n\n# Task\n\n## 🎯 Formål\nFix defect.\n\n## 📋 Acceptance Criteria\n- [ ] AC1: Fixed.\n\n## 🚫 Must NOT\n- Never break invariants.\n"
            ),
        )
        .unwrap();
        p
    };

    // 1. Small bugfix (<= 3 concrete paths) touching feature_root without feature/spec_ref -> MUST PASS
    let small_bug = write_task(
        "010-small-bug.md",
        "scope:\n  - \"src/features/auth/login.rs\"\n  - \"src/features/auth/token.rs\"",
        "",
    );
    let rep_small = check_task_specification(&small_bug, ws.path());
    assert!(
        rep_small.is_valid,
        "Small bugfix (<= 3 concrete files) must not require feature-spec mapping, got: {:?}",
        rep_small.diagnostics
    );

    // 2. Wildcard glob in bugfix scope -> MUST still require feature-spec mapping
    let wildcard_bug = write_task(
        "011-wildcard-bug.md",
        "scope:\n  - \"src/features/auth/**\"",
        "",
    );
    let rep_wildcard = check_task_specification(&wildcard_bug, ws.path());
    assert!(
        rep_wildcard
            .diagnostics
            .iter()
            .any(|d| d.tool_name == "MISSING_TASK_FEATURE_MAPPING"),
        "Wildcard bugfix scope must not bypass feature mapping, got: {:?}",
        rep_wildcard.diagnostics
    );

    // 3. Large bugfix (> 3 files) -> MUST still require feature-spec mapping
    let large_bug = write_task(
        "012-large-bug.md",
        "scope:\n  - \"src/features/a.rs\"\n  - \"src/features/b.rs\"\n  - \"src/features/c.rs\"\n  - \"src/features/d.rs\"",
        "",
    );
    let rep_large = check_task_specification(&large_bug, ws.path());
    assert!(
        rep_large
            .diagnostics
            .iter()
            .any(|d| d.tool_name == "MISSING_TASK_FEATURE_MAPPING"),
        "Bugfix with scope.len() > 3 must require feature mapping, got: {:?}",
        rep_large.diagnostics
    );

    // 4. Small bugfix WITH explicit feature/spec_ref -> MUST still validate missing spec file
    let explicit_bug = write_task(
        "013-explicit-feature-bug.md",
        "scope:\n  - \"src/features/auth/login.rs\"",
        "feature: auth\nspec_ref: \"specs/auth.md\"\n",
    );
    let rep_explicit = check_task_specification(&explicit_bug, ws.path());
    assert!(
        rep_explicit
            .diagnostics
            .iter()
            .any(|d| d.tool_name == "MISSING_FEATURE_SPEC_FILE"),
        "Explicit feature/spec_ref on bugfix must still validate spec file existence, got: {:?}",
        rep_explicit.diagnostics
    );
}
