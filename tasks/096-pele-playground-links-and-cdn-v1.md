---
type: Task Package
title: "Task 096: Pele Playground 1-Klik Links & Official Pele v1 CDN Import"
description: "Udskifter mermaid.live-links i architecture.md og docs/architecture/*.md med hurtige Pele Playground URL'er (https://pele.run/playground#source=...) og opgraderer den lokale Pele HTML-viewer i viewer.rs til den officielle @1/dist/pele.js CDN-indgang."
status: active
generated: { by: process:antigravity-task-init, at: "2026-11-15T15:30:00Z" }
tags: [topology, pele, mermaid, markdown, dx, okf]
feature: topology
spec_ref: "specs/topology.md"
scope:
  - "tasks/096-pele-playground-links-and-cdn-v1.md"
  - "crates/xgauntlet-core/src/features/topology/markdown.rs"
  - "crates/xgauntlet-core/src/features/topology/viewer.rs"
  - "crates/xgauntlet-core/src/features/topology/mod.rs"
  - "architecture.md"
  - "docs/architecture/**"
  - "specs/topology.md"
test_refs:
  - "crates/xgauntlet-core/tests/mermaid_dx_and_feature_scope_test.rs"
---

# Task 096: Pele Playground 1-Klik Links & Official Pele v1 CDN Import

**Status**: `ACTIVE`
**Intent**: `🔄 ENHANCEMENT`
**Oprettet**: `2026-11-15`

## 🎯 Formål
Udnytte **Pele** (`obsidianmd/pele`, væsentligt hurtigere end Mermaids standard-viewer) hele vejen igennem i 1-klik diagram-links og lokal HTML-rendering:

1. **1-Klik Pele Playground links i Markdown (`markdown.rs`)**: Erstatte `https://mermaid.live/view#base64:...` med deterministiske `https://pele.run/playground#source=...` links i både [architecture.md](../architecture.md) og alle feature-diagrammer under [docs/architecture/](../docs/architecture/).
2. **Officiel Pele v1 CDN-indgang (`viewer.rs`)**: Opgradere CDN-importen i `render_pele_viewer_html` fra `https://cdn.jsdelivr.net/npm/pele/+esm` til den officielt dokumenterede ESM-indgang `https://cdn.jsdelivr.net/npm/pele@1/dist/pele.js`.

## 📋 Acceptance Criteria

- [ ] **AC1 (Deterministisk `pele.run/playground#source=...` Encoder i Zero-Dependency Rust)**:
  - `crates/xgauntlet-core/src/features/topology/markdown.rs` udstiller `pele_playground_url(mermaid_source: &str) -> String` (re-eksporteret via `topology/mod.rs`), som koder Mermaid-kildetekst med standard WHATWG `application/x-www-form-urlencoded` (`' '` → `'+'`, `A-Z`/`a-z`/`0-9`/`*`/`-`/`.`/`_` uændrede, øvrige UTF-8 bytes som `%XX` i store hex-tegn) efter præfikset `https://pele.run/playground#source=`.
  - Enhedstest/accepttest verificerer 100% byte-nøjagtigt match mod det empiriske `Copy share link`-eksempel fra [pele.run/playground](https://pele.run/playground).

- [ ] **AC2 (Pele Playground Links i `architecture.md` & `docs/architecture/<feature>.md`)**:
  - `render_architecture_markdown` og `render_feature_architecture_markdown` indsætter øverst et `[🌐 Open Interactive Diagram (Pele Playground)](https://pele.run/playground#source=...)`-link og fjerner `mermaid.live`-afhængigheden (inkl. ubrugt `base64_encode`).
  - `xgauntlet verify --save` synkroniserer [architecture.md](../architecture.md) og samtlige `docs/architecture/<feature>.md` med Pele Playground-links uden selvmutations- eller manifest-fejl.

- [ ] **AC3 (Officiel Pele `@1/dist/pele.js` CDN-import i `viewer.rs`)**:
  - `render_pele_viewer_html` i `crates/xgauntlet-core/src/features/topology/viewer.rs` importerer ESM-modulet fra `https://cdn.jsdelivr.net/npm/pele@1/dist/pele.js` og bevarer `pele.render(source)`, pan/zoom, WCAG AA tokens og offline fallback.

## 🚫 Must NOT
- Tilføj IKKE eksterne Cargo-afhængigheder (f.eks. `flate2`, `miniz_oxide` eller `url`) til `Cargo.toml`; brug ren zero-dependency Rust percent-encoding.
- Fjern IKKE `architecture.mermaid`, ` ```mermaid `-kodeblokke eller `xgauntlet topology open`-kommandoen.
- Overskrid IKKE filgrænserne i `gauntlet.toml` (`max_file_lines = 800`, `max_test_file_lines = 1000`).
- Foretag IKKE `git push`, `git reset --hard` eller `git clean -f` (ADR 0003).

## 📝 Revisions
- 2026-11-15: Oprettet OKF v0.2 opgavepakke efter verifikation af det empiriske `pele.run/playground#source=...` format og `pele.run/api`-CDN-stien.

## 🧪 Verifikation
- `cargo test --package xgauntlet-core --test mermaid_dx_and_feature_scope_test`
- `cargo run -p xgauntlet-cli -- check-spec -t 096`
- `cargo run -p xgauntlet-cli -- verify --task 096 --save`

