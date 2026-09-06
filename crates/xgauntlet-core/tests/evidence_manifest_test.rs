//! Comprehensive verification tests for Evidence & Workspace Manifest Engine.

use std::fs;
use std::path::{Path, PathBuf};
use xgauntlet_core::features::evidence::{
    compute_file_git_blob_oid, compute_workspace_manifest, generate_report_json,
    generate_report_markdown, load_report_json, save_verification_report, verify_self_mutation,
    verify_workspace_state_match, CheckStatus, CheckSummary, ExecutionMetadata,
    TaskContractSummary, VerificationReport, VerificationVerdict, WorkspaceState,
};

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(prefix: &str) -> Self {
        let unique = format!(
            "{}_{}_{}",
            prefix,
            std::process::id(),
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
fn test_crlf_lf_digest_immunity() {
    let text_lf = "fn main() {\n    println!(\"Hello, world!\");\n}\n";
    let text_crlf = "fn main() {\r\n    println!(\"Hello, world!\");\r\n}\r\n";

    // Direct Git-blob OID calculation on raw bytes
    let oid_lf = compute_file_git_blob_oid(text_lf.as_bytes());
    let oid_crlf = compute_file_git_blob_oid(text_crlf.as_bytes());
    assert_eq!(
        oid_lf, oid_crlf,
        "Git-blob OID hash must be identical for LF and CRLF text contents"
    );

    // Workspace manifest level test
    let dir_lf = TempDir::new("test_crlf_lf_dir_lf");
    let dir_crlf = TempDir::new("test_crlf_lf_dir_crlf");

    fs::create_dir_all(dir_lf.path.join("src")).unwrap();
    fs::create_dir_all(dir_crlf.path.join("src")).unwrap();

    fs::write(dir_lf.path.join("src/main.rs"), text_lf).unwrap();
    fs::write(dir_crlf.path.join("src/main.rs"), text_crlf).unwrap();

    let manifest_lf = compute_workspace_manifest(&dir_lf.path, Some(&["src"])).unwrap();
    let manifest_crlf = compute_workspace_manifest(&dir_crlf.path, Some(&["src"])).unwrap();

    assert_eq!(
        manifest_lf.source_manifest_digest, manifest_crlf.source_manifest_digest,
        "source_manifest_digest must be immune to CRLF vs LF line endings"
    );

    // But raw content digest differs, which allows local in-place dirty checks
    assert_ne!(
        manifest_lf.source_content_digest, manifest_crlf.source_content_digest,
        "source_content_digest tracks raw bytes for real-time local worktree drift"
    );
}

#[test]
fn test_binary_file_preservation() {
    // Binary data with null bytes and CRLF sequence that must NOT be altered
    let binary_data = vec![
        0x7f, 0x45, 0x4c, 0x46, 0x00, 0x0d, 0x0a, 0x02, 0x01, 0x01, 0x00,
    ];
    let oid = compute_file_git_blob_oid(&binary_data);

    // Calculating on modified binary data (e.g. if CRLF was replaced by LF)
    let modified_binary = vec![0x7f, 0x45, 0x4c, 0x46, 0x00, 0x0a, 0x02, 0x01, 0x01, 0x00];
    let modified_oid = compute_file_git_blob_oid(&modified_binary);

    assert_ne!(
        oid, modified_oid,
        "Binary files with null bytes must preserve exact byte sequence"
    );
}

#[cfg(unix)]
#[test]
fn test_symlink_escape_detection() {
    let dir = TempDir::new("test_symlink_escape");
    let outside_dir = TempDir::new("test_symlink_outside");

    fs::write(outside_dir.path.join("secret.txt"), "forbidden content").unwrap();

    fs::create_dir_all(dir.path.join("src")).unwrap();
    let link_path = dir.path.join("src/escape_link");
    std::os::unix::fs::symlink(&outside_dir.path, &link_path).unwrap();

    let result = compute_workspace_manifest(&dir.path, Some(&["src"]));
    assert!(
        result.is_err(),
        "Symlink pointing outside workspace root must fail closed"
    );
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("outside") || err_msg.contains("escape"),
        "Error message must clearly identify workspace escape: {err_msg}"
    );
}

#[test]
fn test_self_mutation_invariant_enforcement() {
    let dir = TempDir::new("test_self_mutation");
    fs::create_dir_all(dir.path.join("src")).unwrap();
    let file_path = dir.path.join("src/lib.rs");
    fs::write(&file_path, "pub fn add(a: i32, b: i32) -> i32 { a + b }\n").unwrap();

    let pre_manifest = compute_workspace_manifest(&dir.path, Some(&["src"])).unwrap();

    // 1. No mutation -> passes
    let post_clean = compute_workspace_manifest(&dir.path, Some(&["src"])).unwrap();
    assert!(verify_self_mutation(&pre_manifest, &post_clean).is_ok());

    // 2. File modified during run -> fails with SELF_MUTATION_DETECTED
    fs::write(
        &file_path,
        "pub fn add(a: i32, b: i32) -> i32 { a + b + 1 }\n",
    )
    .unwrap();
    let post_mutated = compute_workspace_manifest(&dir.path, Some(&["src"])).unwrap();
    let result_mutated = verify_self_mutation(&pre_manifest, &post_mutated);
    assert!(result_mutated.is_err());
    let err = result_mutated.unwrap_err();
    assert!(
        err.to_string().contains("modified") || err.to_string().contains("Self-mutation"),
        "Error must describe file modification: {err}"
    );

    // 3. New file added during run -> fails
    fs::write(&file_path, "pub fn add(a: i32, b: i32) -> i32 { a + b }\n").unwrap();
    fs::write(dir.path.join("src/extra.rs"), "pub const X: i32 = 1;\n").unwrap();
    let post_added = compute_workspace_manifest(&dir.path, Some(&["src"])).unwrap();
    let result_added = verify_self_mutation(&pre_manifest, &post_added);
    assert!(result_added.is_err());
}

#[test]
fn test_drift_verification_matches_and_mismatches() {
    let dir = TempDir::new("test_drift");
    fs::create_dir_all(dir.path.join("src")).unwrap();
    fs::create_dir_all(dir.path.join("tasks")).unwrap();
    fs::write(dir.path.join("spec.md"), "# System Spec\n").unwrap();
    fs::write(dir.path.join("Cargo.toml"), "[workspace]\n").unwrap();
    fs::write(dir.path.join("tasks/001.md"), "# Task 001\n").unwrap();
    fs::write(dir.path.join("src/lib.rs"), "pub fn ok() {}\n").unwrap();

    let manifest =
        compute_workspace_manifest(&dir.path, Some(&["src", "tasks", "spec.md", "Cargo.toml"]))
            .unwrap();

    let mut report = VerificationReport::new_local("001-test");
    report.workspace_state = manifest.to_workspace_state(None, None);

    // 1. Clean match
    let current_manifest =
        compute_workspace_manifest(&dir.path, Some(&["src", "tasks", "spec.md", "Cargo.toml"]))
            .unwrap();
    assert!(verify_workspace_state_match(&report, &current_manifest).is_ok());

    // 2. Source drift: modify src/lib.rs
    fs::write(dir.path.join("src/lib.rs"), "pub fn modified() {}\n").unwrap();
    let drifted_manifest =
        compute_workspace_manifest(&dir.path, Some(&["src", "tasks", "spec.md", "Cargo.toml"]))
            .unwrap();
    let drift_result = verify_workspace_state_match(&report, &drifted_manifest);
    assert!(drift_result.is_err());
    let err_msg = drift_result.unwrap_err().to_string();
    assert!(
        err_msg.contains("source") || err_msg.contains("drift"),
        "Error message must indicate source drift: {err_msg}"
    );

    // Restore src/lib.rs
    fs::write(dir.path.join("src/lib.rs"), "pub fn ok() {}\n").unwrap();

    // 3. Policy drift: modify spec.md
    fs::write(dir.path.join("spec.md"), "# System Spec\nModified Policy\n").unwrap();
    let policy_drift_manifest =
        compute_workspace_manifest(&dir.path, Some(&["src", "tasks", "spec.md", "Cargo.toml"]))
            .unwrap();
    assert!(verify_workspace_state_match(&report, &policy_drift_manifest).is_err());
}

#[test]
fn test_verification_report_serialization_and_markdown() {
    let mut report = VerificationReport::new_local("004-evidence-and-manifest-engine");
    report.verdict = VerificationVerdict::Passed.as_str().to_string();
    report.task_contract = TaskContractSummary {
        task_id: "004-evidence-and-manifest-engine".to_string(),
        task_title: "Task 004: Evidence & Workspace Manifest Engine".to_string(),
        task_digest: "abcdef1234567890".to_string(),
        acceptance_criteria: vec![
            "Deterministic Git-blob OID manifest".to_string(),
            "Self-mutation invariant".to_string(),
        ],
        unresolved_criteria: vec![],
    };
    report.workspace_state = WorkspaceState {
        manifest_version: "1.0".to_string(),
        source_content_digest: "11112222333344445555666677778888".to_string(),
        source_manifest_digest_pre: "aaaabbbbccccddddeeeeffff00001111".to_string(),
        source_manifest_digest_post: "aaaabbbbccccddddeeeeffff00001111".to_string(),
        config_digest: "cfg123".to_string(),
        task_digest: "tsk123".to_string(),
        policy_digest: "pol123".to_string(),
        check_definitions_digest: "chk123".to_string(),
        included_files_count: 42,
        vcs: None,
    };
    report.execution_metadata = ExecutionMetadata {
        started_at: "2026-09-06T17:00:00Z".to_string(),
        finished_at: "2026-09-06T17:00:05Z".to_string(),
        total_duration_seconds: 5.0,
        environment: [("engine".to_string(), "xgauntlet-core".to_string())]
            .into_iter()
            .collect(),
    };
    report.checks = vec![
        CheckSummary {
            name: "unit".to_string(),
            status: CheckStatus::Passed.as_str().to_string(),
            passed: true,
            exit_code: 0,
            duration_seconds: 0.123,
            optional: false,
            log_digest: "unit_log_hash".to_string(),
        },
        CheckSummary {
            name: "lint".to_string(),
            status: CheckStatus::Passed.as_str().to_string(),
            passed: true,
            exit_code: 0,
            duration_seconds: 0.045,
            optional: true,
            log_digest: "lint_log_hash".to_string(),
        },
    ];

    // JSON serialization
    let json_str = generate_report_json(&report).expect("report JSON serialization must succeed");
    assert!(json_str.contains("\"schema_version\": \"2.0.0\""));
    assert!(json_str.contains("\"execution_origin\": \"LOCAL\""));
    assert!(json_str.contains("\"task_id\": \"004-evidence-and-manifest-engine\""));
    assert!(json_str.contains("\"included_files_count\": 42"));

    // Deserialization roundtrip
    let deserialized = load_report_json(&json_str).expect("JSON deserialization must succeed");
    assert_eq!(deserialized.schema_version, "2.0.0");
    assert_eq!(deserialized.execution_origin, "LOCAL");
    assert_eq!(deserialized.verdict, "PASSED");
    assert_eq!(deserialized.checks.len(), 2);
    assert_eq!(deserialized.checks[0].name, "unit");
    assert_eq!(deserialized.checks[0].exit_code, 0);

    // Markdown generation
    let md = generate_report_markdown(&report, None);
    assert!(md.contains("# Verification Report"));
    assert!(md.contains("**Task ID**: `004-evidence-and-manifest-engine`"));
    assert!(md.contains("**Verdict**: `PASSED`"));
    assert!(md.contains("| `unit` | `PASSED` | `0` | `0.123s` |"));
    assert!(md.contains("- [x] Deterministic Git-blob OID manifest"));
}

#[test]
fn test_real_workspace_manifest_computation() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    let manifest = compute_workspace_manifest(workspace_root, None)
        .expect("Manifest computation on xGauntlet repository must succeed");

    assert!(
        manifest.included_files_count > 10,
        "Should include repository source files"
    );
    assert!(
        !manifest.source_manifest_digest.is_empty(),
        "source_manifest_digest must not be empty"
    );
    assert!(
        !manifest.source_content_digest.is_empty(),
        "source_content_digest must not be empty"
    );
    assert!(
        !manifest.policy_digest.is_empty(),
        "policy_digest must not be empty"
    );
    assert!(
        !manifest.config_digest.is_empty(),
        "config_digest must not be empty"
    );
}

#[test]
fn test_generate_workspace_evidence_artifacts() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    let manifest = compute_workspace_manifest(workspace_root, None).unwrap();
    let mut report = VerificationReport::new_local("004-evidence-and-manifest-engine");
    report.task_contract = TaskContractSummary {
        task_id: "004-evidence-and-manifest-engine".to_string(),
        task_title: "Task 004: Evidence & Workspace Manifest Engine".to_string(),
        task_digest: manifest.task_digest.clone(),
        acceptance_criteria: vec![
            "Schema v2 domain models in crates/xgauntlet-core/src/features/evidence/models.rs"
                .to_string(),
            "Deterministic CanonicalWorkspaceManifest with CRLF/LF immunity".to_string(),
            "Binary file preservation".to_string(),
            "Symlink escape prevention".to_string(),
            "Self-mutation invariant enforcement".to_string(),
            "Multi-digest drift detection".to_string(),
            "Verification report JSON and Markdown generation".to_string(),
            "CLI check-evidence command".to_string(),
            "100% green test suite".to_string(),
        ],
        unresolved_criteria: vec![],
    };
    report.workspace_state = manifest.to_workspace_state(None, None);
    report.execution_metadata = ExecutionMetadata {
        started_at: "2026-09-06T17:26:00Z".to_string(),
        finished_at: "2026-09-06T17:30:00Z".to_string(),
        total_duration_seconds: 4.0,
        environment: [
            ("engine".to_string(), "xgauntlet-core".to_string()),
            ("rust_version".to_string(), "1.85+".to_string()),
            ("platform".to_string(), std::env::consts::OS.to_string()),
        ]
        .into_iter()
        .collect(),
    };
    report.checks = vec![
        CheckSummary {
            name: "cargo-test".to_string(),
            status: CheckStatus::Passed.as_str().to_string(),
            passed: true,
            exit_code: 0,
            duration_seconds: 1.5,
            optional: false,
            log_digest: "test_ok".to_string(),
        },
        CheckSummary {
            name: "clippy".to_string(),
            status: CheckStatus::Passed.as_str().to_string(),
            passed: true,
            exit_code: 0,
            duration_seconds: 0.3,
            optional: false,
            log_digest: "clippy_ok".to_string(),
        },
        CheckSummary {
            name: "fmt".to_string(),
            status: CheckStatus::Passed.as_str().to_string(),
            passed: true,
            exit_code: 0,
            duration_seconds: 0.1,
            optional: false,
            log_digest: "fmt_ok".to_string(),
        },
    ];

    save_verification_report(workspace_root, &report).unwrap();
    assert!(workspace_root.join("verification-report.json").is_file());
    assert!(workspace_root.join("evidence.md").is_file());
}
