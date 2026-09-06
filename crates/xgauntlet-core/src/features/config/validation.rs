//! Declarative configuration schema and path safety validation.

use std::collections::HashSet;
use std::path::Path;

use super::models::{ConfigValidationIssue, ConfigValidationReport, GauntletConfig};
use super::profiles::SUPPORTED_STACKS;

/// Validates a GauntletConfig against schema, naming, timeouts, and path safety invariants.
pub fn validate_config(
    config: &GauntletConfig,
    workspace: Option<&Path>,
) -> ConfigValidationReport {
    let mut report = ConfigValidationReport::new();

    // 1. Stack Profile Validation
    if config.stack.trim().is_empty() {
        report.add_issue(ConfigValidationIssue::error(
            "MISSING_STACK",
            "Configuration must specify a target programming stack (e.g. 'rust', 'python', 'node', 'go').",
            Some("stack".to_string()),
            Some("Set 'stack = \"rust\"' or other supported stack profile.".to_string()),
        ));
    } else if !SUPPORTED_STACKS.contains(&config.stack.to_ascii_lowercase().as_str()) {
        report.add_issue(ConfigValidationIssue::warning(
            "CUSTOM_STACK_PROFILE",
            format!(
                "Stack '{}' is a custom profile outside the default catalog ({:?}). Ensure all layers are explicitly configured.",
                config.stack, SUPPORTED_STACKS
            ),
            Some("stack".to_string()),
            Some(format!("Recognized standard stacks are: {}", SUPPORTED_STACKS.join(", "))),
        ));
    }

    // 2. Paths Safety Validation (Self-containment and path traversal check)
    validate_path_safety("paths.tasks_dir", &config.paths.tasks_dir, &mut report);
    validate_path_safety("paths.spec_file", &config.paths.spec_file, &mut report);
    validate_path_safety(
        "paths.context_file",
        &config.paths.context_file,
        &mut report,
    );
    validate_path_safety(
        "paths.coding_standards_file",
        &config.paths.coding_standards_file,
        &mut report,
    );
    validate_path_safety("evidence_file", &config.evidence_file, &mut report);
    validate_path_safety(
        "evidence_markdown_file",
        &config.evidence_markdown_file,
        &mut report,
    );

    // If workspace root is supplied, check existence of required docs
    if let Some(ws) = workspace {
        let tasks_path = ws.join(&config.paths.tasks_dir);
        if !tasks_path.exists() {
            report.add_issue(ConfigValidationIssue::warning(
                "TASKS_DIR_MISSING",
                format!(
                    "Configured tasks directory '{}' does not exist on disk.",
                    config.paths.tasks_dir
                ),
                Some("paths.tasks_dir".to_string()),
                Some(format!(
                    "Create directory '{}' or update gauntlet.toml.",
                    config.paths.tasks_dir
                )),
            ));
        }

        let context_path = ws.join(&config.paths.context_file);
        if !context_path.is_file() {
            report.add_issue(ConfigValidationIssue::warning(
                "CONTEXT_FILE_MISSING",
                format!(
                    "Configured domain glossary '{}' does not exist on disk.",
                    config.paths.context_file
                ),
                Some("paths.context_file".to_string()),
                Some(format!(
                    "Create '{}' following Aristotelian formula.",
                    config.paths.context_file
                )),
            ));
        }
    }

    // 3. Layers Validation
    if config.layers.is_empty() {
        report.add_issue(ConfigValidationIssue::error(
            "NO_LAYERS_DEFINED",
            "Gauntlet configuration has zero verification layers defined.",
            Some("layers".to_string()),
            Some("Define at least one [[layers]] entry in gauntlet.toml.".to_string()),
        ));
    }

    let mut seen_layer_names = HashSet::new();

    for (idx, layer) in config.layers.iter().enumerate() {
        let field_prefix = format!("layers[{}]", idx);

        // Name check
        let trimmed_name = layer.name.trim();
        if trimmed_name.is_empty() {
            report.add_issue(ConfigValidationIssue::error(
                "EMPTY_LAYER_NAME",
                format!("Layer at index {} has an empty name.", idx),
                Some(format!("{}.name", field_prefix)),
                Some("Provide a unique alphanumeric name for the layer.".to_string()),
            ));
        } else {
            let is_valid_name = trimmed_name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
            if !is_valid_name {
                report.add_issue(ConfigValidationIssue::error(
                    "INVALID_LAYER_NAME",
                    format!(
                        "Layer name '{}' contains invalid characters. Only alphanumeric, '-' and '_' are permitted.",
                        trimmed_name
                    ),
                    Some(format!("{}.name", field_prefix)),
                    Some("Use names like 'lint', 'types', 'unit-tests', 'mutation'.".to_string()),
                ));
            }

            if !seen_layer_names.insert(trimmed_name.to_string()) {
                report.add_issue(ConfigValidationIssue::error(
                    "DUPLICATE_LAYER_NAME",
                    format!("Duplicate layer name '{}' detected.", trimmed_name),
                    Some(format!("{}.name", field_prefix)),
                    Some("Each layer name in gauntlet.toml must be globally unique.".to_string()),
                ));
            }
        }

        // Command check
        if layer.command.is_empty() {
            report.add_issue(ConfigValidationIssue::error(
                "EMPTY_LAYER_COMMAND",
                format!("Layer '{}' has an empty command list.", layer.name),
                Some(format!("{}.command", field_prefix)),
                Some("Specify an executable command array, e.g. ['cargo', 'test'].".to_string()),
            ));
        } else {
            for (arg_idx, arg) in layer.command.iter().enumerate() {
                if arg.trim().is_empty() {
                    report.add_issue(ConfigValidationIssue::error(
                        "EMPTY_COMMAND_TOKEN",
                        format!(
                            "Layer '{}' contains an empty command token at position {}.",
                            layer.name, arg_idx
                        ),
                        Some(format!("{}.command[{}]", field_prefix, arg_idx)),
                        Some("Remove empty command tokens.".to_string()),
                    ));
                }
            }
        }

        // Timeout check
        if layer.timeout_seconds <= 0.0 || layer.timeout_seconds > 3600.0 {
            report.add_issue(ConfigValidationIssue::error(
                "INVALID_TIMEOUT",
                format!(
                    "Layer '{}' has invalid timeout of {} seconds. Must be between 0.1 and 3600.0 seconds.",
                    layer.name, layer.timeout_seconds
                ),
                Some(format!("{}.timeout_seconds", field_prefix)),
                Some("Set timeout_seconds to a reasonable duration, e.g. 60.0.".to_string()),
            ));
        }
    }

    report
}

fn validate_path_safety(field: &str, path_str: &str, report: &mut ConfigValidationReport) {
    let trimmed = path_str.trim();
    if trimmed.is_empty() {
        report.add_issue(ConfigValidationIssue::error(
            "EMPTY_PATH",
            format!("Path field '{}' cannot be empty.", field),
            Some(field.to_string()),
            Some("Specify a relative path within the workspace.".to_string()),
        ));
        return;
    }

    let p = Path::new(trimmed);
    if p.is_absolute() {
        report.add_issue(ConfigValidationIssue::error(
            "ABSOLUTE_PATH",
            format!(
                "Path field '{}' specifies absolute path '{}'. Only relative workspace paths are permitted.",
                field, trimmed
            ),
            Some(field.to_string()),
            Some("Use relative path, e.g. 'tasks' instead of '/tasks'.".to_string()),
        ));
    }

    for comp in p.components() {
        if let std::path::Component::ParentDir = comp {
            report.add_issue(ConfigValidationIssue::error(
                "PATH_TRAVERSAL",
                format!(
                    "Path field '{}' contains parent directory traversal ('..'): '{}'. Breach of workspace boundary.",
                    field, trimmed
                ),
                Some(field.to_string()),
                Some("Avoid '..' in workspace paths.".to_string()),
            ));
            break;
        }
    }
}
