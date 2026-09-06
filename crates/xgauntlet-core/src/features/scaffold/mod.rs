//! Safe, non-destructive project bootstrap engine.
//!
//! Provides initial workspace scaffolding for supported programming stacks (Rust,
//! Python, Node/TypeScript, Go) without overwriting existing governance files.

pub mod engine;
pub mod models;
pub mod templates;

pub use engine::run_scaffold;
pub use models::{
    ScaffoldAction, ScaffoldError, ScaffoldFileReport, ScaffoldOptions, ScaffoldResult,
};
pub use templates::{generate_templates, ScaffoldTemplate};
