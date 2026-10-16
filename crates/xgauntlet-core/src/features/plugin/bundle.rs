//! Embedded plugin assets and skill catalog.

/// Complete canonical list of the 6 sovereign Markdown-skills for xGauntlet jf. Task 034 & ADR 0008.
pub const ALL_EMBEDDED_SKILLS: &[&str] = &[
    "discover",
    "spec",
    "tdd",
    "review",
    "diagnose",
    "release",
];

/// Returns list of embedded skill names.
pub fn list_embedded_skills() -> &'static [&'static str] {
    ALL_EMBEDDED_SKILLS
}

/// Returns content of an embedded skill.
pub fn get_embedded_skill(name: &str) -> Option<&'static str> {
    match name {
        "discover" => Some(include_str!(
            "../../../../../.agents/plugins/xgauntlet/skills/discover/SKILL.md"
        )),
        "spec" => Some(include_str!(
            "../../../../../.agents/plugins/xgauntlet/skills/spec/SKILL.md"
        )),
        "tdd" => Some(include_str!(
            "../../../../../.agents/plugins/xgauntlet/skills/tdd/SKILL.md"
        )),
        "review" => Some(include_str!(
            "../../../../../.agents/plugins/xgauntlet/skills/review/SKILL.md"
        )),
        "diagnose" => Some(include_str!(
            "../../../../../.agents/plugins/xgauntlet/skills/diagnose/SKILL.md"
        )),
        "release" => Some(include_str!(
            "../../../../../.agents/plugins/xgauntlet/skills/release/SKILL.md"
        )),
        _ => None,
    }
}

/// Embedded companion files for skills (e.g. ADR-FORMAT.md, CONTEXT-FORMAT.md).
pub const COMPANION_FILES: &[(&str, &str, &str)] = &[
    (
        "discover",
        "ADR-FORMAT.md",
        include_str!(
            "../../../../../.agents/plugins/xgauntlet/skills/discover/ADR-FORMAT.md"
        ),
    ),
    (
        "discover",
        "CONTEXT-FORMAT.md",
        include_str!(
            "../../../../../.agents/plugins/xgauntlet/skills/discover/CONTEXT-FORMAT.md"
        ),
    ),
];

/// Returns embedded companion files for skills.
pub fn get_embedded_companion_files() -> &'static [(&'static str, &'static str, &'static str)] {
    COMPANION_FILES
}

/// Returns embedded plugin manifest.
pub fn get_embedded_plugin_manifest() -> &'static str {
    include_str!("../../../../../.agents/plugins/xgauntlet/plugin.json")
}

/// Returns embedded hooks manifest.
pub fn get_embedded_hooks_manifest() -> &'static str {
    include_str!("../../../../../.agents/plugins/xgauntlet/hooks.json")
}
