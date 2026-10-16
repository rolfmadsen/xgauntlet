//! Integration and contract tests for Task 034: Declarative Architecture Gatekeeper.

use std::fs;
use std::path::{Path, PathBuf};

use xgauntlet_core::features::architecture::{
    run_architecture_check, ArchitectureConfig, ArchitectureExemption, ArchitectureFindingCategory,
};
use xgauntlet_core::features::config::load_config_from_str;

static TEST_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(prefix: &str) -> Self {
        let count = TEST_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let unique = format!(
            "{}_{}_{}_{}",
            prefix,
            std::process::id(),
            count,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let path = std::env::temp_dir().join(format!("xgauntlet_arch_test_{}", unique));
        fs::create_dir_all(&path).unwrap();
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[test]
fn test_architecture_config_parsing_from_toml() {
    let toml = r#"
stack = "rust"

[architecture]
max_file_lines = 800
max_test_file_lines = 1000
forbidden_tokens = ["Color::from_rgb", "Color::from_rgba"]
forbidden_imports = ["crate::ui::app", "super::app"]

[[architecture.exemptions]]
path = "src/legacy/parser.rs"
reason = "Parser state machine jf. ADR 0009"
max_lines = 1500
"#;

    let cfg = load_config_from_str(toml, Path::new("gauntlet.toml")).expect("parse toml config");
    assert_eq!(cfg.architecture.max_file_lines, Some(800));
    assert_eq!(cfg.architecture.max_test_file_lines, Some(1000));
    assert_eq!(
        cfg.architecture.forbidden_tokens,
        vec!["Color::from_rgb", "Color::from_rgba"]
    );
    assert_eq!(
        cfg.architecture.forbidden_imports,
        vec!["crate::ui::app", "super::app"]
    );
    assert_eq!(cfg.architecture.exemptions.len(), 1);
    assert_eq!(cfg.architecture.exemptions[0].path, "src/legacy/parser.rs");
    assert_eq!(cfg.architecture.exemptions[0].max_lines, Some(1500));
}

#[test]
fn test_architecture_check_passes_on_compliant_workspace() {
    let temp = TempDir::new("arch_compliant");
    let ws = temp.path();

    let src_dir = ws.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(
        src_dir.join("lib.rs"),
        "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n",
    )
    .unwrap();

    let cfg = ArchitectureConfig {
        max_file_lines: Some(800),
        max_test_file_lines: Some(1000),
        forbidden_tokens: vec!["Color::from_rgb".into()],
        forbidden_imports: vec!["crate::ui::app".into()],
        exemptions: vec![],
    };

    let report = run_architecture_check(ws, &cfg).expect("run architecture check");
    assert!(report.is_valid, "Compliant workspace must be valid");
    assert_eq!(report.findings.len(), 0);
}

#[test]
fn test_architecture_check_fails_on_monolithic_production_file() {
    let temp = TempDir::new("arch_monolith_prod");
    let ws = temp.path();

    let src_dir = ws.join("src");
    fs::create_dir_all(&src_dir).unwrap();

    // Generate 850 lines
    let mut content = String::new();
    for i in 0..850 {
        content.push_str(&format!("// Line {}\n", i));
    }
    fs::write(src_dir.join("huge_service.rs"), content).unwrap();

    let cfg = ArchitectureConfig {
        max_file_lines: Some(800),
        max_test_file_lines: Some(1000),
        forbidden_tokens: vec![],
        forbidden_imports: vec![],
        exemptions: vec![],
    };

    let report = run_architecture_check(ws, &cfg).expect("run architecture check");
    assert!(!report.is_valid);
    assert!(report.findings.iter().any(|f| {
        f.category == ArchitectureFindingCategory::FileExceedsMaxLines
            && f.file_path.contains("huge_service.rs")
    }));
}

#[test]
fn test_architecture_check_fails_on_monolithic_test_file() {
    let temp = TempDir::new("arch_monolith_test");
    let ws = temp.path();

    let tests_dir = ws.join("tests");
    fs::create_dir_all(&tests_dir).unwrap();

    // Generate 1050 lines
    let mut content = String::new();
    for i in 0..1050 {
        content.push_str(&format!("// Test line {}\n", i));
    }
    fs::write(tests_dir.join("huge_acceptance.rs"), content).unwrap();

    let cfg = ArchitectureConfig {
        max_file_lines: Some(800),
        max_test_file_lines: Some(1000),
        forbidden_tokens: vec![],
        forbidden_imports: vec![],
        exemptions: vec![],
    };

    let report = run_architecture_check(ws, &cfg).expect("run architecture check");
    assert!(!report.is_valid);
    assert!(report.findings.iter().any(|f| {
        f.category == ArchitectureFindingCategory::TestFileExceedsMaxLines
            && f.file_path.contains("huge_acceptance.rs")
    }));
}

#[test]
fn test_architecture_check_honors_explicit_exemption() {
    let temp = TempDir::new("arch_exemption");
    let ws = temp.path();

    let src_dir = ws.join("src");
    fs::create_dir_all(&src_dir).unwrap();

    // Generate 950 lines (exceeds default 800, but under exemption ceiling 1200)
    let mut content = String::new();
    for i in 0..950 {
        content.push_str(&format!("// Parser line {}\n", i));
    }
    fs::write(src_dir.join("parser.rs"), content).unwrap();

    let cfg = ArchitectureConfig {
        max_file_lines: Some(800),
        max_test_file_lines: Some(1000),
        forbidden_tokens: vec![],
        forbidden_imports: vec![],
        exemptions: vec![ArchitectureExemption {
            path: "src/parser.rs".into(),
            reason: "Complex grammar table jf. ADR 0009".into(),
            max_lines: Some(1200),
        }],
    };

    let report = run_architecture_check(ws, &cfg).expect("run architecture check");
    assert!(
        report.is_valid,
        "Exempted file below ceiling must pass validation"
    );
}

#[test]
fn test_architecture_check_fails_if_exemption_ceiling_exceeded() {
    let temp = TempDir::new("arch_exemption_ceiling");
    let ws = temp.path();

    let src_dir = ws.join("src");
    fs::create_dir_all(&src_dir).unwrap();

    // Generate 1300 lines (exceeds exemption ceiling of 1200)
    let mut content = String::new();
    for i in 0..1300 {
        content.push_str(&format!("// Parser line {}\n", i));
    }
    fs::write(src_dir.join("parser.rs"), content).unwrap();

    let cfg = ArchitectureConfig {
        max_file_lines: Some(800),
        max_test_file_lines: Some(1000),
        forbidden_tokens: vec![],
        forbidden_imports: vec![],
        exemptions: vec![ArchitectureExemption {
            path: "src/parser.rs".into(),
            reason: "Complex grammar table jf. ADR 0009".into(),
            max_lines: Some(1200),
        }],
    };

    let report = run_architecture_check(ws, &cfg).expect("run architecture check");
    assert!(!report.is_valid);
    assert!(report.findings.iter().any(|f| {
        f.category == ArchitectureFindingCategory::ExemptionCeilingExceeded
            && f.file_path.contains("parser.rs")
    }));
}

#[test]
fn test_architecture_check_catches_forbidden_tokens() {
    let temp = TempDir::new("arch_tokens");
    let ws = temp.path();

    let src_dir = ws.join("src");
    fs::create_dir_all(&src_dir).unwrap();

    let code = r#"
pub fn make_color() {
    let c = Color::from_rgb(255, 0, 0);
}
"#;
    fs::write(src_dir.join("button.rs"), code).unwrap();

    let cfg = ArchitectureConfig {
        max_file_lines: Some(800),
        max_test_file_lines: Some(1000),
        forbidden_tokens: vec!["Color::from_rgb".into()],
        forbidden_imports: vec![],
        exemptions: vec![],
    };

    let report = run_architecture_check(ws, &cfg).expect("run architecture check");
    assert!(!report.is_valid);
    assert!(report.findings.iter().any(|f| {
        f.category == ArchitectureFindingCategory::ForbiddenTokenDetected
            && f.message.contains("Color::from_rgb")
    }));
}

#[test]
fn test_architecture_check_catches_forbidden_imports() {
    let temp = TempDir::new("arch_imports");
    let ws = temp.path();

    let src_dir = ws.join("src/features/concepts");
    fs::create_dir_all(&src_dir).unwrap();

    let code = r#"
use crate::ui::app::App;

pub fn render() {}
"#;
    fs::write(src_dir.join("view.rs"), code).unwrap();

    let cfg = ArchitectureConfig {
        max_file_lines: Some(800),
        max_test_file_lines: Some(1000),
        forbidden_tokens: vec![],
        forbidden_imports: vec!["crate::ui::app".into()],
        exemptions: vec![],
    };

    let report = run_architecture_check(ws, &cfg).expect("run architecture check");
    assert!(!report.is_valid);
    assert!(report.findings.iter().any(|f| {
        f.category == ArchitectureFindingCategory::ForbiddenImportDetected
            && f.message.contains("crate::ui::app")
    }));
}
