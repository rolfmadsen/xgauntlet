//! Release Readiness Gatekeeper and Manifest Harmony Subsystem.
//!
//! Provides automated verification of version harmony across manifests,
//! Keep a Changelog synchronization, and Architecture Decision Record coverage.

pub mod adr;
pub mod changelog;
pub mod engine;
pub mod manifests;
pub mod models;

pub use adr::{check_adr_coverage, AdrCoverageSummary};
pub use changelog::{parse_changelog_versions, validate_changelog};
pub use engine::{check_release_readiness, ReleaseReadinessEngine};
pub use manifests::{extract_declared_versions, ManifestVersionSummary};
pub use models::{
    ReleaseError, ReleaseFinding, ReleaseFindingCategory, ReleaseReadinessOptions,
    ReleaseReadinessReport,
};
