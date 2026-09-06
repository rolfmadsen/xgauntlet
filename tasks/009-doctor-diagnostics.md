---
type: Task Package
title: "Task 009: Fast Environment, Git, and Toolchain Diagnostics Engine"
description: "Etablere features/doctor i crates/xgauntlet-core samt CLI subcommand 'doctor' med hurtig inspektion af host-miljø, Git-tilstand, in-repo styringsfiler, detekterede stakke og toolchain binærer uden dæmoner eller sideeffekter jf. spec.md, CONTEXT.md og README.md"
status: active
generated: { by: process:xgauntlet-task-init, at: "2026-09-06T18:12:00Z" }
tags: [doctor, diagnostics, environment, git, toolchains, governance, cli, rust, adr-0001]
---

# Task 009: Fast Environment, Git, and Toolchain Diagnostics Engine

**Status**: `DONE`
**Intent**: `🚀 NEW FEATURE`
**Oprettet**: `2026-09-06`

## 🎯 Formål
Etablere en dedikeret, højtydende diagnostikmotor (`features/doctor`) i `crates/xgauntlet-core` samt opgradere CLI subcommand `doctor` i `crates/xgauntlet-cli` jf. [spec.md](spec.md), [CONTEXT.md](CONTEXT.md), [README.md](README.md) og [ADR 0001](docs/adr/0001-package-by-feature-architecture.md):
1. Etablere domænemodeller i `features/doctor/models.rs`:
   - `DoctorCategory` (`Host`, `Git`, `Governance`, `Toolchains`, `Engine`).
   - `DoctorCheckStatus` (`Pass`, `Warn`, `Fail`, `Info`).
   - `DoctorCheckItem` (navn, kategori, status, besked, valgfri detalje, valgfrit remediation hint, duration_ms).
   - `DoctorVerdict` (`Healthy`, `Degraded`, `Critical`).
   - `DoctorOptions` (workspace, verbose, strict, valgfri kategori-filtrering).
   - `DoctorReport` (aggregerede fund, tællere, platform-metadata, total varighed og domæneverdikt).
   - `DoctorError` (struktureret fejlhierarki via `thiserror`).
2. Etablere modulære inspektionsmoduler i `features/doctor/checks/`:
   - `host`: Værtsplatform, OS, CPU-arkitektur, core engine version samt validering af temp-katalog adgang.
   - `git`: Git binær tilgængelighed og version, workspace Git-rod, brugeridentitet (`user.name`, `user.email`), working tree tilstand (ucommittede/uovervågede filer) samt håndhævelse af `push.followTags` konfiguration jf. ADR 0003.
   - `governance`: Integritetscheck af in-repo styringsfiler (`gauntlet.toml`, `spec.md`, `CONTEXT.md`, `CODING_STANDARDS.md`, `tasks/`, `docs/adr/`, `.agents/AGENTS.md`, `.agents/hooks.json`), inklusiv konfigurationsvalidering via `validate_config` og spec-validering.
   - `toolchains`: Auto-detektion af workspace stakke (`rust`, `python`, `node`, `go`) via `detect_stack`, verifikation af obligatoriske og anbefalede værktøjer for hver stak (f.eks. `cargo`, `rustc`, `clippy`, `rustfmt`, `node`, `npm`/`pnpm`, `python3`, `go`), deres versioner og PATH-opløsning.
   - `engine`: Wasmtime runtime initialisering og indlejret WebAssembly policy-komponent beredskab (`WasmRuntimeHost`).
3. Etablere diagnostikmotor i `features/doctor/engine.rs`:
   - `run_doctor(options: &DoctorOptions) -> Result<DoctorReport, DoctorError>`.
   - Hurtig, sub-50ms koldstarts-inspektion uden dæmoner eller vedvarende processer.
   - Strukturerede remediation-anbefalinger for advarsler og fejl (f.eks. `xgauntlet init`, `git config push.followTags true`, toolchain installationer).
   - Aggregering af beståede, advarsels-, fejl- og info-tællere samt beregning af samlet `DoctorVerdict` (`Healthy`, `Degraded`, `Critical`).
4. Etablere fuld CLI-understøttelse for `doctor` i `crates/xgauntlet-cli/src/main.rs`:
   - Understøttelse af flag: `--workspace`, `--json`, `--verbose`, `--strict` og `--category`.
   - Menneskevenlig terminal summary med kategoriseret, farvet visning og handlingsanvisende hints.
   - Maskinlæsbart JSON output ved `--json`.
   - Exit code 1 hvis `--strict` og advarsler findes, eller hvis der opstår kritiske fejl.
5. Gennemføre komplet testsuite i `crates/xgauntlet-core/tests/doctor_engine_test.rs`.

## 📋 Acceptance Criteria
- [x] `crates/xgauntlet-core/src/features/doctor/models.rs` definerer `DoctorCategory`, `DoctorCheckStatus`, `DoctorCheckItem`, `DoctorVerdict`, `DoctorOptions`, `DoctorReport` og `DoctorError`.
- [x] `crates/xgauntlet-core/src/features/doctor/checks/` implementerer specialiserede checks for `host`, `git`, `governance`, `toolchains` og `engine`.
- [x] Git-diagnostik verificerer Git-binær, repo-rod, brugerkonfiguration og `push.followTags` jf. ADR 0003.
- [x] Governance-diagnostik validerer `gauntlet.toml`, `spec.md`, `CONTEXT.md`, `tasks/` og `.agents/` konfiguration med konkrete udbedringsforslag.
- [x] Toolchain-diagnostik tilpasser sig automatisk den detekterede stack (`rust`, `python`, `node`, `go`) og rapporterer tilgængelige værktøjer samt versioner.
- [x] Engine-diagnostik verificerer Wasmtime host og embedded zero-ambient-authority policy engine.
- [x] `run_doctor` koordinerer alle checks, overholder sub-50ms koldstart, og beregner `DoctorVerdict` baseret på fund og `strict`-flag.
- [x] `xgauntlet doctor` CLI subcommand understøtter både farvet menneskevenlig visning og maskinlæsbar `--json` eksport samt `--strict` og `--category`.
- [x] `crates/xgauntlet-core/tests/doctor_engine_test.rs` verificerer modeller, individuelle checks, workspace-kørsel, fejlsituationer, strict mode og CLI-flow.
- [x] `check-spec` validerer `tasks/009-doctor-diagnostics.md` med 0 fejl.
- [x] 100% grøn testsuite på tværs af hele workspacet (`cargo test --workspace`).

## 🚫 Must NOT
- Må IKKE introducere baggrunds-dæmoner eller service-managere (Zero-Daemon invariant).
- Må IKKE ændre eller mutere filer på disken under diagnostik-kørslen (skal være 100% read-only og sideeffektfri).
- Må IKKE crashe eller gå i panik ved manglende eksterne værktøjer eller delvist korrupte miljøer (fail-safe diagnostik).
- Må IKKE forringe sub-3ms koldstarts-invarianten for CLI eller basis check-operationer.

## 📝 Revisions
- 2026-09-06: Oprettet som ACTIVE task for Fast Environment, Git, and Toolchain Diagnostics Engine (Task 009).
- 2026-09-06: Samtlige 11 acceptkriterier opfyldt og verificeret med 100% grøn testsuite. Markerest DONE.

## 🧪 Verifikation
- `cargo test -p xgauntlet-core --test doctor_engine_test`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo fmt --check`
- `cargo run -p xgauntlet-cli -- check-spec -t 009-doctor-diagnostics`
- `cargo run -p xgauntlet-cli -- doctor`
- `cargo run -p xgauntlet-cli -- doctor --json`
- `npm test --prefix packages/cli`
