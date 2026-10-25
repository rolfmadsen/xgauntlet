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
            let layer = resolve_test_layer(&options.workspace);
            let res = execute_layer(&layer, &options.workspace).await;
            if res.passed {
                return Err(CheckpointError::RedPhaseUnmet {
                    details: "Expected failing test assertions in RED phase, but tests passed"
                        .to_string(),
                });
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

            let layer = resolve_test_layer(&options.workspace);
            let res = execute_layer(&layer, &options.workspace).await;
            if !res.passed {
                return Err(CheckpointError::GreenPhaseUnmet {
                    details: format!(
                        "Expected passing tests in GREEN phase, but tests failed: {}",
                        res.output.trim()
                    ),
                });
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

            let layer = resolve_test_layer(&options.workspace);
            let pre_manifest = compute_workspace_manifest(&options.workspace, None).ok();
            let res = execute_layer(&layer, &options.workspace).await;
            if !res.passed {
                return Err(CheckpointError::RefactorPhaseUnmet {
                    details: format!(
                        "Expected passing tests in REFACTOR phase, but tests failed: {}",
                        res.output.trim()
                    ),
                });
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
