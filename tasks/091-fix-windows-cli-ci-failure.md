---
type: Task Package
title: "Task 091: Fix Windows CI Failure in xgauntlet-cli Integration Tests (v0.15.1)"
description: "Reproducerer og retter Windows-fejlen i xgauntlet-cli integrationstests (check_attestation_test.rs hvor xgauntlet.exe fejlede på alle subkommandoer inkl. --help) samt klargør patch release v0.15.1."
status: active
intent: bug
generated: { by: process:antigravity-task-init, at: "2026-11-14T22:53:00Z" }
tags: [windows, ci, cli, bugfix, stack-size, v0.15.1]
scope:
  - "crates/xgauntlet-cli/src/main.rs"
  - "crates/xgauntlet-cli/src/**"
  - "crates/xgauntlet-cli/tests/check_attestation_test.rs"
  - "Cargo.toml"
test_refs:
  - "crates/xgauntlet-cli/tests/check_attestation_test.rs"
  - "crates/xgauntlet-cli/tests/cli_args_test.rs"
---

# Task 091: Fix Windows CI Failure in xgauntlet-cli Integration Tests (v0.15.1)

**Status**: `ACTIVE`
**Intent**: `🐛 BUG FIX`
**Oprettet**: `2026-11-14`

---

## 🎯 Formål

Diagnosticere og rette årsagen til at Windows CI (`windows-latest`, `rustc 1.100.0`) fejlede på 5 af 6 tests i [crates/xgauntlet-cli/tests/check_attestation_test.rs](../crates/xgauntlet-cli/tests/check_attestation_test.rs) — herunder allerede ved `xgauntlet check-attestation --help` og uden stdout-output på `DIGEST_MISMATCH`/`REPOSITORY_ID_MISMATCH` — samt sikre at CLI binæren kører deterministisk under Windows' 1 MB tråd-stakgrænse i debug-builds, og derefter klargøre patch release `v0.15.1`.

---

## 📋 Acceptance Criteria

- [ ] **AC1 — Empirisk reproduktion & root-cause verificeret:** Årsagen til at `xgauntlet` fejler på Windows (herunder 1 MB debug-stack / `async fn main()` future-størrelse) er reproduceret og bevist i test (`RED`).
- [ ] **AC2 — Minimal & robust CLI-rettelse:** `xgauntlet` CLI'en eksekverer `Cli::parse()` og alle subkommandoer (inkl. `check-attestation --help` og fulde testscenarier) stabilt inden for en 1 MB debug-stak uden unødvendig `tokio`-runtime eller gigantisk monolitisk async state machine.
- [ ] **AC3 — Forbedret test-diagnostik & grøn verifikation:** `check_attestation_test.rs` logger stdout/stderr/exit-status ved fejl, og hele `cargo test --workspace` + `cargo clippy --workspace --all-targets -- -D warnings` + `cargo fmt --check` passerer.

---

## 🚫 Must NOT

- **Må IKKE** øge `gauntlet.toml` `max_lines` ratchet for `crates/xgauntlet-cli/src/main.rs` (`2180`).
- **Må IKKE** udføre `git push` eller destruktive git-handlinger (ADR 0003).
- **Må IKKE** maskere fejl i tests eller springe Windows-kompatible kontroller over.

---

## 📝 Revisions

- 2026-11-14: Oprettet fejlsøgnings- og rettelsesopgave på baggrund af Windows CI-log for `v0.15.0`.

---

## 🧪 Verifikation

- `cargo test -p xgauntlet-cli --test check_attestation_test`
- `cargo test -p xgauntlet-cli --test cli_args_test`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
