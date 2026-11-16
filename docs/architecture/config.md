# Feature Architecture: `config`

🌐 **[Open Interactive Diagram (Pele Playground)](https://pele.run/playground#source=classDiagram%0A++++direction+TB%0A%0A++++%25%25+%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%0A++++%25%25+%F0%9F%A7%A9+Feature+Slice%3A+config+%28with+1-hop+domain+neighbors%29%0A++++%25%25+%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%0A%0A++++class+ArchitectureConfig+%7B%0A++++++++%2Bmax_file_lines%3A+Option%7Eusize%7E%0A++++++++%2Bmax_test_file_lines%3A+Option%7Eusize%7E%0A++++++++%2Bforbidden_tokens%3A+Vec%7EString%7E%0A++++++++%2Bforbidden_imports%3A+Vec%7EString%7E%0A++++++++%2Blayers%3A+Vec%7EString%7E%0A++++++++%2Bfeature_root%3A+String%0A++++++++%2Bisolate_features%3A+bool%0A++++++++%2Bforbid_core_imports%3A+Vec%7EString%7E%0A++++++++%2Bexemptions%3A+Vec%7EArchitectureExemption%7E%0A++++%7D%0A%0A++++class+ArchitectureExemption+%7B%0A++++++++%2Bpath%3A+String%0A++++++++%2Breason%3A+String%0A++++++++%2Badr%3A+String%0A++++++++%2Bmax_lines%3A+Option%7Eusize%7E%0A++++++++%2Ballow_imports%3A+Vec%7EString%7E%0A++++%7D%0A%0A++++class+CheckpointConfig+%7B%0A++++++++%2Benforce_provenance%3A+bool%0A++++%7D%0A%0A++++class+ConfigValidationIssue+%7B%0A++++++++%2Bseverity%3A+ValidationSeverity%0A++++++++%2Bcode%3A+String%0A++++++++%2Bmessage%3A+String%0A++++++++%2Bfield%3A+Option%7EString%7E%0A++++++++%2Bremediation%3A+Option%7EString%7E%0A++++%7D%0A%0A++++class+ConfigValidationReport+%7B%0A++++++++%2Bis_valid%3A+bool%0A++++++++%2Bissues%3A+Vec%7EConfigValidationIssue%7E%0A++++%7D%0A%0A++++class+GauntletConfig+%7B%0A++++++++%2Bstack%3A+String%0A++++++++%2Bsave_evidence%3A+bool%0A++++++++%2Bevidence_file%3A+String%0A++++++++%2Bevidence_markdown_file%3A+String%0A++++++++%2Bpaths%3A+PathsConfig%0A++++++++%2Barchitecture%3A+ArchitectureConfig%0A++++++++%2Bcheckpoint%3A+CheckpointConfig%0A++++++++%2Bmulti_agent%3A+MultiAgentConfig%0A++++++++%2Blayers%3A+Vec%7ELayerConfig%7E%0A++++%7D%0A%0A++++class+LayerConfig+%7B%0A++++++++%2Bname%3A+String%0A++++++++%2Bcommand%3A+Vec%7EString%7E%0A++++++++%2Boptional%3A+bool%0A++++++++%2Btimeout_seconds%3A+f64%0A++++++++%2Bfail_on_output%3A+bool%0A++++%7D%0A%0A++++class+MultiAgentConfig+%7B%0A++++++++%2Ballow_coordinated_inherit%3A+bool%0A++++++++%2Bshare_cargo_target_dir%3A+bool%0A++++%7D%0A%0A++++class+PathsConfig+%7B%0A++++++++%2Btasks_dir%3A+String%0A++++++++%2Bspec_file%3A+String%0A++++++++%2Bcontext_file%3A+String%0A++++++++%2Bcoding_standards_file%3A+String%0A++++++++%2Bdiagram_file%3A+String%0A++++%7D%0A%0A++++class+StackProfile+%7B%0A++++++++%2Bname%3A+String%0A++++++++%2Bdescription%3A+String%0A++++++++%2Bdetection_files%3A+Vec%7EString%7E%0A++++++++%2Bdefault_layers%3A+Vec%7ELayerConfig%7E%0A++++%7D%0A%0A++++class+ValidationIssue+%7B%0A++++++++%2Bseverity%3A+ValidationSeverity%0A++++++++%2Bpath%3A+String%0A++++++++%2Bmessage%3A+String%0A++++%7D%0A%0A++++class+ValidationSeverity+%7B%0A++++++++%3C%3Cenumeration%3E%3E%0A++++++++Error%0A++++++++Warning%0A++++%7D%0A%0A++++ConfigValidationReport+*--+ConfigValidationIssue+%3A+issues%0A++++ArchitectureConfig+*--+ArchitectureExemption+%3A+exemptions%0A++++ValidationIssue+*--+ValidationSeverity+%3A+severity%0A++++ConfigValidationIssue+*--+ValidationSeverity+%3A+severity%0A++++GauntletConfig+*--+PathsConfig+%3A+paths%0A++++GauntletConfig+*--+ArchitectureConfig+%3A+architecture%0A++++GauntletConfig+*--+CheckpointConfig+%3A+checkpoint%0A++++GauntletConfig+*--+MultiAgentConfig+%3A+multi_agent%0A++++GauntletConfig+*--+LayerConfig+%3A+layers%0A++++StackProfile+*--+LayerConfig+%3A+default_layers)** • ← [Full Architecture](../../architecture.md) • 📋 [Feature Spec](../../specs/config.md)

> Scoped domænemodel for `config` (inkl. direkte 1-hop naborelationer). Åbn lokalt med pan/zoom via `xgauntlet topology open --feature config`.

```mermaid
classDiagram
    direction TB

    %% ====================================================================
    %% 🧩 Feature Slice: config (with 1-hop domain neighbors)
    %% ====================================================================

    class ArchitectureConfig {
        +max_file_lines: Option~usize~
        +max_test_file_lines: Option~usize~
        +forbidden_tokens: Vec~String~
        +forbidden_imports: Vec~String~
        +layers: Vec~String~
        +feature_root: String
        +isolate_features: bool
        +forbid_core_imports: Vec~String~
        +exemptions: Vec~ArchitectureExemption~
    }

    class ArchitectureExemption {
        +path: String
        +reason: String
        +adr: String
        +max_lines: Option~usize~
        +allow_imports: Vec~String~
    }

    class CheckpointConfig {
        +enforce_provenance: bool
    }

    class ConfigValidationIssue {
        +severity: ValidationSeverity
        +code: String
        +message: String
        +field: Option~String~
        +remediation: Option~String~
    }

    class ConfigValidationReport {
        +is_valid: bool
        +issues: Vec~ConfigValidationIssue~
    }

    class GauntletConfig {
        +stack: String
        +save_evidence: bool
        +evidence_file: String
        +evidence_markdown_file: String
        +paths: PathsConfig
        +architecture: ArchitectureConfig
        +checkpoint: CheckpointConfig
        +multi_agent: MultiAgentConfig
        +layers: Vec~LayerConfig~
    }

    class LayerConfig {
        +name: String
        +command: Vec~String~
        +optional: bool
        +timeout_seconds: f64
        +fail_on_output: bool
    }

    class MultiAgentConfig {
        +allow_coordinated_inherit: bool
        +share_cargo_target_dir: bool
    }

    class PathsConfig {
        +tasks_dir: String
        +spec_file: String
        +context_file: String
        +coding_standards_file: String
        +diagram_file: String
    }

    class StackProfile {
        +name: String
        +description: String
        +detection_files: Vec~String~
        +default_layers: Vec~LayerConfig~
    }

    class ValidationIssue {
        +severity: ValidationSeverity
        +path: String
        +message: String
    }

    class ValidationSeverity {
        <<enumeration>>
        Error
        Warning
    }

    ConfigValidationReport *-- ConfigValidationIssue : issues
    ArchitectureConfig *-- ArchitectureExemption : exemptions
    ValidationIssue *-- ValidationSeverity : severity
    ConfigValidationIssue *-- ValidationSeverity : severity
    GauntletConfig *-- PathsConfig : paths
    GauntletConfig *-- ArchitectureConfig : architecture
    GauntletConfig *-- CheckpointConfig : checkpoint
    GauntletConfig *-- MultiAgentConfig : multi_agent
    GauntletConfig *-- LayerConfig : layers
    StackProfile *-- LayerConfig : default_layers
```
