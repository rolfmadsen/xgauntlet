//! JIT Governance telemetry and phase directives.

pub mod jit;
pub mod session;

pub use jit::{get_phase_directive, render_jit_directive, JitPhaseDirective, PipelinePhase};
pub use session::{
    estimate_tokens_from_chars, estimate_tokens_from_str, format_compact_tokens,
    record_session_from_hook_payload, record_session_snapshot, record_session_turn,
    reset_session_health, sync_active_session_for_pre_invocation, SessionHealth,
    SessionHealthStatus, SessionTracker, SessionTrackerState, SESSION_STATE_FILE,
};
