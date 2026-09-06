//! Extractors for property-based and invariant testing outputs (Proptest, Hypothesis).

use crate::features::diagnostics::models::{DiagnosticFinding, FindingType};

/// Extracts Proptest failure findings.
pub fn extract_proptest_findings(output: &str) -> Vec<DiagnosticFinding> {
    let mut findings = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.contains("proptest:") || trimmed.contains("Test failed:") {
            findings.push(DiagnosticFinding::new(
                FindingType::PropertyViolation,
                "proptest",
                "tests",
                trimmed.to_string(),
                "Fix invariant or boundary case discovered by property testing.".to_string(),
            ));
        }
    }
    findings
}

/// Extracts Hypothesis findings.
pub fn extract_hypothesis_findings(output: &str) -> Vec<DiagnosticFinding> {
    let mut findings = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.contains("Falsifying example:") {
            findings.push(DiagnosticFinding::new(
                FindingType::PropertyViolation,
                "hypothesis",
                "tests",
                trimmed.to_string(),
                "Fix state invariant broken by falsifying example.".to_string(),
            ));
        }
    }
    findings
}
