//! Architecture Decision Record (ADR) documentation coverage verification.

use std::fs;
use std::path::Path;

use super::models::{ReleaseError, ReleaseFinding, ReleaseFindingCategory};

/// Result of evaluating ADR coverage against README.md and spec.md.
#[derive(Debug, Clone)]
pub struct AdrCoverageSummary {
    /// Findings for unreferenced Architecture Decision Records.
    pub diagnostics: Vec<ReleaseFinding>,
    /// Relative paths of all inspected ADR files.
    pub inspected_adrs: Vec<String>,
    /// Relative paths of all unreferenced ADR files.
    pub unreferenced_adrs: Vec<String>,
}

/// Verifies that all Architecture Decision Records in docs/adr/ are referenced in README.md or spec.md.
pub fn check_adr_coverage(workspace: &Path) -> Result<AdrCoverageSummary, ReleaseError> {
    let mut diagnostics = Vec::new();
    let mut inspected_adrs = Vec::new();
    let mut unreferenced_adrs = Vec::new();

    let adr_dir = workspace.join("docs").join("adr");
    if !adr_dir.is_dir() {
        return Ok(AdrCoverageSummary {
            diagnostics,
            inspected_adrs,
            unreferenced_adrs,
        });
    }

    let readme_path = workspace.join("README.md");
    let spec_path = workspace.join("spec.md");

    let readme_content = if readme_path.is_file() {
        fs::read_to_string(&readme_path)?
    } else {
        String::new()
    };

    let spec_content = if spec_path.is_file() {
        fs::read_to_string(&spec_path)?
    } else {
        String::new()
    };

    let combined_doc = format!("{readme_content}\n{spec_content}");

    let mut entries: Vec<_> = fs::read_dir(&adr_dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_file() && p.extension().is_some_and(|ext| ext == "md"))
        .collect();
    entries.sort();

    for adr_path in entries {
        let file_name = match adr_path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name,
            None => continue,
        };

        let file_name_lower = file_name.to_ascii_lowercase();
        if file_name_lower == "readme.md" || file_name_lower == "index.md" {
            continue;
        }

        let rel_adr = adr_path
            .strip_prefix(workspace)
            .unwrap_or(&adr_path)
            .to_string_lossy()
            .replace('\\', "/");
        inspected_adrs.push(rel_adr.clone());

        let stem = adr_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(file_name);

        // Extract leading numeric prefix if present (e.g. "0001" from "0001-package-by-feature.md")
        let leading_digits: String = file_name
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect();

        let is_referenced = combined_doc.contains(file_name)
            || combined_doc.contains(&rel_adr)
            || combined_doc.contains(stem)
            || (!leading_digits.is_empty() && {
                let num_padded = &leading_digits;
                let num_unpadded = leading_digits.trim_start_matches('0');
                let unpadded_str = if num_unpadded.is_empty() {
                    "0"
                } else {
                    num_unpadded
                };

                combined_doc.contains(&format!("ADR {num_padded}"))
                    || combined_doc.contains(&format!("ADR {unpadded_str}"))
                    || combined_doc.contains(&format!("ADR-{num_padded}"))
                    || combined_doc.contains(&format!("ADR-{unpadded_str}"))
            });

        if !is_referenced {
            unreferenced_adrs.push(rel_adr.clone());
            diagnostics.push(ReleaseFinding {
                file_path: rel_adr.clone(),
                category: ReleaseFindingCategory::UnreferencedAdr,
                message: format!(
                    "Architecture Decision Record '{file_name}' is not referenced in README.md or spec.md."
                ),
                remediation_hint: format!(
                    "Add a link to [{file_name}]({rel_adr}) in README.md or spec.md."
                ),
            });
        }
    }

    Ok(AdrCoverageSummary {
        diagnostics,
        inspected_adrs,
        unreferenced_adrs,
    })
}
