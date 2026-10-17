use std::fs;
use std::path::PathBuf;
use std::process::Command;

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

fn create_mock_bundle(artifact_name: &str, artifact_digest: &str, repo_id: &str) -> String {
    let statement = serde_json::json!({
        "_type": "https://in-toto.io/Statement/v1",
        "subject": [
            {
                "name": artifact_name,
                "digest": {
                    "sha256": artifact_digest
                }
            }
        ],
        "predicateType": "https://slsa.dev/provenance/v1",
        "predicate": {
            "buildDefinition": {
                "externalParameters": {
                    "workflow": {
                        "repository": "https://github.com/rolfmadsen/xgauntlet",
                        "path": ".github/workflows/release.yml"
                    }
                },
                "internalParameters": {
                    "github": {
                        "repository_id": repo_id
                    }
                }
            }
        }
    });

    // Base64 encode statement payload
    let raw = statement.to_string();
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut encoded = String::new();
    let bytes = raw.as_bytes();
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        encoded.push(ALPHABET[((triple >> 18) & 0x3F) as usize] as char);
        encoded.push(ALPHABET[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            encoded.push(ALPHABET[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            encoded.push('=');
        }
        if chunk.len() > 2 {
            encoded.push(ALPHABET[(triple & 0x3F) as usize] as char);
        } else {
            encoded.push('=');
        }
    }

    let bundle = serde_json::json!({
        "mediaType": "application/vnd.dev.sigstore.bundle.v0.3+json",
        "dsseEnvelope": {
            "payloadType": "application/vnd.in-toto+json",
            "payload": encoded
        }
    });

    bundle.to_string()
}

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

#[test]
fn test_cli_check_attestation_succeeds_with_valid_bundle_and_json_output() {
    let temp = TempDir::new("attest_valid");
    let artifact_path = temp.path.join("xgauntlet-linux-x64.tar.gz");
    let dummy_archive = b"dummy tarball archive content for test";
    fs::write(&artifact_path, dummy_archive).unwrap();
    let digest = xgauntlet_core::compute_file_raw_hash(dummy_archive);

    let bundle_content = create_mock_bundle("xgauntlet-linux-x64.tar.gz", &digest, "1359184794");
    let bundle_path = temp.path.join("attestation.bundle");
    fs::write(&bundle_path, bundle_content).unwrap();

    let bin = env!("CARGO_BIN_EXE_xgauntlet");
    let output = Command::new(bin)
        .args([
            "check-attestation",
            "--bundle",
            bundle_path.to_str().unwrap(),
            "--artifact",
            artifact_path.to_str().unwrap(),
            "--json",
        ])
        .output()
        .expect("failed to execute check-attestation");

    assert!(
        output.status.success(),
        "check-attestation must exit 0 on valid bundle and matching digest"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("output must be valid JSON");
    assert_eq!(json["verdict"], "PASSED");
    assert_eq!(json["attestation_status"], "VALID");
    assert_eq!(json["trust_decision"], "ACCEPTED");
    assert_eq!(json["artifact_digest"], digest);
}

#[test]
fn test_cli_check_attestation_fails_closed_on_digest_mismatch() {
    let temp = TempDir::new("attest_mismatch");
    let artifact_path = temp.path.join("xgauntlet-linux-x64.tar.gz");
    let dummy_archive = b"tampered tarball archive content";
    fs::write(&artifact_path, dummy_archive).unwrap();

    let bundle_content = create_mock_bundle(
        "xgauntlet-linux-x64.tar.gz",
        "0000000000000000000000000000000000000000000000000000000000000000",
        "1359184794",
    );
    let bundle_path = temp.path.join("attestation.bundle");
    fs::write(&bundle_path, bundle_content).unwrap();

    let bin = env!("CARGO_BIN_EXE_xgauntlet");
    let output = Command::new(bin)
        .args([
            "check-attestation",
            "--bundle",
            bundle_path.to_str().unwrap(),
            "--artifact",
            artifact_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to execute check-attestation");

    assert!(
        !output.status.success(),
        "check-attestation must exit with code 1 on digest mismatch"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("DIGEST_MISMATCH"),
        "Output should report DIGEST_MISMATCH"
    );
}

#[test]
fn test_cli_check_attestation_fails_closed_on_repo_id_mismatch() {
    let temp = TempDir::new("attest_wrong_repo");
    let artifact_path = temp.path.join("xgauntlet-linux-x64.tar.gz");
    let dummy_archive = b"sample archive";
    fs::write(&artifact_path, dummy_archive).unwrap();
    let digest = xgauntlet_core::compute_file_raw_hash(dummy_archive);

    // Bundle issued with attacker repo ID
    let bundle_content = create_mock_bundle("xgauntlet-linux-x64.tar.gz", &digest, "999999999");
    let bundle_path = temp.path.join("attestation.bundle");
    fs::write(&bundle_path, bundle_content).unwrap();

    let bin = env!("CARGO_BIN_EXE_xgauntlet");
    let output = Command::new(bin)
        .args([
            "check-attestation",
            "--bundle",
            bundle_path.to_str().unwrap(),
            "--artifact",
            artifact_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to execute check-attestation");

    assert!(
        !output.status.success(),
        "check-attestation must exit with code 1 on repo ID mismatch"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("REPOSITORY_ID_MISMATCH"),
        "Output should report REPOSITORY_ID_MISMATCH"
    );
}

#[test]
fn test_cli_check_attestation_resolves_attestation_json_fallback_when_bundle_missing() {
    let temp = TempDir::new("attest_json_fallback");
    let artifact_path = temp.path.join("xgauntlet-linux-x64.tar.gz");
    let dummy_archive = b"dummy tarball archive content for fallback test";
    fs::write(&artifact_path, dummy_archive).unwrap();
    let digest = xgauntlet_core::compute_file_raw_hash(dummy_archive);

    let bundle_content = create_mock_bundle("xgauntlet-linux-x64.tar.gz", &digest, "1359184794");
    // Write ONLY attestation.json (no attestation.bundle)
    let json_bundle_path = temp.path.join("attestation.json");
    fs::write(&json_bundle_path, bundle_content).unwrap();

    let missing_bundle_path = temp.path.join("attestation.bundle");

    let bin = env!("CARGO_BIN_EXE_xgauntlet");
    let output = Command::new(bin)
        .args([
            "check-attestation",
            "--bundle",
            missing_bundle_path.to_str().unwrap(),
            "--artifact",
            artifact_path.to_str().unwrap(),
            "--json",
        ])
        .output()
        .expect("failed to execute check-attestation");

    assert!(
        output.status.success(),
        "check-attestation must transparently fallback to attestation.json when attestation.bundle is missing"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("output must be valid JSON");
    assert_eq!(json["verdict"], "PASSED");
    assert_eq!(json["attestation_status"], "VALID");
}
