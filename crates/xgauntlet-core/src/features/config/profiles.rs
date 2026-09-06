//! Pre-configured stack profiles and language ecosystem defaults.

use std::path::Path;

use super::models::{GauntletConfig, LayerConfig, PathsConfig, StackProfile};

pub const SUPPORTED_STACKS: &[&str] = &["rust", "python", "node", "typescript", "javascript", "go"];

/// Returns a slice of recognized stack profile identifiers.
pub fn list_supported_stacks() -> &'static [&'static str] {
    SUPPORTED_STACKS
}

/// Retrieves a StackProfile metadata descriptor by stack name.
pub fn get_stack_profile(name: &str) -> Option<StackProfile> {
    match name.to_ascii_lowercase().as_str() {
        "rust" => Some(StackProfile {
            name: "rust".to_string(),
            description: "Modern Rust ecosystem (Cargo, Clippy, Proptest, Mutants)".to_string(),
            detection_files: vec!["Cargo.toml".to_string()],
            default_layers: rust_layers(),
        }),
        "python" => Some(StackProfile {
            name: "python".to_string(),
            description: "Modern Python ecosystem (Ruff, Pyright, Pytest, Hypothesis, Mutants)"
                .to_string(),
            detection_files: vec![
                "pyproject.toml".to_string(),
                "requirements.txt".to_string(),
                "Pipfile".to_string(),
            ],
            default_layers: python_layers(),
        }),
        "node" | "typescript" | "javascript" => Some(StackProfile {
            name: name.to_ascii_lowercase(),
            description: "Node.js & TypeScript ecosystem (ESLint, TSC, Vitest/Jest)".to_string(),
            detection_files: vec!["package.json".to_string(), "tsconfig.json".to_string()],
            default_layers: node_layers(),
        }),
        "go" => Some(StackProfile {
            name: "go".to_string(),
            description: "Go ecosystem (Golangci-lint, Go vet, Go test)".to_string(),
            detection_files: vec!["go.mod".to_string()],
            default_layers: go_layers(),
        }),
        _ => None,
    }
}

/// Inspects workspace indicator files to infer the primary programming stack.
pub fn detect_stack(workspace: &Path) -> &'static str {
    if workspace.join("Cargo.toml").is_file() {
        "rust"
    } else if workspace.join("pyproject.toml").is_file()
        || workspace.join("requirements.txt").is_file()
        || workspace.join("Pipfile").is_file()
    {
        "python"
    } else if workspace.join("package.json").is_file() || workspace.join("tsconfig.json").is_file()
    {
        "node"
    } else if workspace.join("go.mod").is_file() {
        "go"
    } else {
        "rust"
    }
}

/// Constructs a full GauntletConfig populated with standard layers for the stack.
pub fn default_config_for_stack(stack: &str) -> GauntletConfig {
    let normalized = stack.to_ascii_lowercase();
    let layers = match normalized.as_str() {
        "python" => python_layers(),
        "node" | "typescript" | "javascript" => node_layers(),
        "go" => go_layers(),
        _ => rust_layers(),
    };

    GauntletConfig {
        stack: stack.to_string(),
        save_evidence: true,
        evidence_file: "evidence.json".to_string(),
        evidence_markdown_file: "evidence.md".to_string(),
        paths: PathsConfig::default(),
        layers,
    }
}

fn rust_layers() -> Vec<LayerConfig> {
    vec![
        LayerConfig {
            name: "lint".to_string(),
            command: vec![
                "cargo".to_string(),
                "clippy".to_string(),
                "--".to_string(),
                "-D".to_string(),
                "warnings".to_string(),
            ],
            optional: true,
            timeout_seconds: 60.0,
        },
        LayerConfig {
            name: "types".to_string(),
            command: vec!["cargo".to_string(), "check".to_string()],
            optional: true,
            timeout_seconds: 60.0,
        },
        LayerConfig {
            name: "unit".to_string(),
            command: vec!["cargo".to_string(), "test".to_string()],
            optional: false,
            timeout_seconds: 120.0,
        },
        LayerConfig {
            name: "invariants".to_string(),
            command: vec![
                "cargo".to_string(),
                "test".to_string(),
                "--".to_string(),
                "proptest".to_string(),
            ],
            optional: true,
            timeout_seconds: 120.0,
        },
        LayerConfig {
            name: "mutation-testing-gauntlet".to_string(),
            command: vec!["cargo".to_string(), "mutants".to_string()],
            optional: true,
            timeout_seconds: 180.0,
        },
    ]
}

fn python_layers() -> Vec<LayerConfig> {
    vec![
        LayerConfig {
            name: "lint".to_string(),
            command: vec!["ruff".to_string(), "check".to_string()],
            optional: true,
            timeout_seconds: 60.0,
        },
        LayerConfig {
            name: "types".to_string(),
            command: vec!["pyright".to_string()],
            optional: true,
            timeout_seconds: 60.0,
        },
        LayerConfig {
            name: "unit".to_string(),
            command: vec!["pytest".to_string()],
            optional: false,
            timeout_seconds: 120.0,
        },
        LayerConfig {
            name: "invariants".to_string(),
            command: vec![
                "pytest".to_string(),
                "-k".to_string(),
                "hypothesis".to_string(),
            ],
            optional: true,
            timeout_seconds: 120.0,
        },
        LayerConfig {
            name: "mutation-testing-gauntlet".to_string(),
            command: vec!["mutants.py".to_string()],
            optional: true,
            timeout_seconds: 180.0,
        },
    ]
}

fn node_layers() -> Vec<LayerConfig> {
    vec![
        LayerConfig {
            name: "lint".to_string(),
            command: vec!["npm".to_string(), "run".to_string(), "lint".to_string()],
            optional: true,
            timeout_seconds: 60.0,
        },
        LayerConfig {
            name: "types".to_string(),
            command: vec!["npx".to_string(), "tsc".to_string(), "--noEmit".to_string()],
            optional: true,
            timeout_seconds: 60.0,
        },
        LayerConfig {
            name: "unit".to_string(),
            command: vec!["npm".to_string(), "test".to_string()],
            optional: false,
            timeout_seconds: 120.0,
        },
    ]
}

fn go_layers() -> Vec<LayerConfig> {
    vec![
        LayerConfig {
            name: "lint".to_string(),
            command: vec!["golangci-lint".to_string(), "run".to_string()],
            optional: true,
            timeout_seconds: 60.0,
        },
        LayerConfig {
            name: "types".to_string(),
            command: vec!["go".to_string(), "vet".to_string(), "./...".to_string()],
            optional: true,
            timeout_seconds: 60.0,
        },
        LayerConfig {
            name: "unit".to_string(),
            command: vec!["go".to_string(), "test".to_string(), "./...".to_string()],
            optional: false,
            timeout_seconds: 120.0,
        },
    ]
}
