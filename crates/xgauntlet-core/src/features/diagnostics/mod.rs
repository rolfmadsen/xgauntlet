//! Diagnostics feature module providing structured findings and reports.

pub mod extractors;
pub mod models;
pub mod parser;

pub use extractors::*;
pub use models::{DiagnosticFinding, DiagnosticReport, FindingType};
pub use parser::DiagnosticParser;
