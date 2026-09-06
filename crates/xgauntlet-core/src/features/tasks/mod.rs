//! Task & Spec Engine feature module.
//!
//! Provides OKF v0.2 frontmatter parsing, task package loading,
//! Aristotelian domain glossary verification, and check-spec gatekeeping.

pub mod models;
pub mod okf;
pub mod parser;
pub mod validator;

pub use models::{SpecReadinessReport, TaskContract, TaskPackageInfo, TaskStatus};
pub use okf::{
    parse_frontmatter, validate_iso_timestamp, Actor, GeneratedEntry, OkfError, OkfMetadata,
    SourceEntry, VerifiedEntry,
};
pub use parser::{
    has_active_task, is_task_active, parse_task_content, parse_task_file, parse_task_status,
    resolve_active_task_id, resolve_task_contract, TaskError,
};
pub use validator::{
    check_all_tasks, check_task_specification, validate_context_content, validate_context_glossary,
};
