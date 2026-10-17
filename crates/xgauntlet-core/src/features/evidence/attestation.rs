//! In-toto & Sigstore Attestation Verification Engine (ADR 0005).
//! Validates detached attestation bundles against trust policies, cryptographic digests,
//! and static repository / workflow identity constraints.

use crate::features::evidence::models::{AttestationStatus, TrustDecision, VerificationVerdict};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AttestationError {
    #[error("Failed to read artifact: {0}")]
    ArtifactRead(#[from] std::io::Error),
    #[error("Failed to parse attestation bundle JSON: {0}")]
    BundleParse(#[from] serde_json::Error),
    #[error("Invalid attestation bundle structure: {0}")]
    InvalidStructure(String),
    #[error("Base64 decode error: {0}")]
    Base64Decode(String),
}

/// Trust policy specifications for evaluating attestation validity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrustPolicy {
    /// Expected GitHub numeric repository ID (e.g. "1359184794")
    pub expected_repo_id: String,
    /// Expected repository name (e.g. "rolfmadsen/xgauntlet")
    pub expected_repo: String,
    /// Expected release workflow file path (e.g. ".github/workflows/release.yml")
    pub expected_workflow: String,
    /// Expected release git ref or tag (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_tag: Option<String>,
}

impl Default for TrustPolicy {
    fn default() -> Self {
        Self {
            expected_repo_id: "1359184794".to_string(),
            expected_repo: "rolfmadsen/xgauntlet".to_string(),
            expected_workflow: ".github/workflows/release.yml".to_string(),
            expected_tag: None,
        }
    }
}

/// Consolidated attestation verification verdict and evidence report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttestationReport {
    pub bundle_path: String,
    pub artifact_path: String,
    pub artifact_digest: String,
    pub verdict: VerificationVerdict,
    pub attestation_status: AttestationStatus,
    pub trust_decision: TrustDecision,
    pub reasons: Vec<String>,
    pub matched_subject: Option<String>,
    pub claims: BTreeMap<String, String>,
}

/// Pure base64 decoder supporting standard and URL-safe alphabets with optional padding.
pub fn decode_base64(input: &str) -> Result<Vec<u8>, AttestationError> {
    let clean: Vec<char> = input
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '=')
        .collect();

    let mut out = Vec::with_capacity((clean.len() * 3) / 4);
    let mut buffer = 0u32;
    let mut bits = 0u32;

    for &c in &clean {
        let val = match c {
            'A'..='Z' => c as u32 - 'A' as u32,
            'a'..='z' => c as u32 - 'a' as u32 + 26,
            '0'..='9' => c as u32 - '0' as u32 + 52,
            '+' | '-' => 62,
            '/' | '_' => 63,
            _ => {
                return Err(AttestationError::Base64Decode(format!(
                    "Invalid base64 character: {c}"
                )))
            }
        };

        buffer = (buffer << 6) | val;
        bits += 6;

        if bits >= 8 {
            bits -= 8;
            out.push(((buffer >> bits) & 0xFF) as u8);
        }
    }

    Ok(out)
}

/// Verifies a release artifact file against a Sigstore/in-toto attestation bundle file under a trust policy.
pub fn verify_attestation_file(
    bundle_path: &Path,
    artifact_path: &Path,
    policy: &TrustPolicy,
) -> Result<AttestationReport, AttestationError> {
    let artifact_bytes = fs::read(artifact_path).map_err(AttestationError::ArtifactRead)?;
    let mut hasher = Sha256::new();
    hasher.update(&artifact_bytes);
    let artifact_digest = format!("{:x}", hasher.finalize());

    let bundle_content = fs::read_to_string(bundle_path).map_err(AttestationError::ArtifactRead)?;
    let bundle_json: serde_json::Value =
        serde_json::from_str(&bundle_content).map_err(AttestationError::BundleParse)?;

    let mut reasons = Vec::new();
    let mut claims = BTreeMap::new();
    let mut matched_subject = None;

    // 1. Locate and decode DSSE envelope payload
    let dsse = bundle_json
        .get("dsseEnvelope")
        .or_else(|| bundle_json.get("envelope"))
        .ok_or_else(|| {
            AttestationError::InvalidStructure(
                "Missing 'dsseEnvelope' or 'envelope' in attestation bundle".to_string(),
            )
        })?;

    let payload_b64 = dsse
        .get("payload")
        .and_then(|p| p.as_str())
        .ok_or_else(|| {
            AttestationError::InvalidStructure("Missing string 'payload' in envelope".to_string())
        })?;

    let payload_bytes = decode_base64(payload_b64)?;
    let statement: serde_json::Value = serde_json::from_slice(&payload_bytes).map_err(|e| {
        AttestationError::InvalidStructure(format!("Invalid in-toto statement JSON: {e}"))
    })?;

    // 2. Extract and match subjects
    if let Some(subjects) = statement.get("subject").and_then(|s| s.as_array()) {
        for subj in subjects {
            let s_digest = subj
                .get("digest")
                .and_then(|d| d.get("sha256"))
                .and_then(|h| h.as_str());

            if let Some(s_digest) = s_digest {
                if s_digest.eq_ignore_ascii_case(&artifact_digest) {
                    matched_subject = subj
                        .get("name")
                        .and_then(|n| n.as_str())
                        .map(|s| s.to_string());
                    break;
                }
            }
        }
    }

    if matched_subject.is_none() {
        reasons.push(format!(
            "DIGEST_MISMATCH: Computed artifact digest '{artifact_digest}' was not found in any attested subject"
        ));
    }

    // 3. Extract claims from in-toto statement and certificate metadata
    let statement_str = statement.to_string();
    let bundle_str = bundle_json.to_string();

    // Repository ID check
    let found_repo_id = if statement_str.contains(&policy.expected_repo_id)
        || bundle_str.contains(&policy.expected_repo_id)
    {
        Some(policy.expected_repo_id.clone())
    } else {
        None
    };

    if let Some(repo_id) = &found_repo_id {
        claims.insert("repository_id".to_string(), repo_id.clone());
    } else {
        reasons.push(format!(
            "REPOSITORY_ID_MISMATCH: Expected repository ID '{}' not found in attestation claims",
            policy.expected_repo_id
        ));
    }

    // Repository name check
    if statement_str.contains(&policy.expected_repo) || bundle_str.contains(&policy.expected_repo) {
        claims.insert("repository".to_string(), policy.expected_repo.clone());
    } else {
        reasons.push(format!(
            "REPOSITORY_MISMATCH: Expected repository '{}' not found in attestation claims",
            policy.expected_repo
        ));
    }

    // Workflow path check
    if statement_str.contains(&policy.expected_workflow)
        || bundle_str.contains(&policy.expected_workflow)
    {
        claims.insert("workflow".to_string(), policy.expected_workflow.clone());
    } else {
        reasons.push(format!(
            "WORKFLOW_MISMATCH: Expected workflow '{}' not found in attestation claims",
            policy.expected_workflow
        ));
    }

    // Optional tag check
    if let Some(tag) = &policy.expected_tag {
        if statement_str.contains(tag) || bundle_str.contains(tag) {
            claims.insert("tag".to_string(), tag.clone());
        } else {
            reasons.push(format!(
                "TAG_MISMATCH: Expected git tag/ref '{tag}' not found in attestation claims"
            ));
        }
    }

    let is_valid = reasons.is_empty();
    let verdict = if is_valid {
        VerificationVerdict::Passed
    } else {
        VerificationVerdict::Failed
    };
    let attestation_status = if is_valid {
        AttestationStatus::Valid
    } else {
        AttestationStatus::Invalid
    };
    let trust_decision = if is_valid {
        TrustDecision::Accepted
    } else {
        TrustDecision::PolicyRejected
    };

    Ok(AttestationReport {
        bundle_path: bundle_path.to_string_lossy().to_string(),
        artifact_path: artifact_path.to_string_lossy().to_string(),
        artifact_digest,
        verdict,
        attestation_status,
        trust_decision,
        reasons,
        matched_subject,
        claims,
    })
}
