//! Parsing and validation of Keep a Changelog specifications.

use std::fs;
use std::path::Path;

use super::models::{ReleaseError, ReleaseFinding, ReleaseFindingCategory};

/// Parses version header strings from a Keep a Changelog markdown file.
///
/// Recognizes headers such as:
/// - `## [1.2.3] - 2026-09-06`
/// - `## [1.2.3]`
/// - `## [Unreleased]`
/// - `## [v1.2.3] - 2026-09-06`
/// - `## 1.2.3`
pub fn parse_changelog_versions(changelog_path: &Path) -> Result<Vec<String>, ReleaseError> {
    if !changelog_path.is_file() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(changelog_path)?;
    let mut versions = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("## ") {
            let header = rest.trim();
            if let Some(start_bracket) = header.find('[') {
                if let Some(end_bracket) = header[start_bracket + 1..].find(']') {
                    let ver = &header[start_bracket + 1..start_bracket + 1 + end_bracket];
                    let cleaned = ver.trim().to_string();
                    if !cleaned.is_empty() && !versions.contains(&cleaned) {
                        versions.push(cleaned);
                    }
                    continue;
                }
            }

            // Fallback for headers without brackets like `## 1.2.3 - 2026-09-06`
            let token = header.split([' ', '-']).next().unwrap_or("").trim();
            if !token.is_empty()
                && (token.chars().next().is_some_and(|c| c.is_ascii_digit())
                    || token.starts_with('v'))
            {
                let cleaned = token.to_string();
                if !versions.contains(&cleaned) {
                    versions.push(cleaned);
                }
            }
        }
    }

    Ok(versions)
}

/// Validates that CHANGELOG.md exists and contains the declared release version.
pub fn validate_changelog(
    workspace: &Path,
    declared_version: &str,
    allow_unreleased: bool,
) -> Result<(Vec<ReleaseFinding>, Vec<String>, bool), ReleaseError> {
    let changelog_path = workspace.join("CHANGELOG.md");
    let mut findings = Vec::new();

    if !changelog_path.is_file() {
        findings.push(ReleaseFinding {
            file_path: "CHANGELOG.md".to_string(),
            category: ReleaseFindingCategory::MissingChangelog,
            message: "Missing 'CHANGELOG.md' in workspace root.".to_string(),
            remediation_hint: "Create a CHANGELOG.md following the Keep a Changelog specification."
                .to_string(),
        });
        return Ok((findings, Vec::new(), false));
    }

    let versions = parse_changelog_versions(&changelog_path)?;

    let clean_declared = declared_version.trim().trim_start_matches('v');
    let has_matching_version = versions.iter().any(|v| {
        let clean_v = v.trim().trim_start_matches('v');
        clean_v == clean_declared
    });

    let has_unreleased = versions
        .iter()
        .any(|v| v.trim().eq_ignore_ascii_case("unreleased"));

    if has_matching_version {
        return Ok((findings, versions, true));
    }

    if allow_unreleased && has_unreleased {
        // Permitted under advisory or development pre-release mode
        return Ok((findings, versions, true));
    }

    findings.push(ReleaseFinding {
        file_path: "CHANGELOG.md".to_string(),
        category: ReleaseFindingCategory::ChangelogVersionMismatch,
        message: format!(
            "CHANGELOG.md does not contain an entry for declared release version '{declared_version}'. Found versions: {versions:?}."
        ),
        remediation_hint: format!(
            "Add a '## [{declared_version}] - YYYY-MM-DD' section to CHANGELOG.md with release notes before publishing."
        ),
    });

    Ok((findings, versions, false))
}
