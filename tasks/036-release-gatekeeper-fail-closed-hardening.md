---
type: Task Package
title: "Task 036: Release Gatekeeper Fail-Closed Hardening and Evidence Integrity"
description: "Fail-closed hærdning af check-release gater, to-trins genbrugeligt CI release-workflow, transaktionel WASM bump, fail_on_output til Go-format, genopretning af standardprofiler, bootstrapper exit-kode, CLI-alias og dokumentationsoprydning"
status: active
generated: { by: process:xgauntlet-task-init, at: "2026-10-17T11:20:00Z" }
tags: [release, gatekeeper, fail-closed, wasm, evidence, drift, profiles, ci, bootstrapper, adr-0005]
scope:
  - "crates/xgauntlet-core/src/features/release/**"
  - "crates/xgauntlet-core/src/features/config/**"
  - "crates/xgauntlet-core/src/features/gauntlet/**"
  - "crates/xgauntlet-cli/src/**"
  - "packages/cli/bin/xgauntlet.js"
  - "packages/cli/test/**"
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
  - "packages/cli/test/launcher.test.js"
---

# Task 036: Release Gatekeeper Fail-Closed Hardening and Evidence Integrity

**Status**: `ACTIVE`  
**Intent**: `🐛 BUG FIX`  
**Oprettet**: `2026-10-17`  

## 🎯 Formål
Eliminere alle "silent passes" og falske positiver i release-gatekeeperen (`check-release`), sikre at obligatoriske kontroller der ikke kan gennemføres altid fejler lukket (exit 1), etablere en reel forudgående release-gate i GitHub Actions via adskilte jobs (`ci` via `workflow_call` og `gate-check-release`), fastlægge rapportens livscyklus så release-gaten validerer det committede evidensdokument mod den aktuelle checkout, gøre versions- og WASM-opdatering transaktionel med atomisk staging og automatisk rollback ved fejl, implementere platformsuafhængig `fail_on_output` til `gofmt`, rulle breaking stack-profiler tilbage til sikre defaults, sikre ikke-nul exit-kode i bootstrapperen ved manglende engine, tilføje `--task-id` som officielt CLI-alias, og rette uoverensstemmelser i CHANGELOG.md.

## 📋 Acceptance Criteria
- [ ] **AC1 (Fail-Closed Format Gate i check-release)**: `check-release` evaluerer formatering via gauntlettens layer-runner med gældende `timeout_seconds` for alle konfigurerede, obligatoriske formatlag (herunder xGauntlets egen `optional = false` konfiguration). Manglende format-binary, spawn-fejl eller formateringsfejl registreres som blokerende `ReleaseFinding` og giver exit 1.
- [ ] **AC2 (Fail-Closed WASM Policy Gate)**: `check-release` validerer `gauntlet_policy.wasm` deterministisk uden slugte fejl. Korrupt bytecode, I/O-fejl eller versionsmismatch registreres som blokerende `ReleaseFinding` og giver exit 1.
- [ ] **AC3 (Portable `fail_on_output` & Go Format Fix)**: `LayerConfig` udvides med `fail_on_output: bool`. Hvis `true`, fejler laget hvis stdout/stderr er ikke-tom (bruges af `gofmt -l .` på tværs af Linux, macOS og Windows).
- [ ] **AC4 (Genopretning af Stack Profiler for Eksterne Projekter)**: I `default_config_for_stack` sættes `format`, `lint` og `types` til `optional: true` for zero-config eksterne projekter, så udrullede projekter ikke knækker hvis et værktøj mangler under `verify` (giver `PARTIAL` i stedet for `FAILED`). Dækkes af unit tests pr. profil.
- [ ] **AC5 (Strict Evidence & Drift Gate i check-release med defineret livscyklus)**:
  - `verification-report.json` er et forseglet evidensdokument i repository-roden, forpligtet i Git forud for release-tagget.
  - `xgauntlet check-release --strict` kontrollerer, at rapporten findes, at dens JSON-skema er gyldigt, at dens registrerede digests (source, policy, config og task) matcher den aktuelle checkout med 0 drift, og at dens overordnede verdict er godkendt (`PASSED` eller `PARTIAL` uden obligatoriske fejl).
  - En manglende, forældet eller drifted rapport blokerer release fatalt med exit 1.
- [ ] **AC6 (Transaktionel WASM & Manifest Bump med Atomisk Staging)**:
  - `release bump` udfører preflight-validering af WASM-byggekæden (`cargo check -p gauntlet-policy-engine --target wasm32-unknown-unknown`) før nogen filer røres.
  - Planlagte manifest- og changelog-ændringer beregnes in-memory. Oprindelige filindhold gemmes som transaktionsbackup, før skrivning påbegyndes.
  - Fejler WASM-kompileringen, rulles eventuelle påbegyndte ændringer automatisk tilbage fra backup før afslutning, så arbejdstræet efterlades 100% uændret uden brug af `git checkout` eller destruktive git-handlinger. Processen afsluttes fatalt med exit 1 og klar diagnostik.
  - `serde_json` konfigureres med `preserve_order` og regressionstest for nøglerækkefølge i `package.json`.
- [ ] **AC7 (Gated Release Workflow med Adskilte CI- og Gate-jobs)**:
  - `.github/workflows/ci.yml` udvides med `workflow_call:` trigger.
  - `.github/workflows/release.yml` opdeles i en eksplicit job-DAG:
    1. Job `ci`: kalder det genbrugelige workflow via `uses: ./.github/workflows/ci.yml`.
    2. Job `gate-check-release`: har `needs: ci`, tjekker koden ud på `ubuntu-latest`, bygger CLI-motoren (`cargo build -p xgauntlet-cli`) og eksekverer `cargo run -p xgauntlet-cli -- check-release --strict` (der validerer den committede evidensrapport og 0 drift).
    3. Job `build-binaries`: har `needs: gate-check-release`.
    4. Job `create-github-release`: har `needs: [gate-check-release, build-binaries]`.
    5. Job `publish-npm`: har `needs: [gate-check-release, create-github-release]`.
  - Hverken GitHub Release eller NPM-publicering kan køre, hvis enten CI eller `check-release --strict` fejler.
  - Validerer at `${GITHUB_REF_NAME#v}` matcher `package.json`.
- [ ] **AC8 (Bootstrapper Fail-Closed Exit-kode)**: `packages/cli/bin/xgauntlet.js` returnerer en ikke-nul exit-kode (exit 1), hvis den native engine ikke findes under `status` eller `doctor`. Dækkes af regressionstest i `packages/cli/test/launcher.test.js`, der verificerer både stdout og exit-kode.
- [ ] **AC9 (CLI `--task-id` Alias & Konsistens)**: Clap `#[arg(short, long, alias = "task-id")]` tilføjes til `task`-argumentet i `Verify` (samt `CheckSpec` og `Handoff` for ensartethed), så README's dokumenterede kommando `xgauntlet verify --task-id <id>` virker direkte. Dækkes af regressionstest.
- [ ] **AC10 (Dokumentation & Gauntlet Sanering)**: CHANGELOG.md rettes for forkerte nøgler under 0.8.0 (`max_file_lines`, `max_test_file_lines`, `[[architecture.exemptions]]`), 0.8.2-teksten korrigeres for evidens-påstande med en note om, at v0.8.3 hærder Tier 1 lokal evidens- og driftkontrol, `gauntlet.toml` clippy tilføjes `--all-targets`, og `invariants`-laget saneres.

## 🚫 Must NOT
- Må IKKE indføre fallbacks i `verify`, der binder evidens til afsluttede opgaver eller tillader opgaveløse commits at passere som grønne (ADR 0005).
- Må IKKE benytte `git checkout -- <files>` eller destruktive git-handlinger (`git reset --hard`) til fejlhåndtering.
- Må IKKE kombinere et genbrugeligt workflow (`uses:`) med jobtrin (`steps:`) i samme GitHub Actions job.
- Må IKKE benytte shell-afhængige kommandoer (`sh -c`, `test -z`) i standardprofiler, der bryder Windows-kompatibilitet.
- Må IKKE udføre automatiske netværkskald via `npx` under gauntlet-kørsler i standardprofiler.
- Må IKKE tillade udgivelse af hverken GitHub Release eller NPM-pakke, før `check-release --strict` og fuld CI er bestået.

## 📦 Out of Scope
- **Task 037**: Policy engine kommandotokenisering og obligatorisk task-scope håndhævelse.
- **Task 038**: Antigravity PreInvocation schema-validering og ikke-destruktiv hook-merge.
- **Task 039**: Sigstore/in-toto attestation for native distributionsarkiver i CI og kryptografisk verificering i bootstrapperen (ADR 0005 Tier 2).
- **L4 (Generic Release Artifact Paths)**: Abstraktion af xgauntlet-specifikke stier til `[release] artifacts` i `gauntlet.toml` udskydes til en fremtidig arkitekturopgave.
- **ADR 0009**: Eventuel fremtidig obligatorisk hærdning af standardprofiler håndteres i en særskilt ADR-proces med overgangsordning.

## 📝 Revisions
- 2026-10-17: Oprettet opgavepakke til v0.8.3 patch efter to-akset review, re-review og iterative præciseringer af GitHub Actions workflow-DAG, transaktionel recovery, evidens-livscyklus og profil-afgrænsning.

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
