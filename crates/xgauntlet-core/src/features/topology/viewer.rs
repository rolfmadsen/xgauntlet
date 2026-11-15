//! Zero-Daemon HTML viewer generator powered by Pele (`obsidianmd/pele`).

use super::models::TopologyError;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Renders a self-contained static HTML viewer using `obsidianmd/pele` (`classDiagram`)
/// with pan/zoom controls, WCAG AA contrast tokens, dark/light theme support, and raw fallback.
pub fn render_pele_viewer_html(
    mermaid_source: &str,
    feature: Option<&str>,
    level_label: &str,
) -> String {
    let scope_badge = feature
        .map(|f| format!("feature: {f}"))
        .unwrap_or_else(|| "full workspace".to_string());
    let title_suffix = feature.map(|f| format!(" — {f}")).unwrap_or_default();

    // Prevent premature </script> termination inside embedded <script type="text/plain">
    let safe_source = mermaid_source.replace("</script>", "<\\/script>");
    let escaped_html_source = html_escape(mermaid_source);
    let escaped_scope = html_escape(&scope_badge);
    let escaped_level = html_escape(level_label);
    let escaped_title = html_escape(&title_suffix);

    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>xGauntlet Architecture Viewer{escaped_title}</title>
  <style>
    :root {{
      --bg-surface: #ffffff;
      --bg-elevated: #f3f4f6;
      --bg-canvas: #fafafa;
      --text-primary: #111827;
      --text-secondary: #374151;
      --border-subtle: #d1d5db;
      --accent: #1d4ed8;
      --accent-text: #ffffff;
      --badge-bg: #e5e7eb;
      --badge-text: #111827;
      --error-bg: #fef2f2;
      --error-text: #991b1b;
    }}
    @media (prefers-color-scheme: dark) {{
      :root {{
        --bg-surface: #0f172a;
        --bg-elevated: #1e293b;
        --bg-canvas: #0b1120;
        --text-primary: #f8fafc;
        --text-secondary: #cbd5e1;
        --border-subtle: #334155;
        --accent: #3b82f6;
        --accent-text: #0f172a;
        --badge-bg: #334155;
        --badge-text: #f8fafc;
        --error-bg: #450a0a;
        --error-text: #fecaca;
      }}
    }}
    * {{ box-sizing: border-box; }}
    html, body {{
      margin: 0;
      padding: 0;
      width: 100%;
      height: 100%;
      background: var(--bg-surface);
      color: var(--text-primary);
      font-family: ui-sans-serif, system-ui, -apple-system, Segoe UI, Roboto, sans-serif;
      overflow: hidden;
    }}
    header {{
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 12px;
      padding: 10px 16px;
      background: var(--bg-elevated);
      border-bottom: 1px solid var(--border-subtle);
      flex-wrap: wrap;
    }}
    .brand {{
      display: flex;
      align-items: center;
      gap: 8px;
      font-weight: 700;
      font-size: 14px;
    }}
    .badge {{
      background: var(--badge-bg);
      color: var(--badge-text);
      padding: 2px 8px;
      border-radius: 999px;
      font-size: 12px;
      font-weight: 600;
    }}
    .controls {{
      display: flex;
      align-items: center;
      gap: 8px;
      flex-wrap: wrap;
    }}
    button {{
      background: var(--bg-surface);
      color: var(--text-primary);
      border: 1px solid var(--border-subtle);
      border-radius: 6px;
      padding: 6px 10px;
      font-size: 12px;
      font-weight: 600;
      cursor: pointer;
    }}
    button:hover {{ border-color: var(--accent); }}
    button.primary {{
      background: var(--accent);
      color: var(--accent-text);
      border-color: var(--accent);
    }}
    #viewport {{
      position: relative;
      width: 100%;
      height: calc(100% - 54px);
      background: var(--bg-canvas);
      overflow: hidden;
      cursor: grab;
    }}
    #viewport.dragging {{ cursor: grabbing; }}
    #pan-zoom-layer {{
      transform-origin: 0 0;
      display: inline-block;
      padding: 32px;
    }}
    #mermaid-fallback {{
      display: none;
      max-width: 960px;
      margin: 24px auto;
      padding: 16px;
      background: var(--error-bg);
      color: var(--error-text);
      border: 1px solid var(--border-subtle);
      border-radius: 8px;
      overflow: auto;
      max-height: calc(100% - 48px);
    }}
    #mermaid-fallback pre {{
      margin: 12px 0 0;
      padding: 12px;
      background: var(--bg-surface);
      color: var(--text-primary);
      border-radius: 6px;
      overflow: auto;
      font-size: 12px;
      line-height: 1.45;
    }}
  </style>
</head>
<body>
  <header>
    <div class="brand">
      <span>xGauntlet Pele Viewer</span>
      <span class="badge">{escaped_scope}</span>
      <span class="badge">{escaped_level}</span>
      <span class="badge" id="render-status">Rendering...</span>
    </div>
    <div class="controls">
      <button type="button" id="zoom-out" aria-label="Zoom out">-</button>
      <span id="zoom-label" class="badge">100%</span>
      <button type="button" id="zoom-in" aria-label="Zoom in">+</button>
      <button type="button" id="zoom-reset">Reset</button>
      <button type="button" id="copy-mermaid" class="primary">Copy Mermaid</button>
    </div>
  </header>

  <main id="viewport">
    <div id="pan-zoom-layer">
      <div id="diagram-canvas"></div>
    </div>
    <section id="mermaid-fallback" aria-live="polite">
      <strong id="fallback-reason">Offline fallback: rendering raw Mermaid source.</strong>
      <pre>{escaped_html_source}</pre>
    </section>
  </main>

  <script type="text/plain" id="mermaid-source">{safe_source}</script>
  <script type="module">
    const source = document.getElementById('mermaid-source').textContent || '';
    const canvas = document.getElementById('diagram-canvas');
    const fallback = document.getElementById('mermaid-fallback');
    const fallbackReason = document.getElementById('fallback-reason');
    const statusBadge = document.getElementById('render-status');
    const viewport = document.getElementById('viewport');
    const layer = document.getElementById('pan-zoom-layer');
    const zoomLabel = document.getElementById('zoom-label');

    let scale = 1;
    let panX = 0;
    let panY = 0;
    let dragging = false;
    let startX = 0;
    let startY = 0;

    function applyTransform() {{
      layer.style.transform = `translate(${{panX}}px, ${{panY}}px) scale(${{scale}})`;
      zoomLabel.textContent = `${{Math.round(scale * 100)}}%`;
    }}

    document.getElementById('zoom-in').addEventListener('click', () => {{
      scale = Math.min(5, scale * 1.2);
      applyTransform();
    }});
    document.getElementById('zoom-out').addEventListener('click', () => {{
      scale = Math.max(0.15, scale / 1.2);
      applyTransform();
    }});
    document.getElementById('zoom-reset').addEventListener('click', () => {{
      scale = 1;
      panX = 0;
      panY = 0;
      applyTransform();
    }});
    document.getElementById('copy-mermaid').addEventListener('click', async () => {{
      try {{
        await navigator.clipboard.writeText(source);
        statusBadge.textContent = 'Copied!';
      }} catch (_err) {{
        statusBadge.textContent = 'Copy unavailable';
      }}
    }});

    viewport.addEventListener('wheel', (e) => {{
      e.preventDefault();
      const factor = e.deltaY < 0 ? 1.1 : 0.9;
      scale = Math.min(5, Math.max(0.15, scale * factor));
      applyTransform();
    }}, {{ passive: false }});

    viewport.addEventListener('pointerdown', (e) => {{
      if (e.target.closest('button')) return;
      dragging = true;
      startX = e.clientX - panX;
      startY = e.clientY - panY;
      viewport.classList.add('dragging');
    }});
    window.addEventListener('pointermove', (e) => {{
      if (!dragging) return;
      panX = e.clientX - startX;
      panY = e.clientY - startY;
      applyTransform();
    }});
    window.addEventListener('pointerup', () => {{
      dragging = false;
      viewport.classList.remove('dragging');
    }});

    try {{
      // Render synchronously via obsidianmd/pele (MIT, zero runtime deps)
      const pele = await import('https://cdn.jsdelivr.net/npm/pele@1/dist/pele.js');
      const t0 = performance.now();
      const result = pele.render(source);
      const dt = (performance.now() - t0).toFixed(2);
      canvas.innerHTML = result.svg;
      statusBadge.textContent = `Pele: ${{dt}} ms`;
    }} catch (err) {{
      statusBadge.textContent = 'Fallback';
      fallback.style.display = 'block';
      fallbackReason.textContent = `Pele renderer unavailable (${{err?.message || 'offline'}}). Showing raw Mermaid source:`;
    }}
  </script>
</body>
</html>
"##
    )
}

fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Writes the Pele HTML viewer to `.xgauntlet/architecture-viewer.html` (or an explicit path).
pub fn write_pele_viewer_html(
    workspace: &Path,
    out_path: Option<&Path>,
    mermaid_source: &str,
    feature: Option<&str>,
    level_label: &str,
) -> Result<PathBuf, TopologyError> {
    let target = match out_path {
        Some(p) if p.is_absolute() => p.to_path_buf(),
        Some(p) => workspace.join(p),
        None => workspace.join(".xgauntlet/architecture-viewer.html"),
    };
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    let html = render_pele_viewer_html(mermaid_source, feature, level_label);
    fs::write(&target, html)?;
    Ok(target)
}

/// Opens a local file path in the platform's default browser without spawning a daemon.
pub fn open_in_browser(path: &Path) -> Result<(), TopologyError> {
    let abs = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let target = abs.to_string_lossy().to_string();

    #[cfg(target_os = "linux")]
    let mut cmd = {
        let mut c = Command::new("xdg-open");
        c.arg(&target);
        c
    };

    #[cfg(target_os = "macos")]
    let mut cmd = {
        let mut c = Command::new("open");
        c.arg(&target);
        c
    };

    #[cfg(target_os = "windows")]
    let mut cmd = {
        let mut c = Command::new("cmd");
        c.args(["/C", "start", "", &target]);
        c
    };

    cmd.spawn().map_err(TopologyError::Io)?;
    Ok(())
}
