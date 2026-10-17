//! Zero-dependency streaming TOML loader, JSON parser, and TOML serializer.

use std::fs;
use std::path::Path;

use super::models::{ArchitectureExemption, ConfigError, GauntletConfig, LayerConfig};
use super::profiles::{default_config_for_stack, detect_stack};

/// Loads configuration from workspace root looking for gauntlet.toml, gauntlet.json, or fallback to stack detection.
pub fn load_config(workspace: &Path) -> Result<GauntletConfig, ConfigError> {
    let toml_path = workspace.join("gauntlet.toml");
    if toml_path.is_file() {
        let content = fs::read_to_string(&toml_path)?;
        return parse_toml(&content);
    }

    let json_path = workspace.join("gauntlet.json");
    if json_path.is_file() {
        let content = fs::read_to_string(&json_path)?;
        return parse_json(&content);
    }

    let detected_stack = detect_stack(workspace);
    Ok(default_config_for_stack(detected_stack))
}

/// Loads configuration from string content given a filename or path hint.
pub fn load_config_from_str(content: &str, path: &Path) -> Result<GauntletConfig, ConfigError> {
    if path.extension().and_then(|s| s.to_str()) == Some("json") {
        parse_json(content)
    } else {
        parse_toml(content)
    }
}

/// Parses gauntlet.json into GauntletConfig via serde_json.
pub fn parse_json(content: &str) -> Result<GauntletConfig, ConfigError> {
    serde_json::from_str(content).map_err(ConfigError::JsonError)
}

/// Parses gauntlet.toml into GauntletConfig using a lightweight streaming parser.
pub fn parse_toml(content: &str) -> Result<GauntletConfig, ConfigError> {
    let mut config = GauntletConfig::default();
    if content.contains("[[layers]]") {
        config.layers.clear();
    }

    let mut current_section = "";
    let mut current_layer: Option<LayerConfig> = None;
    let mut current_exemption: Option<ArchitectureExemption> = None;

    for (line_idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if trimmed == "[[layers]]" {
            if let Some(layer) = current_layer.take() {
                config.layers.push(layer);
            }
            if let Some(ex) = current_exemption.take() {
                config.architecture.exemptions.push(ex);
            }
            current_layer = Some(LayerConfig {
                name: String::new(),
                command: Vec::new(),
                optional: false,
                timeout_seconds: 60.0,
                fail_on_output: false,
            });
            current_section = "layers";
            continue;
        }

        if trimmed == "[[architecture.exemptions]]" {
            if let Some(layer) = current_layer.take() {
                config.layers.push(layer);
            }
            if let Some(ex) = current_exemption.take() {
                config.architecture.exemptions.push(ex);
            }
            current_exemption = Some(ArchitectureExemption::default());
            current_section = "architecture.exemptions";
            continue;
        }

        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            if let Some(layer) = current_layer.take() {
                config.layers.push(layer);
            }
            if let Some(ex) = current_exemption.take() {
                config.architecture.exemptions.push(ex);
            }
            let section_name = trimmed.trim_start_matches('[').trim_end_matches(']').trim();
            current_section = match section_name {
                "paths" => "paths",
                "architecture" => "architecture",
                other => other,
            };
            continue;
        }

        if let Some((k, v)) = trimmed.split_once('=') {
            let key = k.trim();
            let val = v.trim();

            if current_section == "layers" {
                if let Some(ref mut l) = current_layer {
                    match key {
                        "name" => l.name = parse_str_val(val),
                        "command" => l.command = parse_str_list(val),
                        "optional" => l.optional = parse_bool_val(val),
                        "timeout_seconds" => l.timeout_seconds = parse_float_val(val, 60.0),
                        "fail_on_output" => l.fail_on_output = parse_bool_val(val),
                        _ => {}
                    }
                }
            } else if current_section == "architecture.exemptions" {
                if let Some(ref mut ex) = current_exemption {
                    match key {
                        "path" => ex.path = parse_str_val(val),
                        "reason" => ex.reason = parse_str_val(val),
                        "max_lines" => ex.max_lines = Some(parse_usize_val(val)),
                        _ => {}
                    }
                }
            } else if current_section == "architecture" {
                match key {
                    "max_file_lines" => {
                        config.architecture.max_file_lines = Some(parse_usize_val(val))
                    }
                    "max_test_file_lines" => {
                        config.architecture.max_test_file_lines = Some(parse_usize_val(val))
                    }
                    "forbidden_tokens" => {
                        config.architecture.forbidden_tokens = parse_str_list(val)
                    }
                    "forbidden_imports" => {
                        config.architecture.forbidden_imports = parse_str_list(val)
                    }
                    _ => {}
                }
            } else if current_section == "paths" {
                match key {
                    "tasks_dir" => config.paths.tasks_dir = parse_str_val(val),
                    "spec_file" => config.paths.spec_file = parse_str_val(val),
                    "context_file" => config.paths.context_file = parse_str_val(val),
                    "coding_standards_file" => {
                        config.paths.coding_standards_file = parse_str_val(val)
                    }
                    _ => {}
                }
            } else {
                match key {
                    "stack" => config.stack = parse_str_val(val),
                    "save_evidence" => config.save_evidence = parse_bool_val(val),
                    "evidence_file" => config.evidence_file = parse_str_val(val),
                    "evidence_markdown_file" => config.evidence_markdown_file = parse_str_val(val),
                    _ => {}
                }
            }
        } else {
            return Err(ConfigError::TomlError(format!(
                "Syntax error on line {}: expected 'key = value', found '{}'",
                line_idx + 1,
                trimmed
            )));
        }
    }

    if let Some(layer) = current_layer.take() {
        config.layers.push(layer);
    }
    if let Some(ex) = current_exemption.take() {
        config.architecture.exemptions.push(ex);
    }

    Ok(config)
}

/// Serializes a GauntletConfig into clean, canonical TOML format.
pub fn render_toml(config: &GauntletConfig) -> String {
    let mut out = String::new();
    out.push_str("# xGauntlet Declarative Configuration\n");
    out.push_str(&format!("stack = \"{}\"\n", config.stack));
    out.push_str(&format!("save_evidence = {}\n", config.save_evidence));
    out.push_str(&format!("evidence_file = \"{}\"\n", config.evidence_file));
    out.push_str(&format!(
        "evidence_markdown_file = \"{}\"\n\n",
        config.evidence_markdown_file
    ));

    out.push_str("[paths]\n");
    out.push_str(&format!("tasks_dir = \"{}\"\n", config.paths.tasks_dir));
    out.push_str(&format!("spec_file = \"{}\"\n", config.paths.spec_file));
    out.push_str(&format!(
        "context_file = \"{}\"\n",
        config.paths.context_file
    ));
    out.push_str(&format!(
        "coding_standards_file = \"{}\"\n\n",
        config.paths.coding_standards_file
    ));

    out.push_str("[architecture]\n");
    if let Some(max) = config.architecture.max_file_lines {
        out.push_str(&format!("max_file_lines = {}\n", max));
    }
    if let Some(max) = config.architecture.max_test_file_lines {
        out.push_str(&format!("max_test_file_lines = {}\n", max));
    }
    if !config.architecture.forbidden_tokens.is_empty() {
        let tokens_fmt: Vec<String> = config
            .architecture
            .forbidden_tokens
            .iter()
            .map(|t| format!("\"{}\"", t.replace('\"', "\\\"")))
            .collect();
        out.push_str(&format!("forbidden_tokens = [{}]\n", tokens_fmt.join(", ")));
    }
    if !config.architecture.forbidden_imports.is_empty() {
        let imports_fmt: Vec<String> = config
            .architecture
            .forbidden_imports
            .iter()
            .map(|i| format!("\"{}\"", i.replace('\"', "\\\"")))
            .collect();
        out.push_str(&format!(
            "forbidden_imports = [{}]\n",
            imports_fmt.join(", ")
        ));
    }
    out.push('\n');

    for ex in &config.architecture.exemptions {
        out.push_str("[[architecture.exemptions]]\n");
        out.push_str(&format!("path = \"{}\"\n", ex.path));
        out.push_str(&format!("reason = \"{}\"\n", ex.reason));
        if let Some(ml) = ex.max_lines {
            out.push_str(&format!("max_lines = {}\n", ml));
        }
        out.push('\n');
    }

    for layer in &config.layers {
        out.push_str("[[layers]]\n");
        out.push_str(&format!("name = \"{}\"\n", layer.name));
        let cmd_formatted: Vec<String> = layer
            .command
            .iter()
            .map(|arg| format!("\"{}\"", arg.replace('\"', "\\\"")))
            .collect();
        out.push_str(&format!("command = [{}]\n", cmd_formatted.join(", ")));
        out.push_str(&format!("optional = {}\n", layer.optional));
        if layer.fail_on_output {
            out.push_str("fail_on_output = true\n");
        }
        out.push_str(&format!(
            "timeout_seconds = {:.1}\n\n",
            layer.timeout_seconds
        ));
    }

    out
}

fn parse_usize_val(val: &str) -> usize {
    val.trim().parse::<usize>().unwrap_or(0)
}

fn parse_str_val(val: &str) -> String {
    val.trim().trim_matches('"').trim_matches('\'').to_string()
}

fn parse_bool_val(val: &str) -> bool {
    matches!(val.trim().to_ascii_lowercase().as_str(), "true" | "1")
}

fn parse_float_val(val: &str, default: f64) -> f64 {
    val.trim().parse::<f64>().unwrap_or(default)
}

fn parse_str_list(val: &str) -> Vec<String> {
    let trimmed = val.trim();
    if !trimmed.starts_with('[') || !trimmed.ends_with(']') {
        return vec![parse_str_val(trimmed)];
    }
    let inner = &trimmed[1..trimmed.len() - 1];
    let mut items = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut quote_char = '"';

    for ch in inner.chars() {
        if in_quotes {
            if ch == quote_char {
                in_quotes = false;
                items.push(current.clone());
                current.clear();
            } else {
                current.push(ch);
            }
        } else if ch == '"' || ch == '\'' {
            in_quotes = true;
            quote_char = ch;
        } else if ch == ',' {
            let item = current.trim();
            if !item.is_empty() {
                items.push(item.to_string());
                current.clear();
            }
        } else if !ch.is_whitespace() {
            current.push(ch);
        }
    }

    let final_item = current.trim();
    if !final_item.is_empty() {
        items.push(final_item.to_string());
    }

    items
}
