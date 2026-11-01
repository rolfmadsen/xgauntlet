---
type: Task Package
title: "Task 052: DX Optimization, Scoped Test Execution, Fast-Track Bugfix Protocol & Hook Hardening"
description: "Eliminerer unødig udviklingstid, fulde workspace-rebuilds under TDD-checkpoints, tavse hook-blokeringer og dobbelt telemetri-støj i xGauntlet v0.10.x uden at svække uafhængig blackbox-validering eller Zero Ambient Authority."
status: active
generated: { by: process:xgauntlet-task-init, at: "2026-11-01T18:25:00Z" }
tags: [dx, performance, fast-track, inverse-proof, checkpoint, test-refs, hooks, antigravity, telemetry, migrate, rust-profile]
scope:
  - "crates/xgauntlet-core/**"
  - "crates/xgauntlet-cli/**"
  - "plugins/antigravity/**"
  - ".agents/**"
  - "skills/**"
  - "docs/adr/**"
test_refs:
  - "crates/xgauntlet-core/tests/fast_track_and_scoped_checkpoint_test.rs"
  - "crates/xgauntlet-core/tests/hook_policy_and_telemetry_test.rs"
---

# Task 052: DX Optimization, Scoped Test Execution, Fast-Track Bugfix Protocol & Hook Hardening

**Status**: `ACTIVE`
**Intent**: `🔄 ENHANCEMENT`
**Oprettet**: `2026-11-01`

---

## 🎯 Formål & Empirisk Diagnose

Efter udgivelsen af `xgauntlet v0.10.0` ses en markant øget udviklingstid pr. opgave og spildte LLM-runder i større Rust-workspaces (empirisk målt i `kant` v0.9.1 med 113 tasks og 34 test-executables). Formålet er at reducere ventetid, linker-overhead og ceremoniel friktion drastisk, **mens 100 % af xGauntlets uafhængige kontrol med AI/LLM blackbox-output fastholdes**.

### Dokumenterede Root Causes (Baseline fra `kant`)
1. **Uscopede TDD Pre-flight Testkørsler (`checkpoint`):** `xgauntlet checkpoint -p red`, `-p green` og `-p refactor` kører hele stackens test-suite (`cargo test` = 34 test-binarier, `59,4s` CPU-tid og 15–25s vægtid ved re-link) fremfor kun opgavens deklarerede `test_refs` (`1,70s` vægtid).
2. **Tunge 6-fase Gennemløb ved Simple Bugfixes:** Selv små, afgrænsede fejlrettelser på 1–3 filer tvinges igennem `SPEC → RED → GREEN → REFACTOR → VERIFY → DONE` med fulde builds ved hvert trin.
3. **Hook Sandbox Selvmål & Tavs `stderr` (`xgauntlet hook antigravity`):**
   - `PreToolUse` (`matcher: "*"`) blokerer `read_file` og `write_file` på alle stier uden for workspace-roden — **herunder Antigravitys egne xGauntlet-skills** (`~/.gemini/config/plugins/xgauntlet/skills/*/SKILL.md`) og konversations-artifacts (`~/.gemini/antigravity/brain/<conversation-id>/`)!
   - Ved afvisning returnerer hooken `exit status 1` med **tom `stderr`**, så agenten ikke får årsagen at vide og spilder ekstra ture på blinde gæt.
4. **Dobbelt Telemetri-støj & Node.js Bootstrapper Overhead:**
   - Den 13-linjers `[XGAUNTLET COCKPIT TELEMETRY]`-blok injiceres **dobbelt** på både `UserPromptSubmit` og værktøjskald (`26 linjer` pr. kommando), hvilket forurener LLM'ens kontekstvindue over 30–40 tool calls og øger *Time-To-First-Token*.
   - Hvert hook-kald starter en Node.js v22 wrapper (`36 ms`) i stedet for at kalde den cachede native Rust-binærfil direkte (`~2–3 ms`).
5. **Redundante Lag i Rust `gauntlet.toml` & Multi-Binary Linker-skalering:**
   - Standard Rust-profilen kører både `cargo clippy` og `cargo check` (100 % redundant), samt `cargo test` efterfulgt af `cargo test -- proptest` (gennemløber alle 34 binarier to gange).
   - `tdd`-skillen guider agenten til at oprette en ny `[[test]]`-binærfil i `Cargo.toml` pr. feature i stedet for modul-opdelte filer under én samlet integrationstest-harness.

---

## 📋 Acceptance Criteria

- [ ] **AC1 — `test_refs`-bundet Pre-flight Eksekvering i `xgauntlet checkpoint`:**
  - `xgauntlet checkpoint` i faserne `red`, `green` og `refactor` parser opgavens `test_refs` fra OKF v0.2 frontmatter og afvikler **udelukkende** disse specifikke tests fremfor hele workspace-suiten.
  - **Rust Target Resolver:** For en sti i `test_refs` (f.eks. `tests/features/foo_acceptance_test.rs`) inspicerer resolveren `Cargo.toml` og `tests/`:
    - Hvis filen er et selvstændigt integrationstest-target (`[[test]]` eller top-level `tests/<stem>.rs`), køres `cargo test --test <target_name>`.
    - Hvis filen er et undermodul under en samlet harness (f.eks. `tests/features.rs` eller `tests/features_acceptance.rs`), køres `cargo test --test <harness_name> <module_stem>`.
    - Hvis filen ligger i `src/`, køres `cargo test --lib <module_path>`.
  - **Kvalificeret `RED`-bevis:** I `-p red` skelner pre-flight mellem en syntaks-/kompileringsfejl i eksisterende kode og en reel test-fejl; test-targetet skal enten kompilere og fejle ved testkørsel (`exit code 101` fra test-runner) eller fejle kontrolleret på et nyt manglende symbol logget eksplicit i outputtet.
  - Fuld multi-layer verifikation (`gauntlet.toml`) eksekveres uændret og uafkortet ved `xgauntlet verify` og `xgauntlet checkpoint -p done`.

- [ ] **AC2 — Fast-Track Bugfix Protokol & Worktree Inverse Proof (`intent: bug`):**
  - En opgave kvalificerer automatisk til **Fast-Track Bugfix Lane**, når samtlige 3 betingelser er opfyldt:
    1. Taskens intent er `bug` (`intent: bug` / `🐛 BUG FIX`),
    2. `scope` indeholder højst 3 kildefiler (`scope.len() <= 3`), og
    3. Ingen filer i `docs/adr/` eller `gauntlet.toml` indgår i ændringen.
  - **2-trins `RED → DONE` Flow:** For kvalificerede Fast-Track tasks er separat `SPEC`-commit og separat `REFACTOR`-commit valgfrie. Tilstandsmaskinen tillader direkte `checkpoint -p red` (kun `test_refs`) → kodeændring i `scope` → `checkpoint -p done` (som afvikler fuld `verify --save` og forsegler evidens).
  - **1-trins Atomisk Worktree Inverse Proof:** Hvis en kvalificeret Fast-Track task kalder `verify` / `checkpoint -p done` med både regressionstest (`test_refs`) og bugfix (`scope`) i samme trin (uden forudgående `RED`-commit), udfører `xgauntlet` mekanisk invers verifikation i en isoleret midlertidig `git worktree` i OS temp-dir (med garanteret oprydning ved succes, fejl og panic):
    1. Udcheck `HEAD` i den midlertidige worktree og overfør **kun** ændringerne fra `test_refs` (mens `scope`-produktionskoden forbliver uændret på `HEAD`).
    2. Afvikl den scopede test fra `test_refs`.
    3. **Gatekeeper-krav:** Hvis testen **passerer** mod uændret `HEAD`, afvises verifikationen hårdt med `BORN_GREEN_TEST: Regression test passed without production fix`. Hvis testen ikke kan kompilere mod `HEAD` pga. nye API-signaturer, gives en klar fejlbesked om at benytte eksplicit `RED → DONE` flow.
    4. Hvis testen **fejler** mod `HEAD`, og samtlige lag i `gauntlet.toml` **passerer** i det aktive workspace med `scope`-ændringen, godkendes beviset og logges i `verification-report.json` (`proof_mode: "worktree_inverse_proof"`).
  - For `intent: feature`, `intent: refactor` eller opgaver med `scope.len() > 3` fastholdes den fulde protokol (`SPEC → RED → GREEN → [REFACTOR] → DONE`).

- [ ] **AC3 — Hook Policy Engine: Harness Whitelist, Sikker `run_command` & Actionable `stderr`:**
  - **`run_command` forbliver fuldt gatekeeped:** `PreToolUse` evaluerer fortsat både fil-skriveværktøjer (`edit_file`, `write_file` m.fl.) og shell-/eksekveringsværktøjer (`run_command`), så en blackbox-agent aldrig kan omgå `scope`, fase-gates eller git-invarianter (`git push`, `git reset --hard`) via shell-scripts.
  - **Read-Only & Harness-Sti Whitelist:**
    - Ikke-muterende værktøjer (`read_file`, søgeværktøjer samt interaktive UI-/planlægningsværktøjer som `ask_question` og `manage_task`) blokeres **ikke** i `SPEC`-fasen.
    - Read-only adgang tillades eksplicit til kendte agent-harness konfigurations- og skill-mapper uden for workspace (`~/.gemini/config/plugins/`, `~/.gemini/antigravity/builtin/`, `~/.agents/`, `~/.claude/`, `~/.codex/`), så agenten altid kan læse `SKILL.md`-filer.
    - Læsning og skrivning i Antigravitys konversations-specifikke artifact/scratch-mappe (`~/.gemini/antigravity/brain/<conversation-id>/`) tillades uden at udløse workspace path-containment fejl.
  - **Obligatorisk `stderr` Kontrakt (Nul tavse `exit 1`):** Enhver policy-afvisning i `xgauntlet hook` SKAL skrive en struktureret fejlforklaring på `stderr`:
    `[XGAUNTLET POLICY BLOCKED] Tool: <name> | Task: <id> (Phase: <phase>) | Reason: <violation> | Recovery: <exact command or action>`.

- [ ] **AC4 — Telemetri-Deduplikering, State-Diffing & Direkte Native Binær Hook:**
  - **Kontekst-besparende Telemetri:**
    - Fuldt `[XGAUNTLET COCKPIT TELEMETRY]` kort emitteres på `UserPromptSubmit` (præcis én gang pr. bruger-tur).
    - `PreToolUse` udskriver **intet** ved tilladte kald (`exit 0`).
    - `PostToolUse` udskriver **intet** på read-only værktøjer og emitterer kun telemetri på muterende/eksekverende kald, når `Phase`, `Task` eller `Verdict` faktisk har ændret sig siden sidste emission.
  - **Deduplikering af Hook-registreringer (`plugin` & `doctor`):**
    - `xgauntlet plugin` og `xgauntlet doctor` detekterer og fjerner dublerede hook-registreringer på tværs af global `~/.gemini/settings.json` og plugin-filen `~/.gemini/config/plugins/xgauntlet/hooks.json`.
  - **Direkte Rust-Binærfil i Hooks:**
    - `xgauntlet plugin` binder `hooks.json` direkte til stien for den verificerede, cachede native `xgauntlet` Rust-executable fremfor `#!/usr/bin/env node` bootstrapperen (~36 ms → ~2–3 ms pr. kald), med idempotent opdatering ved versionsskift.

- [ ] **AC5 — Dedupliceret Rust Stack-Profil, `xgauntlet migrate` & Konsolideret Test-Harness Mønster:**
  - **Opdateret Rust `gauntlet.toml` profil (`xgauntlet init`):**
    - Fjerner det redundante `types` (`cargo check`) lag og sætter `lint` til `["cargo", "clippy", "--all-targets", "--", "-D", "warnings"]`.
    - Konfigurerer `invariants` målrettet (f.eks. `["cargo", "test", "--test", "proptests"]`), så hele flåden af integrationstest-binarier ikke gennemløbes to gange under `verify`.
  - **Kirurgisk Opgradering i `xgauntlet migrate`:**
    - Fjerner idempotent redundant `types = ["cargo", "check"]` i eksisterende Rust `gauntlet.toml` når `clippy` er aktiv, strammer `invariants`-kommandoen hvis `tests/proptests.rs` eksisterer, og rapporterer en anbefaling i `doctor`/`migrate` hvis `Cargo.toml` har > 10 separate `[[test]]`-binarier.
  - **Opdaterede Skills (`skills/tdd/SKILL.md` & `skills/spec/SKILL.md`):**
    - Dokumenterer Fast-Track Bugfix Lane (`intent: bug`) samt modul-baserede Rust accepttests under én samlet harness (`tests/features.rs` / `tests/features/mod.rs`, hvor hver modulfil fortsat overholder `< 1.000` linjer jf. `max_test_file_lines`, men linkes som én binærfil).

- [ ] **AC6 — ADR & Selv-Verifikation:**
  - Ny ADR i `docs/adr/` dokumenterer sikkerhedsmodellen for **Fast-Track Worktree Inverse Proof** og **Scoped Pre-flight vs. Full Final Gate**.
  - Samtlige unit-, integrations- og invariant-tests i `xgauntlet` passerer `xgauntlet verify --save`.

---

## 🚫 Must NOT (Ufravigelige Gatekeeper-Invarianter)

- Må **ALDRIG** fjerne `run_command` (eller andre shell-/exec-værktøjer) fra `PreToolUse` policy-evalueringen — en blackbox-LLM må under ingen omstændigheder kunne mutere filer uden for `scope` eller køre destruktive git-kommandoer via shell.
- Må **ALDRIG** lade en test passere i Fast-Track uden mekanisk bevis for, at testen fejler mod uændret produktionskode (`RED`-checkpoint eller Worktree Inverse Proof), medmindre nødflaget `--allow-born-green` er eksplicit sat.
- Må **ALDRIG** køre `git stash`, `git checkout` eller `git reset` i udviklerens aktive working tree under Inverse Proof — al invers testkørsel SKAL foregå i en isoleret midlertidig `git worktree` med garanteret oprydning.
- Må **ALDRIG** afslutte `xgauntlet hook` med `exit status != 0` uden en eksplicit fejlforklaring på `stderr`.
- Må **ALDRIG** reducere verifikationsdybden i det afsluttende `xgauntlet verify` ved `DONE` — fuld regressions-, arkitektur-, spec- og evidensforsegling skal altid gennemføres før lukning.

---

## 📝 Revisions

- 2026-11-01: Oprettet på baggrund af empirisk performance- og sikkerhedsaudit i Kant (v0.9.1 / Task 113).

---

## 🧪 Verifikation

- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `xgauntlet check-spec`
- `xgauntlet check-adr`
- `xgauntlet verify --save`