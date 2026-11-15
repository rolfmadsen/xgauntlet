//! Shared git-worktree-aware workspace resolution across all harness adapters (ADR 0006 & ADR 0014).

use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

/// Checks if a directory represents a valid xGauntlet repository or linked git worktree root.
pub fn is_valid_workspace(path: &Path) -> bool {
    path.is_dir()
        && (path.join("gauntlet.toml").is_file()
            || path.join("tasks").is_dir()
            || path.join(".agents").is_dir())
}

/// Resolves the canonical `.git` common directory for a repository root or linked `git worktree` without spawning processes.
fn resolve_git_common_dir(dir: &Path) -> Option<PathBuf> {
    let dot_git = dir.join(".git");
    if dot_git.is_dir() {
        return dot_git.canonicalize().ok();
    }
    if dot_git.is_file() {
        let content = fs::read_to_string(&dot_git).ok()?;
        let gitdir_raw = content
            .lines()
            .find_map(|line| line.trim().strip_prefix("gitdir:"))?
            .trim();
        let gitdir_path = if Path::new(gitdir_raw).is_absolute() {
            PathBuf::from(gitdir_raw)
        } else {
            dir.join(gitdir_raw)
        };
        let canonical_gitdir = gitdir_path.canonicalize().ok()?;
        let commondir_file = canonical_gitdir.join("commondir");
        if commondir_file.is_file() {
            let rel = fs::read_to_string(&commondir_file).ok()?;
            let trimmed = rel.trim();
            if !trimmed.is_empty() {
                let common = if Path::new(trimmed).is_absolute() {
                    PathBuf::from(trimmed)
                } else {
                    canonical_gitdir.join(trimmed)
                };
                if let Ok(canon) = common.canonicalize() {
                    return Some(canon);
                }
            }
        }
        return Some(canonical_gitdir);
    }
    None
}

/// Resolves the primary repository root directory from a linked `git worktree` (or the repo itself).
pub fn resolve_primary_repo_root(dir: &Path) -> Option<PathBuf> {
    let common_dir = resolve_git_common_dir(dir)?;
    if common_dir.file_name().and_then(|s| s.to_str()) == Some(".git") {
        return common_dir.parent().map(|p| p.to_path_buf());
    }
    None
}

fn is_local_clone_of(base_ws: &Path, base_common: &Path, candidate: &Path) -> bool {
    let cand_git = candidate.join(".git");
    if !cand_git.is_dir() {
        return false;
    }

    // 1. Check .git/objects/info/alternates pointing at base repo objects
    let alternates = cand_git.join("objects/info/alternates");
    if alternates.is_file() {
        if let Ok(content) = fs::read_to_string(&alternates) {
            if let Ok(base_objects) = base_common.join("objects").canonicalize() {
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.is_empty() || trimmed.starts_with('#') {
                        continue;
                    }
                    let alt_path = Path::new(trimmed);
                    let resolved = if alt_path.is_absolute() {
                        alt_path.to_path_buf()
                    } else {
                        cand_git.join("objects").join(alt_path)
                    };
                    if let Ok(canon) = resolved.canonicalize() {
                        if canon == base_objects {
                            return true;
                        }
                    }
                }
            }
        }
    }

    // 2. Check .git/config [remote "origin"] url matching canonicalized base_ws
    let git_config = cand_git.join("config");
    if git_config.is_file() {
        if let (Ok(content), Ok(canon_base)) =
            (fs::read_to_string(&git_config), base_ws.canonicalize())
        {
            let mut in_origin = false;
            for raw_line in content.lines() {
                let line = raw_line.trim();
                if line.starts_with('[') && line.ends_with(']') {
                    in_origin = line.eq_ignore_ascii_case("[remote \"origin\"]");
                    continue;
                }
                if in_origin {
                    if let Some((k, v)) = line.split_once('=') {
                        if k.trim().eq_ignore_ascii_case("url") {
                            let url_raw = v.trim();
                            let url_path_str = url_raw.strip_prefix("file://").unwrap_or(url_raw);
                            let url_path = Path::new(url_path_str);
                            if let Ok(canon_url) = url_path.canonicalize() {
                                if canon_url == canon_base {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    false
}

/// Verifies that `candidate` belongs to the exact same Git repository as `base_ws`
/// (primary repo, linked git worktree, or local `Workspace: "branch"` clone of `base_ws`).
pub fn is_same_git_repo_worktree(base_ws: &Path, candidate: &Path) -> bool {
    match (
        resolve_git_common_dir(base_ws),
        resolve_git_common_dir(candidate),
    ) {
        (Some(base_common), Some(cand_common)) => {
            base_common == cand_common || is_local_clone_of(base_ws, &base_common, candidate)
        }
        _ => false,
    }
}

fn collect_payload_candidates(payload: &Value) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(cwd_str) = payload.get("cwd").and_then(|v| v.as_str()) {
        if !cwd_str.trim().is_empty() {
            candidates.push(PathBuf::from(cwd_str));
        }
    }
    if let Some(arr) = payload.get("workspacePaths").and_then(|v| v.as_array()) {
        for item in arr {
            if let Some(ws_str) = item.as_str() {
                if !ws_str.trim().is_empty() {
                    candidates.push(PathBuf::from(ws_str));
                }
            }
        }
    }
    candidates
}

/// Resolves the effective workspace directory from stdin hook payload across all harnesses.
///
/// Security contract (ADR 0006 + ADR 0014):
/// - If the hook process CWD (`workspace`) is already a valid workspace, unrelated repositories in `cwd`/`workspacePaths` are NEVER trusted.
/// - However, if the payload carries a verified `git worktree` sharing the exact same Git common directory as `workspace`, that worktree becomes the effective workspace.
pub fn resolve_effective_workspace(workspace: &Path, payload: &Value) -> PathBuf {
    if is_valid_workspace(workspace) {
        let canon_base = workspace.canonicalize().ok();
        for cand in collect_payload_candidates(payload) {
            if !is_valid_workspace(&cand) {
                continue;
            }
            if let (Some(ref b), Ok(c)) = (&canon_base, cand.canonicalize()) {
                if b == &c {
                    return workspace.to_path_buf();
                }
            }
            if is_same_git_repo_worktree(workspace, &cand) {
                return cand;
            }
        }
        return workspace.to_path_buf();
    }

    // Fallback when hook was invoked from an unconfigured parent directory (e.g. $HOME)
    if let Some(arr) = payload.get("workspacePaths").and_then(|v| v.as_array()) {
        for item in arr {
            if let Some(ws_str) = item.as_str() {
                let p = PathBuf::from(ws_str);
                if p.is_dir() {
                    return p;
                }
            }
        }
    }

    if let Some(cwd_str) = payload.get("cwd").and_then(|v| v.as_str()) {
        let p = PathBuf::from(cwd_str);
        if is_valid_workspace(&p) {
            return p;
        }
    }

    workspace.to_path_buf()
}
