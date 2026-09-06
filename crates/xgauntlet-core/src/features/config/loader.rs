//! Zero-dependency streaming TOML loader, JSON parser, and TOML serializer.

use std::fs;
use std::path::Path;

use super::models::{ConfigError, GauntletConfig, LayerConfig};
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

    for (line_idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if trimmed == "[[layers]]" {
            if let Some(layer) = current_layer.take() {
                config.layers.push(layer);
            }
            current_layer = Some(LayerConfig {
                name: String::new(),
                command: Vec::new(),
                optional: false,
                timeout_seconds: 60.0,
            });
            current_section = "layers";
            continue;
        }

        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            if let Some(layer) = current_layer.take() {
                config.layers.push(layer);
            }
            let section_name = trimmed.trim_start_matches('[').trim_end_matches(']').trim();
            current_section = match section_name {
                "paths" => "paths",
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
                        _ => {}
                    }
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
        out.push_str(&format!(
            "timeout_seconds = {:.1}\n\n",
            layer.timeout_seconds
        ));
    }

    out
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
