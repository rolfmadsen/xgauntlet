//! Deterministic, fail-closed canonical workspace manifest and state binding.

use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use thiserror::Error;

use super::models::{VcsMetadata, WorkspaceState};

pub const DEFAULT_SCOPES: &[&str] = &[
    "crates",
    "packages",
    "src",
    "tests",
    "wit",
    "docs",
    "tasks",
    "tools",
    "plugins",
    "spec.md",
    "README.md",
    "Cargo.toml",
    "package.json",
    "gauntlet.toml",
];

pub const EXCLUDED_DIRS: &[&str] = &[
    ".git",
    ".hypothesis",
    ".idea",
    ".mypy_cache",
    ".pytest_cache",
    ".ruff_cache",
    ".venv",
    ".vscode",
    "__pycache__",
    "target",
    "node_modules",
    "dist",
    ".cache",
];

pub const EXCLUDED_FILES: &[&str] = &[
    ".DS_Store",
    ".coverage",
    "attestation.bundle",
    "coverage.xml",
    "evidence.json",
    "evidence.md",
    "verification-report.json",
];

#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("Symlink '{0}' resolves outside workspace root: target='{1}', resolved='{2}'")]
    WorkspaceEscape(String, String, String),

    #[error("Path contains invalid UTF-8 encoding: '{0}'")]
    InvalidPathEncoding(String),

    #[error("Failed to read file '{0}': {1}")]
    FileReadError(String, String),

    #[error("Failed to canonicalize path '{0}': {1}")]
    CanonicalizationError(String, String),
}

/// Metadata item for a single file in the canonical workspace manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestItem {
    pub relative_path: String,
    pub raw_content_hash: String,
    pub git_blob_oid: String,
    pub mode: String,
}

/// Canonical workspace manifest capturing reproducible state and multi-digests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalWorkspaceManifest {
    pub files: Vec<String>,
    pub included_files_count: usize,
    pub source_content_digest: String,
    pub source_manifest_digest: String,
    pub config_digest: String,
    pub task_digest: String,
    pub policy_digest: String,
    pub check_definitions_digest: String,
    pub vcs: Option<VcsMetadata>,
    pub items: BTreeMap<String, ManifestItem>,
}

impl CanonicalWorkspaceManifest {
    pub fn to_workspace_state(
        &self,
        pre_digest: Option<&str>,
        check_definitions_digest: Option<&str>,
    ) -> WorkspaceState {
        WorkspaceState {
            manifest_version: "1.0".to_string(),
            source_content_digest: self.source_content_digest.clone(),
            source_manifest_digest_pre: pre_digest
                .map(|s| s.to_string())
                .unwrap_or_else(|| self.source_manifest_digest.clone()),
            source_manifest_digest_post: self.source_manifest_digest.clone(),
            config_digest: self.config_digest.clone(),
            task_digest: self.task_digest.clone(),
            policy_digest: self.policy_digest.clone(),
            check_definitions_digest: check_definitions_digest
                .map(|s| s.to_string())
                .unwrap_or_else(|| self.check_definitions_digest.clone()),
            included_files_count: self.included_files_count,
            vcs: self.vcs.clone(),
        }
    }
}

/// Determines whether a byte buffer represents binary content by inspecting for null bytes.
pub fn is_binary_buffer(bytes: &[u8]) -> bool {
    let limit = bytes.len().min(8192);
    bytes[..limit].contains(&0x00)
}

/// Computes the Git-blob OID SHA-256 for a file buffer, normalizing CRLF to LF for text.
pub fn compute_file_git_blob_oid(bytes: &[u8]) -> String {
    let normalized: Vec<u8> = if is_binary_buffer(bytes) {
        bytes.to_vec()
    } else {
        // Text normalization: convert \r\n -> \n
        let mut out = Vec::with_capacity(bytes.len());
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'\r' && i + 1 < bytes.len() && bytes[i + 1] == b'\n' {
                out.push(b'\n');
                i += 2;
            } else {
                out.push(bytes[i]);
                i += 1;
            }
        }
        out
    };

    let mut hasher = Sha256::new();
    let header = format!("blob {}\0", normalized.len());
    hasher.update(header.as_bytes());
    hasher.update(&normalized);
    hex::encode(hasher.finalize())
}

/// Computes raw SHA-256 digest of exact file bytes.
pub fn compute_file_raw_hash(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

/// Check if a relative path matches exclusion rules.
pub fn is_excluded(rel_path: &Path) -> bool {
    for comp in rel_path.components() {
        let s = comp.as_os_str().to_string_lossy();
        if EXCLUDED_DIRS.contains(&s.as_ref()) || s.ends_with(".egg-info") {
            return true;
        }
    }

    if let Some(file_name) = rel_path.file_name() {
        let name_str = file_name.to_string_lossy();
        if EXCLUDED_FILES.contains(&name_str.as_ref()) || name_str.ends_with(".pyc") {
            return true;
        }
    }

    false
}

#[cfg(unix)]
fn get_file_mode(path: &Path, is_symlink: bool) -> String {
    use std::os::unix::fs::PermissionsExt;
    if is_symlink {
        return "120000".to_string();
    }
    match fs::symlink_metadata(path) {
        Ok(meta) => {
            let mode = meta.permissions().mode();
            let is_exec = (mode & 0o111) != 0;
            if is_exec {
                "100755".to_string()
            } else {
                "100644".to_string()
            }
        }
        Err(_) => "100644".to_string(),
    }
}

#[cfg(not(unix))]
fn get_file_mode(path: &Path, is_symlink: bool) -> String {
    if is_symlink {
        return "120000".to_string();
    }
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        if matches!(ext, "sh" | "cmd" | "bat" | "exe") {
            return "100755".to_string();
        }
    }
    "100644".to_string()
}

/// Computes length-prefixed SHA-256 digest over a sequence of files.
pub fn compute_digest_of_files(root: &Path, files: &[PathBuf]) -> String {
    let mut sorted_files = files.to_vec();
    sorted_files.sort();

    let mut hasher = Sha256::new();
    for f in sorted_files {
        if !f.is_file() {
            continue;
        }
        if let Ok(rel) = f.strip_prefix(root) {
            if is_excluded(rel) {
                continue;
            }
            if let Ok(data) = fs::read(&f) {
                let rel_str = rel.to_string_lossy().replace('\\', "/");
                let rel_bytes = rel_str.as_bytes();
                hasher.update((rel_bytes.len() as u64).to_be_bytes());
                hasher.update(rel_bytes);
                hasher.update((data.len() as u64).to_be_bytes());
                hasher.update(&data);
            }
        }
    }
    hex::encode(hasher.finalize())
}

/// Computes the deterministic canonical workspace manifest.
pub fn compute_workspace_manifest(
    root: &Path,
    scopes: Option<&[&str]>,
) -> Result<CanonicalWorkspaceManifest, ManifestError> {
    let canonical_root = root.canonicalize().map_err(|e| {
        ManifestError::CanonicalizationError(root.display().to_string(), e.to_string())
    })?;

    let target_scopes = scopes.unwrap_or(DEFAULT_SCOPES);
    let mut items_map = BTreeMap::new();

    for scope in target_scopes {
        let candidate = canonical_root.join(scope);
        let Ok(meta) = fs::symlink_metadata(&candidate) else {
            continue;
        };

        if meta.file_type().is_symlink() || meta.is_file() {
            collect_file(&canonical_root, &candidate, &mut items_map)?;
        } else if meta.is_dir() {
            collect_dir_recursive(&canonical_root, &candidate, &mut items_map)?;
        }
    }

    let mut file_list = Vec::new();
    let mut manifest_hasher = Sha256::new();
    let mut content_hasher = Sha256::new();

    for (rel_path, item) in &items_map {
        file_list.push(rel_path.clone());
        let path_bytes = rel_path.as_bytes();

        // Length-prefixed manifest digest (Git OID + mode + relative path)
        manifest_hasher.update(item.git_blob_oid.as_bytes());
        manifest_hasher.update(item.mode.as_bytes());
        manifest_hasher.update((path_bytes.len() as u32).to_be_bytes());
        manifest_hasher.update(path_bytes);

        // Length-prefixed raw content digest
        content_hasher.update(item.raw_content_hash.as_bytes());
        content_hasher.update((path_bytes.len() as u32).to_be_bytes());
        content_hasher.update(path_bytes);
    }

    let source_manifest_digest = hex::encode(manifest_hasher.finalize());
    let source_content_digest = hex::encode(content_hasher.finalize());

    // Auxiliary digests
    let config_files = vec![
        canonical_root.join("gauntlet.toml"),
        canonical_root.join("Cargo.toml"),
        canonical_root.join("package.json"),
    ];
    let config_digest = compute_digest_of_files(&canonical_root, &config_files);

    let mut task_files = Vec::new();
    let task_dir = canonical_root.join("tasks");
    if task_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&task_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() && p.extension().is_some_and(|e| e == "md") {
                    task_files.push(p);
                }
            }
        }
    }
    let task_digest = compute_digest_of_files(&canonical_root, &task_files);

    let mut policy_files = vec![
        canonical_root.join("spec.md"),
        canonical_root.join("CONTEXT.md"),
        canonical_root.join(".agents").join("AGENTS.md"),
        canonical_root.join(".agents").join("hooks.json"),
    ];
    let adr_dir = canonical_root.join("docs").join("adr");
    if adr_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&adr_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() && p.extension().is_some_and(|e| e == "md") {
                    policy_files.push(p);
                }
            }
        }
    }
    let workflows_dir = canonical_root.join(".github").join("workflows");
    if workflows_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&workflows_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() && p.extension().is_some_and(|e| e == "yml" || e == "yaml") {
                    policy_files.push(p);
                }
            }
        }
    }
    let policy_digest = compute_digest_of_files(&canonical_root, &policy_files);

    // Git probe
    let vcs = probe_git(&canonical_root);

    Ok(CanonicalWorkspaceManifest {
        files: file_list.clone(),
        included_files_count: file_list.len(),
        source_content_digest,
        source_manifest_digest,
        config_digest,
        task_digest,
        policy_digest,
        check_definitions_digest: String::new(),
        vcs,
        items: items_map,
    })
}

fn collect_dir_recursive(
    root: &Path,
    dir: &Path,
    items: &mut BTreeMap<String, ManifestItem>,
) -> Result<(), ManifestError> {
    let entries = fs::read_dir(dir)
        .map_err(|e| ManifestError::FileReadError(dir.display().to_string(), e.to_string()))?;

    for entry in entries.flatten() {
        let path = entry.path();
        let meta = match fs::symlink_metadata(&path) {
            Ok(m) => m,
            Err(_) => continue,
        };

        if meta.file_type().is_symlink() || meta.is_file() {
            collect_file(root, &path, items)?;
        } else if meta.is_dir() {
            if let Ok(rel) = path.strip_prefix(root) {
                if is_excluded(rel) {
                    continue;
                }
            }
            collect_dir_recursive(root, &path, items)?;
        }
    }
    Ok(())
}

fn collect_file(
    root: &Path,
    path: &Path,
    items: &mut BTreeMap<String, ManifestItem>,
) -> Result<(), ManifestError> {
    let rel_path = path.strip_prefix(root).map_err(|_| {
        ManifestError::WorkspaceEscape(
            path.display().to_string(),
            String::new(),
            root.display().to_string(),
        )
    })?;

    if is_excluded(rel_path) {
        return Ok(());
    }

    let rel_str = rel_path
        .to_str()
        .ok_or_else(|| ManifestError::InvalidPathEncoding(rel_path.display().to_string()))?
        .replace('\\', "/");

    let meta = fs::symlink_metadata(path)
        .map_err(|e| ManifestError::FileReadError(path.display().to_string(), e.to_string()))?;

    let is_symlink = meta.file_type().is_symlink();
    let mode = get_file_mode(path, is_symlink);

    let (raw_content_hash, git_blob_oid) = if is_symlink {
        let link_target = fs::read_link(path)
            .map_err(|e| ManifestError::FileReadError(path.display().to_string(), e.to_string()))?;

        // Canonical resolution to check workspace escape
        let resolved = path.canonicalize().map_err(|_| {
            ManifestError::WorkspaceEscape(
                rel_str.clone(),
                link_target.display().to_string(),
                "unresolvable".to_string(),
            )
        })?;

        if !resolved.starts_with(root) {
            return Err(ManifestError::WorkspaceEscape(
                rel_str,
                link_target.display().to_string(),
                resolved.display().to_string(),
            ));
        }

        let target_str = link_target.to_string_lossy().replace('\\', "/");
        let raw_hash = compute_file_raw_hash(target_str.as_bytes());
        let blob_oid = compute_file_git_blob_oid(target_str.as_bytes());
        (raw_hash, blob_oid)
    } else {
        let bytes = fs::read(path)
            .map_err(|e| ManifestError::FileReadError(path.display().to_string(), e.to_string()))?;
        let raw_hash = compute_file_raw_hash(&bytes);
        let blob_oid = compute_file_git_blob_oid(&bytes);
        (raw_hash, blob_oid)
    };

    items.insert(
        rel_str.clone(),
        ManifestItem {
            relative_path: rel_str,
            raw_content_hash,
            git_blob_oid,
            mode,
        },
    );

    Ok(())
}

fn probe_git(root: &Path) -> Option<VcsMetadata> {
    if !root.join(".git").exists() {
        return None;
    }

    let head_output = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .current_dir(root)
        .output()
        .ok()?;

    if !head_output.status.success() {
        return None;
    }
    let head = String::from_utf8_lossy(&head_output.stdout)
        .trim()
        .to_string();

    let commit_output = Command::new("git")
        .args(["log", "-1", "--format=%h"])
        .current_dir(root)
        .output()
        .ok();

    let commit = commit_output
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| head.clone());

    let status_output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(root)
        .output()
        .ok();

    let is_dirty = status_output
        .filter(|o| o.status.success())
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false);

    Some(VcsMetadata {
        vcs_type: "git".to_string(),
        head,
        commit,
        is_dirty,
    })
}

mod hex {
    pub fn encode<T: AsRef<[u8]>>(data: T) -> String {
        let bytes = data.as_ref();
        let mut s = String::with_capacity(bytes.len() * 2);
        for &b in bytes {
            use std::fmt::Write;
            let _ = write!(s, "{:02x}", b);
        }
        s
    }
}
