# Feature Architecture: `tasks`

🌐 **[Open Interactive Diagram (Pele Playground)](https://pele.run/playground#source=classDiagram%0A++++direction+TB%0A%0A++++%25%25+%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%0A++++%25%25+%F0%9F%A7%A9+Feature+Slice%3A+tasks+%28with+1-hop+domain+neighbors%29%0A++++%25%25+%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%0A%0A++++class+AdrReadinessReport+%7B%0A++++++++%2Bis_valid%3A+bool%0A++++++++%2Badr_id%3A+String%0A++++++++%2Bdiagnostics%3A+Vec%7EDiagnosticFinding%7E%0A++++++++%2Binspected_files%3A+Vec%7EString%7E%0A++++%7D%0A%0A++++class+DiagnosticFinding+%7B%0A++++++++%2Bfinding_type%3A+FindingType%0A++++++++%2Btool_name%3A+String%0A++++++++%2Bfile_path%3A+String%0A++++++++%2Bline_number%3A+Option%7Eusize%7E%0A++++++++%2Bcolumn_number%3A+Option%7Eusize%7E%0A++++++++%2Bmessage%3A+String%0A++++++++%2Bremediation_hint%3A+String%0A++++++++%2Braw_context%3A+String%0A++++%7D%0A%0A++++class+GlossaryDriftReport+%7B%0A++++++++%2Bis_valid%3A+bool%0A++++++++%2Borphan_models%3A+Vec%7EString%7E%0A++++++++%2Bphantom_terms%3A+Vec%7EString%7E%0A++++++++%2Bavoid_violations%3A+Vec%7EDiagnosticFinding%7E%0A++++++++%2Bdiagnostics%3A+Vec%7EDiagnosticFinding%7E%0A++++%7D%0A%0A++++class+SpecReadinessReport+%7B%0A++++++++%2Bis_valid%3A+bool%0A++++++++%2Btask_id%3A+String%0A++++++++%2Bdiagnostics%3A+Vec%7EDiagnosticFinding%7E%0A++++++++%2Binspected_files%3A+Vec%7EString%7E%0A++++++++%2Bmust_not_rules%3A+Vec%7EString%7E%0A++++++++%2Bacceptance_criteria%3A+Vec%7EString%7E%0A++++%7D%0A%0A++++class+TaskContract+%7B%0A++++++++%2Btask_id%3A+String%0A++++++++%2Btask_title%3A+String%0A++++++++%2Btask_digest%3A+String%0A++++++++%2Bacceptance_criteria%3A+Vec%7EString%7E%0A++++++++%2Bunresolved_criteria%3A+Vec%7EString%7E%0A++++%7D%0A%0A++++class+TaskPackageInfo+%7B%0A++++++++%2Btask_id%3A+String%0A++++++++%2Btitle%3A+String%0A++++++++%2Bstatus%3A+TaskStatus%0A++++++++%2Bintent%3A+Option%7EString%7E%0A++++++++%2Bpurpose%3A+String%0A++++++++%2Bacceptance_criteria%3A+Vec%7EString%7E%0A++++++++%2Bunresolved_criteria%3A+Vec%7EString%7E%0A++++++++%2Bmust_not%3A+Vec%7EString%7E%0A++++++++%2Bscope%3A+Vec%7EString%7E%0A++++++++%2Btest_refs%3A+Vec%7EString%7E%0A++++++++%2Bdomain%3A+Option%7EString%7E%0A++++++++%2Bfeature%3A+Option%7EString%7E%0A++++++++%2Bspec_ref%3A+Option%7EString%7E%0A++++++++%2Bmetadata%3A+Option%7EOkfMetadata%7E%0A++++%7D%0A%0A++++class+TaskStatus+%7B%0A++++++++%3C%3Cenumeration%3E%3E%0A++++++++Draft%0A++++++++Active%0A++++++++InProgress%0A++++++++Wip%0A++++++++Todo%0A++++++++Reopened%0A++++++++Done%0A++++++++Completed%0A++++++++Deprecated%0A++++++++Superseded%0A++++++++Rejected%0A++++++++Unknown%0A++++%7D%0A%0A++++class+VerificationReport+%7B%0A++++++++%2Bschema%3A+String%0A++++++++%2Bschema_version%3A+String%0A++++++++%2Bexecution_origin%3A+ExecutionOrigin%0A++++++++%2Bverdict%3A+VerificationVerdict%0A++++++++%2Btask_contract%3A+TaskContract%0A++++++++%2Bworkspace_state%3A+WorkspaceState%0A++++++++%2Bexecution_metadata%3A+ExecutionMetadata%0A++++++++%2Bchecks%3A+Vec%7ECheckSummary%7E%0A++++++++%2Bdiagnostics%3A+Vec%7EDiagnosticFinding%7E%0A++++++++%2Blineage%3A+SessionLineageTrail%7E%0A++++++++%2Btest_provenance%3A+TestProvenanceEvidence%7E%7E%0A++++++++%2Bproof_mode%3A+Option%7EString%7E%0A++++++++%2Battestation%3A+Option%7EAttestationReport%7E%0A++++%7D%0A%0A++++GlossaryDriftReport+*--+DiagnosticFinding+%3A+avoid_violations%0A++++GlossaryDriftReport+*--+DiagnosticFinding+%3A+diagnostics%0A++++AdrReadinessReport+*--+DiagnosticFinding+%3A+diagnostics%0A++++SpecReadinessReport+*--+DiagnosticFinding+%3A+diagnostics%0A++++VerificationReport+*--+TaskContract+%3A+task_contract%0A++++VerificationReport+*--+DiagnosticFinding+%3A+diagnostics%0A++++TaskPackageInfo+*--+TaskStatus+%3A+status)** • ← [Full Architecture](../../architecture.md) • 📋 [Feature Spec](../../specs/tasks.md)

> Scoped domænemodel for `tasks` (inkl. direkte 1-hop naborelationer). Åbn lokalt med pan/zoom via `xgauntlet topology open --feature tasks`.

```mermaid
classDiagram
    direction TB

    %% ====================================================================
    %% 🧩 Feature Slice: tasks (with 1-hop domain neighbors)
    %% ====================================================================

    class AdrReadinessReport {
        +is_valid: bool
        +adr_id: String
        +diagnostics: Vec~DiagnosticFinding~
        +inspected_files: Vec~String~
    }

    class DiagnosticFinding {
        +finding_type: FindingType
        +tool_name: String
        +file_path: String
        +line_number: Option~usize~
        +column_number: Option~usize~
        +message: String
        +remediation_hint: String
        +raw_context: String
    }

    class GlossaryDriftReport {
        +is_valid: bool
        +orphan_models: Vec~String~
        +phantom_terms: Vec~String~
        +avoid_violations: Vec~DiagnosticFinding~
        +diagnostics: Vec~DiagnosticFinding~
    }

    class SpecReadinessReport {
        +is_valid: bool
        +task_id: String
        +diagnostics: Vec~DiagnosticFinding~
        +inspected_files: Vec~String~
        +must_not_rules: Vec~String~
        +acceptance_criteria: Vec~String~
    }

    class TaskContract {
        +task_id: String
        +task_title: String
        +task_digest: String
        +acceptance_criteria: Vec~String~
        +unresolved_criteria: Vec~String~
    }

    class TaskPackageInfo {
        +task_id: String
        +title: String
        +status: TaskStatus
        +intent: Option~String~
        +purpose: String
        +acceptance_criteria: Vec~String~
        +unresolved_criteria: Vec~String~
        +must_not: Vec~String~
        +scope: Vec~String~
        +test_refs: Vec~String~
        +domain: Option~String~
        +feature: Option~String~
        +spec_ref: Option~String~
        +metadata: Option~OkfMetadata~
    }

    class TaskStatus {
        <<enumeration>>
        Draft
        Active
        InProgress
        Wip
        Todo
        Reopened
        Done
        Completed
        Deprecated
        Superseded
        Rejected
        Unknown
    }

    class VerificationReport {
        +schema: String
        +schema_version: String
        +execution_origin: ExecutionOrigin
        +verdict: VerificationVerdict
        +task_contract: TaskContract
        +workspace_state: WorkspaceState
        +execution_metadata: ExecutionMetadata
        +checks: Vec~CheckSummary~
        +diagnostics: Vec~DiagnosticFinding~
        +lineage: SessionLineageTrail~
        +test_provenance: TestProvenanceEvidence~~
        +proof_mode: Option~String~
        +attestation: Option~AttestationReport~
    }

    GlossaryDriftReport *-- DiagnosticFinding : avoid_violations
    GlossaryDriftReport *-- DiagnosticFinding : diagnostics
    AdrReadinessReport *-- DiagnosticFinding : diagnostics
    SpecReadinessReport *-- DiagnosticFinding : diagnostics
    VerificationReport *-- TaskContract : task_contract
    VerificationReport *-- DiagnosticFinding : diagnostics
    TaskPackageInfo *-- TaskStatus : status
```
