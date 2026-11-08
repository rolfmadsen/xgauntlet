---
name: spec
description: Sokratisk kravsspecifikation, nedbrydning i OKF v0.2 opgavepakker (tasks/<id>-<slug>.md) med obligatorisk scope, test_refs og tilgængelige design tokens (WAI 2.0 / WCAG AA).
---

# Sovereign Skill: spec

Fase 2 i xGauntlets Agentic SDLC: **Specification & Task Binding**.

Denne skill forvandler den fælles forståelse fra `discover` til en formel, maskinelt validerbar OKF v0.2 opgavepakke i `tasks/<number>-<slug>.md`.

---

## 📐 Krav til OKF v0.2 Opgavepakker

Hver opgave SKAL starte med komplet YAML frontmatter og overholde følgende regler:

```markdown
---
type: Task Package
title: "Task <number>: <Title>"
description: "<Kort formålsbeskrivelse>"
status: active
generated: { by: process:antigravity-task-init, at: "<YYYY-MM-DDTHH:MM:SSZ>" }
tags: [<feature-tags>]
scope:
  - "crates/<crate>/src/features/<feature>/**"
test_refs:
  - "crates/<crate>/tests/<feature>_test.rs"
---

# Task <number>: <Title>

**Status**: `ACTIVE`
**Intent**: `🚀 NEW FEATURE` | `🐛 BUG FIX` | `🔄 REFACTOR` | `🔄 ENHANCEMENT`
**Oprettet**: `YYYY-MM-DD`

## 🎯 Formål
Konkret målsætning og operationel afgrænsning.

## 📋 Acceptance Criteria
- [ ] **AC1**: Eksekverbart kriterium med klart input og forventet output.
- [ ] **AC2**: Tilgængelighed og Design Tokens (hvis UI berøres: WCAG AA >= 4.5:1 kontrast og genbrug af tokens).

## 🚫 Must NOT
- Negative begrænsninger og arkitekturinvarianter (f.eks. forbud mod giga-filer over 800 linjer).

## 📝 Revisions
- YYYY-MM-DD: Task initialiseret.

## 🧪 Verifikation
- Specifikke testkommandoer til validering.
```

---

## 🛡️ Arkitektoniske Invarianter i Spec
1. **Deterministisk Scope:** Hver opgave SKAL deklarere `scope: [...]`. Agenter blokeres fysisk af WASM policy motoren, hvis de forsøger at skrive til filer uden for dette scope.
2. **Anti-Monolit Test Refs:** Hver opgave SKAL have isolerede testfiler i `test_refs: [...]`. `check-spec` forbyder deklaration af testfiler over 1.000 linjer. For Rust-workspaces understøttes modul-baserede accepttests under én samlet harness (`tests/features.rs` / `tests/features/*.rs`).
3. **Fast-Track Bugfix Bounding:** Opgaver med `intent: bug` kvalificerer til den accelererede Fast-Track Lane jf. ADR 0011, hvis `scope` indeholder højst 3 kildefiler (`scope.len() <= 3`) og ingen ADR- eller `gauntlet.toml`-filer berøres.
4. **Design Tokens & WAI 2.0:** UI-komponenter må ALDRIG indeholde rå farveværdier eller ad-hoc styling. Alle farver, typografi og afstande skal referere til projektets design-tokens.
5. **De 4 Universelle Lag & Levende Slice-Specifikationer (`app -> features -> core -> shared` & `spec_ref`):**
   - **De 4 Lag (`app -> features -> core -> shared`):** Top-down envejs-imports håndhæves af `xgauntlet check-architecture` (`LayerDirectionViolation`, `CrossSliceImportViolation`, `ForbiddenCoreImportViolation`).
   - **Slice -> Task (`spec_ref` — 1- eller 2-niveau):** Aktive feature/enhancement/bug tasks i slice-baserede projekter binder til deres levende slice-specifikation i enten 1-niveau (`specs/<slice>.md` via `slice` og `spec_ref`) eller 2-niveau (`specs/<domain>/<slice>.md` via `subdomain`/`domain`, `slice` og `spec_ref`, f.eks. `xgauntlet task new <slug> --slice <slice> [--subdomain <domain>]`).
6. **Shift-Left & Post-Implementation Gatekeeper:** Opgaven valideres før kodning med `xgauntlet check-spec -t <id>`, og efter `GREEN` verificeres kildemodeller mod `CONTEXT.md` med `xgauntlet check-spec --context`.
