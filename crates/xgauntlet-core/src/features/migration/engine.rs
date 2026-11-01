//! Core in-repo schema migration and doctor auto-remediation engine.

use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

use super::models::{MigrationError, MigrationItem, MigrationOptions, MigrationReport};

/// Standard Architectural Decision Record template content.
const ADR_TEMPLATE_CONTENT: &str = r#"---
type: Architectural Decision Record
title: 'ADR [Number]: [Short Title]'
status: draft
tags: [architecture, adr]
generated:
  by: human:<author>
  at: 'YYYY-MM-DDTHH:MM:SSZ'
verified: []
---

# [ADR Number]. [Short Title]

**Status**: `draft | accepted | deprecated | superseded`  
**Date**: `YYYY-MM-DD`  

## Context
Beskriv baggrunden, problemstillingen, forretningskontekst og de tekniske udfordringer eller begrænsninger, som denne beslutning adresserer.

## Decision
Beskriv den valgte arkitektur- eller designbeslutning, herunder hvilke mønstre, snitflader og invariante regler der indføres.

## Consequences
Beskriv de positive og negative konsekvenser, trade-offs og eventuel teknisk gæld, der følger af denne beslutning.
"#;

/// Executes in-place migration and modernization of workspace governance artifacts.
pub fn run_migration(options: &MigrationOptions) -> Result<MigrationReport, MigrationError> {
    let ws = &options.workspace;
    let mut items = Vec::new();
    let mut files_modified = Vec::new();

    // 1. Surgical migration of .agents/hooks.json
    if let Some(item) = migrate_hooks_json(ws, options.dry_run, &mut files_modified)? {
        items.push(item);
    }

    // 2. Surgical migration of gauntlet.toml
    if let Some(item) = migrate_gauntlet_toml(ws, options.dry_run, &mut files_modified)? {
        items.push(item);
    }

    // 3. Scaffolding of missing docs/adr/template.md
    if let Some(item) = scaffold_missing_adr_template(ws, options.dry_run, &mut files_modified)? {
        items.push(item);
    }

    // 4. Recommendation for integration test harness consolidation if [[test]] count > 10
    let cargo_path = ws.join("Cargo.toml");
    if cargo_path.is_file() {
        if let Ok(cargo_content) = fs::read_to_string(&cargo_path) {
            let test_binary_count = cargo_content
                .lines()
                .filter(|l| l.trim() == "[[test]]")
                .count();
            if test_binary_count > 10 {
                items.push(MigrationItem {
                    file: PathBuf::from("Cargo.toml"),
                    description: "Rust integration test harness linker optimization recommendation".to_string(),
                    changes: vec![format!(
                        "Notice: Found {test_binary_count} separate [[test]] binaries in Cargo.toml. Consider consolidating into module-based tests under tests/features.rs (ADR 0011) to eliminate linker overhead."
                    )],
                });
            }
        }
    }

    Ok(MigrationReport {
        is_success: true,
        items,
        files_modified,
    })
}

fn migrate_hooks_json(
    workspace: &Path,
    dry_run: bool,
    files_modified: &mut Vec<PathBuf>,
) -> Result<Option<MigrationItem>, MigrationError> {
    let hooks_path = workspace.join(".agents").join("hooks.json");
    if !hooks_path.is_file() {
        return Ok(None);
    }

    let content = fs::read_to_string(&hooks_path)?;
    let mut root: Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            return Err(MigrationError::JsonParse {
                path: hooks_path,
                source: e,
            })
        }
    };

    let mut changes = Vec::new();

    let root_obj = match root.as_object_mut() {
        Some(obj) => obj,
        None => return Ok(None),
    };

    // 1. Resolve and modernize namespace: handle legacy, modern, or merge of both
    let legacy_key = if root_obj.contains_key("agent-gauntlet-gatekeeper") {
        Some("agent-gauntlet-gatekeeper")
    } else if root_obj.contains_key("gauntlet-gatekeeper") {
        Some("gauntlet-gatekeeper")
    } else {
        None
    };

    let mut group = match (legacy_key, root_obj.contains_key("xgauntlet")) {
        (Some(leg_key), true) => {
            // Both legacy and modern group exist: merge legacy into modern without clobbering
            let legacy_val = root_obj.remove(leg_key).unwrap();
            let mut canonical_val = root_obj.remove("xgauntlet").unwrap();

            changes.push(format!(
                "Merged legacy namespace '{leg_key}' into 'xgauntlet' without data loss"
            ));

            if let (Some(leg_obj), Some(canon_obj)) =
                (legacy_val.as_object(), canonical_val.as_object_mut())
            {
                // Merge PreToolUse
                if let Some(leg_pre_tool) = leg_obj.get("PreToolUse").and_then(|v| v.as_array()) {
                    let canon_pre_tool = canon_obj
                        .entry("PreToolUse".to_string())
                        .or_insert_with(|| Value::Array(Vec::new()))
                        .as_array_mut()
                        .unwrap();

                    for item in leg_pre_tool {
                        let matcher = item.get("matcher").and_then(|m| m.as_str());
                        if !canon_pre_tool
                            .iter()
                            .any(|c| c.get("matcher").and_then(|m| m.as_str()) == matcher)
                        {
                            canon_pre_tool.push(item.clone());
                        }
                    }
                }

                // Merge PreInvocation
                if let Some(leg_pre_inv) = leg_obj.get("PreInvocation").and_then(|v| v.as_array()) {
                    let canon_pre_inv = canon_obj
                        .entry("PreInvocation".to_string())
                        .or_insert_with(|| Value::Array(Vec::new()))
                        .as_array_mut()
                        .unwrap();

                    for item in leg_pre_inv {
                        let cmd = item.get("command").and_then(|c| c.as_str());
                        if !canon_pre_inv
                            .iter()
                            .any(|c| c.get("command").and_then(|c| c.as_str()) == cmd)
                        {
                            canon_pre_inv.push(item.clone());
                        }
                    }
                }

                // Copy any other custom properties from legacy
                for (k, v) in leg_obj {
                    if k != "PreToolUse"
                        && k != "PreInvocation"
                        && k != "enabled"
                        && !canon_obj.contains_key(k)
                    {
                        canon_obj.insert(k.clone(), v.clone());
                    }
                }
            }
            canonical_val
        }
        (Some(leg_key), false) => {
            let legacy = root_obj.remove(leg_key).unwrap();
            changes.push(format!(
                "Renamed legacy namespace '{leg_key}' to 'xgauntlet'"
            ));
            legacy
        }
        (None, true) => root_obj.remove("xgauntlet").unwrap(),
        (None, false) => return Ok(None),
    };

    let group_obj = match group.as_object_mut() {
        Some(obj) => obj,
        None => return Ok(None),
    };

    // Ensure enabled is true
    if !group_obj.contains_key("enabled") {
        group_obj.insert("enabled".to_string(), Value::Bool(true));
        changes.push("Set 'enabled: true' in xgauntlet hook configuration".to_string());
    }

    // 2. Modernize PreToolUse commands without altering matchers or custom tools
    if let Some(pre_tool) = group_obj
        .get_mut("PreToolUse")
        .and_then(|v| v.as_array_mut())
    {
        for entry in pre_tool.iter_mut() {
            if let Some(hooks_arr) = entry.get_mut("hooks").and_then(|h| h.as_array_mut()) {
                for hook in hooks_arr.iter_mut() {
                    if let Some(cmd) = hook.get_mut("command").and_then(|c| c.as_str()) {
                        if cmd.contains("agent_gauntlet hook antigravity")
                            || cmd.contains("agent-gauntlet hook antigravity")
                        {
                            *hook.get_mut("command").unwrap() =
                                Value::String("xgauntlet hook antigravity".to_string());
                            changes.push(
                                "Updated PreToolUse hook command to 'xgauntlet hook antigravity'"
                                    .to_string(),
                            );
                        }
                    }
                }
            }
        }
    }

    // 3. Ensure PreInvocation telemetry hook is injected and normalized to canonical format
    let pre_inv = group_obj
        .entry("PreInvocation".to_string())
        .or_insert_with(|| Value::Array(Vec::new()));

    let pre_inv_arr = match pre_inv.as_array_mut() {
        Some(arr) => arr,
        None => {
            *pre_inv = Value::Array(Vec::new());
            group_obj
                .get_mut("PreInvocation")
                .unwrap()
                .as_array_mut()
                .unwrap()
        }
    };

    let canonical_cmd = crate::features::adapters::antigravity::CANONICAL_PRE_INVOCATION_CMD;
    let mut has_telemetry = false;

    for item in pre_inv_arr.iter_mut() {
        if let Some(cmd) = item.get_mut("command") {
            if let Some(cmd_str) = cmd.as_str() {
                if cmd_str.contains("telemetry") {
                    if cmd_str != canonical_cmd {
                        *cmd = Value::String(canonical_cmd.to_string());
                        changes.push(format!(
                            "Normalized PreInvocation telemetry hook to canonical format: '{canonical_cmd}'"
                        ));
                    }
                    has_telemetry = true;
                }
            }
        } else if let Some(hooks_arr) = item.get_mut("hooks").and_then(|h| h.as_array_mut()) {
            for h in hooks_arr.iter_mut() {
                if let Some(cmd) = h.get_mut("command") {
                    if let Some(cmd_str) = cmd.as_str() {
                        if cmd_str.contains("telemetry") {
                            if cmd_str != canonical_cmd {
                                *cmd = Value::String(canonical_cmd.to_string());
                                changes.push(format!(
                                    "Normalized PreInvocation telemetry hook to canonical format: '{canonical_cmd}'"
                                ));
                            }
                            has_telemetry = true;
                        }
                    }
                }
            }
        }
    }

    if !has_telemetry {
        pre_inv_arr.push(serde_json::json!({
            "type": "command",
            "command": canonical_cmd
        }));
        changes
            .push("Injected PreInvocation telemetry hook for Antigravity response HUD".to_string());
    }

    // Re-insert modern group under "xgauntlet"
    root_obj.insert("xgauntlet".to_string(), group);

    if changes.is_empty() {
        return Ok(None);
    }

    if !dry_run {
        let formatted =
            serde_json::to_string_pretty(&root).map_err(|e| MigrationError::JsonParse {
                path: hooks_path.clone(),
                source: e,
            })?;
        fs::write(&hooks_path, format!("{formatted}\n"))?;
        files_modified.push(PathBuf::from(".agents/hooks.json"));
    }

    Ok(Some(MigrationItem {
        file: PathBuf::from(".agents/hooks.json"),
        description: "Modernize .agents/hooks.json namespace and telemetry hooks".to_string(),
        changes,
    }))
}

fn migrate_gauntlet_toml(
    workspace: &Path,
    dry_run: bool,
    files_modified: &mut Vec<PathBuf>,
) -> Result<Option<MigrationItem>, MigrationError> {
    let toml_path = workspace.join("gauntlet.toml");
    if !toml_path.is_file() {
        return Ok(None);
    }

    let content = fs::read_to_string(&toml_path)?;
    let mut changes = Vec::new();
    let mut new_content = content.clone();

    // In-place replacement of legacy evidence_file = "evidence.json"
    if content.contains("evidence_file = \"evidence.json\"") {
        new_content = new_content.replace(
            "evidence_file = \"evidence.json\"",
            "evidence_file = \"verification-report.json\"",
        );
        changes.push("Modernized evidence_file to 'verification-report.json'".to_string());
    } else if content.contains("evidence_file = 'evidence.json'") {
        new_content = new_content.replace(
            "evidence_file = 'evidence.json'",
            "evidence_file = \"verification-report.json\"",
        );
        changes.push("Modernized evidence_file to 'verification-report.json'".to_string());
    }

    // In-place modernization of redundant Rust layers
    let modernized_rust =
        modernize_rust_profile_layers_with_workspace(&new_content, Some(workspace));
    if modernized_rust != new_content {
        new_content = modernized_rust;
        changes.push(
            "Modernized Rust profile: removed redundant cargo check and optimized invariants"
                .to_string(),
        );
    }

    if changes.is_empty() {
        return Ok(None);
    }

    if !dry_run {
        fs::write(&toml_path, new_content)?;
        files_modified.push(PathBuf::from("gauntlet.toml"));
    }

    Ok(Some(MigrationItem {
        file: PathBuf::from("gauntlet.toml"),
        description: "Modernize declarative configuration schema".to_string(),
        changes,
    }))
}

fn scaffold_missing_adr_template(
    workspace: &Path,
    dry_run: bool,
    files_modified: &mut Vec<PathBuf>,
) -> Result<Option<MigrationItem>, MigrationError> {
    let adr_dir = workspace.join("docs").join("adr");
    if !adr_dir.is_dir() {
        return Ok(None);
    }

    let template_path = adr_dir.join("template.md");
    if template_path.is_file() {
        return Ok(None);
    }

    let changes = vec!["Scaffolded standard ADR template at docs/adr/template.md".to_string()];

    if !dry_run {
        fs::write(&template_path, ADR_TEMPLATE_CONTENT)?;
        files_modified.push(PathBuf::from("docs/adr/template.md"));
    }

    Ok(Some(MigrationItem {
        file: PathBuf::from("docs/adr/template.md"),
        description: "Scaffold missing architectural decision record template".to_string(),
        changes,
    }))
}

/// Modernizes Rust profile layers in gauntlet.toml: removes redundant cargo check
/// when clippy is configured and optimizes targeted invariant testing.
pub fn modernize_rust_profile_layers(toml_content: &str) -> String {
    modernize_rust_profile_layers_with_workspace(toml_content, None)
}

/// Modernizes Rust profile layers with optional workspace inspection for tests/proptests.rs existence.
pub fn modernize_rust_profile_layers_with_workspace(
    toml_content: &str,
    workspace: Option<&Path>,
) -> String {
    let mut lines: Vec<String> = toml_content.lines().map(|s| s.to_string()).collect();
    let has_clippy = lines.iter().any(|l| l.contains("clippy"));

    if has_clippy {
        // 1. Remove inline types layer only if it contains "check"
        lines.retain(|l| {
            let t = l.trim_start();
            let is_types = t.starts_with("types =") || t.starts_with("types=");
            !(is_types && t.contains("check"))
        });

        // 2. Remove table block [[layers]] with name = "types" only if command is cargo check
        let mut filtered_lines: Vec<String> = Vec::new();
        let mut in_types_block = false;
        let mut block_buffer: Vec<String> = Vec::new();

        for line in lines {
            let trimmed = line.trim();
            if trimmed == "[[layers]]" {
                if !in_types_block {
                    filtered_lines.append(&mut block_buffer);
                } else {
                    let is_cargo_check = block_buffer.iter().any(|b| b.contains("check"));
                    if !is_cargo_check {
                        filtered_lines.append(&mut block_buffer);
                    }
                    block_buffer.clear();
                    in_types_block = false;
                }
                block_buffer.push(line);
            } else if in_types_block {
                if trimmed.starts_with('[') {
                    let is_cargo_check = block_buffer.iter().any(|b| b.contains("check"));
                    if !is_cargo_check {
                        filtered_lines.append(&mut block_buffer);
                    }
                    in_types_block = false;
                    filtered_lines.push(line);
                } else {
                    block_buffer.push(line);
                }
            } else if block_buffer.iter().any(|l| l.trim() == "[[layers]]") {
                if trimmed == "name = \"types\"" || trimmed == "name = 'types'" {
                    in_types_block = true;
                    block_buffer.push(line);
                } else {
                    block_buffer.push(line);
                }
            } else {
                filtered_lines.push(line);
            }
        }
        if in_types_block {
            let is_cargo_check = block_buffer.iter().any(|b| b.contains("check"));
            if !is_cargo_check {
                filtered_lines.append(&mut block_buffer);
            }
        } else {
            filtered_lines.append(&mut block_buffer);
        }
        lines = filtered_lines;
    }

    // Only rewrite invariants if tests/proptests.rs exists in the workspace
    let should_rewrite_proptests =
        workspace.is_some_and(|ws| ws.join("tests/proptests.rs").is_file());
    if should_rewrite_proptests {
        for line in lines.iter_mut() {
            if line.contains("cargo\", \"test\", \"--\", \"proptest\"")
                || line.contains("cargo\", \"test\", \"--\", \"proptests\"")
            {
                *line = line
                    .replace(
                        "[\"cargo\", \"test\", \"--\", \"proptest\"]",
                        "[\"cargo\", \"test\", \"--test\", \"proptests\"]",
                    )
                    .replace(
                        "[\"cargo\", \"test\", \"--\", \"proptests\"]",
                        "[\"cargo\", \"test\", \"--test\", \"proptests\"]",
                    );
            }
        }
    }

    lines.join("\n")
}
