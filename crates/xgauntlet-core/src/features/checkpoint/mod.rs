//! Phase-Bound TDD Checkpoint Engine (ADR 0001, ADR 0003).
//!
//! Provides mechanical, fail-closed pre-flight validation for each phase
//! of the TDD lifecycle and generates clean conventional Git commits.

pub mod engine;
pub mod git;
pub mod models;
pub mod preflight;
pub mod provenance;

pub use engine::run_checkpoint;
pub use git::{
    compose_commit_message, execute_git_commit, extract_short_task_id, stage_workspace_changes,
};
pub use models::{
    CheckpointError, CheckpointOptions, CheckpointPhase, CheckpointResult, SessionHandoff,
};
pub use preflight::run_preflight_check;
pub use provenance::{
    collect_task_test_provenance, detect_born_green_tests, extract_test_identifiers,
    verify_inverse_test_failure, TestProvenanceEvidence, TestProvenanceStatus,
};
