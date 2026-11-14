//! Transactional archiving of completed tasks during release preparation.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use super::models::ReleaseError;
use crate::features::tasks::{TaskArchiveOptions, TaskArchiveResult, TaskScaffolder};

/// Captures staged task archive transitions and spec backups for transactional release rollback.
#[derive(Debug, Clone)]
pub struct ReleaseArchiveStaging {
    archive_result: TaskArchiveResult,
    spec_backups: BTreeMap<PathBuf, String>,
    archive_dir: PathBuf,
    archive_dir_existed: bool,
}

impl ReleaseArchiveStaging {
    /// Returns the underlying task archive summary.
    pub fn archive_result(&self) -> &TaskArchiveResult {
        &self.archive_result
    }

    /// Returns unique workspace-relative paths touched by the archive step (old task path, new archive path, and updated specs).
    pub fn touched_relative_paths(&self, workspace: &Path) -> Vec<String> {
        let mut unique = BTreeSet::new();
        for entry in &self.archive_result.archived {
            for p in [&entry.from_path, &entry.to_path] {
                let rel = p
                    .strip_prefix(workspace)
                    .unwrap_or(p)
                    .to_string_lossy()
                    .replace('\\', "/");
                let clean = rel.trim_start_matches("./").to_string();
                unique.insert(clean);
            }
            for spec in &entry.updated_specs {
                unique.insert(spec.trim_start_matches("./").to_string());
            }
        }
        unique.into_iter().collect()
    }

    /// Restores updated specs, moves archived task files back to their original paths, and removes an empty newly-created `tasks/archive/` directory.
    pub fn rollback(&self) {
        if self.archive_result.dry_run {
            return;
        }
        for (spec_path, original) in &self.spec_backups {
            let _ = fs::write(spec_path, original);
        }
        for entry in self.archive_result.archived.iter().rev() {
            if entry.to_path.is_file() {
                if let Some(parent) = entry.from_path.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                let _ = fs::rename(&entry.to_path, &entry.from_path);
            }
        }
        if !self.archive_dir_existed && self.archive_dir.is_dir() {
            let _ = fs::remove_dir(&self.archive_dir);
        }
    }
}

/// Archives all `DONE`/`COMPLETED` tasks in `tasks/` before release evidence sealing, recording backups for rollback.
pub fn stage_done_tasks_for_release(
    workspace: &Path,
    dry_run: bool,
    no_archive: bool,
) -> Result<ReleaseArchiveStaging, ReleaseError> {
    let archive_dir = workspace.join("tasks/archive");
    let archive_dir_existed = archive_dir.is_dir();

    if no_archive || !workspace.join("tasks").is_dir() {
        return Ok(ReleaseArchiveStaging {
            archive_result: TaskArchiveResult {
                archived: Vec::new(),
                skipped_count: 0,
                dry_run,
            },
            spec_backups: BTreeMap::new(),
            archive_dir,
            archive_dir_existed,
        });
    }

    let preview = TaskScaffolder::archive_tasks(&TaskArchiveOptions {
        workspace: workspace.to_path_buf(),
        task_id: None,
        all_done: true,
        dry_run: true,
    })
    .map_err(|e| ReleaseError::General(format!("Task archive preview failed: {e}")))?;

    if dry_run || preview.archived.is_empty() {
        return Ok(ReleaseArchiveStaging {
            archive_result: TaskArchiveResult { dry_run, ..preview },
            spec_backups: BTreeMap::new(),
            archive_dir,
            archive_dir_existed,
        });
    }

    let mut spec_backups = BTreeMap::new();
    for entry in &preview.archived {
        for rel_spec in &entry.updated_specs {
            let full_spec = workspace.join(rel_spec);
            if !spec_backups.contains_key(&full_spec) && full_spec.is_file() {
                if let Ok(content) = fs::read_to_string(&full_spec) {
                    spec_backups.insert(full_spec, content);
                }
            }
        }
    }

    let archive_result = TaskScaffolder::archive_tasks(&TaskArchiveOptions {
        workspace: workspace.to_path_buf(),
        task_id: None,
        all_done: true,
        dry_run: false,
    })
    .map_err(|e| ReleaseError::General(format!("Task archive execution failed: {e}")))?;

    Ok(ReleaseArchiveStaging {
        archive_result,
        spec_backups,
        archive_dir,
        archive_dir_existed,
    })
}

/// Collects release manifest and evidence candidate paths for transactional staging backup.
pub fn collect_release_backup_candidates(workspace: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for name in [
        "Cargo.toml",
        "Cargo.lock",
        "CHANGELOG.md",
        "package.json",
        "crates/xgauntlet-core/wasm/gauntlet_policy.wasm",
        "verification-report.json",
        "evidence.md",
    ] {
        let p = workspace.join(name);
        if p.is_file() {
            files.push(p);
        }
    }

    for (dir_name, file_name) in [("crates", "Cargo.toml"), ("packages", "package.json")] {
        let dir = workspace.join(dir_name);
        if dir.is_dir() {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let candidate = entry.path().join(file_name);
                    if candidate.is_file() {
                        files.push(candidate);
                    }
                }
            }
        }
    }

    files
}
