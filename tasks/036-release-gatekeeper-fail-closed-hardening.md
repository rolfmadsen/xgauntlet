---
type: Task Package
title: "Task 036: Release Gatekeeper Fail-Closed Hardening and Evidence Integrity"
description: "Fail-closed hærdning af check-release gater, fail_on_output til Go-format, fatal WASM-build, genopretning af standardprofiler, drift-gate og dokumentationsoprydning"
status: active
generated: { by: process:xgauntlet-task-init, at: "2026-10-17T10:55:00Z" }
tags: [release, gatekeeper, fail-closed, wasm, evidence, drift, profiles, adr-0005]
scope:
  - "crates/xgauntlet-core/src/features/release/**"
  - "crates/xgauntlet-core/src/features/config/**"
  - "crates/xgauntlet-core/src/features/gauntlet/**"
  - "crates/xgauntlet-cli/src/**"
  - "gauntlet.toml"
  - "Cargo.toml"
  - "CHANGELOG.md"
  - ".github/workflows/release.yml"
test_refs:
  - "crates/xgauntlet-core/tests/release_engine_test.rs"
  - "crates/xgauntlet-core/tests/gauntlet_execution_test.rs"
  - "crates/xgauntlet-core/tests/config_engine_test.rs"
---

# Task 036: Release Gatekeeper Fail-Closed Hardening and Evidence Integrity

**Status**: `ACTIVE`  
**Intent**: `🐛 BUG FIX`  
**Oprettet**: `2026-10-17`  

## 🎯 Formål
Eliminere alle "silent passes" og falske positiver i release-gatekeeperen (`check-release`), gøre proces- og formatfejl fail-closed, implementere platformsuafhængig `fail_on_output` til `gofmt`, rulle breaking stack-profiler tilbage til sikre defaults, integrere obligatorisk evidens-drift kontrol i `--strict`, samt rette fejl og uoverensstemmelser i CHANGELOG.md.

## 📋 Acceptance Criteria
- [ ] **AC1 (Fail-Closed Format Gate med Timeout)**: `check-release` evaluerer formatering via gauntlettens layer-runner med gældende `timeout_seconds`. Manglende format-binary eller konfigurationsfejl registreres som blokerende `ReleaseFinding` og giver exit 1.
- [ ] **AC2 (Fail-Closed WASM Policy Gate)**: `check-release` validerer `gauntlet_policy.wasm` deterministisk uden slugte fejl. Korrupt bytecode, I/O-fejl eller versionsmismatch registreres som blokerende `ReleaseFinding`.
- [ ] **AC3 (Portable `fail_on_output` & Go Format Fix)**: `LayerConfig` udvides med `fail_on_output: bool`. Hvis `true`, fejler laget hvis stdout/stderr er ikke-tom (bruges af `gofmt -l .` på tværs af Linux, macOS og Windows).
- [ ] **AC4 (Genopretning af Stack Profiler)**: I `default_config_for_stack` sættes `format`, `lint` og `types` til `optional: true` for zero-config eksterne projekter. Dækkes af unit tests pr. profil.
- [ ] **AC5 (Strict Evidence & Drift Gate i check-release)**: `xgauntlet check-release --strict` kræver en gyldig, uforfalsket `verification-report.json` med nul drift på tværs af source, policy, config og task digests.
- [ ] **AC6 (Fatal WASM Build & Fejlsikker Recovery i Bump)**: Fejler WASM-kompileringen under `release bump`, afbrydes processen fatalt med exit 1 og instruktion i at fortryde manifestændringer med `git checkout -- <files>`. `serde_json` konfigureres med `preserve_order` og regressionstest for nøglerækkefølge.
- [ ] **AC7 (Hærdning af Release Workflow)**: `.github/workflows/release.yml` validerer at `${GITHUB_REF_NAME#v}` matcher `package.json` og eksekverer `xgauntlet check-release --strict` før npm publish.
- [ ] **AC8 (Dokumentation & Gauntlet Sanering)**: CHANGELOG.md rettes for forkerte nøgler under 0.8.0, 0.8.2-teksten korrigeres for evidens-påstande, `gauntlet.toml` clippy tilføjes `--all-targets`, og `invariants`-laget saneres.

## 🚫 Must NOT
- Må IKKE indføre fallbacks i `verify`, der binder evidens til afsluttede opgaver eller tillader opgaveløse commits at passere som grønne (ADR 0005).
- Må IKKE benytte shell-afhængige kommandoer (`sh -c`, `test -z`) i standardprofiler, der bryder Windows-kompatibilitet.
- Må IKKE udføre automatiske netværkskald via `npx` under gauntlet-kørsler i standardprofiler.
- Må IKKE foretage remote `git push` eller destruktive git-handlinger (`git reset --hard`).

## 📦 Out of Scope
- **L4 (Generic Release Artifact Paths)**: Abstraktion af xgauntlet-specifikke stier til `[release] artifacts` i `gauntlet.toml` udskydes til en fremtidig arkitekturopgave.
- **ADR 0009**: Eventuel fremtidig obligatorisk hærdning af standardprofiler håndteres i en særskilt ADR-proces med overgangsordning.

## 📝 Revisions
- 2026-10-17: Oprettet opgavepakke til v0.8.3 patch efter to-akset review og re-review.

## 🧪 Verifikation
- `cargo test -p xgauntlet-core --test release_engine_test`
- `cargo test -p xgauntlet-core --test gauntlet_execution_test`
- `cargo test -p xgauntlet-core --test config_engine_test`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo run -p xgauntlet-cli -- check-release --strict`
- `cargo run -p xgauntlet-cli -- check-evidence`
