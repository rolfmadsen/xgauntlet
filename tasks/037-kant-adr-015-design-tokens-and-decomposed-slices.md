---
type: Task Package
title: "Task 037: Kant ADR 015 Accessible Design Tokens & Decomposed UI Slices"
description: "Formulering og godkendelse af arkitekturbeslutning ADR 015 for Kants tilgængelige design-system og feature-opdeling jf. WAI 2.0 / WCAG AA"
status: todo
generated: { by: process:xgauntlet-task-init, at: "2026-10-16T21:48:30Z" }
tags: [kant, adr, adr-015, design-tokens, accessibility, wcag-aa, decomposition]
scope:
  - "docs/adr/**"
  - "README.md"
  - "spec.md"
test_refs:
  - "tests/adr_coverage_test.rs"
---

# Task 037: Kant ADR 015 Accessible Design Tokens & Decomposed UI Slices

**Status**: `TODO`  
**Intent**: `🚀 NEW FEATURE`  
**Oprettet**: `2026-10-16`  

## 🎯 Formål
Formulere og godkende den bindende arkitekturbeslutning [ADR 015] i Kant (`/home/rolfmadsen/Github/kant`), der definerer:
1. Afskaffelse af vilkårlig ad-hoc styling og forbud mod rå RGB-instansiering i UI-komponenter.
2. Fastsættelse af WAI 2.0 / WCAG AA krav om minimum $4.5:1$ kontrastforhold mellem tekst og baggrund.
3. Dekomponeringsstrategi for `src/ui/app.rs` fra én gigantisk monolit til vertikale feature-slices under `src/features/<feature>/ui/`.

## 📋 Acceptance Criteria
- [ ] **AC1 (Formulering af ADR 015)**: Opret `docs/adr/015-accessible-design-tokens-and-decomposed-ui-slices.md` med OKF v0.2 frontmatter, status `Accepted`, og klar kontekst, beslutning og konsekvenser.
- [ ] **AC2 (WCAG AA Invariant)**: ADR 015 fastlægger formelt, at alle tekstfarver og interaktive kontrolelementer skal have $\ge 4.5:1$ kontrast ratio mod deres baggrund.
- [ ] **AC3 (Modulopdelings-standard)**: ADR 015 fastlægger `Package-by-Feature` princippet for UI: view- og update-logik flyttes til `src/features/<feature>/ui/`, mens `src/ui/app.rs` reduceres til en tynd overordnet shell/routing.
- [ ] **AC4 (Dokumentationssynkronisering)**: ADR 015 refereres i `README.md`, `spec.md` og `docs/adr/README.md`.
- [ ] **AC5 (ADR Coverage Validering)**: `xgauntlet check-release` bekræfter 100% ADR-dækning og reference-integritet.

## 🚫 Must NOT
- Må IKKE tillade udokumenterede undtagelser for WCAG AA kontrastkravet.
- Må IKKE introducere eksterne CSS-frameworks eller tunge runtime-afhængigheder, der bryder Iceds native arkitektur.

## 📝 Revisions
- 2026-10-16: Task 037 oprettet til formalisering af ADR 015 i Kant.

## 🧪 Verifikation
- `xgauntlet check-release` (i Kant workspace)
- `xgauntlet check-spec -t 037-kant-adr-015-design-tokens-and-decomposed-slices`
