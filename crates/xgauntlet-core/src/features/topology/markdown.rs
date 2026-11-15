//! Markdown wrapper generators for GitHub and IDE Mermaid Preview rendering.

/// Renders the root `architecture.md` document wrapping the canonical Mermaid diagram
/// alongside direct links to feature-scoped diagrams under `docs/architecture/`.
pub fn render_architecture_markdown(mermaid_source: &str, features: &[String]) -> String {
    let mut out = String::new();
    out.push_str("# xGauntlet Domain & Architecture Diagram\n\n");
    out.push_str("> Automatisk synkroniseret fra kodebasens AST via `xgauntlet verify --save`. ");
    out.push_str("Kan åbnes direkte med pan/zoom via `xgauntlet topology open`.\n\n");

    if !features.is_empty() {
        out.push_str("## 🧩 Feature-Scoped Diagrams\n\n");
        for feat in features {
            out.push_str(&format!("- [`{feat}`](docs/architecture/{feat}.md)\n"));
        }
        out.push('\n');
    }

    out.push_str("## 🗺️ Full Domain Model\n\n");
    out.push_str("```mermaid\n");
    out.push_str(mermaid_source.trim_end());
    out.push_str("\n```\n");
    out
}

/// Renders a focused `docs/architecture/<feature>.md` Markdown document for one feature slice.
pub fn render_feature_architecture_markdown(feature: &str, mermaid_source: &str) -> String {
    let mut out = String::new();
    out.push_str(&format!("# Feature Architecture: `{feature}`\n\n"));
    out.push_str(&format!(
        "← [Full Architecture](../../architecture.md) • 📋 [Feature Spec](../../specs/{feature}.md)\n\n"
    ));
    out.push_str(&format!(
        "> Scoped domænemodel for `{feature}` (inkl. direkte 1-hop naborelationer). ",
    ));
    out.push_str(&format!(
        "Åbn interaktivt med `xgauntlet topology open --feature {feature}`.\n\n"
    ));
    out.push_str("```mermaid\n");
    out.push_str(mermaid_source.trim_end());
    out.push_str("\n```\n");
    out
}
