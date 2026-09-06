//! Gauntlet feature module providing multi-layer execution, process timeouts, config, and pipeline orchestration.

pub mod config;
pub mod models;
pub mod pipeline;
pub mod runner;

pub use config::{ConfigError, GauntletConfig, LayerConfig};
pub use models::{
    GauntletReport, LayerDefinition, LayerExecutionStatus, LayerRequirement, LayerResult,
};
pub use pipeline::{
    execute_gauntlet_pipeline, GauntletExecutionOutcome, GauntletOptions, GauntletPipelineError,
};
pub use runner::{execute_layer, run_gauntlet};
