//! Multi-Session Lineage & Evidence Traceability data models and chaining primitives.
//!
//! Provides a tamper-evident, append-only audit trail (`SessionLineageTrail`)
//! capturing AI engineer roles, session nonces, phase checkpoints, and Git commit OIDs
//! across isolated chat sessions.

use serde::{Deserialize, Serialize};
use thiserror::Error;

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
        _session_nonce: impl Into<String>,
        _role: impl Into<String>,
        _phase: impl Into<String>,
        _commit_oid: impl Into<String>,
        _timestamp: Option<String>,
    ) -> SessionLineageEntry {
        // RED stub: implementation pending
        panic!("RED: append not implemented yet");
    }

    /// Verifies the cryptographic integrity of the entire lineage trail from Genesis to head.
    pub fn verify_integrity(&self) -> Result<(), LineageError> {
        // RED stub: implementation pending
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
