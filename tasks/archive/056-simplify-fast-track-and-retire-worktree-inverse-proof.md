---
type: Task Package
title: "Task 056: Simplify Fast-Track Protocol, Retire Worktree Inverse Proof & Ultralight Bugfix DX"
description: "Forenkler Fast-Track bugfix-protokollen til et rent 2-trins RED -> DONE flow, udfaser 1-trins Worktree Inverse Proof og eliminerer ceremoni/friktion via ét-trins 'xgauntlet task new --intent bug --scope --test' og fritagelse for feature-spec mapping på små bugfixes (scope <= 3)."
status: done
intent: refactor
feature: checkpoint
spec_ref: "specs/checkpoint.md"
generated: { by: process:antigravity-task-init, at: "2026-11-02T07:58:00Z" }
tags: [fast-track, bugfix, dx, simplification, tdd, provenance, adr-0011, refactor]
scope:
  - "crates/xgauntlet-core/src/features/checkpoint/**"
  - "crates/xgauntlet-core/src/features/gauntlet/pipeline.rs"
  - "crates/xgauntlet-core/src/features/tasks/lifecycle.rs"
  - "crates/xgauntlet-core/src/features/tasks/validator.rs"
  - "crates/xgauntlet-cli/src/main.rs"
  - "crates/xgauntlet-cli/src/checkers.rs"
  - "crates/xgauntlet-cli/tests/cli_args_test.rs"
  - "crates/xgauntlet-core/tests/fast_track_and_scoped_checkpoint_test.rs"
  - "crates/xgauntlet-core/tests/cross_platform_ci_parity_test.rs"
  - "docs/adr/0011-fast-track-worktree-inverse-proof-and-scoped-preflight.md"
  - "CONTEXT.md"
  - "spec.md"
  - "specs/checkpoint.md"
  - "specs/tasks.md"
  - "CHANGELOG.md"
  - ".agents/plugins/xgauntlet/skills/tdd/SKILL.md"
  - ".agents/plugins/xgauntlet/skills/diagnose/SKILL.md"
  - "tasks/056-simplify-fast-track-and-retire-worktree-inverse-proof.md"
test_refs:
  - "crates/xgauntlet-core/tests/fast_track_and_scoped_checkpoint_test.rs"
  - "crates/xgauntlet-cli/tests/cli_args_test.rs"
---

# Task 056: Simplify Fast-Track Protocol, Retire Worktree Inverse Proof & Ultralight Bugfix DX

**Status**: `DONE`
**Intent**: `🔄 REFACTOR`
**Oprettet**: `2026-11-02`

---

## 🎯 Formål & Baggrund

Audit af ændringerne fra `v0.10.0` til `v0.11.3` samt daglig brug af Fast-Track Bugfix Lane viste to sider af samme problem — unødig kompleksitet og ceremoni ved små fejlrettelser (`intent: bug`, `scope.len() <= 3`):
1. **Cold-Cache Kompileringslatens:** 1-trins `evaluate_fast_track_inverse_proof` afvikler tests i en midlertidig `git worktree` med `CARGO_TARGET_DIR = target/xgauntlet-preflight`, hvilket tvinger en genkompilering af hele dependency-træet i en separat target-mappe (+30–60 sekunder).
2. **Skrøbelighed & Dominoeffekt på OS-niveau:** Dynamisk håndtering af worktrees i midlertidige mapper udløste tre akutte releases på 24 timer (`v0.11.1` Task 053 Windows path/env, `v0.11.2` Task 054 macOS `/var` -> `/private/var` og timestamp-kollisioner, `v0.11.3` Task 055 Windows Clippy-fejl).
3. **API-inkompatibilitet ved nye signaturer:** Hvis en regressionstest introducerer en ny type eller hjælpefunktion, fejler 1-trins worktree inverse proof mod `HEAD` alligevel med opfordring til 2-trins flow.
4. **Høj oprettelses- og spec-ceremoni på små bugfixes:** `xgauntlet task new` mangler `--scope` og `--test` til ét-trins bugfix-scaffolding, og `validate_task_feature_mapping` blokerer stadig `intent: bug` tasks med `MISSING_TASK_FEATURE_MAPPING` / `MISSING_FEATURE_SPEC_FILE`, selv om en fejlrettelse genskaber eksisterende adfærd frem for at indføre en ny feature.
5. **TDD-Harmoni:** Det rene 2-trins flow (`task new --intent bug` -> `RED` commit -> bugfix i `scope` -> `DONE` verify/commit) er i perfekt overensstemmelse med xGauntlets TDD-kernefilosofi, genbruger den varme lokale compiler-cache og garanterer ægte negativ testevne uden flygtige worktrees.

Formålet med Task 056 (sammenlagt med bugfix-DX delen fra Task 080) er at standardisere Fast-Track Bugfix Lane på det transparente 2-trins flow, udfase 1-trins `evaluate_fast_track_inverse_proof`, indføre ét-trins bugfix-scaffolding og fritage små bugfixes (`scope.len() <= 3`) for feature-spec mapping.

---

## 📋 Acceptance Criteria

- [x] **AC1 — Forenkling af Fast-Track Tilstandsmaskine (Rent 2-Trins `RED -> DONE` Flow):**
  - Fast-Track Bugfix Lane for opgaver med `intent: bug`, `scope.len() <= 3` og konkrete filstier uden `*`/`**`-wildcards (uden ændringer i ADR eller `gauntlet.toml`) fastholdes, men standardiseres udelukkende på det robuste 2-trins flow:
    1. `xgauntlet checkpoint -p red` (afvikler scoped preflight via `test_refs` og opretter lokalt `RED`-commit).
    2. Rettelse udføres i `scope`.
    3. `xgauntlet checkpoint -p done` eller `xgauntlet verify --save` (verificerer og forsegler evidens baseret på bekræftet `RedCheckpointVerified` provenance og registrerer `proof_mode = "fast_track_red_to_done"`).
  - Forsøg på at køre `checkpoint -p done` eller `execute_gauntlet_pipeline` (`verify`) på en opgave uden forudgående `RED`-commit afvises klart og fail-closed (for Fast-Track opgaver med konkret vejledning om at køre `xgauntlet checkpoint -p red` først).
- [x] **AC2 — Udfasning af 1-Trins `evaluate_fast_track_inverse_proof`:**
  - Fjern `evaluate_fast_track_inverse_proof` og den flygtige `target/xgauntlet-preflight`-worktree-synkronisering fra fast-track valideringsstien i `crates/xgauntlet-core/src/features/checkpoint/provenance.rs` og `crates/xgauntlet-core/src/features/gauntlet/pipeline.rs`.
  - Bevar isoleret worktree-funktionalitet (`verify_inverse_test_failure`) udelukkende hvor den er strengt nødvendig (Task 047 / defensiv verifikation mod eksisterende commits i `Green`/`Refactor`), isoleret fra Fast-Track `Done`-stien.
- [x] **AC3 — Kompakt CLI One-Liner til Bugfixes (`task new`):**
  - `xgauntlet task new <slug> --intent bug --scope <path> --test <test_path>` (understøttet generelt på `task new` med gentagne `--scope`/`--test` flags eller kommaseparerede stier via `value_delimiter = ','`, hvor angivne værdier erstatter default `"src/**"`/`"tests/**"` og overstyrer auto-genereret `--feature` scope) opretter en gyldig `ACTIVE` task i ét skridt, som straks består `check-spec`.
- [x] **AC4 — Ingen Feature-Spec Blokering på Små Bugfixes:**
  - Bugfix-tasks med `intent: bug`, `scope.len() <= 3` og uden `*`/`**`-wildcards blokeres ikke af `MISSING_TASK_FEATURE_MAPPING` / `MISSING_FEATURE_SPEC_FILE` i `tasks/validator.rs`, når de blot udbedrer en defekt i eksisterende kode (med mindre tasken eksplicit angiver `domain`/`feature`/`spec_ref`).
- [x] **AC5 — Opdatering af Governance, Living Specs, Glossary & Skills:**
  - Opdater [ADR 0011](docs/adr/0011-fast-track-worktree-inverse-proof-and-scoped-preflight.md), `CONTEXT.md` (fjern `**Worktree Inverse Proof**`, da `**Inverse Preflight**` dækker Task 047), `spec.md`, `specs/checkpoint.md` og `specs/tasks.md` til at afspejle, at Fast-Track Bugfix Lane defineres som det deterministiske 2-trins flow (`RED -> DONE`) uden skjult worktree-rekompilering.
  - Opdater `.agents/plugins/xgauntlet/skills/diagnose/SKILL.md` og `.agents/plugins/xgauntlet/skills/tdd/SKILL.md` så de dokumenterer det hurtige 2-trins flow (`task new --intent bug` -> `checkpoint -p red` -> fix -> `checkpoint -p done`) og fjerner 1-trins worktree inverse proof.
- [x] **AC6 — Regressionstest & Paritet:**
  - Tilpas `crates/xgauntlet-core/tests/fast_track_and_scoped_checkpoint_test.rs`, `crates/xgauntlet-core/tests/cross_platform_ci_parity_test.rs` og `crates/xgauntlet-cli/tests/cli_args_test.rs` til det forenklede flow, `task new --intent bug --scope --test` og validator-undtagelsen, og bevis at samtlige tests i workspace passerer rent uden parallelle worktree-kollisioner.

---

## 🚫 Must NOT

- **Må IKKE** slække på reglen om, at regressionstests skal bevises røde via `checkpoint -p red` (ingen "født grønne" tests tilladt).
- **Må IKKE** ændre på den scoped `test_refs` preflight i `checkpoint -p red/green/refactor` eller Task 047's `verify_inverse_test_failure` i `Green`/`Refactor` (denne sikkerheds- og tidsbesparelse skal bevares intakt).
- **Må IKKE** tillade `Fast-Track` for opgaver med `intent: feature`, `intent: enhancement`, `intent: refactor`, `scope.len() > 3` eller opgaver der rører `docs/adr/` eller `gauntlet.toml`.
- **Må IKKE** udføre remote publication (`git push`) jf. ADR 0003.

---

## 📝 Revisions

- 2026-11-02: Oprettet Task 056 på baggrund af DX- og nødvendigheds-audit for v0.10.0..v0.11.3.
- 2026-11-14: Sammenlagt med Fast-Track/bugfix-DX kravene fra Task 080 (`task new --intent bug --scope --test`, fritagelse for `MISSING_TASK_FEATURE_MAPPING`, opdatering af `diagnose`/`tdd` skills); `exit_plan_mode` (tidl. 080 AC3) skilt ud til selvstændig opgave.
- 2026-11-14: Aktiveret Task 056 og præciseret wildcard-guard (ingen `*`/`**`-globs i Fast-Track/AC4), `proof_mode = "fast_track_red_to_done"`, `CONTEXT.md`-oprydning samt fuldt fil-scope (`pipeline.rs`, `checkers.rs`, `cross_platform_ci_parity_test.rs`, `cli_args_test.rs`).

---

## 🧪 Verifikation

- `cargo run -p xgauntlet-cli -- check-spec -t 056`
- `cargo test -p xgauntlet-core --test fast_track_and_scoped_checkpoint_test`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`

