//! In-repo governance, specification, and configuration diagnostic checks.

use crate::features::config::{load_config, validate_config, ValidationSeverity};
use crate::features::doctor::models::{DoctorCategory, DoctorCheckItem, DoctorCheckStatus};
use crate::features::tasks::{resolve_active_task_id, validate_context_glossary};
use std::fs;
use std::path::Path;
use std::time::Instant;

/// Inspects workspace governance artifacts: gauntlet.toml, spec.md, CONTEXT.md, tasks/, docs/adr/, and agent files.
pub fn check_governance(workspace: &Path) -> Vec<DoctorCheckItem> {
    let mut checks = Vec::new();

    // 1. Declarative gauntlet configuration (gauntlet.toml / gauntlet.json)
    let start = Instant::now();
    let toml_path = workspace.join("gauntlet.toml");
    let json_path = workspace.join("gauntlet.json");
    let has_config = toml_path.is_file() || json_path.is_file();

    if has_config {
        match load_config(workspace) {
            Ok(cfg) => {
                let report = validate_config(&cfg, Some(workspace));
                let duration = start.elapsed().as_millis() as u64;

                if report.is_valid {
                    let layer_count = cfg.layers.len();
                    checks.push(DoctorCheckItem {
                        name: "governance_gauntlet_toml".to_string(),
                        category: DoctorCategory::Governance,
                        status: DoctorCheckStatus::Pass,
                        message: format!(
                            "Valid declarative configuration ({layer_count} layers configured)"
                        ),
                        detail: Some(format!("Stack profile: {}", cfg.stack)),
                        remediation: None,
                        duration_ms: duration,
                    });
                } else {
                    let has_errors = report
                        .issues
                        .iter()
                        .any(|i| i.severity == ValidationSeverity::Error);
                    let status = if has_errors {
                        DoctorCheckStatus::Fail
                    } else {
                        DoctorCheckStatus::Warn
                    };

                    let detail = report
                        .issues
                        .iter()
                        .map(|i| format!("[{:?}] {}: {}", i.severity, i.code, i.message))
                        .collect::<Vec<_>>()
                        .join("; ");

                    let remediation = report
                        .issues
                        .first()
                        .and_then(|i| i.remediation.clone())
                        .or_else(|| {
                            Some("Correct configuration defects in gauntlet.toml.".to_string())
                        });

                    checks.push(DoctorCheckItem {
                        name: "governance_gauntlet_toml".to_string(),
                        category: DoctorCategory::Governance,
                        status,
                        message: format!("Configuration has {} issue(s)", report.issues.len()),
                        detail: Some(detail),
                        remediation,
                        duration_ms: duration,
                    });
                }
            }
            Err(e) => {
                let duration = start.elapsed().as_millis() as u64;
                checks.push(DoctorCheckItem {
                    name: "governance_gauntlet_toml".to_string(),
                    category: DoctorCategory::Governance,
                    status: DoctorCheckStatus::Fail,
                    message: format!("Failed to parse gauntlet.toml: {e}"),
                    detail: None,
                    remediation: Some(
                        "Fix syntax errors in gauntlet.toml or run 'xgauntlet init --force' to regenerate."
                            .to_string(),
                    ),
                    duration_ms: duration,
                });
            }
        }
    } else {
        let duration = start.elapsed().as_millis() as u64;
        checks.push(DoctorCheckItem {
            name: "governance_gauntlet_toml".to_string(),
            category: DoctorCategory::Governance,
            status: DoctorCheckStatus::Warn,
            message: "Missing 'gauntlet.toml' configuration in workspace root".to_string(),
            detail: None,
            remediation: Some(
                "Run 'xgauntlet init' to generate a declarative gauntlet.toml with stack defaults."
                    .to_string(),
            ),
            duration_ms: duration,
        });
    }

    // 2. Macro Specification (spec.md)
    let start = Instant::now();
    let spec_path = workspace.join("spec.md");
    let duration = start.elapsed().as_millis() as u64;

    if spec_path.is_file() {
        let size = fs::metadata(&spec_path).map(|m| m.len()).unwrap_or(0);
        checks.push(DoctorCheckItem {
            name: "governance_spec".to_string(),
            category: DoctorCategory::Governance,
            status: DoctorCheckStatus::Pass,
            message: format!("spec.md present ({size} bytes)"),
            detail: Some(format!("Path: {}", spec_path.display())),
            remediation: None,
            duration_ms: duration,
        });
    } else {
        checks.push(DoctorCheckItem {
            name: "governance_spec".to_string(),
            category: DoctorCategory::Governance,
            status: DoctorCheckStatus::Warn,
            message: "Missing 'spec.md' macro specification".to_string(),
            detail: None,
            remediation: Some(
                "Run 'xgauntlet init' or create 'spec.md' to define system invariants and capabilities."
                    .to_string(),
            ),
            duration_ms: duration,
        });
    }

    // 3. Ubiquitous Domain Glossary (CONTEXT.md)
    let start = Instant::now();
    let context_path = workspace.join("CONTEXT.md");

    if context_path.is_file() {
        let findings = validate_context_glossary(workspace);
        let duration = start.elapsed().as_millis() as u64;

        if findings.is_empty() {
            checks.push(DoctorCheckItem {
                name: "governance_context".to_string(),
                category: DoctorCategory::Governance,
                status: DoctorCheckStatus::Pass,
                message: "CONTEXT.md valid (Aristotelian definitions verified)".to_string(),
                detail: Some(format!("Path: {}", context_path.display())),
                remediation: None,
                duration_ms: duration,
            });
        } else {
            let detail = findings
                .iter()
                .map(|f| format!("{}: {}", f.tool_name, f.message))
                .collect::<Vec<_>>()
                .join("; ");
            let remediation = findings
                .first()
                .map(|f| f.remediation_hint.clone())
                .unwrap_or_else(|| "Align CONTEXT.md with Aristotle's formula.".to_string());

            checks.push(DoctorCheckItem {
                name: "governance_context".to_string(),
                category: DoctorCategory::Governance,
                status: DoctorCheckStatus::Warn,
                message: format!("CONTEXT.md has {} glossary defect(s)", findings.len()),
                detail: Some(detail),
                remediation: Some(remediation),
                duration_ms: duration,
            });
        }
    } else {
        let duration = start.elapsed().as_millis() as u64;
        checks.push(DoctorCheckItem {
            name: "governance_context".to_string(),
            category: DoctorCategory::Governance,
            status: DoctorCheckStatus::Warn,
            message: "Missing 'CONTEXT.md' domain glossary".to_string(),
            detail: None,
            remediation: Some(
                "Run 'xgauntlet init' to generate CONTEXT.md with core ubiquitous domain terms."
                    .to_string(),
            ),
            duration_ms: duration,
        });
    }

    // 4. Tasks Directory and Active Task Tracking (tasks/)
    let start = Instant::now();
    let tasks_dir = workspace.join("tasks");

    if tasks_dir.is_dir() {
        let entries = fs::read_dir(&tasks_dir)
            .ok()
            .map(|rd| {
                rd.filter_map(|e| e.ok().map(|d| d.path()))
                    .filter(|p| p.extension().is_some_and(|ext| ext == "md"))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let active_id = resolve_active_task_id(workspace);
        let duration = start.elapsed().as_millis() as u64;

        let active_str = match &active_id {
            Some(id) => format!("Active task: {id}"),
            None => "No active task".to_string(),
        };

        checks.push(DoctorCheckItem {
            name: "governance_tasks".to_string(),
            category: DoctorCategory::Governance,
            status: DoctorCheckStatus::Pass,
            message: format!("tasks/ present ({} tasks, {active_str})", entries.len()),
            detail: Some(format!("Directory: {}", tasks_dir.display())),
            remediation: None,
            duration_ms: duration,
        });
    } else {
        let duration = start.elapsed().as_millis() as u64;
        checks.push(DoctorCheckItem {
            name: "governance_tasks".to_string(),
            category: DoctorCategory::Governance,
            status: DoctorCheckStatus::Warn,
            message: "Missing 'tasks/' directory".to_string(),
            detail: None,
            remediation: Some(
                "Run 'xgauntlet init' to create tasks/ with initial bootstrap task package."
                    .to_string(),
            ),
            duration_ms: duration,
        });
    }

    // 5. Architecture Decision Records (docs/adr/)
    let start = Instant::now();
    let adr_dir = workspace.join("docs").join("adr");
    let duration = start.elapsed().as_millis() as u64;

    if adr_dir.is_dir() {
        let count = fs::read_dir(&adr_dir)
            .ok()
            .map(|rd| {
                rd.filter_map(|e| e.ok().map(|d| d.path()))
                    .filter(|p| p.extension().is_some_and(|ext| ext == "md"))
                    .count()
            })
            .unwrap_or(0);

        checks.push(DoctorCheckItem {
            name: "governance_adr".to_string(),
            category: DoctorCategory::Governance,
            status: DoctorCheckStatus::Pass,
            message: format!("docs/adr/ present ({count} records)"),
            detail: Some(format!("Directory: {}", adr_dir.display())),
            remediation: None,
            duration_ms: duration,
        });
    } else {
        checks.push(DoctorCheckItem {
            name: "governance_adr".to_string(),
            category: DoctorCategory::Governance,
            status: DoctorCheckStatus::Info,
            message: "No 'docs/adr/' directory found".to_string(),
            detail: None,
            remediation: Some(
                "Create 'docs/adr/' to record architectural decisions and irreversible trade-offs."
                    .to_string(),
            ),
            duration_ms: duration,
        });
    }

    // 6. Agent Configuration (.agents/AGENTS.md and hooks.json)
    let start = Instant::now();
    let agents_md = workspace.join(".agents").join("AGENTS.md");
    let claude_md = workspace.join("CLAUDE.md");
    let hooks_json = workspace.join(".agents").join("hooks.json");
    let duration = start.elapsed().as_millis() as u64;

    let has_guidelines = agents_md.is_file() || claude_md.is_file();
    let has_hooks = hooks_json.is_file();

    if has_guidelines && has_hooks {
        checks.push(DoctorCheckItem {
            name: "governance_agent_harness".to_string(),
            category: DoctorCategory::Governance,
            status: DoctorCheckStatus::Pass,
            message: "Agent guidelines and policy hooks configured".to_string(),
            detail: Some(
                "Found .agents/AGENTS.md (or CLAUDE.md) and .agents/hooks.json".to_string(),
            ),
            remediation: None,
            duration_ms: duration,
        });
    } else if has_guidelines {
        checks.push(DoctorCheckItem {
            name: "governance_agent_harness".to_string(),
            category: DoctorCategory::Governance,
            status: DoctorCheckStatus::Info,
            message: "Agent guidelines found, but .agents/hooks.json is missing".to_string(),
            detail: None,
            remediation: Some(
                "Run 'xgauntlet init' to configure PreToolUse policy hooks.".to_string(),
            ),
            duration_ms: duration,
        });
    } else {
        checks.push(DoctorCheckItem {
            name: "governance_agent_harness".to_string(),
            category: DoctorCategory::Governance,
            status: DoctorCheckStatus::Info,
            message: "No agent guidelines (.agents/AGENTS.md) or hooks configured".to_string(),
            detail: None,
            remediation: Some(
                "Run 'xgauntlet init' to scaffold AI agent guidelines and security hooks."
                    .to_string(),
            ),
            duration_ms: duration,
        });
    }

    checks
}
