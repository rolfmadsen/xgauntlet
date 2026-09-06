//! Integration tests for xGauntlet Fast Environment, Git, and Toolchain Diagnostics Engine (Task 009).

use std::fs;
use std::path::{Path, PathBuf};
use xgauntlet_core::features::doctor::{
    run_doctor, DoctorCategory, DoctorCheckItem, DoctorCheckStatus, DoctorOptions, DoctorReport,
    DoctorVerdict,
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
        let path = std::env::temp_dir().join(format!("xgauntlet_doctor_test_{}", unique));
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
fn test_doctor_models_and_json_roundtrip() {
    let check = DoctorCheckItem {
        name: "test_check".to_string(),
        category: DoctorCategory::Host,
        status: DoctorCheckStatus::Pass,
        message: "Check succeeded".to_string(),
        detail: Some("Platform detail info".to_string()),
        remediation: None,
        duration_ms: 12,
    };

    let report = DoctorReport {
        workspace: PathBuf::from("/tmp/test-ws"),
        timestamp: "2026-09-06T18:00:00Z".to_string(),
        engine_version: "0.1.0".to_string(),
        target_platform: "linux-x86_64".to_string(),
        checks: vec![check.clone()],
        passed_count: 1,
        warn_count: 0,
        failed_count: 0,
        info_count: 0,
        duration_total_ms: 12,
        verdict: DoctorVerdict::Healthy,
    };

    assert!(report.is_healthy());
    assert!(!report.has_failures());
    assert_eq!(report.checks.len(), 1);
    assert_eq!(check.status.as_str(), "PASS");
    assert_eq!(report.verdict.as_str(), "HEALTHY");

    // JSON roundtrip
    let serialized = serde_json::to_string(&report).expect("Serialization must succeed");
    let deserialized: DoctorReport =
        serde_json::from_str(&serialized).expect("Deserialization must succeed");

    assert_eq!(report, deserialized);
}

#[test]
fn test_run_doctor_on_real_workspace() {
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("Parent dir")
        .parent()
        .expect("Workspace root")
        .to_path_buf();

    let options = DoctorOptions {
        workspace: workspace.clone(),
        verbose: true,
        strict: false,
        category: None,
    };

    let report = run_doctor(&options).expect("Doctor execution must succeed on real workspace");

    assert_eq!(report.workspace, workspace);
    assert!(!report.checks.is_empty(), "Must produce diagnostic checks");
    assert!(report.duration_total_ms > 0 || !report.checks.is_empty());

    // Host category check present
    assert!(
        report
            .checks
            .iter()
            .any(|c| c.category == DoctorCategory::Host),
        "Host check must be present"
    );

    // Git category check present
    assert!(
        report
            .checks
            .iter()
            .any(|c| c.category == DoctorCategory::Git),
        "Git check must be present"
    );

    // Governance category check present
    assert!(
        report
            .checks
            .iter()
            .any(|c| c.category == DoctorCategory::Governance),
        "Governance check must be present"
    );

    // Toolchains category check present
    assert!(
        report
            .checks
            .iter()
            .any(|c| c.category == DoctorCategory::Toolchains),
        "Toolchains check must be present"
    );

    // Engine category check present
    assert!(
        report
            .checks
            .iter()
            .any(|c| c.category == DoctorCategory::Engine),
        "Engine check must be present"
    );

    // Overall verdict on valid repo should be Healthy or at most Degraded
    assert_ne!(
        report.verdict,
        DoctorVerdict::Critical,
        "Real workspace should not have Critical verdict"
    );
}

#[test]
fn test_run_doctor_filter_by_category() {
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("Parent dir")
        .parent()
        .expect("Workspace root")
        .to_path_buf();

    let options = DoctorOptions {
        workspace,
        verbose: false,
        strict: false,
        category: Some(DoctorCategory::Git),
    };

    let report = run_doctor(&options).expect("Doctor execution should succeed");

    assert!(!report.checks.is_empty());
    for check in &report.checks {
        assert_eq!(
            check.category,
            DoctorCategory::Git,
            "Only Git checks should be returned when filtered"
        );
    }
}

#[test]
fn test_host_diagnostics_check() {
    let workspace = PathBuf::from(".");
    let options = DoctorOptions {
        workspace,
        verbose: true,
        strict: false,
        category: Some(DoctorCategory::Host),
    };

    let report = run_doctor(&options).expect("Host checks must succeed");
    assert!(!report.checks.is_empty());

    let os_check = report
        .checks
        .iter()
        .find(|c| c.name == "host_os")
        .expect("host_os check required");
    assert_eq!(os_check.status, DoctorCheckStatus::Pass);

    let temp_check = report
        .checks
        .iter()
        .find(|c| c.name == "temp_dir")
        .expect("temp_dir check required");
    assert_eq!(temp_check.status, DoctorCheckStatus::Pass);
}

#[test]
fn test_git_diagnostics_check() {
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("Parent dir")
        .parent()
        .expect("Workspace root")
        .to_path_buf();

    let options = DoctorOptions {
        workspace,
        verbose: true,
        strict: false,
        category: Some(DoctorCategory::Git),
    };

    let report = run_doctor(&options).expect("Git checks must succeed");
    let git_binary = report
        .checks
        .iter()
        .find(|c| c.name == "git_binary")
        .expect("git_binary check required");
    assert_eq!(git_binary.status, DoctorCheckStatus::Pass);

    let git_repo = report
        .checks
        .iter()
        .find(|c| c.name == "git_repository")
        .expect("git_repository check required");
    assert_eq!(git_repo.status, DoctorCheckStatus::Pass);
}

#[test]
fn test_governance_diagnostics_check() {
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("Parent dir")
        .parent()
        .expect("Workspace root")
        .to_path_buf();

    let options = DoctorOptions {
        workspace,
        verbose: true,
        strict: false,
        category: Some(DoctorCategory::Governance),
    };

    let report = run_doctor(&options).expect("Governance checks must succeed");

    let toml_check = report
        .checks
        .iter()
        .find(|c| c.name == "governance_gauntlet_toml")
        .expect("gauntlet.toml check required");
    assert_eq!(toml_check.status, DoctorCheckStatus::Pass);

    let spec_check = report
        .checks
        .iter()
        .find(|c| c.name == "governance_spec")
        .expect("spec.md check required");
    assert_eq!(spec_check.status, DoctorCheckStatus::Pass);

    let context_check = report
        .checks
        .iter()
        .find(|c| c.name == "governance_context")
        .expect("CONTEXT.md check required");
    assert_eq!(context_check.status, DoctorCheckStatus::Pass);
}

#[test]
fn test_toolchains_diagnostics_check() {
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("Parent dir")
        .parent()
        .expect("Workspace root")
        .to_path_buf();

    let options = DoctorOptions {
        workspace,
        verbose: true,
        strict: false,
        category: Some(DoctorCategory::Toolchains),
    };

    let report = run_doctor(&options).expect("Toolchain checks must succeed");

    let stack_check = report
        .checks
        .iter()
        .find(|c| c.name == "detected_stack")
        .expect("detected_stack check required");
    assert_eq!(stack_check.status, DoctorCheckStatus::Pass);
    assert!(stack_check.message.contains("rust"));

    let cargo_check = report
        .checks
        .iter()
        .find(|c| c.name == "tool_cargo")
        .expect("tool_cargo check required");
    assert_eq!(cargo_check.status, DoctorCheckStatus::Pass);
}

#[test]
fn test_engine_wasm_diagnostics_check() {
    let workspace = PathBuf::from(".");
    let options = DoctorOptions {
        workspace,
        verbose: true,
        strict: false,
        category: Some(DoctorCategory::Engine),
    };

    let report = run_doctor(&options).expect("Engine checks must succeed");

    let wasm_check = report
        .checks
        .iter()
        .find(|c| c.name == "wasm_runtime_host")
        .expect("wasm_runtime_host check required");
    assert_eq!(wasm_check.status, DoctorCheckStatus::Pass);
}

#[test]
fn test_run_doctor_missing_governance_provides_remediations() {
    let temp = TempDir::new("missing_gov");
    let options = DoctorOptions {
        workspace: temp.path().to_path_buf(),
        verbose: false,
        strict: false,
        category: Some(DoctorCategory::Governance),
    };

    let report = run_doctor(&options).expect("Doctor should run on empty directory");

    assert!(report.failed_count > 0 || report.warn_count > 0);

    let failing_or_warning = report
        .checks
        .iter()
        .filter(|c| c.status == DoctorCheckStatus::Fail || c.status == DoctorCheckStatus::Warn)
        .collect::<Vec<_>>();

    assert!(!failing_or_warning.is_empty());

    // Should have remediation recommending xgauntlet init
    let has_init_remediation = failing_or_warning
        .iter()
        .any(|c| c.remediation.as_deref().unwrap_or("").contains("init"));
    assert!(
        has_init_remediation,
        "Missing governance files must suggest 'init' remediation"
    );
}

#[test]
fn test_run_doctor_strict_mode() {
    let report_with_warn = DoctorReport {
        workspace: PathBuf::from("/test"),
        timestamp: "2026-09-06T18:00:00Z".to_string(),
        engine_version: "0.1.0".to_string(),
        target_platform: "linux-x86_64".to_string(),
        checks: vec![DoctorCheckItem {
            name: "warn_check".to_string(),
            category: DoctorCategory::Git,
            status: DoctorCheckStatus::Warn,
            message: "Uncommitted changes".to_string(),
            detail: None,
            remediation: Some("Commit your changes".to_string()),
            duration_ms: 5,
        }],
        passed_count: 0,
        warn_count: 1,
        failed_count: 0,
        info_count: 0,
        duration_total_ms: 5,
        verdict: DoctorVerdict::Degraded,
    };

    assert_eq!(report_with_warn.verdict, DoctorVerdict::Degraded);
    assert!(!report_with_warn.is_strictly_healthy());
    assert!(report_with_warn.is_healthy()); // non-strict: degraded is not critical
}
