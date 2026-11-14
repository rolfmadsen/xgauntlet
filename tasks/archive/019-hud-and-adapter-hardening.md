---
type: Task Package
title: "Task 019: HUD & Harness Adapter Hardening, OS Resilience & Refactor"
description: "Udbedre review-fund: boks-ellipsing/bredde, Windows-sti-normalisering, scaffold-fejlhåndtering, DRY JSON hook merging og lokalt hooks.json"
status: done
generated: { by: process:xgauntlet-task-init, at: "2026-09-09T20:24:02Z" }
tags: [task-lifecycle, intent, scaffolding, rust]
---

# Task 019: HUD & Harness Adapter Hardening, OS Resilience & Refactor

**Status**: `DONE`
**Intent**: `🔄 REFACTOR`
**Oprettet**: `2026-09-09`

## 🎯 Formål
Udbedre tekniske gældsposter og robusthedsmangler identificeret under code review af HUD-adapterne for de 3 harnesses (Claude Code, Codex, Antigravity) og operativsystemer (Linux, macOS, Windows) jf. [spec.md](spec.md), [CONTEXT.md](CONTEXT.md), [ADR 0001](docs/adr/0001-package-by-feature-architecture.md), [ADR 0004](docs/adr/0004-harness-adapter-slices.md) og [ADR 0006](docs/adr/0006-multi-harness-policy-adapter-contract.md):

1. **Boks-layout & OS-resilience (`crates/xgauntlet-core/src/features/tasks/telemetry.rs`)**:
   - Garantere præcis 64 tegns bredde for samtlige 6 linjer i `render_box_card` uanset længden af `scope`, `branch`, `status`, `git` eller `evidence`. Implementere deterministisk afkortning/ellipsing (f.eks. `.../pkg` eller `prefix...`), så linjebredden aldrig overskrides eller skubber højre ramme `│`.
   - Normalisere `self.file_path` til POSIX-skråstreger (`/`), så Windows backslashes (`tasks\019.md`) konverteres til ensartede, klikbare stier (`tasks/019.md`) i både `Ref:` rækken og `.starts_with("tasks/")`.

2. **Fejlhåndtering i Scaffolding & CLI (`crates/xgauntlet-cli/src/main.rs` & `xgauntlet-core`)**:
   - Fjerne `let _ =` ved kald til harness scaffolding (`ClaudeCodeAdapter::scaffold_settings`, `CodexAdapter::scaffold_hooks`, `AntigravityAdapter::scaffold_hooks`) i `main.rs`. Propagere fejl korrekt jf. C-GOOD-ERR og No Symptom Masking.
   - Sikre at harness scaffolding-filer registreres i `ScaffoldResult` outputtet ved `xgauntlet init --harness ... --json`.

3. **DRY JSON Hook Merging & Harness-aliaser (`features/adapters/`)**:
   - Udtrække den delte JSON merge-algoritme for `PostToolUse` (som er identisk mellem Claude Code og Codex) til en fælles hjælpefunktion i `adapters/`.
   - Ensrette harness-aliaser (`codex`, `openai`, `openai_codex` og `antigravity`, `google_antigravity`) via en stærk parsing-type (`HarnessKind`).

4. **Opgradering af Repository Hooks (`.agents/hooks.json` & `AntigravityAdapter`)**:
   - Justere `AntigravityAdapter::generate_hooks_json`, så den ikke anser legacy `python3 -m agent_gauntlet...` for at være permanent gyldig, men opgraderer den til `xgauntlet hook antigravity`.
   - Opdatere det aktive repositories `.agents/hooks.json` til den kanoniske `xgauntlet hook antigravity` kommando.

5. **CLI Harness-Wrapping for Checkpoint & Verify**:
   - Understøtte et `--harness` flag i `Commands::Checkpoint` og `Commands::Verify` i `xgauntlet-cli`, der forbinder `wrap_response` til outputtet ved behov.

## 📋 Acceptance Criteria
- [x] `TaskTelemetry::render_box_card` garanterer præcis 64 tegn pr. linje selv ved ekstreme strenglængder for scope, git branch, status eller ref-stier.
- [x] `TaskTelemetry::render_box_card` og sti-håndtering normaliserer Windows-stier med backslashes (`\`) til standard fremadrettede POSIX-skråstreger (`/`).
- [x] `crates/xgauntlet-cli/src/main.rs` fjerner `let _ =` og propagerer IO-fejl ved harness-scaffolding under `xgauntlet init`.
- [x] Harness-specifikke filer inkluderes i `ScaffoldResult` ved `xgauntlet init --harness ...`.
- [x] Den delte JSON merge-logik for `PostToolUse` i Claude Code og Codex er konsolideret i et fælles modul uden kodeduplikering.
- [x] Harness-aliaser (`codex`, `openai`, `openai_codex`, `antigravity`, `claude_code`) er ensartet defineret på tværs af CLI og core.
- [x] `AntigravityAdapter::generate_hooks_json` opgraderer forældede python-hooks til `xgauntlet hook antigravity`.
- [x] Workspace `.agents/hooks.json` er opdateret til at anvende `xgauntlet hook antigravity` i stedet for `python3`.
- [x] `Commands::Checkpoint` og `Commands::Verify` understøtter `--harness` respons-wrapping med det respektive telemetry-kort.
- [x] Unit- og integrationstests i `harness_adapters_test.rs` og `task_lifecycle_test.rs` dækker samtlige nye grænsetilfælde for lange navne, Windows-stier og fejlforhold.
- [x] `cargo test --workspace` og `cargo clippy` forbliver 100% grønne uden advarsler.

## 🚫 Must NOT
- Må IKKE bryde eksisterende arkitektur-invarianter eller API-kontrakter.
- Må IKKE foretage remote publication handlinger (`git push`) jf. ADR 0003.
- Må IKKE introducere baggrunds-dæmoner jf. Zero-Daemon invarianten.
- Må IKKE sluge filsystem-fejl under scaffolding uden fejlmeddelelse.
- Må IKKE ødelægge eksisterende brugerdefinerede hooks i `.agents/hooks.json`, `.claude/settings.json` eller `.codex/hooks.json`.

## 📝 Revisions
- 2026-09-09: Task opdateret med konkrete udbedringskriterier baseret på grundigt code review.
- 2026-09-09: 100% implementeret og verificeret (DONE). Boks-ellipsing, Windows-sti-normalisering, scaffold-fejlhåndtering, DRY hook merge, HarnessKind aliaser, lokal hooks opgradering og CLI wrap_response integration for checkpoint og verify.

## 🧪 Verifikation
- `cargo run -p xgauntlet-cli -- check-spec -t 019`
- `cargo test -p xgauntlet-core --test task_lifecycle_test`
- `cargo test -p xgauntlet-core --test harness_adapters_test`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo fmt --check`


