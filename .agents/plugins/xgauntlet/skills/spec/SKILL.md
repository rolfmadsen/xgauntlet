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
2. **Anti-Monolit Test Refs:** Hver opgave SKAL have isolerede testfiler i `test_refs: [...]`. `check-spec` forbyder deklaration af testfiler over 1.000 linjer.
3. **Design Tokens & WAI 2.0:** UI-komponenter må ALDRIG indeholde rå farveværdier eller ad-hoc styling. Alle farver, typografi og afstande skal referere til projektets design-tokens.
4. **Shift-Left Gatekeeper:** Opgaven valideres altid med:
   `xgauntlet check-spec -t <id>`
   før der skrives produktionskode.
