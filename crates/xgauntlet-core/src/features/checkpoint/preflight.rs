//! Phase-specific pre-flight invariant verification engine.

use std::path::Path;

use super::models::{CheckpointError, CheckpointOptions, CheckpointPhase};
use crate::features::config::GauntletConfig;
use crate::features::evidence::{compute_workspace_manifest, verify_self_mutation};
use crate::features::gauntlet::{
    execute_gauntlet_pipeline, execute_layer, GauntletOptions, LayerDefinition, LayerRequirement,
};
use crate::features::tasks::check_task_specification;

/// Resolves the primary test layer from workspace configuration.
pub fn resolve_test_layer(workspace: &Path) -> LayerDefinition {
    if let Ok(config) = GauntletConfig::load(workspace) {
        let layers = config.to_layer_definitions();
        if let Some(layer) = layers
            .iter()
            .find(|l| l.name == "unit" || l.name.contains("test"))
        {
            return layer.clone();
        }
        if let Some(layer) = layers
            .iter()
            .find(|l| l.requirement == LayerRequirement::Required)
        {
            return layer.clone();
        }
        if let Some(first) = layers.first() {
            return first.clone();
        }
    }

    LayerDefinition::new("test", vec!["cargo".to_string(), "test".to_string()])
}

/// Target resolution for scoped Rust test execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RustTestTarget {
    IntegrationTest {
        target_name: String,
        filter: Option<String>,
        command: Vec<String>,
    },
    HarnessSubmodule {
        harness_name: String,
        submodule: String,
        command: Vec<String>,
    },
    LibModule {
        module_path: String,
        command: Vec<String>,
    },
    Custom {
        command: Vec<String>,
    },
}

impl RustTestTarget {
    pub fn command(&self) -> &[String] {
        match self {
            RustTestTarget::IntegrationTest { command, .. } => command,
            RustTestTarget::HarnessSubmodule { command, .. } => command,
            RustTestTarget::LibModule { command, .. } => command,
            RustTestTarget::Custom { command } => command,
        }
    }
}

/// Resolves targeted test commands from declared test_refs in task specifications.
pub fn resolve_scoped_test_command(workspace: &Path, test_ref: &str) -> RustTestTarget {
    let clean = test_ref.replace('\\', "/");
    let clean_ref = clean.trim_start_matches("./");

    // Case C: Library module in src/
    if let Some(after_src) = clean_ref
        .strip_prefix("src/")
        .or_else(|| clean_ref.find("/src/").map(|idx| &clean_ref[idx + 5..]))
    {
        let stripped = after_src.strip_suffix(".rs").unwrap_or(after_src);
        let mut module_path = stripped.replace('/', "::");
        if module_path.ends_with("::mod") {
            module_path = module_path
                .strip_suffix("::mod")
                .unwrap_or(&module_path)
                .to_string();
        }
        let command = vec![
            "cargo".to_string(),
            "test".to_string(),
            "--lib".to_string(),
            module_path.clone(),
        ];
        return RustTestTarget::LibModule {
            module_path,
            command,
        };
    }

    // Check for tests directory
    let tests_part = if let Some(idx) = clean_ref.find("/tests/") {
        &clean_ref[idx + 7..]
    } else if let Some(after) = clean_ref.strip_prefix("tests/") {
        after
    } else {
        clean_ref
    };

    let base_tests_dir = if let Some(idx) = clean_ref.find("/tests/") {
        workspace.join(&clean_ref[..idx + 6])
    } else {
        workspace.join("tests")
    };

    let parts: Vec<&str> = tests_part.split('/').collect();
    if parts.len() >= 2 {
        let harness_candidate = parts[0];
        let harness_file = base_tests_dir.join(format!("{harness_candidate}.rs"));
        let harness_alt = base_tests_dir.join(format!("{harness_candidate}_acceptance.rs"));
        if harness_file.is_file() || harness_alt.is_file() {
            let actual_harness = if harness_file.is_file() {
                harness_candidate.to_string()
            } else {
                format!("{harness_candidate}_acceptance")
            };
            let submod_filename = parts.last().unwrap();
            let submod = submod_filename
                .strip_suffix(".rs")
                .unwrap_or(submod_filename)
                .to_string();
            let command = vec![
                "cargo".to_string(),
                "test".to_string(),
                "--test".to_string(),
                actual_harness.clone(),
                submod.clone(),
            ];
            return RustTestTarget::HarnessSubmodule {
                harness_name: actual_harness,
                submodule: submod,
                command,
            };
        }
    }

    // Case A: Standalone integration test
    let file_name = parts.last().unwrap_or(&"");
    let target_name = file_name
        .strip_suffix(".rs")
        .unwrap_or(file_name)
        .to_string();
    let command = vec![
        "cargo".to_string(),
        "test".to_string(),
        "--test".to_string(),
        target_name.clone(),
    ];
    RustTestTarget::IntegrationTest {
        target_name,
        filter: None,
        command,
    }
}

fn should_enforce_provenance(options: &CheckpointOptions) -> bool {
    if options.allow_born_green {
        return false;
    }
    if options.strict_provenance {
        return true;
    }
    GauntletConfig::load(&options.workspace)
        .map(|c| c.checkpoint.enforce_provenance)
        .unwrap_or(true)
}

fn resolve_preflight_test_layers(workspace: &Path, task_id: &str) -> Vec<LayerDefinition> {
    let base_layer = resolve_test_layer(workspace);
    let is_cargo_project = workspace.join("Cargo.toml").is_file()
        && base_layer.command.first().map(|c| c.as_str()) == Some("cargo");

    if !is_cargo_project {
        return vec![base_layer];
    }

    let task_info = super::engine::load_task_package_info(workspace, task_id);
    let test_refs = task_info
        .as_ref()
        .map(|t| t.test_refs.clone())
        .unwrap_or_default();

    let concrete_refs: Vec<String> = test_refs
        .into_iter()
        .filter(|r| !r.contains('*') && r.ends_with(".rs"))
        .collect();

    if !concrete_refs.is_empty() {
        concrete_refs
            .into_iter()
            .map(|tr| {
                let target = resolve_scoped_test_command(workspace, &tr);
                LayerDefinition::new(format!("scoped-test:{tr}"), target.command().to_vec())
            })
            .collect()
    } else {
        vec![base_layer]
    }
}

/// Executes pre-flight invariant verification for a designated TDD phase.
pub async fn run_preflight_check(options: &CheckpointOptions) -> Result<(), CheckpointError> {
    let task_id =
        super::engine::resolve_checkpoint_task_id(options).ok_or(CheckpointError::NoActiveTask)?;

    match options.phase {
        CheckpointPhase::Spec => {
            let tasks_dir = options.workspace.join("tasks");
            let mut task_path = tasks_dir.join(format!("{task_id}.md"));
            if !task_path.is_file() {
                if let Ok(entries) = std::fs::read_dir(&tasks_dir) {
                    for entry in entries.flatten() {
                        let p = entry.path();
                        if p.extension().is_some_and(|e| e == "md") {
                            let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                            if stem == task_id || stem.starts_with(&format!("{task_id}-")) {
                                task_path = p;
                                break;
                            }
                        }
                    }
                }
            }
            let report = check_task_specification(&task_path, &options.workspace);
            if !report.is_valid {
                let msg = report
                    .diagnostics
                    .iter()
                    .map(|d| d.message.as_str())
                    .collect::<Vec<_>>()
                    .join("; ");
                return Err(CheckpointError::SpecPhaseUnmet(msg));
            }
            Ok(())
        }
        CheckpointPhase::Red => {
            let layers = resolve_preflight_test_layers(&options.workspace, &task_id);
            for layer in layers {
                let res = execute_layer(&layer, &options.workspace).await;
                if res.passed {
                    return Err(CheckpointError::RedPhaseUnmet {
                        details: format!(
                            "Expected failing test assertions in RED phase for '{}', but tests passed",
                            layer.name
                        ),
                    });
                }
            }
            Ok(())
        }
        CheckpointPhase::Green => {
            // Epistemic test provenance check (Task 032)
            if should_enforce_provenance(options) {
                if let Ok(new_tests) =
                    super::provenance::detect_born_green_tests(&options.workspace, "HEAD")
                {
                    if !new_tests.is_empty() {
                        if options.strict_provenance {
                            return Err(CheckpointError::BornGreenTestDetected {
                                tests: new_tests,
                            });
                        }
                        let inv_evidences = super::provenance::verify_inverse_test_failure(
                            &options.workspace,
                            &new_tests,
                        )?;
                        if let Ok(mut report) =
                            crate::features::evidence::load_verification_report(&options.workspace)
                        {
                            let mut list = report.test_provenance.unwrap_or_default();
                            for ev in inv_evidences {
                                if !list.iter().any(|e| e.test_identifier == ev.test_identifier) {
                                    list.push(ev);
                                }
                            }
                            report.test_provenance = Some(list);
                            let _ = crate::features::evidence::save_verification_report(
                                &options.workspace,
                                &report,
                            );
                        }
                    }
                }
            }

            let layers = resolve_preflight_test_layers(&options.workspace, &task_id);
            for layer in layers {
                let res = execute_layer(&layer, &options.workspace).await;
                if !res.passed {
                    return Err(CheckpointError::GreenPhaseUnmet {
                        details: format!(
                            "Expected passing tests in GREEN phase for '{}', but tests failed: {}",
                            layer.name,
                            res.output.trim()
                        ),
                    });
                }
            }
            Ok(())
        }
        CheckpointPhase::Refactor => {
            // Refactor invariant: No new tests may be introduced during refactoring without provenance
            if should_enforce_provenance(options) {
                if let Ok(new_tests) =
                    super::provenance::detect_born_green_tests(&options.workspace, "HEAD")
                {
                    if !new_tests.is_empty() {
                        if options.strict_provenance {
                            return Err(CheckpointError::BornGreenTestDetected {
                                tests: new_tests,
                            });
                        }
                        let inv_evidences = super::provenance::verify_inverse_test_failure(
                            &options.workspace,
                            &new_tests,
                        )?;
                        if let Ok(mut report) =
                            crate::features::evidence::load_verification_report(&options.workspace)
                        {
                            let mut list = report.test_provenance.unwrap_or_default();
                            for ev in inv_evidences {
                                if !list.iter().any(|e| e.test_identifier == ev.test_identifier) {
                                    list.push(ev);
                                }
                            }
                            report.test_provenance = Some(list);
                            let _ = crate::features::evidence::save_verification_report(
                                &options.workspace,
                                &report,
                            );
                        }
                    }
                }
            }

            let layers = resolve_preflight_test_layers(&options.workspace, &task_id);
            let pre_manifest = compute_workspace_manifest(&options.workspace, None).ok();
            for layer in layers {
                let res = execute_layer(&layer, &options.workspace).await;
                if !res.passed {
                    return Err(CheckpointError::RefactorPhaseUnmet {
                        details: format!(
                            "Expected passing tests in REFACTOR phase for '{}', but tests failed: {}",
                            layer.name,
                            res.output.trim()
                        ),
                    });
                }
            }
            if let Some(pre) = pre_manifest {
                if let Ok(post) = compute_workspace_manifest(&options.workspace, None) {
                    if let Err(err) = verify_self_mutation(&pre, &post) {
                        return Err(CheckpointError::RefactorPhaseUnmet {
                            details: format!("Self-mutation detected in REFACTOR phase: {err}"),
                        });
                    }
                }
            }
            Ok(())
        }
        CheckpointPhase::Done => {
            let outcome = execute_gauntlet_pipeline(
                &options.workspace,
                &GauntletOptions {
                    task_id: Some(task_id),
                    save_evidence: true,
                    layers: None,
                },
            )
            .await
            .map_err(|e| CheckpointError::DonePhaseUnmet(format!("Pipeline error: {e}")))?;

            if !outcome.success || outcome.is_self_mutated {
                return Err(CheckpointError::DonePhaseUnmet(format!(
                    "Gauntlet verification pipeline failed: verdict={}",
                    outcome.report.verdict
                )));
            }
            Ok(())
        }
    }
}
