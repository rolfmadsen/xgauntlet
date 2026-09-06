//! Release Readiness Gatekeeper orchestration engine.

use std::collections::BTreeSet;
use std::time::Instant;

use super::adr::check_adr_coverage;
use super::changelog::validate_changelog;
use super::manifests::extract_declared_versions;
use super::models::{
    ReleaseError, ReleaseFinding, ReleaseFindingCategory, ReleaseReadinessOptions,
    ReleaseReadinessReport,
};

/// Orchestrator for evaluating release readiness and manifest harmony across the repository.
pub struct ReleaseReadinessEngine;

impl ReleaseReadinessEngine {
    /// Evaluates the workspace against release invariants.
    pub fn evaluate(
        options: &ReleaseReadinessOptions,
    ) -> Result<ReleaseReadinessReport, ReleaseError> {
        let start = Instant::now();
        let ws = &options.workspace;

        if !ws.is_dir() {
            return Err(ReleaseError::InvalidWorkspace(ws.display().to_string()));
        }

        let mut diagnostics = Vec::new();
        let mut inspected_files = Vec::new();

        // 1. Extract declared versions from manifests
        let manifest_summary = extract_declared_versions(ws)?;
        inspected_files.extend(manifest_summary.inspected_files);

        let unique_versions: BTreeSet<&String> =
            manifest_summary.versions_by_source.values().collect();

        let declared_version = if unique_versions.is_empty() {
            diagnostics.push(ReleaseFinding {
                file_path: "Cargo.toml".to_string(),
                category: ReleaseFindingCategory::MissingManifest,
                message: "No version declaration found across supported project manifests (Cargo.toml, package.json, pyproject.toml).".to_string(),
                remediation_hint: "Declare a version string in Cargo.toml or package.json.".to_string(),
            });
            "0.0.0".to_string()
        } else if unique_versions.len() > 1 {
            let mismatch_details = manifest_summary
                .versions_by_source
                .iter()
                .map(|(k, v)| format!("{k}: {v}"))
                .collect::<Vec<_>>()
                .join(", ");

            let primary_version = unique_versions
                .iter()
                .next()
                .map(|s| (*s).clone())
                .unwrap_or_else(|| "0.0.0".to_string());

            diagnostics.push(ReleaseFinding {
                file_path: inspected_files
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "Cargo.toml".to_string()),
                category: ReleaseFindingCategory::ConfigVersionMismatch,
                message: format!(
                    "Version mismatch detected across project manifests: {mismatch_details}."
                ),
                remediation_hint: "Synchronize version numbers across all package manifest files."
                    .to_string(),
            });

            primary_version
        } else {
            unique_versions
                .iter()
                .next()
                .map(|s| (*s).clone())
                .unwrap_or_else(|| "0.0.0".to_string())
        };

        // 2. Validate CHANGELOG.md
        let (cl_diagnostics, changelog_versions, _) =
            validate_changelog(ws, &declared_version, options.allow_unreleased)?;
        if ws.join("CHANGELOG.md").is_file() {
            inspected_files.push("CHANGELOG.md".to_string());
        }
        diagnostics.extend(cl_diagnostics);

        // 3. Validate Architecture Decision Record coverage
        let adr_summary = check_adr_coverage(ws)?;
        inspected_files.extend(adr_summary.inspected_adrs);
        diagnostics.extend(adr_summary.diagnostics);

        // 4. Overall readiness
        let is_ready = diagnostics.is_empty();
        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(ReleaseReadinessReport {
            is_ready,
            declared_version,
            versions_by_source: manifest_summary.versions_by_source,
            inspected_files,
            changelog_versions,
            unreferenced_adrs: adr_summary.unreferenced_adrs,
            diagnostics,
            duration_ms,
        })
    }
}

/// Convenience functional entrypoint evaluating release readiness.
pub fn check_release_readiness(
    options: &ReleaseReadinessOptions,
) -> Result<ReleaseReadinessReport, ReleaseError> {
    ReleaseReadinessEngine::evaluate(options)
}
