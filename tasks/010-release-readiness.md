---
type: Task Package
title: "Task 010: Release Readiness Gatekeeper & Manifest Harmony Engine"
description: "Etablere features/release i crates/xgauntlet-core samt CLI subcommand 'check-release' med manifest versionsharmoni, CHANGELOG.md validering og ADR-dækningskontrol jf. spec.md, CONTEXT.md og README.md"
status: active
generated: { by: process:xgauntlet-task-init, at: "2026-09-06T18:22:00Z" }
tags: [release, readiness, manifest, changelog, adr, cli, rust, adr-0001, adr-0003]
---

# Task 010: Release Readiness Gatekeeper & Manifest Harmony Engine

**Status**: `DONE`
**Intent**: `🚀 NEW FEATURE`
**Oprettet**: `2026-09-06`

## 🎯 Formål
Etablere en dedikeret, mekanisk release-gate (`features/release`) i `crates/xgauntlet-core` samt CLI subcommand `check-release` i `crates/xgauntlet-cli` jf. [spec.md](spec.md), [CONTEXT.md](CONTEXT.md), [README.md](README.md) og [ADR 0001](docs/adr/0001-package-by-feature-architecture.md):
1. **Versions-Harmoni**:
   - Automatisk udtræk af deklarerede versioner på tværs af projektets manifests:
     - `Cargo.toml` (root workspace og crates i `crates/*/Cargo.toml`, med fuld understøttelse af `version.workspace = true` arv).
     - `package.json` (root og underpakker i `packages/*/package.json`).
     - `pyproject.toml` (hvis til stede).
   - Validering af at alle deklarerede versioner stemmer 100% overens; udløsning af struktureret `CONFIG_VERSION_MISMATCH` ved diskrepans.
2. **Changelog-Synkronisering**:
   - Parsing af `CHANGELOG.md` jf. Keep a Changelog specifikationen.
   - Verifikation af at den deklarerede release-version har en tilsvarende sektion (`## [X.Y.Z] - YYYY-MM-DD`).
   - Håndtering af manglende `CHANGELOG.md` (`MISSING_CHANGELOG`) samt mismatch (`CHANGELOG_VERSION_MISMATCH`).
   - Understøttelse af `--allow-unreleased` flag, som tillader en `## [Unreleased]` sektion i advisory-tilstand.
3. **ADR-Dækningskontrol (Arkitekturintegritet)**:
   - Scanning af samtlige Architecture Decision Records i `docs/adr/*.md` (undtagen `README.md` og `index.md`).
   - Verifikation af at hver ADR er refereret i `README.md` eller `spec.md` (via filnavn, relativ sti, ADR-nummer som `ADR 0001` / `ADR 1` eller overskrift-stem).
   - Udløsning af `UNREFERENCED_ADR` diagnostik med clickable fil-links og konkrete remediation hints.
4. **Højtydende & Fasedelt Engine**:
   - Sub-5ms koldstarts-evaluering uden dæmoner eller vedvarende processer.
   - Fasedelt drift: Kører som en særskilt præ-publicerings-gate uden at forsinke den daglige TDD-udviklingsloop i `verify`.
5. **CLI & JSON Integration**:
   - `xgauntlet check-release` subcommand med farvekodet, kategoriseret terminalvisning samt maskinlæsbart `--json` output.
   - Exit code 0 hvis release-klar, exit code 1 hvis blokeret eller hvis advarsler findes i `--strict` mode.
6. **Kanonisk CHANGELOG.md**:
   - Etablering af rodfæstet `CHANGELOG.md` for xGauntlet med `## [0.1.0] - 2026-09-06`.

## 📋 Acceptance Criteria
- [x] `crates/xgauntlet-core/src/features/release/models.rs` definerer `ReleaseFindingCategory`, `ReleaseFinding`, `ReleaseReadinessOptions`, `ReleaseReadinessReport` og `ReleaseError`.
- [x] `crates/xgauntlet-core/src/features/release/manifests.rs` implementerer versionsekstraktion for `Cargo.toml` (med `version.workspace = true` arv), `package.json` og `pyproject.toml`, og detekterer versionsmismatches med præcise kildeangivelser.
- [x] `crates/xgauntlet-core/src/features/release/changelog.rs` parser `CHANGELOG.md` headerformater og validerer release-version med understøttelse af `--allow-unreleased`.
- [x] `crates/xgauntlet-core/src/features/release/adr.rs` scanner `docs/adr/` og verificerer at samtlige ADR-filer er refereret i `README.md` eller `spec.md`.
- [x] `crates/xgauntlet-core/src/features/release/engine.rs` orkestrerer samtlige delkontroller, beregner `is_ready` og samlet eksekveringstid, og overholder sub-5ms koldstarts-invarianten.
- [x] `xgauntlet check-release` subcommand er tilgængelig i `crates/xgauntlet-cli` med understøttelse af `--workspace`, `--allow-unreleased`, `--strict` og `--json`.
- [x] Rodfæstet `CHANGELOG.md` findes i workspacet med Keep a Changelog format for version 0.1.0.
- [x] `crates/xgauntlet-core/tests/release_engine_test.rs` verificerer alle aspekter (harmoniske versioner, version mismatch, manglende changelog, changelog mismatch, allow-unreleased, urefererede ADRs, mock- og virkeligt workspace samt CLI integration).
- [x] `check-spec` validerer `tasks/010-release-readiness.md` med 0 fejl.
- [x] 100% grøn testsuite på tværs af hele workspacet (`cargo test --workspace` og `npm test --prefix packages/cli`).

## 🚫 Must NOT
- Må IKKE introducere baggrunds-dæmoner eller service-managere (Zero-Daemon invariant).
- Må IKKE ændre kildekode- eller konfigurationsfiler automatisk under release-checket (skal være 100% read-only).
- Må IKKE fejle hvis valgfri økosystemfiler ikke eksisterer i et projekt (f.eks. må den ikke kræve `pyproject.toml` i et Rust-projekt).
- Må IKKE foretage remote git publication handlinger (`git push`).
- Må IKKE forringe sub-3ms koldstarten for CLI eller core engine.

## 📝 Revisions
- 2026-09-06: Oprettet som ACTIVE task for Release Readiness Gatekeeper & Manifest Harmony Engine (Task 010).
- 2026-09-06: Samtlige 10 acceptkriterier opfyldt og verificeret med 100% grøn testsuite. Markerest DONE.

## 🧪 Verifikation
- `cargo test -p xgauntlet-core --test release_engine_test`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo fmt --check`
- `cargo run -p xgauntlet-cli -- check-spec -t 010-release-readiness`
- `cargo run -p xgauntlet-cli -- check-release`
- `cargo run -p xgauntlet-cli -- check-release --json`
- `npm test --prefix packages/cli`
