---
type: Task Package
title: "Task 017: OpenAI Codex Telemetry HUD & Hook Adapter"
description: "Etablere OpenAI Codex adapterudvidelse for Markdown Telemetry HUD integration via PostToolUse lifecycle hook interception (.codex/hooks.json), respons-wrapping (checkpoint/verify) og agent-gauntlet skill-skabeloner samt scaffolding jf. spec.md, ADR 0001, ADR 0004 og ADR 0006"
status: todo
generated: { by: process:xgauntlet-task-init, at: "2026-09-09T18:18:10Z" }
tags: [hud, telemetry, markdown, codex, hooks, skills, agent-gauntlet, gfm, cli, scaffold, adr-0004, adr-0006]
---

# Task 017: OpenAI Codex Telemetry HUD & Hook Adapter

**Status**: `TODO`
**Intent**: `🚀 NEW FEATURE`
**Oprettet**: `2026-09-09`

## 🎯 Formål
Etablere OpenAI Codex adapterudvidelse i `crates/xgauntlet-core` for struktureret Markdown Telemetry HUD integration via native lifecycle hooks (`.codex/hooks.json`), værktøjsresponser og bundled skill-skabeloner jf. [spec.md](spec.md), [CONTEXT.md](CONTEXT.md), [ADR 0001](docs/adr/0001-package-by-feature-architecture.md), [ADR 0004](docs/adr/0004-harness-adapter-slices.md) og [ADR 0006](docs/adr/0006-multi-harness-policy-adapter-contract.md):

1. **Codex Lifecycle Hook Interception (`.codex/hooks.json` & `.codex/config.toml`)**:
   - Jf. OpenAI Codex dokumentationen (`https://learn.chatgpt.com/docs/hooks`) understøtter Codex et native hook framework med `PostToolUse` events placeret i `.codex/hooks.json` eller `.codex/config.toml`.
   - Værktøjs-matching understøtter `apply_patch` (for filmodifikationer via matcher `apply_patch|Edit|Write`) samt `Bash`.
   - Dokumentationen fastslår eksplicit: *"Plain text on stdout is ignored."* Telemetri-output **skal** derfor struktureres som JSON på stdout:
     ```json
     {
       "hookSpecificOutput": {
         "hookEventName": "PostToolUse",
         "additionalContext": "<GFM Markdown Cockpit Card>"
       }
     }
     ```
     Dette tilføjes som udvikler- og modelkontekst (`additionalContext`) umiddelbart efter hver gennemført værktøjshandling.
   - Forbinde dette til `xgauntlet telemetry --format codex-hook` (eller dele implementeringen med `claude-hook`, da JSON-kontrakten er konvergent).

2. **Codex Adapter Response Wrapping (`features/adapters/codex/mod.rs`)**:
   - Når operationer som `xgauntlet checkpoint` eller `xgauntlet verify` kaldes i en Codex-kontekst, indkapsles JSON-responset med det standardiserede GFM Markdown Telemetry Card.
   - Sikre at JSON-payloads og turn summaries i Codex formateres så markdown tabeller og visual badges parser rent i VS Code extension webviews og terminal paneler.

3. **Agent-Gauntlet Skill Templates Integration (`.agents/plugins/agent-gauntlet/`)**:
   - Opdatere bundled skill templates i `.agents/plugins/agent-gauntlet/skills/`:
     * `diagnose/SKILL.md`: Afslutter diagnoseforløb med det opdaterede cockpit-kort.
     * `code-review/SKILL.md`: Inkluderer cockpit telemetri over for standarder og spec-afvigelser.
     * `grill-me/SKILL.md` og `grill-with-docs/SKILL.md`: Viser det aktuelle criteria- og cockpit-billede ved sessionens afslutning.

4. **Scaffolding for OpenAI Codex (`features/scaffold/`)**:
   - Opdatere `xgauntlet scaffold init --harness codex` til automatisk at generere:
     * `.codex/hooks.json` med `PostToolUse` hook for `apply_patch|Write|Edit|Bash`, der kalder `xgauntlet telemetry --format codex-hook`.
     * `.agents/AGENTS.md` og opdaterede skill-skabeloner med Codex-specifik markdown HUD embedding.
   - Sikre at `ai-plugin.json` og tilhørende manifest-filer forbliver 100% gyldige i henhold til `xgauntlet validate-plugin`.

5. **Conformance & Regression Tests**:
   - Udvide `crates/xgauntlet-core/tests/harness_adapters_test.rs` med testcases der validerer:
     * Codex hook scaffolding genererer gyldig `.codex/hooks.json` med korrekte matchere (`apply_patch|Edit|Write`).
     * Hook JSON output overholder OpenAI Codex kontrakt (`hookSpecificOutput.additionalContext`).
     * Response wrapping formaterer gyldig GFM syntaks.
   - Køre parameteriseret conformance testsuite for alle adaptere for at bevise nul regression for Google Antigravity og Claude Code.

## 📋 Acceptance Criteria
- [ ] `crates/xgauntlet-cli` understøtter `xgauntlet telemetry --format [codex-hook|json|ansi|markdown]` og genererer gyldig JSON med `hookSpecificOutput: { hookEventName: "PostToolUse", additionalContext: "..." }`.
- [ ] `crates/xgauntlet-core/src/features/adapters/codex/mod.rs` understøtter scaffolding af `.codex/hooks.json` med `PostToolUse` hooks (matcher: `apply_patch|Edit|Write|Bash`).
- [ ] `xgauntlet scaffold init --harness codex` provisjonerer `.codex/hooks.json` med telemetry hooket og opdaterer `.agents/AGENTS.md` uden destruktiv overskrivning.
- [ ] `crates/xgauntlet-core/src/features/adapters/codex/mod.rs` understøtter indkapsling af verifikations- og checkpoint-responser med GFM Markdown Telemetry Card.
- [ ] Bundled skill-skabeloner i `.agents/plugins/agent-gauntlet/` (`diagnose`, `code-review`, `grill-me`) er opdateret til at udstille markdown cockpit-kortet.
- [ ] `xgauntlet validate-plugin` godkender den genererede Codex plugin- og hook-konfiguration med 0 advarsler og 0 fejl.
- [ ] Conformance tests i `crates/xgauntlet-core/tests/harness_adapters_test.rs` validerer Codex hook scaffolding, response wrapping og GFM tabelsyntaks.
- [ ] `cargo test --workspace` forbliver 100% grøn uden regressioner for eksisterende Antigravity eller Claude Code adaptere.

## 🚫 Must NOT
- Må IKKE introducere baggrundsdæmoner eller runtime-sockets (Zero-Daemon Invariant).
- Må IKKE tillade remote git publication (`git push`) jf. ADR 0003.
- Må IKKE tillade untrusted tool payloads at overskrive `EnforcementContext` parametre jf. ADR 0006.
- Må IKKE bryde bagudkompatibilitet med Google Antigravity IDE eller eksisterende `.agents/` workflows.
- Må IKKE overtræde fail-closed policy gates ved malformed eller uventede tool payloads.

## 📝 Revisions
- 2026-09-09: Oprettet som planlagt opgave (TODO) for OpenAI Codex Telemetry HUD & Hook Adapter (Task 017) opdateret mod officiel Codex hooks specifikation.

## 🧪 Verifikation
- `cargo run -p xgauntlet-cli -- check-spec -t 017`
- `cargo test -p xgauntlet-core --test harness_adapters_test`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo fmt --check`
