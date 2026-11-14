//! Workspace resolution and path traversal logic for Google Antigravity.

use serde_json::Value;
use std::path::{Path, PathBuf};

/// Resolves the true workspace directory from stdin hook payload, honoring verified sibling/linked git worktrees of the same repo while rejecting unrelated repositories (ADR 0006 & ADR 0014).
pub fn resolve_effective_workspace(workspace: &Path, payload: &Value) -> PathBuf {
    crate::features::adapters::workspace::resolve_effective_workspace(workspace, payload)
}
