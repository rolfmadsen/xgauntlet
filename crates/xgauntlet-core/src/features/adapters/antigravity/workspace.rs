//! Workspace resolution and path traversal logic for Google Antigravity.

use serde_json::Value;
use std::path::{Path, PathBuf};

/// Checks if a directory represents a valid repository workspace root.
fn is_valid_workspace(path: &Path) -> bool {
    path.is_dir()
        && (path.join("gauntlet.toml").is_file()
            || path.join("tasks").is_dir()
            || path.join(".agents").is_dir())
}

/// Resolves the true workspace directory from stdin hook payload, preserving a valid workspace or falling back to workspacePaths/cwd.
pub fn resolve_effective_workspace(workspace: &Path, payload: &Value) -> PathBuf {
    // 1. If the process workspace is already a valid workspace root, preserve it unconditionally (ADR 0006)
    if is_valid_workspace(workspace) {
        return workspace.to_path_buf();
    }

    // 2. Fallback: Antigravity envelope field workspacePaths: ["/path/to/project"]
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

    // 3. Fallback: Antigravity / generic cwd field
    if let Some(cwd_str) = payload.get("cwd").and_then(|v| v.as_str()) {
        let p = PathBuf::from(cwd_str);
        if is_valid_workspace(&p) {
            return p;
        }
    }

    workspace.to_path_buf()
}
