//! JIT Governance telemetry and phase directives.

pub mod jit;

pub use jit::{get_phase_directive, render_jit_directive, JitPhaseDirective, PipelinePhase};
