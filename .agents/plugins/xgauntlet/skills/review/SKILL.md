---
name: review
description: To-akset kode- og arkitekturgennemgang (Akse 1: Specifikation og Invarianter vs. Akse 2: Kodestandarder, Fowler code smells og arkitekturgrænser).
---

# Sovereign Skill: review

Fase 5 i xGauntlets Agentic SDLC: **Standards & Spec Audit (Two-Axis Review)**.

Gennemfører en uafhængig, stringent evaluering af ændringer baseret på to ortogonale akser: **Specifikation** og **Standarder/Arkitektur**.

---

## 🧭 Den To-Aksede Review Model

### Akse 1: Specifikation & Invariant-opfyldelse
- [ ] **Acceptance Criteria:** Er samtlige kriterier i `tasks/<id>.md` opfyldt og bevist med tests?
- [ ] **Must NOT Overholdelse:** Er alle negative begrænsninger og arkitektoniske barrierer respekteret?
- [ ] **Scope Integritet:** Blev ændringerne holdt strengt inden for opgavens deklarerede `scope` uden utilsigtet drift?

### Akse 2: Kodestandarder, Fowler Smells & Arkitekturgrænser
- [ ] **Arkitekturgrænser:** Overholder kildekoden `[architecture]` i `gauntlet.toml`? (`xgauntlet check-architecture`).
- [ ] **Design Tokens & WAI 2.0:** Er alle UI-elementer fri for hårdkodede farver/størrelser, og overholder de tilgængelighedsstandarder?
- [ ] **Fowler Code Smells:**
  - *Large Class / Long Method:* Filer $\le 800$ linjer, metoder $\le 100$ linjer.
  - *Feature Envy:* Er logik placeret i de rigtige vertikale slices?
  - *Shotgun Surgery:* Er ændringer lokaliseret frem for spredt over mange moduler?
- [ ] **Linter & Typesikkerhed:** Passerer koden `cargo clippy --workspace --all-targets -- -D warnings` (eller stack-ækvivalent) med 0 advarsler?

---

## 📋 Review Rapport Format
Gennemgangen afsluttes med en klar rapport:
1. **Spec Verdict:** `[APPROVED | CHANGES_REQUESTED]`
2. **Architecture Verdict:** `[COMPLIANT | VIOLATIONS_FOUND]`
3. **Konkrete Fund & Handlingsanvisninger** (hvis relevant).
