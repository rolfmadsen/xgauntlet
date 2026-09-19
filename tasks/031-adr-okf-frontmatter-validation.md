---
type: Task Package
title: "Task 031: ADR OKF v0.2 Frontmatter Validation & Template Hardening"
description: "Håndhæve OKF v0.2 påkrævede felter (generated, verified) i ADR-filer via check-spec-validering og rette scaffold-template til at inkludere dem"
status: active
generated: { by: process:xgauntlet-task-init, at: "2026-09-19T13:50:00Z" }
tags: [okf, adr, frontmatter, validation, check-spec, scaffold, template, docs]
---

# Task 031: ADR OKF v0.2 Frontmatter Validation & Template Hardening

**Status**: `ACTIVE`  
**Intent**: `🚀 NEW FEATURE`  
**Oprettet**: `2026-09-19`  

## 🎯 Formål & Løsningsdesign
ADR-filer i klientprojekter scaffoldet via `xgauntlet init` mangler OKF v0.2 felternes `generated` og `verified` i frontmatteren, fordi `render_adr_template_md()` i `features/scaffold/templates.rs` genererer et ufuldstændigt skelet. Derudover håndhæves disse felter ikke af nogen gate — hverken WASM-policy-engine, `check-spec` eller gauntlet — selvom OKF-parseren (`okf.rs`) fuldt ud understøtter dem på `OkfMetadata`-strukturen som valgfrie felter.

Opgaven retter dette ved to adskilte angrebsvinkler:

### 1. Template Hardening (`features/scaffold/templates.rs`)
Opdatere `render_adr_template_md()` til at generere et komplet OKF v0.2 skelet med placeholdere for `generated` og `verified`, identisk med formatet i xGauntlets egne ADR-filer:
```yaml
---
type: Architectural Decision Record
title: 'ADR [Number]: [Short Title]'
status: draft
tags: [architecture, adr]
generated:
  by: human:<author>
  at: 'YYYY-MM-DDTHH:MM:SSZ'
verified: []
---
```

### 2. ADR OKF Validering (`features/tasks/validator.rs` + `lib.rs` + CLI)
Tilføje en ny public funktion `check_adr_frontmatter(adr_path: &Path) -> AdrReadinessReport` i `validator.rs`, der validerer en enkelt ADR-fil mod OKF v0.2 krav:
- **Påkrævet**: `type: Architectural Decision Record`, `title`, `status`
- **Påkrævet (ny regel)**: `generated.by` (valid OKF Actor: `human:<id>`, `process:<id>`, eller `<agent>/<version>`) og `generated.at` (valid ISO 8601 UTC timestamp)
- **Anbefalet (warning)**: `verified` er til stede som liste (kan være tom `[]`)

En ny `check-adr` CLI-subkommando giver udviklere og agenter mulighed for at validere enkelt- eller alle ADRs i `docs/adr/`:
```bash
xgauntlet check-adr                                # Alle ADRs i docs/adr/
xgauntlet check-adr -f docs/adr/001-bootstrap.md   # Enkelt ADR
xgauntlet check-adr --all --json                   # JSON output
```

---

## 📋 Acceptance Criteria
- [ ] **AC1 (Template Fix i `templates.rs`)**: `render_adr_template_md()` returnerer et ADR-skelet med `generated:` og `verified:` blokke i OKF v0.2 format, inkl. placeholdere. Eksisterende tests opdateres/tilføjes for at bekræfte det nye indhold.
- [ ] **AC2 (`AdrReadinessReport` model i `models.rs`)**: Ny struct `AdrReadinessReport` (analogt med `SpecReadinessReport`) eksponeres fra `features/tasks/models.rs` med felter `adr_id`, `is_valid`, `diagnostics`, `inspected_files`.
- [ ] **AC3 (`check_adr_frontmatter` funktion i `validator.rs`)**: Ny funktion `check_adr_frontmatter(adr_path: &Path) -> AdrReadinessReport` validerer:
  - `type` er `"Architectural Decision Record"` (fejl hvis fraværende eller forkert)
  - `title` er tilstede og ikke tom (fejl)
  - `status` er tilstede og ikke tom (fejl)
  - `generated.by` er tilstede og er en valid OKF Actor (fejl)
  - `generated.at` er tilstede og er en valid ISO 8601 UTC timestamp (fejl)
  - `verified` er tilstede (warning, ikke fejl — kan være tom liste)
- [ ] **AC4 (`check_all_adrs` bulk funktion i `validator.rs`)**: Ny funktion `check_all_adrs(workspace: &Path) -> Vec<AdrReadinessReport>` scanner alle `*.md` filer i `docs/adr/` (eksklusiv `template.md` og `README.md`) og kører `check_adr_frontmatter` på hver.
- [ ] **AC5 (`check-adr` CLI subkommando i `main.rs`)**: Ny `check-adr` subkommando tilgængelig i `xgauntlet-cli` med:
  - `-f`/`--file <PATH>`: Valider enkelt ADR-fil
  - `-w`/`--workspace <PATH>`: Workspace-rod (default: `.`)
  - `--all`: Valider alle ADRs i `docs/adr/`
  - `--json`: Struktureret JSON output
  - Exit code 1 ved validationsfejl (warnings giver ikke exit code 1)
- [ ] **AC6 (Pub-eksponering i `lib.rs`)**: `check_adr_frontmatter` og `check_all_adrs` eksporteres fra `xgauntlet-core` crate root.
- [ ] **AC7 (Tests i `task_lifecycle_test.rs` eller ny `adr_validation_test.rs`)**: Tests verificerer:
  - ADR med komplet OKF v0.2 header validerer OK
  - ADR uden `generated` giver `GENERATED_MISSING` fejl
  - ADR med ugyldigt `generated.by` format giver `INVALID_ACTOR` fejl
  - ADR med ugyldigt `generated.at` timestamp giver `INVALID_TIMESTAMP` fejl  
  - ADR med korrekt header men manglende `verified` giver warning (ikke fejl)
  - `check_all_adrs` springer `template.md` og `README.md` over
  - `cargo run -p xgauntlet-cli -- check-spec -t 031-adr-okf-frontmatter-validation` validerer med 0 fejl
- [ ] **AC8 (xGauntlets egne ADRs validerer grønt)**: `xgauntlet check-adr --all` på xgauntlet-repositoriet selv returnerer 0 fejl.

---

## 🚫 Must NOT
- Må IKKE bryde eksisterende `check_task_specification` adfærd eller API-kontrakter for task-pakker.
- Må IKKE kræve `verified` som hård fejl — kun warning — da tomme `verified: []` er valid OKF v0.2 praksis for nyoprettede ADRs.
- Må IKKE touch WASM policy engine (`gauntlet-policy-engine`): ADR-validering er et dokumentvalideringslag, ikke et capability enforcement lag.
- Må IKKE foretage remote publication handlinger (`git push`) jf. ADR 0003.
- Må IKKE introducere baggrundsdæmoner jf. Zero-Daemon invarianten.

---

## 📝 Revisions
- 2026-09-19: Task oprettet som ACTIVE baseret på observation af OKF-header-divergens mellem `edge/docs/adr/` (mangler `generated`/`verified`) og `xgauntlet/docs/adr/` (fuldt OKF v0.2).

---

## 🧪 Verifikation
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo fmt --check`
- `cargo run -p xgauntlet-cli -- check-spec -t 031-adr-okf-frontmatter-validation`
- `cargo run -p xgauntlet-cli -- check-adr --all`
