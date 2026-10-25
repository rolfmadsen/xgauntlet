---
type: Task Package
title: "Task 044: Session Health, Turn & Token Telemetry"
description: "Implementere letvægts Session Health telemetri med turn-tælling, payload-størrelse og deterministisk token-estimering i Cockpit HUD og JIT-direktiver"
status: active
generated: { by: process:antigravity-task-init, at: "2026-10-25T11:25:00Z" }
tags: [telemetry, session-health, token-budget, cockpit-hud, jit-governance, anti-context-rot]
scope: ["crates/xgauntlet-core/src/features/telemetry/**", "crates/xgauntlet-core/src/features/tasks/telemetry.rs", "crates/xgauntlet-core/src/features/adapters/**", "crates/xgauntlet-cli/**"]
test_refs: ["crates/xgauntlet-core/tests/session_health_telemetry_test.rs"]
---

# Task 044: Session Health, Turn & Token Telemetry

**Status**: `ACTIVE`  
**Intent**: `🚀 NEW FEATURE`  
**Oprettet**: `2026-10-25`  

## 🎯 Formål
Formålet med denne opgave er at give udvikleren og AI-agenten realtidsindsigt i samtalens fysiske fodaftryk (*Session Health*) for at modvirke **Context Rot** og **Attention Dilution** på en fuldstændig transparent og deterministisk måde.

I stedet for skrøbelige prompt-heuristikker (som "Canary tags" i chatteksten) eller uforståelige automatiske værktøjslåse, skal xGauntlets telemetrimotor monitorere samtalens turn-historik, målte payload-størrelser og estimerede tokenforbrug for den aktive opgave. Telemetrien viser løbende en diskret, informativ sundhedsstatus i det 5-linjers Cockpit HUD og advarer proaktivt i JIT-direktivet, når det er tid til at afslutte opgaven og tage et rent Session Handoff.

## 📋 Acceptance Criteria
- [x] **AC1 (Session Health Datamodel)**:
  - Definere `SessionHealth` og `SessionHealthStatus` i `crates/xgauntlet-core/src/features/telemetry/`:
    - `turns_count: usize`: Antal interaktions-turns siden tasken blev aktiv eller seneste checkpoint.
    - `payload_bytes: usize`: Akkumulerede bytes læst eller eksekveret via værktøjer for den aktive task.
    - `estimated_tokens: usize`: Deterministisk estimeret tokenforbrug.
    - `status: SessionHealthStatus`:
      - `Optimal` (grøn, typisk 1–10 turns).
      - `Aging` (gul, typisk 11–18 turns).
      - `Saturated` (rød, typisk >18 turns).
  - Integrere `session_health: Option<SessionHealth>` i `TaskTelemetry`.
- [x] **AC2 (Turn & Payload Tracking Engine)**:
  - Implementere letvægts, in-repo sessions-tracking (f.eks. via `.xgauntlet/session_state.json` eller Git commit/checkpoint inspektion), der registrerer turns for den aktuelle opgave.
  - Zero-Daemon invariant: Må ikke kræve baggrundsprocesser eller dæmoner; opdateres rent in-process ved hook-kald eller checkpoint.
- [x] **AC3 (Deterministisk Token & Payload Beregning)**:
  - Implementere hurtig, deterministisk BPE/tegn-ratio estimator ($\sim 1 \text{ token} \approx 3.8 \text{ tegn}$), som fungerer på tværs af alle harnesses uden eksterne netværkskald.
  - Støtte indlæsning af rå token-tal, hvis harness'et eksplicit leverer det i hook-payloaden.
- [x] **AC4 (Cockpit HUD & JIT Telemetri Integration)**:
  - Opdatere `render_blockquote_hud` og `format_ephemeral_telemetry` i `AntigravityAdapter` og øvrige adapters:
    - Vise Session Health i HUD: f.eks. `Session: 🟢 Optimal (4 turns • ~15k tok)` eller ved saturation: `Session: 🟡 Aging (14 turns) • Forbered Handoff`.
    - Når `Saturated` nås: Injicere en venlig, ikke-blokerende anbefaling i JIT-direktivet: `💡 Anbefaling: Opgaven trækker ud (>18 turns). Forsegl checkpoints og start en frisk session via 'xgauntlet handoff'.`
- [x] **AC5 (CLI & Formateringsstøtte)**:
  - Sikre at `xgauntlet telemetry --format json`, `--format box` og `--format antigravity-hook` indeholder de nye sundhedsmetrikker.
- [x] **AC6 (Testsuite & Verifikation)**:
  - Oprette `crates/xgauntlet-core/tests/session_health_telemetry_test.rs` med tests for:
    - Korrekt kategorisering af `Optimal`, `Aging` og `Saturated`.
    - Deterministisk token-estimering på kendte strenge.
    - Korrekt gengivelse i HUD og JSON telemetry.
  - 100% grøn testsuite (`cargo test --workspace`).
  - Verificere at `cargo run -p xgauntlet-cli -- check-spec -t 044` godkendes med 0 fejl.

## 🚫 Must NOT
- Må IKKE automatisk låse eller blokere skriveværktøjer baseret på session health; telemetrien er udelukkende rådgivende og transparent for brugeren.
- Må IKKE introducere baggrundsdæmoner eller socket-tjenester; state-beregning skal være øjeblikkelig og in-process (< 5ms).
- Må IKKE foretage netværkskald for at forespørge eksterne tokeniserings-API'er.

## 📝 Revisions
- 2026-10-25: Task oprettet for at give deterministisk realtidsindsigt i turn-count, payload og token-forbrug mod context rot.

## 🧪 Verifikation
- `cargo test -p xgauntlet-core --test session_health_telemetry_test`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo run -p xgauntlet-cli -- check-spec -t 044`
