# HTML Report Format

The architectural review is rendered as a single self-contained HTML file in the OS temp directory. Tailwind and Mermaid both come from CDNs.

## Scaffold

```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <title>Architecture Review</title>
    <script src="https://cdn.tailwindcss.com"></script>
    <script type="module">
      import mermaid from "https://cdn.jsdelivr.net/npm/mermaid@11/dist/mermaid.esm.min.mjs";
      mermaid.initialize({ startOnLoad: true, theme: "neutral", securityLevel: "loose" });
    </script>
    <style>
      .seam { stroke-dasharray: 4 4; }
      .leak { stroke: #dc2626; }
      .deep { background: linear-gradient(135deg, #0f172a, #1e293b); }
    </style>
  </head>
  <body class="bg-stone-50 text-slate-900 font-sans">
    <main class="max-w-5xl mx-auto px-6 py-12 space-y-12">
      <header class="border-b pb-6">
        <h1 class="text-3xl font-bold font-serif">Architecture Review</h1>
      </header>
      <section id="candidates" class="space-y-10"></section>
      <section id="top-recommendation" class="p-6 bg-emerald-50 border border-emerald-200 rounded-xl"></section>
    </main>
  </body>
</html>
```

## Candidate Card Checklist

- **Title**: short, names the deepening (e.g. "Collapse the Order intake pipeline").
- **Badge row**: recommendation strength (`Strong`, `Worth exploring`, `Speculative`).
- **Files**: monospaced list.
- **Before / After diagram**: Mermaid `flowchart LR` or cross-section.
- **Problem**: one sentence. What hurts.
- **Solution**: one sentence. What changes.
- **Wins**: bullets, ≤6 words each, strictly in glossary terms (*locality*, *leverage*).
