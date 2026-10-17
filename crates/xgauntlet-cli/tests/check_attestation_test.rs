use std::process::Command;

#[test]
fn test_cli_check_attestation_help() {
    let bin = env!("CARGO_BIN_EXE_xgauntlet");
    let output = Command::new(bin)
        .args(["check-attestation", "--help"])
        .output()
        .expect("failed to execute xgauntlet check-attestation --help");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("check-attestation") || stdout.contains("attestation"),
        "check-attestation --help should describe attestation verification"
    );
    assert!(
        stdout.contains("--bundle") || stdout.contains("-b"),
        "check-attestation should have --bundle flag"
    );
    assert!(
        stdout.contains("--artifact") || stdout.contains("-a"),
        "check-attestation should have --artifact flag"
    );
    assert!(
        stdout.contains("--expected-repo-id"),
        "check-attestation should have --expected-repo-id flag"
    );
}

#[test]
fn test_cli_check_attestation_fails_closed_on_missing_bundle() {
    let bin = env!("CARGO_BIN_EXE_xgauntlet");
    let output = Command::new(bin)
        .args([
            "check-attestation",
            "--bundle",
            "non_existent_bundle.json",
            "--artifact",
            "Cargo.toml",
        ])
        .output()
        .expect("failed to run xgauntlet check-attestation");
    assert!(
        !output.status.success(),
        "check-attestation must exit with non-zero on non-existent bundle"
    );
}
