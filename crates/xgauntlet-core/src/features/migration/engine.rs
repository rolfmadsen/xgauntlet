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

    // 1. Rename legacy namespace to "xgauntlet"
    let mut group = if let Some(legacy) = root_obj.remove("agent-gauntlet-gatekeeper") {
        changes.push(
            "Renamed legacy namespace 'agent-gauntlet-gatekeeper' to 'xgauntlet'".to_string(),
        );
        legacy
    } else if let Some(legacy) = root_obj.remove("gauntlet-gatekeeper") {
        changes.push("Renamed legacy namespace 'gauntlet-gatekeeper' to 'xgauntlet'".to_string());
        legacy
    } else if let Some(canonical) = root_obj.get("xgauntlet").cloned() {
        canonical
    } else {
        // No gauntlet hook configuration present
        return Ok(None);
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

    // 3. Ensure PreInvocation telemetry hook is injected
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

    let has_telemetry = pre_inv_arr.iter().any(|item| {
        let cmd = item.get("command").and_then(|c| c.as_str()).unwrap_or("");
        cmd.contains("xgauntlet telemetry") || cmd.contains("agent_gauntlet telemetry")
    });

    if !has_telemetry {
        pre_inv_arr.push(serde_json::json!({
            "command": "xgauntlet telemetry --harness antigravity-hook"
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
