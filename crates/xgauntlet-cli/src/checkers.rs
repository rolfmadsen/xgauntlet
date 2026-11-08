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

pub fn render_check_adr_summary(reports: &[xgauntlet_core::AdrReadinessReport]) {
    println!("\n=== xGauntlet ADR OKF v0.2 Gatekeeper ===\n");
    for rep in reports {
        let status_badge = if rep.is_valid {
            "\x1b[32m[PASSED]\x1b[0m"
        } else {
            "\x1b[31m[FAILED]\x1b[0m"
        };
        println!("ADR: {} {}", rep.adr_id, status_badge);
        println!(
            "  - Inspected files:           {}",
            rep.inspected_files.join(", ")
        );

        if !rep.diagnostics.is_empty() {
            println!("  - Diagnostics findings ({}):", rep.diagnostics.len());
            for d in &rep.diagnostics {
                let badge = if d.finding_type == xgauntlet_core::FindingType::Warning {
                    "\x1b[33m[!]\x1b[0m"
                } else {
                    "\x1b[31m[!]\x1b[0m"
                };
                println!(
                    "    {} [{}] {}: {}",
                    badge, d.tool_name, d.file_path, d.message
                );
                if !d.remediation_hint.is_empty() {
                    println!("        Hint: {}", d.remediation_hint);
                }
            }
        }
        println!();
    }
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

pub fn run_check_spec_context(workspace: &Path, json: bool) -> anyhow::Result<bool> {
    let options = xgauntlet_core::GlossaryDriftOptions {
        strict: true,
        check_avoid: true,
    };
    let report = xgauntlet_core::validate_glossary_drift(workspace, &options);
    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!("\n=== xGauntlet Two-Way Glossary Drift Gate ===");
        if report.is_valid {
            println!("Status: [PASSED] 0 orphan models, 0 avoid violations.");
        } else {
            println!("Status: [FAILED]");
            for orphan in &report.orphan_models {
                println!("  🛑 Orphan Model: {orphan}");
            }
            for violation in &report.avoid_violations {
                println!("  🛑 Prohibited Synonym: {}", violation.message);
            }
            for d in &report.diagnostics {
                println!("  💡 Remediation: {}", d.remediation_hint);
            }
        }
    }
    Ok(report.is_valid)
}

pub fn run_topology_export(workspace: &Path, format: &str, level: &str) -> anyhow::Result<()> {
    let options = xgauntlet_core::TopologyOptions {
        workspace_root: workspace.to_path_buf(),
        ..Default::default()
    };
    let graph = xgauntlet_core::scan_workspace_topology(&options)?;
    let fmt = format.trim().to_ascii_lowercase();
    if fmt == "json" {
        println!("{}", graph.export_json()?);
    } else {
        let mermaid_level = if level.trim().eq_ignore_ascii_case("detailed") {
            xgauntlet_core::MermaidLevel::Detailed
        } else {
            xgauntlet_core::MermaidLevel::Conceptual
        };
        let export_opts = xgauntlet_core::MermaidExportOptions {
            direction: "TB".to_string(),
            level: mermaid_level,
            domain_only: mermaid_level == xgauntlet_core::MermaidLevel::Conceptual,
        };
        let diagram = xgauntlet_core::export_mermaid_class_diagram(&graph, &export_opts);
        println!("{diagram}");
    }
    Ok(())
}
