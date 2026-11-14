use std::fs;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("canonicalize workspace root")
}

#[test]
fn test_roadmap_no_subagents_and_aligned_two_axis_review() {
    let ws = workspace_root();
    let roadmap_path = ws.join("ROADMAP.md");
    let content = fs::read_to_string(&roadmap_path).expect("read ROADMAP.md");

    // Must NOT contain references to running subagents or parallel subagents
    assert!(
        !content.to_lowercase().contains("parallelle subagenter"),
        "ROADMAP.md must not contain 'parallelle subagenter'"
    );
    assert!(
        !content.to_lowercase().contains("subagenter parallelt"),
        "ROADMAP.md must not contain 'subagenter parallelt'"
    );
    assert!(
        !content.to_lowercase().contains("uafhængige subagenter"),
        "ROADMAP.md must not contain 'uafhængige subagenter'"
    );

    // Must specify sequential evaluation in dedicated session / Code Reviewer role
    assert!(
        content.contains("Code Reviewer") || content.contains("dedikeret, isoleret session"),
        "ROADMAP.md must describe two-axis review as evaluated by the Code Reviewer role in an isolated session"
    );
}

#[test]
fn test_agents_guidelines_review_skill_aligned() {
    let ws = workspace_root();
    let agents_path = ws.join(".agents/AGENTS.md");
    let content = fs::read_to_string(&agents_path).expect("read .agents/AGENTS.md");

    assert!(
        !content.contains("running independent parallel sub-agents"),
        ".agents/AGENTS.md must not contain 'running independent parallel sub-agents'"
    );
    assert!(
        content.contains("running as an independent code reviewer role in a fresh session with Fowler code smells baseline"),
        ".agents/AGENTS.md must describe review skill running as an independent code reviewer role in a fresh session"
    );
}

#[test]
fn test_review_skill_free_of_subagent_dogmas() {
    let ws = workspace_root();
    let review_skill_path = ws.join(".agents/plugins/xgauntlet/skills/review/SKILL.md");
    let content = fs::read_to_string(&review_skill_path).expect("read review/SKILL.md");

    assert!(
        !content.to_lowercase().contains("subagent"),
        "review/SKILL.md must not contain subagent references"
    );
    assert!(
        !content.to_lowercase().contains("sub-agent"),
        "review/SKILL.md must not contain sub-agent references"
    );
}

#[test]
fn test_readme_and_cli_readme_worktree_multi_agent_architecture_note() {
    let ws = workspace_root();
    let readmes = [ws.join("README.md"), ws.join("packages/cli/README.md")];

    for readme_path in readmes {
        let content = fs::read_to_string(&readme_path).expect("read README file");

        // Under De 4 AI-roller, must document ADR 0014 worktree-isolated multi-agent & delegation model
        assert!(
            content.contains("Worktree-Isoleret Multi-Agent & Subagent Supervision")
                && content.contains("0014-worktree-isolated-multi-agent-and-subagent-supervision.md"),
            "README at {:?} must document Worktree-Isoleret Multi-Agent & Subagent Supervision (ADR 0014)",
            readme_path
        );
        assert!(
            content.contains("xgauntlet task bind")
                && content.contains("workspace: \"inherit\"")
                && content.contains("fail-closed"),
            "README at {:?} must document worktree task bind and fail-closed workspace: inherit protection",
            readme_path
        );
    }
}
