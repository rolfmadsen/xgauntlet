//! Embedded plugin assets and skill catalog.

/// Complete canonical list of the 11 Markdown-skills for xGauntlet jf. Task 023.
pub const ALL_EMBEDDED_SKILLS: &[&str] = &[
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

/// Returns list of embedded skill names.
///
/// In RED phase, returns legacy 5 skills to demonstrate failing acceptance test.
pub fn list_embedded_skills() -> &'static [&'static str] {
    &[
        "old-coder",
        "grill-me",
        "grill-with-docs",
        "diagnose",
        "code-review",
    ]
}

/// Returns content of an embedded skill.
pub fn get_embedded_skill(name: &str) -> Option<&'static str> {
    match name {
        "grill-me" => Some(include_str!("../../../../../.agents/plugins/agent-gauntlet/skills/grill-me/SKILL.md")),
        "grill-with-docs" => Some(include_str!("../../../../../.agents/plugins/agent-gauntlet/skills/grill-with-docs/SKILL.md")),
        "old-coder" => Some(include_str!("../../../../../.agents/plugins/agent-gauntlet/skills/old-coder/SKILL.md")),
        "diagnose" => Some(include_str!("../../../../../.agents/plugins/agent-gauntlet/skills/diagnose/SKILL.md")),
        "code-review" => Some(include_str!("../../../../../.agents/plugins/agent-gauntlet/skills/code-review/SKILL.md")),
        _ => None,
    }
}

/// Returns embedded plugin manifest.
pub fn get_embedded_plugin_manifest() -> &'static str {
    include_str!("../../../../../.agents/plugins/agent-gauntlet/plugin.json")
}

/// Returns embedded hooks manifest.
pub fn get_embedded_hooks_manifest() -> &'static str {
    include_str!("../../../../../.agents/plugins/agent-gauntlet/hooks.json")
}
