---
type: Task Package
title: "Task 020: Global Plugin Distribution & JIT Skill Injection"
description: "Etablere global plugin distribution (xgauntlet omdøbning og registrering i ~/.gemini/config/plugins/xgauntlet) samt Just-In-Time fase-instruktion via harness telemetry hooks"
status: active
generated: { by: process:xgauntlet-task-init, at: "2026-09-12T09:14:30Z" }
tags: [plugin, distribution, jit-skills, telemetry, antigravity, claude-code, codex, adr-0004, adr-0006]
---

# Task 020: Global Plugin Distribution & JIT Skill Injection

**Status**: `ACTIVE`
**Intent**: `🚀 NEW FEATURE`
**Oprettet**: `2026-09-12`

## 🎯 Formål
Etablere en ren og decoupled distribution af xGauntlets agent-skills og metoderegler uden at forurene individuelle projekt-repositories med statiske skill-filer jf. [spec.md](spec.md), [CONTEXT.md](CONTEXT.md), [ADR 0004](docs/adr/0004-harness-adapter-slices.md) og [ADR 0006](docs/adr/0006-multi-harness-policy-adapter-contract.md):
1. **Omdøbning af plugin-pakken**:
   - Flytte og omdøbe `.agents/plugins/agent-gauntlet/` til `.agents/plugins/xgauntlet/`.
   - Opdatere `plugin.json` (`"name": "xgauntlet"`), `hooks.json` og tilhørende skill-metadata.
2. **Global Plugin Distribution**:
   - Etablere CLI subcommand `xgauntlet plugin install` (med `--global` og `--dry-run`), der kan installere eller oprette et symbolsk link til xGauntlet i Google Antigravitys globale konfigurationskatalog (`~/.gemini/config/plugins/xgauntlet`) samt forberede Claude Code / Codex harnesses.
3. **Just-In-Time (JIT) Phase Context / Skill Injection**:
   - Udbygge telemetrimotoren og harness-adapterne (`antigravity`, `claude_code`, `codex`), så hooks dynamisk injicerer de nødvendige metodiske regler direkte i LLM'ens prompt-kontekst baseret på den aktive opgavefase:
     - `SPEC / Uafklaret`: Socratic Grilling vejledning (`grill-me` / `grill-with-docs`) og Aristoteles-glossary krav.
     - `RED`: Disciplined Diagnosis loop (`diagnose`: reproduce -> minimize -> hypothesize -> test).
     - `GREEN / REFACTOR`: TDD disciplin og non-functional invariants.
     - `DONE / GAUNTLET`: Two-axis code review og verification report sealing.
4. **Verifikation & Kvalitetssikring**:
   - Omfattende testsuite i `crates/xgauntlet-core/tests/` for global plugin-installation, manifest-validering og JIT fase-injektion.

## 📋 Acceptance Criteria
- [ ] Mappen `.agents/plugins/agent-gauntlet/` i xgauntlet-repoet er omdøbt til `.agents/plugins/xgauntlet/` med opdateret `plugin.json` (`name: "xgauntlet"`).
- [ ] `cargo run -p xgauntlet-cli -- validate-plugin --plugin-dir .agents/plugins/xgauntlet` validerer med 0 fejl.
- [ ] CLI subcommand `xgauntlet plugin install` er tilgængelig og understøtter flagene `--global`, `--target <dir>`, `--dry-run` og `--json`.
- [ ] Global plugin-installation opretter en gyldig Antigravity plugin-struktur i `~/.gemini/config/plugins/xgauntlet` uden at overskrive brugerdata uden `--force`.
- [ ] Telemetrimotoren (`features/telemetry/`) genererer JIT fase-specifikke prompt-instruktioner baseret på aktiv opgavefase.
- [ ] Harness-adapterne udstiller de genererede fase-instruktioner i deres respektive hook-payloads (`PreInvocation` for Antigravity, `additionalContext` for Claude Code og Codex).
- [ ] `AGENTS.md` og `CLAUDE.md` skabelonerne opdateres til at referere til `xgauntlet` pluginnet og dets globale/JIT tilgængelighed.
- [ ] Conformance-tests i `crates/xgauntlet-core/tests/` dækker plugin-installation og JIT fase-injektion med 100% grøn status.
- [ ] `cargo run -p xgauntlet-cli -- check-spec -t 020-global-plugin-distribution-and-jit-skill-injection` validerer med 0 fejl.
- [ ] Fuld workspace testsuite passerer (`cargo test --workspace`) uden linter-advarsler (`cargo clippy --workspace --all-targets -- -D warnings`).

## 🚫 Must NOT
- Må IKKE destruktivt overskrive eksisterende brugerkonfigurationer i `~/.gemini/config/` uden eksplicit `--force`.
- Må IKKE tvinge unødvendige statiske skill-filer ind i forretningsprojekters kildetræ under `xgauntlet init`.
- Må IKKE forringe sub-3ms koldstarts-invarianten for hook- og telemetriafvikling.
- Må IKKE foretage remote publication handlinger (`git push`).
- Må IKKE introducere baggrunds-dæmoner jf. Zero-Daemon invarianten.

## 📝 Revisions
- 2026-09-12: Task 020 oprettet som ACTIVE for Global Plugin Distribution & JIT Skill Injection.

## 🧪 Verifikation
- `cargo run -p xgauntlet-cli -- check-spec -t 020-global-plugin-distribution-and-jit-skill-injection`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo run -p xgauntlet-cli -- validate-plugin --plugin-dir .agents/plugins/xgauntlet`

