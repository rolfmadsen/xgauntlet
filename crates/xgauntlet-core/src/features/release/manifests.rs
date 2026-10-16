//! Manifest version extraction and harmony evaluation engine.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use super::models::ReleaseError;

/// Result of extracting versions across project manifest files.
#[derive(Debug, Clone)]
pub struct ManifestVersionSummary {
    /// Mapping of relative manifest path to declared version string.
    pub versions_by_source: BTreeMap<String, String>,
    /// List of relative paths of all inspected manifest files.
    pub inspected_files: Vec<String>,
}

/// Helper to parse version string and workspace inheritance from a Cargo.toml content.
pub fn parse_cargo_toml_versions(content: &str) -> (Option<String>, Option<String>, bool) {
    // Returns (workspace_package_version, package_version, inherits_workspace)
    let mut current_section = "";
    let mut ws_pkg_version = None;
    let mut pkg_version = None;
    let mut inherits_workspace = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            current_section = trimmed.trim_start_matches('[').trim_end_matches(']').trim();
            continue;
        }

        if current_section == "workspace.package" {
            if let Some(val) = extract_string_value(trimmed, "version") {
                ws_pkg_version = Some(val);
            }
        } else if current_section == "package" {
            if trimmed == "version.workspace = true" || trimmed == "version.workspace=true" {
                inherits_workspace = true;
            } else if let Some(val) = extract_string_value(trimmed, "version") {
                pkg_version = Some(val);
            }
        }
    }

    (ws_pkg_version, pkg_version, inherits_workspace)
}

/// Helper to extract string value like `key = "value"` or `key = 'value'`.
fn extract_string_value(line: &str, key: &str) -> Option<String> {
    let mut parts = line.splitn(2, '=');
    let k = parts.next()?.trim();
    if k != key {
        return None;
    }
    let val_part = parts.next()?.trim();
    if ((val_part.starts_with('"') && val_part.ends_with('"'))
        || (val_part.starts_with('\'') && val_part.ends_with('\'')))
        && val_part.len() >= 2
    {
        let inner = &val_part[1..val_part.len() - 1];
        return Some(inner.trim().to_string());
    }
    None
}

/// Helper to extract version from pyproject.toml
pub fn parse_pyproject_toml_version(content: &str) -> Option<String> {
    let mut current_section = "";
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            current_section = trimmed.trim_start_matches('[').trim_end_matches(']').trim();
            continue;
        }
        if current_section == "project"
            || current_section == "tool.poetry"
            || current_section.is_empty()
        {
            if let Some(v) = extract_string_value(trimmed, "version") {
                return Some(v);
            }
        }
    }
    None
}

/// Extracts declared versions across all supported manifest files in the given workspace.
pub fn extract_declared_versions(workspace: &Path) -> Result<ManifestVersionSummary, ReleaseError> {
    let mut versions_by_source = BTreeMap::new();
    let mut inspected_files = Vec::new();

    // 1. Root Cargo.toml
    let root_cargo = workspace.join("Cargo.toml");
    let mut root_ws_version = None;
    if root_cargo.is_file() {
        inspected_files.push("Cargo.toml".to_string());
        let content = fs::read_to_string(&root_cargo)?;
        let (ws_ver, pkg_ver, _) = parse_cargo_toml_versions(&content);
        if let Some(v) = ws_ver.clone() {
            root_ws_version = Some(v.clone());
            versions_by_source.insert("Cargo.toml".to_string(), v);
        } else if let Some(v) = pkg_ver {
            versions_by_source.insert("Cargo.toml".to_string(), v);
        }
    }

    // 2. Member Cargo.toml crates in crates/*/Cargo.toml
    let crates_dir = workspace.join("crates");
    if crates_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&crates_dir) {
            let mut crate_paths: Vec<_> = entries
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.is_dir())
                .collect();
            crate_paths.sort();

            for crate_dir in crate_paths {
                let cargo_file = crate_dir.join("Cargo.toml");
                if cargo_file.is_file() {
                    let rel_path = cargo_file
                        .strip_prefix(workspace)
                        .unwrap_or(&cargo_file)
                        .to_string_lossy()
                        .replace('\\', "/");
                    inspected_files.push(rel_path.clone());

                    let content = fs::read_to_string(&cargo_file)?;
                    let (_, pkg_ver, inherits) = parse_cargo_toml_versions(&content);

                    if inherits {
                        if let Some(ws_v) = &root_ws_version {
                            versions_by_source.insert(rel_path, ws_v.clone());
                        }
                    } else if let Some(v) = pkg_ver {
                        versions_by_source.insert(rel_path, v);
                    }
                }
            }
        }
    }

    // 3. Root package.json
    let root_pkg_json = workspace.join("package.json");
    if root_pkg_json.is_file() {
        inspected_files.push("package.json".to_string());
        if let Ok(content) = fs::read_to_string(&root_pkg_json) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(v) = val.get("version").and_then(|v| v.as_str()) {
                    if !v.trim().is_empty() {
                        versions_by_source.insert("package.json".to_string(), v.trim().to_string());
                    }
                }
            }
        }
    }

    // 4. Subpackages packages/*/package.json
    let packages_dir = workspace.join("packages");
    if packages_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&packages_dir) {
            let mut subpkg_dirs: Vec<_> = entries
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.is_dir())
                .collect();
            subpkg_dirs.sort();

            for pkg_dir in subpkg_dirs {
                let pkg_json = pkg_dir.join("package.json");
                if pkg_json.is_file() {
                    let rel_path = pkg_json
                        .strip_prefix(workspace)
                        .unwrap_or(&pkg_json)
                        .to_string_lossy()
                        .replace('\\', "/");
                    inspected_files.push(rel_path.clone());

                    if let Ok(content) = fs::read_to_string(&pkg_json) {
                        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                            if let Some(v) = val.get("version").and_then(|v| v.as_str()) {
                                if !v.trim().is_empty() {
                                    versions_by_source.insert(rel_path, v.trim().to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 5. pyproject.toml
    let pyproject = workspace.join("pyproject.toml");
    if pyproject.is_file() {
        inspected_files.push("pyproject.toml".to_string());
        let content = fs::read_to_string(&pyproject)?;
        if let Some(v) = parse_pyproject_toml_version(&content) {
            versions_by_source.insert("pyproject.toml".to_string(), v);
        }
    }

    Ok(ManifestVersionSummary {
        versions_by_source,
        inspected_files,
    })
}

/// Updates declared versions across all recognized project manifests (Cargo.toml, package.json, pyproject.toml)
/// with the specified target version string.
///
/// Returns a list of relative paths for all manifest files that were modified.
pub fn update_manifest_versions(
    workspace: &Path,
    new_version: &str,
) -> Result<Vec<String>, ReleaseError> {
    let clean_version = new_version.trim().trim_start_matches(['v', 'V']);
    let mut updated_files = Vec::new();

    // 1. Root Cargo.toml
    let root_cargo = workspace.join("Cargo.toml");
    if root_cargo.is_file() {
        let content = fs::read_to_string(&root_cargo)?;
        let mut new_lines = Vec::new();
        let mut in_target_section = false;
        let mut modified = false;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                let section = trimmed.trim_start_matches('[').trim_end_matches(']').trim();
                in_target_section = section == "workspace.package" || section == "package";
                new_lines.push(line.to_string());
                continue;
            }

            if in_target_section && trimmed.starts_with("version") {
                let mut parts = trimmed.splitn(2, '=');
                let key = parts.next().unwrap_or("").trim();
                if key == "version" {
                    let indent = line
                        .chars()
                        .take_while(|c| c.is_whitespace())
                        .collect::<String>();
                    new_lines.push(format!("{indent}version = \"{clean_version}\""));
                    modified = true;
                    continue;
                }
            }

            new_lines.push(line.to_string());
        }

        if modified {
            fs::write(&root_cargo, new_lines.join("\n") + "\n")?;
            updated_files.push("Cargo.toml".to_string());
        }
    }

    // 2. Child crates/*/Cargo.toml (only if package.version is not inheriting workspace)
    let crates_dir = workspace.join("crates");
    if crates_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(crates_dir) {
            for entry in entries.flatten() {
                let sub_cargo = entry.path().join("Cargo.toml");
                if sub_cargo.is_file() {
                    let content = fs::read_to_string(&sub_cargo)?;
                    let (_, _, inherits) = parse_cargo_toml_versions(&content);
                    if !inherits {
                        let mut new_lines = Vec::new();
                        let mut in_pkg = false;
                        let mut modified = false;

                        for line in content.lines() {
                            let trimmed = line.trim();
                            if trimmed == "[package]" {
                                in_pkg = true;
                                new_lines.push(line.to_string());
                                continue;
                            } else if trimmed.starts_with('[') && trimmed.ends_with(']') {
                                in_pkg = false;
                                new_lines.push(line.to_string());
                                continue;
                            }

                            if in_pkg && trimmed.starts_with("version") {
                                let mut parts = trimmed.splitn(2, '=');
                                let key = parts.next().unwrap_or("").trim();
                                if key == "version" {
                                    let indent = line
                                        .chars()
                                        .take_while(|c| c.is_whitespace())
                                        .collect::<String>();
                                    new_lines
                                        .push(format!("{indent}version = \"{clean_version}\""));
                                    modified = true;
                                    continue;
                                }
                            }

                            new_lines.push(line.to_string());
                        }

                        if modified {
                            fs::write(&sub_cargo, new_lines.join("\n") + "\n")?;
                            let rel = sub_cargo
                                .strip_prefix(workspace)
                                .unwrap_or(&sub_cargo)
                                .to_string_lossy()
                                .replace('\\', "/");
                            updated_files.push(rel);
                        }
                    }
                }
            }
        }
    }

    // 3. package.json and packages/*/package.json
    let mut pkg_json_targets = Vec::new();
    let root_pkg = workspace.join("package.json");
    if root_pkg.is_file() {
        pkg_json_targets.push(root_pkg);
    }
    let packages_dir = workspace.join("packages");
    if packages_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&packages_dir) {
            for entry in entries.flatten() {
                let sub_pkg = entry.path().join("package.json");
                if sub_pkg.is_file() {
                    pkg_json_targets.push(sub_pkg);
                }
            }
        }
    }

    for pkg_json in pkg_json_targets {
        let content = fs::read_to_string(&pkg_json)?;
        if let Ok(mut val) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(obj) = val.as_object_mut() {
                if obj.contains_key("version") {
                    obj.insert(
                        "version".to_string(),
                        serde_json::Value::String(clean_version.to_string()),
                    );
                    let formatted = serde_json::to_string_pretty(&val)?;
                    fs::write(&pkg_json, formatted + "\n")?;
                    let rel = pkg_json
                        .strip_prefix(workspace)
                        .unwrap_or(&pkg_json)
                        .to_string_lossy()
                        .replace('\\', "/");
                    updated_files.push(rel);
                }
            }
        }
    }

    // 4. pyproject.toml
    let pyproject = workspace.join("pyproject.toml");
    if pyproject.is_file() {
        let content = fs::read_to_string(&pyproject)?;
        let mut new_lines = Vec::new();
        let mut in_project = false;
        let mut modified = false;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                let section = trimmed.trim_start_matches('[').trim_end_matches(']').trim();
                in_project = section == "project" || section == "tool.poetry";
                new_lines.push(line.to_string());
                continue;
            }

            if in_project && trimmed.starts_with("version") {
                let mut parts = trimmed.splitn(2, '=');
                let key = parts.next().unwrap_or("").trim();
                if key == "version" {
                    let indent = line
                        .chars()
                        .take_while(|c| c.is_whitespace())
                        .collect::<String>();
                    new_lines.push(format!("{indent}version = \"{clean_version}\""));
                    modified = true;
                    continue;
                }
            }

            new_lines.push(line.to_string());
        }

        if modified {
            fs::write(&pyproject, new_lines.join("\n") + "\n")?;
            updated_files.push("pyproject.toml".to_string());
        }
    }

    Ok(updated_files)
}
