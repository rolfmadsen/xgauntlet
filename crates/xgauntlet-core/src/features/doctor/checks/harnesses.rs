//! Diagnostic checks for installed AI agent harnesses and xGauntlet plugin status.

use crate::features::doctor::models::{DoctorCategory, DoctorCheckItem, DoctorCheckStatus};
use crate::features::plugin::discover_installed_harnesses;
use std::path::Path;

/// Evaluates detected harnesses and global xGauntlet plugin installations.
pub fn check_harnesses(_workspace: &Path) -> Vec<DoctorCheckItem> {
    let mut checks = Vec::new();
    let harnesses = discover_installed_harnesses();

    for h in harnesses {
        let check_name = format!("harness_{}", h.name);
        let (status, message, detail, remediation) = if h.detected && h.plugin_installed {
            (
                DoctorCheckStatus::Pass,
                format!(
                    "Harness '{}' detected with xGauntlet plugin installed",
                    h.name
                ),
                Some(format!(
                    "Config: {}, Plugin: {}",
                    h.config_dir.display(),
                    h.plugin_dir.display()
                )),
                None,
            )
        } else if h.detected {
            (
                DoctorCheckStatus::Info,
                format!(
                    "Harness '{}' detected but xGauntlet plugin is not installed",
                    h.name
                ),
                Some(format!("Target plugin path: {}", h.plugin_dir.display())),
                Some(format!(
                    "Run 'xgauntlet plugin install --harness {}' to install plugin",
                    h.name
                )),
            )
        } else {
            (
                DoctorCheckStatus::Info,
                format!("Harness '{}' not detected on host", h.name),
                Some(format!("Expected config path: {}", h.config_dir.display())),
                None,
            )
        };

        checks.push(DoctorCheckItem {
            name: check_name,
            category: DoctorCategory::Harnesses,
            status,
            message,
            detail,
            remediation,
            duration_ms: 1,
        });
    }

    checks
}
