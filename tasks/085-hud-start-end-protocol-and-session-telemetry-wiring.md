---
type: Task Package
title: "Task 085: HUD Start/End Protocol & Active Session Telemetry Wiring"
description: "Begræns visuelt Cockpit Task HUD til start, eskalering og slut (ingen HUD på mellemliggende baggrunds-ventesvar), og kobl Session Health / token-telemetri aktivt på PreInvocation-hooks og HUD-skabelonen."
status: active
intent: enhancement
feature: telemetry
spec_ref: "specs/telemetry.md"
generated: { by: process:antigravity-task-init, at: "2026-11-14T17:58:30Z" }
tags: [hud, telemetry, session-health, token-usage, agents-md, antigravity]
scope:
  - "spec.md"
  - "specs/telemetry.md"
  - "tasks/085-hud-start-end-protocol-and-session-telemetry-wiring.md"
  - ".gitignore"
  - ".agents/AGENTS.md"
  - "gauntlet.toml"
  - "crates/xgauntlet-core/src/features/scaffold/templates.rs"
  - "crates/xgauntlet-core/src/features/telemetry/**"
  - "crates/xgauntlet-core/src/features/tasks/telemetry.rs"
  - "crates/xgauntlet-core/src/features/checkpoint/engine.rs"
  - "crates/xgauntlet-core/src/features/adapters/antigravity/**"
  - "crates/xgauntlet-cli/src/main.rs"
  - "crates/xgauntlet-core/tests/session_health_telemetry_test.rs"
  - "crates/xgauntlet-core/tests/antigravity_hooks_test.rs"
  - "crates/xgauntlet-core/tests/scaffold_engine_test.rs"
  - "crates/xgauntlet-core/tests/harness_adapters_test.rs"
  - "crates/xgauntlet-core/tests/checkpoint_engine_test.rs"
test_refs:
  - "crates/xgauntlet-core/tests/session_health_telemetry_test.rs"
  - "crates/xgauntlet-core/tests/antigravity_hooks_test.rs"
  - "crates/xgauntlet-core/tests/scaffold_engine_test.rs"
  - "crates/xgauntlet-core/tests/checkpoint_engine_test.rs"
---

# Task 085: HUD Start/End Protocol & Active Session Telemetry Wiring

**Status**: `ACTIVE`  
**Intent**: `🔄 ENHANCEMENT`  
**Oprettet**: `2026-11-14`

## 🎯 Formål
Løse to dokumenterede mangler ved Cockpit Task HUD observeret i Task 081-sessionen, uden at introducere Git-støj eller bryde filstørrelseslofter:

1. **Reduceret visuel HUD-støj (Start/Eskalering/Slut-protokol):**
   - I dag kræver [.agents/AGENTS.md](.agents/AGENTS.md) og `render_agents_md` i `crates/xgauntlet-core/src/features/scaffold/templates.rs` HUD øverst i *samtlige synlige agent-svar*, hvilket producerede 7 unødige HUD-kort pga. mellemliggende baggrunds-ventesvar (`WaitMsBeforeAsync`) uden brugerinvolvering.
   - Visuel HUD skal fremover **kun** vises ved:
     1. **Start/aktivering** af opgave eller session,
     2. **Brugerrettet eskalering/spørgsmål/godkendelse**, og
     3. **Afslutning** (`DONE` / `SESSION HANDOFF`),
     og **udelades** på mellemliggende svar uden brugerbeslutning (f.eks. *"afventer baggrundsopgave"* eller interne autonome trin), mens `PreInvocation` fortsat leverer `[XGAUNTLET COCKPIT TELEMETRY]` internt til modellen.
2. **Aktivering af LLM-/Session-Health-telemetri fra Task 044:**
   - `record_session_turn` i `crates/xgauntlet-core/src/features/telemetry/session.rs` bliver i dag aldrig kaldt i CLI eller hooks, så `.xgauntlet/session_state.json` ikke oprettes, og HUD-skabelonen i `.agents/AGENTS.md` / `templates.rs` mangler `Session:`-feltet.
   - Turn-tælling kobles deterministisk på `PreInvocation` (aldrig `PreToolUse`, jf. `<5ms` koldstart og korrekt turn-granularitet), `.xgauntlet/` ignoreres i Git og `checkpoint`, og `handoff` nulstiller session-tilstanden mellem sessioner.

## 📋 Acceptance Criteria
- [ ] **AC1 (HUD Start/Eskalering/Slut-Regel i `.agents/AGENTS.md` & Scaffold):**
  - `.agents/AGENTS.md` og `render_agents_md` i `crates/xgauntlet-core/src/features/scaffold/templates.rs` definerer utvetydigt, at det 5-linjers HUD-kort **kun** vises ved start/aktivering af opgave/session, ved brugerrettet spørgsmål/eskalering/godkendelse og ved afslutning (`DONE` / `SESSION HANDOFF`), og **udelades** på mellemliggende svar uden brugerbeslutning (herunder *"afventer baggrundsopgave"*).
- [ ] **AC2 (Aktiv `PreInvocation` Session-Wiring, Dedup & `.xgauntlet/` Isolation):**
  - `Commands::Telemetry --format antigravity-hook` i `crates/xgauntlet-cli/src/main.rs` og `PreInvocation`-grenen i `AntigravityAdapter::handle_hook` kalder en fælles helper i `features/telemetry/` for den **aktive** task (`status: active`) **efter** `< 1500ms` duplikat-check (og aldrig i `PreToolUse` / `Commands::Hook`), så `.xgauntlet/session_state.json` opdateres én gang pr. reel model-invocation.
  - `should_suppress_duplicate_telemetry` inkluderer `SessionHealthStatus` (`Optimal` / `Aging` / `Saturated`) i sin state-sammenligning, så overgange til `Aging`/`Saturated` og handoff-anbefalingen ikke skjules af 120s-undertrykkelse.
  - `.gitignore` ignorerer `.xgauntlet/`, `checkpoint/engine.rs` (`validate_staged_files_scope`) undtager `.xgauntlet/`, og `Commands::Handoff` kalder `reset_session_health` efter generering af handoff-kort.
- [ ] **AC3 (Synkroniseret HUD-Skabelon med `Session:`-Metrik):**
  - HUD-skabelonen i `.agents/AGENTS.md` og `scaffold/templates.rs` inkluderer `` | `Session: <Optimal|Aging|Saturated> (<N> turns • ~<tok>)` `` i statuslinjen på linje med `AntigravityAdapter::render_blockquote_hud` (stadig præcis 5 blockquote-linjer).
  - `AntigravityAdapter::render_blockquote_hud` bevarer bagudkompatibel fallback (udelader `Session:` når `session_health` er `None`, og viser den når `Some`).
- [ ] **AC4 (Snapshot-Sikker Transcript-/Payload-Berigelse uden Dæmon eller Netværk):**
  - En ren helper i `features/telemetry/` beriger session-tilstanden fra stdin-hook-payloaden med **snapshot-semantik** (aldrig kvadratisk akkumulering af snapshots):
    - Hvis payloaden indeholder eksplicitte token-felter (`total_tokens` / `token_count`) eller en eksisterende lokal `transcript_path` / `transcriptPath` (JSONL med `PLANNER_RESPONSE`/`assistant`-trin og tekstfelter som `content`, `thinking`, `tool_calls`), beregnes turns og `estimate_tokens_from_chars` deterministisk som monotont snapshot (`max(existing, parsed)`).
    - Hvis ingen af delene findes, faldes deterministisk tilbage til `record_session_turn` baseret på stdin-payloadens byte-størrelse.

## 🚫 Must NOT
- Må IKKE introducere baggrundsdæmoner, sockets eller eksterne netværkskald for token-beregning.
- Må IKKE koble `record_session_turn` på `PreToolUse` (`Commands::Hook`) eller kompromittere `<5ms` koldstart for tool-policy-evaluering.
- Må IKKE blokere skriveværktøjer baseret på `SessionHealth` (telemetrien er strengt rådgivende jf. Task 044).
- Må IKKE fjerne `PreInvocation`-indsprøjtningen til modellens egen kontekst undervejs i arbejdet.
- Må IKKE lade `.xgauntlet/session_state.json` gøre Git dirty eller udløse `ScopeViolation` i `xgauntlet checkpoint`.
- Må IKKE overskride `max_file_lines = 800` i `crates/xgauntlet-core/src/features/adapters/antigravity/mod.rs` (pt. 762 linjer) eller ratchet-loftet `max_lines = 2180` i `crates/xgauntlet-cli/src/main.rs` (pt. 2.102 linjer); ny telemetri-logik lægges i `crates/xgauntlet-core/src/features/telemetry/`.

## 📝 Revisions
- 2026-11-14: Oprettet draft task baseret på session-analyse (7 unødige baggrunds-HUD'er og ubrugt `record_session_turn` fra Task 044).
- 2026-11-14: Præciseret AC1–AC4 efter arkitektursparring (`.gitignore` + `checkpoint`-undtagelse for `.xgauntlet/`, `PreInvocation`-only turn-tælling + status-bevidst dedup + `handoff`-reset, snapshot-semantik i AC4, samt filstørrelseslofter i `Must NOT`), oprettet `specs/telemetry.md` og aktiveret opgaven (`status: active`).

## 🧪 Verifikation
- `cargo run -p xgauntlet-cli -- check-spec -t 085`
- `cargo run -p xgauntlet-cli -- check-architecture`
- `cargo test -p xgauntlet-core --test session_health_telemetry_test`
- `cargo test -p xgauntlet-core --test antigravity_hooks_test`
- `cargo test -p xgauntlet-core --test scaffold_engine_test`
- `cargo test -p xgauntlet-core --test harness_adapters_test`
- `cargo test -p xgauntlet-core --test checkpoint_engine_test`
