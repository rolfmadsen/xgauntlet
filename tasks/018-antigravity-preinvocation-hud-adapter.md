---
type: Task Package
title: "Task 018: Google Antigravity Telemetry Hook & PreInvocation Integration"
description: "Etablere automatiseret PreInvocation hook telemetry injektion for Google Antigravity IDE i crates/xgauntlet-core og xgauntlet-cli, samt bevare det klikbare Markdown Blockquote HUD for det menneskelige interface jf. spec.md, ADR 0001, ADR 0004 og ADR 0006"
status: todo
generated: { by: process:xgauntlet-task-init, at: "2026-09-09T21:35:00Z" }
tags: [hud, telemetry, antigravity, pre-invocation, hooks, blockquote, scaffold, adr-0004, adr-0006]
---

# Task 018: Google Antigravity Telemetry Hook & PreInvocation Integration

**Status**: `TODO`
**Intent**: `🚀 NEW FEATURE`
**Oprettet**: `2026-09-09`

## 🎯 Formål
Etablere en dedikeret `PreInvocation` hook-adapter for Google Antigravity IDE i `crates/xgauntlet-core` og `crates/xgauntlet-cli`, der automatisk injicerer autoritativ cockpit-telemetri ind i modellens kontekstvindue forud for hvert agentsvar, samtidig med at det rige, klikbare Markdown Blockquote HUD bibeholdes for det menneskelige interface jf. [spec.md](spec.md), [CONTEXT.md](CONTEXT.md), [ADR 0001](docs/adr/0001-package-by-feature-architecture.md), [ADR 0004](docs/adr/0004-harness-adapter-slices.md) og [ADR 0006](docs/adr/0006-multi-harness-policy-adapter-contract.md):

1. **PreInvocation Hook Interception (`features/adapters/antigravity/mod.rs`)**:
   - Google Antigravity IDE adskiller sig fra terminal-harnesses (Claude Code, Codex) ved at understøtte `PreInvocation` livscyklus-hændelsen i `.agents/hooks.json`.
   - Implementere håndtering af `PreInvocation` i `AntigravityAdapter::handle_hook`, som returnerer det kanoniske Antigravity injektions-payload:
     ```json
     {
       "injectSteps": [
         {
           "ephemeralMessage": "[XGAUNTLET COCKPIT TELEMETRY]\nTask: <ID> | Phase: <PHASE> | Verdict: <STATUS>\nInvariants: <X/Y PASS> | Mutation: <PERCENT>%\nGit: <branch>@<oid> (<clean/dirty>) | Drift: <PERCENT>%\nEvidence: <DIGEST>"
         }
       ]
     }
     ```
   - Dette sikrer, at agenten altid modtager autoritativ sandhed om git-state, aktiv opgave og verifikationsstatus direkte fra gauntlet-motoren uden at skulle gætte eller udføre overflødige bash-kald.

2. **Menneske-rettet HUD Layout Kontrakt (Blockquote med klikbare links)**:
   - Hvor terminal-harnesses (Claude Code, Codex) anvender faste Unicode-bokse med rå filstier, har Google Antigravity IDE en rig, browserbaseret chat-brugerflade med proportional typografi (`Google Sans Flex`) og native GFM markdown rendering.
   - Fastholde og formalisere standarden for det menneske-vendte HUD i Antigravity som det transparente, 5-linjers blockquote-kort med klikbare hyperlinks:
     ```markdown
     > ### 🛡️ [Task: <Task Title / ID>] `[<Task Type>: <Phase>]`
     > **Status**: `Phase: <SPEC | RED | GREEN | REFACTOR | GAUNTLET | DONE>` | `Gauntlet: <PASS | FAIL | PENDING>` | `Git: <branch>@<oid> • <clean | dirty>`
     > **Progress**: `Criteria: X/Y [■■□□□]` | `Scope: <affected crates/paths>`
     > **Links**: 📋 [Task](tasks/) • 📄 [Spec](spec.md) • 📖 [Glossary](CONTEXT.md) • 🏛️ [ADR](docs/adr/) • 🧪 [Evidence](evidence.md)
     > 💡 **Next Action:** <kort beskrivelse af næste umiddelbare handling>
     ```
   - De klikbare links benytter direkte editor-venlige relative stier, som åbner opgavefiler, specifikationer og ADR'er direkte i IDE'ens editorfaner.

3. **CLI Wiring (`crates/xgauntlet-cli`)**:
   - Udvide telemetry-kommandoen med formatet: `xgauntlet telemetry --format antigravity-hook`.
   - Sikre at outputtet er deterministisk, formateret som gyldig JSON med `injectSteps`, og eksekverer under 5ms (Zero-Daemon Invariant).

4. **Scaffolding Integration (`features/scaffold/`)**:
   - Opdatere `xgauntlet scaffold init --harness antigravity` til automatisk at konfigurere både `PreToolUse` (sikkerhedsgate) og `PreInvocation` (telemetriinjektion) i `.agents/hooks.json`.
   - Sikre idempotent opdatering af `.agents/hooks.json`, så eksisterende brugerdefinerede hooks ikke slettes eller overskrives destruktivt.

5. **Unit-, Conformance- og Regressionstests**:
   - Udvide `crates/xgauntlet-core/tests/harness_adapters_test.rs` med testcases for:
     * `PreInvocation` payload parsing og generering af `injectSteps.ephemeralMessage`.
     * Validering af `.agents/hooks.json` scaffolding med både `PreToolUse` og `PreInvocation`.
     * Validering af, at Antigravity blockquote HUD-strukturen indeholder alle påkrævede felter og valide markdown-links.
   - Sikre at `cargo test --workspace` forbliver 100% grøn på tværs af samtlige tre harnesses (Antigravity, Claude Code, Codex).

## 📋 Acceptance Criteria
- [ ] `crates/xgauntlet-core/src/features/adapters/antigravity/mod.rs` understøtter håndtering af `PreInvocation` events og genererer `injectSteps` JSON jf. Antigravity hooks specifikationen.
- [ ] `crates/xgauntlet-cli` understøtter `xgauntlet telemetry --format antigravity-hook`, der udskriver gyldig JSON med `injectSteps: [{ "ephemeralMessage": "..." }]`.
- [ ] Telemetri-injektionen indeholder task ID, aktuel TDD-fase, invariant-status, mutation score, git HEAD hash, drift-indikator og evidens-digest.
- [ ] Scaffolderen i `crates/xgauntlet-core/src/features/scaffold/` understøtter generering og idempotent merge af `PreInvocation` hooks i `.agents/hooks.json`.
- [ ] Specifikationen og systemprompter i `.agents/AGENTS.md` definerer eksplicit det 5-linjers blockquote HUD med klikbare navigation-links for Antigravity.
- [ ] Conformance tests i `crates/xgauntlet-core/tests/harness_adapters_test.rs` verificerer Antigravity `PreInvocation` payload, hook-eksekvering og blockquote layout.
- [ ] Alle tre harness-adaptere (Antigravity, Claude Code, Codex) understøtter deres respektive telemetry hooks uden indbyrdes regressionsfejl.
- [ ] `cargo test --workspace` passerer 100% uden fejl eller advarsler.

## 🚫 Must NOT
- Må IKKE introducere baggrundsdæmoner eller runtime-sockets (Zero-Daemon Invariant).
- Må IKKE tillade remote git publication (`git push`) jf. ADR 0003.
- Må IKKE fjerne eller degradere de klikbare markdown-links i Antigravity IDE interfacet.
- Må IKKE destruktivt overskrive eksisterende brugerhooks i `.agents/hooks.json` under scaffolding.
- Må IKKE fejle åbent (fail-open) hvis telemetridata er korrupt eller ufuldstændig.

## 📝 Revisions
- 2026-09-09: Oprettet som planlagt opgave (TODO) for Google Antigravity Telemetry Hook & PreInvocation Integration (Task 018), der formaliserer PreInvocation datainjektion og bevarer det klikbare blockquote layout.

## 🧪 Verifikation
- `cargo run -p xgauntlet-cli -- check-spec -t 018`
- `cargo test -p xgauntlet-core --test harness_adapters_test`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo fmt --check`
