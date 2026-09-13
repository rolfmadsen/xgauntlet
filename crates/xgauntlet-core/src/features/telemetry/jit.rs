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
        } else if lower.contains("tdd")
            || lower.contains("red")
            || lower.contains("green")
            || lower.contains("refactor")
            || lower.contains("impl")
        {
            Some(Self::ImplementationTdd)
        } else if lower.contains("verify") || lower.contains("gauntlet") || lower.contains("qa") {
            Some(Self::MultiLayerVerification)
        } else if lower.contains("audit") || lower.contains("standards") || lower.contains("review")
        {
            Some(Self::StandardsAndSpecAudit)
        } else if lower.contains("evidence")
            || lower.contains("drift")
            || lower.contains("checkpoint")
        {
            Some(Self::EvidenceIntegrityAndDriftCheck)
        } else if lower.contains("release") || lower.contains("handoff") || lower.contains("ready")
        {
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
        let full_text = format!(
            "{} {} {} {}",
            self.active_role, self.target, self.gate, self.pointer
        );
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
pub fn get_phase_directive(phase: PipelinePhase) -> Option<JitPhaseDirective> {
    match phase {
        PipelinePhase::IdeationAndContext => Some(JitPhaseDirective {
            phase,
            active_role: "Active Role: System Architect (Scope & Invariants)",
            target: "Establish operational boundaries and glossary in CONTEXT.md",
            gate: "Human approves scope and domain glossary",
            pointer: "Socratic: invoke grill-with-docs or domain-modeling for CONTEXT.md",
        }),
        PipelinePhase::SpecificationAndTaskBinding => Some(JitPhaseDirective {
            phase,
            active_role: "Active Role: Requirements Engineer (Contracts & Criteria)",
            target: "Formalize criteria checkboxes and Must NOT invariants in tasks/<id>.md",
            gate: "Command 'xgauntlet check-spec -t <id>' exits 0",
            pointer: "Spec synthesis: invoke to-spec and to-tasks for task binding",
        }),
        PipelinePhase::ImplementationTdd => Some(JitPhaseDirective {
            phase,
            active_role: "Active Role: TDD Craftsman (Red-Green-Refactor)",
            target: "Execute tight TDD: failing test (RED) ➔ fix (GREEN) ➔ refactor",
            gate: "All acceptance assertions pass with atomic git checkpoints",
            pointer: "TDD loop: invoke old-coder or diagnose for isolation",
        }),
        PipelinePhase::MultiLayerVerification => Some(JitPhaseDirective {
            phase,
            active_role: "Active Role: QA Engineer (Gauntlet & Anti-Tamper)",
            target: "Execute verification gauntlet and seal anti-tamper evidence",
            gate: "'xgauntlet verify --task <id> --save' records PASSED",
            pointer: "Refactor: invoke codebase-design or improve-codebase-architecture",
        }),
        PipelinePhase::StandardsAndSpecAudit => Some(JitPhaseDirective {
            phase,
            active_role: "Active Role: Code Reviewer (Standards & Smells)",
            target: "Two-axis review: Axis A (standards), Axis B (task criteria)",
            gate: "Human approval of audit findings before task closure",
            pointer: "Code review: invoke code-review to audit git diff",
        }),
        PipelinePhase::EvidenceIntegrityAndDriftCheck => Some(JitPhaseDirective {
            phase,
            active_role: "Active Role: Evidence Auditor (Drift & Trust Boundary)",
            target: "Verify workspace Git OID integrity and commit sealed checkpoint",
            gate: "'xgauntlet check-evidence' passes and local checkpoint committed",
            pointer: "Checkpoint gate: invoke 'xgauntlet checkpoint --phase done'",
        }),
        PipelinePhase::ReleaseReadiness => Some(JitPhaseDirective {
            phase,
            active_role: "Active Role: Release Engineer (Release & Attestation)",
            target: "Verify 100% clean git worktree and sync release readiness",
            gate: "Git worktree confirmed clean and 🏁 SESSION HANDOFF card displayed",
            pointer: "Retrospective: invoke retro and inspect 'git status'",
        }),
    }
}

/// Renders a JIT directive for a phase.
pub fn render_jit_directive(phase: PipelinePhase, task_id: Option<&str>) -> String {
    if let Some(directive) = get_phase_directive(phase) {
        directive.render(task_id)
    } else {
        String::new()
    }
}
