//! Diagnostics and Doctor integration tests for composite TypeScript validation (Task 029 AC3).

use std::fs;
use std::path::{Path, PathBuf};

use xgauntlet_core::features::doctor::{
    run_doctor, DoctorCategory, DoctorCheckStatus, DoctorOptions,
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
        let path = std::env::temp_dir().join(format!("xgauntlet_doc_test_{}", unique));
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

#[test]
fn test_doctor_warns_on_tsc_no_emit_in_composite_typescript_repo() {
    let temp = TempDir::new("doctor_composite_warn");
    let ws = temp.path();

    // 1. Setup composite tsconfig
    let composite_tsconfig = r#"{
      "files": [],
      "references": [
        { "path": "./tsconfig.app.json" },
        { "path": "./tsconfig.node.json" }
      ]
    }"#;
    fs::write(ws.join("tsconfig.json"), composite_tsconfig).unwrap();

    // 2. Setup gauntlet.toml that mistakenly uses `tsc --noEmit`
    let gauntlet_toml = r#"
stack = "node"
save_evidence = true

[[layers]]
name = "types"
command = ["npx", "tsc", "--noEmit"]
"#;
    fs::write(ws.join("gauntlet.toml"), gauntlet_toml).unwrap();

    let options = DoctorOptions {
        workspace: ws.to_path_buf(),
        verbose: true,
        strict: false,
        category: Some(DoctorCategory::Governance),
    };

    let report = run_doctor(&options).expect("Doctor check should run successfully");

    // Look for composite tsconfig warning
    let composite_warn = report.checks.iter().find(|c| {
        (c.name == "governance_composite_typescript"
            || c.name == "governance_gauntlet_toml"
            || c.message.contains("composite")
            || c.detail.as_deref().unwrap_or("").contains("composite"))
            && c.status == DoctorCheckStatus::Warn
    });

    assert!(
        composite_warn.is_some(),
        "Expected doctor to warn about 'tsc --noEmit' in a composite TypeScript project. Checks: {:?}",
        report.checks
    );

    let warn_item = composite_warn.unwrap();
    assert!(
        warn_item.message.to_ascii_lowercase().contains("composite")
            || warn_item
                .detail
                .as_deref()
                .unwrap_or("")
                .to_ascii_lowercase()
                .contains("composite"),
        "Warning message or detail must explain composite TypeScript bypass risk"
    );
}

#[test]
fn test_doctor_passes_when_composite_typescript_uses_tsc_b() {
    let temp = TempDir::new("doctor_composite_pass");
    let ws = temp.path();

    // 1. Setup composite tsconfig
    let composite_tsconfig = r#"{
      "files": [],
      "references": [
        { "path": "./tsconfig.app.json" }
      ]
    }"#;
    fs::write(ws.join("tsconfig.json"), composite_tsconfig).unwrap();

    // 2. Setup gauntlet.toml correctly configured with `tsc -b`
    let gauntlet_toml = r#"
stack = "node"
save_evidence = true

[[layers]]
name = "types"
command = ["npx", "tsc", "-b"]
"#;
    fs::write(ws.join("gauntlet.toml"), gauntlet_toml).unwrap();

    let options = DoctorOptions {
        workspace: ws.to_path_buf(),
        verbose: true,
        strict: false,
        category: Some(DoctorCategory::Governance),
    };

    let report = run_doctor(&options).expect("Doctor check should run successfully");

    // Must NOT have any warning about composite tsconfig
    let composite_warn = report.checks.iter().find(|c| {
        (c.name == "governance_composite_typescript"
            || c.detail.as_deref().unwrap_or("").contains("COMPOSITE"))
            && c.status == DoctorCheckStatus::Warn
    });

    assert!(
        composite_warn.is_none(),
        "Doctor must NOT warn when 'tsc -b' is correctly configured for composite project references"
    );
}
