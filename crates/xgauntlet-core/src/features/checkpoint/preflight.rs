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
        package: Option<String>,
        target_name: String,
        filter: Option<String>,
        command: Vec<String>,
    },
    HarnessSubmodule {
        package: Option<String>,
        harness_name: String,
        submodule: String,
        command: Vec<String>,
    },
    LibModule {
        package: Option<String>,
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

    // Detect crate package from path prefix before /src/ or /tests/
    let package = if let Some(idx) = clean_ref
        .find("/tests/")
        .or_else(|| clean_ref.find("/src/"))
    {
        let prefix = &clean_ref[..idx];
        let crate_cargo = workspace.join(prefix).join("Cargo.toml");
        if crate_cargo.is_file() {
            if let Ok(content) = std::fs::read_to_string(&crate_cargo) {
                let mut in_pkg = false;
                let mut pkg_name = None;
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed == "[package]" {
                        in_pkg = true;
                    } else if in_pkg && trimmed.starts_with('[') {
                        in_pkg = false;
                    } else if in_pkg && trimmed.starts_with("name") {
                        if let Some((_, val)) = trimmed.split_once('=') {
                            pkg_name =
                                Some(val.trim().trim_matches('"').trim_matches('\'').to_string());
                            break;
                        }
                    }
                }
                pkg_name.or_else(|| prefix.split('/').next_back().map(|s| s.to_string()))
            } else {
                prefix.split('/').next_back().map(|s| s.to_string())
            }
        } else {
            prefix.split('/').next_back().map(|s| s.to_string())
        }
    } else {
        None
    };

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
        let mut command = vec!["cargo".to_string(), "test".to_string()];
        if let Some(ref pkg) = package {
            command.push("-p".to_string());
            command.push(pkg.clone());
        }
        command.push("--lib".to_string());
        command.push(module_path.clone());
        return RustTestTarget::LibModule {
            package,
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
            let mut command = vec!["cargo".to_string(), "test".to_string()];
            if let Some(ref pkg) = package {
                command.push("-p".to_string());
                command.push(pkg.clone());
            }
            command.push("--test".to_string());
            command.push(actual_harness.clone());
            command.push(submod.clone());
            return RustTestTarget::HarnessSubmodule {
                package,
                harness_name: actual_harness,
                submodule: submod,
                command,
            };
        }
    }

    // Check if Cargo.toml has an explicit [[test]] mapping for this file
    let candidate_cargo = if let Some(idx) = clean_ref.find("/tests/") {
        workspace.join(&clean_ref[..idx]).join("Cargo.toml")
    } else {
        workspace.join("Cargo.toml")
    };
    let mut explicit_target_name = None;
    if candidate_cargo.is_file() {
        if let Ok(content) = std::fs::read_to_string(&candidate_cargo) {
            let mut in_test_block = false;
            let mut current_name: Option<String> = None;
            let mut current_path: Option<String> = None;
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed == "[[test]]" {
                    if let (Some(n), Some(p)) = (current_name.take(), current_path.take()) {
                        let norm_p = p.replace('\\', "/");
                        if clean_ref.ends_with(&norm_p) {
                            explicit_target_name = Some(n);
                            break;
                        }
                    }
                    in_test_block = true;
                } else if in_test_block && trimmed.starts_with('[') {
                    if let (Some(n), Some(p)) = (current_name.take(), current_path.take()) {
                        let norm_p = p.replace('\\', "/");
                        if clean_ref.ends_with(&norm_p) {
                            explicit_target_name = Some(n);
                            break;
                        }
                    }
                    in_test_block = false;
                } else if in_test_block {
                    if trimmed.starts_with("name") {
                        if let Some((_, val)) = trimmed.split_once('=') {
                            current_name =
                                Some(val.trim().trim_matches('"').trim_matches('\'').to_string());
                        }
                    } else if trimmed.starts_with("path") {
                        if let Some((_, val)) = trimmed.split_once('=') {
                            current_path =
                                Some(val.trim().trim_matches('"').trim_matches('\'').to_string());
                        }
                    }
                }
            }
            if let (Some(n), Some(p)) = (current_name, current_path) {
                let norm_p = p.replace('\\', "/");
                if clean_ref.ends_with(&norm_p) {
                    explicit_target_name = Some(n);
                }
            }
        }
    }

    // Case A: Standalone integration test
    let file_name = parts.last().unwrap_or(&"");
    let fallback_target_name = file_name
        .strip_suffix(".rs")
        .unwrap_or(file_name)
        .to_string();
    let target_name = explicit_target_name.unwrap_or(fallback_target_name);
    let mut command = vec!["cargo".to_string(), "test".to_string()];
    if let Some(ref pkg) = package {
        command.push("-p".to_string());
        command.push(pkg.clone());
    }
    command.push("--test".to_string());
    command.push(target_name.clone());
    RustTestTarget::IntegrationTest {
        package,
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
                // AC1: Differentiate between a test assertion failure / missing symbol in test vs syntax/manifest failure in production code
                let output = &res.output;
                let has_syntax_or_manifest_err = (output.contains("error: expected one of")
                    || output.contains("error[E0658]")
                    || output.contains("error: could not parse")
                    || output.contains("error: failed to parse manifest")
                    || output.contains("syntax error"))
                    && (output.contains("/src/")
                        || output.contains("src/")
                        || output.contains("Cargo.toml"));
                if has_syntax_or_manifest_err {
                    return Err(CheckpointError::RedPhaseUnmet {
                        details: format!(
                            "Syntax or manifest error in production code detected during RED preflight for '{}': {}",
                            layer.name,
                            output.trim()
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
            // Blocker 1: Check phase progression invariants
            let task_info = super::engine::load_task_package_info(&options.workspace, &task_id);
            let has_red = has_task_red_commit(&options.workspace, &task_id);

            if !has_red {
                if let Some(ref info) = task_info {
                    let changed =
                        get_working_tree_changed_files(&options.workspace).unwrap_or_default();
                    if !super::provenance::is_fast_track_eligible(info, &changed) {
                        return Err(CheckpointError::DonePhaseUnmet(
                            "Skipping TDD phases (RED/GREEN) is only permitted for Fast-Track Bugfix Lane tasks (intent: bug, scope <= 3 files).".to_string(),
                        ));
                    }
                }
            }

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

/// Checks if a task has a preceding RED checkpoint commit in Git history.
pub fn has_task_red_commit(workspace: &Path, task_id: &str) -> bool {
    let short_id = super::git::extract_short_task_id(task_id);
    let output = match std::process::Command::new("git")
        .args(["log", "-n", "50", "--format=%s"])
        .current_dir(workspace)
        .output()
    {
        Ok(out) => out,
        Err(_) => return false,
    };
    if !output.status.success() {
        return false;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("test(")
            && (trimmed.contains(task_id) || (!short_id.is_empty() && trimmed.contains(short_id)))
            && (trimmed.contains("[RED]") || trimmed.contains("red"))
        {
            return true;
        }
    }
    false
}

/// Lists modified, added, or deleted files in the working tree.
pub fn get_working_tree_changed_files(workspace: &Path) -> Result<Vec<String>, std::io::Error> {
    let output = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(workspace)
        .output()?;
    if !output.status.success() {
        return Ok(Vec::new());
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut files = Vec::new();
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.len() > 3 {
            let path_part = trimmed[3..].trim();
            files.push(path_part.to_string());
        }
    }
    Ok(files)
}
