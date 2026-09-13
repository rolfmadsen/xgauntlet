---
type: Task Package
title: "Task 026: In-Repo Migration Engine and Doctor Auto-Remediation"
description: "Etablere 'xgauntlet migrate' og 'xgauntlet doctor --fix' til kirurgisk, in-place schema-migrering af hooks.json og gauntlet.toml uden tab af brugertilpasninger"
status: active
generated: { by: process:xgauntlet-task-init, at: "2026-09-13T21:02:50Z" }
tags: [migration, doctor, hooks, schema, governance, dx, rust]
---

# Task 026: In-Repo Migration Engine and Doctor Auto-Remediation

**Status**: `ACTIVE`
**Intent**: `🚀 NEW FEATURE`
**Oprettet**: `2026-09-13`

## 🎯 Formål
Etablere en robust Developer Experience (DX) opgraderingsmotor (`xgauntlet migrate` og `xgauntlet doctor --fix`), der identificerer forældede schema-strukturer og udfører kirurgiske, in-place migreringer i eksisterende projekter uden risiko for at overskrive custom forretningslogik, egne opgaver eller tilpassede verifikationslag.

## 📋 Acceptance Criteria
- [ ] **Dybde-inspektion i `xgauntlet doctor`**:
  - [ ] Udvid `governance_agent_harness` i doctor-motoren til at parse og inspicere indholdet af `.agents/hooks.json`.
  - [ ] Flag `[WARN]` hvis `.agents/hooks.json` mangler `PreInvocation` telemetri-hook eller benytter det forældede `"agent-gauntlet-gatekeeper"` namespace.
  - [ ] Flag `[WARN]` hvis `gauntlet.toml` refererer forældet `evidence_file = "evidence.json"` i stedet for `"verification-report.json"`.
  - [ ] Give actionable remediation-meddelelse: `Action: Run 'xgauntlet migrate' (or 'xgauntlet doctor --fix')`.
- [ ] **Kirurgisk Migrations-motor (`xgauntlet migrate`)**:
  - [ ] Etablere CLI subcommand `xgauntlet migrate` i `crates/xgauntlet-cli` og kerne-logik i `crates/xgauntlet-core/src/features/migration/`.
  - [ ] Kirurgisk opdatering af `.agents/hooks.json`: Omdøb `"agent-gauntlet-gatekeeper"` til `"xgauntlet"` og injicer `"PreInvocation"` hook uden at overskrive brugerens egne `PreToolUse` matcher-regler.
  - [ ] Kirurgisk opdatering af `gauntlet.toml`: Modernisér `evidence_file` til `"verification-report.json"` uden at berøre eksisterende bruger-definerede `[[layers]]`.
  - [ ] Automatisk scaffolding af manglende template-filer (f.eks. `docs/adr/template.md`).
  - [ ] Flag-understøttelse: `--dry-run` (viser planlagte ændringer uden diskskrivning), `--workspace <path>` og `--json`.
- [ ] **Auto-remediation i `xgauntlet doctor --fix`**:
  - [ ] Tilføje `--fix` flag til `xgauntlet doctor`, som automatisk kalder migrationsmotoren for identificerede governance-advarsler.
- [ ] **Runtime Bagudkompatibilitet**:
  - [ ] `xgauntlet hook` accepterer fortsat legacy namespace `"agent-gauntlet-gatekeeper"` i `.agents/hooks.json` med en opgraderings-anbefaling i stderr.
- [ ] **Multi-layer Verifikation**:
  - [ ] Tilføje omfattende integrationstests i `crates/xgauntlet-core/tests/migration_engine_test.rs` der beviser in-place migrering mod ældre projektopsætninger.
  - [ ] Verificere at samtlige tests og invariant-tjek passerer (`cargo test --workspace`, `cargo clippy`, `xgauntlet check-spec`).

## 🚫 Must NOT
- Må IKKE overskrive brugerens egne `CONTEXT.md`, `spec.md`, `tasks/` eller custom `[[layers]]` i `gauntlet.toml`.
- Må IKKE fjerne eller overskrive brugerens egne skræddersyede agent-hooks i `.agents/hooks.json`.
- Må IKKE introducere eksterne netværkskald eller baggrundsdæmoner jf. Zero-Daemon invarianten.
- Må IKKE foretage remote publication handlinger (`git push`).

## 📝 Revisions
- 2026-09-13: Oprettet efter sparring om opgraderings-DX og konfigurations-drift i forretningsprojekter som `knowledgegraphstudio`.

## 🧪 Verifikation
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo run -p xgauntlet-cli -- check-spec -t 026-in-repo-migration-engine-and-doctor-auto-remediation`
- `cargo run -p xgauntlet-cli -- migrate --dry-run`
