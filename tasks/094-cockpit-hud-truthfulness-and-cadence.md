---
type: Task Package
title: "Task 094: Cockpit HUD Truthfulness, Session Resolution & Antigravity Start-HUD Cadence"
description: "Gør Cockpit HUD-telemetrien empirisk sandfærdig på tværs af harnesses, retter Task 093's session_id-læsefejl i inspect_task_telemetry, opgraderer Antigravity PreInvocation fra forældet ephemeralMessage til systemMessage + metadata.user_markdown med Smart Kadence (inkl. kompakt Read-Only banner ved Task: NONE) og opdaterer AGENTS.md samt scaffold-skabeloner."
status: active
intent: enhancement
generated: { by: process:antigravity-task-init, at: "2026-11-15T14:20:00Z" }
tags: [hud, telemetry, antigravity, adapters, session-health, ux, dx]
feature: telemetry
spec_ref: "specs/telemetry.md"
scope:
  - "tasks/094-cockpit-hud-truthfulness-and-cadence.md"
  - "CONTEXT.md"
  - "spec.md"
  - "specs/telemetry.md"
  - "specs/adapters.md"
  - ".agents/AGENTS.md"
  - "crates/xgauntlet-core/src/features/tasks/**"
  - "crates/xgauntlet-core/src/features/telemetry/**"
  - "crates/xgauntlet-core/src/features/adapters/**"
  - "crates/xgauntlet-core/src/features/scaffold/templates.rs"
  - "crates/xgauntlet-cli/src/main.rs"
  - "crates/xgauntlet-cli/src/handlers.rs"
  - "crates/xgauntlet-cli/src/render.rs"
  - "crates/xgauntlet-core/tests/harness_adapters_test.rs"
  - "crates/xgauntlet-core/tests/harness_contracts_test.rs"
test_refs:
  - "crates/xgauntlet-core/tests/cockpit_hud_truthfulness_test.rs"
  - "crates/xgauntlet-core/tests/antigravity_hooks_test.rs"
  - "crates/xgauntlet-core/tests/session_health_telemetry_test.rs"
  - "crates/xgauntlet-core/tests/task_lifecycle_test.rs"
---

# Task 094: Cockpit HUD Truthfulness, Session Resolution & Antigravity Start-HUD Cadence

**Status**: `ACTIVE`
**Intent**: `🔄 ENHANCEMENT`
**Oprettet**: `2026-11-15`

## 🎯 Formål
Et gennemført UX/DX-review af Cockpit HUD afdækkede tre centrale svagheder efter Task 093:
1. **Utroværdige / hårdkodede felter**: `AntigravityAdapter::format_ephemeral_telemetry`, `render_blockquote_hud` og `TaskTelemetry::render_box_card` gætter `Gauntlet: PASS` ud fra `phase`, hårdkoder `"14/14 PASS"`, `"Mutation: 100%"`, `"Drift: 0%"` og `"crates/*"`, og ignorerer `verification-report.json`, opgavens reelle `scope`, dirty out-of-scope filer, worktree/task-binding samt `commits_ahead`.
2. **Task 093 session-læsefejl + turn-only sundhed**: `prepare_pre_invocation_payload` gemmer per-session data i `.xgauntlet/sessions/<hash>.json` (med `session_id`), men kalder derefter `inspect_task_telemetry(workspace, explicit_task)` med `session_id = None`, så `Session: ...` forsvinder i praksis; desuden måler `SessionHealthStatus` kun turns og ignorerer `estimated_tokens`.
3. **Forældet og passiv HUD-kadence i Antigravity**: `PreInvocation` udsender stadig forældet `ephemeralMessage`, returnerer tom `injectSteps: []` ved `Task: NONE` (eller forfalsket `14/14 PASS` i `handle_hook`), undertrykker gentagne prompts i 120s selv ved ny brugerbesked og viser i dag HUD'et alt for sent (først i agentens slutbesked i stedet for straks under brugerens prompt).

Opgaven samler sandheden i delte telemetri-modeller (`TaskTelemetry` / `CockpitTelemetry` / `IdleWorkspaceTelemetry`), migrerer Antigravity `PreInvocation` til `systemMessage` + `metadata.user_markdown` med **Smart Kadence**, håndterer `Task: NONE` med en kompakt Read-Only status-banner og synkroniserer både `render_box_card()` og `AGENTS.md`-dokumentationen.

## 📋 Acceptance Criteria

### AC1: Sandfærdig `TaskTelemetry`, Per-Session Opslag & Token-Aware `SessionHealth`
- [x] `inspect_task_telemetry_for_session(workspace, task_id_or_path, session_id)` slår session-telemetri op via `SessionTracker::load_for_session(..., session_id)` (og når `session_id` er `None`, falder sikkert tilbage til nyeste opgave-matchende fil i `.xgauntlet/sessions/*.json` hvis `.xgauntlet/session_state.json` ikke matcher); `AntigravityAdapter::prepare_pre_invocation_payload` giver det ekstraherede `session_id` videre, så Antigravity-sessioner altid viser aktuel turn- og token-belastning.
- [x] `SessionHealthStatus::from_usage(turns, estimated_tokens)` (brugt af `SessionHealth::new`) kombinerer turns og estimerede tokens deterministisk (`Optimal`: `turns <= 10 && estimated_tokens < 60_000`; `Aging`: `turns >= 11 || estimated_tokens >= 60_000`; `Saturated`: `turns > 18 || estimated_tokens >= 120_000`).
- [x] `TaskTelemetry` beriges (via bagudkompatibelt `#[serde(default, skip_serializing_if = "Option::is_none")] pub cockpit: Option<CockpitTelemetry>` samt befolkede `scope`, `invariants` og `evidence`) med hurtige (`<5ms`, ingen fulde builds/rekursiv hashing) og empirisk sande signaler:
  - **Verification Freshness & Lag-tæller** fra `verification-report.json`: `NONE` (mangler), `STALE` (forkert `task_id`, dirty working tree, eller `report` commit != `HEAD`), `PASS`/`FAIL` med reelt beståede/fejlede lag og reel mutationsstatus — aldrig hårdkodet `"14/14 PASS"`, `"Mutation: 100%"`, `"Drift: 0%"` eller phase-gættet `PASS`.
  - **Scope & Scope Hygiene**: faktisk `scope`-liste fra opgavens YAML frontmatter (eller `"unscoped"` når tom — aldrig opdigtet `"crates/*"`) samt optælling af dirty filer inden for vs. uden for opgavens `scope` (Scope Hygiene).
  - **VCS Worktree, Task Binding & Ahead**: om checkout'et er `primary` eller `worktree:<navn>` (`resolve_primary_repo_root`), hvordan opgaven er bundet (`.xgauntlet/active_task` vs. auto/explicit), samt `commits_ahead` mod `@{upstream}` når upstream findes.

### AC2: Antigravity `PreInvocation` (`systemMessage` + `metadata.user_markdown`), Smart Kadence & `Task: NONE`
- [x] `AntigravityAdapter::format_pre_invocation_payload` og `prepare_pre_invocation_payload` anvender det autoritative Antigravity `injectSteps`-format: `{ "systemMessage": { "systemMessage": "<model-kontekst>", "metadata": { "user_markdown": "<Start-HUD markdown>" } } }` i stedet for forældet `ephemeralMessage`; `adapters/antigravity/validation.rs` anerkender desuden `"SessionStart"` i `VALID_ANTIGRAVITY_EVENTS`.
- [x] **Smart Kadence** (scoping per `(workspace, session_id)` med eksplicit testbar cache-dir-seam):
  - Første prompt i en session eller prompt med **strukturelt state-skifte** (`task_id`, `phase`, `verification_freshness`, `criteria` progress, `worktree`/`branch`, `session_health.status`, eller `dirty_out_of_scope > 0`) ved `invocationNum == 1` (eller udeladt): udsender **fuld Start-HUD** i `metadata.user_markdown` og fuld `[XGAUNTLET COCKPIT TELEMETRY]` i `systemMessage`.
  - Efterfølgende bruger-prompts (`invocationNum == 1` eller udeladt) med **uændret** strukturelt state (efter `<1500ms` identisk duplikat-vagt for samme `invocationNum`): kollapser `metadata.user_markdown` til en **1-linjes kompakt status-chip** (`> 🛡️ ...`) og bevarer frisk turn/token-telemetri i `systemMessage`.
  - Interne tool-loops (`invocationNum > 1`): udelader **altid** `metadata.user_markdown` (`injectSteps: []` ved uændret state; kun `systemMessage` uden `metadata.user_markdown` hvis state skiftede under tool-loopet).
- [x] **`Task: NONE` Read-Only Banner**: Både `xgauntlet telemetry --format antigravity-hook` og `xgauntlet hook antigravity` (`PreInvocation`) anvender én fælles `NoActiveTask`-sti, der returnerer en ærlig, kompakt Read-Only status-banner (Git branch/OID/ahead/dirty/worktree, `Policy: Read-Only / Planning`, draft/done opgaveantal og `verification-report.json` friskhed) underlagt Smart Kadence — aldrig tom `injectSteps: []` eller opdigtet `"14/14 PASS"`.

### AC3: Synkroniseret Blockquote HUD, `render_box_card()`, CLI & `AGENTS.md`
- [x] `AntigravityAdapter::render_blockquote_hud` og `format_ephemeral_telemetry` viser de sande `CockpitTelemetry`-signaler (inkl. `Gauntlet` friskhed, `Scope` + eventuel out-of-scope advarsel, `Worktree` / `Primary`, `ahead` og `Session`), uden falsk `PASS`-gætning.
- [x] `TaskTelemetry::render_box_card()` (Claude Code / Codex / Mistral) erstatter vildledende fallbacks (`"14/14 PASS"` -> `"unverified"`, `"crates/*"` -> `"unscoped"`, og udnytter ægte `cockpit`-signaler når sat) under streng overholdelse af 64 synlige tegn pr. linje og 6 linjers total højde.
- [x] `.agents/AGENTS.md` og `crates/xgauntlet-core/src/features/scaffold/templates.rs` (`render_agents_md`) opdateres til at reflektere det opdaterede HUD-design og kadence-protokollen: brugeren får Start-HUD direkte efter prompten via hook (`metadata.user_markdown`), mens agentens egen svar-HUD udelukkende vises ved **state-delta / faseskift**, **eskalering / bruger-gate** eller **afslutning (`DONE` / `🏁 SESSION HANDOFF`)**.
- [x] `crates/xgauntlet-cli` (`xgauntlet telemetry`) spejler den sandfærdige `TaskTelemetry` / `IdleWorkspaceTelemetry` ved både `--format antigravity-hook`, `--format json` og standardtekst.

## 🔄 Migrerede Eksisterende Test-Kontrakter (Gennemsigtig Gate 1-Oversigt)
Følgende eksisterende test-assertions Tester direkte de forældede felter/formater og opdateres eksplicit i denne opgave:
- `crates/xgauntlet-core/tests/antigravity_hooks_test.rs`: læser `injectSteps[0]["ephemeralMessage"]` → opdateres til `injectSteps[0]["systemMessage"]["systemMessage"]`.
- `crates/xgauntlet-core/tests/harness_adapters_test.rs`: asserterer `ephemeralMessage` samt hårdkodede strenge `"Mutation: 100%"` og `"Drift: 0%"` → opdateres til `systemMessage` + sandfærdige felter.
- `crates/xgauntlet-core/tests/harness_contracts_test.rs` (linje 937 + `TaskTelemetry` literals): asserterer `inject_steps[0]["ephemeralMessage"]` → opdateres til `systemMessage`.
- `crates/xgauntlet-core/tests/session_health_telemetry_test.rs`: evt. direkte `TaskTelemetry { ... }` struct literals tilføjes `cockpit: None`.

## 🚫 Must NOT
- Kør IKKE tunge builds (`cargo test`, `cargo clippy`, `cargo mutants`) eller rekursiv SHA-256 hashing af alle filer under `PreInvocation` eller `inspect_task_telemetry` (bevar `< 5ms` hurtig sti pr. `specs/telemetry.md`).
- Indsæt ALDRIG hårdkodede optimisme-værdier (`"14/14 PASS"`, `"Mutation: 100%"`, `"Drift: 0%"`, `"crates/*"`) eller phase-gættet `Gauntlet: PASS` når evidens mangler eller er stale.
- Udsend ALDRIG `metadata.user_markdown` under interne tool-loops (`invocationNum > 1`).
- Bryd IKKE filgrænserne i `gauntlet.toml` (`max_file_lines = 800`, `max_test_file_lines = 1000`); udskil Antigravity HUD-/kadencelogik i et dedikeret delmodul (f.eks. `crates/xgauntlet-core/src/features/adapters/antigravity/hud.rs`) og læg nye accepttests i `crates/xgauntlet-core/tests/cockpit_hud_truthfulness_test.rs`.
- Foretag ALDRIG `git push`, `git reset --hard` eller `git clean -f` (ADR 0003).
- Tilføj IKKE nye eksporterede `pub struct` / `pub enum` i `features/` uden tilsvarende Aristoteles-begreber i `CONTEXT.md` (`check-spec --context`).

## 📝 Revisions
- 2026-11-15: Oprettet draft-opgavepakke fra UX/DX HUD-review og Discovery Q&A.

## 🧪 Verifikation
- `cargo test --package xgauntlet-core --test cockpit_hud_truthfulness_test`
- `cargo test --package xgauntlet-core --test antigravity_hooks_test --test harness_adapters_test --test harness_contracts_test --test session_health_telemetry_test --test task_lifecycle_test`
- `cargo run -p xgauntlet-cli -- check-spec -t 094`
- `cargo run -p xgauntlet-cli -- check-spec --context`
- `cargo run -p xgauntlet-cli -- verify --task 094 --save`

