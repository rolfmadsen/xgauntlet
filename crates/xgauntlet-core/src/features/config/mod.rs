//! Declarative configuration and stack profiles feature module.
//!
//! Provides zero-dependency `gauntlet.toml` loading, JSON deserialization,
//! stack profile presets (Rust, Python, Node/TS, Go), and schema validation.

pub mod loader;
pub mod models;
pub mod profiles;
pub mod validation;

pub use loader::{load_config, parse_json, parse_toml, render_toml};
pub use models::{
    ConfigError, ConfigValidationIssue, ConfigValidationReport, GauntletConfig, LayerConfig,
    PathsConfig, StackProfile, ValidationSeverity,
};
pub use profiles::{
    default_config_for_stack, detect_stack, get_stack_profile, list_supported_stacks,
    SUPPORTED_STACKS,
};
pub use validation::validate_config;
