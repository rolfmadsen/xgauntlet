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
pub fn list_embedded_skills() -> &'static [&'static str] {
    ALL_EMBEDDED_SKILLS
}

/// Returns content of an embedded skill.
pub fn get_embedded_skill(name: &str) -> Option<&'static str> {
    match name {
        "grill-me" => Some(include_str!("../../../../../.agents/plugins/xgauntlet/skills/grill-me/SKILL.md")),
        "grill-with-docs" => Some(include_str!("../../../../../.agents/plugins/xgauntlet/skills/grill-with-docs/SKILL.md")),
        "domain-modeling" => Some(include_str!("../../../../../.agents/plugins/xgauntlet/skills/domain-modeling/SKILL.md")),
        "to-spec" => Some(include_str!("../../../../../.agents/plugins/xgauntlet/skills/to-spec/SKILL.md")),
        "to-tasks" => Some(include_str!("../../../../../.agents/plugins/xgauntlet/skills/to-tasks/SKILL.md")),
        "old-coder" => Some(include_str!("../../../../../.agents/plugins/xgauntlet/skills/old-coder/SKILL.md")),
        "diagnose" => Some(include_str!("../../../../../.agents/plugins/xgauntlet/skills/diagnose/SKILL.md")),
        "codebase-design" => Some(include_str!("../../../../../.agents/plugins/xgauntlet/skills/codebase-design/SKILL.md")),
        "improve-codebase-architecture" => Some(include_str!("../../../../../.agents/plugins/xgauntlet/skills/improve-codebase-architecture/SKILL.md")),
        "code-review" => Some(include_str!("../../../../../.agents/plugins/xgauntlet/skills/code-review/SKILL.md")),
        "retro" => Some(include_str!("../../../../../.agents/plugins/xgauntlet/skills/retro/SKILL.md")),
        _ => None,
    }
}

/// Returns embedded plugin manifest.
pub fn get_embedded_plugin_manifest() -> &'static str {
    include_str!("../../../../../.agents/plugins/xgauntlet/plugin.json")
}

/// Returns embedded hooks manifest.
pub fn get_embedded_hooks_manifest() -> &'static str {
    include_str!("../../../../../.agents/plugins/xgauntlet/hooks.json")
}
