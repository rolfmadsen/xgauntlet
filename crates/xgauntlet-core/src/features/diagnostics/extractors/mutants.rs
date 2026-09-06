//! Extractors for mutation testing outputs (Cargo mutants, Mutants.py, Stryker).

use crate::features::diagnostics::models::{DiagnosticFinding, FindingType};

/// Extracts Cargo mutants surviving mutant findings.
pub fn extract_cargo_mutants_findings(output: &str) -> Vec<DiagnosticFinding> {
    let mut findings = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        // Format: MISSED src/features/policy/mod.rs:12: replace != with ==
        if trimmed.starts_with("MISSED ") || trimmed.starts_with("SURVIVED ") {
            let rest = if trimmed.starts_with("MISSED ") {
                trimmed.trim_start_matches("MISSED ").trim()
            } else {
                trimmed.trim_start_matches("SURVIVED ").trim()
            };

            let mut file_path = "src/lib.rs".to_string();
            let mut line_number = None;
            let mut message = rest.to_string();

            if let Some((loc, mutant_desc)) = rest.split_once(':') {
                file_path = loc.to_string();
                if let Some((l_str, desc)) = mutant_desc.split_once(':') {
                    line_number = l_str.trim().parse::<usize>().ok();
                    message = format!("Mutant survived: {}", desc.trim());
                } else {
                    message = format!("Mutant survived: {}", mutant_desc.trim());
                }
            }

            let mut finding = DiagnosticFinding::new(
                FindingType::MutantSurvived,
                "cargo-mutants",
                file_path,
                message,
                "Add test assertion killing this surviving mutation.".to_string(),
            );
            finding.line_number = line_number;
            finding.raw_context = trimmed.to_string();
            findings.push(finding);
        }
    }
    findings
}

/// Extracts Mutants.py findings.
pub fn extract_mutants_py_findings(output: &str) -> Vec<DiagnosticFinding> {
    let mut findings = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.contains("survived") || trimmed.contains("SURVIVED") {
            findings.push(DiagnosticFinding::new(
                FindingType::MutantSurvived,
                "mutants.py",
                "src",
                trimmed.to_string(),
                "Strengthen unit test assertions to kill surviving mutants.".to_string(),
            ));
        }
    }
    findings
}
