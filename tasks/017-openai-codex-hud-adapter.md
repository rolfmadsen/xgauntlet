---
type: Task Package
title: "Task 017: OpenAI Codex Telemetry HUD & Skill Adapter"
description: "Etablere OpenAI Codex adapterudvidelse for Markdown Telemetry HUD integration i værktøjsresponser (checkpoint/verify) og agent-gauntlet skill-skabeloner (diagnose, code-review, grill-me) samt scaffolding jf. spec.md, ADR 0001, ADR 0004 og ADR 0006"
status: todo
generated: { by: process:xgauntlet-task-init, at: "2026-09-09T18:18:10Z" }
tags: [hud, telemetry, markdown, codex, skills, agent-gauntlet, gfm, cli, scaffold, adr-0004, adr-0006]
---

# Task 017: OpenAI Codex Telemetry HUD & Skill Adapter

**Status**: `TODO`
**Intent**: `🚀 NEW FEATURE`
**Oprettet**: `2026-09-09`

## 🎯 Formål
Etablere OpenAI Codex adapterudvidelse i `crates/xgauntlet-core` for struktureret Markdown Telemetry HUD integration i værktøjsresponser og bundled skill-skabeloner jf. [spec.md](spec.md), [CONTEXT.md](CONTEXT.md), [ADR 0001](docs/adr/0001-package-by-feature-architecture.md), [ADR 0004](docs/adr/0004-harness-adapter-slices.md) og [ADR 0006](docs/adr/0006-multi-harness-policy-adapter-contract.md):

1. **Codex Adapter Response Wrapping (`features/adapters/codex/mod.rs`)**:
   - Da OpenAI Codex opererer via chat/tool-calling uden Antigravity IDE's native GUI-sidecars, skal telemetrien formateres som GFM markdown direkte i tool execution responserne.
   - Opdatere `codex/mod.rs` således at når `xgauntlet checkpoint` eller `xgauntlet verify` kaldes i en Codex-kontekst, indkapsles responspayloadet med det standardiserede GFM Markdown Telemetry Card (fra `render_markdown_card`).
   - Sikre at JSON-payloads og turn summaries i Codex formateres så markdown tabeller og visual badges parser rent i VS Code extension webviews og terminal paneler.

2. **Agent-Gauntlet Skill Templates Integration (`.agents/plugins/agent-gauntlet/`)**:
   - Opdatere bundled skill templates i `.agents/plugins/agent-gauntlet/skills/`:
     * `diagnose/SKILL.md`: Afslutter diagnoseforløb med det opdaterede cockpit-kort.
     * `code-review/SKILL.md`: Inkluderer cockpit telemetri over for standarder og spec-afvigelser.
     * `grill-me/SKILL.md` og `grill-with-docs/SKILL.md`: Viser det aktuelle criteria- og cockpit-billede ved sessionens afslutning.
   - Sikre at agent-instruktionerne i disse skills specifikt refererer til og anvender markdown cockpit-kortet.

3. **Scaffolding for OpenAI Codex (`features/scaffold/`)**:
   - Opdatere `xgauntlet scaffold init --harness codex` til at konfigurere `.agents/AGENTS.md` og plugin-strukturer med Codex-specifik markdown HUD embedding.
   - Sikre at `ai-plugin.json` og tilhørende manifest-filer forbliver 100% gyldige i henhold til `validate-plugin`.

4. **Conformance & Regression Tests**:
   - Udvide `crates/xgauntlet-core/tests/harness_adapters_test.rs` med testcases der validerer:
     * Codex scaffold generering opretter korrekte skill templates med markdown card integration.
     * Response wrapping formaterer gyldig GFM syntaks (rigtige pipes, lukkede klammer, korrekte kolonneantal).
   - Køre parameteriseret conformance testsuite for alle adaptere for at bevise nul regression for Google Antigravity og Claude Code.

## 📋 Acceptance Criteria
- [ ] `crates/xgauntlet-core/src/features/adapters/codex/mod.rs` understøtter indkapsling af verifikations- og checkpoint-responser med GFM Markdown Telemetry Card.
- [ ] Tool execution payloads i Codex harnessen formateres med ren GFM-syntaks egnet til webview-paneler og samtalespor.
- [ ] Bundled skill-skabeloner i `.agents/plugins/agent-gauntlet/` (`diagnose`, `code-review`, `grill-me`) er opdateret til at udstille markdown cockpit-kortet.
- [ ] `xgauntlet scaffold init --harness codex` provisjonerer projektopsætning med de opdaterede Codex skill-skabeloner og markdown HUD retningslinjer.
- [ ] `xgauntlet validate-plugin` godkender den genererede Codex plugin-konfiguration med 0 advarsler og 0 fejl.
- [ ] Conformance tests i `crates/xgauntlet-core/tests/harness_adapters_test.rs` validerer Codex response wrapping og skill output formattering.
- [ ] `cargo test --workspace` forbliver 100% grøn uden regressioner for eksisterende Antigravity eller Claude Code adaptere.

## 🚫 Must NOT
- Må IKKE introducere baggrundsdæmoner eller runtime-sockets (Zero-Daemon Invariant).
- Må IKKE tillade remote git publication (`git push`) jf. ADR 0003.
- Må IKKE tillade untrusted tool payloads at overskrive `EnforcementContext` parametre jf. ADR 0006.
- Må IKKE bryde bagudkompatibilitet med Google Antigravity IDE eller eksisterende `.agents/` workflows.
- Må IKKE overtræde fail-closed policy gates ved malformed eller uventede tool payloads.

## 📝 Revisions
- 2026-09-09: Oprettet som planlagt opgave (TODO) for OpenAI Codex Telemetry HUD & Skill Adapter (Task 017).

## 🧪 Verifikation
- `cargo run -p xgauntlet-cli -- check-spec -t 017`
- `cargo test -p xgauntlet-core --test harness_adapters_test`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo fmt --check`
