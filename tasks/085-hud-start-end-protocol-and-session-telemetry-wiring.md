---
type: Task Package
title: "Task 085: HUD Start/End Protocol & Active Session Telemetry Wiring"
description: "Begræns visuelt Cockpit Task HUD til start, slut og eskalering (ingen HUD på mellemliggende baggrunds-ventesvar), og kobl Session Health / token-telemetri aktivt på hook-kald og HUD-skabelonen."
status: draft
intent: enhancement
feature: telemetry
spec_ref: "specs/telemetry.md"
generated: { by: process:antigravity-task-init, at: "2026-11-14T17:58:30Z" }
tags: [hud, telemetry, session-health, token-usage, agents-md, antigravity]
scope:
  - "specs/telemetry.md"
  - "tasks/085-hud-start-end-protocol-and-session-telemetry-wiring.md"
  - ".agents/AGENTS.md"
  - "crates/xgauntlet-core/src/features/scaffold/templates.rs"
  - "crates/xgauntlet-core/src/features/telemetry/**"
  - "crates/xgauntlet-core/src/features/tasks/telemetry.rs"
  - "crates/xgauntlet-core/src/features/adapters/antigravity/mod.rs"
  - "crates/xgauntlet-cli/src/main.rs"
  - "crates/xgauntlet-core/tests/session_health_telemetry_test.rs"
  - "crates/xgauntlet-core/tests/antigravity_hooks_test.rs"
  - "crates/xgauntlet-core/tests/scaffold_engine_test.rs"
  - "crates/xgauntlet-core/tests/harness_adapters_test.rs"
test_refs:
  - "crates/xgauntlet-core/tests/session_health_telemetry_test.rs"
  - "crates/xgauntlet-core/tests/antigravity_hooks_test.rs"
  - "crates/xgauntlet-core/tests/scaffold_engine_test.rs"
---

# Task 085: HUD Start/End Protocol & Active Session Telemetry Wiring

**Status**: `DRAFT`  
**Intent**: `🔄 ENHANCEMENT`  
**Oprettet**: `2026-11-14`

## 🎯 Formål
Løse to dokumenterede mangler ved Cockpit Task HUD observeret i Task 081-sessionen:

1. **Reduceret visuel HUD-støj (Start/Slut-protokol):**
   - I dag kræver `.agents/AGENTS.md` og `crates/xgauntlet-core/src/features/scaffold/templates.rs` HUD øverst i *samtlige synlige agent-svar*, hvilket producerede 7 mellemliggende HUD-kort alene pga. baggrunds-ventesvar (`WaitMsBeforeAsync`) uden brugerinvolvering.
   - Visuel HUD skal fremover kun vises ved **opstart/aktivering** (eller når brugerens input/godkendelse behøves) og ved **afslutning** (`DONE` / `SESSION HANDOFF`), mens `PreInvocation` fortsat leverer `[XGAUNTLET COCKPIT TELEMETRY]` internt til modellen.
2. **Aktivering af LLM-/Session-Health-telemetri fra Task 044:**
   - `record_session_turn` i `crates/xgauntlet-core/src/features/telemetry/session.rs` bliver i dag aldrig kaldt i CLI eller hooks (`Commands::Hook` / `Commands::Telemetry`), så `.xgauntlet/session_state.json` ikke oprettes, og HUD-skabelonen i `.agents/AGENTS.md` / `templates.rs` mangler `Session:`-feltet.

*(Bemærk: Den levende feature-specifikation `specs/telemetry.md` oprettes i `SPEC`-fasen ved aktivering.)*

## 📋 Acceptance Criteria
- [ ] **AC1 (HUD Start/Slut-Regel i `.agents/AGENTS.md` & Scaffold):**
  - `.agents/AGENTS.md` og `render_agents_md` i `crates/xgauntlet-core/src/features/scaffold/templates.rs` definerer klart, at det 5-linjers HUD-kort **kun** vises ved start/aktivering, ved brugerrettet eskalering/spørgsmål og ved afslutning (`DONE` / `SESSION HANDOFF`), og **udelades** på mellemliggende *"afventer baggrundsopgave"* svar.
- [ ] **AC2 (Aktiv `record_session_turn`-Wiring i CLI Hooks):**
  - `Commands::Hook` og/eller `Commands::Telemetry --format antigravity-hook` i `crates/xgauntlet-cli/src/main.rs` kalder `record_session_turn(...)` deterministisk og in-process for den aktive task, så `.xgauntlet/session_state.json` opdateres løbende og eksponeres i `[XGAUNTLET COCKPIT TELEMETRY]` (`Session: ...`) samt `render_blockquote_hud`.
- [ ] **AC3 (Synkroniseret HUD-Skabelon med `Session:`-Metrik):**
  - HUD-skabelonen i `.agents/AGENTS.md` og `scaffold/templates.rs` inkluderer `Session: <badge> (<N> turns • ~<tok>)` i statuslinjen på linje med `AntigravityAdapter::render_blockquote_hud`.
- [ ] **AC4 (Valgfri Transcript-/Token-Tracing uden Dæmon eller Netværk):**
  - Hvis hook-payloaden indeholder eksplicitte token-felter eller en lokal `transcript_path`, kan `telemetry` udlede observerede session-metrikker (LLM-kald / `thinking`-tegn / komprimeringer) med deterministisk fallback til `estimate_tokens_from_chars`.

## 🚫 Must NOT
- Må IKKE introducere baggrundsdæmoner, sockets eller eksterne netværkskald for token-beregning.
- Må IKKE blokere skriveværktøjer baseret på `SessionHealth` (telemetrien er strengt rådgivende jf. Task 044).
- Må IKKE fjerne `PreInvocation`-indsprøjtningen til modellens egen kontekst undervejs i arbejdet.

## 📝 Revisions
- 2026-11-14: Oprettet draft task baseret på session-analyse (7 unødige baggrunds-HUD'er og ubrugt `record_session_turn` fra Task 044).

## 🧪 Verifikation
- `cargo run -p xgauntlet-cli -- check-spec -t 085`
- `cargo test -p xgauntlet-core --test session_health_telemetry_test`
- `cargo test -p xgauntlet-core --test antigravity_hooks_test`
- `cargo test -p xgauntlet-core --test scaffold_engine_test`
- `cargo test -p xgauntlet-core --test harness_adapters_test`

