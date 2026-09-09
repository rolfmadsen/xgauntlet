---
type: Task Package
title: "Task 017: OpenAI Codex Telemetry HUD & Hook Adapter"
description: "Etablere OpenAI Codex adapterudvidelse for Box-Drawing Telemetry HUD integration (Variant B med Ref-stier) via PostToolUse lifecycle hook interception (.codex/hooks.json), respons-wrapping (checkpoint/verify) og agent-gauntlet skill-skabeloner samt scaffolding jf. spec.md, ADR 0001, ADR 0004 og ADR 0006"
status: active
generated: { by: process:xgauntlet-task-init, at: "2026-09-09T18:18:10Z" }
tags: [hud, telemetry, box-drawing, codex, hooks, skills, agent-gauntlet, terminal, cli, scaffold, adr-0004, adr-0006]
---

# Task 017: OpenAI Codex Telemetry HUD & Hook Adapter

**Status**: `ACTIVE`
**Intent**: `🚀 NEW FEATURE`
**Oprettet**: `2026-09-09`

## 🎯 Formål
Etablere OpenAI Codex adapterudvidelse i `crates/xgauntlet-core` for struktureret Box-Drawing Telemetry HUD integration (Variant B) via native lifecycle hooks (`.codex/hooks.json`), værktøjsresponser og bundled skill-skabeloner jf. [spec.md](spec.md), [CONTEXT.md](CONTEXT.md), [ADR 0001](docs/adr/0001-package-by-feature-architecture.md), [ADR 0004](docs/adr/0004-harness-adapter-slices.md) og [ADR 0006](docs/adr/0006-multi-harness-policy-adapter-contract.md):

1. **Codex Lifecycle Hook Interception (`.codex/hooks.json` & `.codex/config.toml`)**:
   - Jf. OpenAI Codex dokumentationen (`https://learn.chatgpt.com/docs/hooks`) understøtter Codex et native hook framework med `PostToolUse` events placeret i `.codex/hooks.json` eller `.codex/config.toml`.
   - Værktøjs-matching understøtter `apply_patch` (for filmodifikationer via matcher `apply_patch|Edit|Write`) samt `Bash`.
   - Dokumentationen fastslår eksplicit: *"Plain text on stdout is ignored."* Telemetri-output **skal** derfor struktureres som JSON på stdout:
     ```json
     {
       "hookSpecificOutput": {
         "hookEventName": "PostToolUse",
         "additionalContext": "<Unicode Box-Drawing Telemetry Card>"
       }
     }
     ```
     Dette tilføjes som udvikler- og modelkontekst (`additionalContext`) umiddelbart efter hver gennemført værktøjshandling.
   - Forbinde dette til `xgauntlet telemetry --format codex-hook` (som deler den deterministiske boks-generator `render_box_card` fra Task 016).

2. **Codex Terminal Box-Drawing Format (Variant B)**:
   - Codex anvender terminalpaneler og CLI-interfaces, hvor fast monospace-bredde og `Cmd+Click` / `Ctrl+Click` på rå filstier giver den bedste ergonomi uden syntaksforvrængning fra rå markdown:
     ```text
     ┌─── xgauntlet: Task 017 ──────────────────────────────────────┐
     │ Status: RED (Tests failing)   Scope: crates/xgauntlet-core   │
     │ Progress: [██████░░░░] 60%    Invariants: 14/14 PASS         │
     │ Git: main@093b533 (dirty)     Evidence: pending              │
     │ Ref: tasks/017.md • spec.md • docs/adr/README.md             │
     └──────────────────────────────────────────────────────────────┘
     ```
   - Når operationer som `xgauntlet checkpoint` eller `xgauntlet verify` kaldes i en Codex-kontekst, indkapsles responset med dette deterministiske Box-Drawing Telemetry Card.

3. **Agent-Gauntlet Skill Templates Integration (`.agents/plugins/agent-gauntlet/`)**:
   - Opdatere bundled skill templates i `.agents/plugins/agent-gauntlet/skills/`:
     * `diagnose/SKILL.md`: Afslutter diagnoseforløb med det opdaterede cockpit boks-kort.
     * `code-review/SKILL.md`: Inkluderer cockpit telemetri over for standarder og spec-afvigelser.
     * `grill-me/SKILL.md` og `grill-with-docs/SKILL.md`: Viser det aktuelle criteria- og cockpit-billede ved sessionens afslutning.

4. **Scaffolding for OpenAI Codex (`features/scaffold/`)**:
   - Opdatere `xgauntlet scaffold init --harness codex` til automatisk at generere:
     * `.codex/hooks.json` med `PostToolUse` hook for `apply_patch|Write|Edit|Bash`, der kalder `xgauntlet telemetry --format codex-hook`.
     * `.agents/AGENTS.md` og opdaterede skill-skabeloner med Codex-specifik boks-layout vejledning.
   - Sikre at `ai-plugin.json` og tilhørende manifest-filer forbliver 100% gyldige i henhold til `xgauntlet validate-plugin`.

5. **Conformance & Regression Tests**:
   - Udvide `crates/xgauntlet-core/tests/harness_adapters_test.rs` med testcases der validerer:
     * Codex hook scaffolding genererer gyldig `.codex/hooks.json` med korrekte matchere (`apply_patch|Edit|Write`).
     * Hook JSON output overholder OpenAI Codex kontrakt (`hookSpecificOutput.additionalContext`).
     * Response wrapping formaterer gyldig Box-Drawing syntaks (Variant B) med `Ref:` række.
   - Køre parameteriseret conformance testsuite for alle adaptere for at bevise nul regression for Google Antigravity og Claude Code.

## 📋 Acceptance Criteria
- [ ] `crates/xgauntlet-cli` understøtter `xgauntlet telemetry --format [codex-hook|box|json|ansi]` og genererer gyldig JSON med `hookSpecificOutput: { hookEventName: "PostToolUse", additionalContext: "..." }`.
- [ ] Det genererede telemetri-kort formateres som Variant B Unicode Box-Drawing med fast bredde og `Ref:` række med rene klikbare stier.
- [ ] `crates/xgauntlet-core/src/features/adapters/codex/mod.rs` understøtter scaffolding af `.codex/hooks.json` med `PostToolUse` hooks (matcher: `apply_patch|Edit|Write|Bash`).
- [ ] `xgauntlet scaffold init --harness codex` provisjonerer `.codex/hooks.json` med telemetry hooket og opdaterer `.agents/AGENTS.md` uden destruktiv overskrivning.
- [ ] `crates/xgauntlet-core/src/features/adapters/codex/mod.rs` understøtter indkapsling af verifikations- og checkpoint-responser med Variant B Box-Drawing kortet.
- [ ] Bundled skill-skabeloner i `.agents/plugins/agent-gauntlet/` (`diagnose`, `code-review`, `grill-me`) er opdateret til at udstille boks cockpit-kortet.
- [ ] `xgauntlet validate-plugin` godkender den genererede Codex plugin- og hook-konfiguration med 0 advarsler og 0 fejl.
- [ ] Conformance tests i `crates/xgauntlet-core/tests/harness_adapters_test.rs` validerer Codex hook scaffolding, response wrapping og boks-syntaks.
- [ ] `cargo test --workspace` forbliver 100% grøn uden regressioner for eksisterende Antigravity eller Claude Code adaptere.

## 🚫 Must NOT
- Må IKKE introducere baggrundsdæmoner eller runtime-sockets (Zero-Daemon Invariant).
- Må IKKE tillade remote git publication (`git push`) jf. ADR 0003.
- Må IKKE tillade untrusted tool payloads at overskrive `EnforcementContext` parametre jf. ADR 0006.
- Må IKKE bryde bagudkompatibilitet med Google Antigravity IDE eller eksisterende `.agents/` workflows.
- Må IKKE overtræde fail-closed policy gates ved malformed eller uventede tool payloads.

## 📝 Revisions
- 2026-09-09: Opdateret til eksplicit at forankre Variant B Unicode Box-Drawing layout med rene `Ref:` stier for optimal terminal-ergonomi og native klikbarhed.
- 2026-09-09: Oprettet som planlagt opgave (TODO) for OpenAI Codex Telemetry HUD & Hook Adapter (Task 017) opdateret mod officiel Codex hooks specifikation.

## 🧪 Verifikation
- `cargo run -p xgauntlet-cli -- check-spec -t 017`
- `cargo test -p xgauntlet-core --test harness_adapters_test`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo fmt --check`
