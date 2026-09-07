//! Phase-specific pre-flight invariant verification engine.

use std::path::Path;

use super::models::{CheckpointError, CheckpointOptions, CheckpointPhase};
use crate::features::config::GauntletConfig;
use crate::features::evidence::{compute_workspace_manifest, verify_self_mutation};
use crate::features::gauntlet::{
    execute_gauntlet_pipeline, execute_layer, GauntletOptions, LayerDefinition, LayerRequirement,
};
use crate::features::tasks::{check_task_specification, resolve_active_task_id};

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

/// Executes pre-flight invariant verification for a designated TDD phase.
pub async fn run_preflight_check(options: &CheckpointOptions) -> Result<(), CheckpointError> {
    let task_id =
        resolve_active_task_id(&options.workspace).ok_or(CheckpointError::NoActiveTask)?;

    match options.phase {
        CheckpointPhase::Spec => {
            let tasks_dir = options.workspace.join("tasks");
            let task_path = tasks_dir.join(format!("{task_id}.md"));
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
