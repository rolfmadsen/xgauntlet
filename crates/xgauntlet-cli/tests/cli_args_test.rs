use std::process::Command;

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
