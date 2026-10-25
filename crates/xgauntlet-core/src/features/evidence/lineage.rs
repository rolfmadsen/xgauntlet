//! Multi-Session Lineage & Evidence Traceability data models and chaining primitives.
//!
//! Provides a tamper-evident, append-only audit trail (`SessionLineageTrail`)
//! capturing AI engineer roles, session nonces, phase checkpoints, and Git commit OIDs
//! across isolated chat sessions.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

use super::manifest::hex;
use super::time::current_iso_utc;

/// Genesis digest used as previous digest for the initial checkpoint in a lineage chain.
pub const GENESIS_DIGEST: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// Structured errors emitted during lineage trail verification and processing.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum LineageError {
    #[error("Lineage tamper detected at entry index {index}: expected digest '{expected}', got '{actual}'")]
    TamperDetected {
        index: usize,
        expected: String,
        actual: String,
    },
    #[error("Invalid lineage entry: {0}")]
    InvalidEntry(String),
}

/// Computes a chained SHA-256 digest binding previous digest and current entry properties.
pub fn compute_lineage_entry_digest(
    prev_digest: &str,
    session_nonce: &str,
    role: &str,
    phase: &str,
    commit_oid: &str,
    timestamp: &str,
) -> String {
    let payload = format!("{prev_digest}|{session_nonce}|{role}|{phase}|{commit_oid}|{timestamp}");
    let mut hasher = Sha256::new();
    hasher.update(payload.as_bytes());
    hex::encode(hasher.finalize())
}

/// Generates a unique pseudo-random session nonce for an AI engineering session.
pub fn generate_session_nonce() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let pid = std::process::id();
    let payload = format!("xgauntlet-session-{nanos}-{pid}");
    let mut hasher = Sha256::new();
    hasher.update(payload.as_bytes());
    let hex_str = hex::encode(hasher.finalize());
    format!("sess-{}", &hex_str[..12])
}

/// A single immutable audit entry representing a recorded phase checkpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionLineageEntry {
    /// Unique session identifier.
    pub session_nonce: String,
    /// Active AI engineer role (e.g., "System Architect", "TDD Craftsman", "Code Reviewer").
    pub role: String,
    /// TDD phase at checkpoint time (e.g., "SPEC", "RED", "GREEN", "REFACTOR", "DONE").
    pub phase: String,
    /// Git commit OID (SHA-1).
    pub commit_oid: String,
    /// ISO-8601 UTC timestamp.
    pub timestamp: String,
    /// SHA-256 hash chaining of previous digest + current entry payload.
    pub digest: String,
}

/// An append-only cryptographically chained trail of session checkpoint entries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SessionLineageTrail {
    pub entries: Vec<SessionLineageEntry>,
}

impl SessionLineageTrail {
    /// Creates a new empty lineage trail.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Appends a new checkpoint entry to the trail with chained SHA-256 digest calculation.
    pub fn append(
        &mut self,
        session_nonce: impl Into<String>,
        role: impl Into<String>,
        phase: impl Into<String>,
        commit_oid: impl Into<String>,
        timestamp: Option<String>,
    ) -> SessionLineageEntry {
        let nonce = session_nonce.into();
        let r = role.into();
        let p = phase.into();
        let oid = commit_oid.into();
        let ts = timestamp.unwrap_or_else(current_iso_utc);

        let prev_digest = self
            .entries
            .last()
            .map(|e| e.digest.as_str())
            .unwrap_or(GENESIS_DIGEST);

        let digest = compute_lineage_entry_digest(prev_digest, &nonce, &r, &p, &oid, &ts);

        let entry = SessionLineageEntry {
            session_nonce: nonce,
            role: r,
            phase: p,
            commit_oid: oid,
            timestamp: ts,
            digest,
        };

        self.entries.push(entry.clone());
        entry
    }

    /// Verifies the cryptographic integrity of the entire lineage trail from Genesis to head.
    pub fn verify_integrity(&self) -> Result<(), LineageError> {
        let mut prev_digest = GENESIS_DIGEST;
        for (i, entry) in self.entries.iter().enumerate() {
            let expected = compute_lineage_entry_digest(
                prev_digest,
                &entry.session_nonce,
                &entry.role,
                &entry.phase,
                &entry.commit_oid,
                &entry.timestamp,
            );
            if entry.digest != expected {
                return Err(LineageError::TamperDetected {
                    index: i,
                    expected,
                    actual: entry.digest.clone(),
                });
            }
            prev_digest = &entry.digest;
        }
        Ok(())
    }

    /// Returns a slice of all recorded entries.
    pub fn entries(&self) -> &[SessionLineageEntry] {
        &self.entries
    }

    /// Number of recorded lineage entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the trail contains zero entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns the most recently appended lineage entry, if any.
    pub fn latest(&self) -> Option<&SessionLineageEntry> {
        self.entries.last()
    }
}
