use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static TEST_COUNTER: AtomicU64 = AtomicU64::new(1);

#[test]
fn test_cli_task_id_alias_supported_on_verify() {
    let bin = env!("CARGO_BIN_EXE_xgauntlet");
    let output = Command::new(bin)
        .args(["verify", "--help"])
        .output()
        .expect("failed to execute xgauntlet verify --help");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("--task-id") || stdout.contains("-t, --task"),
        "verify --help should recognize --task-id alias"
    );

    // Running with invalid task-id should not fail on unrecognized option
    let run_output = Command::new(bin)
        .args(["verify", "--task-id", "036", "--help"])
        .output()
        .expect("failed to run xgauntlet");
    assert!(run_output.status.success());
}

#[test]
fn test_cli_task_id_alias_supported_on_check_spec() {
    let bin = env!("CARGO_BIN_EXE_xgauntlet");
    let run_output = Command::new(bin)
        .args(["check-spec", "--task-id", "036", "--help"])
        .output()
        .expect("failed to run xgauntlet");
    assert!(run_output.status.success());
}

#[test]
fn test_cli_task_id_alias_supported_on_handoff() {
    let bin = env!("CARGO_BIN_EXE_xgauntlet");
    let run_output = Command::new(bin)
        .args(["handoff", "--task-id", "036", "--help"])
        .output()
        .expect("failed to run xgauntlet");
    assert!(run_output.status.success());
}

#[test]
fn test_cli_task_new_bugfix_one_liner_with_scope_and_test() {
    let bin = env!("CARGO_BIN_EXE_xgauntlet");
    let count = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let ws = std::env::temp_dir().join(format!(
        "xgauntlet_cli_task_new_bug_{}_{count}_{nanos}",
        std::process::id(),
    ));
    std::fs::create_dir_all(ws.join("specs")).unwrap();
    std::fs::write(
        ws.join("CONTEXT.md"),
        "**Task**:\nAn executable unit of work.\n_Avoid_: Ticket.\n",
    )
    .unwrap();
    std::fs::write(
        ws.join("gauntlet.toml"),
        "stack = \"rust\"\n[architecture]\nfeature_root = \"src/features\"\nisolate_features = true\n",
    )
    .unwrap();

    // 1. Repeated --scope and --test flags
    let out1 = Command::new(bin)
        .args([
            "task",
            "new",
            "fix-parser-panic",
            "--intent",
            "bug",
            "--scope",
            "src/features/parser.rs",
            "--scope",
            "src/features/lexer.rs",
            "--test",
            "tests/parser_test.rs",
            "--workspace",
            ws.to_str().unwrap(),
        ])
        .output()
        .expect("failed to execute xgauntlet task new");
    assert!(
        out1.status.success(),
        "task new with --scope/--test must succeed: stderr={}",
        String::from_utf8_lossy(&out1.stderr)
    );

    let task1_path = ws.join("tasks/001-fix-parser-panic.md");
    assert!(task1_path.is_file(), "Expected scaffolded task 001");
    let content1 = std::fs::read_to_string(&task1_path).unwrap();
    assert!(content1.contains("status: active"));
    assert!(content1.contains("**Status**: `ACTIVE`"));
    assert!(content1.contains("intent: bug"));
    assert!(content1.contains("\"src/features/parser.rs\""));
    assert!(content1.contains("\"src/features/lexer.rs\""));
    assert!(!content1.contains("\"src/**\""));
    assert!(content1.contains("\"tests/parser_test.rs\""));
    assert!(!content1.contains("\"tests/**\""));

    // Immediately passes check-spec
    let check1 = Command::new(bin)
        .args([
            "check-spec",
            "-t",
            "001",
            "--workspace",
            ws.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run check-spec");
    assert!(
        check1.status.success(),
        "One-liner bugfix task must immediately pass check-spec: stdout={} stderr={}",
        String::from_utf8_lossy(&check1.stdout),
        String::from_utf8_lossy(&check1.stderr)
    );

    // 2. Comma-separated --scope and --test flags
    let out2 = Command::new(bin)
        .args([
            "task",
            "new",
            "fix-comma-flags",
            "--intent",
            "bug",
            "--scope",
            "src/features/a.rs,src/features/b.rs",
            "--test",
            "tests/a_test.rs,tests/b_test.rs",
            "--workspace",
            ws.to_str().unwrap(),
        ])
        .output()
        .expect("failed to execute xgauntlet task new with comma args");
    assert!(
        out2.status.success(),
        "task new with comma-separated --scope/--test must succeed: stderr={}",
        String::from_utf8_lossy(&out2.stderr)
    );
    let content2 = std::fs::read_to_string(ws.join("tasks/002-fix-comma-flags.md")).unwrap();
    assert!(content2.contains("\"src/features/a.rs\""));
    assert!(content2.contains("\"src/features/b.rs\""));
    assert!(content2.contains("\"tests/a_test.rs\""));
    assert!(content2.contains("\"tests/b_test.rs\""));

    let _ = std::fs::remove_dir_all(&ws);
}
