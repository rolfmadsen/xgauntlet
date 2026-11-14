//! Subcommand execution handlers extracted from `main.rs` to keep per-frame
//! stack usage small on Windows debug builds and avoid eager Tokio runtime init.

use std::path::Path;

use crate::render::{format_checkpoint_body, render_checkpoint_summary};

/// Runs an async future on a multi-threaded Tokio runtime whose worker threads
/// use an 8 MiB stack, keeping large async state machines off Windows' 1 MiB
/// main-thread stack.
#[inline(never)]
pub(crate) fn run_async_on_large_stack<F, T>(fut: F) -> anyhow::Result<T>
where
    F: std::future::Future<Output = anyhow::Result<T>> + Send + 'static,
    T: Send + 'static,
{
    let rt = tokio::runtime::Builder::new_multi_thread()
        .thread_stack_size(8 * 1024 * 1024)
        .enable_all()
        .build()?;
    let handle = rt.spawn(fut);
    rt.block_on(handle)
        .map_err(|e| anyhow::anyhow!("Async worker task panicked: {e}"))?
}

#[allow(clippy::too_many_arguments)]
#[inline(never)]
pub(crate) fn handle_check_attestation(
    bundle: &Path,
    artifact: &Path,
    expected_repo_id: &str,
    expected_repo: &str,
    expected_workflow: &str,
    expected_tag: Option<&str>,
    json: bool,
) -> anyhow::Result<()> {
    let resolved_bundle = if !bundle.exists() {
        let json_candidate = bundle.with_file_name("attestation.json");
        if json_candidate.exists() {
            json_candidate
        } else {
            bundle.to_path_buf()
        }
    } else {
        bundle.to_path_buf()
    };
    let policy = xgauntlet_core::TrustPolicy {
        expected_repo_id: expected_repo_id.to_owned(),
        expected_repo: expected_repo.to_owned(),
        expected_workflow: expected_workflow.to_owned(),
        expected_tag: expected_tag.map(str::to_owned),
    };
    match xgauntlet_core::verify_attestation_file(&resolved_bundle, artifact, &policy) {
        Ok(report) => {
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                render_attestation_report(&report);
            }
            if report.verdict != xgauntlet_core::VerificationVerdict::Passed {
                std::process::exit(1);
            }
        }
        Err(err) => {
            if json {
                let err_obj = serde_json::json!({
                    "verdict": "ERROR",
                    "error": err.to_string(),
                });
                eprintln!("{}", serde_json::to_string_pretty(&err_obj)?);
            } else {
                eprintln!("\x1b[31mError: {}\x1b[0m", err);
            }
            std::process::exit(1);
        }
    }
    Ok(())
}

#[inline(never)]
pub(crate) fn handle_verify(
    workspace: &Path,
    task: Option<&str>,
    save: bool,
    layer: Option<&str>,
    json: bool,
    harness: Option<&str>,
) -> anyhow::Result<()> {
    let ws_owned = workspace.to_path_buf();
    let task_owned = task.map(str::to_owned);
    let layer_owned = layer.map(str::to_owned);
    let outcome = run_async_on_large_stack(async move {
        run_verify(
            &ws_owned,
            task_owned.as_deref(),
            save,
            layer_owned.as_deref(),
        )
        .await
    })?;

    if json {
        println!("{}", serde_json::to_string_pretty(&outcome.report)?);
    } else if let Some(h) = harness {
        let canonical_ws = if workspace.is_absolute() {
            workspace.to_path_buf()
        } else {
            std::env::current_dir()?.join(workspace)
        };
        let telemetry = xgauntlet_core::inspect_task_telemetry(&canonical_ws, task).ok();
        let summary_body = format!(
            "Verification Verdict: {}\nExecuted checks: {} ({} passed, {} failed)",
            outcome.report.verdict,
            outcome.report.checks.len(),
            outcome
                .report
                .checks
                .iter()
                .filter(|c| c.status == xgauntlet_core::CheckStatus::Passed)
                .count(),
            outcome
                .report
                .checks
                .iter()
                .filter(|c| c.status != xgauntlet_core::CheckStatus::Passed)
                .count(),
        );
        match xgauntlet_core::HarnessKind::parse_alias(h) {
            Some(xgauntlet_core::HarnessKind::Antigravity) => {
                let hud = telemetry
                    .as_ref()
                    .map(|t| xgauntlet_core::AntigravityAdapter::render_blockquote_hud(t, None))
                    .unwrap_or_default();
                println!(
                    "{}",
                    xgauntlet_core::AntigravityAdapter::wrap_response(&hud, &summary_body)
                );
            }
            Some(xgauntlet_core::HarnessKind::Codex)
            | Some(xgauntlet_core::HarnessKind::ClaudeCode)
            | Some(xgauntlet_core::HarnessKind::Mistral) => {
                let card = telemetry
                    .as_ref()
                    .map(|t| t.render_box_card())
                    .unwrap_or_default();
                println!(
                    "{}",
                    xgauntlet_core::CodexAdapter::wrap_response(&card, &summary_body)
                );
            }
            None => {
                render_verify_summary(&outcome, save);
            }
        }
    } else {
        render_verify_summary(&outcome, save);
    }

    if outcome.report.verdict == xgauntlet_core::VerificationVerdict::Failed
        || outcome.report.verdict == xgauntlet_core::VerificationVerdict::Incomplete
    {
        std::process::exit(1);
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
#[inline(never)]
pub(crate) fn handle_checkpoint(
    phase: &str,
    task: Option<&str>,
    message: Option<&str>,
    workspace: &Path,
    skip_verify: bool,
    json: bool,
    harness: Option<&str>,
    strict_provenance: bool,
    allow_born_green: bool,
) -> anyhow::Result<()> {
    let canonical_ws = if workspace.is_absolute() {
        workspace.to_path_buf()
    } else {
        std::env::current_dir()?.join(workspace)
    };

    let parsed_phase = match phase.parse::<xgauntlet_core::CheckpointPhase>() {
        Ok(p) => p,
        Err(e) => {
            if json {
                let err_json = serde_json::json!({
                    "success": false,
                    "phase": phase,
                    "error": e.to_string(),
                });
                println!("{}", serde_json::to_string_pretty(&err_json)?);
            } else {
                eprintln!("🛑 Invalid checkpoint phase: {e}");
            }
            std::process::exit(1);
        }
    };

    let mut opts = xgauntlet_core::CheckpointOptions::new(parsed_phase, &canonical_ws)
        .with_skip_verify(skip_verify)
        .with_strict_provenance(strict_provenance)
        .with_allow_born_green(allow_born_green);
    if let Some(t) = task {
        opts = opts.with_task_id(t);
    }
    if let Some(msg) = message {
        opts = opts.with_message(msg);
    }

    let result = run_async_on_large_stack(async move {
        xgauntlet_core::run_checkpoint(&opts)
            .await
            .map_err(anyhow::Error::from)
    });
    match result {
        Ok(res) => {
            if json {
                println!("{}", serde_json::to_string_pretty(&res)?);
            } else if let Some(h) = harness {
                let telemetry = xgauntlet_core::inspect_task_telemetry(&canonical_ws, task).ok();
                let body = format_checkpoint_body(&res);
                match xgauntlet_core::HarnessKind::parse_alias(h) {
                    Some(xgauntlet_core::HarnessKind::Antigravity) => {
                        let hud = telemetry
                            .as_ref()
                            .map(|t| {
                                xgauntlet_core::AntigravityAdapter::render_blockquote_hud(t, None)
                            })
                            .unwrap_or_default();
                        println!(
                            "{}",
                            xgauntlet_core::AntigravityAdapter::wrap_response(&hud, &body)
                        );
                    }
                    Some(xgauntlet_core::HarnessKind::Codex)
                    | Some(xgauntlet_core::HarnessKind::ClaudeCode)
                    | Some(xgauntlet_core::HarnessKind::Mistral) => {
                        let card = telemetry
                            .as_ref()
                            .map(|t| t.render_box_card())
                            .unwrap_or_default();
                        println!(
                            "{}",
                            xgauntlet_core::CodexAdapter::wrap_response(&card, &body)
                        );
                    }
                    None => {
                        render_checkpoint_summary(&res);
                    }
                }
            } else {
                render_checkpoint_summary(&res);
            }
        }
        Err(err) => {
            if json {
                let err_json = serde_json::json!({
                    "success": false,
                    "phase": phase,
                    "error": err.to_string(),
                });
                println!("{}", serde_json::to_string_pretty(&err_json)?);
            } else {
                eprintln!("\n=== xGauntlet Phase Checkpoint Engine ===");
                eprintln!("🛑 Checkpoint rejected: {err}");
            }
            std::process::exit(1);
        }
    }
    Ok(())
}

fn render_attestation_report(report: &xgauntlet_core::AttestationReport) {
    println!("\n=== xGauntlet Attestation & Trust Gatekeeper ===\n");
    let status_badge = if report.verdict == xgauntlet_core::VerificationVerdict::Passed {
        "\x1b[32m[VERIFIED]\x1b[0m"
    } else {
        "\x1b[31m[FAILED / REJECTED]\x1b[0m"
    };

    println!("Artifact:    {} {}", report.artifact_path, status_badge);
    println!("SHA-256:     {}", report.artifact_digest);
    println!("Bundle:      {}", report.bundle_path);
    if let Some(ref subj) = report.matched_subject {
        println!("Subject:     {}", subj);
    }
    println!("Trust State: {:?}", report.trust_decision);

    if !report.claims.is_empty() {
        println!("Claims:");
        for (k, v) in &report.claims {
            println!("  - {}: {}", k, v);
        }
    }

    if !report.reasons.is_empty() {
        println!("\nViolations ({}):", report.reasons.len());
        for r in &report.reasons {
            println!("  \x1b[31m✖ {}\x1b[0m", r);
        }
    }
    println!();
}

async fn run_verify(
    workspace: &Path,
    task_filter: Option<&str>,
    save: bool,
    layer_filter: Option<&str>,
) -> anyhow::Result<xgauntlet_core::GauntletExecutionOutcome> {
    let resolved_task = if let Some(t) = task_filter {
        let tasks_dir = workspace.join("tasks");
        let mut target_id = t.to_string();
        if tasks_dir.is_dir() {
            for entry in std::fs::read_dir(&tasks_dir)?.flatten() {
                let p = entry.path();
                if p.extension().is_some_and(|ext| ext == "md") {
                    let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                    let name = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
                    if stem == t || name == t || stem.starts_with(&format!("{t}-")) {
                        target_id = stem.to_string();
                        break;
                    }
                }
            }
        }
        Some(target_id)
    } else {
        xgauntlet_core::resolve_active_task_id(workspace)
    };

    let layers = if let Some(layer_name) = layer_filter {
        let config = xgauntlet_core::GauntletConfig::load(workspace)?;
        let defs = config.to_layer_definitions();
        let filtered: Vec<_> = defs.into_iter().filter(|l| l.name == layer_name).collect();
        if filtered.is_empty() {
            anyhow::bail!("Layer '{layer_name}' not found in gauntlet configuration");
        }
        Some(filtered)
    } else {
        None
    };

    let options = xgauntlet_core::GauntletOptions {
        task_id: resolved_task,
        save_evidence: save,
        layers,
    };

    let outcome = xgauntlet_core::execute_gauntlet_pipeline(workspace, &options).await?;
    Ok(outcome)
}

fn render_verify_summary(outcome: &xgauntlet_core::GauntletExecutionOutcome, saved: bool) {
    println!("\n=== xGauntlet Verification Execution Engine ===\n");
    let badge = match outcome.report.verdict.as_str() {
        "PASSED" => "\x1b[32m[PASSED]\x1b[0m",
        "PARTIAL" => "\x1b[33m[PARTIAL]\x1b[0m",
        "INCOMPLETE" => "\x1b[33m[INCOMPLETE]\x1b[0m",
        _ => "\x1b[31m[FAILED]\x1b[0m",
    };

    println!(
        "Verdict: {} (Task: {})",
        badge, outcome.report.task_contract.task_id
    );
    println!(
        "  - Self-mutation status:   {}",
        if outcome.is_self_mutated {
            "\x1b[31m[VIOLATION DETECTED]\x1b[0m"
        } else {
            "\x1b[32m[CLEAN]\x1b[0m"
        }
    );
    println!(
        "  - Pre-manifest digest:    {}",
        if outcome
            .report
            .workspace_state
            .source_manifest_digest_pre
            .len()
            > 16
        {
            &outcome.report.workspace_state.source_manifest_digest_pre[..16]
        } else {
            &outcome.report.workspace_state.source_manifest_digest_pre
        }
    );
    println!(
        "  - Post-manifest digest:   {}",
        if outcome
            .report
            .workspace_state
            .source_manifest_digest_post
            .len()
            > 16
        {
            &outcome.report.workspace_state.source_manifest_digest_post[..16]
        } else {
            &outcome.report.workspace_state.source_manifest_digest_post
        }
    );
    println!(
        "  - Execution duration:     {:.3}s",
        outcome.report.execution_metadata.total_duration_seconds
    );

    println!("  - Verification checks ({}):", outcome.report.checks.len());
    for check in &outcome.report.checks {
        let status_badge = if check.passed {
            "\x1b[32m[PASSED]\x1b[0m"
        } else if check.optional {
            "\x1b[33m[OPTIONAL FAILED]\x1b[0m"
        } else {
            "\x1b[31m[FAILED]\x1b[0m"
        };
        println!(
            "    {} {:<25} (exit: {}, duration: {:.3}s)",
            status_badge, check.name, check.exit_code, check.duration_seconds
        );
    }

    if !outcome.report.diagnostics.is_empty() {
        println!(
            "  - Diagnostics findings ({}):",
            outcome.report.diagnostics.len()
        );
        for d in &outcome.report.diagnostics {
            println!(
                "    \x1b[31m[!]\x1b[0m [{}] {}: {}",
                d.tool_name, d.file_path, d.message
            );
            if !d.remediation_hint.is_empty() {
                println!("        Hint: {}", d.remediation_hint);
            }
        }
    }

    if saved {
        println!("  - Artifacts saved:        verification-report.json, evidence.md");
    }
    println!();
}
