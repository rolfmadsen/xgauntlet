//! Shift-left specification validation and Aristotelian domain glossary enforcement.

use std::fs;
use std::path::Path;

use crate::features::diagnostics::{DiagnosticFinding, FindingType};
use crate::features::tasks::models::{AdrReadinessReport, SpecReadinessReport};
use crate::features::tasks::okf::{parse_frontmatter, OkfError};
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

    // 6. Validate Task Scope and Test Targets (for active tasks)
    if task_info.status.is_active() {
        if task_info.scope.is_empty() {
            report.diagnostics.push(DiagnosticFinding::new(
                FindingType::GeneralError,
                "MISSING_TASK_SCOPE",
                &rel_task,
                "Active task specification is missing declared target paths in frontmatter 'scope: [...]'.",
                "Add frontmatter 'scope: [\"src/features/<slice>/**\"]' declaring the bounded directory scope for this task.",
            ));
        }

        if task_info.test_refs.is_empty() {
            report.diagnostics.push(DiagnosticFinding::new(
                FindingType::GeneralError,
                "MISSING_TASK_TEST_REFS",
                &rel_task,
                "Active task specification is missing declared test file references in frontmatter 'test_refs: [...]'.",
                "Add frontmatter 'test_refs: [\"tests/<slice>_test.rs\"]' referencing the dedicated test target for this task.",
            ));
        }

        // Anti-Monolithic Test Guard: Check if declared test target exists and exceeds 1,000 lines
        for test_ref in &task_info.test_refs {
            let test_path = if Path::new(test_ref).is_absolute() {
                Path::new(test_ref).to_path_buf()
            } else {
                workspace.join(test_ref)
            };
            if test_path.is_file() {
                if let Ok(test_content) = fs::read_to_string(&test_path) {
                    let line_count = test_content.lines().count();
                    let is_refactor = task_info
                        .intent
                        .as_deref()
                        .map(|i| i.to_ascii_lowercase().contains("refactor"))
                        .unwrap_or(false);
                    if line_count > 1000 && !is_refactor {
                        report.diagnostics.push(DiagnosticFinding::new(
                            FindingType::GeneralError,
                            "MONOLITHIC_TEST_FILE_PROHIBITED",
                            &rel_task,
                            format!("Test target '{test_ref}' contains {line_count} lines (exceeds 1000 line limit). Monolithic test files are prohibited."),
                            "Decompose and extract slice-specific tests into a separate, focused test file (e.g. tests/<slice>_test.rs).",
                        ));
                    }
                }
            }
        }
    }

    // 7. Validate CONTEXT.md Glossary Format
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

/// Validates an Architectural Decision Record (ADR) file against OKF v0.2 frontmatter rules.
pub fn check_adr_frontmatter(adr_path: &Path) -> AdrReadinessReport {
    let adr_id = adr_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    let rel_adr = adr_path.display().to_string();
    let mut report = AdrReadinessReport::new(adr_id);

    if !adr_path.is_file() {
        report.is_valid = false;
        report.diagnostics.push(DiagnosticFinding::new(
            FindingType::GeneralError,
            "ADR_FILE_NOT_FOUND",
            &rel_adr,
            format!("ADR file '{rel_adr}' does not exist."),
            "Specify a valid path to an existing ADR file in docs/adr/.",
        ));
        return report;
    }

    report.inspected_files.push(rel_adr.clone());

    let raw_content = match fs::read_to_string(adr_path) {
        Ok(c) => c,
        Err(err) => {
            report.is_valid = false;
            report.diagnostics.push(DiagnosticFinding::new(
                FindingType::GeneralError,
                "ADR_READ_ERROR",
                &rel_adr,
                format!("Failed to read ADR file: {err}"),
                "Ensure ADR file has readable permissions.",
            ));
            return report;
        }
    };

    let meta = match parse_frontmatter(&raw_content) {
        Ok((Some(meta), _)) => meta,
        Ok((None, _)) => {
            report.is_valid = false;
            report.diagnostics.push(DiagnosticFinding::new(
                FindingType::GeneralError,
                "INVALID_OKF_METADATA",
                &rel_adr,
                "ADR file is missing required Open Knowledge Format (OKF v0.2) YAML frontmatter.",
                "Add YAML frontmatter with 'type: Architectural Decision Record', 'title', 'status', and 'generated' block.",
            ));
            return report;
        }
        Err(OkfError::InvalidActor(actor)) => {
            report.is_valid = false;
            report.diagnostics.push(DiagnosticFinding::new(
                FindingType::GeneralError,
                "INVALID_ACTOR",
                &rel_adr,
                format!("Invalid actor format: {actor}"),
                "Use valid OKF actor format: 'human:<id>', 'process:<id>', or '<agent>/<version>'.",
            ));
            return report;
        }
        Err(OkfError::InvalidTimestamp(ts, msg)) => {
            report.is_valid = false;
            report.diagnostics.push(DiagnosticFinding::new(
                FindingType::GeneralError,
                "INVALID_TIMESTAMP",
                &rel_adr,
                format!("Invalid timestamp '{ts}': {msg}"),
                "Use strict ISO 8601 UTC timestamp format (e.g. '2026-09-19T13:50:00Z').",
            ));
            return report;
        }
        Err(OkfError::MissingType) => {
            report.is_valid = false;
            report.diagnostics.push(DiagnosticFinding::new(
                FindingType::GeneralError,
                "MISSING_TYPE",
                &rel_adr,
                "ADR file is missing required 'type' field in frontmatter.",
                "Add 'type: Architectural Decision Record' to frontmatter.",
            ));
            return report;
        }
        Err(err) => {
            report.is_valid = false;
            report.diagnostics.push(DiagnosticFinding::new(
                FindingType::GeneralError,
                "INVALID_OKF_METADATA",
                &rel_adr,
                format!("Frontmatter parse error: {err}"),
                "Ensure YAML frontmatter is well-formed.",
            ));
            return report;
        }
    };

    // 1. Validate doc_type
    if meta.doc_type != "Architectural Decision Record" {
        report.diagnostics.push(DiagnosticFinding::new(
            FindingType::GeneralError,
            "INVALID_TYPE",
            &rel_adr,
            format!(
                "Invalid ADR doc_type '{}', expected 'Architectural Decision Record'.",
                meta.doc_type
            ),
            "Set 'type: Architectural Decision Record' in frontmatter.",
        ));
    }

    // 2. Validate title
    match meta.title.as_deref() {
        Some(t) if !t.trim().is_empty() => {}
        _ => {
            report.diagnostics.push(DiagnosticFinding::new(
                FindingType::GeneralError,
                "MISSING_TITLE",
                &rel_adr,
                "ADR is missing required 'title' field in frontmatter.",
                "Add 'title: [Title]' to frontmatter.",
            ));
        }
    }

    // 3. Validate status
    match meta.status.as_deref() {
        Some(s) if !s.trim().is_empty() => {}
        _ => {
            report.diagnostics.push(DiagnosticFinding::new(
                FindingType::GeneralError,
                "MISSING_STATUS",
                &rel_adr,
                "ADR is missing required 'status' field in frontmatter.",
                "Add 'status: draft | accepted | deprecated | superseded' to frontmatter.",
            ));
        }
    }

    // 4. Validate generated block
    match meta.generated.as_ref() {
        None => {
            report.diagnostics.push(DiagnosticFinding::new(
                FindingType::GeneralError,
                "GENERATED_MISSING",
                &rel_adr,
                "ADR is missing required 'generated' metadata block in frontmatter.",
                "Add 'generated: { by: human:<author>, at: \"YYYY-MM-DDTHH:MM:SSZ\" }' to frontmatter.",
            ));
        }
        Some(gen) => {
            if gen.by.trim().is_empty() {
                report.diagnostics.push(DiagnosticFinding::new(
                    FindingType::GeneralError,
                    "INVALID_ACTOR",
                    &rel_adr,
                    "ADR 'generated.by' is empty.",
                    "Specify a valid OKF actor in 'generated.by'.",
                ));
            }
            match gen.at.as_deref() {
                Some(at) if !at.trim().is_empty() => {}
                _ => {
                    report.diagnostics.push(DiagnosticFinding::new(
                        FindingType::GeneralError,
                        "INVALID_TIMESTAMP",
                        &rel_adr,
                        "ADR 'generated' block is missing required 'at' timestamp.",
                        "Add valid ISO 8601 UTC timestamp to 'generated.at'.",
                    ));
                }
            }
        }
    }

    // 5. Validate verified block (warning if missing, not an error)
    if !meta.has_verified {
        report.diagnostics.push(DiagnosticFinding::warning(
            "MISSING_VERIFIED",
            &rel_adr,
            "ADR is missing recommended 'verified' field in frontmatter.",
            "Add 'verified: []' or a list of verification entries.",
        ));
    }

    report.is_valid = !report
        .diagnostics
        .iter()
        .any(|d| d.finding_type != FindingType::Warning);
    report
}

/// Validates all ADR markdown files in `docs/adr/`.
pub fn check_all_adrs(workspace: &Path) -> Vec<AdrReadinessReport> {
    let adr_dir = workspace.join("docs").join("adr");
    if !adr_dir.is_dir() {
        return Vec::new();
    }

    let entries = match fs::read_dir(&adr_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let mut paths: Vec<_> = entries
        .filter_map(|e| e.ok().map(|d| d.path()))
        .filter(|p| {
            if !p.is_file() || p.extension().is_none_or(|ext| ext != "md") {
                return false;
            }
            let name = p
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            name != "readme.md" && name != "template.md"
        })
        .collect();
    paths.sort();

    paths
        .into_iter()
        .map(|p| check_adr_frontmatter(&p))
        .collect()
}
