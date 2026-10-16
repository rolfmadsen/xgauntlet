//! Declarative Architecture Gate & Quality Boundary Enforcement.
//!
//! Enforces file line limits, test file boundaries, explicit exemptions
//! (Ratchet-model), forbidden tokens (e.g. raw styling bypassing Design Tokens),
//! and forbidden cross-boundary module imports.

pub mod checker;
pub mod models;

pub use checker::run_architecture_check;
pub use models::{
    ArchitectureConfig, ArchitectureExemption, ArchitectureFinding, ArchitectureFindingCategory,
    ArchitectureReport,
};
