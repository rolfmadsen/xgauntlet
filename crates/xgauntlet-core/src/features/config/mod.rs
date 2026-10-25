//! Declarative configuration and stack profiles feature module.
//!
//! Provides zero-dependency `gauntlet.toml` loading, JSON deserialization,
//! stack profile presets (Rust, Python, Node/TS, Go), and schema validation.

pub mod loader;
pub mod models;
pub mod profiles;
pub mod validation;

pub use loader::{load_config, load_config_from_str, parse_json, parse_toml, render_toml};
pub use models::{
    ArchitectureConfig, ArchitectureExemption, ConfigError, ConfigValidationIssue,
    ConfigValidationReport, GauntletConfig, LayerConfig, PathsConfig, StackProfile,
    ValidationSeverity,
};
pub use profiles::{
    default_config_for_stack, default_config_for_stack_and_workspace, detect_composite_typescript,
    detect_stack, get_stack_profile, is_composite_typescript_tsconfig, list_supported_stacks,
    SUPPORTED_STACKS,
};
pub use validation::validate_config;
