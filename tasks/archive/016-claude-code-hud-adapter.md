---
type: Task Package
title: "Task 016: Claude Code Telemetry HUD & Hook Adapter"
description: "Etablere Box-Drawing Telemetry Formatter (Variant B med Ref-stier) i crates/xgauntlet-core og xgauntlet-cli samt Claude Code adapterudvidelse med PostToolUse hook interception (.claude/settings.json) og scaffolding jf. spec.md, ADR 0001, ADR 0004 og ADR 0006"
status: done
generated: { by: process:xgauntlet-task-init, at: "2026-09-09T18:18:00Z" }
tags: [hud, telemetry, box-drawing, claude-code, hooks, terminal, cli, scaffold, adr-0004, adr-0006]
---

# Task 016: Claude Code Telemetry HUD & Hook Adapter

**Status**: `DONE`
**Intent**: `🚀 NEW FEATURE`
**Oprettet**: `2026-09-09`

## 🎯 Formål
Etablere en high-density Box-Drawing Telemetry Formatter (Variant B) i `crates/xgauntlet-core` samt forbinde den direkte til Claude Code harnessen via `PostToolUse` hook-interception og konfigurationsscaffolding jf. [spec.md](spec.md), [CONTEXT.md](CONTEXT.md), [ADR 0001](docs/adr/0001-package-by-feature-architecture.md), [ADR 0004](docs/adr/0004-harness-adapter-slices.md) og [ADR 0006](docs/adr/0006-multi-harness-policy-adapter-contract.md):

1. **Terminal Box-Drawing Telemetry Formatter (`features/tasks/telemetry.rs`)**:
   - Claude Code opererer som et terminal-baseret CLI-værktøj i monospace-miljøer. For at undgå syntaksstøj og ustabile kolonnebredder fra traditionelle Markdown-hyperlinks (`[Tekst](sti)`), standardiseres formatet som en 5-linjers Unicode Box-Drawing ramme (Variant B):
     ```text
     ┌─── xgauntlet: Task 016 ──────────────────────────────────────┐
     │ Status: RED (Tests failing)   Scope: crates/xgauntlet-core   │
     │ Progress: [██████░░░░] 60%    Invariants: 14/14 PASS         │
     │ Git: main@093b533 (dirty)     Evidence: pending              │
     │ Ref: tasks/016.md • spec.md • docs/adr/README.md             │
     └──────────────────────────────────────────────────────────────┘
     ```
   - Implementere `render_box_card(&self) -> String` på `TaskTelemetry` / `CockpitState`:
     * Header-ramme med fast bredde (f.eks. 64 tegn): `┌─── xgauntlet: <TASK_ID> ──...──┐`
     * Række 1: Status/Fase og berørt Scope
     * Række 2: Progress (visual Unicode block bar) og Invariants status
     * Række 3: Git branch@oid, dirty/clean indikator og Evidens-digest
     * Række 4 (`Ref:`): Rene relative filstier (`tasks/<id>.md • spec.md • docs/adr/README.md`), som moderne terminaler (VS Code, iTerm, Kitty, Alacritty, Warp) automatisk gør klikbare via `Cmd+Click` / `Ctrl+Click`.
     * Bundramme: `└──────────────────...──────────────────────────┘`
   - Implementere `render_box_compact(&self) -> String`: Én-linjes ultra-kompakt badge-variant.
   - Høj performance (<3ms) og determinisme uden tunge eksterne rendering-afhængigheder (standard Rust string formatting med fast kolonnejustering).

2. **CLI Subcommand Wiring (`crates/xgauntlet-cli`)**:
   - Udvide CLI-underkommandoen: `xgauntlet telemetry --format [box|claude-hook|json|ansi|compact-box]`.
   - `--format box` streamer det formaterede Box-Drawing kort direkte til stdout.

3. **Claude Code Adapter & Hook Interception (`features/adapters/claude_code/` & `features/scaffold/`)**:
   - Udvide `claude_code/mod.rs` med understøttelse for hook-generering til `.claude/settings.json`.
   - Konfigurere et `PostToolUse` lifecycle hook (med tool matchers: `Edit|Write`), der kalder:
     `xgauntlet telemetry --format claude-hook`
   - Jf. officiel Claude Code dokumentation (v2.1.248+) forventer Claude Code struktureret JSON på stdout og rapporterer ellers en ikke-blokerende `<hook> hook error` ved raw tekst. Telemetry-motoren skal derfor under `--format claude-hook` returnere det kanoniske JSON format:
     ```json
     {
       "hookSpecificOutput": {
         "hookEventName": "PostToolUse",
         "additionalContext": "<Unicode Box-Drawing Telemetry Card>"
       }
     }
     ```
     Dette injicerer telemetriboksen direkte i Claudes kontekstvindue som en systempåmindelse ved siden af værktøjsresultatet.
   - Opdatere `xgauntlet scaffold init --harness claude_code` til at provisjonere `.claude/settings.json` med denne hook-opsætning sammen med eksisterende instruktioner i `CLAUDE.md`.

4. **Unit- og Integrationstestsuite**:
   - Unit tests i `crates/xgauntlet-core/tests/` der verificerer, at `render_box_card` formaterer tomme, fejlende og fuldt grønne telemetritilstande med millimeterpræcise rammer og korrekt justering.
   - Validering af `Ref:` linjens filstier og ingen rå markdown klammer inde i boksen.
   - Integrationstest i `crates/xgauntlet-core/tests/harness_adapters_test.rs` der validerer Claude Code `.claude/settings.json` scaffolding og det korrekte JSON payload contract (`hookSpecificOutput.additionalContext`).

## 📋 Acceptance Criteria
- [x] `crates/xgauntlet-core/src/features/tasks/telemetry.rs` indeholder `render_box_card(&self) -> String` og `render_box_compact(&self) -> String`.
- [x] Det genererede boks-kort følger Variant B med fast bredde, præcise hjørner (`┌`, `┐`, `└`, `┘`), status, progress, git drift og en `Ref:` række med rene stier.
- [x] `crates/xgauntlet-cli` understøtter `xgauntlet telemetry --format [box|claude-hook|json|ansi|compact-box]`.
- [x] Under `--format claude-hook` udskrives gyldig JSON med `hookSpecificOutput: { hookEventName: "PostToolUse", additionalContext: "..." }` indeholdende boks-kortet.
- [x] `crates/xgauntlet-core/src/features/adapters/claude_code/mod.rs` understøtter generering af `.claude/settings.json` med `PostToolUse` hooks (matcher: `Edit|Write`).
- [x] `PostToolUse` hooket i `.claude/settings.json` eksekverer `xgauntlet telemetry --format claude-hook`.
- [x] `xgauntlet scaffold init --harness claude_code` opretter eller opdaterer `.claude/settings.json` med det specificerede telemetry hook uden at overskrive brugerdefinerede felter.
- [x] Unit tests verificerer boks-rammer, kolonneflugtning og Claude Code JSON-wrapping for tomme, delvise og fuldt grønne tilstande.
- [x] Conformance integrationstest i `crates/xgauntlet-core/tests/harness_adapters_test.rs` verificerer Claude Code hook scaffolding.
- [x] `cargo test --workspace` forbliver 100% grøn uden regressioner for eksisterende Antigravity adapter.

## 🚫 Must NOT
- Må IKKE introducere baggrundsdæmoner eller runtime-sockets (Zero-Daemon Invariant).
- Må IKKE tillade remote git publication (`git push`) jf. ADR 0003.
- Må IKKE introducere tunge eksterne formaterings-crates; skal anvende standard Rust string formatting med deterministisk kolonneberegning.
- Må IKKE bryde fail-closed policy evaluering ved manglende eller malformed input jf. ADR 0006.
- Må IKKE overskrive eksisterende brugerdefinerede indstillinger i `.claude/settings.json` destruktivt.

## 📝 Revisions
- 2026-09-09: Opdateret til eksplicit at forankre Variant B Unicode Box-Drawing layout med rene `Ref:` stier for optimal terminal-ergonomi og native klikbarhed.
- 2026-09-09: Oprettet som planlagt opgave (TODO) for Claude Code Telemetry HUD & Hook Adapter (Task 016).

## 🧪 Verifikation
- `cargo run -p xgauntlet-cli -- check-spec -t 016`
- `cargo test -p xgauntlet-core --test harness_adapters_test`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo fmt --check`
