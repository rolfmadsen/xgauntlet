---
type: Task Package
title: "Task 016: Claude Code Telemetry HUD & Hook Adapter"
description: "Etablere universel Markdown Telemetry Card Formatter i crates/xgauntlet-core og xgauntlet-cli samt Claude Code adapterudvidelse med PostToolUse hook interception (.claude/settings.json) og automatiseret scaffolding jf. spec.md, ADR 0001, ADR 0004 og ADR 0006"
status: todo
generated: { by: process:xgauntlet-task-init, at: "2026-09-09T18:18:00Z" }
tags: [hud, telemetry, markdown, claude-code, hooks, gfm, cli, scaffold, adr-0004, adr-0006]
---

# Task 016: Claude Code Telemetry HUD & Hook Adapter

**Status**: `TODO`
**Intent**: `🚀 NEW FEATURE`
**Oprettet**: `2026-09-09`

## 🎯 Formål
Etablere den universelle Markdown Telemetry Card Formatter i `crates/xgauntlet-core` samt forbinde den direkte til Claude Code harnessen via `PostToolUse` hook-interception og konfigurationsscaffolding jf. [spec.md](spec.md), [CONTEXT.md](CONTEXT.md), [ADR 0001](docs/adr/0001-package-by-feature-architecture.md), [ADR 0004](docs/adr/0004-harness-adapter-slices.md) og [ADR 0006](docs/adr/0006-multi-harness-policy-adapter-contract.md):

1. **Universel Markdown Telemetry Card Formatter (`features/tasks/telemetry.rs`)**:
   - Da Claude Code opererer som et terminal-baseret CLI-værktøj uden Google Antigravity IDE's proprietære GUI-container og sidecars, skal cockpit-tilstanden formateres som GitHub Flavored Markdown (GFM).
   - Implementere `render_markdown_card(&self) -> String` på `TaskTelemetry` / `CockpitState`:
     * Header: `🛡️ XGAUNTLET COCKPIT` | `Phase: <TASK_ID>` | `Verdict: <PASS/FAIL/IN_PROGRESS>`
     * Række 1: Invarianter bestået/fejlet med visuelle indikatorer (`●●●●●` eller `14/14 ✔`)
     * Række 2: Mutation testing score & killed count (`[██████░░░░] 60%`)
     * Række 3: Policy Boundary status (`IN-BOUNDS (Local-only)` vs overtrædelser) samt Git Drift (`0.0%`)
     * Række 4: Evidence digest prefix (`SHA256: ...`) og aktuelt HEAD commit hash.
   - Implementere `render_markdown_compact(&self) -> String`: Én-linjes kompakt GFM badge-variant.
   - Høj performance (<3ms) og determinisme uden tunge eksterne rendering-afhængigheder (standard Rust string formatting med Unicode badges og markdown tabeller).

2. **CLI Subcommand Wiring (`crates/xgauntlet-cli`)**:
   - Tilføje eller udvide CLI-underkommandoen: `xgauntlet telemetry --format [json|ansi|markdown|compact-markdown]`.
   - `--format markdown` streamer det formaterede GFM telemetry-kort direkte til stdout.

3. **Claude Code Adapter & Hook Interception (`features/adapters/claude_code/` & `features/scaffold/`)**:
   - Udvide `claude_code/mod.rs` med understøttelse for hook-generering til `.claude/settings.json`.
   - Konfigurere et `PostToolUse` lifecycle hook (med tool matchers: `Edit|Write`), der kalder:
     `xgauntlet telemetry --format claude-hook`
   - Jf. officiel Claude Code dokumentation (v2.1.248+) forventer Claude Code struktureret JSON på stdout og rapporterer ellers en ikke-blokerende `<hook> hook error` ved raw tekst. Telemetry-motoren skal derfor under `--format claude-hook` returnere det kanoniske JSON format:
     ```json
     {
       "hookSpecificOutput": {
         "hookEventName": "PostToolUse",
         "additionalContext": "<GFM Markdown Cockpit Card>"
       }
     }
     ```
     Dette injicerer telemetrien direkte i Claudes kontekstvindue som en systempåmindelse ved siden af værktøjsresultatet.
   - Opdatere `xgauntlet scaffold init --harness claude_code` til at provisjonere `.claude/settings.json` med denne hook-opsætning sammen med eksisterende instruktioner i `CLAUDE.md`.

4. **Unit- og Integrationstestsuite**:
   - Unit tests i `crates/xgauntlet-core/tests/` der verificerer, at `render_markdown_card` formaterer tomme, fejlende og fuldt grønne telemetritilstande korrekt.
   - Markdown-syntaksvalidering (valide pipes, lukkede klammer, korrekte kolonneantal, ingen malformed tabeller).
   - Integrationstest i `crates/xgauntlet-core/tests/harness_adapters_test.rs` der validerer Claude Code `.claude/settings.json` scaffolding og det korrekte JSON payload contract (`hookSpecificOutput.additionalContext`).

## 📋 Acceptance Criteria
- [ ] `crates/xgauntlet-core/src/features/tasks/telemetry.rs` indeholder `render_markdown_card(&self) -> String` og `render_markdown_compact(&self) -> String`.
- [ ] Det genererede markdown-kort indeholder sektioner for Header (Task ID, Verdict), Invariants, Mutation score, Policy Boundary/Drift og Evidence digest/HEAD commit.
- [ ] `crates/xgauntlet-cli` understøtter `xgauntlet telemetry --format [json|ansi|markdown|compact-markdown|claude-hook]`.
- [ ] Under `--format claude-hook` udskrives gyldig JSON med `hookSpecificOutput: { hookEventName: "PostToolUse", additionalContext: "..." }` jf. Claude Code hooks specifikationen.
- [ ] `crates/xgauntlet-core/src/features/adapters/claude_code/mod.rs` understøtter generering af `.claude/settings.json` med `PostToolUse` hooks (matcher: `Edit|Write`).
- [ ] `PostToolUse` hooket i `.claude/settings.json` eksekverer `xgauntlet telemetry --format claude-hook`.
- [ ] `xgauntlet scaffold init --harness claude_code` opretter eller opdaterer `.claude/settings.json` med det specificerede telemetry hook uden at overskrive brugerdefinerede felter.
- [ ] Unit tests verificerer GFM-syntaks og rendering for tomme, delvise og fuldt grønne telemetritilstande samt Claude Code JSON-wrapping.
- [ ] Conformance integrationstest i `crates/xgauntlet-core/tests/harness_adapters_test.rs` verificerer Claude Code hook scaffolding.
- [ ] `cargo test --workspace` forbliver 100% grøn uden regressioner for eksisterende Antigravity adapter.

## 🚫 Must NOT
- Må IKKE introducere baggrundsdæmoner eller runtime-sockets (Zero-Daemon Invariant).
- Må IKKE tillade remote git publication (`git push`) jf. ADR 0003.
- Må IKKE introducere tunge eksterne formaterings-crates; skal anvende standard Rust string formatting.
- Må IKKE bryde fail-closed policy evaluering ved manglende eller malformed input jf. ADR 0006.
- Må IKKE overskrive eksisterende brugerdefinerede indstillinger i `.claude/settings.json` destruktivt.

## 📝 Revisions
- 2026-09-09: Oprettet som planlagt opgave (TODO) for Claude Code Telemetry HUD & Hook Adapter (Task 016).

## 🧪 Verifikation
- `cargo run -p xgauntlet-cli -- check-spec -t 016`
- `cargo test -p xgauntlet-core --test harness_adapters_test`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo fmt --check`
