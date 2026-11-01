//! Workspace resolution and path traversal logic for Google Antigravity.

use serde_json::Value;
use std::path::{Path, PathBuf};

/// Resolves the true workspace directory from stdin hook payload, falling back to workspace or path ancestor traversal.
pub fn resolve_effective_workspace(workspace: &Path, payload: &Value) -> PathBuf {
    // 1. Antigravity common field: workspacePaths: ["/path/to/project"]
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

    // 2. Antigravity / generic cwd field
    if let Some(cwd_str) = payload.get("cwd").and_then(|v| v.as_str()) {
        let p = PathBuf::from(cwd_str);
        if p.is_dir() && (p.join("tasks").is_dir() || p.join("gauntlet.toml").is_file()) {
            return p;
        }
    }

    // 3. Fallback: inspect target_resource from toolCall if present
    let target = payload
        .get("toolCall")
        .and_then(|t| t.get("args"))
        .and_then(|args| {
            args.get("Path")
                .or_else(|| args.get("path"))
                .or_else(|| args.get("file_path"))
                .or_else(|| args.get("filePath"))
        })
        .and_then(|v| v.as_str());

    if let Some(target_path_str) = target {
        let target_path = Path::new(target_path_str);
        if target_path.is_absolute() {
            let mut current = target_path.parent();
            while let Some(parent) = current {
                if parent.join("tasks").is_dir()
                    || parent.join("gauntlet.toml").is_file()
                    || parent.join(".agents").is_dir()
                    || parent.join(".git").is_dir()
                {
                    return parent.to_path_buf();
                }
                current = parent.parent();
            }
        }
    }

    workspace.to_path_buf()
}
