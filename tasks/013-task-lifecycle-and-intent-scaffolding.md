---
type: Task Package
title: "Task 013: Task Lifecycle Engine & Intent Scaffolding"
description: "Etablere features/tasks motorudvidelser samt CLI underkommandoer for 'task new', 'task status' og 'task list' med sekventiel autonummerering, kriterie-telemetri og JSON-eksport jf. spec.md og CONTEXT.md"
status: completed
generated: { by: process:xgauntlet-task-init, at: "2026-09-06T22:32:00Z" }
tags: [task-lifecycle, intent, scaffolding, cli, telemetry, json, rust, adr-0001]
---

# Task 013: Task Lifecycle Engine & Intent Scaffolding

**Status**: `DONE`
**Intent**: `🚀 NEW FEATURE`
**Oprettet**: `2026-09-06`

## 🎯 Formål
Etablere en dedikeret task lifecycle motor i `crates/xgauntlet-core/src/features/tasks` samt CLI-underkommandoer (`xgauntlet task new`, `xgauntlet task status`, `xgauntlet task list`) i `crates/xgauntlet-cli` jf. [spec.md](spec.md), [CONTEXT.md](CONTEXT.md) og [ADR 0001](docs/adr/0001-package-by-feature-architecture.md):
1. **`xgauntlet task new <name>` (Autonummerering & Intent Scaffolding)**:
   - Automatisk detektion af næste sekventielle opgavenummer i `tasks/` (f.eks. scanning af `tasks/*.md`, udtræk af højeste numeriske præfiks og generering af `tasks/014-<name>.md` uden manuelle fejl).
   - Generering af fuldgyldigt OKF v0.2 frontmatter med `type: Task Package`, timestamp og tags.
   - Etablering af den komplette opgavestruktur: `# Task <num>: <Title>`, `**Status**: ACTIVE`, `**Intent**: ...`, `## 🎯 Formål`, `## 📋 Acceptance Criteria` (inkl. konkrete `- [ ]` punkter), `## 🚫 Must NOT`, `## 📝 Revisions` og `## 🧪 Verifikation`.
   - Understøttelse af `--intent <feature|bug|refactor>` samt interaktiv prompt af formål hvis terminalen er interaktiv.
2. **`xgauntlet task status` (Kriterie- & Git Telemetri)**:
   - Automatisk identifikation af aktiv opgave i `tasks/` (eller via `--task <id>`).
   - Parsing af opgavens acceptkriterier med opgørelse af udførte (`- [x]`) vs udestående (`- [ ]`) punkter samt beregning af fremdriftsindikator (`[■■■□□] 60%`).
   - Indhentning af lokal Git-telemetri (aktuel branch, commit OID og antal dirty/staged filer) uden eksterne tunge biblioteker.
   - Understøttelse af `--json` flag til maskinel integration (f.eks. til dynamisk fremføring i agent Cockpit HUD eller IDE statuslinje).
3. **`xgauntlet task list` (Opgaveoverblik)**:
   - Tabellarisk terminal-oversigt over samtlige opgaver i `tasks/` med id, titel, status (`ACTIVE` / `DONE`) og kriterie-fremdrift.
   - Understøttelse af maskinlæsbar `--json` eksport.
4. **Højtydende & Fail-Closed**:
   - Sub-5ms koldstarts-evaluering og nul dæmoner jf. Zero-Daemon arkitekturen.
   - Sikker validering af skabelonens overensstemmelse med `check-spec`.

## 📋 Acceptance Criteria
- [x] `crates/xgauntlet-core/src/features/tasks/lifecycle.rs` implementerer opgave-scaffolding med automatisk nummerinkrementering baseret på eksisterende `tasks/` filer.
- [x] `TaskScaffolder` genererer en komplet opgavefil med gyldigt OKF v0.2 YAML frontmatter og samtlige standardsektioner (Formål, Acceptance Criteria, Must NOT, Revisions, Verifikation).
- [x] `crates/xgauntlet-core/src/features/tasks/telemetry.rs` implementerer `TaskTelemetry` model og inspektion, som parser `- [x]` vs `- [ ]` samt indsamler Git branch, HEAD OID og dirty file count.
- [x] `xgauntlet task new <name>` subcommand er tilgængelig i `crates/xgauntlet-cli` med understøttelse af `--intent`, `--title` og `--workspace`.
- [x] `xgauntlet task status` subcommand er tilgængelig i `crates/xgauntlet-cli` med terminal progress-bar visning og maskinlæsbar `--json` eksport.
- [x] `xgauntlet task list` subcommand er tilgængelig i `crates/xgauntlet-cli` med tabelvisning over samtlige opgaver og `--json` eksport.
- [x] `crates/xgauntlet-core/tests/task_lifecycle_test.rs` verificerer autonummerering, template-validering, kriterie-tælling og telemetri-beregning i et mock-workspace.
- [x] `cargo run -p xgauntlet-cli -- check-spec -t 013-task-lifecycle-and-intent-scaffolding` validerer med 0 fejl.
- [x] 100% grøn testsuite på tværs af hele workspacet (`cargo test --workspace`).

## 🚫 Must NOT
- Må IKKE overskrive eksisterende opgavefiler i `tasks/` uden eksplicit force-flag (skal fejle sikkert ved kollision).
- Må IKKE crashe eller fejle hvis workspacet ikke er et Git-repository (skal gracefully rapportere fallback-værdier for branch/dirty).
- Må IKKE foretage remote publication kommandoer (`git push`) jf. ADR 0003.
- Må IKKE introducere baggrunds-dæmoner eller runtime sockets jf. Zero-Daemon invarianten.

## 📝 Revisions
- 2026-09-06: Task oprettet som ACTIVE for Task Lifecycle Engine & Intent Scaffolding (Task 013).
- 2026-09-07: Implementering gennemført for TaskScaffolder, TaskTelemetry og CLI kommandoer (Task 013 markeret som DONE).

## 🧪 Verifikation
- `cargo test -p xgauntlet-core --test task_lifecycle_test`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo fmt --check`
- `cargo run -p xgauntlet-cli -- check-spec -t 013-task-lifecycle-and-intent-scaffolding`
- `cargo run -p xgauntlet-cli -- task list`
- `cargo run -p xgauntlet-cli -- task status`
- `cargo run -p xgauntlet-cli -- task status --json`
