---
type: Task Package
title: "Task 054: IsolatedWorktree Parallel Collision Fix & Local Cross-Platform CI Parity Suite (v0.11.2)"
description: "Eliminerer parallel sti-kollision i IsolatedWorktree::create på macOS/Windows og indfører en lokal krydsplatform/concurrency CI-paritets-testsuite, så macOS/Windows-specifikke fejl fanges deterministisk lokalt før git push."
status: active
intent: bug
generated: { by: process:antigravity-task-init, at: "2026-11-01T22:44:00Z" }
tags: [bugfix, macos, windows, ci-parity, concurrency, worktree, temp-dir, determinism, release]
scope:
  - "crates/xgauntlet-core/**"
  - "crates/xgauntlet-cli/**"
  - "CHANGELOG.md"
  - "Cargo.toml"
  - "Cargo.lock"
  - "package.json"
  - "packages/cli/package.json"
  - "verification-report.json"
  - "evidence.md"
  - "tasks/054-isolated-worktree-concurrency-and-ci-parity.md"
test_refs:
  - "crates/xgauntlet-core/tests/cross_platform_ci_parity_test.rs"
---

# Task 054: IsolatedWorktree Parallel Collision Fix & Local Cross-Platform CI Parity Suite (v0.11.2)

**Status**: `ACTIVE`
**Intent**: `🐛 BUG FIX`
**Oprettet**: `2026-11-01`

---

## 🎯 Formål

GitHub Actions `macos-latest` (`aarch64-apple-darwin`) fejlede for `v0.11.1` i `provenance_engine_test.rs` med:
`git ["worktree", "add", "--detach", "--quiet", "/var/folders/.../xgauntlet-preflight-8139-1793569047577312000", "HEAD"] failed during inverse preflight: fatal: gitfile does not point to a valid repository`.

### Root Cause
1. **Parallel sti-kollision ved mikrosekund-uropløsning på macOS:** I `IsolatedWorktree::create` (`crates/xgauntlet-core/src/features/checkpoint/provenance.rs`) blev worktree-stien dannet alene ud fra `std::process::id()` (`8139`) og `SystemTime::now()...as_nanos()` (`1793569047577312000`) **uden** en atomisk monoton tæller (`AtomicU64`). På macOS tikker `SystemTime` i trin af `1.000 ns` (`1 µs`), mens Linux x86_64 har `1 ns` opløsning. Da tråd `25146` og tråd `25147` eksekverede parallelt i samme mikrosekund, valgte begge **identisk worktree-sti**, så to samtidige `git worktree add`-processer korrumperede hinandens `.git`-gitfile.
2. **macOS `/var` -> `/private/var` symlink i `std::env::temp_dir()`:** På macOS returnerer `std::env::temp_dir()` `/var/folders/...` (symlink til `/private/var/folders/...`), mens `git rev-parse --show-toplevel` returnerer den kanoniske `/private/var/folders/...`-sti. Worktree-stier bør oprettes under en kanoniseret temp-rod (med Windows `\\?\`-rensning) og atomisk `create_dir` / unik tæller.
3. **Manglende lokal CI-paritets-gate:** Lokale kørsler på Linux fangede ikke fejlen, fordi 2 tråde sjældent rammer præcis samme nanosekund på Linux, og fordi Linux `/tmp` ikke er et symlink som macOS `/var`.

Formålet er at udbedre alle temp-sti/concurrency/symlink-kilder i kodebasen og etablere en permanent lokal **Cross-Platform & Concurrency CI Parity Suite**, der deterministisk fremprovokerer macOS/Windows-fejlmønstre på udviklerens egen Linux-maskine under hver eneste `cargo test` og `xgauntlet verify`.

---

## 📋 Acceptance Criteria

- [ ] **AC1 — Kollisionsfri & Kanoniseret `IsolatedWorktree` (macOS + Windows + Parallelitet):**
  - `IsolatedWorktree::create` i `crates/xgauntlet-core/src/features/checkpoint/provenance.rs` anvender en proces-global `AtomicU64`-tæller kombineret med `pid` og tidsstempel, samt kanoniseret temp-forældremappe (med fjernelse af Windows `\\?\`-præfiks, så macOS `/var` -> `/private/var` opløses konsistent før `git worktree add`).
  - Samtlige øvrige `temp_dir()`-baserede midlertidige filer og mapper i `crates/` auditeres og sikres mod parallelle kollisioner og race conditions (inkl. atomisk skrivning af telemetri-deduplikerings-cache via unikt temp-filnavn + `rename`).
- [ ] **AC2 — Lokal Barriere-Synkroniseret Concurrency Stress-Test:**
  - `crates/xgauntlet-core/tests/cross_platform_ci_parity_test.rs` starter mindst 16 samtidige tråde synkroniseret på en `std::sync::Barrier`, som i præcis samme øjeblik kalder `verify_inverse_test_failure` og `evaluate_fast_track_inverse_proof`, og beviser `0` sti-kollisioner og `0` efterladte worktrees.
- [ ] **AC3 — Lokal macOS Symlink (`/var` -> `/private/var`) & Windows-Sti Simulering:**
  - Testsuiten opretter lokalt et workspace og en `TMPDIR`-lignende sti bag et mappe-symlink (`symlink_root -> private_real_root`) og verificerer at `IsolatedWorktree`, `WorkspaceRelativePath::sanitize`, `AntigravityAdapter::handle_hook` og `run_checkpoint` fungerer fejlfrit gennem ikke-kanoniserede symlink-stier.
- [ ] **AC4 — Mekanisk Kildekode-Invariant mod fremtidige CI-Overraskelser & Release `v0.11.2`:**
  - En statisk arkitektur-/paritetstest scanner `crates/` og fejler lokalt, hvis nogen produktions- eller testfil bruger `temp_dir()` sammen med `as_nanos()`/`process::id()` uden `AtomicU64`, eller bruger `env::var("HOME")` uden `USERPROFILE` i krydsplatform-kode.
  - Dokumenter i `CHANGELOG.md`, bump til `0.11.2`, forsegl evidens og opret lokalt tag `v0.11.2`.

---

## 🚫 Must NOT

- **Må IKKE** foretage `git push`, sletning af remote tags eller destruktive git-kommandoer (ADR 0003).
- **Må IKKE** generere midlertidige worktree- eller testmapper uden en garanteret unik proces-intern monoton tæller (`AtomicU64`).
- **Må IKKE** efterlade midlertidige mapper eller git worktree-registreringer efter testkørsel.

---

## 📝 Revisions
- 2026-11-01: Oprettet Task 054 efter `macos-latest` parallel worktree-kollision i `v0.11.1` og ønske om lokal CI-paritetsfangst.

## 🧪 Verifikation
- `./target/release/xgauntlet check-spec -t 054`
- `cargo test -p xgauntlet-core --test cross_platform_ci_parity_test`
- `./target/release/xgauntlet verify --save -t 054`
- `npm test`
