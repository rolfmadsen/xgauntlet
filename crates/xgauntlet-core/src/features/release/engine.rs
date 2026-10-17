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

        // 4. Validate code formatting for all configured, mandatory format layers
        let gauntlet_toml = ws.join("gauntlet.toml");
        let gauntlet_json = ws.join("gauntlet.json");
        if gauntlet_toml.is_file() || gauntlet_json.is_file() {
            match crate::features::config::GauntletConfig::load(ws) {
                Ok(config) => {
                    let format_layers: Vec<_> = config
                        .layers
                        .iter()
                        .filter(|l| l.name == "format" && !l.optional)
                        .collect();

                    for fl in format_layers {
                        let layer_def = fl.to_layer_definition();
                        let result = block_on(crate::features::gauntlet::runner::execute_layer(
                            &layer_def, ws,
                        ));
                        if !result.passed {
                            let details = result.output.trim();
                            let details_snippet = if details.is_empty() {
                                format!("exit code {}", result.exit_code)
                            } else {
                                details.lines().take(3).collect::<Vec<_>>().join(" ")
                            };
                            let config_file = if gauntlet_toml.is_file() {
                                "gauntlet.toml"
                            } else {
                                "gauntlet.json"
                            };
                            diagnostics.push(ReleaseFinding {
                                file_path: config_file.to_string(),
                                category: ReleaseFindingCategory::UnformattedCode,
                                message: format!(
                                    "Code formatting check failed ({}): {details_snippet}",
                                    layer_def.command.join(" ")
                                ),
                                remediation_hint:
                                    "Run code formatter before releasing (e.g. cargo fmt / ruff format / prettier)."
                                        .to_string(),
                            });
                        }
                    }
                }
                Err(err) => {
                    let config_file = if gauntlet_toml.is_file() {
                        "gauntlet.toml"
                    } else {
                        "gauntlet.json"
                    };
                    diagnostics.push(ReleaseFinding {
                        file_path: config_file.to_string(),
                        category: ReleaseFindingCategory::UnformattedCode,
                        message: format!("Failed to load gauntlet configuration: {err}"),
                        remediation_hint:
                            "Fix configuration syntax in gauntlet.toml or gauntlet.json."
                                .to_string(),
                    });
                }
            }
        }

        // 5. Validate WASM Policy Engine bytecode version matches declared release version
        let policy_crate = ws.join("crates/gauntlet-policy-engine");
        if policy_crate.is_dir() {
            let wasm_path = ws.join("crates/xgauntlet-core/wasm/gauntlet_policy.wasm");
            if wasm_path.is_file() {
                inspected_files.push("crates/xgauntlet-core/wasm/gauntlet_policy.wasm".to_string());
                match std::fs::read(&wasm_path) {
                    Ok(bytes) => {
                        match crate::features::policy::WasmPolicyEngine::from_wasm_bytes(&bytes) {
                            Ok(mut engine) => match engine.get_policy_version() {
                                Ok(wasm_ver) => {
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
                                Err(err) => {
                                    diagnostics.push(ReleaseFinding {
                                        file_path: "crates/xgauntlet-core/wasm/gauntlet_policy.wasm"
                                            .to_string(),
                                        category: ReleaseFindingCategory::GeneralError,
                                        message: format!(
                                            "Failed to query get_policy_version from WebAssembly policy bytecode: {err}"
                                        ),
                                        remediation_hint: "Recompile gauntlet-policy-engine to wasm32-unknown-unknown and synchronize.".to_string(),
                                    });
                                }
                            },
                            Err(err) => {
                                diagnostics.push(ReleaseFinding {
                                    file_path: "crates/xgauntlet-core/wasm/gauntlet_policy.wasm"
                                        .to_string(),
                                    category: ReleaseFindingCategory::GeneralError,
                                    message: format!(
                                        "Embedded WebAssembly policy bytecode is invalid or corrupt: {err}"
                                    ),
                                    remediation_hint: "Rebuild gauntlet-policy-engine: cargo build -p gauntlet-policy-engine --target wasm32-unknown-unknown --release && cp target/wasm32-unknown-unknown/release/gauntlet_policy_engine.wasm crates/xgauntlet-core/wasm/gauntlet_policy.wasm".to_string(),
                                });
                            }
                        }
                    }
                    Err(err) => {
                        diagnostics.push(ReleaseFinding {
                            file_path: "crates/xgauntlet-core/wasm/gauntlet_policy.wasm".to_string(),
                            category: ReleaseFindingCategory::GeneralError,
                            message: format!(
                                "Failed to read WebAssembly policy bytecode: {err}"
                            ),
                            remediation_hint: "Ensure crates/xgauntlet-core/wasm/gauntlet_policy.wasm is readable.".to_string(),
                        });
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

        // 6. Strict mode validations: Clean worktree & sealed verification-report.json with 0 drift
        if options.strict {
            if ws.join(".git").exists() {
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

            let report_path = ws.join("verification-report.json");
            if !report_path.is_file() {
                diagnostics.push(ReleaseFinding {
                    file_path: "verification-report.json".to_string(),
                    category: ReleaseFindingCategory::MissingManifest,
                    message: "Required verification report (verification-report.json) is missing from workspace root.".to_string(),
                    remediation_hint: "Run 'xgauntlet verify --save' and commit verification-report.json before releasing.".to_string(),
                });
            } else {
                inspected_files.push("verification-report.json".to_string());
                match crate::features::evidence::load_verification_report(ws) {
                    Err(err) => {
                        diagnostics.push(ReleaseFinding {
                            file_path: "verification-report.json".to_string(),
                            category: ReleaseFindingCategory::GeneralError,
                            message: format!("Failed to parse verification-report.json schema: {err}"),
                            remediation_hint: "Re-generate verification-report.json via 'xgauntlet verify --save'.".to_string(),
                        });
                    }
                    Ok(vr) => {
                        let verdict = vr.verdict.to_uppercase();
                        let verdict_approved = if verdict == "PASSED" {
                            true
                        } else if verdict == "PARTIAL" {
                            vr.checks.iter().all(|c| {
                                let is_required = !c.optional;
                                if is_required {
                                    c.passed && c.exit_code == 0
                                } else {
                                    true
                                }
                            })
                        } else {
                            false
                        };

                        if !verdict_approved {
                            diagnostics.push(ReleaseFinding {
                                file_path: "verification-report.json".to_string(),
                                category: ReleaseFindingCategory::GeneralError,
                                message: format!("Verification report verdict '{verdict}' is not approved for release."),
                                remediation_hint: "Run 'xgauntlet verify --save' to ensure all required layers pass with exit 0.".to_string(),
                            });
                        }

                        let canonical_manifest =
                            match crate::features::evidence::manifest::compute_workspace_manifest(
                                ws, None,
                            ) {
                                Ok(m) => m,
                                Err(e) => {
                                    diagnostics.push(ReleaseFinding {
                                        file_path: "verification-report.json".to_string(),
                                        category: ReleaseFindingCategory::GeneralError,
                                        message: format!(
                                            "Failed to build canonical workspace manifest: {e}"
                                        ),
                                        remediation_hint: "Ensure workspace files are readable."
                                            .to_string(),
                                    });
                                    return Ok(ReleaseReadinessReport {
                                        is_ready: false,
                                        declared_version,
                                        versions_by_source: manifest_summary.versions_by_source,
                                        inspected_files,
                                        changelog_versions,
                                        unreferenced_adrs: adr_summary.unreferenced_adrs,
                                        diagnostics,
                                        duration_ms: start.elapsed().as_millis() as u64,
                                    });
                                }
                            };

                        if let Err(drift_violation) =
                            crate::features::evidence::drift::verify_workspace_state_match(
                                &vr,
                                &canonical_manifest,
                            )
                        {
                            diagnostics.push(ReleaseFinding {
                                file_path: "verification-report.json".to_string(),
                                category: ReleaseFindingCategory::GeneralError,
                                message: format!("Workspace drift detected against committed verification-report.json: {drift_violation}"),
                                remediation_hint: "Re-run 'xgauntlet verify --save' to seal fresh evidence without workspace drift.".to_string(),
                            });
                        }
                    }
                }
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

fn block_on<F: std::future::Future>(f: F) -> F::Output {
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        tokio::task::block_in_place(|| handle.block_on(f))
    } else {
        tokio::runtime::Runtime::new()
            .expect("failed to create tokio runtime")
            .block_on(f)
    }
}
