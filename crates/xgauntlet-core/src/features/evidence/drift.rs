//! Workspace state verification and drift detection engine.

use super::manifest::CanonicalWorkspaceManifest;
use super::models::VerificationReport;
use crate::features::diagnostics::{DiagnosticFinding, FindingType};
use thiserror::Error;

/// Violation when the current workspace state has drifted from a recorded verification report.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub struct DriftViolation {
    pub source_drift: bool,
    pub policy_drift: bool,
    pub config_drift: bool,
    pub task_drift: bool,
    pub expected_source_digest: String,
    pub current_source_digest: String,
    pub expected_policy_digest: String,
    pub current_policy_digest: String,
    pub expected_config_digest: String,
    pub current_config_digest: String,
    pub expected_task_digest: String,
    pub current_task_digest: String,
}

impl std::fmt::Display for DriftViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Workspace state drift detected:")?;
        if self.source_drift {
            write!(
                f,
                " [source manifest drifted: expected {}, got {}]",
                truncate_digest(&self.expected_source_digest),
                truncate_digest(&self.current_source_digest),
            )?;
        }
        if self.policy_drift {
            write!(
                f,
                " [policy drifted: expected {}, got {}]",
                truncate_digest(&self.expected_policy_digest),
                truncate_digest(&self.current_policy_digest),
            )?;
        }
        if self.config_drift {
            write!(
                f,
                " [config drifted: expected {}, got {}]",
                truncate_digest(&self.expected_config_digest),
                truncate_digest(&self.current_config_digest),
            )?;
        }
        if self.task_drift {
            write!(
                f,
                " [task contract drifted: expected {}, got {}]",
                truncate_digest(&self.expected_task_digest),
                truncate_digest(&self.current_task_digest),
            )?;
        }
        Ok(())
    }
}

impl DriftViolation {
    pub fn to_diagnostic_finding(&self) -> DiagnosticFinding {
        DiagnosticFinding::new(
            FindingType::DriftDetected,
            "xgauntlet-check-evidence",
            "verification-report.json",
            format!("{self}"),
            "Re-run 'xgauntlet verify' to validate and seal fresh evidence for current workspace state.",
        )
    }
}

fn truncate_digest(d: &str) -> &str {
    if d.len() > 16 {
        &d[..16]
    } else {
        d
    }
}

fn digest_matches(expected: &str, current: &str) -> bool {
    if expected.is_empty() || current.is_empty() {
        return false;
    }
    expected == current
}

/// Verifies that current workspace manifest matches the recorded verification report.
pub fn verify_workspace_state_match(
    report: &VerificationReport,
    current: &CanonicalWorkspaceManifest,
) -> Result<(), Box<DriftViolation>> {
    let rep_source = &report.workspace_state.source_manifest_digest_post;
    let cur_source = &current.source_manifest_digest;

    let source_drift = !digest_matches(rep_source, cur_source);

    let rep_policy = &report.workspace_state.policy_digest;
    let cur_policy = &current.policy_digest;
    let policy_drift = if !rep_policy.is_empty() && !cur_policy.is_empty() {
        !digest_matches(rep_policy, cur_policy)
    } else {
        false
    };

    let rep_config = &report.workspace_state.config_digest;
    let cur_config = &current.config_digest;
    let config_drift = if !rep_config.is_empty() && !cur_config.is_empty() {
        !digest_matches(rep_config, cur_config)
    } else {
        false
    };

    let rep_task = &report.workspace_state.task_digest;
    let cur_task = &current.task_digest;
    let task_drift = if !rep_task.is_empty() && !cur_task.is_empty() {
        !digest_matches(rep_task, cur_task)
    } else {
        false
    };

    if source_drift || policy_drift || config_drift || task_drift {
        return Err(Box::new(DriftViolation {
            source_drift,
            policy_drift,
            config_drift,
            task_drift,
            expected_source_digest: rep_source.clone(),
            current_source_digest: cur_source.clone(),
            expected_policy_digest: rep_policy.clone(),
            current_policy_digest: cur_policy.clone(),
            expected_config_digest: rep_config.clone(),
            current_config_digest: cur_config.clone(),
            expected_task_digest: rep_task.clone(),
            current_task_digest: cur_task.clone(),
        }));
    }

    Ok(())
}
