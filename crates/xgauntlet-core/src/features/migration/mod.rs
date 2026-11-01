//! In-repo schema migration and doctor auto-remediation feature module.

pub mod engine;
pub mod models;

pub use engine::{modernize_rust_profile_layers, run_migration};
pub use models::{MigrationError, MigrationItem, MigrationOptions, MigrationReport};
