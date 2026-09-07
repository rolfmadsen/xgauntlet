pub mod features;

pub use features::adapters::{
    get_adapter, AdapterHookVerdict, AdapterValidationResult, AntigravityAdapter,
    ClaudeCodeAdapter, CodexAdapter, HarnessAdapter, NormalizedToolCall, ValidationIssue,
    ValidationSeverity, SUPPORTED_HARNESSES,
};
pub use features::checkpoint::{
    compose_commit_message, execute_git_commit, extract_short_task_id, run_checkpoint,
    run_preflight_check, stage_workspace_changes, CheckpointError, CheckpointOptions,
    CheckpointPhase, CheckpointResult,
};
pub use features::config::{
    default_config_for_stack, detect_stack, get_stack_profile, list_supported_stacks, load_config,
    validate_config, ConfigValidationIssue, ConfigValidationReport, PathsConfig, StackProfile,
};
pub use features::diagnostics::{
    extract_cargo_check_findings, extract_cargo_mutants_findings, extract_cargo_test_findings,
    extract_clippy_findings, extract_eslint_findings, extract_hypothesis_findings,
    extract_mutants_py_findings, extract_proptest_findings, extract_pytest_findings,
    extract_ruff_findings, extract_tsc_findings, extract_vitest_findings, DiagnosticFinding,
    DiagnosticParser, DiagnosticReport, FindingType,
};
pub use features::doctor::{
    run_doctor, DoctorCategory, DoctorCheckItem, DoctorCheckStatus, DoctorError, DoctorOptions,
    DoctorReport, DoctorVerdict,
};
pub use features::evidence::{
    classify_evidence_payload, compute_digest_of_files, compute_file_git_blob_oid,
    compute_file_raw_hash, compute_workspace_manifest, generate_report_json,
    generate_report_markdown, is_binary_buffer, is_excluded, load_report_json,
    load_verification_report, save_verification_report, verify_self_mutation,
    verify_workspace_state_match, AttestationStatus, CanonicalWorkspaceManifest, CheckStatus,
    CheckSummary, DriftViolation, ExecutionMetadata, ExecutionOrigin, ManifestError, ManifestItem,
    ReportError, SelfMutationViolation, TaskContractSummary, TrustDecision, VcsMetadata,
    VerificationReport, VerificationVerdict, WorkspaceState, DEFAULT_SCOPES, EXCLUDED_DIRS,
    EXCLUDED_FILES,
};
pub use features::gauntlet::{
    execute_gauntlet_pipeline, execute_layer, run_gauntlet, ConfigError, GauntletConfig,
    GauntletExecutionOutcome, GauntletOptions, GauntletPipelineError, GauntletReport, LayerConfig,
    LayerDefinition, LayerExecutionStatus, LayerRequirement, LayerResult,
};
pub use features::policy::{
    CapabilityRequest, DecisionVerdict, EnforcementContext, PathSecurityError, PolicyDecision,
    PolicyError, PolicyEvaluator, ToolActionType, WasmPolicyEngine, WorkspaceRelativePath,
};
pub use features::release::{
    check_adr_coverage, check_release_readiness, extract_declared_versions,
    parse_changelog_versions, validate_changelog, AdrCoverageSummary, ManifestVersionSummary,
    ReleaseError, ReleaseFinding, ReleaseFindingCategory, ReleaseReadinessEngine,
    ReleaseReadinessOptions, ReleaseReadinessReport,
};
pub use features::scaffold::{
    generate_templates, run_scaffold, ScaffoldAction, ScaffoldError, ScaffoldFileReport,
    ScaffoldOptions, ScaffoldResult, ScaffoldTemplate,
};
pub use features::tasks::{
    check_all_tasks, check_task_specification, collect_git_telemetry, has_active_task,
    inspect_task_telemetry, is_task_active, list_workspace_tasks, parse_criteria_progress,
    parse_frontmatter, parse_task_content, parse_task_file, parse_task_status,
    resolve_active_task_id, resolve_latest_done_task_id, resolve_task_contract,
    validate_context_content, validate_context_glossary, validate_iso_timestamp, Actor,
    CriteriaProgress, GeneratedEntry, GitTelemetry, OkfError, OkfMetadata, ScaffoldTaskOptions,
    SourceEntry, SpecReadinessReport, TaskContract, TaskError, TaskPackageInfo, TaskScaffoldResult,
    TaskScaffolder, TaskStatus, TaskSummaryItem, TaskTelemetry, VerifiedEntry,
};
pub use features::wasm::{WasmError, WasmRuntimeHost};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineInfo {
    pub name: &'static str,
    pub version: &'static str,
}

pub fn get_engine_info() -> EngineInfo {
    EngineInfo {
        name: "xgauntlet-core",
        version: VERSION,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_info() {
        let info = get_engine_info();
        assert_eq!(info.name, "xgauntlet-core");
        assert!(!info.version.is_empty());
    }
}
