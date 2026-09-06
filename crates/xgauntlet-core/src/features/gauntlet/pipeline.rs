//! End-to-end gauntlet verification pipeline orchestrator.

use std::collections::BTreeMap;
use std::path::Path;
use thiserror::Error;

use crate::features::config::{ConfigError, GauntletConfig};
use crate::features::diagnostics::DiagnosticParser;
use crate::features::evidence::{
    compute_workspace_manifest, current_iso_utc, save_verification_report, verify_self_mutation,
    CheckSummary, ExecutionMetadata, ManifestError, ReportError, TaskContractSummary,
    VerificationReport, WorkspaceState,
};
use crate::features::gauntlet::models::{GauntletReport, LayerDefinition, LayerRequirement};
use crate::features::gauntlet::runner::run_gauntlet;
use crate::features::tasks::{resolve_task_contract, TaskContract, TaskError};

#[derive(Debug, Error)]
pub enum GauntletPipelineError {
    #[error("Manifest error: {0}")]
    Manifest(#[from] ManifestError),

    #[error("Task error: {0}")]
    Task(#[from] TaskError),

    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("Report error: {0}")]
    Report(#[from] ReportError),
}

/// Options controlling pipeline execution.
#[derive(Debug, Clone, Default)]
pub struct GauntletOptions {
    pub task_id: Option<String>,
    pub save_evidence: bool,
    pub layers: Option<Vec<LayerDefinition>>,
}

/// Consolidated outcome of running the gauntlet pipeline.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GauntletExecutionOutcome {
    pub success: bool,
    pub is_self_mutated: bool,
    pub report: VerificationReport,
    pub gauntlet_report: GauntletReport,
}

/// Executes the full multi-layer gauntlet verification pipeline.
pub async fn execute_gauntlet_pipeline(
    workspace: &Path,
    options: &GauntletOptions,
) -> Result<GauntletExecutionOutcome, GauntletPipelineError> {
    let started_iso = current_iso_utc();
    let start_instant = std::time::Instant::now();

    // 1. Compute Pre-Verification Workspace Manifest
    let manifest_pre = compute_workspace_manifest(workspace, None)?;

    // 2. Resolve Active Task Contract
    let task_contract = match resolve_task_contract(workspace, options.task_id.as_deref()) {
        Ok(tc) => tc,
        Err(_) => {
            let id = options
                .task_id
                .clone()
                .unwrap_or_else(|| "default-task".to_string());
            TaskContract {
                task_id: id,
                title: String::new(),
                acceptance_criteria: Vec::new(),
                unresolved_criteria: Vec::new(),
            }
        }
    };

    // 3. Resolve Verification Layers
    let layers = match &options.layers {
        Some(l) => l.clone(),
        None => {
            let config = GauntletConfig::load(workspace)?;
            config.to_layer_definitions()
        }
    };

    // 4. Run Sequential Gauntlet with Fail-Closed Semantics
    let gauntlet_report = run_gauntlet(&layers, workspace).await;

    // 5. Compute Post-Verification Workspace Manifest
    let manifest_post = compute_workspace_manifest(workspace, None)?;

    // 6. Enforce Self-Mutation Invariant
    let self_mutation_check = verify_self_mutation(&manifest_pre, &manifest_post);
    let is_self_mutated = self_mutation_check.is_err();

    // 7. Extract Diagnostic Findings
    let parser = DiagnosticParser::new();
    let mut all_findings = Vec::new();

    for layer_res in &gauntlet_report.layers {
        if !layer_res.passed {
            let cmd_slice: Vec<String> = layers
                .iter()
                .find(|l| l.name == layer_res.name)
                .map(|l| l.command.clone())
                .unwrap_or_else(|| vec![layer_res.name.clone()]);
            let layer_findings = parser.extract_findings(
                &layer_res.name,
                &cmd_slice,
                layer_res.exit_code,
                &layer_res.output,
            );
            all_findings.extend(layer_findings);
        }
    }

    if let Err(violation) = &self_mutation_check {
        all_findings.push(violation.to_diagnostic_finding());
    }

    // 8. Compute Check Summaries
    let checks: Vec<CheckSummary> = gauntlet_report
        .layers
        .iter()
        .map(|layer| {
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(layer.output.as_bytes());
            let log_digest = format!("{:x}", hasher.finalize());

            CheckSummary {
                name: layer.name.clone(),
                status: layer.status.as_str().to_string(),
                exit_code: layer.exit_code,
                duration_seconds: layer.duration_seconds,
                optional: layer.requirement == LayerRequirement::Optional,
                passed: layer.passed,
                log_digest,
            }
        })
        .collect();

    // 9. Compute Check Definitions Digest
    use sha2::{Digest, Sha256};
    let mut check_defs_hasher = Sha256::new();
    for layer in &layers {
        let cmd_str = layer.command.join(" ");
        check_defs_hasher
            .update(format!("{}:{}:{}\n", layer.name, cmd_str, layer.optional).as_bytes());
    }
    let check_definitions_digest = format!("{:x}", check_defs_hasher.finalize());

    // 10. Workspace State and Metadata
    let workspace_state = WorkspaceState {
        manifest_version: "1.0".to_string(),
        source_content_digest: manifest_post.source_content_digest.clone(),
        source_manifest_digest_pre: manifest_pre.source_manifest_digest.clone(),
        source_manifest_digest_post: manifest_post.source_manifest_digest.clone(),
        config_digest: manifest_post.config_digest.clone(),
        task_digest: manifest_post.task_digest.clone(),
        policy_digest: manifest_post.policy_digest.clone(),
        check_definitions_digest,
        included_files_count: manifest_post.included_files_count,
        vcs: manifest_post.vcs.clone(),
    };

    let finished_iso = current_iso_utc();
    let total_duration = start_instant.elapsed().as_secs_f64();
    let mut env_map = BTreeMap::new();
    env_map.insert("platform".to_string(), std::env::consts::OS.to_string());
    env_map.insert("arch".to_string(), std::env::consts::ARCH.to_string());
    env_map.insert("engine".to_string(), "xgauntlet-core".to_string());
    env_map.insert("version".to_string(), env!("CARGO_PKG_VERSION").to_string());

    let execution_metadata = ExecutionMetadata {
        started_at: started_iso,
        finished_at: finished_iso,
        total_duration_seconds: total_duration,
        environment: env_map,
    };

    // 11. Determine Final Verification Verdict
    let has_mandatory_failures = checks
        .iter()
        .any(|c| !c.optional && (!c.passed || c.exit_code != 0));
    let has_optional_failures = checks
        .iter()
        .any(|c| c.optional && (!c.passed || c.exit_code != 0));
    let has_unresolved_criteria = !task_contract.unresolved_criteria.is_empty();

    let verdict = if is_self_mutated
        || has_mandatory_failures
        || !gauntlet_report.success
        || checks.is_empty()
    {
        "FAILED"
    } else if has_unresolved_criteria {
        "INCOMPLETE"
    } else if has_optional_failures {
        "PARTIAL"
    } else {
        "PASSED"
    };

    let report = VerificationReport {
        schema: "https://agent-gauntlet.dev/schemas/v2/verification-report.json".to_string(),
        schema_version: "2.0.0".to_string(),
        execution_origin: "LOCAL".to_string(),
        verdict: verdict.to_string(),
        task_contract: TaskContractSummary {
            task_id: task_contract.task_id,
            task_title: task_contract.title,
            task_digest: manifest_post.task_digest.clone(),
            acceptance_criteria: task_contract.acceptance_criteria,
            unresolved_criteria: task_contract.unresolved_criteria,
        },
        workspace_state,
        execution_metadata,
        checks,
        diagnostics: all_findings,
    };

    // 12. Save Evidence Artifacts if requested
    if options.save_evidence {
        save_verification_report(workspace, &report)?;
    }

    Ok(GauntletExecutionOutcome {
        success: verdict == "PASSED" || verdict == "PARTIAL",
        is_self_mutated,
        report,
        gauntlet_report,
    })
}
