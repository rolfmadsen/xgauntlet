//! Markdown wrapper generators for GitHub and IDE Mermaid Preview rendering.

use super::mermaid::{
    discover_topology_features, export_feature_mermaid_class_diagram, export_mermaid_class_diagram,
    MermaidExportOptions, MermaidLevel,
};
use super::models::TopologyOptions;
use super::parser::scan_workspace_topology;
use std::path::Path;

/// Builds a 1-click `https://mermaid.live/view#base64:...` URL encoded deterministically
/// so Markdown previews without Mermaid extensions can open the diagram directly in a browser.
pub fn mermaid_live_url(mermaid_source: &str) -> String {
    let payload = serde_json::json!({
        "code": mermaid_source.trim_end(),
        "mermaid": { "theme": "default" },
        "autoSync": true,
        "updateEditor": false
    });
    let json_bytes = payload.to_string().into_bytes();
    let encoded = base64_encode(&json_bytes);
    format!("https://mermaid.live/view#base64:{encoded}")
}

fn base64_encode(input: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(input.len().div_ceil(3) * 4);
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[((triple >> 18) & 0x3F) as usize] as char);
        out.push(TABLE[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            out.push(TABLE[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(TABLE[(triple & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

/// Renders the root `architecture.md` document wrapping the canonical Mermaid diagram
/// alongside direct links to feature-scoped diagrams under `docs/architecture/`.
pub fn render_architecture_markdown(mermaid_source: &str, features: &[String]) -> String {
    let live_url = mermaid_live_url(mermaid_source);
    let mut out = String::new();
    out.push_str("# xGauntlet Domain & Architecture Diagram\n\n");
    out.push_str(&format!(
        "🌐 **[Open Interactive Diagram (Mermaid Live)]({live_url})** • 💻 `xgauntlet topology open`\n\n"
    ));
    out.push_str(
        "> Automatisk synkroniseret fra kodebasens AST via `xgauntlet verify --save`.\n\n",
    );

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
    let live_url = mermaid_live_url(mermaid_source);
    let mut out = String::new();
    out.push_str(&format!("# Feature Architecture: `{feature}`\n\n"));
    out.push_str(&format!(
        "🌐 **[Open Interactive Diagram (Mermaid Live)]({live_url})** • ← [Full Architecture](../../architecture.md) • 📋 [Feature Spec](../../specs/{feature}.md)\n\n"
    ));
    out.push_str(&format!(
        "> Scoped domænemodel for `{feature}` (inkl. direkte 1-hop naborelationer). ",
    ));
    out.push_str(&format!(
        "Åbn lokalt med pan/zoom via `xgauntlet topology open --feature {feature}`.\n\n"
    ));
    out.push_str("```mermaid\n");
    out.push_str(mermaid_source.trim_end());
    out.push_str("\n```\n");
    out
}

/// Synchronizes `diagram_file` (`architecture.mermaid`), `architecture.md`, and
/// `docs/architecture/<feature>.md` for all discovered domain features.
pub fn sync_workspace_diagrams(workspace: &Path, diagram_file: &str) {
    let trimmed = diagram_file.trim();
    if trimmed.is_empty() {
        return;
    }
    let topology_opts = TopologyOptions {
        workspace_root: workspace.to_path_buf(),
        ..Default::default()
    };
    let Ok(graph) = scan_workspace_topology(&topology_opts) else {
        return;
    };
    let export_opts = MermaidExportOptions {
        direction: "TB".to_string(),
        level: MermaidLevel::Conceptual,
        domain_only: true,
    };
    let diagram = export_mermaid_class_diagram(&graph, &export_opts);
    let _ = std::fs::write(workspace.join(trimmed), &diagram);

    let features = discover_topology_features(&graph, &export_opts);
    let arch_dir = workspace.join("docs/architecture");
    if !features.is_empty() && std::fs::create_dir_all(&arch_dir).is_ok() {
        for feat in &features {
            if let Ok(feat_diagram) =
                export_feature_mermaid_class_diagram(&graph, &export_opts, feat)
            {
                let feat_md = render_feature_architecture_markdown(feat, &feat_diagram);
                let _ = std::fs::write(arch_dir.join(format!("{feat}.md")), feat_md);
            }
        }
    }
    let arch_md = render_architecture_markdown(&diagram, &features);
    let _ = std::fs::write(workspace.join("architecture.md"), arch_md);
}
