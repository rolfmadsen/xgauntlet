//! Verification and validation subcommand runners for xGauntlet CLI.

use std::path::Path;

#[derive(Debug, serde::Serialize)]
pub struct CheckEvidenceOutput {
    pub passed: bool,
    pub task_id: String,
    pub report_source_manifest: String,
    pub current_source_manifest: String,
    pub findings: Vec<xgauntlet_core::DiagnosticFinding>,
}

pub fn run_check_spec(
    workspace: &Path,
    task_filter: Option<&str>,
    all: bool,
) -> anyhow::Result<Vec<xgauntlet_core::SpecReadinessReport>> {
    let tasks_dir = workspace.join("tasks");

    if all {
        return Ok(xgauntlet_core::check_all_tasks(workspace));
    }

    if let Some(target) = task_filter {
        let mut target_path = None;
        if tasks_dir.is_dir() {
            for entry in std::fs::read_dir(&tasks_dir)?.flatten() {
                let p = entry.path();
                if p.extension().is_some_and(|ext| ext == "md") {
                    let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                    let name = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
                    if stem == target
                        || name == target
                        || stem.starts_with(&format!("{target}-"))
                        || name.starts_with(&format!("{target}-"))
                        || stem.contains(target)
                    {
                        target_path = Some(p);
                        break;
                    }
                }
            }
        }

        let p = target_path.unwrap_or_else(|| tasks_dir.join(format!("{target}.md")));
        return Ok(vec![xgauntlet_core::check_task_specification(
            &p, workspace,
        )]);
    }

    if let Some(active_id) = xgauntlet_core::resolve_active_task_id(workspace) {
        let task_path = tasks_dir.join(format!("{active_id}.md"));
        return Ok(vec![xgauntlet_core::check_task_specification(
            &task_path, workspace,
        )]);
    }

    // No active task found: validate CONTEXT.md and return empty report with glossary diags
    let diags = xgauntlet_core::validate_context_glossary(workspace);
    let mut rep = xgauntlet_core::SpecReadinessReport::new("no-active-task");
    rep.is_valid = diags.is_empty();
    rep.diagnostics = diags;
    if workspace.join("CONTEXT.md").is_file() {
        rep.inspected_files.push("CONTEXT.md".to_string());
    }
    Ok(vec![rep])
}

pub fn run_check_adr(
    workspace: &Path,
    file_filter: Option<&Path>,
    _all: bool,
) -> anyhow::Result<Vec<xgauntlet_core::AdrReadinessReport>> {
    let adr_dir = workspace.join("docs").join("adr");

    if let Some(target) = file_filter {
        let p = if target.is_absolute() {
            target.to_path_buf()
        } else if workspace.join(target).exists() {
            workspace.join(target)
        } else {
            adr_dir.join(target)
        };
        return Ok(vec![xgauntlet_core::check_adr_frontmatter(&p)]);
    }

    Ok(xgauntlet_core::check_all_adrs(workspace))
}

pub fn run_check_evidence(workspace: &Path) -> anyhow::Result<CheckEvidenceOutput> {
    let report_result = xgauntlet_core::load_verification_report(workspace);
    let report = match report_result {
        Ok(r) => r,
        Err(e) => {
            return Ok(CheckEvidenceOutput {
                passed: false,
                task_id: "unknown".to_string(),
                report_source_manifest: String::new(),
                current_source_manifest: String::new(),
                findings: vec![xgauntlet_core::DiagnosticFinding::new(
                    xgauntlet_core::FindingType::GeneralError,
                    "xgauntlet-check-evidence",
                    "verification-report.json",
                    format!("Failed to load verification report: {e}"),
                    "Run 'xgauntlet verify' first to seal evidence for the active task.",
                )],
            });
        }
    };

    let current_manifest = xgauntlet_core::compute_workspace_manifest(workspace, None)?;
    let match_result = xgauntlet_core::verify_workspace_state_match(&report, &current_manifest);

    let (mut passed, mut findings) = match match_result {
        Ok(()) => (true, Vec::new()),
        Err(drift) => (false, vec![drift.to_diagnostic_finding()]),
    };

    if let Some(ref lineage) = report.lineage {
        if let Err(e) = lineage.verify_integrity() {
            passed = false;
            findings.push(xgauntlet_core::DiagnosticFinding::new(
                xgauntlet_core::FindingType::GeneralError,
                "xgauntlet-check-evidence",
                "verification-report.json",
                format!("Session lineage integrity violation: {e}"),
                "Tamper detected in multi-session lineage trail. Ensure lineage entries and SHA-256 hash chains have not been modified.",
            ));
        }
    }

    Ok(CheckEvidenceOutput {
        passed,
        task_id: report.task_contract.task_id,
        report_source_manifest: report.workspace_state.source_manifest_digest_post,
        current_source_manifest: current_manifest.source_manifest_digest,
        findings,
    })
}
