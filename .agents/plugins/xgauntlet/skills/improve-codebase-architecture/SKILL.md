---
name: improve-codebase-architecture
description: Scan a codebase for deepening opportunities, present them as a visual HTML report with Mermaid and Tailwind, then grill through the chosen candidate.
---

# Improve Codebase Architecture

Surface architectural friction and propose **deepening opportunities**: refactors that turn shallow modules into deep ones. The aim is testability, locality, and AI-navigability.

This command is informed by the project's domain model (`CONTEXT.md`) and built on the shared design vocabulary from `codebase-design` (**module**, **interface**, **depth**, **seam**, **adapter**, **leverage**, **locality**).

---

## Process

### 1. Explore

**Scope before you scan: YAGNI.** Deepening a module pays off by making future changes to it easier, so put extra weight on the parts of the codebase that have recently changed. Decide *where* to look before you look:
- If the user named a direction, take it.
- Otherwise, walk back commit history (`git log --oneline -n 30`) to find hot spots—files and crates that keep coming up.

Read `CONTEXT.md` and relevant ADRs in `docs/adr/`. Then inspect where friction lives:
- Where does understanding one concept require bouncing between many small modules?
- Where are modules **shallow**, with an interface nearly as complex as the implementation?
- Where have pure functions been extracted just for testability, but the real bugs hide in how they're called (no **locality**)?
- Where do tightly-coupled modules leak across their seams?
- Which parts of the codebase are untested, or hard to test through their current interface?

Apply the **deletion test**: would deleting it concentrate complexity, or just move it? *"Yes, concentrates"* is the signal you want.

### 2. Present candidates as an HTML report

Write a self-contained HTML file to the OS temp directory (`/tmp/architecture-review-<timestamp>.html` on Linux/macOS or `%TEMP%` on Windows). Open it or provide the path to the user.

The report uses **Tailwind via CDN** and **Mermaid via CDN** for diagrams. Each candidate gets a **before/after visualisation**.

For each candidate, render a card with:
- **Files**: which files/modules are involved.
- **Problem**: why the current architecture is causing friction.
- **Solution**: plain English description of what would change.
- **Benefits**: explained in terms of locality and leverage, and how tests would improve.
- **Before / After diagram**: side-by-side (Mermaid graph, cross-section, or boxes-and-arrows).
- **Recommendation strength**: `Strong`, `Worth exploring`, or `Speculative`.

End the report with a **Top recommendation** section. See [HTML-REPORT.md](./HTML-REPORT.md) for scaffold and styling details.

### 3. Grilling loop

Once the user picks a candidate, run a grilling loop:
- Walk the decision tree: constraints, dependencies, shape of the deepened module, what sits behind the seam, what tests survive.
- If naming a deepened module introduces a new domain concept, update `CONTEXT.md`.
- If the user rejects a candidate with a load-bearing reason, offer an ADR in `docs/adr/` so future reviews don't re-suggest it.
- If exploring alternative interfaces, use the Design It Twice sub-agent pattern.
