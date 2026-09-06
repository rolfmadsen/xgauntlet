//! Host platform and system environment diagnostic checks.

use crate::features::doctor::models::{DoctorCategory, DoctorCheckItem, DoctorCheckStatus};
use std::fs;
use std::path::Path;
use std::time::Instant;

/// Inspects host OS, architecture, platform metadata, and temp directory permissions.
pub fn check_host(_workspace: &Path) -> Vec<DoctorCheckItem> {
    let mut checks = Vec::new();

    // 1. Host OS and Architecture
    let start = Instant::now();
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    let family = std::env::consts::FAMILY;
    let duration = start.elapsed().as_millis() as u64;

    checks.push(DoctorCheckItem {
        name: "host_os".to_string(),
        category: DoctorCategory::Host,
        status: DoctorCheckStatus::Pass,
        message: format!("{os} ({arch}, {family})"),
        detail: Some(format!(
            "Target OS: {os}, Architecture: {arch}, Family: {family}"
        )),
        remediation: None,
        duration_ms: duration,
    });

    // 2. Core Engine Info
    let start = Instant::now();
    let version = crate::VERSION;
    let duration = start.elapsed().as_millis() as u64;

    checks.push(DoctorCheckItem {
        name: "core_engine_version".to_string(),
        category: DoctorCategory::Host,
        status: DoctorCheckStatus::Pass,
        message: format!("xgauntlet-core v{version}"),
        detail: Some(format!("Compiled version: {version}")),
        remediation: None,
        duration_ms: duration,
    });

    // 3. System Temporary Directory Access
    let start = Instant::now();
    let temp_dir = std::env::temp_dir();
    let (status, msg, detail, remediation) = if temp_dir.is_dir() {
        let probe_file = temp_dir.join(format!(
            "xgauntlet_doctor_probe_{}_{}.tmp",
            std::process::id(),
            start.elapsed().as_nanos()
        ));
        match fs::write(&probe_file, b"probe") {
            Ok(()) => {
                let _ = fs::remove_file(&probe_file);
                (
                    DoctorCheckStatus::Pass,
                    format!("Writable: {}", temp_dir.display()),
                    Some(format!("Temp path: {}", temp_dir.display())),
                    None,
                )
            }
            Err(e) => (
                DoctorCheckStatus::Fail,
                format!("Temporary directory is not writable: {e}"),
                Some(format!("Temp path: {}", temp_dir.display())),
                Some("Ensure write permissions for the system temp directory.".to_string()),
            ),
        }
    } else {
        (
            DoctorCheckStatus::Fail,
            format!(
                "System temp directory does not exist: {}",
                temp_dir.display()
            ),
            None,
            Some("Set a valid TMPDIR or TEMP environment variable.".to_string()),
        )
    };
    let duration = start.elapsed().as_millis() as u64;

    checks.push(DoctorCheckItem {
        name: "temp_dir".to_string(),
        category: DoctorCategory::Host,
        status,
        message: msg,
        detail,
        remediation,
        duration_ms: duration,
    });

    checks
}
