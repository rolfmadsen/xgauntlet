//! Evidence authority, canonical workspace manifest, and Tier 1 verification report engine.

pub mod drift;
pub mod invariant;
pub mod manifest;
pub mod models;
pub mod report;
pub mod time;

pub use drift::{verify_workspace_state_match, DriftViolation};
pub use invariant::{verify_self_mutation, SelfMutationViolation};
pub use manifest::{
    compute_digest_of_files, compute_file_git_blob_oid, compute_file_raw_hash,
    compute_workspace_manifest, is_binary_buffer, is_excluded, CanonicalWorkspaceManifest,
    ManifestError, ManifestItem, DEFAULT_SCOPES, EXCLUDED_DIRS, EXCLUDED_FILES,
};
pub use models::{
    AttestationStatus, CheckStatus, CheckSummary, ExecutionMetadata, ExecutionOrigin,
    TaskContractSummary, TrustDecision, VcsMetadata, VerificationReport, VerificationVerdict,
    WorkspaceState,
};
pub use report::{
    classify_evidence_payload, generate_report_json, generate_report_markdown, load_report_json,
    load_verification_report, save_verification_report, ReportError,
};
pub use time::current_iso_utc;
