//! Two-way domain glossary drift validation between source code models and CONTEXT.md.

use std::fs;
use std::path::{Path, PathBuf};

use super::models::{GlossaryDriftOptions, GlossaryDriftReport};
use crate::features::diagnostics::{DiagnosticFinding, FindingType};

fn find_model_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let p = entry.path();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.starts_with('.') || name_str == "target" || name_str == "node_modules" {
            continue;
        }
        if p.is_dir() {
            find_model_files(&p, out);
        } else if p.is_file() && (name_str == "models.rs" || name_str.ends_with("_models.rs")) {
            out.push(p);
        }
    }
}

/// Validates two-way glossary drift between source code models and CONTEXT.md.
pub fn validate_glossary_drift(
    workspace: &Path,
    options: &GlossaryDriftOptions,
) -> GlossaryDriftReport {
    let mut report = GlossaryDriftReport::new();
    let context_path = workspace.join("CONTEXT.md");
    if !context_path.is_file() {
        report.is_valid = false;
        report.diagnostics.push(DiagnosticFinding::new(
            FindingType::GeneralError,
            "MISSING_CONTEXT_FILE",
            "CONTEXT.md",
            "CONTEXT.md ubiquitous language glossary not found in workspace root.",
            "Create CONTEXT.md defining core domain concepts.",
        ));
        return report;
    }

    let context_content = match fs::read_to_string(&context_path) {
        Ok(c) => c,
        Err(e) => {
            report.is_valid = false;
            report.diagnostics.push(DiagnosticFinding::new(
                FindingType::GeneralError,
                "CONTEXT_READ_ERROR",
                "CONTEXT.md",
                format!("Failed to read CONTEXT.md: {e}"),
                "Ensure CONTEXT.md has valid UTF-8 permissions.",
            ));
            return report;
        }
    };

    // Extract terms and avoid lists
    let mut glossary_terms: Vec<(String, String)> = Vec::new();
    let mut avoid_words: Vec<String> = Vec::new();

    let mut current_term: Option<String> = None;
    let mut current_body = String::new();

    for line in context_content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("**") && trimmed.contains("**:") {
            if let Some(prev) = current_term.take() {
                glossary_terms.push((prev, current_body.trim().to_string()));
                current_body.clear();
            }
            if let Some(end_idx) = trimmed.find("**:") {
                let term = trimmed[2..end_idx].trim().to_string();
                current_term = Some(term);
            }
        } else if current_term.is_some() {
            current_body.push_str(line);
            current_body.push('\n');
        }
    }
    if let Some(last) = current_term {
        glossary_terms.push((last, current_body.trim().to_string()));
    }

    for (_, body) in &glossary_terms {
        let lower = body.to_ascii_lowercase();
        if let Some(idx) = lower.find("_avoid_:") {
            let rest = &body[idx + "_avoid_:".len()..];
            let avoid_line = rest.lines().next().unwrap_or(rest);
            for word in avoid_line.split(',') {
                let clean = word
                    .trim()
                    .trim_matches(|c: char| !c.is_alphanumeric())
                    .to_ascii_lowercase();
                if !clean.is_empty() && !avoid_words.contains(&clean) {
                    avoid_words.push(clean);
                }
            }
        }
    }

    // Normalized terms list (includes primary concept and parenthetical `/` or `,` separated model aliases)
    let mut norm_terms: Vec<String> = Vec::new();
    for (t, _) in &glossary_terms {
        for part in t.split(['(', ')', '/', ',']) {
            let norm: String = part
                .chars()
                .filter(|c| c.is_alphanumeric())
                .collect::<String>()
                .to_ascii_lowercase();
            if !norm.is_empty() && !norm_terms.contains(&norm) {
                norm_terms.push(norm);
            }
        }
    }

    // Find model files
    let mut model_files = Vec::new();
    find_model_files(workspace, &mut model_files);

    for file_path in &model_files {
        let rel_path = file_path
            .strip_prefix(workspace)
            .unwrap_or(file_path)
            .to_string_lossy()
            .replace('\\', "/");

        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        for (line_idx, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            // Struct or enum detection
            let struct_or_enum = trimmed
                .strip_prefix("pub struct ")
                .or_else(|| trimmed.strip_prefix("pub enum "))
                .map(|rest| rest.trim());

            if let Some(raw_decl) = struct_or_enum {
                let ident = raw_decl
                    .split(|c: char| !c.is_alphanumeric() && c != '_')
                    .next()
                    .unwrap_or("")
                    .trim();

                if !ident.is_empty() {
                    let norm_ident = ident
                        .chars()
                        .filter(|c| c.is_alphanumeric())
                        .collect::<String>()
                        .to_ascii_lowercase();
                    let is_matched = norm_terms.iter().any(|t| {
                        norm_ident == *t || norm_ident.starts_with(t) || t.starts_with(&norm_ident)
                    });

                    if !is_matched {
                        report.orphan_models.push(format!("{rel_path}::{ident}"));
                        report.diagnostics.push(
                            DiagnosticFinding::new(
                                FindingType::InvariantViolation,
                                "ORPHAN_DOMAIN_MODEL",
                                &rel_path,
                                format!("Domain entity '{ident}' in '{rel_path}' is not defined in CONTEXT.md."),
                                format!("Define '{ident}' in CONTEXT.md using Aristotle's formula (**Term**: A genus, that differentia. _Avoid_: ...) before introducing production models."),
                            )
                            .with_line(line_idx + 1),
                        );
                    }
                }
            }

            // Avoid synonyms check
            if options.check_avoid
                && !trimmed.starts_with("pub fn ")
                && !trimmed.starts_with("pub async fn ")
                && !trimmed.starts_with("pub const ")
                && !trimmed.starts_with("pub type ")
                && !trimmed.starts_with("pub use ")
                && !trimmed.starts_with("pub struct ")
                && !trimmed.starts_with("pub enum ")
                && !trimmed.starts_with("pub mod ")
                && !trimmed.starts_with("pub trait ")
                && trimmed.contains(':')
            {
                if let Some(field_decl) = trimmed.strip_prefix("pub ") {
                    let field_name = field_decl.split(':').next().unwrap_or("").trim();

                    let field_lower = field_name.to_ascii_lowercase();
                    for avoid in &avoid_words {
                        if field_lower == *avoid
                            || field_lower.contains(&format!("{avoid}_"))
                            || field_lower.contains(&format!("_{avoid}"))
                        {
                            let finding = DiagnosticFinding::new(
                                FindingType::InvariantViolation,
                                "PROHIBITED_SYNONYM_USED",
                                &rel_path,
                                format!("Field '{field_name}' in '{rel_path}' uses prohibited synonym '{avoid}' forbidden by CONTEXT.md."),
                                format!("Rename field '{field_name}' to align with ubiquitous domain terms in CONTEXT.md."),
                            )
                            .with_line(line_idx + 1);
                            report.avoid_violations.push(finding.clone());
                            report.diagnostics.push(finding);
                        }
                    }
                }
            }
        }
    }

    report.is_valid = report.orphan_models.is_empty()
        && report.avoid_violations.is_empty()
        && report.diagnostics.is_empty();

    report
}
