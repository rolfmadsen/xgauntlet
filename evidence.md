# Verification Report

**Task ID**: `036-release-gatekeeper-fail-closed-hardening`  
**Task Title**: Task 036: Release Gatekeeper Fail-Closed Hardening and Evidence Integrity  
**Verdict**: `PARTIAL`  
**Execution Origin**: `LOCAL`  
**Source Manifest Digest**: `c5aa515bbb37cc6480d8df5a8f70dfc3fd8587f5d05291411c0b8d7e45fc923e`  
**Timestamp**: `2026-10-17T13:18:31Z`  
**Head**: `0e27384`  
**Commit**: `0e27384`  

## Acceptance Criteria

- [x] **AC1 (Fail-Closed Format Gate i check-release)**: `check-release` evaluerer formatering via gauntlettens layer-runner med gældende `timeout_seconds` for alle konfigurerede, obligatoriske formatlag (herunder xGauntlets egen `optional = false` konfiguration). Manglende format-binary, spawn-fejl eller formateringsfejl registreres som blokerende `ReleaseFinding` og giver exit 1.
- [x] **AC2 (Fail-Closed WASM Policy Gate)**: `check-release` validerer `gauntlet_policy.wasm` deterministisk uden slugte fejl. Korrupt bytecode, I/O-fejl eller versionsmismatch registreres som blokerende `ReleaseFinding` og giver exit 1.
- [x] **AC3 (Portable `fail_on_output` & Go Format Fix)**: `LayerConfig` udvides med `fail_on_output: bool`. Hvis `true`, fejler laget hvis stdout/stderr er ikke-tom (bruges af `gofmt -l .` på tværs af Linux, macOS og Windows).
- [x] **AC4 (Genopretning af Stack Profiler for Eksterne Projekter)**: I `default_config_for_stack` sættes `format`, `lint` og `types` til `optional: true` for zero-config eksterne projekter, så udrullede projekter ikke knækker hvis et værktøj mangler under `verify` (giver `PARTIAL` i stedet for `FAILED`). Dækkes af unit tests pr. profil.
- [x] **AC5 (Strict Evidence & Drift Gate i check-release med defineret livscyklus)**:
- [x] **AC6 (Transaktionel WASM & Manifest Bump med Fuld Build og Disk-Staging Backup)**:
- [x] **AC7 (Gated Release Workflow med Adskilte CI- og Gate-jobs)**:
- [x] **AC8 (Bootstrapper Fail-Closed Exit-kode)**: `packages/cli/bin/xgauntlet.js` returnerer en ikke-nul exit-kode (exit 1), hvis den native engine ikke findes under `status` eller `doctor`. Dækkes af regressionstest i `packages/cli/test/launcher.test.js`, der verificerer både stdout og exit-kode.
- [x] **AC9 (CLI `--task-id` Alias & Konsistens)**: Clap `#[arg(short, long, alias = "task-id")]` tilføjes til `task`-argumentet i `Verify` (samt `CheckSpec` og `Handoff` for ensartethed), så README's dokumenterede kommando `xgauntlet verify --task-id <id>` virker direkte. Dækkes af regressionstest.
- [x] **AC10 (Dokumentation & Gauntlet Sanering)**: CHANGELOG.md rettes for forkerte nøgler under 0.8.0 (`max_file_lines`, `max_test_file_lines`, `[[architecture.exemptions]]`), 0.8.2-teksten korrigeres for evidens-påstande med en note om, at v0.8.3 hærder Tier 1 lokal evidens- og driftkontrol, `gauntlet.toml` clippy tilføjes `--all-targets`, og `invariants`-laget saneres.

---

## Verification Checks

| Check Name | Status | Exit Code | Duration (s) |
|---|---|---|---|
| `architecture` | `PASSED` | `0` | `0.076s` |
| `format` | `PASSED` | `0` | `0.152s` |
| `lint` | `PASSED` | `0` | `0.095s` |
| `types` | `PASSED` | `0` | `0.081s` |
| `unit` | `PASSED` | `0` | `18.245s` |
| `npm` | `PASSED` | `0` | `0.640s` |
| `invariants` | `PASSED` | `0` | `0.055s` |
| `mutation-testing-gauntlet` | `FAILED` | `101` | `0.012s` |

---
