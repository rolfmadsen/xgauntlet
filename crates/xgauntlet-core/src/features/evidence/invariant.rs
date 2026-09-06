//! Enforcement of the Self-Mutation Invariant.
//!
//! A mathematical integrity contract that verifies workspace manifests pre- and post-test
//! execution, rejecting any verification run where source files, test assertions, or configurations
//! were altered during test execution.

use super::manifest::CanonicalWorkspaceManifest;
use crate::features::diagnostics::{DiagnosticFinding, FindingType};
use thiserror::Error;

/// Violation details when workspace changes occur during verification.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub struct SelfMutationViolation {
    pub modified_files: Vec<String>,
    pub added_files: Vec<String>,
    pub removed_files: Vec<String>,
    pub pre_digest: String,
    pub post_digest: String,
}

impl std::fmt::Display for SelfMutationViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Self-mutation detected during verification (pre: {}, post: {}).",
            &self.pre_digest[..self.pre_digest.len().min(16)],
            &self.post_digest[..self.post_digest.len().min(16)],
        )?;
        if !self.modified_files.is_empty() {
            write!(f, " Modified: [{}]", self.modified_files.join(", "))?;
        }
        if !self.added_files.is_empty() {
            write!(f, " Added: [{}]", self.added_files.join(", "))?;
        }
        if !self.removed_files.is_empty() {
            write!(f, " Removed: [{}]", self.removed_files.join(", "))?;
        }
        Ok(())
    }
}

impl SelfMutationViolation {
    /// Transforms the violation into a structured diagnostic finding for LLM / AI agents.
    pub fn to_diagnostic_finding(&self) -> DiagnosticFinding {
        let mut affected = Vec::new();
        affected.extend(self.modified_files.iter().cloned());
        affected.extend(self.added_files.iter().cloned());
        affected.extend(self.removed_files.iter().cloned());

        let target_file = affected
            .first()
            .cloned()
            .unwrap_or_else(|| "workspace".to_string());

        DiagnosticFinding::new(
            FindingType::InvariantViolation,
            "xgauntlet-self-mutation",
            target_file,
            format!("{self}"),
            "Ensure tests and verification commands are purely observational and do not alter source code, test assertions, or build configurations.",
        )
    }
}

/// Verifies that no self-mutation occurred between pre- and post-execution manifests.
pub fn verify_self_mutation(
    pre: &CanonicalWorkspaceManifest,
    post: &CanonicalWorkspaceManifest,
) -> Result<(), SelfMutationViolation> {
    let mut modified_files = Vec::new();
    let mut added_files = Vec::new();
    let mut removed_files = Vec::new();

    for (path, pre_item) in &pre.items {
        match post.items.get(path) {
            Some(post_item) => {
                if pre_item.raw_content_hash != post_item.raw_content_hash
                    || pre_item.git_blob_oid != post_item.git_blob_oid
                    || pre_item.mode != post_item.mode
                {
                    modified_files.push(path.clone());
                }
            }
            None => {
                removed_files.push(path.clone());
            }
        }
    }

    for path in post.items.keys() {
        if !pre.items.contains_key(path) {
            added_files.push(path.clone());
        }
    }

    if !modified_files.is_empty()
        || !added_files.is_empty()
        || !removed_files.is_empty()
        || pre.source_manifest_digest != post.source_manifest_digest
        || pre.source_content_digest != post.source_content_digest
    {
        return Err(SelfMutationViolation {
            modified_files,
            added_files,
            removed_files,
            pre_digest: pre.source_manifest_digest.clone(),
            post_digest: post.source_manifest_digest.clone(),
        });
    }

    Ok(())
}
