//! Extractors for linter outputs (Clippy, Ruff, ESLint).

use crate::features::diagnostics::models::{DiagnosticFinding, FindingType};

/// Extracts Clippy warnings and errors from compiler output.
pub fn extract_clippy_findings(output: &str) -> Vec<DiagnosticFinding> {
    let mut findings = Vec::new();
    let lines: Vec<&str> = output.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("error[E")
            || trimmed.starts_with("error:")
            || trimmed.starts_with("warning:")
        {
            let message = trimmed.to_string();
            let mut file_path = "src/lib.rs".to_string();
            let mut line_number = None;
            let mut column_number = None;

            // Search for " --> path:line:col" in the next few lines
            for next_line in lines.iter().skip(i + 1).take(5) {
                let trimmed_next = next_line.trim();
                if let Some(stripped) = trimmed_next.strip_prefix("-->") {
                    let loc_part = stripped.trim();
                    let parts: Vec<&str> = loc_part.split(':').collect();
                    if !parts.is_empty() {
                        file_path = parts[0].to_string();
                    }
                    if parts.len() > 1 {
                        line_number = parts[1].parse::<usize>().ok();
                    }
                    if parts.len() > 2 {
                        column_number = parts[2].parse::<usize>().ok();
                    }
                    break;
                }
            }

            let remediation_hint =
                "Fix compiler/clippy warning or error indicated by rustc.".to_string();
            let mut finding = DiagnosticFinding::new(
                FindingType::LintError,
                "clippy",
                file_path,
                message,
                remediation_hint,
            );
            finding.line_number = line_number;
            finding.column_number = column_number;
            finding.raw_context = trimmed.to_string();
            findings.push(finding);
        }
    }

    findings
}

/// Extracts Ruff linter findings from python output.
pub fn extract_ruff_findings(output: &str) -> Vec<DiagnosticFinding> {
    let mut findings = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        // Format: path/file.py:line:col: RULE message
        if let Some((loc, rest)) = trimmed.split_once(": ") {
            let loc_parts: Vec<&str> = loc.split(':').collect();
            if loc_parts.len() >= 3 {
                let file_path = loc_parts[0].to_string();
                let line_number = loc_parts[1].parse::<usize>().ok();
                let column_number = loc_parts[2].parse::<usize>().ok();
                let message = rest.to_string();
                let mut finding = DiagnosticFinding::new(
                    FindingType::LintError,
                    "ruff",
                    file_path,
                    message,
                    "Address Ruff violation jf. style guide.".to_string(),
                );
                finding.line_number = line_number;
                finding.column_number = column_number;
                finding.raw_context = trimmed.to_string();
                findings.push(finding);
            }
        }
    }
    findings
}

/// Extracts ESLint / Biome findings.
pub fn extract_eslint_findings(output: &str) -> Vec<DiagnosticFinding> {
    let mut findings = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if (trimmed.contains("error") || trimmed.contains("warning")) && trimmed.contains(':') {
            let parts: Vec<&str> = trimmed.split(':').collect();
            if parts.len() >= 3 {
                let file_path = parts[0].trim().to_string();
                let line_number = parts[1].trim().parse::<usize>().ok();
                let message = parts[2..].join(":").trim().to_string();
                let mut finding = DiagnosticFinding::new(
                    FindingType::LintError,
                    "eslint",
                    file_path,
                    message,
                    "Fix ESLint/Biome rule violation.".to_string(),
                );
                finding.line_number = line_number;
                finding.raw_context = trimmed.to_string();
                findings.push(finding);
            }
        }
    }
    findings
}
