//! Plugin and manifest validation for Google Antigravity.

use serde_json::Value;
use std::fs;
use std::path::Path;

use crate::features::adapters::models::{
    AdapterValidationResult, ValidationIssue, ValidationSeverity,
};

pub const VALID_ANTIGRAVITY_EVENTS: &[&str] = &[
    "PreToolUse",
    "PostToolUse",
    "PreInvocation",
    "PostInvocation",
    "Stop",
];

pub fn validate_plugin(plugin_dir: &Path) -> AdapterValidationResult {
    let mut issues = Vec::new();

    if !plugin_dir.is_dir() {
        issues.push(ValidationIssue {
            severity: ValidationSeverity::Error,
            path: plugin_dir.display().to_string(),
            message: format!(
                "Plugin directory '{}' does not exist.",
                plugin_dir.display()
            ),
        });
        return AdapterValidationResult::failure(issues);
    }

    // 1. Validate plugin.json
    let manifest_path = plugin_dir.join("plugin.json");
    let has_hooks = plugin_dir.join("hooks.json").is_file();
    let mut manifest_data: Option<Value> = None;

    if !manifest_path.is_file() {
        if !has_hooks {
            issues.push(ValidationIssue {
                severity: ValidationSeverity::Error,
                path: "plugin.json".to_string(),
                message: "Missing required manifest file 'plugin.json'.".to_string(),
            });
        }
    } else {
        match fs::read_to_string(&manifest_path) {
            Ok(content) => match serde_json::from_str::<Value>(&content) {
                Ok(val) => {
                    if !val.is_object() {
                        issues.push(ValidationIssue {
                            severity: ValidationSeverity::Error,
                            path: "plugin.json".to_string(),
                            message: "'plugin.json' must be a JSON object.".to_string(),
                        });
                    } else {
                        manifest_data = Some(val);
                    }
                }
                Err(e) => {
                    issues.push(ValidationIssue {
                        severity: ValidationSeverity::Error,
                        path: "plugin.json".to_string(),
                        message: format!("Invalid JSON in 'plugin.json': {e}"),
                    });
                }
            },
            Err(e) => {
                issues.push(ValidationIssue {
                    severity: ValidationSeverity::Error,
                    path: "plugin.json".to_string(),
                    message: format!("Could not read 'plugin.json': {e}"),
                });
            }
        }
    }

    if let Some(manifest) = manifest_data {
        let name_valid = manifest
            .get("name")
            .and_then(|n| n.as_str())
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false);
        if !name_valid {
            issues.push(ValidationIssue {
                severity: ValidationSeverity::Error,
                path: "plugin.json".to_string(),
                message: "Missing required field 'name' in 'plugin.json'.".to_string(),
            });
        }

        let version_valid = manifest
            .get("version")
            .and_then(|v| v.as_str())
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false);
        if !version_valid {
            issues.push(ValidationIssue {
                severity: ValidationSeverity::Error,
                path: "plugin.json".to_string(),
                message: "Missing required field 'version' in 'plugin.json'.".to_string(),
            });
        }

        // 2. Validate declared skills
        if let Some(skills) = manifest.get("skills").and_then(|s| s.as_array()) {
            for skill in skills {
                if let Some(skill_name) = skill.as_str() {
                    let skill_file = plugin_dir.join("skills").join(skill_name).join("SKILL.md");

                    if !skill_file.is_file() {
                        issues.push(ValidationIssue {
                            severity: ValidationSeverity::Error,
                            path: format!("skills/{}/SKILL.md", skill_name),
                            message: format!(
                                "Declared skill '{}' is missing 'SKILL.md' file.",
                                skill_name
                            ),
                        });
                    } else {
                        match fs::read_to_string(&skill_file) {
                            Ok(content) => {
                                let has_frontmatter = content.starts_with("---")
                                    && content.contains("name:")
                                    && content.contains("description:");
                                if !has_frontmatter {
                                    issues.push(ValidationIssue {
                                        severity: ValidationSeverity::Error,
                                        path: format!("skills/{}/SKILL.md", skill_name),
                                        message: format!(
                                            "Skill '{}' is missing valid YAML frontmatter (name, description).",
                                            skill_name
                                        ),
                                    });
                                }
                            }
                            Err(e) => {
                                issues.push(ValidationIssue {
                                    severity: ValidationSeverity::Error,
                                    path: format!("skills/{}/SKILL.md", skill_name),
                                    message: format!("Failed to read SKILL.md: {e}"),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    // 3. Validate hooks.json if present
    let hooks_path = plugin_dir.join("hooks.json");
    if hooks_path.is_file() {
        match fs::read_to_string(&hooks_path) {
            Ok(content) => match serde_json::from_str::<Value>(&content) {
                Ok(val) => {
                    if let Some(root_obj) = val.as_object() {
                        if root_obj.is_empty() {
                            issues.push(ValidationIssue {
                                severity: ValidationSeverity::Error,
                                path: "hooks.json".to_string(),
                                message: "'hooks.json' root object cannot be empty.".to_string(),
                            });
                        } else {
                            for (hook_name, hook_config) in root_obj {
                                let Some(h_obj) = hook_config.as_object() else {
                                    issues.push(ValidationIssue {
                                        severity: ValidationSeverity::Error,
                                        path: "hooks.json".to_string(),
                                        message: format!("Configuration for hook '{hook_name}' must be an object."),
                                    });
                                    continue;
                                };

                                let has_valid_event = h_obj
                                    .keys()
                                    .any(|k| VALID_ANTIGRAVITY_EVENTS.contains(&k.as_str()));

                                if !has_valid_event {
                                    issues.push(ValidationIssue {
                                        severity: ValidationSeverity::Error,
                                        path: "hooks.json".to_string(),
                                        message: format!(
                                            "Hook '{hook_name}' must define at least one lifecycle event from {:?}",
                                            VALID_ANTIGRAVITY_EVENTS
                                        ),
                                    });
                                }
                            }
                        }
                    } else {
                        issues.push(ValidationIssue {
                            severity: ValidationSeverity::Error,
                            path: "hooks.json".to_string(),
                            message: "'hooks.json' root must be a JSON object.".to_string(),
                        });
                    }
                }
                Err(e) => {
                    issues.push(ValidationIssue {
                        severity: ValidationSeverity::Error,
                        path: "hooks.json".to_string(),
                        message: format!("Invalid JSON in 'hooks.json': {e}"),
                    });
                }
            },
            Err(e) => {
                issues.push(ValidationIssue {
                    severity: ValidationSeverity::Error,
                    path: "hooks.json".to_string(),
                    message: format!("Failed to read 'hooks.json': {e}"),
                });
            }
        }
    }

    let has_errors = issues
        .iter()
        .any(|i| i.severity == ValidationSeverity::Error);
    AdapterValidationResult {
        valid: !has_errors,
        issues,
    }
}
