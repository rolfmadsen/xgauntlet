//! Pre-configured stack profiles and language ecosystem defaults.

use std::path::Path;

use super::models::{ArchitectureConfig, GauntletConfig, LayerConfig, PathsConfig, StackProfile};

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
            default_layers: node_layers(false),
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
    default_config_for_stack_and_workspace(stack, None)
}

/// Constructs a full GauntletConfig populated with standard layers for the stack,
/// adapting layers if workspace hints (such as composite TypeScript project references) are detected.
pub fn default_config_for_stack_and_workspace(
    stack: &str,
    workspace: Option<&Path>,
) -> GauntletConfig {
    let normalized = stack.to_ascii_lowercase();
    let is_composite = workspace.map(detect_composite_typescript).unwrap_or(false);

    let layers = match normalized.as_str() {
        "python" => python_layers(),
        "node" | "typescript" | "javascript" => node_layers(is_composite),
        "go" => go_layers(),
        _ => rust_layers(),
    };

    GauntletConfig {
        stack: stack.to_string(),
        save_evidence: true,
        evidence_file: "evidence.json".to_string(),
        evidence_markdown_file: "evidence.md".to_string(),
        paths: PathsConfig::default(),
        architecture: ArchitectureConfig::default(),
        checkpoint: super::models::CheckpointConfig::default(),
        layers,
    }
}

fn rust_layers() -> Vec<LayerConfig> {
    vec![
        LayerConfig {
            name: "format".to_string(),
            command: vec![
                "cargo".to_string(),
                "fmt".to_string(),
                "--check".to_string(),
            ],
            optional: true,
            timeout_seconds: 30.0,
            fail_on_output: false,
        },
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
            fail_on_output: false,
        },
        LayerConfig {
            name: "types".to_string(),
            command: vec!["cargo".to_string(), "check".to_string()],
            optional: true,
            timeout_seconds: 60.0,
            fail_on_output: false,
        },
        LayerConfig {
            name: "unit".to_string(),
            command: vec!["cargo".to_string(), "test".to_string()],
            optional: false,
            timeout_seconds: 120.0,
            fail_on_output: false,
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
            fail_on_output: false,
        },
        LayerConfig {
            name: "mutation-testing-gauntlet".to_string(),
            command: vec!["cargo".to_string(), "mutants".to_string()],
            optional: true,
            timeout_seconds: 180.0,
            fail_on_output: false,
        },
    ]
}

fn python_layers() -> Vec<LayerConfig> {
    vec![
        LayerConfig {
            name: "format".to_string(),
            command: vec![
                "ruff".to_string(),
                "format".to_string(),
                "--check".to_string(),
            ],
            optional: true,
            timeout_seconds: 30.0,
            fail_on_output: false,
        },
        LayerConfig {
            name: "lint".to_string(),
            command: vec!["ruff".to_string(), "check".to_string()],
            optional: true,
            timeout_seconds: 60.0,
            fail_on_output: false,
        },
        LayerConfig {
            name: "types".to_string(),
            command: vec!["pyright".to_string()],
            optional: true,
            timeout_seconds: 60.0,
            fail_on_output: false,
        },
        LayerConfig {
            name: "unit".to_string(),
            command: vec!["pytest".to_string()],
            optional: false,
            timeout_seconds: 120.0,
            fail_on_output: false,
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
            fail_on_output: false,
        },
        LayerConfig {
            name: "mutation-testing-gauntlet".to_string(),
            command: vec!["mutants.py".to_string()],
            optional: true,
            timeout_seconds: 180.0,
            fail_on_output: false,
        },
    ]
}

fn node_layers(is_composite: bool) -> Vec<LayerConfig> {
    let types_cmd = if is_composite {
        vec!["npx".to_string(), "tsc".to_string(), "-b".to_string()]
    } else {
        vec!["npx".to_string(), "tsc".to_string(), "--noEmit".to_string()]
    };

    vec![
        LayerConfig {
            name: "format".to_string(),
            command: vec![
                "npx".to_string(),
                "prettier".to_string(),
                "--check".to_string(),
                ".".to_string(),
            ],
            optional: true,
            timeout_seconds: 30.0,
            fail_on_output: false,
        },
        LayerConfig {
            name: "lint".to_string(),
            command: vec!["npm".to_string(), "run".to_string(), "lint".to_string()],
            optional: true,
            timeout_seconds: 60.0,
            fail_on_output: false,
        },
        LayerConfig {
            name: "types".to_string(),
            command: types_cmd,
            optional: true,
            timeout_seconds: 60.0,
            fail_on_output: false,
        },
        LayerConfig {
            name: "unit".to_string(),
            command: vec!["npm".to_string(), "test".to_string()],
            optional: false,
            timeout_seconds: 120.0,
            fail_on_output: false,
        },
    ]
}

fn go_layers() -> Vec<LayerConfig> {
    vec![
        LayerConfig {
            name: "format".to_string(),
            command: vec!["gofmt".to_string(), "-l".to_string(), ".".to_string()],
            optional: true,
            timeout_seconds: 30.0,
            fail_on_output: true,
        },
        LayerConfig {
            name: "lint".to_string(),
            command: vec!["golangci-lint".to_string(), "run".to_string()],
            optional: true,
            timeout_seconds: 60.0,
            fail_on_output: false,
        },
        LayerConfig {
            name: "types".to_string(),
            command: vec!["go".to_string(), "vet".to_string(), "./...".to_string()],
            optional: true,
            timeout_seconds: 60.0,
            fail_on_output: false,
        },
        LayerConfig {
            name: "unit".to_string(),
            command: vec!["go".to_string(), "test".to_string(), "./...".to_string()],
            optional: false,
            timeout_seconds: 120.0,
            fail_on_output: false,
        },
    ]
}

/// Strips line comments (`//`) and block comments (`/* ... */`) from JSON text.
fn strip_json_comments(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut in_string = false;
    let mut escape = false;

    while let Some(ch) = chars.next() {
        if in_string {
            output.push(ch);
            if escape {
                escape = false;
            } else if ch == '\\' {
                escape = true;
            } else if ch == '"' {
                in_string = false;
            }
        } else if ch == '"' {
            in_string = true;
            output.push(ch);
        } else if ch == '/' {
            if let Some(&'/') = chars.peek() {
                chars.next();
                for c in chars.by_ref() {
                    if c == '\n' {
                        output.push('\n');
                        break;
                    }
                }
            } else if let Some(&'*') = chars.peek() {
                chars.next();
                while let Some(c) = chars.next() {
                    if c == '*' && chars.peek() == Some(&'/') {
                        chars.next();
                        break;
                    }
                }
            } else {
                output.push(ch);
            }
        } else {
            output.push(ch);
        }
    }
    output
}

/// Analyzes tsconfig.json content for composite project references structure (AC1).
///
/// Returns true if the configuration has non-empty `"references"` and empty or missing
/// direct source inclusions (`"files"` and `"include"`).
pub fn is_composite_typescript_tsconfig(content: &str) -> bool {
    let clean = strip_json_comments(content);
    let val: serde_json::Value = match serde_json::from_str(&clean) {
        Ok(v) => v,
        Err(_) => return false,
    };

    let obj = match val.as_object() {
        Some(o) => o,
        None => return false,
    };

    // 1. Must have non-empty "references" array
    let has_references = obj
        .get("references")
        .and_then(|r| r.as_array())
        .map(|arr| !arr.is_empty())
        .unwrap_or(false);

    if !has_references {
        return false;
    }

    // 2. "files" must be absent or empty array
    let files_empty_or_absent = match obj.get("files") {
        None => true,
        Some(f) => f.as_array().map(|arr| arr.is_empty()).unwrap_or(false),
    };

    // 3. "include" must be absent or empty array
    let include_empty_or_absent = match obj.get("include") {
        None => true,
        Some(inc) => inc.as_array().map(|arr| arr.is_empty()).unwrap_or(false),
    };

    files_empty_or_absent && include_empty_or_absent
}

/// Inspects workspace root for composite TypeScript project references (AC1).
pub fn detect_composite_typescript(workspace: &Path) -> bool {
    let tsconfig = workspace.join("tsconfig.json");
    if !tsconfig.is_file() {
        return false;
    }

    match std::fs::read_to_string(&tsconfig) {
        Ok(content) => is_composite_typescript_tsconfig(&content),
        Err(_) => false,
    }
}
