//! Conformance and acceptance tests for Task 023:
//! 7-Step Pipeline JIT Governance & Global Plugin Distribution.

use std::fs;
use std::path::{Path, PathBuf};
use xgauntlet_core::features::doctor::{
    run_doctor, DoctorCategory, DoctorOptions,
};
use xgauntlet_core::features::plugin::{
    discover_harnesses_in, get_embedded_plugin_manifest, get_embedded_skill,
    list_embedded_skills, run_plugin_install, PlatformTarget, PluginInstallOptions,
    ALL_EMBEDDED_SKILLS,
};
use xgauntlet_core::features::scaffold::generate_templates;
use xgauntlet_core::features::telemetry::{
    get_phase_directive, render_jit_directive, PipelinePhase,
};

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
        let path = std::env::temp_dir().join(format!("xgauntlet_plugin_test_{}", unique));
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

fn find_xgauntlet_binary() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let candidates = [
        manifest_dir.join("../../target/debug/xgauntlet"),
        manifest_dir.join("../../target/release/xgauntlet"),
        manifest_dir.join("../../target/debug/xgauntlet.exe"),
        manifest_dir.join("../../target/release/xgauntlet.exe"),
    ];
    for c in candidates {
        if c.is_file() {
            return c;
        }
    }
    PathBuf::from("xgauntlet")
}

// ============================================================================
// 1. Omdøbning til .agents/plugins/xgauntlet/ & Indlejring af 11-Skill Suite
// ============================================================================

#[test]
fn test_embedded_11_skill_suite_and_plugin_rename() {
    // 1. All 11 canonical skills must be listed in ALL_EMBEDDED_SKILLS
    let expected_skills = [
        "grill-me",
        "grill-with-docs",
        "domain-modeling",
        "to-spec",
        "to-tasks",
        "old-coder",
        "diagnose",
        "codebase-design",
        "improve-codebase-architecture",
        "code-review",
        "retro",
    ];

    assert_eq!(
        ALL_EMBEDDED_SKILLS.len(),
        11,
        "ALL_EMBEDDED_SKILLS must define exactly 11 canonical skills"
    );
    for skill in &expected_skills {
        assert!(
            ALL_EMBEDDED_SKILLS.contains(skill),
            "ALL_EMBEDDED_SKILLS must contain '{skill}'"
        );
    }

    // 2. list_embedded_skills() must expose all 11 skills
    let embedded = list_embedded_skills();
    assert_eq!(
        embedded.len(),
        11,
        "list_embedded_skills must return all 11 skills, found: {:?}",
        embedded
    );

    // 3. Every single skill must have embedded non-empty Markdown with frontmatter
    for skill in &expected_skills {
        let content = get_embedded_skill(skill)
            .unwrap_or_else(|| panic!("Skill '{skill}' must be embedded via include_str!"));
        assert!(
            content.starts_with("---\n"),
            "Skill '{skill}' must have valid YAML frontmatter header"
        );
        assert!(
            content.contains("name:"),
            "Skill '{skill}' must contain name in frontmatter"
        );
        assert!(
            content.contains("description:"),
            "Skill '{skill}' must contain description in frontmatter"
        );
    }

    // 4. Embedded plugin manifest plugin.json must have name 'xgauntlet' and 11 skills
    let manifest_str = get_embedded_plugin_manifest();
    let manifest: serde_json::Value =
        serde_json::from_str(manifest_str).expect("Embedded plugin.json must be valid JSON");
    assert_eq!(
        manifest["name"].as_str(),
        Some("xgauntlet"),
        "Plugin manifest 'name' must be renamed to 'xgauntlet'"
    );

    let manifest_skills = manifest["skills"]
        .as_array()
        .expect("Plugin manifest must contain 'skills' array");
    assert_eq!(
        manifest_skills.len(),
        11,
        "Plugin manifest skills array must contain 11 skills"
    );

    // 5. Repository folder .agents/plugins/xgauntlet/ must exist on disk and agent-gauntlet removed
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let old_plugin_dir = repo_root.join(".agents/plugins/agent-gauntlet");
    let new_plugin_dir = repo_root.join(".agents/plugins/xgauntlet");

    assert!(
        new_plugin_dir.is_dir(),
        "Repository must contain renamed directory .agents/plugins/xgauntlet/"
    );
    assert!(
        !old_plugin_dir.exists(),
        "Legacy directory .agents/plugins/agent-gauntlet/ must be removed"
    );
}

// ============================================================================
// 2. Cross-Platform Harness Discovery Engine (Linux, macOS, Windows)
// ============================================================================

#[test]
fn test_cross_platform_harness_discovery_linux() {
    let temp = TempDir::new("harness_discovery_linux");
    let home = temp.path();

    // Setup mock Linux harness directories
    let gemini_plugin = home.join(".gemini/config/plugins/xgauntlet");
    let claude_skills = home.join(".claude/skills/xgauntlet");
    let codex_plugins = home.join(".codex/plugins/xgauntlet");
    let vibe_plugins = home.join(".vibe/plugins/xgauntlet");

    fs::create_dir_all(&gemini_plugin).unwrap();
    fs::create_dir_all(&claude_skills).unwrap();
    fs::create_dir_all(&codex_plugins).unwrap();
    fs::create_dir_all(&vibe_plugins).unwrap();

    let discovered = discover_harnesses_in(home, PlatformTarget::Linux);
    assert_eq!(
        discovered.len(),
        4,
        "Linux discovery must identify all 4 canonical harnesses when present"
    );

    let names: Vec<_> = discovered.iter().map(|d| d.name.as_str()).collect();
    assert!(names.contains(&"antigravity"));
    assert!(names.contains(&"claude_code"));
    assert!(names.contains(&"codex"));
    assert!(names.contains(&"mistral"));

    for h in &discovered {
        assert!(h.detected, "Harness '{}' must be marked detected", h.name);
        assert!(
            h.plugin_installed,
            "Harness '{}' plugin must be marked installed",
            h.name
        );
    }
}

#[test]
fn test_cross_platform_harness_discovery_macos_and_windows() {
    let temp = TempDir::new("harness_discovery_mac_win");
    let home = temp.path();

    // macOS Discovery
    let mac_discovered = discover_harnesses_in(home, PlatformTarget::MacOS);
    assert_eq!(
        mac_discovered.len(),
        4,
        "macOS discovery must return 4 harness descriptors"
    );
    for h in &mac_discovered {
        assert!(!h.detected, "Non-existent harness '{}' must not be detected", h.name);
        assert!(!h.plugin_installed);
    }

    // Windows Discovery
    let win_discovered = discover_harnesses_in(home, PlatformTarget::Windows);
    assert_eq!(
        win_discovered.len(),
        4,
        "Windows discovery must return 4 harness descriptors"
    );
    for h in &win_discovered {
        assert!(!h.detected, "Non-existent harness '{}' must not be detected", h.name);
        assert!(!h.plugin_installed);
    }
}

// ============================================================================
// 3. CLI Subcommand 'xgauntlet plugin install'
// ============================================================================

#[test]
fn test_plugin_install_target_and_dry_run() {
    let temp = TempDir::new("plugin_install_target");
    let target = temp.path().join("installed_plugin");

    // 1. Dry run execution
    let dry_options = PluginInstallOptions {
        global: false,
        harness: None,
        dry_run: true,
        force: false,
        target: Some(target.clone()),
        json: false,
    };
    let dry_report = run_plugin_install(&dry_options).expect("Dry-run install must succeed");
    assert!(dry_report.is_dry_run);
    assert!(dry_report.success);
    assert!(
        !target.exists(),
        "Dry-run must not create target directory on disk"
    );

    // 2. Real installation execution
    let install_options = PluginInstallOptions {
        global: false,
        harness: None,
        dry_run: false,
        force: false,
        target: Some(target.clone()),
        json: false,
    };
    let report = run_plugin_install(&install_options).expect("Real install must succeed");
    assert!(!report.is_dry_run);
    assert!(report.success);

    // Check plugin.json and all 11 skills exist on disk
    assert!(target.join("plugin.json").is_file());
    assert!(target.join("hooks.json").is_file());
    for skill in ALL_EMBEDDED_SKILLS {
        let skill_file = target.join(format!("skills/{skill}/SKILL.md"));
        assert!(
            skill_file.is_file(),
            "Installed skill file '{}' must exist on disk",
            skill_file.display()
        );
    }
}

#[test]
fn test_plugin_install_safe_non_destructive_and_force() {
    let temp = TempDir::new("plugin_install_safe");
    let target = temp.path().join("plugin");
    fs::create_dir_all(target.join("skills/old-coder")).unwrap();

    let custom_content = "# CUSTOM USER MODIFIED OLD CODER SKILL";
    let old_coder_path = target.join("skills/old-coder/SKILL.md");
    fs::write(&old_coder_path, custom_content).unwrap();

    // 1. Install without force: must preserve user-modified skill
    let options_safe = PluginInstallOptions {
        global: false,
        harness: None,
        dry_run: false,
        force: false,
        target: Some(target.clone()),
        json: false,
    };
    let report_safe = run_plugin_install(&options_safe).expect("Install should succeed");
    assert!(report_safe.success);
    assert_eq!(
        fs::read_to_string(&old_coder_path).unwrap(),
        custom_content,
        "Non-destructive install MUST preserve existing custom skills when force=false"
    );

    // 2. Install with force: must overwrite
    let options_force = PluginInstallOptions {
        global: false,
        harness: None,
        dry_run: false,
        force: true,
        target: Some(target.clone()),
        json: false,
    };
    let report_force = run_plugin_install(&options_force).expect("Force install should succeed");
    assert!(report_force.success);
    assert_ne!(
        fs::read_to_string(&old_coder_path).unwrap(),
        custom_content,
        "Force install MUST overwrite existing skills when force=true"
    );
}

#[test]
fn test_cli_plugin_install_subcommand() {
    let bin = find_xgauntlet_binary();
    if !bin.is_file() {
        return;
    }

    let temp = TempDir::new("cli_plugin_install");
    let target = temp.path().join("cli_target");

    let output = std::process::Command::new(&bin)
        .args([
            "plugin",
            "install",
            "--target",
            target.to_str().unwrap(),
            "--dry-run",
            "--json",
        ])
        .output()
        .expect("xgauntlet plugin install command execution");

    assert!(
        output.status.success(),
        "CLI plugin install --dry-run must succeed, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("CLI output must be valid JSON");
    assert_eq!(parsed["is_dry_run"].as_bool(), Some(true));
    assert_eq!(parsed["success"].as_bool(), Some(true));
}

// ============================================================================
// 4. De 7 JIT Fasedirektiver & Specialiserede Roller
// ============================================================================

#[test]
fn test_seven_jit_phase_directives_roles_targets_and_gates() {
    let all_phases = PipelinePhase::all();
    assert_eq!(
        all_phases.len(),
        7,
        "There must be exactly 7 canonical pipeline phases"
    );

    for phase in all_phases {
        let directive = get_phase_directive(*phase)
            .unwrap_or_else(|| panic!("Directive for phase {:?} must be implemented", phase));

        // 1. Explicit Active Role check
        assert!(
            directive.active_role.starts_with("Active Role:"),
            "Phase {:?} active_role must start with 'Active Role:', found: {}",
            phase,
            directive.active_role
        );
        assert!(
            directive.active_role.contains('(') && directive.active_role.contains(')'),
            "Phase {:?} active_role must contain functional anchor in parentheses",
            phase
        );

        // 2. Positive Target check (0 negative words / prohibitions)
        let forbidden_negations = ["not", "never", "must not", "don't", "ikke", "forbud"];
        let target_lower = directive.target.to_ascii_lowercase();
        for neg in forbidden_negations {
            assert!(
                !target_lower.contains(neg),
                "Phase {:?} target contains negation '{}': '{}'",
                phase,
                neg,
                directive.target
            );
        }

        // 3. Checkable Gate (Done) check
        assert!(
            !directive.gate.is_empty(),
            "Phase {:?} gate must not be empty",
            phase
        );

        // 4. Front-loaded Context Pointer check
        assert!(
            !directive.pointer.is_empty(),
            "Phase {:?} pointer must not be empty",
            phase
        );

        // 5. Token Budget check (<45 tokens)
        let tokens = directive.estimate_token_count();
        assert!(
            tokens < 45,
            "Phase {:?} directive token count {} exceeds 45 token budget",
            phase,
            tokens
        );

        // 6. Rendered output contract
        let rendered = render_jit_directive(*phase, Some("023"));
        assert!(rendered.contains("Phase:"));
        assert!(rendered.contains("Active Role:"));
        assert!(rendered.contains("Target:"));
        assert!(rendered.contains("Gate (Done):"));
        assert!(rendered.contains("Pointer:"));
    }
}

#[test]
fn test_jit_clean_worktree_guarantee_and_session_handoff() {
    // Phase 6: Evidence Integrity & Drift Check
    let p6 = get_phase_directive(PipelinePhase::EvidenceIntegrityAndDriftCheck)
        .expect("Phase 6 directive");
    assert!(
        p6.target.contains("commit final sealed task checkpoint")
            || p6.gate.contains("local checkpoint committed"),
        "Phase 6 must enforce atomic local checkpoint commit"
    );
    assert!(
        p6.pointer.contains("xgauntlet checkpoint --phase done"),
        "Phase 6 pointer must trigger 'xgauntlet checkpoint --phase done'"
    );

    // Phase 7: Release Readiness
    let p7 = get_phase_directive(PipelinePhase::ReleaseReadiness).expect("Phase 7 directive");
    assert!(
        p7.target.contains("100% clean git worktree"),
        "Phase 7 target must require 100% clean git worktree"
    );
    assert!(
        p7.gate.contains("Git worktree confirmed clean")
            && p7.gate.contains("SESSION HANDOFF"),
        "Phase 7 gate must require clean worktree and SESSION HANDOFF card"
    );
}

// ============================================================================
// 5. Decoupling af Forretnings-ADRs (docs/adr/template.md i stedet for 0001)
// ============================================================================

#[test]
fn test_decoupling_of_business_adrs_scaffolds_template() {
    let templates = generate_templates("rust", "test-decouple-app");

    // 1. Must scaffold docs/adr/template.md
    let template_entry = templates
        .iter()
        .find(|t| t.relative_path == "docs/adr/template.md");
    assert!(
        template_entry.is_some(),
        "generate_templates must generate neutral 'docs/adr/template.md'"
    );

    // 2. Must NOT occupy docs/adr/0001-package-by-feature-architecture.md
    let adr_0001_entry = templates
        .iter()
        .find(|t| t.relative_path == "docs/adr/0001-package-by-feature-architecture.md");
    assert!(
        adr_0001_entry.is_none(),
        "generate_templates MUST NOT occupy 'docs/adr/0001-package-by-feature-architecture.md'"
    );

    // 3. Neutral ADR template content check
    let template_content = &template_entry.unwrap().content;
    assert!(
        template_content.contains("# [ADR Number]. [Short Title]"),
        "Template must have neutral ADR title placeholder"
    );
    assert!(
        template_content.contains("## Context") && template_content.contains("## Decision"),
        "Template must contain Context and Decision sections"
    );

    // 4. .agents/AGENTS.md must NOT reference local 0003 ADR file
    let agents_md = templates
        .iter()
        .find(|t| t.relative_path == ".agents/AGENTS.md")
        .expect(".agents/AGENTS.md template");
    assert!(
        !agents_md.content.contains("0003-surgical-gatekeeper-and-no-remote-push.md"),
        ".agents/AGENTS.md must NOT reference broken local docs/adr/0003-... file"
    );
}

// ============================================================================
// 6. 'xgauntlet doctor' Harnesses-Kategori
// ============================================================================

#[test]
fn test_doctor_harnesses_category() {
    // 1. Category naming and parsing
    assert_eq!(DoctorCategory::Harnesses.as_str(), "harnesses");
    assert_eq!(
        DoctorCategory::parse_str("harnesses"),
        Some(DoctorCategory::Harnesses)
    );
    assert_eq!(
        DoctorCategory::parse_str("harness"),
        Some(DoctorCategory::Harnesses)
    );

    // 2. Diagnostic execution filtered by Harnesses
    let temp = TempDir::new("doctor_harnesses");
    let options = DoctorOptions {
        workspace: temp.path().to_path_buf(),
        verbose: true,
        strict: false,
        category: Some(DoctorCategory::Harnesses),
    };

    let report = run_doctor(&options).expect("Doctor harnesses execution must succeed");
    assert!(
        !report.checks.is_empty(),
        "run_doctor with category Harnesses must return diagnostic check items"
    );

    for check in &report.checks {
        assert_eq!(
            check.category,
            DoctorCategory::Harnesses,
            "All checks returned must belong to Harnesses category"
        );
    }

    let harness_names = ["antigravity", "claude_code", "codex", "mistral"];
    for name in harness_names {
        assert!(
            report.checks.iter().any(|c| c.name.contains(name)),
            "Doctor must inspect harness '{}'",
            name
        );
    }
}
