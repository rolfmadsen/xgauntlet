//! Just-In-Time (JIT) 7-phase governance and specialized AI roles jf. Task 023.

use serde::{Deserialize, Serialize};

/// The 7 distinct pipeline phases defined in Task 023.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PipelinePhase {
    IdeationAndContext,
    SpecificationAndTaskBinding,
    ImplementationTdd,
    MultiLayerVerification,
    StandardsAndSpecAudit,
    EvidenceIntegrityAndDriftCheck,
    ReleaseReadiness,
}

impl PipelinePhase {
    /// Returns all 7 canonical phases in pipeline order.
    pub fn all() -> &'static [PipelinePhase] {
        &[
            PipelinePhase::IdeationAndContext,
            PipelinePhase::SpecificationAndTaskBinding,
            PipelinePhase::ImplementationTdd,
            PipelinePhase::MultiLayerVerification,
            PipelinePhase::StandardsAndSpecAudit,
            PipelinePhase::EvidenceIntegrityAndDriftCheck,
            PipelinePhase::ReleaseReadiness,
        ]
    }

    /// Canonical display title of the phase.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::IdeationAndContext => "Ideation & Context",
            Self::SpecificationAndTaskBinding => "Specification & Task Binding",
            Self::ImplementationTdd => "Implementation (TDD)",
            Self::MultiLayerVerification => "Multi-Layer Verification",
            Self::StandardsAndSpecAudit => "Standards & Spec Audit",
            Self::EvidenceIntegrityAndDriftCheck => "Evidence Integrity & Drift Check",
            Self::ReleaseReadiness => "Release Readiness",
        }
    }

    /// Resolves canonical pipeline phase from free-text string or alias.
    pub fn parse_phase(s: &str) -> Option<Self> {
        let lower = s.trim().to_ascii_lowercase();
        if lower.contains("ideation") || lower.contains("context") {
            Some(Self::IdeationAndContext)
        } else if lower.contains("spec") || lower.contains("criteria") || lower.contains("task") {
            Some(Self::SpecificationAndTaskBinding)
        } else if lower.contains("tdd") || lower.contains("red") || lower.contains("green") || lower.contains("refactor") || lower.contains("impl") {
            Some(Self::ImplementationTdd)
        } else if lower.contains("verify") || lower.contains("gauntlet") || lower.contains("qa") {
            Some(Self::MultiLayerVerification)
        } else if lower.contains("audit") || lower.contains("standards") || lower.contains("review") {
            Some(Self::StandardsAndSpecAudit)
        } else if lower.contains("evidence") || lower.contains("drift") || lower.contains("checkpoint") {
            Some(Self::EvidenceIntegrityAndDriftCheck)
        } else if lower.contains("release") || lower.contains("handoff") || lower.contains("ready") {
            Some(Self::ReleaseReadiness)
        } else {
            None
        }
    }
}

/// JIT Phase Directive with specialized active persona, positive target, checkable gate, and front-loaded pointer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JitPhaseDirective {
    pub phase: PipelinePhase,
    pub active_role: &'static str,
    pub target: &'static str,
    pub gate: &'static str,
    pub pointer: &'static str,
}

impl JitPhaseDirective {
    /// Estimates token count (approx. words * 1.3).
    pub fn estimate_token_count(&self) -> usize {
        let full_text = format!("{} {} {} {}", self.active_role, self.target, self.gate, self.pointer);
        let words = full_text.split_whitespace().count();
        (words as f64 * 1.3).ceil() as usize
    }

    /// Renders the compact JIT directive string.
    pub fn render(&self, task_id: Option<&str>) -> String {
        let task_str = task_id.unwrap_or("<id>");
        let gate_rendered = self.gate.replace("<id>", task_str);
        format!(
            "Phase: {}\n{}.\nTarget: {}.\nGate (Done): {}.\nPointer: {}.",
            self.phase.as_str(),
            self.active_role,
            self.target.trim_end_matches('.'),
            gate_rendered.trim_end_matches('.'),
            self.pointer.trim_end_matches('.')
        )
    }
}

/// Returns the JIT phase directive for the given phase.
///
/// In RED phase, returns None stub.
pub fn get_phase_directive(_phase: PipelinePhase) -> Option<JitPhaseDirective> {
    // RED phase: stub returns None
    None
}

/// Renders a JIT directive for a phase.
///
/// In RED phase, returns empty string stub.
pub fn render_jit_directive(_phase: PipelinePhase, _task_id: Option<&str>) -> String {
    // RED phase: stub returns empty string
    String::new()
}
