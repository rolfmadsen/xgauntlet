//! Integration tests for Task 031: ADR OKF v0.2 Frontmatter Validation & Template Hardening.

use std::fs;
use std::path::{Path, PathBuf};

use xgauntlet_core::features::diagnostics::FindingType;
use xgauntlet_core::features::scaffold::generate_templates;
use xgauntlet_core::{check_adr_frontmatter, check_all_adrs};

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
        let path = std::env::temp_dir().join(format!("xgauntlet_adr_test_{}", unique));
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
fn test_adr_with_complete_okf_header_validates_ok() {
    let temp = TempDir::new("adr_valid");
    let adr_path = temp.path().join("0001-valid-architecture.md");

    let content = r#"---
type: Architectural Decision Record
title: 'ADR 0001: Valid Architecture'
status: accepted
tags: [architecture, adr]
generated:
  by: human:alice
  at: '2026-09-19T13:50:00Z'
verified: []
---

# 1. Valid Architecture
"#;
    fs::write(&adr_path, content).unwrap();

    let report = check_adr_frontmatter(&adr_path);
    assert!(report.is_valid, "Report should be valid: {:?}", report.diagnostics);
    assert_eq!(report.adr_id, "0001-valid-architecture");
    let errors: Vec<_> = report
        .diagnostics
        .iter()
        .filter(|d| d.finding_type != FindingType::Warning)
        .collect();
    assert!(errors.is_empty(), "Expected no error findings, got: {:?}", errors);
}

#[test]
fn test_adr_without_generated_gives_generated_missing_error() {
    let temp = TempDir::new("adr_missing_gen");
    let adr_path = temp.path().join("0002-missing-gen.md");

    let content = r#"---
type: Architectural Decision Record
title: 'ADR 0002: Missing Generated'
status: draft
tags: [architecture, adr]
verified: []
---

# 2. Missing Generated
"#;
    fs::write(&adr_path, content).unwrap();

    let report = check_adr_frontmatter(&adr_path);
    assert!(!report.is_valid, "Expected report to be invalid");
    assert!(
        report
            .diagnostics
            .iter()
            .any(|d| d.tool_name == "GENERATED_MISSING"),
        "Expected GENERATED_MISSING finding, got: {:?}",
        report.diagnostics
    );
}

#[test]
fn test_adr_with_invalid_actor_gives_invalid_actor_error() {
    let temp = TempDir::new("adr_invalid_actor");
    let adr_path = temp.path().join("0003-invalid-actor.md");

    let content = r#"---
type: Architectural Decision Record
title: 'ADR 0003: Invalid Actor'
status: draft
tags: [architecture, adr]
generated:
  by: invalid_actor_convention
  at: '2026-09-19T13:50:00Z'
verified: []
---

# 3. Invalid Actor
"#;
    fs::write(&adr_path, content).unwrap();

    let report = check_adr_frontmatter(&adr_path);
    assert!(!report.is_valid, "Expected report to be invalid");
    assert!(
        report
            .diagnostics
            .iter()
            .any(|d| d.tool_name == "INVALID_ACTOR"),
        "Expected INVALID_ACTOR finding, got: {:?}",
        report.diagnostics
    );
}

#[test]
fn test_adr_with_invalid_timestamp_gives_invalid_timestamp_error() {
    let temp = TempDir::new("adr_invalid_ts");
    let adr_path = temp.path().join("0004-invalid-ts.md");

    let content = r#"---
type: Architectural Decision Record
title: 'ADR 0004: Invalid Timestamp'
status: draft
tags: [architecture, adr]
generated:
  by: human:bob
  at: 'invalid-date-format'
verified: []
---

# 4. Invalid Timestamp
"#;
    fs::write(&adr_path, content).unwrap();

    let report = check_adr_frontmatter(&adr_path);
    assert!(!report.is_valid, "Expected report to be invalid");
    assert!(
        report
            .diagnostics
            .iter()
            .any(|d| d.tool_name == "INVALID_TIMESTAMP"),
        "Expected INVALID_TIMESTAMP finding, got: {:?}",
        report.diagnostics
    );
}

#[test]
fn test_adr_with_missing_verified_gives_warning_not_error() {
    let temp = TempDir::new("adr_missing_verified");
    let adr_path = temp.path().join("0005-missing-verified.md");

    let content = r#"---
type: Architectural Decision Record
title: 'ADR 0005: Missing Verified'
status: draft
tags: [architecture, adr]
generated:
  by: process:xgauntlet-init
  at: '2026-09-19T13:50:00Z'
---

# 5. Missing Verified
"#;
    fs::write(&adr_path, content).unwrap();

    let report = check_adr_frontmatter(&adr_path);
    assert!(
        report.is_valid,
        "Report should remain valid despite missing verified warning"
    );
    let warnings: Vec<_> = report
        .diagnostics
        .iter()
        .filter(|d| d.finding_type == FindingType::Warning)
        .collect();
    assert_eq!(
        warnings.len(),
        1,
        "Expected exactly 1 warning for missing verified: {:?}",
        report.diagnostics
    );
    assert!(
        warnings[0].tool_name.contains("VERIFIED"),
        "Warning tool_name should reference VERIFIED: {:?}",
        warnings[0]
    );
}

#[test]
fn test_check_all_adrs_skips_template_and_readme() {
    let temp = TempDir::new("adr_bulk_scan");
    let adr_dir = temp.path().join("docs").join("adr");
    fs::create_dir_all(&adr_dir).unwrap();

    // 1. Files to skip
    fs::write(adr_dir.join("README.md"), "# ADR Readme\n").unwrap();
    fs::write(adr_dir.join("template.md"), "# ADR Template\n").unwrap();

    // 2. Real ADRs
    let valid_adr = r#"---
type: Architectural Decision Record
title: 'ADR 0001: First'
status: accepted
tags: [architecture, adr]
generated:
  by: human:alice
  at: '2026-09-19T13:50:00Z'
verified: []
---
"#;
    fs::write(adr_dir.join("0001-first.md"), valid_adr).unwrap();
    fs::write(adr_dir.join("0002-second.md"), valid_adr).unwrap();

    let reports = check_all_adrs(temp.path());
    assert_eq!(reports.len(), 2, "Expected exactly 2 reports, got: {:?}", reports);
    let ids: Vec<_> = reports.iter().map(|r| r.adr_id.as_str()).collect();
    assert!(ids.contains(&"0001-first"));
    assert!(ids.contains(&"0002-second"));
    assert!(!ids.contains(&"README"));
    assert!(!ids.contains(&"template"));
}

#[test]
fn test_scaffold_template_includes_generated_and_verified() {
    let templates = generate_templates("rust", "test-project");
    let adr_tmpl = templates
        .iter()
        .find(|t| t.relative_path == "docs/adr/template.md")
        .expect("docs/adr/template.md should be in scaffold templates");

    assert!(
        adr_tmpl.content.contains("generated:"),
        "Scaffold template should contain 'generated:' block"
    );
    assert!(
        adr_tmpl.content.contains("by: human:<author>"),
        "Scaffold template should contain 'by: human:<author>'"
    );
    assert!(
        adr_tmpl.content.contains("verified: []"),
        "Scaffold template should contain 'verified: []'"
    );
}

#[test]
fn test_xgauntlet_own_adrs_validate_cleanly() {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let reports = check_all_adrs(workspace);
    assert!(
        !reports.is_empty(),
        "Should find xGauntlet's own ADRs in docs/adr/"
    );

    for report in &reports {
        assert!(
            report.is_valid,
            "ADR {} failed validation: {:?}",
            report.adr_id, report.diagnostics
        );
        let errors: Vec<_> = report
            .diagnostics
            .iter()
            .filter(|d| d.finding_type != FindingType::Warning)
            .collect();
        assert!(
            errors.is_empty(),
            "ADR {} has errors: {:?}",
            report.adr_id, errors
        );
    }
}
