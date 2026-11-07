---
type: Task Package
title: "Task 056: Simplify Fast-Track Protocol & Retire Ephemeral Worktree Inverse Proof"
description: "Forenkler Fast-Track bugfix-protokollen til et rent 2-trins RED -> DONE flow, udfaser det skrøbelige 1-trins Worktree Inverse Proof og eliminerer cold-cache compilat-overhead og krydsplatform filsystemsårbarheder."
status: draft
intent: refactor
generated: { by: process:antigravity-task-init, at: "2026-11-02T07:58:00Z" }
tags: [fast-track, dx, simplification, tdd, provenance, adr-0011, refactor]
scope:
  - "crates/xgauntlet-core/src/features/checkpoint/**"
  - "crates/xgauntlet-core/tests/fast_track_and_scoped_checkpoint_test.rs"
  - "docs/adr/0011-fast-track-worktree-inverse-proof-and-scoped-preflight.md"
  - "spec.md"
  - "CHANGELOG.md"
  - "tasks/056-simplify-fast-track-and-retire-worktree-inverse-proof.md"
test_refs:
  - "crates/xgauntlet-core/tests/fast_track_and_scoped_checkpoint_test.rs"
---

# Task 056: Simplify Fast-Track Protocol & Retire Ephemeral Worktree Inverse Proof

**Status**: `DRAFT`
**Intent**: `🔄 REFACTOR`
**Oprettet**: `2026-11-02`

---

## 🎯 Formål & Baggrund

Audit af ændringerne fra `v0.10.0` til `v0.11.3` afslørede, at introduktionen af det 1-trins atomiske "Worktree Inverse Proof" i Task 052 / ADR 0011 skabte utilsigtet arkitektonisk over-engineering og krydsplatform-ustabilitet:
1. **Cold-Cache Kompileringslatens:** 1-trins inverse proof afvikler tests i en midlertidig `git worktree` med `CARGO_TARGET_DIR = target/xgauntlet-preflight`. Dette tvinger en genkompilering af hele dependency-træet i en separat target-mappe, hvilket forlænger udviklingstiden med 30–60 sekunder i stedet for at spare tid.
2. **Skrøbelighed & Dominoeffekt på OS-niveau:** Dynamisk håndtering af worktrees i midlertidige mapper udløste tre akutte releases på 24 timer:
   - `v0.11.1` (Task 053): Windows CI-fejl på `USERPROFILE` og backslash-normalisering.
   - `v0.11.2` (Task 054): macOS CI-fejl pga. mikrosekund-uropløsning i worktree-sti-kollisioner og `/var` $\to$ `/private/var` symlink-afvigelser.
   - `v0.11.3` (Task 055): Windows CI Clippy-fejl på Unix-specifikke imports i den nye paritets-testsuite.
3. **API-inkompatibilitet ved nye signaturer:** Hvis en regressionstest introducerer en ny type eller hjælpefunktion, fejler 1-trins worktree inverse proof mod `HEAD` alligevel med en opfordring til at bruge det eksplicitte 2-trins flow.
4. **TDD-Harmoni:** Det 2-trins flow (`RED` commit $\to$ implementation $\to$ `DONE` verify/commit) er i perfekt overensstemmelse med xGauntlets TDD-kernefilosofi, genbruger den varme lokale compiler-cache og garanterer ægte negativ testevne uden behov for flygtige worktrees.

Formålet med Task 056 er at forenkle Fast-Track Bugfix Lane til det transparente 2-trins flow, udfase 1-trins worktree inverse proof i `checkpoint`, opdatere ADR 0011 og `spec.md`, og fjerne unødig kompleksitet.

---

## 📋 Acceptance Criteria

- [ ] **AC1 — Forenkling af Fast-Track Tilstandsmaskine (Rent 2-Trins `RED -> DONE` Flow):**
  - Fast-Track Bugfix Lane for opgaver med `intent: bug` og `scope.len() <= 3` (uden ændringer i ADR eller `gauntlet.toml`) fastholdes, men standardiseres udelukkende på det robuste 2-trins flow:
    1. `xgauntlet checkpoint -p red` (afvikler scoped preflight via `test_refs` og opretter lokalt `RED`-commit).
    2. Rettelse udføres i `scope`.
    3. `xgauntlet checkpoint -p done` eller `xgauntlet verify --save` (verificerer og forsegler evidens baseret på bekræftet `RedCheckpointVerified` provenance).
  - Forsøg på at køre `checkpoint -p done` på en opgave uden forudgående `RED`-commit for Fast-Track opgaver afvises klart og fail-closed med vejledning om at køre `xgauntlet checkpoint -p red` først.
- [ ] **AC2 — Udfasning af 1-Trins `evaluate_fast_track_inverse_proof`:**
  - Fjern den flygtige `target/xgauntlet-preflight`-kompilering og worktree-synkronisering fra fast-track valideringsstien i `crates/xgauntlet-core/src/features/checkpoint/`.
  - Bevar isoleret worktree-funktionalitet udelukkende hvor den er strengt nødvendig (Task 047 / defensiv verifikation mod eksisterende commits), men isolér den fra den daglige TDD inner-loop.
- [ ] **AC3 — Opdatering af Governance & Dokumentation (ADR 0011 & `spec.md`):**
  - Opdater [ADR 0011](docs/adr/0011-fast-track-worktree-inverse-proof-and-scoped-preflight.md) og `spec.md` til at afspejle, at Fast-Track Bugfix Lane defineres som det deterministiske 2-trins flow (`RED -> DONE`) uden skjult worktree-rekompilering.
- [ ] **AC4 — Regressionstest & Paritet:**
  - Tilpas `crates/xgauntlet-core/tests/fast_track_and_scoped_checkpoint_test.rs` til det forenklede flow, og bevis at samtlige tests i workspace passerer rent uden parallelle worktree-kollisioner.

---

## 🚫 Must NOT

- **Må IKKE** slække på reglen om, at regressionstests skal bevises røde (ingen "født grønne" tests tilladt).
- **Må IKKE** ændre på den scoped `test_refs` preflight i `checkpoint -p red/green/refactor` (denne tidsbesparelse skal bevares intakt).
- **Må IKKE** tillade `Fast-Track` for opgaver med `intent: feature`, `intent: refactor`, `scope.len() > 3` eller opgaver der rører `docs/adr/` eller `gauntlet.toml`.
- **Må IKKE** udføre remote publication (`git push`) jf. ADR 0003.

---

## 📝 Revisions

- 2026-11-02: Oprettet Task 056 på baggrund af DX- og nødvendigheds-audit for v0.10.0..v0.11.3.

---

## 🧪 Verifikation

- `./target/release/xgauntlet check-spec -t 056`
- `cargo test -p xgauntlet-core --test fast_track_and_scoped_checkpoint_test`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`

