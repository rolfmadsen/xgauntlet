//! Native CLI executable for xGauntlet.

use clap::{Parser, Subcommand};
use xgauntlet_core::get_engine_info;

#[derive(Parser, Debug)]
#[command(name = "xgauntlet")]
#[command(about = "Universal cross-platform verification gauntlet & policy sidecar", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Show platform and engine status
    Status,
    /// Fast workspace, environment, Git, and toolchain diagnostics engine
    Doctor {
        /// Path to repository workspace root
        #[arg(short, long, default_value = ".")]
        workspace: std::path::PathBuf,

        /// Verbose output displaying full check details
        #[arg(short, long)]
        verbose: bool,

        /// Strict mode: fail if any warnings exist
        #[arg(long)]
        strict: bool,

        /// Filter diagnostics to a specific category ('host', 'git', 'governance', 'toolchains', 'engine')
        #[arg(long)]
        category: Option<String>,

        /// Output diagnostic report in structured JSON format
        #[arg(long)]
        json: bool,
    },
    /// Shift-left specification, task package, and Aristotelian domain glossary gatekeeper
    CheckSpec {
        /// Path to repository workspace root
        #[arg(short, long, default_value = ".")]
        workspace: std::path::PathBuf,

        /// Specific task identifier to validate (e.g. '003' or '003-task-and-spec-engine')
        #[arg(short, long)]
        task: Option<String>,

        /// Validate all tasks in tasks/ directory
        #[arg(short, long)]
        all: bool,

        /// Output findings in structured JSON format
        #[arg(long)]
        json: bool,
    },
    /// Verify workspace source manifest and multi-digest integrity against verification-report.json
    CheckEvidence {
        /// Path to repository workspace root
        #[arg(short, long, default_value = ".")]
        workspace: std::path::PathBuf,

        /// Output verification results in structured JSON format
        #[arg(long)]
        json: bool,
    },
    /// Execute multi-layer verification gauntlet with timeouts, self-mutation checks, and evidence generation
    Verify {
        /// Path to repository workspace root
        #[arg(short, long, default_value = ".")]
        workspace: std::path::PathBuf,

        /// Specific task identifier to verify (e.g. '005' or '005-gauntlet-execution-engine')
        #[arg(short, long)]
        task: Option<String>,

        /// Save verification-report.json and evidence.md to workspace root
        #[arg(short, long)]
        save: bool,

        /// Specific layer name to execute (e.g. 'unit' or 'lint')
        #[arg(short, long)]
        layer: Option<String>,

        /// Output verification report in structured JSON format
        #[arg(long)]
        json: bool,
    },
    /// Intercept agent tool calls and evaluate capability requests against policy engine
    Hook {
        /// Target harness environment (e.g. 'antigravity', 'claude_code', 'codex')
        #[arg(default_value = "antigravity")]
        harness: String,

        /// Path to repository workspace root
        #[arg(short, long, default_value = ".")]
        workspace: std::path::PathBuf,
    },
    /// Mechanically validate plugin manifest, skills, and hooks for an agent harness
    ValidatePlugin {
        /// Path to plugin directory (e.g. '.agents' or '.claude')
        #[arg(short, long, default_value = ".agents")]
        plugin_dir: std::path::PathBuf,

        /// Target harness environment (e.g. 'antigravity', 'claude_code', 'codex')
        #[arg(long, default_value = "antigravity")]
        harness: String,

        /// Output results in structured JSON format
        #[arg(long)]
        json: bool,
    },
    /// Validate declarative gauntlet configuration and stack profile
    CheckConfig {
        /// Path to repository workspace root
        #[arg(short, long, default_value = ".")]
        workspace: std::path::PathBuf,

        /// Output results in structured JSON format
        #[arg(long)]
        json: bool,
    },
    /// Initialize in-repo governance files, stack profile, and agent hooks with safe non-destructive guarantee
    Init {
        /// Path to repository workspace root
        #[arg(short, long, default_value = ".")]
        workspace: std::path::PathBuf,

        /// Target programming stack profile (e.g. 'rust', 'python', 'node', 'go')
        #[arg(short, long)]
        stack: Option<String>,

        /// Overwrite existing template files (force non-destructive bypass)
        #[arg(short, long)]
        force: bool,

        /// Preview scaffold actions without writing files to disk
        #[arg(short, long)]
        dry_run: bool,

        /// Project name (defaults to directory name)
        #[arg(long)]
        name: Option<String>,

        /// Output scaffold results in structured JSON format
        #[arg(long)]
        json: bool,

        /// Target harness integration (e.g. 'claude_code', 'codex', 'antigravity')
        #[arg(long)]
        harness: Option<String>,
    },
    /// Mechanical release readiness gatekeeper, manifest version harmony, and ADR documentation coverage
    CheckRelease {
        /// Path to repository workspace root
        #[arg(short, long, default_value = ".")]
        workspace: std::path::PathBuf,

        /// Allow 'Unreleased' changelog section instead of strict version match
        #[arg(long)]
        allow_unreleased: bool,

        /// Strict mode: fail if any warnings exist
        #[arg(long)]
        strict: bool,

        /// Output release readiness report in structured JSON format
        #[arg(long)]
        json: bool,
    },
    /// Task lifecycle management, intent scaffolding, and telemetry
    Task {
        #[command(subcommand)]
        command: TaskCommands,
    },
    /// Dynamic response HUD telemetry card and harness hook stream
    Telemetry {
        /// Explicit task identifier (e.g. '016' or '016-claude-code-hud-adapter')
        #[arg(short, long)]
        task: Option<String>,

        /// Path to repository workspace root
        #[arg(short, long, default_value = ".")]
        workspace: std::path::PathBuf,

        /// Format to output telemetry in: box, claude-hook, codex-hook, json, ansi, compact-box
        #[arg(short, long, default_value = "box")]
        format: String,
    },
    /// Create a phase-bound TDD checkpoint with pre-flight invariant verification and local conventional git commit
    Checkpoint {
        /// TDD phase for this checkpoint: spec, red, green, refactor, done
        #[arg(short, long)]
        phase: String,

        /// Explicit task identifier (e.g. '015' or '015-security-and-policy-boundary-hardening')
        #[arg(short, long)]
        task: Option<String>,

        /// Commit message (conventional prefix will be formatted automatically if omitted)
        #[arg(short, long)]
        message: Option<String>,

        /// Path to repository workspace root
        #[arg(short, long, default_value = ".")]
        workspace: std::path::PathBuf,

        /// Skip phase pre-flight verification checks (emergency override)
        #[arg(long)]
        skip_verify: bool,

        /// Output checkpoint result in structured JSON format
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand, Debug)]
enum TaskCommands {
    /// Scaffold a new task package with sequential numbering and OKF v0.2 frontmatter
    New {
        /// Task package slug name (e.g. 'task-lifecycle-and-intent-scaffolding')
        name: String,

        /// Explicit human-readable title (defaults to title-cased name)
        #[arg(short, long)]
        title: Option<String>,

        /// Task engineering intent ('feature', 'bug', 'refactor')
        #[arg(short, long, default_value = "feature")]
        intent: String,

        /// Concrete task purpose statement
        #[arg(short, long)]
        purpose: Option<String>,

        /// Path to repository workspace root
        #[arg(short, long, default_value = ".")]
        workspace: std::path::PathBuf,

        /// Overwrite existing task package if collision occurs
        #[arg(short, long)]
        force: bool,

        /// Output scaffold results in structured JSON format
        #[arg(long)]
        json: bool,
    },
    /// Inspect active or specified task status, criteria progress, and Git telemetry
    Status {
        /// Specific task identifier to inspect (e.g. '013' or '013-task-lifecycle-and-intent-scaffolding')
        #[arg(short, long)]
        task: Option<String>,

        /// Path to repository workspace root
        #[arg(short, long, default_value = ".")]
        workspace: std::path::PathBuf,

        /// Output task telemetry in structured JSON format
        #[arg(long)]
        json: bool,
    },
    /// List all task packages with status, title, and criteria progression
    List {
        /// Path to repository workspace root
        #[arg(short, long, default_value = ".")]
        workspace: std::path::PathBuf,

        /// Output task list in structured JSON format
        #[arg(long)]
        json: bool,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let engine = get_engine_info();

    match &cli.command {
        Some(Commands::Status) => {
            println!("xGauntlet engine: {} v{}", engine.name, engine.version);
            println!(
                "Platform target: {}-{}",
                std::env::consts::OS,
                std::env::consts::ARCH
            );
        }
        Some(Commands::Doctor {
            workspace,
            verbose,
            strict,
            category,
            json,
        }) => {
            let cat_enum = category
                .as_deref()
                .and_then(xgauntlet_core::DoctorCategory::parse_str);
            let options = xgauntlet_core::DoctorOptions {
                workspace: workspace.clone(),
                verbose: *verbose,
                strict: *strict,
                category: cat_enum,
            };

            let report = xgauntlet_core::run_doctor(&options)?;

            if *json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                render_doctor_summary(&report, *verbose);
            }

            let is_success = if *strict {
                report.is_strictly_healthy()
            } else {
                report.is_healthy()
            };

            if !is_success {
                std::process::exit(1);
            }
        }
        Some(Commands::CheckSpec {
            workspace,
            task,
            all,
            json,
        }) => {
            let reports = run_check_spec(workspace, task.as_deref(), *all)?;
            let all_valid = reports.iter().all(|r| r.is_valid);

            if *json {
                println!("{}", serde_json::to_string_pretty(&reports)?);
            } else {
                render_check_spec_summary(&reports);
            }

            if !all_valid {
                std::process::exit(1);
            }
        }
        Some(Commands::CheckEvidence { workspace, json }) => {
            let res = run_check_evidence(workspace)?;
            if *json {
                println!("{}", serde_json::to_string_pretty(&res)?);
            } else {
                render_check_evidence_summary(&res);
            }

            if !res.passed {
                std::process::exit(1);
            }
        }
        Some(Commands::Verify {
            workspace,
            task,
            save,
            layer,
            json,
        }) => {
            let outcome = run_verify(workspace, task.as_deref(), *save, layer.as_deref()).await?;
            if *json {
                println!("{}", serde_json::to_string_pretty(&outcome.report)?);
            } else {
                render_verify_summary(&outcome, *save);
            }

            if outcome.report.verdict == "FAILED" || outcome.report.verdict == "INCOMPLETE" {
                std::process::exit(1);
            }
        }
        Some(Commands::Hook { harness, workspace }) => {
            let canonical_ws = workspace
                .canonicalize()
                .unwrap_or_else(|_| workspace.clone());
            let adapter = match xgauntlet_core::get_adapter(harness) {
                Some(a) => a,
                None => {
                    eprintln!(
                        "🛑 Unsupported harness '{harness}'. Supported harnesses: {:?}",
                        xgauntlet_core::SUPPORTED_HARNESSES
                    );
                    std::process::exit(1);
                }
            };

            use std::io::Read;
            let mut buffer = String::new();
            std::io::stdin().read_to_string(&mut buffer)?;

            let (exit_code, output) = adapter.handle_hook(&canonical_ws, &buffer);
            println!("{output}");
            if exit_code != 0 {
                std::process::exit(exit_code);
            }
        }
        Some(Commands::ValidatePlugin {
            plugin_dir,
            harness,
            json,
        }) => {
            let adapter = match xgauntlet_core::get_adapter(harness) {
                Some(a) => a,
                None => {
                    eprintln!(
                        "🛑 Unsupported harness '{harness}'. Supported harnesses: {:?}",
                        xgauntlet_core::SUPPORTED_HARNESSES
                    );
                    std::process::exit(1);
                }
            };

            let target = if plugin_dir.is_absolute() {
                plugin_dir.clone()
            } else {
                std::env::current_dir()?.join(plugin_dir)
            };

            let res = adapter.validate_plugin(&target);
            if *json {
                println!("{}", serde_json::to_string_pretty(&res)?);
            } else {
                let status_label = if res.valid { "VALID" } else { "INVALID" };
                println!(
                    "[{status_label}] Plugin validation for '{}' ({harness}):",
                    target.display()
                );
                if res.issues.is_empty() {
                    println!("  [+] Manifest, skills, and hooks are intact and valid.");
                } else {
                    for issue in &res.issues {
                        let tag = match issue.severity {
                            xgauntlet_core::ValidationSeverity::Error => "[!]",
                            xgauntlet_core::ValidationSeverity::Warning => "[*]",
                        };
                        println!(
                            "  {} {} ({}): {}",
                            tag,
                            issue.severity.as_str(),
                            issue.path,
                            issue.message
                        );
                    }
                }
            }

            if !res.valid {
                std::process::exit(1);
            }
        }
        Some(Commands::CheckConfig { workspace, json }) => {
            let canonical_ws = if workspace.is_absolute() {
                workspace.clone()
            } else {
                std::env::current_dir()?.join(workspace)
            };

            let config = xgauntlet_core::GauntletConfig::load(&canonical_ws)?;
            let report = xgauntlet_core::validate_config(&config, Some(&canonical_ws));

            if *json {
                let payload = serde_json::json!({
                    "stack": config.stack,
                    "is_valid": report.is_valid,
                    "issues": report.issues,
                    "layers_count": config.layers.len(),
                });
                println!("{}", serde_json::to_string_pretty(&payload)?);
            } else {
                render_check_config_summary(&config, &report);
            }

            if !report.is_valid {
                std::process::exit(1);
            }
        }
        Some(Commands::Init {
            workspace,
            stack,
            force,
            dry_run,
            name,
            json,
            harness,
        }) => {
            let canonical_ws = if workspace.is_absolute() {
                workspace.clone()
            } else {
                std::env::current_dir()?.join(workspace)
            };

            let options = xgauntlet_core::ScaffoldOptions {
                workspace: canonical_ws.clone(),
                stack: stack.clone(),
                force: *force,
                dry_run: *dry_run,
                project_name: name.clone(),
            };

            let result = xgauntlet_core::run_scaffold(&options)?;

            if let Some(ref h) = harness {
                if (h == "claude_code" || h == "claude") && !*dry_run {
                    let _ = xgauntlet_core::ClaudeCodeAdapter::scaffold_settings(&canonical_ws);
                } else if (h == "codex" || h == "openai_codex") && !*dry_run {
                    let _ = xgauntlet_core::CodexAdapter::scaffold_hooks(&canonical_ws);
                }
            }

            if *json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                render_init_summary(&result, *dry_run);
            }
        }
        Some(Commands::CheckRelease {
            workspace,
            allow_unreleased,
            strict,
            json,
        }) => {
            let canonical_ws = if workspace.is_absolute() {
                workspace.clone()
            } else {
                std::env::current_dir()?.join(workspace)
            };

            let options = xgauntlet_core::ReleaseReadinessOptions {
                workspace: canonical_ws,
                allow_unreleased: *allow_unreleased,
                strict: *strict,
            };

            let report = xgauntlet_core::check_release_readiness(&options)?;

            if *json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                render_release_summary(&report);
            }

            if !report.is_ready {
                std::process::exit(1);
            }
        }
        Some(Commands::Task { command }) => match command {
            TaskCommands::New {
                name,
                title,
                intent,
                purpose,
                workspace,
                force,
                json,
            } => {
                let canonical_ws = if workspace.is_absolute() {
                    workspace.clone()
                } else {
                    std::env::current_dir()?.join(workspace)
                };

                let options = xgauntlet_core::ScaffoldTaskOptions {
                    name: name.clone(),
                    title: title.clone(),
                    intent: Some(intent.clone()),
                    purpose: purpose.clone(),
                    workspace: canonical_ws,
                    force: *force,
                };

                let res = xgauntlet_core::TaskScaffolder::scaffold(&options)?;

                if *json {
                    println!("{}", serde_json::to_string_pretty(&res)?);
                } else {
                    render_task_scaffold_summary(&res);
                }
            }
            TaskCommands::Status {
                task,
                workspace,
                json,
            } => {
                let canonical_ws = if workspace.is_absolute() {
                    workspace.clone()
                } else {
                    std::env::current_dir()?.join(workspace)
                };

                let telemetry =
                    xgauntlet_core::inspect_task_telemetry(&canonical_ws, task.as_deref())?;

                if *json {
                    println!("{}", serde_json::to_string_pretty(&telemetry)?);
                } else {
                    render_task_status_summary(&telemetry);
                }
            }
            TaskCommands::List { workspace, json } => {
                let canonical_ws = if workspace.is_absolute() {
                    workspace.clone()
                } else {
                    std::env::current_dir()?.join(workspace)
                };

                let items = xgauntlet_core::list_workspace_tasks(&canonical_ws)?;

                if *json {
                    println!("{}", serde_json::to_string_pretty(&items)?);
                } else {
                    render_task_list_summary(&items);
                }
            }
        },

        Some(Commands::Telemetry {
            task,
            workspace,
            format,
        }) => {
            let canonical_ws = if workspace.is_absolute() {
                workspace.clone()
            } else {
                std::env::current_dir()?.join(workspace)
            };

            let telemetry = xgauntlet_core::inspect_task_telemetry(&canonical_ws, task.as_deref())?;

            match format.to_ascii_lowercase().as_str() {
                "claude-hook" | "claude" => {
                    let box_card = telemetry.render_box_card();
                    let payload =
                        xgauntlet_core::ClaudeCodeAdapter::format_post_tool_use_payload(&box_card);
                    println!("{}", serde_json::to_string_pretty(&payload)?);
                }
                "codex-hook" | "codex" => {
                    let box_card = telemetry.render_box_card();
                    let payload =
                        xgauntlet_core::CodexAdapter::format_post_tool_use_payload(&box_card);
                    println!("{}", serde_json::to_string_pretty(&payload)?);
                }
                "json" => {
                    println!("{}", serde_json::to_string_pretty(&telemetry)?);
                }
                "compact-box" | "compact" => {
                    println!("{}", telemetry.render_box_compact());
                }
                "box" | "ansi" => {
                    println!("{}", telemetry.render_box_card());
                }
                _ => {
                    println!("{}", telemetry.render_box_card());
                }
            }
        }

        Some(Commands::Checkpoint {
            phase,
            task,
            message,
            workspace,
            skip_verify,
            json,
        }) => {
            let canonical_ws = if workspace.is_absolute() {
                workspace.clone()
            } else {
                std::env::current_dir()?.join(workspace)
            };

            let parsed_phase = match phase.parse::<xgauntlet_core::CheckpointPhase>() {
                Ok(p) => p,
                Err(e) => {
                    if *json {
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
                .with_skip_verify(*skip_verify);
            if let Some(t) = task {
                opts = opts.with_task_id(t);
            }
            if let Some(msg) = message {
                opts = opts.with_message(msg);
            }

            match xgauntlet_core::run_checkpoint(&opts).await {
                Ok(res) => {
                    if *json {
                        println!("{}", serde_json::to_string_pretty(&res)?);
                    } else {
                        render_checkpoint_summary(&res);
                    }
                }
                Err(err) => {
                    if *json {
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
        }

        None => {
            println!(
                "xGauntlet v{} - Run with --help for options",
                engine.version
            );
        }
    }

    Ok(())
}

fn run_check_spec(
    workspace: &std::path::Path,
    task_filter: Option<&str>,
    all: bool,
) -> anyhow::Result<Vec<xgauntlet_core::SpecReadinessReport>> {
    let tasks_dir = workspace.join("tasks");

    if all {
        return Ok(xgauntlet_core::check_all_tasks(workspace));
    }

    if let Some(target) = task_filter {
        let mut target_path = None;
        if tasks_dir.is_dir() {
            for entry in std::fs::read_dir(&tasks_dir)?.flatten() {
                let p = entry.path();
                if p.extension().is_some_and(|ext| ext == "md") {
                    let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                    let name = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
                    if stem == target
                        || name == target
                        || stem.starts_with(&format!("{target}-"))
                        || name.starts_with(&format!("{target}-"))
                        || stem.contains(target)
                    {
                        target_path = Some(p);
                        break;
                    }
                }
            }
        }

        let p = target_path.unwrap_or_else(|| tasks_dir.join(format!("{target}.md")));
        return Ok(vec![xgauntlet_core::check_task_specification(
            &p, workspace,
        )]);
    }

    if let Some(active_id) = xgauntlet_core::resolve_active_task_id(workspace) {
        let task_path = tasks_dir.join(format!("{active_id}.md"));
        return Ok(vec![xgauntlet_core::check_task_specification(
            &task_path, workspace,
        )]);
    }

    // No active task found: validate CONTEXT.md and return empty report with glossary diags
    let diags = xgauntlet_core::validate_context_glossary(workspace);
    let mut rep = xgauntlet_core::SpecReadinessReport::new("no-active-task");
    rep.is_valid = diags.is_empty();
    rep.diagnostics = diags;
    if workspace.join("CONTEXT.md").is_file() {
        rep.inspected_files.push("CONTEXT.md".to_string());
    }
    Ok(vec![rep])
}

fn render_check_spec_summary(reports: &[xgauntlet_core::SpecReadinessReport]) {
    println!("\n=== xGauntlet Specification & Shift-Left Gatekeeper ===\n");
    for rep in reports {
        let status_badge = if rep.is_valid {
            "\x1b[32m[PASSED]\x1b[0m"
        } else {
            "\x1b[31m[FAILED]\x1b[0m"
        };
        println!("Task: {} {}", rep.task_id, status_badge);
        println!(
            "  - Acceptance criteria count: {}",
            rep.acceptance_criteria.len()
        );
        println!(
            "  - Must NOT rules count:      {}",
            rep.must_not_rules.len()
        );
        println!(
            "  - Inspected files:           {}",
            rep.inspected_files.join(", ")
        );

        if !rep.diagnostics.is_empty() {
            println!("  - Diagnostics findings ({}):", rep.diagnostics.len());
            for d in &rep.diagnostics {
                println!(
                    "    \x1b[31m[!]\x1b[0m [{}] {}: {}",
                    d.tool_name, d.file_path, d.message
                );
                if !d.remediation_hint.is_empty() {
                    println!("        Hint: {}", d.remediation_hint);
                }
            }
        }
        println!();
    }
}

#[derive(Debug, serde::Serialize)]
struct CheckEvidenceOutput {
    passed: bool,
    task_id: String,
    report_source_manifest: String,
    current_source_manifest: String,
    findings: Vec<xgauntlet_core::DiagnosticFinding>,
}

fn run_check_evidence(workspace: &std::path::Path) -> anyhow::Result<CheckEvidenceOutput> {
    let report_result = xgauntlet_core::load_verification_report(workspace);
    let report = match report_result {
        Ok(r) => r,
        Err(e) => {
            return Ok(CheckEvidenceOutput {
                passed: false,
                task_id: "unknown".to_string(),
                report_source_manifest: String::new(),
                current_source_manifest: String::new(),
                findings: vec![xgauntlet_core::DiagnosticFinding::new(
                    xgauntlet_core::FindingType::GeneralError,
                    "xgauntlet-check-evidence",
                    "verification-report.json",
                    format!("Failed to load verification report: {e}"),
                    "Run 'xgauntlet verify' first to seal evidence for the active task.",
                )],
            });
        }
    };

    let current_manifest = xgauntlet_core::compute_workspace_manifest(workspace, None)?;
    let match_result = xgauntlet_core::verify_workspace_state_match(&report, &current_manifest);

    let (passed, findings) = match match_result {
        Ok(()) => (true, Vec::new()),
        Err(drift) => (false, vec![drift.to_diagnostic_finding()]),
    };

    Ok(CheckEvidenceOutput {
        passed,
        task_id: report.task_contract.task_id,
        report_source_manifest: report.workspace_state.source_manifest_digest_post,
        current_source_manifest: current_manifest.source_manifest_digest,
        findings,
    })
}

fn render_check_evidence_summary(res: &CheckEvidenceOutput) {
    println!("\n=== xGauntlet Evidence & Drift Gatekeeper ===\n");
    let status_badge = if res.passed {
        "\x1b[32m[PASSED]\x1b[0m"
    } else {
        "\x1b[31m[DRIFT DETECTED]\x1b[0m"
    };
    println!("Task: {} {}", res.task_id, status_badge);
    if !res.report_source_manifest.is_empty() {
        println!(
            "  - Recorded manifest digest: {}",
            if res.report_source_manifest.len() > 16 {
                &res.report_source_manifest[..16]
            } else {
                &res.report_source_manifest
            }
        );
    }
    if !res.current_source_manifest.is_empty() {
        println!(
            "  - Current manifest digest:  {}",
            if res.current_source_manifest.len() > 16 {
                &res.current_source_manifest[..16]
            } else {
                &res.current_source_manifest
            }
        );
    }

    if !res.findings.is_empty() {
        println!("  - Drift findings ({}):", res.findings.len());
        for d in &res.findings {
            println!(
                "    \x1b[31m[!]\x1b[0m [{}] {}: {}",
                d.tool_name, d.file_path, d.message
            );
            if !d.remediation_hint.is_empty() {
                println!("        Hint: {}", d.remediation_hint);
            }
        }
    } else {
        println!(
            "  - Status: Workspace source tree and policies are 100% in sync with verified report."
        );
    }
    println!();
}

async fn run_verify(
    workspace: &std::path::Path,
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

fn render_check_config_summary(
    config: &xgauntlet_core::GauntletConfig,
    report: &xgauntlet_core::ConfigValidationReport,
) {
    println!("\n=== xGauntlet Declarative Configuration & Stack Profile ===\n");
    println!("Stack:              {}", config.stack);
    println!("Save evidence:      {}", config.save_evidence);
    println!("Evidence file:      {}", config.evidence_file);
    println!("Evidence markdown:  {}", config.evidence_markdown_file);
    println!("Tasks directory:    {}", config.paths.tasks_dir);
    println!("Spec file:          {}", config.paths.spec_file);
    println!("Context file:       {}", config.paths.context_file);
    println!("Standards file:     {}", config.paths.coding_standards_file);
    println!("\nVerification Layers ({}):", config.layers.len());
    for l in &config.layers {
        let opt_label = if l.optional {
            "\x1b[33m[optional]\x1b[0m"
        } else {
            "\x1b[32m[required]\x1b[0m"
        };
        println!(
            "  - Layer: {:<25} {} (timeout: {:.0}s) -> {:?}",
            l.name, opt_label, l.timeout_seconds, l.command
        );
    }
    println!();
    if report.is_valid {
        println!("\x1b[32m[PASSED] Configuration is VALID (0 errors).\x1b[0m\n");
    } else {
        println!("\x1b[31m[FAILED] Configuration has ERRORS:\x1b[0m");
        for issue in &report.issues {
            println!(
                "  - [{:?}] {}: {}",
                issue.severity, issue.code, issue.message
            );
            if let Some(ref rem) = issue.remediation {
                println!("    Hint: {}", rem);
            }
        }
        println!();
    }
}

fn render_init_summary(result: &xgauntlet_core::ScaffoldResult, dry_run: bool) {
    println!("\n=== xGauntlet Project Initialization & Scaffold Engine ===\n");
    if dry_run {
        println!(
            "\x1b[33m[DRY RUN] Previewing file actions without modifying filesystem:\x1b[0m\n"
        );
    }

    println!("Workspace:     {}", result.workspace.display());
    println!("Stack profile: {}", result.stack);
    println!("\nFiles:");

    for file in &result.files {
        let (tag, color) = match file.action {
            xgauntlet_core::ScaffoldAction::Created => ("[+] CREATED", "\x1b[32m"),
            xgauntlet_core::ScaffoldAction::Skipped => ("[*] SKIPPED", "\x1b[33m"),
            xgauntlet_core::ScaffoldAction::Overwritten => ("[!] OVERWRITTEN", "\x1b[35m"),
            xgauntlet_core::ScaffoldAction::WouldCreate => ("[+] WOULD CREATE", "\x1b[32m"),
            xgauntlet_core::ScaffoldAction::WouldSkip => ("[*] WOULD SKIP", "\x1b[33m"),
            xgauntlet_core::ScaffoldAction::WouldOverwrite => ("[!] WOULD OVERWRITE", "\x1b[35m"),
        };

        let reason_str = match &file.reason {
            Some(r) => format!(" ({r})"),
            None => String::new(),
        };

        println!("  {}{:<17}\x1b[0m {}{}", color, tag, file.path, reason_str);
    }

    println!();
    if dry_run {
        let would_create = result
            .files
            .iter()
            .filter(|f| f.action == xgauntlet_core::ScaffoldAction::WouldCreate)
            .count();
        let would_skip = result
            .files
            .iter()
            .filter(|f| f.action == xgauntlet_core::ScaffoldAction::WouldSkip)
            .count();
        let would_overwrite = result
            .files
            .iter()
            .filter(|f| f.action == xgauntlet_core::ScaffoldAction::WouldOverwrite)
            .count();
        println!(
            "Summary: {} would create, {} would skip, {} would overwrite.",
            would_create, would_skip, would_overwrite
        );
    } else {
        println!(
            "Summary: {} created, {} skipped, {} overwritten.",
            result.created_count, result.skipped_count, result.overwritten_count
        );
        println!("\n\x1b[32m[SUCCESS] In-repo governance files initialized safely.\x1b[0m");
        println!("Next steps:");
        println!("  1. Inspect 'gauntlet.toml' and 'CONTEXT.md'.");
        println!("  2. Run 'xgauntlet check-spec' to validate specification contracts.");
        println!("  3. Run 'xgauntlet verify' to execute your verification gauntlet.\n");
    }
}

fn render_doctor_summary(report: &xgauntlet_core::DoctorReport, verbose: bool) {
    println!("\n=== xGauntlet Workspace Diagnostics & Doctor Engine ===\n");
    println!("Workspace: {}", report.workspace.display());
    println!(
        "Platform:  {} (Engine v{})",
        report.target_platform, report.engine_version
    );
    println!("Timestamp: {}", report.timestamp);
    println!();

    let categories = [
        xgauntlet_core::DoctorCategory::Host,
        xgauntlet_core::DoctorCategory::Git,
        xgauntlet_core::DoctorCategory::Governance,
        xgauntlet_core::DoctorCategory::Toolchains,
        xgauntlet_core::DoctorCategory::Engine,
    ];

    for cat in &categories {
        let cat_checks: Vec<_> = report
            .checks
            .iter()
            .filter(|c| c.category == *cat)
            .collect();
        if cat_checks.is_empty() {
            continue;
        }

        println!("Category: \x1b[1m{}\x1b[0m", cat.as_str().to_uppercase());
        for check in cat_checks {
            let (tag, color) = match check.status {
                xgauntlet_core::DoctorCheckStatus::Pass => ("[PASS]", "\x1b[32m"),
                xgauntlet_core::DoctorCheckStatus::Warn => ("[WARN]", "\x1b[33m"),
                xgauntlet_core::DoctorCheckStatus::Fail => ("[FAIL]", "\x1b[31m"),
                xgauntlet_core::DoctorCheckStatus::Info => ("[INFO]", "\x1b[36m"),
            };

            println!(
                "  {}{:<6}\x1b[0m {:<28} {} ({:.1}ms)",
                color, tag, check.name, check.message, check.duration_ms
            );

            if verbose {
                if let Some(ref detail) = check.detail {
                    println!("         \x1b[90mDetail: {}\x1b[0m", detail);
                }
            }

            if let Some(ref remediation) = check.remediation {
                println!("         \x1b[33mAction: {}\x1b[0m", remediation);
            }
        }
        println!();
    }

    println!("--------------------------------------------------");
    println!(
        "Summary: {} passed, {} warning(s), {} failed, {} info. Total time: {}ms",
        report.passed_count,
        report.warn_count,
        report.failed_count,
        report.info_count,
        report.duration_total_ms
    );

    match report.verdict {
        xgauntlet_core::DoctorVerdict::Healthy => {
            println!("\x1b[32m[HEALTHY] Workspace environment and toolchains are ready.\x1b[0m\n");
        }
        xgauntlet_core::DoctorVerdict::Degraded => {
            println!(
                "\x1b[33m[DEGRADED] Workspace has warnings. Inspect remediation hints above.\x1b[0m\n"
            );
        }
        xgauntlet_core::DoctorVerdict::Critical => {
            println!(
                "\x1b[31m[CRITICAL] Workspace has failing checks. Remediation required.\x1b[0m\n"
            );
        }
    }
}

fn render_release_summary(report: &xgauntlet_core::ReleaseReadinessReport) {
    println!("\n=== \x1b[1m🛡️ xGauntlet Release Readiness Gatekeeper\x1b[0m ===\n");
    println!(
        "Target Declared Version: \x1b[1m{}\x1b[0m",
        report.declared_version
    );
    println!("Inspected Files:         {}", report.inspected_files.len());
    println!("Execution Duration:      {}ms\n", report.duration_ms);

    println!("Manifests & Versions:");
    if report.versions_by_source.is_empty() {
        println!("  \x1b[33m(no manifests found)\x1b[0m");
    } else {
        for (src, ver) in &report.versions_by_source {
            println!("  • {:<30} \x1b[32m{}\x1b[0m", src, ver);
        }
    }
    println!();

    println!("Changelog Releases:");
    if report.changelog_versions.is_empty() {
        println!("  \x1b[33m(no releases recorded in CHANGELOG.md)\x1b[0m");
    } else {
        let preview: Vec<_> = report.changelog_versions.iter().take(5).collect();
        println!(
            "  • Found {} release(s): {}",
            report.changelog_versions.len(),
            preview
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    println!();

    let adr_count = report
        .inspected_files
        .iter()
        .filter(|f| f.starts_with("docs/adr/"))
        .count();
    let unref_count = report.unreferenced_adrs.len();
    let ref_count = adr_count.saturating_sub(unref_count);
    println!("Architecture Decision Records (ADRs):");
    if adr_count == 0 {
        println!("  \x1b[90m(no ADRs found in docs/adr/)\x1b[0m");
    } else {
        let pct = (ref_count * 100).checked_div(adr_count).unwrap_or(100);
        println!(
            "  • Coverage: {}/{} ADRs referenced in README.md/spec.md ({}%)",
            ref_count, adr_count, pct
        );
    }
    println!();

    if !report.diagnostics.is_empty() {
        println!("--------------------------------------------------");
        println!("\x1b[31m[!] Release Discrepancies & Blockers:\x1b[0m");
        for d in &report.diagnostics {
            println!(
                "  \x1b[31m• [{}] {}\x1b[0m: {}",
                d.category, d.file_path, d.message
            );
            println!("    \x1b[33mAction: {}\x1b[0m", d.remediation_hint);
        }
        println!("--------------------------------------------------\n");
    }

    if report.is_ready {
        println!("\x1b[32;1m✅ RELEASE READY\x1b[0m: All manifests in harmony, changelog synced, and ADR coverage complete.\n");
    } else {
        println!(
            "\x1b[31;1m❌ RELEASE BLOCKED\x1b[0m: Resolve the {} issue(s) detailed above before publishing.\n",
            report.diagnostics.len()
        );
    }
}

fn render_task_scaffold_summary(res: &xgauntlet_core::TaskScaffoldResult) {
    println!("✅ Scaffolding complete for task [{}]", res.task_id);
    println!("   Number: {:03}", res.task_number);
    println!("   Title:  {}", res.title);
    println!("   Intent: {}", res.intent);
    println!("   Path:   {}", res.path.display());
}

fn render_task_status_summary(telemetry: &xgauntlet_core::TaskTelemetry) {
    println!(
        "🛡️  Task Telemetry: [{}] {}",
        telemetry.task_id, telemetry.title
    );
    let intent_str = telemetry.intent.as_deref().unwrap_or("🚀 NEW FEATURE");
    println!(
        "   Status:   {} | Intent: {}",
        telemetry.status.as_str(),
        intent_str
    );
    println!(
        "   Progress: Criteria: {}/{} {} {}%",
        telemetry.criteria.completed,
        telemetry.criteria.total,
        telemetry.criteria.bar,
        telemetry.criteria.percentage
    );
    let git_status = if telemetry.git.is_clean {
        "clean".to_string()
    } else {
        format!("dirty: {} files", telemetry.git.dirty_count)
    };
    println!(
        "   Git:      {}@{} • {}",
        telemetry.git.branch, telemetry.git.head_oid, git_status
    );
    println!("   Path:     {}", telemetry.file_path);
}

fn render_task_list_summary(items: &[xgauntlet_core::TaskSummaryItem]) {
    if items.is_empty() {
        println!("No task packages discovered in tasks/.");
        return;
    }
    println!("📋 Task Packages ({} total):", items.len());
    println!("{:<6} {:<10} {:<18} TITLE", "ID", "STATUS", "PROGRESS");
    println!("{}", "-".repeat(75));
    for item in items {
        let num_str = item
            .task_number
            .map(|n| format!("{n:03}"))
            .unwrap_or_else(|| "---".to_string());
        let prog_str = format!(
            "{}/{} {} {:>3}%",
            item.criteria.completed,
            item.criteria.total,
            item.criteria.bar,
            item.criteria.percentage
        );
        println!(
            "{:<6} {:<10} {:<18} {}",
            num_str,
            item.status.as_str(),
            prog_str,
            item.title
        );
    }
}

fn render_checkpoint_summary(res: &xgauntlet_core::CheckpointResult) {
    println!("\n=== xGauntlet Phase Checkpoint Engine ===");
    let phase_badge = match res.phase {
        xgauntlet_core::CheckpointPhase::Spec => "\x1b[36m[SPEC]\x1b[0m",
        xgauntlet_core::CheckpointPhase::Red => "\x1b[31m[RED]\x1b[0m",
        xgauntlet_core::CheckpointPhase::Green => "\x1b[32m[GREEN]\x1b[0m",
        xgauntlet_core::CheckpointPhase::Refactor => "\x1b[33m[REFACTOR]\x1b[0m",
        xgauntlet_core::CheckpointPhase::Done => "\x1b[35m[DONE]\x1b[0m",
    };
    println!(
        "🛡️  Phase:      {} {}",
        res.phase.as_str().to_uppercase(),
        phase_badge
    );
    println!("📋 Task ID:    {}", res.task_id);
    let oid_short = res
        .commit_oid
        .as_deref()
        .map(|oid| if oid.len() >= 7 { &oid[..7] } else { oid })
        .unwrap_or("unknown");
    println!(
        "🔒 Commit OID: {} ({})",
        oid_short,
        res.commit_oid.as_deref().unwrap_or("none")
    );
    println!("💬 Message:    {}", res.commit_message);
    println!("📦 Staged:     {} file(s)", res.staged_files.len());
    for f in &res.staged_files {
        println!("   + {}", f);
    }
    println!();
}
