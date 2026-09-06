//! Shift-left specification validation and Aristotelian domain glossary enforcement.

use std::fs;
use std::path::Path;

use crate::features::diagnostics::{DiagnosticFinding, FindingType};
use crate::features::tasks::models::SpecReadinessReport;
use crate::features::tasks::okf::parse_frontmatter;
use crate::features::tasks::parser::parse_task_content;

/// Validates raw content of CONTEXT.md against Aristotle's formula (*definitio per genus et differentiam*).
pub fn validate_context_content(content: &str) -> Vec<DiagnosticFinding> {
    let mut diagnostics = Vec::new();

    // Extract terms formatted as **Term**:
    let mut terms: Vec<(String, String)> = Vec::new();
    let mut current_term: Option<String> = None;
    let mut current_body = String::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("**") && trimmed.contains("**:") {
            if let Some(prev_term) = current_term.take() {
                terms.push((prev_term, current_body.trim().to_string()));
                current_body.clear();
            }

            if let Some(end_idx) = trimmed.find("**:") {
                let term_name = trimmed[2..end_idx].trim().to_string();
                current_term = Some(term_name);
            }
        } else if current_term.is_some() {
            current_body.push_str(line);
            current_body.push('\n');
        }
    }

    if let Some(last_term) = current_term {
        terms.push((last_term, current_body.trim().to_string()));
    }

    if terms.is_empty() {
        diagnostics.push(DiagnosticFinding::new(
            FindingType::GeneralError,
            "ARISTOTLE_FORMAT_VIOLATION",
            "CONTEXT.md",
            "No valid Aristotle glossary terms ('**Term**:') found in CONTEXT.md.",
            "Define terms using '**Term**:\n<Definition>\n_Avoid_: <synonyms>'.",
        ));
        return diagnostics;
    }

    for (term_name, body) in terms {
        let lower = body.to_ascii_lowercase();
        let has_avoid = lower.contains("_avoid_:") || lower.contains("_avoid:");

        if !has_avoid {
            diagnostics.push(DiagnosticFinding::new(
                FindingType::GeneralError,
                "ARISTOTLE_FORMAT_VIOLATION",
                "CONTEXT.md",
                format!("Term '{term_name}' in CONTEXT.md is missing an '_Avoid_:' line with prohibited synonyms."),
                format!("Add '_Avoid_: <synonyms>' under '**{term_name}**:' definition."),
            ));
        }

        // Genus et differentiam check: verify non-empty definition preceding _Avoid_:
        let def_part = if let Some(idx) = lower.find("_avoid") {
            body[..idx].trim()
        } else {
            body.trim()
        };

        if def_part.is_empty() {
            diagnostics.push(DiagnosticFinding::new(
                FindingType::GeneralError,
                "ARISTOTLE_FORMAT_VIOLATION",
                "CONTEXT.md",
                format!("Term '{term_name}' in CONTEXT.md is missing an Aristotle definition sentence (genus et differentiam)."),
                format!("Provide a definition sentence specifying genus and differentia for '**{term_name}**:'.")
            ));
        }
    }

    diagnostics
}

/// Validates that `CONTEXT.md` in the workspace root adheres to the Aristotle definition format.
pub fn validate_context_glossary(workspace: &Path) -> Vec<DiagnosticFinding> {
    let context_path = workspace.join("CONTEXT.md");
    if !context_path.is_file() {
        return vec![DiagnosticFinding::new(
            FindingType::GeneralError,
            "MISSING_CONTEXT_GLOSSARY",
            "CONTEXT.md",
            "Missing 'CONTEXT.md' ubiquitous language glossary in workspace root.",
            "Create CONTEXT.md defining core domain terms using Aristotle's formula.",
        )];
    }

    match fs::read_to_string(&context_path) {
        Ok(content) => validate_context_content(&content),
        Err(err) => vec![DiagnosticFinding::new(
            FindingType::GeneralError,
            "CONTEXT_READ_ERROR",
            "CONTEXT.md",
            format!("Failed to read 'CONTEXT.md': {err}"),
            "Ensure CONTEXT.md has valid UTF-8 permissions.",
        )],
    }
}

/// Evaluates a task package file for specification completeness and business rules.
pub fn check_task_specification(task_path: &Path, workspace: &Path) -> SpecReadinessReport {
    let resolved_task = if task_path.is_absolute() {
        task_path.to_path_buf()
    } else {
        workspace.join(task_path)
    };

    let rel_task = match resolved_task.strip_prefix(workspace) {
        Ok(p) => p.display().to_string(),
        Err(_) => resolved_task.display().to_string(),
    };

    let task_id = resolved_task
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    let mut report = SpecReadinessReport::new(task_id);

    if !resolved_task.is_file() {
        report.is_valid = false;
        report.diagnostics.push(DiagnosticFinding::new(
            FindingType::GeneralError,
            "TASK_FILE_NOT_FOUND",
            &rel_task,
            format!("Task file '{rel_task}' does not exist."),
            "Specify a valid path to an existing task file in tasks/.",
        ));
        return report;
    }

    report.inspected_files.push(rel_task.clone());

    let raw_content = match fs::read_to_string(&resolved_task) {
        Ok(c) => c,
        Err(err) => {
            report.is_valid = false;
            report.diagnostics.push(DiagnosticFinding::new(
                FindingType::GeneralError,
                "TASK_READ_ERROR",
                &rel_task,
                format!("Failed to read task file: {err}"),
                "Ensure task file has readable permissions.",
            ));
            return report;
        }
    };

    // 1. OKF Frontmatter Validation
    let frontmatter_result = parse_frontmatter(&raw_content);
    let has_valid_okf = match frontmatter_result {
        Ok((Some(ref meta), _)) => !meta.doc_type.trim().is_empty(),
        _ => false,
    };

    if !has_valid_okf {
        report.diagnostics.push(DiagnosticFinding::new(
            FindingType::GeneralError,
            "INVALID_OKF_METADATA",
            &rel_task,
            "Task file is missing required Open Knowledge Format (OKF v0.2) YAML frontmatter.",
            "Add YAML frontmatter with 'type: Task Package', 'title', 'status', and 'generated' block.",
        ));
    }

    // 2. Parse Task Package Information
    let task_info = match parse_task_content(&raw_content, &report.task_id) {
        Ok(info) => info,
        Err(err) => {
            report.diagnostics.push(DiagnosticFinding::new(
                FindingType::GeneralError,
                "TASK_PARSE_ERROR",
                &rel_task,
                format!("Failed to parse task markdown: {err}"),
                "Correct task format and structure.",
            ));
            report.is_valid = false;
            return report;
        }
    };

    report.must_not_rules = task_info.must_not.clone();
    report.acceptance_criteria = task_info.acceptance_criteria.clone();

    // 3. Validate Purpose (Formål)
    let lower_content = raw_content.to_ascii_lowercase();
    let has_purpose_header =
        lower_content.contains("formål") || lower_content.contains("## purpose");
    if !has_purpose_header || task_info.purpose.trim().is_empty() {
        report.diagnostics.push(DiagnosticFinding::new(
            FindingType::GeneralError,
            "MISSING_PURPOSE",
            &rel_task,
            "Task specification is missing a '## 🎯 Formål' or '## Purpose' section.",
            "Add a '## 🎯 Formål' section detailing the concrete objective and scope.",
        ));
    }

    // 4. Validate Acceptance Criteria (Acceptkriterier)
    if task_info.acceptance_criteria.is_empty() {
        report.diagnostics.push(DiagnosticFinding::new(
            FindingType::GeneralError,
            "MISSING_ACCEPTANCE_CRITERIA",
            &rel_task,
            "Task specification is missing executable acceptance criteria items ('- [ ]').",
            "Add concrete '- [ ]' acceptance criteria under '## 📋 Acceptance Criteria'.",
        ));
    }

    // 5. Validate Business Rules (Must NOT Invariants)
    if task_info.must_not.is_empty() {
        report.diagnostics.push(DiagnosticFinding::new(
            FindingType::GeneralError,
            "MISSING_MUST_NOT",
            &rel_task,
            "Task specification is missing negative business constraints and invariants under '## 🚫 Must NOT'.",
            "Add at least one explicit negative constraint or boundary under '## 🚫 Must NOT' to prevent unauthorized architectural deviations.",
        ));
    }

    // 6. Validate CONTEXT.md Glossary Format
    let context_diags = validate_context_glossary(workspace);
    if workspace.join("CONTEXT.md").is_file() {
        report.inspected_files.push("CONTEXT.md".to_string());
    }
    report.diagnostics.extend(context_diags);

    report.is_valid = report.diagnostics.is_empty();
    report
}

/// Evaluates all task package files found in `tasks/`.
pub fn check_all_tasks(workspace: &Path) -> Vec<SpecReadinessReport> {
    let tasks_dir = workspace.join("tasks");
    if !tasks_dir.is_dir() {
        return Vec::new();
    }

    let entries = match fs::read_dir(&tasks_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let mut paths: Vec<_> = entries
        .filter_map(|e| e.ok().map(|d| d.path()))
        .filter(|p| p.extension().is_some_and(|ext| ext == "md"))
        .collect();
    paths.sort();

    paths
        .into_iter()
        .map(|p| check_task_specification(&p, workspace))
        .collect()
}
