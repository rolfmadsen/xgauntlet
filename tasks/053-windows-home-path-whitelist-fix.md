---
type: Task Package
title: "Task 053: Cross-Platform Windows USERPROFILE Whitelist & Backslash Traversal Fix (v0.11.1)"
description: "Retter Windows CI-fejl i v0.11.0 ved manglende HOME miljøvariabel under PowerShell (USERPROFILE), normaliserer Windows drev-/backslash-stier deterministisk og hærder mod backslash-baseret '..'-traversal på tværs af OS."
status: active
intent: bug
generated: { by: process:antigravity-task-init, at: "2026-11-01T22:20:00Z" }
tags: [bugfix, windows, ci, policy, whitelist, cross-platform, security, release]
scope:
  - "crates/xgauntlet-core/**"
  - "CHANGELOG.md"
  - "Cargo.toml"
  - "Cargo.lock"
  - "package.json"
  - "packages/cli/package.json"
  - "verification-report.json"
  - "evidence.md"
  - "tasks/053-windows-home-path-whitelist-fix.md"
test_refs:
  - "crates/xgauntlet-core/tests/hook_policy_and_telemetry_test.rs"
---

# Task 053: Cross-Platform Windows USERPROFILE Whitelist & Backslash Traversal Fix (v0.11.1)

**Status**: `ACTIVE`
**Intent**: `🐛 BUG FIX`
**Oprettet**: `2026-11-01`

---

## 🎯 Formål

GitHub Actions `windows-latest` CI fejlede for `v0.11.0` (`27593c5`) i `crates/xgauntlet-core/tests/hook_policy_and_telemetry_test.rs` på testene `test_harness_whitelist_paths` og `test_session_brain_artifact_paths`.

Årsagen er todelt:
1. I Windows PowerShell (`pwsh.EXE`) på GitHub Actions er miljøvariablen `HOME` ikke sat — Windows anvender `USERPROFILE` (`C:\Users\runneradmin`). Testene faldt tilbage til `"/home/user"`, mens produktionskoden slog `USERPROFILE` op, hvorved `strip_prefix` afviste stien.
2. Produktionskodens `is_harness_whitelisted_path` og `is_session_brain_artifact_path` evaluerede kun den første miljøvariabel via `.or_else(...)` uden at filtrere tomme/rod-værdier (`HOME=""` eller `"/"`), manglede eksplicit lexikal normalisering af Windows-skråstreger (`\` vs `/`), `\\?\`-præfikser og drive-letter casing (`C:` vs `c:`), og kontrollerede kun `Component::ParentDir` før backslash-normalisering (hvilket på POSIX tillod `.agents\..\..\etc\passwd` at passere `Component::ParentDir`).

Formålet er at reproducere disse fejl deterministisk i `[RED]`, rette sti-normalisering og traversal-kontrollen på tværs af OS i `[GREEN]`, og klargøre patch-release `v0.11.1`.

---

## 📋 Acceptance Criteria

- [ ] **AC1 — Multi-Root `HOME` & `USERPROFILE` Evaluering & Windows-Sti Normalisering:**
  - `is_harness_whitelisted_path` og `is_session_brain_artifact_path` i `crates/xgauntlet-core/src/features/policy/mod.rs` evaluerer alle gyldige (ikke-tomme, ikke-rod) kandidat-hjemmemapper fra både `HOME` og `USERPROFILE`.
  - Eksponer deterministiske hjælpefunktioner (`is_harness_whitelisted_path_with_homes` og `is_session_brain_artifact_path_with_homes`), der normaliserer `\` til `/`, håndterer Windows extended-length præfiks (`\\?\C:\...`) og sammenligner Windows drevbogstaver case-insensitivt (`C:/` vs `c:/`).
- [ ] **AC2 — Cross-Platform Backslash `..`-Traversal Hærdning:**
  - Både `crates/xgauntlet-core/src/features/policy/mod.rs` og `crates/xgauntlet-core/src/features/adapters/models.rs` afviser enhver `..`-segment-komponent og `\0` efter normalisering af både `/` og `\`, så backslash-baseret traversal (`.agents\..\..\etc\passwd` eller `C:\Users\runneradmin\.gemini\config\plugins\..\..\ssh\id_rsa`) blokeres fail-closed på både Linux, macOS og Windows.
- [ ] **AC3 — Deterministiske Krydsplatform Regressionstests:**
  - `test_harness_whitelist_paths` og `test_session_brain_artifact_paths` i `crates/xgauntlet-core/tests/hook_policy_and_telemetry_test.rs` resolver hjemmemappen konsistent via `HOME` / `USERPROFILE` og suppleres med dedikerede krydsplatform-tests for Windows `USERPROFILE` (`C:\Users\runneradmin`), blandede separatorer, tom `HOME=""`, rod-ankre (`"/"`, `"C:\\"`) og backslash-traversal.
- [ ] **AC4 — Release `v0.11.1` Forsegling:**
  - Opdater `CHANGELOG.md`, bump version til `0.11.1` på tværs af manifester og `gauntlet_policy.wasm`, forsegl `verification-report.json` og `evidence.md`, og opret lokalt annoteret tag `v0.11.1` verificeret med `xgauntlet check-release --strict`.

---

## 🚫 Must NOT

- **Må IKKE** foretage `git push`, `git push --force`, sletning af eksisterende tags eller destruktive git-operationer (ADR 0003).
- **Må IKKE** acceptere tomme strenge (`""`) eller filsystemrødder (`"/"`, `"\\"`, `"C:\\"`, `"C:/"`) som gyldig `$HOME` / `USERPROFILE` rod.
- **Må IKKE** tillade sti-traversal (`..`) med hverken `/` eller `\` i whitelisted harness- eller brain-artifact-stier.

---

## 📝 Revisions
- 2026-11-01: Oprettet Task 053 til udbedring af Windows CI `USERPROFILE` / backslash-fejl fra `v0.11.0` og klargøring af `v0.11.1`.

## 🧪 Verifikation
- `./target/release/xgauntlet check-spec -t 053`
- `cargo test -p xgauntlet-core --test hook_policy_and_telemetry_test`
- `./target/release/xgauntlet verify --save -t 053`
- `./target/release/xgauntlet check-release --strict`
