//! Core coordination engine for the xGauntlet Fast Environment, Git, and Toolchain Diagnostics Engine.

use crate::features::doctor::checks::{
    check_engine, check_git, check_governance, check_host, check_toolchains,
};
use crate::features::doctor::models::{
    DoctorCategory, DoctorCheckStatus, DoctorError, DoctorOptions, DoctorReport, DoctorVerdict,
};
use crate::features::evidence::current_iso_utc;
use std::time::Instant;

/// Executes all or categorized diagnostic checks on a target workspace.
///
/// # Arguments
/// * `options` - Runtime options controlling target workspace, strict mode, and category filtering.
///
/// # Returns
/// A consolidated `DoctorReport` containing individual check items, counts, duration, and overall verdict.
pub fn run_doctor(options: &DoctorOptions) -> Result<DoctorReport, DoctorError> {
    let overall_start = Instant::now();
    let workspace = match options.workspace.canonicalize() {
        Ok(canon) => {
            #[cfg(windows)]
            {
                let s = canon.to_string_lossy();
                if let Some(stripped) = s.strip_prefix(r"\\?\UNC\") {
                    std::path::PathBuf::from(format!(r"\\{}", stripped))
                } else if let Some(stripped) = s.strip_prefix(r"\\?\") {
                    std::path::PathBuf::from(stripped)
                } else {
                    canon
                }
            }
            #[cfg(not(windows))]
            canon
        }
        Err(_) => options.workspace.clone(),
    };

    let mut checks = Vec::new();

    // 1. Host Category
    if options.category.is_none() || options.category == Some(DoctorCategory::Host) {
        checks.extend(check_host(&workspace));
    }

    // 2. Git Category
    if options.category.is_none() || options.category == Some(DoctorCategory::Git) {
        checks.extend(check_git(&workspace));
    }

    // 3. Governance Category
    if options.category.is_none() || options.category == Some(DoctorCategory::Governance) {
        checks.extend(check_governance(&workspace));
    }

    // 4. Toolchains Category
    if options.category.is_none() || options.category == Some(DoctorCategory::Toolchains) {
        checks.extend(check_toolchains(&workspace));
    }

    // 5. Engine Category
    if options.category.is_none() || options.category == Some(DoctorCategory::Engine) {
        checks.extend(check_engine(&workspace));
    }

    // Filter checks if a specific category was requested
    if let Some(target_cat) = options.category {
        checks.retain(|c| c.category == target_cat);
    }

    let passed_count = checks
        .iter()
        .filter(|c| c.status == DoctorCheckStatus::Pass)
        .count();
    let warn_count = checks
        .iter()
        .filter(|c| c.status == DoctorCheckStatus::Warn)
        .count();
    let failed_count = checks
        .iter()
        .filter(|c| c.status == DoctorCheckStatus::Fail)
        .count();
    let info_count = checks
        .iter()
        .filter(|c| c.status == DoctorCheckStatus::Info)
        .count();

    let duration_total_ms = overall_start.elapsed().as_millis() as u64;

    let verdict = if failed_count > 0 {
        DoctorVerdict::Critical
    } else if warn_count > 0 {
        DoctorVerdict::Degraded
    } else {
        DoctorVerdict::Healthy
    };

    let target_platform = format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH);

    Ok(DoctorReport {
        workspace,
        timestamp: current_iso_utc(),
        engine_version: crate::VERSION.to_string(),
        target_platform,
        checks,
        passed_count,
        warn_count,
        failed_count,
        info_count,
        duration_total_ms,
        verdict,
    })
}
