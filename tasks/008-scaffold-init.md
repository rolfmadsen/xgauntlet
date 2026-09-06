---
type: Task Package
title: "Task 008: Safe Non-Destructive Project Bootstrap Engine"
description: "Etablere features/scaffold i crates/xgauntlet-core samt CLI subcommand 'init' med ikke-destruktiv garanti, skabeloner for understøttede stack-profiler og fuld integration mod spec.md og CONTEXT.md"
status: active
generated: { by: process:xgauntlet-task-init, at: "2026-09-06T18:05:00Z" }
tags: [scaffold, init, non-destructive, templates, cli, rust, python, node, go, adr-0001]
---

# Task 008: Safe Non-Destructive Project Bootstrap Engine

**Status**: `DONE`
**Intent**: `🚀 NEW FEATURE`
**Oprettet**: `2026-09-06`

## 🎯 Formål
Etablere en dedikeret, ikke-destruktiv projekt-initieringsmotor (`features/scaffold`) i `crates/xgauntlet-core` samt CLI subcommand `init` i `crates/xgauntlet-cli` jf. [spec.md](spec.md), [CONTEXT.md](CONTEXT.md), [README.md](README.md) og [ADR 0001](docs/adr/0001-package-by-feature-architecture.md):
1. Etablere domænemodeller i `features/scaffold/models.rs`:
   - `ScaffoldOptions`, `ScaffoldAction`, `ScaffoldFileReport`, `ScaffoldResult` og `ScaffoldError`.
2. Etablere skabelon-generering i `features/scaffold/templates.rs`:
   - Samtlige 9 kanoniske in-repo styringsfiler jf. README:
     1. `gauntlet.toml`: Genereret via `default_config_for_stack(stack).render_toml()`
     2. `CONTEXT.md`: Aristoteles-glossary med domænebegreber og `_Avoid_:` klausuler
     3. `CODING_STANDARDS.md`: Multi-stack kodestandarder og håndværksmæssige invarianter
     4. `spec.md`: OKF makrospecifikation med systeminvarianter og verifikationskontrakter
     5. `tasks/001-bootstrap.md`: OKF opgavepakke (`ACTIVE`, `🚀 NEW FEATURE`, acceptkriterier, `Must NOT`)
     6. `docs/adr/0001-package-by-feature-architecture.md`: Initial ADR skabelon
     7. `.agents/AGENTS.md`: Agent guidelines med Task HUD, skills og loop
     8. `.agents/hooks.json`: PreToolUse hook konfiguration til `xgauntlet hook antigravity`
     9. `CLAUDE.md`: Claude Code retningslinjer og sikkerhedsinvarianter
3. Etablere ikke-destruktiv init-motor i `features/scaffold/engine.rs`:
   - **Ikke-destruktiv garanti (Safety First)**: Overskriver ALDRIG eksisterende filer i projektet uden `--force`.
   - Auto-detektion af stack-profil baseret på workspace-indikatorer via `detect_stack`.
   - Eksplicit stack-valg via `--stack` (`rust`, `python`, `node`, `go`).
   - Forhåndsvisning via `--dry-run` uden at skrive filer til disken.
   - Detaljeret fil-handling rapportering (`Created`, `Skipped`, `Overwritten`, `WouldCreate`, `WouldSkip`, `WouldOverwrite`).
4. Etablere CLI subcommand `init` i `crates/xgauntlet-cli/src/main.rs`:
   - Understøttelse af flag: `--workspace`, `--stack`, `--force`, `--dry-run`, `--name` og `--json`.
   - Menneskevenlig terminal summary med farvede action tags samt maskinlæsbart JSON output.
5. Gennemføre komplet testsuite i `crates/xgauntlet-core/tests/scaffold_engine_test.rs`.

## 📋 Acceptance Criteria
- [x] `crates/xgauntlet-core/src/features/scaffold/models.rs` definerer `ScaffoldOptions`, `ScaffoldAction`, `ScaffoldFileReport`, `ScaffoldResult` og `ScaffoldError`.
- [x] `crates/xgauntlet-core/src/features/scaffold/templates.rs` genererer alle 9 in-repo styringsfiler tilpasset den valgte eller detekterede stack.
- [x] Genererede skabeloner for `CONTEXT.md` og `tasks/001-bootstrap.md` valideres med `check-spec` og giver 0 fejl.
- [x] Genereret `gauntlet.toml` valideres med `validate_config` og giver 0 fejl.
- [x] `crates/xgauntlet-core/src/features/scaffold/engine.rs` håndhæver den ikke-destruktive garanti: eksisterende filer markeres som `Skipped` og røres ikke, medmindre `force: true` er sat.
- [x] `dry_run: true` rapporterer planlagte handlinger (`WouldCreate`, `WouldSkip`, `WouldOverwrite`) uden at skrive eller ændre nogen filer.
- [x] `xgauntlet init` subcommand er tilgængelig i `crates/xgauntlet-cli` og understøtter både menneskevenligt output og `--json`.
- [x] `crates/xgauntlet-core/tests/scaffold_engine_test.rs` verificerer alle kerneaspekter (ikke-destruktiv sikkerhed, force, dry-run, stack-profiler, skabelon-validering og CLI-flow).
- [x] `check-spec` validerer `tasks/008-scaffold-init.md` med 0 fejl.
- [x] 100% grøn testsuite på tværs af hele workspacet (`cargo test --workspace`).

## 🚫 Must NOT
- Må IKKE overskrive eksisterende filer i projektet uden eksplicit `--force` flag.
- Må IKKE fejle åbent (fail-open) eller efterlade inkonsistente partielle tilstande uden rapportering.
- Må IKKE tillade path traversal (`..`) uden for workspace-roden.
- Må IKKE forringe sub-3ms koldstarts-invarianten for CLI eller core engine.

## 📝 Revisions
- 2026-09-06: Oprettet som ACTIVE task for Safe Non-Destructive Project Bootstrap Engine (Task 008).
- 2026-09-06: Samtlige 10 acceptkriterier opfyldt og verificeret med 100% grøn testsuite. Markerest DONE.

## 🧪 Verifikation
- `cargo test -p xgauntlet-core --test scaffold_engine_test`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo fmt --check`
- `cargo run -p xgauntlet-cli -- check-spec -t 008-scaffold-init`
- `cargo run -p xgauntlet-cli -- check-config`
- `cargo run -p xgauntlet-cli -- verify --task 008-scaffold-init --save`
