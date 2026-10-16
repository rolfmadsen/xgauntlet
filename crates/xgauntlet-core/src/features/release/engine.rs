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

        // 4. Validate code formatting if gauntlet.toml is present and defines a format layer
        let gauntlet_toml = ws.join("gauntlet.toml");
        if gauntlet_toml.is_file() {
            if let Ok(config) = crate::features::config::GauntletConfig::load(ws) {
                if let Some(format_layer) = config.layers.iter().find(|l| l.name == "format") {
                    if !format_layer.command.is_empty() {
                        let prog = &format_layer.command[0];
                        let args = &format_layer.command[1..];
                        if let Ok(output) = std::process::Command::new(prog)
                            .args(args)
                            .current_dir(ws)
                            .output()
                        {
                            if !output.status.success() {
                                let stderr = String::from_utf8_lossy(&output.stderr);
                                let stdout = String::from_utf8_lossy(&output.stdout);
                                let details = if !stderr.trim().is_empty() {
                                    stderr.trim().to_string()
                                } else {
                                    stdout.lines().take(3).collect::<Vec<_>>().join(" ")
                                };
                                diagnostics.push(ReleaseFinding {
                                    file_path: "gauntlet.toml".to_string(),
                                    category: ReleaseFindingCategory::UnformattedCode,
                                    message: format!(
                                        "Code formatting check failed ({prog} {}): {details}",
                                        args.join(" ")
                                    ),
                                    remediation_hint:
                                        "Run code formatter before releasing (e.g. cargo fmt / ruff format / prettier)."
                                            .to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }

        // 5. Validate WASM Policy Engine bytecode version matches declared release version
        let policy_crate = ws.join("crates/gauntlet-policy-engine");
        if policy_crate.is_dir() {
            let wasm_path = ws.join("crates/xgauntlet-core/wasm/gauntlet_policy.wasm");
            if wasm_path.is_file() {
                inspected_files.push("crates/xgauntlet-core/wasm/gauntlet_policy.wasm".to_string());
                if let Ok(bytes) = std::fs::read(&wasm_path) {
                    if let Ok(mut engine) =
                        crate::features::policy::WasmPolicyEngine::from_wasm_bytes(&bytes)
                    {
                        if let Ok(wasm_ver) = engine.get_policy_version() {
                            if wasm_ver != declared_version {
                                diagnostics.push(ReleaseFinding {
                                    file_path: "crates/xgauntlet-core/wasm/gauntlet_policy.wasm"
                                        .to_string(),
                                    category: ReleaseFindingCategory::ConfigVersionMismatch,
                                    message: format!(
                                        "Embedded WebAssembly policy bytecode version ({wasm_ver}) does not match workspace declared release version ({declared_version})."
                                    ),
                                    remediation_hint: "Rebuild wasm policy engine and synchronize: cargo build -p gauntlet-policy-engine --target wasm32-unknown-unknown --release && cp target/wasm32-unknown-unknown/release/gauntlet_policy_engine.wasm crates/xgauntlet-core/wasm/gauntlet_policy.wasm".to_string(),
                                });
                            }
                        }
                    }
                }
            } else {
                diagnostics.push(ReleaseFinding {
                    file_path: "crates/xgauntlet-core/wasm/gauntlet_policy.wasm".to_string(),
                    category: ReleaseFindingCategory::MissingManifest,
                    message: "Required WebAssembly policy bytecode binary (gauntlet_policy.wasm) is missing from crates/xgauntlet-core/wasm/.".to_string(),
                    remediation_hint: "Compile gauntlet-policy-engine to wasm32-unknown-unknown and place in crates/xgauntlet-core/wasm/gauntlet_policy.wasm".to_string(),
                });
            }
        }

        // 6. Clean worktree check under strict mode
        if options.strict && ws.join(".git").exists() {
            if let Ok(false) = crate::features::checkpoint::git::is_worktree_clean(ws) {
                diagnostics.push(ReleaseFinding {
                    file_path: ".git".to_string(),
                    category: ReleaseFindingCategory::DirtyWorktree,
                    message: "Working tree contains uncommitted changes or unstaged files."
                        .to_string(),
                    remediation_hint:
                        "Commit or stash all changes before evaluating release readiness or tagging."
                            .to_string(),
                });
            }
        }

        // 7. Overall readiness
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
