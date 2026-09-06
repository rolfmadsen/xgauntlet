//! Unsigned verification report engine and markdown generator (Schema v2).

use std::fs;
use std::path::Path;
use thiserror::Error;

use super::models::VerificationReport;

#[derive(Debug, Error)]
pub enum ReportError {
    #[error("JSON serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("IO error while handling verification report: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Verification report file not found at '{0}'")]
    FileNotFound(String),
}

/// Serializes a verification report into Schema v2 formatted JSON.
pub fn generate_report_json(report: &VerificationReport) -> Result<String, ReportError> {
    serde_json::to_string_pretty(report).map_err(ReportError::SerializationError)
}

/// Deserializes a verification report from a JSON string.
pub fn load_report_json(json_str: &str) -> Result<VerificationReport, ReportError> {
    serde_json::from_str(json_str).map_err(ReportError::SerializationError)
}

/// Saves verification-report.json and evidence.md atomically to the workspace root.
pub fn save_verification_report(
    workspace: &Path,
    report: &VerificationReport,
) -> Result<(), ReportError> {
    let json_content = generate_report_json(report)?;
    let md_content = generate_report_markdown(report, None);

    fs::write(workspace.join("verification-report.json"), json_content)?;
    fs::write(workspace.join("evidence.md"), md_content)?;

    Ok(())
}

/// Loads verification-report.json from the given workspace root.
pub fn load_verification_report(workspace: &Path) -> Result<VerificationReport, ReportError> {
    let path = workspace.join("verification-report.json");
    if !path.is_file() {
        return Err(ReportError::FileNotFound(path.display().to_string()));
    }
    let data = fs::read_to_string(&path)?;
    load_report_json(&data)
}

/// Classifies an evidence JSON payload string according to the Two-Tier Trust Model.
pub fn classify_evidence_payload(json_str: &str) -> String {
    let val: serde_json::Value = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(_) => return "MALFORMED_PAYLOAD".to_string(),
    };

    if val.get("schema_version").is_some() || val.get("workspace_state").is_some() {
        let origin = val
            .get("execution_origin")
            .and_then(|v| v.as_str())
            .unwrap_or("LOCAL");
        if origin == "LOCAL" {
            "LOCAL_UNATTESTED".to_string()
        } else {
            "CI_UNPRIVILEGED".to_string()
        }
    } else if val.get("signature").is_some() || val.get("source_tree_hash").is_some() {
        "LEGACY_UNATTESTED".to_string()
    } else {
        "UNKNOWN_PAYLOAD".to_string()
    }
}

/// Formats a verification report into human-readable evidence.md markdown.
pub fn generate_report_markdown(report: &VerificationReport, title: Option<&str>) -> String {
    let title_str = title.unwrap_or("Verification Report");
    let vcs_head = report
        .workspace_state
        .vcs
        .as_ref()
        .map(|v| v.head.as_str())
        .unwrap_or("(no git)");
    let vcs_commit = report
        .workspace_state
        .vcs
        .as_ref()
        .map(|v| v.commit.as_str())
        .unwrap_or("(no git)");

    let mut lines = Vec::new();
    lines.push(format!("# {title_str}"));
    lines.push(String::new());

    let task_id = if !report.task_contract.task_id.is_empty() {
        &report.task_contract.task_id
    } else {
        "default-run"
    };
    lines.push(format!("**Task ID**: `{task_id}`  "));

    if !report.task_contract.task_title.is_empty() {
        lines.push(format!(
            "**Task Title**: {}  ",
            report.task_contract.task_title
        ));
    }

    let manifest_digest = if !report
        .workspace_state
        .source_manifest_digest_post
        .is_empty()
    {
        &report.workspace_state.source_manifest_digest_post
    } else {
        "(none)"
    };

    let timestamp = if !report.execution_metadata.finished_at.is_empty() {
        &report.execution_metadata.finished_at
    } else {
        "N/A"
    };

    lines.push(format!("**Verdict**: `{}`  ", report.verdict));
    lines.push(format!(
        "**Execution Origin**: `{}`  ",
        report.execution_origin
    ));
    lines.push(format!("**Source Manifest Digest**: `{manifest_digest}`  "));
    lines.push(format!("**Timestamp**: `{timestamp}`  "));
    lines.push(format!("**Head**: `{vcs_head}`  "));
    lines.push(format!("**Commit**: `{vcs_commit}`  "));

    if !report.task_contract.acceptance_criteria.is_empty() {
        lines.push(String::new());
        lines.push("## Acceptance Criteria".to_string());
        lines.push(String::new());
        for crit in &report.task_contract.acceptance_criteria {
            let is_unresolved = report
                .task_contract
                .unresolved_criteria
                .iter()
                .any(|u| u == crit);
            let check_icon = if is_unresolved { "[ ]" } else { "[x]" };
            lines.push(format!("- {check_icon} {crit}"));
        }
    }

    if !report.checks.is_empty() {
        lines.push(String::new());
        lines.push("---".to_string());
        lines.push(String::new());
        lines.push("## Verification Checks".to_string());
        lines.push(String::new());
        lines.push("| Check Name | Status | Exit Code | Duration (s) |".to_string());
        lines.push("|---|---|---|---|".to_string());
        for c in &report.checks {
            lines.push(format!(
                "| `{}` | `{}` | `{}` | `{:.3}s` |",
                c.name, c.status, c.exit_code, c.duration_seconds
            ));
        }
    }

    lines.push(String::new());
    lines.push("---".to_string());
    lines.push(String::new());

    lines.join("\n")
}
