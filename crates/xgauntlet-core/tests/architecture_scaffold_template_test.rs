use std::fs;
use std::path::Path;
use xgauntlet_core::features::config::parse_toml;
use xgauntlet_core::features::scaffold::generate_templates;

#[test]
fn test_scaffold_gauntlet_toml_contains_architecture_guidance() {
    let templates = generate_templates("rust", "test_proj");
    let gauntlet_toml = templates
        .iter()
        .find(|t| t.relative_path == "gauntlet.toml")
        .expect("gauntlet.toml should be in scaffold templates");

    assert!(
        gauntlet_toml
            .content
            .contains("# layers = [\"app\", \"features\", \"core\", \"shared\"]"),
        "scaffolded gauntlet.toml should contain commented example for layers"
    );
    assert!(
        gauntlet_toml
            .content
            .contains("# slice_root = \"src/features\""),
        "scaffolded gauntlet.toml should contain commented example for slice_root"
    );
    assert!(
        gauntlet_toml.content.contains("# isolate_slices = true"),
        "scaffolded gauntlet.toml should contain commented example for isolate_slices"
    );
    assert!(
        gauntlet_toml.content.contains("# forbid_core_imports ="),
        "scaffolded gauntlet.toml should contain commented example for forbid_core_imports"
    );

    // Verify it still parses cleanly
    let parsed = parse_toml(&gauntlet_toml.content).expect("scaffolded TOML must parse cleanly");
    assert_eq!(
        parsed.architecture.layers.len(),
        0,
        "layers must remain unconfigured/empty by default"
    );
    assert!(
        !parsed.architecture.isolate_slices,
        "isolate_slices must remain false by default"
    );
}

#[test]
fn test_readme_documents_architecture_gates_and_safe_mode() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    let root_readme =
        fs::read_to_string(repo_root.join("README.md")).expect("README.md must exist");
    let cli_readme = fs::read_to_string(repo_root.join("packages/cli/README.md"))
        .expect("cli README must exist");

    for (name, content) in [
        ("README.md", &root_readme),
        ("packages/cli/README.md", &cli_readme),
    ] {
        assert!(
            content.contains("Safe-Mode") || content.contains("safe-mode"),
            "{} should document safe-mode for brownfield compatibility",
            name
        );
        assert!(
            content.contains("Gate A") && content.contains("Gate B") && content.contains("Gate C"),
            "{} should document Gate A, Gate B, and Gate C",
            name
        );
        assert!(
            content.contains("isolate_slices") && content.contains("forbid_core_imports"),
            "{} should document configuration tokens isolate_slices and forbid_core_imports",
            name
        );
    }
}

#[test]
fn test_readme_documents_brownfield_migration_blueprint() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    let root_readme =
        fs::read_to_string(repo_root.join("README.md")).expect("README.md must exist");
    let cli_readme = fs::read_to_string(repo_root.join("packages/cli/README.md"))
        .expect("cli README must exist");

    for (name, content) in [
        ("README.md", &root_readme),
        ("packages/cli/README.md", &cli_readme),
    ] {
        assert!(
            content.contains("Architecture Baseline Audit") || content.contains("Baseline Audit"),
            "{} should document the architecture baseline audit step",
            name
        );
        assert!(
            content.contains("Big-Bang")
                || content.contains("Big Bang")
                || content.contains("big-bang"),
            "{} should document avoiding the big-bang trap via baseline exemptions",
            name
        );
        assert!(
            content.contains("Action") || content.contains("Action/Event"),
            "{} should document decoupling features via local Action/Event enums",
            name
        );
    }
}
