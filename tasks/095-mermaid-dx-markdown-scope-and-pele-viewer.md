---
type: Task Package
title: "Task 095: 1-Klik Mermaid DX, Feature-Scoped Diagrammer, Pele Viewer & HUD Links"
description: "Forbedrer diagram-DX med auto-synkroniseret Markdown-wrapper (architecture.md), feature-scoped diagrammer (docs/architecture/<feature>.md via --feature), hurtig browser-viewer baseret på obsidianmd/pele (xgauntlet topology open) og direkte 🗺️ [Arch]-links i Cockpit HUD, spec.md, specs/*.md samt README.md."
status: done
generated: { by: process:antigravity-task-init, at: "2026-11-15T14:57:00Z" }
tags: [topology, mermaid, pele, dx, markdown, hud, feature-scope, okf]
feature: topology
spec_ref: "specs/topology.md"
scope:
  - "tasks/095-mermaid-dx-markdown-scope-and-pele-viewer.md"
  - "crates/xgauntlet-core/src/features/topology/**"
  - "crates/xgauntlet-core/src/features/gauntlet/pipeline.rs"
  - "crates/xgauntlet-core/src/features/evidence/manifest.rs"
  - "crates/xgauntlet-core/src/features/checkpoint/engine.rs"
  - "crates/xgauntlet-core/src/features/tasks/**"
  - "crates/xgauntlet-core/src/features/adapters/antigravity/hud.rs"
  - "crates/xgauntlet-core/src/features/scaffold/templates.rs"
  - "crates/xgauntlet-core/src/lib.rs"
  - "crates/xgauntlet-cli/src/main.rs"
  - "crates/xgauntlet-cli/src/checkers.rs"
  - "crates/xgauntlet-cli/src/handlers.rs"
  - "crates/xgauntlet-cli/src/render.rs"
  - "architecture.mermaid"
  - "architecture.md"
  - "docs/architecture/**"
  - "specs/topology.md"
  - "specs/adapters.md"
  - "spec.md"
  - "README.md"
  - "packages/cli/README.md"
  - "README_OLD.md"
  - "ROADMAP.md"
  - "crates/xgauntlet-core/tests/single_agent_governance_test.rs"
  - ".agents/AGENTS.md"
  - "CONTEXT.md"
test_refs:
  - "crates/xgauntlet-core/tests/mermaid_dx_and_feature_scope_test.rs"
  - "crates/xgauntlet-core/tests/mermaid_auto_sync_test.rs"
  - "crates/xgauntlet-core/tests/cockpit_hud_truthfulness_test.rs"
  - "crates/xgauntlet-cli/tests/topology_dx_cli_test.rs"
---

# Task 095: 1-Klik Mermaid DX, Feature-Scoped Diagrammer, Pele Viewer & HUD Links

**Status**: `DONE`
**Intent**: `🚀 NEW FEATURE`
**Oprettet**: `2026-11-15`

## 🎯 Formål
Gøre arkitektur- og domænediagrammer **1-klik læsbare** direkte i GitHub, IDE Markdown-preview, Cockpit HUD og browser — uden manuel copy-paste til eksterne playgrounds og uden at drukne i ét 745-linjers monolitdiagram:

1. **GitHub & IDE Markdown-paritet (`architecture.md`)**: Bevare [architecture.mermaid](file:///home/rolfmadsen/Github/xgauntlet/architecture.mermaid) som kanonisk rå Mermaid-kilde, og automatisk generere/synkronisere søsterfilen `architecture.md` med standard ` ```mermaid `-kodeblok samt indeks-links til feature-diagrammer under `docs/architecture/`.
2. **Feature-scoped diagramopdeling (`--feature <navn>` & `docs/architecture/<feature>.md`)**: Udvide `topology`-motoren med deterministisk feature-filtrering (modeller i `features/<navn>/` + direkte 1-hop naborelationer), så hvert feature-diagram er kompakt (~10–25 klasser) og øjeblikkeligt læsbart i `specs/<feature>.md`, `spec.md` og `README.md`.
3. **Lynhurtig lokal Pele-viewer (`xgauntlet topology open`)**: Generere en Zero-Daemon, statisk HTML-viewer i `.xgauntlet/architecture-viewer.html` baseret på **Pele** (`obsidianmd/pele`, MIT) med pan/zoom, dark/light CSS-variabler og feature-filter, som åbnes direkte i standardbrowseren (med `--no-browser` og `--out` til deterministisk testning).
4. **Cockpit HUD & Dokumentations-links**: Berige Cockpit HUD's `Links`-linje med et `<1ms` opløst `🗺️ [Arch](...)`-link, der automatisk peger på opgavens scoped `docs/architecture/<feature>.md` (når `feature` er sat), ellers `architecture.md`, samt opdatere `spec.md`, `specs/*.md`, `README.md` og `.agents/AGENTS.md`.

## 📋 Acceptance Criteria

- [x] **AC1 (Feature-Scoped Mermaid Eksport)**:
  - `export_feature_mermaid_class_diagram` (og `xgauntlet topology export --format mermaid [--feature <navn>] [--level conceptual|detailed]`) understøtter deterministisk feature-filtrering uden at bryde eksisterende `MermaidExportOptions`-kontrakter.
  - Når `feature` er angivet, inkluderes deterministisk alle synlige domænemodeller fra det pågældende feature-slice samt direkte 1-hop tilknyttede nabotyper (`*--`, `..|>`), sorteret alfabetisk uden støj fra uafhængige features.
  - Hvis et ukendt feature-navn angives, returneres en klar, handlingsorienteret fejlbesked på CLI/API frem for et tomt diagram.

- [x] **AC2 (`architecture.md` & `docs/architecture/<feature>.md` Auto-Sync uden Tamper-Drift)**:
  - `xgauntlet verify --save` genererer/opdaterer både `architecture.mermaid` (uændret rå Mermaid), `architecture.md` (Markdown-wrapper med ` ```mermaid `-blok + oversigt over feature-diagrammer) og `docs/architecture/<feature>.md` for opdagede features under `architecture.feature_root`.
  - `crates/xgauntlet-core/src/features/evidence/manifest.rs` (`EXCLUDED_FILES` / genererede diagramstier) og `crates/xgauntlet-core/src/features/checkpoint/engine.rs` håndterer `architecture.md` og `docs/architecture/` uden at udløse selvmutations- eller dirty-manifest-falskpositiver.

- [x] **AC3 (Zero-Daemon Pele HTML-Viewer via `xgauntlet topology open`)**:
  - `TopologyCommands::Open` (`xgauntlet topology open [--feature <navn>] [--level conceptual|detailed] [--no-browser] [--out <sti>]`) genererer en self-contained statisk HTML-fil (default `.xgauntlet/architecture-viewer.html`) med pan/zoom, dark/light tema (`prefers-color-scheme`), kopier-Mermaid-knap og `obsidianmd/pele` `classDiagram`-rendering (inkl. ren fallback til rå kildetekst hvis browseren kører uden netværk og ingen lokal bundle er tilgængelig).
  - HTML-generatoren overholder WCAG AA kontrast (>= 4.5:1) for UI-kontroller og status-tekst via CSS design-tokens.
  - Med `--no-browser` skrives HTML-filen deterministisk uden at starte OS-browserproces (100% testbar i headless CI).

- [x] **AC4 (Cockpit HUD `🗺️ [Arch]` Link & `<5ms` Invariant)**:
  - `CockpitTelemetry` / `TaskTelemetry` opløser et letvægts diagram-link (`docs/architecture/<feature>.md` hvis opgavens `feature` er sat og filen findes, ellers `architecture.md`) via et enkelt `Path::exists()`-tjek (`< 1ms`, nul AST-scanning under `PreInvocation`).
  - `AntigravityAdapter::render_blockquote_hud` i `hud.rs` (samt `.agents/AGENTS.md` og `scaffold/templates.rs`) inkluderer `🗺️ [Arch](<link>)` på `Links`-linjen.

- [x] **AC5 (Dokumentation & Living Specs Sync)**:
  - `spec.md`, `README.md`, `specs/topology.md` og `specs/adapters.md` opdateres med klikbare links til `architecture.md` og relevante `docs/architecture/<feature>.md`-diagrammer.
  - Alle nye/ændrede domænetyper følger `CONTEXT.md` (`xgauntlet check-spec --context`).

- [x] **AC6 (1-Klik Mermaid Live Link i Markdown & Oprydning af `README_OLD.md` / `ROADMAP.md`)**:
  - `architecture.md` og `docs/architecture/<feature>.md` indeholder øverst et 1-klik `[🌐 Open Interactive Diagram (Mermaid Live)](https://mermaid.live/view#base64:...)`-link, så diagrammet kan åbnes med ét klik direkte fra Antigravity IDE Preview uden udvidelser.
  - Forældede `README_OLD.md` og `ROADMAP.md` fjernes sammen med `ROADMAP.md`-afhængigheden i `single_agent_governance_test.rs`.

## 🚫 Must NOT
- Fjern IKKE `architecture.mermaid` eller bryd bagudkompatibilitet med `[paths] diagram_file` i `gauntlet.toml` og `mermaid_auto_sync_test.rs`.
- Introducér IKKE baggrundsdæmoner, lokale HTTP-servere/porte eller runtime-afhængighed til Node.js fra Rust CLI'et (`Zero-Daemon` + standalone native binær-invariant i `spec.md`).
- Kør IKKE AST-scanning, Pele-rendering eller zlib-komprimering under `PreInvocation` (`hud.rs` / `inspect_task_telemetry`); bevar `<5ms` hurtig sti.
- Lad IKKE `verify --save` mutere menneske-kuraterede styringsfiler (`spec.md`, `specs/*.md`, `CONTEXT.md`); hold genererede diagrammer isoleret til `architecture.mermaid`, `architecture.md`, `docs/architecture/*.md` og `.xgauntlet/`.
- Overskrid IKKE filgrænserne i `gauntlet.toml` (`max_file_lines = 800`, `max_test_file_lines = 1000`); placér ny Markdown- og HTML-viewer-logik i fokuserede delmoduler (f.eks. `topology/markdown.rs`, `topology/viewer.rs`) og isolerede accepttests i `mermaid_dx_and_feature_scope_test.rs`.
- Foretag IKKE `git push`, `git reset --hard` eller `git clean -f` (ADR 0003).

## 📝 Revisions
- 2026-11-15: Oprettet draft-opgavepakke (`SPEC`) efter UX/DX-sparring om Markdown-rendering, feature-opdeling, Pele-viewer og HUD-links.
- 2026-11-15: Gate 1 godkendt; implementeret `RED → GREEN → REFACTOR → GAUNTLET`, tilføjet 1-klik `https://mermaid.live/view#base64:...` links til Antigravity IDE Preview og fjernet forældede `README_OLD.md` / `ROADMAP.md` (alle 6 obligatoriske lag bestået).

## 🧪 Verifikation
- `cargo test --package xgauntlet-core --test mermaid_dx_and_feature_scope_test --test mermaid_auto_sync_test --test cockpit_hud_truthfulness_test`
- `cargo run -p xgauntlet-cli -- topology export --format mermaid --feature topology`
- `cargo run -p xgauntlet-cli -- topology open --feature topology --no-browser`
- `cargo run -p xgauntlet-cli -- check-spec -t 095`
- `cargo run -p xgauntlet-cli -- check-spec --context`
- `cargo run -p xgauntlet-cli -- verify --task 095 --save`

