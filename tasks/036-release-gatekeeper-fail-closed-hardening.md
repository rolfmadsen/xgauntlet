---
type: Task Package
title: "Task 036: Release Gatekeeper Fail-Closed Hardening and Evidence Integrity"
description: "Fail-closed hærdning af check-release gater, reusable CI release-gate, fail_on_output til Go-format, fatal WASM-build, genopretning af standardprofiler, bootstrapper exit-kode, CLI-alias og dokumentationsoprydning"
status: active
generated: { by: process:xgauntlet-task-init, at: "2026-10-17T11:05:00Z" }
tags: [release, gatekeeper, fail-closed, wasm, evidence, drift, profiles, ci, bootstrapper, adr-0005]
scope:
  - "crates/xgauntlet-core/src/features/release/**"
  - "crates/xgauntlet-core/src/features/config/**"
  - "crates/xgauntlet-core/src/features/gauntlet/**"
  - "crates/xgauntlet-cli/src/**"
  - "packages/cli/bin/xgauntlet.js"
  - "gauntlet.toml"
  - "Cargo.toml"
  - "CHANGELOG.md"
  - ".github/workflows/ci.yml"
  - ".github/workflows/release.yml"
test_refs:
  - "crates/xgauntlet-core/tests/release_engine_test.rs"
  - "crates/xgauntlet-core/tests/gauntlet_execution_test.rs"
  - "crates/xgauntlet-core/tests/config_engine_test.rs"
  - "crates/xgauntlet-cli/tests/cli_args_test.rs"
---

# Task 036: Release Gatekeeper Fail-Closed Hardening and Evidence Integrity

**Status**: `ACTIVE`  
**Intent**: `🐛 BUG FIX`  
**Oprettet**: `2026-10-17`  

## 🎯 Formål
Eliminere alle "silent passes" og falske positiver i release-gatekeeperen (`check-release`), sikre at obligatoriske kontroller der ikke kan gennemføres altid fejler lukket (exit 1), etablere en reel forudgående release-gate i GitHub Actions via et genbrugeligt CI-workflow (`workflow_call`), implementere platformsuafhængig `fail_on_output` til `gofmt`, rulle breaking stack-profiler tilbage til sikre defaults, sikre ikke-nul exit-kode i bootstrapperen ved manglende engine, tilføje `--task-id` som officielt CLI-alias, og rette uoverensstemmelser i CHANGELOG.md.

## 📋 Acceptance Criteria
- [ ] **AC1 (Fail-Closed Format Gate med Timeout)**: `check-release` evaluerer formatering via gauntlettens layer-runner med gældende `timeout_seconds`. Manglende format-binary, spawn-fejl eller konfigurationsfejl registreres som blokerende `ReleaseFinding` og giver exit 1.
- [ ] **AC2 (Fail-Closed WASM Policy Gate)**: `check-release` validerer `gauntlet_policy.wasm` deterministisk uden slugte fejl. Korrupt bytecode, I/O-fejl eller versionsmismatch registreres som blokerende `ReleaseFinding`.
- [ ] **AC3 (Portable `fail_on_output` & Go Format Fix)**: `LayerConfig` udvides med `fail_on_output: bool`. Hvis `true`, fejler laget hvis stdout/stderr er ikke-tom (bruges af `gofmt -l .` på tværs af Linux, macOS og Windows).
- [ ] **AC4 (Genopretning af Stack Profiler)**: I `default_config_for_stack` sættes `format`, `lint` og `types` til `optional: true` for zero-config eksterne projekter. Dækkes af unit tests pr. profil.
- [ ] **AC5 (Strict Evidence & Drift Gate i check-release)**: `xgauntlet check-release --strict` kræver en gyldig, uforfalsket `verification-report.json` med nul drift på tværs af source, policy, config og task digests.
- [ ] **AC6 (Fatal WASM Build & Fejlsikker Recovery i Bump)**: Fejler WASM-kompileringen under `release bump`, afbrydes processen fatalt med exit 1 og instruktion i at fortryde manifestændringer med `git checkout -- <files>`. `serde_json` konfigureres med `preserve_order` og regressionstest for nøglerækkefølge.
- [ ] **AC7 (Gated Release Pipeline med Genbrugeligt CI-workflow)**:
  - `.github/workflows/ci.yml` udvides med `workflow_call:` trigger.
  - `.github/workflows/release.yml` etablerer et indledende gate-job (`gate-check-release`), der eksekverer `xgauntlet check-release --strict` OG kalder det genbrugelige CI-workflow via `uses: ./.github/workflows/ci.yml`.
  - Hverken `build-binaries`, `create-github-release` eller `publish-npm` kan starte eller publicere noget, hvis gate-tjekket eller CI fejler (`needs: [gate-check-release]`).
  - Validerer at `${GITHUB_REF_NAME#v}` matcher `package.json`.
- [ ] **AC8 (Bootstrapper Fail-Closed Exit-kode)**: `packages/cli/bin/xgauntlet.js` returnerer en ikke-nul exit-kode (exit 1), hvis den native engine ikke findes under `status` eller `doctor`. Dækkes af test, der verificerer både stdout og exit-kode.
- [ ] **AC9 (CLI `--task-id` Alias & Konsistens)**: Clap `#[arg(short, long, alias = "task-id")]` tilføjes til `task`-argumentet i `Verify` (samt `CheckSpec` og `Handoff` for ensartethed), så README's dokumenterede kommando `xgauntlet verify --task-id <id>` virker direkte. Dækkes af regressionstest.
- [ ] **AC10 (Dokumentation & Gauntlet Sanering)**: CHANGELOG.md rettes for forkerte nøgler under 0.8.0, 0.8.2-teksten korrigeres for evidens-påstande, `gauntlet.toml` clippy tilføjes `--all-targets`, og `invariants`-laget saneres.

## 🚫 Must NOT
- Må IKKE indføre fallbacks i `verify`, der binder evidens til afsluttede opgaver eller tillader opgaveløse commits at passere som grønne (ADR 0005).
- Må IKKE benytte shell-afhængige kommandoer (`sh -c`, `test -z`) i standardprofiler, der bryder Windows-kompatibilitet.
- Må IKKE udføre automatiske netværkskald via `npx` under gauntlet-kørsler i standardprofiler.
- Må IKKE tillade udgivelse af hverken GitHub Release eller NPM-pakke, før `check-release --strict` og fuld CI er bestået.
- Må IKKE foretage remote `git push` eller destruktive git-handlinger (`git reset --hard`).

## 📦 Out of Scope
- **Task 037**: Policy engine kommandotokenisering og obligatorisk task-scope håndhævelse.
- **Task 038**: Antigravity PreInvocation schema-validering og ikke-destruktiv hook-merge.
- **Task 039**: Sigstore/in-toto attestation for native binærfiler i CI og kryptografisk verificering i bootstrapperen.
- **L4 (Generic Release Artifact Paths)**: Abstraktion af xgauntlet-specifikke stier til `[release] artifacts` i `gauntlet.toml` udskydes til en fremtidig arkitekturopgave.
- **ADR 0009**: Eventuel fremtidig obligatorisk hærdning af standardprofiler håndteres i en særskilt ADR-proces med overgangsordning.

## 📝 Revisions
- 2026-10-17: Oprettet opgavepakke til v0.8.3 patch efter to-akset review, re-review og ChatGPT second-opinion sparring.

## 🧪 Verifikation
- `cargo test -p xgauntlet-core --test release_engine_test`
- `cargo test -p xgauntlet-core --test gauntlet_execution_test`
- `cargo test -p xgauntlet-core --test config_engine_test`
- `cargo test -p xgauntlet-cli`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo run -p xgauntlet-cli -- check-release --strict`
- `cargo run -p xgauntlet-cli -- check-evidence`
- `npm test --workspaces`
