//! Plugin installation engine.

use std::fs;
use std::path::{Path, PathBuf};

use super::bundle::{
    get_embedded_companion_files, get_embedded_hooks_manifest, get_embedded_plugin_manifest,
    get_embedded_skill, ALL_EMBEDDED_SKILLS,
};
use super::discovery::discover_installed_harnesses;
use super::models::{
    PluginError, PluginInstallOptions, PluginInstallReport, PluginInstallStatus,
    PluginTargetInstallReport,
};

/// Executes plugin installation into target directories or global harnesses.
pub fn run_plugin_install(
    options: &PluginInstallOptions,
) -> Result<PluginInstallReport, PluginError> {
    let mut targets_to_install = Vec::new();

    if let Some(ref target) = options.target {
        let harness_name = options
            .harness
            .clone()
            .unwrap_or_else(|| "custom".to_string());
        targets_to_install.push((harness_name, target.clone()));
    } else {
        let discovered = discover_installed_harnesses();
        if options.global {
            if let Some(ref target_harness) = options.harness {
                let matched = discovered
                    .into_iter()
                    .filter(|h| h.name.eq_ignore_ascii_case(target_harness))
                    .collect::<Vec<_>>();
                if matched.is_empty() {
                    return Err(PluginError::UnknownHarness(target_harness.clone()));
                }
                for h in matched {
                    targets_to_install.push((h.name, h.plugin_dir));
                }
            } else {
                for h in discovered {
                    targets_to_install.push((h.name, h.plugin_dir));
                }
            }
        } else if let Some(ref target_harness) = options.harness {
            let matched = discovered
                .into_iter()
                .find(|h| h.name.eq_ignore_ascii_case(target_harness));
            if let Some(h) = matched {
                targets_to_install.push((h.name, h.plugin_dir));
            } else {
                return Err(PluginError::UnknownHarness(target_harness.clone()));
            }
        } else {
            // Default to Antigravity global plugin directory
            let home = std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("."));
            let plugin_dir = home.join(".gemini/config/plugins/xgauntlet");
            targets_to_install.push(("antigravity".to_string(), plugin_dir));
        }
    }

    let mut target_reports = Vec::new();

    for (harness, target_dir) in targets_to_install {
        let report = install_into_directory(&harness, &target_dir, options)?;
        target_reports.push(report);
    }

    Ok(PluginInstallReport {
        targets: target_reports,
        success: true,
        is_dry_run: options.dry_run,
    })
}

fn install_into_directory(
    harness: &str,
    target_dir: &Path,
    options: &PluginInstallOptions,
) -> Result<PluginTargetInstallReport, PluginError> {
    let mut files_written = Vec::new();
    let mut files_skipped = Vec::new();

    // Prepare list of relative paths and contents
    let mut assets: Vec<(PathBuf, String)> = Vec::new();
    assets.push((
        PathBuf::from("plugin.json"),
        get_embedded_plugin_manifest().to_string(),
    ));
    assets.push((
        PathBuf::from("hooks.json"),
        get_embedded_hooks_manifest().to_string(),
    ));

    for skill in ALL_EMBEDDED_SKILLS {
        if let Some(skill_content) = get_embedded_skill(skill) {
            let rel_path = PathBuf::from(format!("skills/{skill}/SKILL.md"));
            assets.push((rel_path, skill_content.to_string()));
        }
    }

    for (skill, filename, companion_content) in get_embedded_companion_files() {
        let rel_path = PathBuf::from(format!("skills/{skill}/{filename}"));
        assets.push((rel_path, companion_content.to_string()));
    }

    for (rel_path, content) in assets {
        let dest_path = target_dir.join(&rel_path);
        let exists = dest_path.is_file();

        if exists && !options.force {
            files_skipped.push(dest_path);
        } else {
            if !options.dry_run {
                if let Some(parent) = dest_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(&dest_path, content)?;
            }
            files_written.push(dest_path);
        }
    }

    let status = if options.dry_run {
        PluginInstallStatus::DryRunSimulated
    } else if !files_written.is_empty() {
        PluginInstallStatus::Installed
    } else {
        PluginInstallStatus::UpToDate
    };

    Ok(PluginTargetInstallReport {
        harness: harness.to_string(),
        target_dir: target_dir.to_path_buf(),
        status,
        files_written,
        files_skipped,
    })
}
