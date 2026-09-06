---
type: Task Package
title: "Task 007: Declarative Config & Stack Profiles"
description: "Etablere features/config i crates/xgauntlet-core med en nul-afhængigheds gauntlet.toml loader, deklarativ skemavalidering, multi-stack profiler (Rust, Python, Node/TS, Go) samt fuld bagudkompatibel integration med gauntlet pipelinen"
status: active
generated: { by: process:xgauntlet-task-init, at: "2026-09-06T17:55:00Z" }
tags: [config, toml, json, validation, stack-profiles, rust, python, node, go, adr-0001]
---

# Task 007: Declarative Config & Stack Profiles

**Status**: `DONE`
**Intent**: `🚀 NEW FEATURE`
**Oprettet**: `2026-09-06`

## 🎯 Formål
Etablere en dedikeret og autonom konfigurationsfeature (`features/config`) i `crates/xgauntlet-core` jf. [spec.md](spec.md), [CONTEXT.md](CONTEXT.md) samt [ADR 0001](docs/adr/0001-package-by-feature-architecture.md):
1. Etablere domænemodeller i `features/config/models.rs`:
   - `GauntletConfig`, `PathsConfig`, `LayerConfig`, `StackProfile`, `ConfigValidationReport`, `ConfigValidationIssue`, `ValidationSeverity` og `ConfigError`.
2. Etablere multi-stack profiler i `features/config/profiles.rs`:
   - Prædefinerede profiler for `rust`, `python`, `node`/`typescript` og `go` med standardlag for linters, types, tests, invarianter og mutation testing.
   - Automatisk stack-detektering baseret på workspace-indikatorer (`Cargo.toml`, `pyproject.toml`, `package.json`, `go.mod`).
   - Fleksibel sammenfletning og lag-overriding mod standardprofiler.
3. Etablere nul-afhængigheds loader og serializer i `features/config/loader.rs`:
   - Højtydende, streaming `gauntlet.toml` parser uden eksterne crates (bevarer sub-3ms koldstart og zero-daemon princippet).
   - Parser for `gauntlet.json` via `serde_json`.
   - Workspace-scanning og automatisk fallback til detekterede stack-profiler.
   - Kanonisk TOML rendering (`render_toml`) for eksport og init-scaffolding.
4. Etablere deklarativ valideringsmotor i `features/config/validation.rs`:
   - Unikke lagnavne (afviser duplikerede lag).
   - Validering af lagkommandoer (må ikke være tomme).
   - Bounded positiv timeoutkontrol (`0.0 < timeout_seconds <= 3600.0`).
   - Stisikkerhed og containment: afviser path traversal (`..`) og stier, der undslipper workspace-roden.
   - Strukturerede diagnostiske fund (`ConfigValidationIssue`) med udbedringsforslag.
5. Sikre fuld bagudkompatibilitet og Package-by-Feature oprydning:
   - Re-eksport fra `crates/xgauntlet-core/src/features/config/mod.rs` og `src/lib.rs`.
   - `features/gauntlet/config.rs` delegerer til `features/config` uden duplikering.
6. Gennemføre en komplet testsuite i `crates/xgauntlet-core/tests/config_engine_test.rs`.

## 📋 Acceptance Criteria
- [x] `crates/xgauntlet-core/src/features/config/models.rs` definerer `GauntletConfig`, `PathsConfig`, `LayerConfig`, `StackProfile`, `ConfigValidationReport`, `ConfigValidationIssue`, `ValidationSeverity` og `ConfigError`.
- [x] `crates/xgauntlet-core/src/features/config/profiles.rs` implementerer registrerede stack-profiler (`rust`, `python`, `node`/`typescript`, `go`) samt automatisk workspace stack-detektion.
- [x] `crates/xgauntlet-core/src/features/config/loader.rs` parser `gauntlet.toml` uden eksterne parser-afhængigheder og understøtter top-level nøgler, `[paths]`, gentagne `[[layers]]`, kommentarer og strenge.
- [x] `crates/xgauntlet-core/src/features/config/loader.rs` understøtter `gauntlet.json` og rendering til kanonisk TOML format.
- [x] `crates/xgauntlet-core/src/features/config/validation.rs` fanger duplikerede lagnavne, tomme kommandoer, ugyldige timeouts og path traversal forsøg.
- [x] `crates/xgauntlet-core/src/features/gauntlet/` genbruger og delegerer til `features/config` med fuld bagudkompatibilitet for eksisterende kode.
- [x] `crates/xgauntlet-core/tests/config_engine_test.rs` verificerer indlæsning, validering, profiler, serialisering og fejlhåndtering.
- [x] `check-spec` validerer `tasks/007-declarative-config.md` med 0 fejl.
- [x] 100% grøn testsuite på tværs af hele workspacet (`cargo test --workspace`).

## 🚫 Must NOT
- Må IKKE introducere tunge eksterne parsing crates, der forringer sub-3ms koldstarts-invarianten.
- Må IKKE tillade path traversal (`..`) eller stier uden for workspace-roden i konfigurationen.
- Må IKKE fejle åbent (fail-open) ved ugyldige eller modstridende lagdefinitioner.
- Må IKKE bryde eksisterende interfaces eller tests i `features/gauntlet`.

## 📝 Revisions
- 2026-09-06: Oprettet som ACTIVE task for Declarative Config & Stack Profiles (Task 007).
- 2026-09-06: Alle 9 acceptkriterier opfyldt og verificeret med 100% grøn testsuite. Markerest DONE.

## 🧪 Verifikation
- `cargo test -p xgauntlet-core --test config_engine_test`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo fmt --check`
- `cargo run -p xgauntlet-cli -- check-spec -t 007-declarative-config`
- `cargo run -p xgauntlet-cli -- check-config`
- `cargo run -p xgauntlet-cli -- verify --task 007-declarative-config --save`
