//! Canonical workspace-relative path representation and security containment.
//!
//! Enforces ADR 0006 and fail-closed path authorization invariants:
//! - Blocks directory traversal escapes (`..`, `../`)
//! - Blocks absolute POSIX paths (`/etc/passwd`)
//! - Blocks Windows drive roots (`C:\...`) and UNC paths (`\\server\share`)
//! - Guarantees the sanitized path remains strictly inside the workspace root.

use std::path::{Component, Path};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PathSecurityError {
    #[error("Path contains null bytes")]
    NullByte,

    #[error("Absolute paths are prohibited in workspace authorization: '{0}'")]
    AbsolutePath(String),

    #[error("Windows drive letters or UNC paths are prohibited: '{0}'")]
    WindowsPrefix(String),

    #[error("Path traversal outside workspace root detected: '{0}'")]
    WorkspaceEscape(String),

    #[error("Path is empty or resolves to empty string")]
    EmptyPath,
}

/// A sanitized, canonical workspace-relative path guaranteed to remain strictly inside the workspace.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WorkspaceRelativePath {
    inner: String,
}

impl WorkspaceRelativePath {
    /// Sanitizes an untrusted raw path string against the canonical workspace boundary.
    pub fn sanitize(_workspace: &Path, raw: &str) -> Result<Self, PathSecurityError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(PathSecurityError::EmptyPath);
        }

        if trimmed.contains('\0') {
            return Err(PathSecurityError::NullByte);
        }

        // Convert Windows backslashes to forward slashes for cross-platform lexical inspection
        let normalized = trimmed.replace('\\', "/");

        // Reject Windows UNC paths and drive letters (e.g. "\\server", "C:", "d:/")
        if normalized.starts_with("//") {
            return Err(PathSecurityError::WindowsPrefix(raw.to_string()));
        }
        if normalized.len() >= 2 {
            let bytes = normalized.as_bytes();
            if bytes[0].is_ascii_alphabetic() && bytes[1] == b':' {
                return Err(PathSecurityError::WindowsPrefix(raw.to_string()));
            }
        }

        // Reject absolute POSIX paths
        if normalized.starts_with('/') {
            return Err(PathSecurityError::AbsolutePath(raw.to_string()));
        }

        // Lexical component normalization and escape verification
        let p = Path::new(&normalized);
        let mut stack: Vec<&str> = Vec::new();

        for component in p.components() {
            match component {
                Component::Prefix(_) => {
                    return Err(PathSecurityError::WindowsPrefix(raw.to_string()));
                }
                Component::RootDir => {
                    return Err(PathSecurityError::AbsolutePath(raw.to_string()));
                }
                Component::CurDir => {
                    // Ignore '.'
                }
                Component::ParentDir => {
                    // Attempting to pop above root is an escape attempt
                    if stack.pop().is_none() {
                        return Err(PathSecurityError::WorkspaceEscape(raw.to_string()));
                    }
                }
                Component::Normal(comp) => {
                    let s = comp.to_str().ok_or_else(|| {
                        PathSecurityError::WorkspaceEscape("invalid UTF-8 in component".into())
                    })?;
                    stack.push(s);
                }
            }
        }

        if stack.is_empty() {
            return Err(PathSecurityError::EmptyPath);
        }

        let clean_path = stack.join("/");
        Ok(Self { inner: clean_path })
    }

    /// Returns the sanitized relative path string with forward slashes.
    pub fn as_str(&self) -> &str {
        &self.inner
    }

    /// Consumes the wrapper into the inner string.
    pub fn into_inner(self) -> String {
        self.inner
    }
}

impl std::fmt::Display for WorkspaceRelativePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_relative_paths() {
        let ws = Path::new("/workspace");
        assert_eq!(
            WorkspaceRelativePath::sanitize(ws, "src/lib.rs").unwrap().as_str(),
            "src/lib.rs"
        );
        assert_eq!(
            WorkspaceRelativePath::sanitize(ws, "./tasks/001.md").unwrap().as_str(),
            "tasks/001.md"
        );
        assert_eq!(
            WorkspaceRelativePath::sanitize(ws, "docs/sub/../guide.md").unwrap().as_str(),
            "docs/guide.md"
        );
        assert_eq!(
            WorkspaceRelativePath::sanitize(ws, "src\\models\\mod.rs").unwrap().as_str(),
            "src/models/mod.rs"
        );
    }

    #[test]
    fn test_rejects_escapes_and_absolute_paths() {
        let ws = Path::new("/workspace");
        assert!(matches!(
            WorkspaceRelativePath::sanitize(ws, "../outside.rs"),
            Err(PathSecurityError::WorkspaceEscape(_))
        ));
        assert!(matches!(
            WorkspaceRelativePath::sanitize(ws, "docs/../../outside.rs"),
            Err(PathSecurityError::WorkspaceEscape(_))
        ));
        assert!(matches!(
            WorkspaceRelativePath::sanitize(ws, "/etc/passwd"),
            Err(PathSecurityError::AbsolutePath(_))
        ));
        assert!(matches!(
            WorkspaceRelativePath::sanitize(ws, "C:\\Windows\\System32"),
            Err(PathSecurityError::WindowsPrefix(_))
        ));
        assert!(matches!(
            WorkspaceRelativePath::sanitize(ws, "\\\\server\\share"),
            Err(PathSecurityError::WindowsPrefix(_))
        ));
    }
}
