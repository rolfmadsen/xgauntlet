//! Integration tests for Gauntlet Execution Engine (Task 005).

use std::fs;
use std::path::Path;
use std::path::PathBuf;
use xgauntlet_core::features::diagnostics::{DiagnosticParser, FindingType};
use xgauntlet_core::features::gauntlet::{
    execute_gauntlet_pipeline, run_gauntlet, GauntletOptions, LayerDefinition,
    LayerExecutionStatus, LayerRequirement,
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
        let path = std::env::temp_dir().join(format!("xgauntlet_test_{}", unique));
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

#[tokio::test]
async fn test_layer_definition_and_execution_success() {
    let layer = LayerDefinition::new("echo-test", vec!["echo".to_string(), "hello".to_string()])
        .with_timeout(10.0);

    let result =
        xgauntlet_core::features::gauntlet::runner::execute_layer(&layer, Path::new(".")).await;
    assert_eq!(result.name, "echo-test");
    assert_eq!(result.exit_code, 0);
    assert!(result.passed);
    assert_eq!(result.status, LayerExecutionStatus::Passed);
    assert!(result.output.contains("hello"));
    assert!(result.duration_seconds >= 0.0);
}

#[tokio::test]
async fn test_fail_closed_sequential_gauntlet() {
    let layers = vec![
        LayerDefinition::new("pass-1", vec!["echo".to_string(), "first".to_string()]),
        LayerDefinition::new("fail-required", vec!["false".to_string()])
            .with_requirement(LayerRequirement::Required),
        LayerDefinition::new(
            "should-not-run",
            vec!["echo".to_string(), "third".to_string()],
        ),
    ];

    let report = run_gauntlet(&layers, Path::new(".")).await;
    assert!(!report.success);
    assert_eq!(report.layers.len(), 2);
    assert_eq!(report.layers[0].status, LayerExecutionStatus::Passed);
    assert_eq!(report.layers[1].status, LayerExecutionStatus::Failed);
    assert_eq!(report.layers[1].exit_code, 1);
}

#[tokio::test]
async fn test_optional_layer_failure_allows_continuation() {
    let layers = vec![
        LayerDefinition::new("optional-failure", vec!["false".to_string()]).optional(),
        LayerDefinition::new(
            "required-pass",
            vec!["echo".to_string(), "continued".to_string()],
        ),
    ];

    let report = run_gauntlet(&layers, Path::new(".")).await;
    assert!(report.success);
    assert_eq!(report.layers.len(), 2);
    assert_eq!(report.layers[0].status, LayerExecutionStatus::Failed);
    assert_eq!(report.layers[1].status, LayerExecutionStatus::Passed);
}

#[tokio::test]
async fn test_process_timeout_handling() {
    let layer = LayerDefinition::new("sleep-timeout", vec!["sleep".to_string(), "10".to_string()])
        .with_timeout(0.05);

    let result =
        xgauntlet_core::features::gauntlet::runner::execute_layer(&layer, Path::new(".")).await;
    assert_eq!(result.status, LayerExecutionStatus::TimedOut);
    assert_eq!(result.exit_code, 124);
    assert!(!result.passed);
    assert!(result.output.contains("timed out"));
}

#[tokio::test]
async fn test_unavailable_binary_handling() {
    let layer = LayerDefinition::new(
        "missing-binary",
        vec!["__totally_non_existent_binary_12345__".to_string()],
    );

    let result =
        xgauntlet_core::features::gauntlet::runner::execute_layer(&layer, Path::new(".")).await;
    assert_eq!(result.status, LayerExecutionStatus::Unavailable);
    assert_eq!(result.exit_code, 127);
    assert!(!result.passed);
}

#[test]
fn test_diagnostic_finding_extraction() {
    let parser = DiagnosticParser::new();

    // Clippy warning/error output
    let clippy_output = r#"
error[E0308]: mismatched types
 --> src/main.rs:15:9
  |
15|     let x: u32 = "hello";
  |            ---   ^^^^^^^ expected `u32`, found `&str`
"#;
    let findings = parser.extract_findings(
        "lint",
        &["cargo".to_string(), "clippy".to_string()],
        1,
        clippy_output,
    );
    assert!(!findings.is_empty());
    assert_eq!(findings[0].file_path, "src/main.rs");
    assert_eq!(findings[0].line_number, Some(15));
    assert!(findings[0].message.contains("mismatched types"));

    // Cargo test failure output
    let test_output = r#"
running 1 test
test tests::test_bad_math ... FAILED

failures:

---- tests::test_bad_math stdout ----
thread 'tests::test_bad_math' panicked at src/lib.rs:42:5:
assertion `left == right` failed
  left: 4
 right: 5
"#;
    let test_findings = parser.extract_findings(
        "unit",
        &["cargo".to_string(), "test".to_string()],
        101,
        test_output,
    );
    assert!(!test_findings.is_empty());
    assert_eq!(test_findings[0].file_path, "src/lib.rs");
    assert_eq!(test_findings[0].line_number, Some(42));
    assert_eq!(test_findings[0].finding_type, FindingType::TestFailure);
}

#[tokio::test]
async fn test_self_mutation_invariant_enforcement() {
    let tmp = TempDir::new("gauntlet_test");
    let ws = tmp.path();

    // Setup workspace files
    fs::create_dir_all(ws.join("src")).unwrap();
    fs::create_dir_all(ws.join("tasks")).unwrap();
    fs::write(
        ws.join("src/lib.rs"),
        "pub fn add(a: i32, b: i32) -> i32 { a + b }\n",
    )
    .unwrap();
    fs::write(
        ws.join("tasks/001-test.md"),
        "---\ntype: Task Package\ntitle: Test\nstatus: active\n---\n# Task 001\n## Acceptance Criteria\n- [x] Done\n",
    ).unwrap();

    // A layer that illegally mutates the workspace
    let command = if cfg!(windows) {
        vec![
            "cmd".to_string(),
            "/C".to_string(),
            "echo // mutated >> src\\lib.rs".to_string(),
        ]
    } else {
        vec![
            "sh".to_string(),
            "-c".to_string(),
            "echo '// mutated' >> src/lib.rs".to_string(),
        ]
    };
    let layers = vec![LayerDefinition::new("mutating-layer", command)];

    let opts = GauntletOptions {
        task_id: Some("001-test".to_string()),
        save_evidence: false,
        layers: Some(layers),
    };

    let outcome = execute_gauntlet_pipeline(ws, &opts).await.unwrap();
    assert_eq!(outcome.report.verdict, "FAILED");
    assert!(outcome.is_self_mutated);
    assert!(outcome
        .report
        .diagnostics
        .iter()
        .any(|d| d.finding_type == FindingType::InvariantViolation));
}

#[tokio::test]
async fn test_end_to_end_pipeline_and_evidence_generation() {
    let tmp = TempDir::new("gauntlet_test");
    let ws = tmp.path();

    fs::create_dir_all(ws.join("src")).unwrap();
    fs::create_dir_all(ws.join("tasks")).unwrap();
    fs::write(ws.join("src/lib.rs"), "pub fn ok() -> bool { true }\n").unwrap();
    fs::write(
        ws.join("tasks/001-init.md"),
        "---\ntype: Task Package\ntitle: Init\nstatus: active\n---\n# Task 001\n## Acceptance Criteria\n- [x] Initial criteria\n",
    ).unwrap();

    let layers = vec![LayerDefinition::new(
        "test-pass",
        vec!["echo".to_string(), "pass".to_string()],
    )];

    let opts = GauntletOptions {
        task_id: Some("001-init".to_string()),
        save_evidence: true,
        layers: Some(layers),
    };

    let outcome = execute_gauntlet_pipeline(ws, &opts).await.unwrap();
    assert_eq!(outcome.report.verdict, "PASSED");
    assert!(!outcome.is_self_mutated);
    assert_eq!(outcome.report.checks.len(), 1);
    assert_eq!(outcome.report.checks[0].name, "test-pass");
    assert_eq!(outcome.report.checks[0].exit_code, 0);

    // Evidence artifacts should be written
    assert!(ws.join("verification-report.json").is_file());
    assert!(ws.join("evidence.md").is_file());
}
