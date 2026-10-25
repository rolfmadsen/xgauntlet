//! Domain models for the Architecture Quality Gate.

pub use crate::features::config::models::{ArchitectureConfig, ArchitectureExemption};

/// Category classification of an architectural defect or boundary violation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ArchitectureFindingCategory {
    /// Production source file exceeds `max_file_lines`.
    FileExceedsMaxLines,
    /// Test file exceeds `max_test_file_lines`.
    TestFileExceedsMaxLines,
    /// An exempted file has exceeded its declared `max_lines` ceiling.
    ExemptionCeilingExceeded,
    /// A forbidden token (e.g. raw RGB constructor bypassing design tokens) was detected.
    ForbiddenTokenDetected,
    /// A forbidden cross-boundary module import was detected.
    ForbiddenImportDetected,
    /// An architectural exemption is missing the mandatory `adr` attribution.
    ExemptionMissingAdr,
    /// An architectural exemption references an ADR file that does not exist.
    ExemptionAdrNotFound,
    /// An architectural exemption references an ADR that is not in `accepted` or `stable` status.
    ExemptionAdrNotAccepted,
}

/// An individual finding produced during architectural analysis.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ArchitectureFinding {
    pub file_path: String,
    pub line_number: Option<usize>,
    pub category: ArchitectureFindingCategory,
    pub message: String,
    pub remediation_hint: String,
}

/// Summary report of an architectural validation pass.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ArchitectureReport {
    pub is_valid: bool,
    pub files_scanned: usize,
    pub findings: Vec<ArchitectureFinding>,
}

impl ArchitectureReport {
    pub fn new() -> Self {
        Self {
            is_valid: true,
            files_scanned: 0,
            findings: Vec::new(),
        }
    }

    pub fn add_finding(&mut self, finding: ArchitectureFinding) {
        self.is_valid = false;
        self.findings.push(finding);
    }
}

impl Default for ArchitectureReport {
    fn default() -> Self {
        Self::new()
    }
}
