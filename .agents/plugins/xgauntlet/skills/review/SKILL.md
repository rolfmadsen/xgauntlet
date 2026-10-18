---
name: review
description: To-akset kode- og arkitekturgennemgang (Akse 1: Specifikation og Invarianter vs. Akse 2: Kodestandarder, Fowler code smells og arkitekturgrænser) styret af en eksplicit 'Definition of Enough'.
---

# Sovereign Skill: review

Fase 5 i xGauntlets Agentic SDLC: **Standards & Spec Audit (Two-Axis Review)**.

Gennemfører en uafhængig, stringent evaluering af ændringer baseret på to ortogonale akser: **Specifikation** og **Standarder/Arkitektur** under en fast, konvergerende **Definition of Enough**.

---

## 🎯 Definition of Enough & Trusselsmodel (Anti-Fejlfinder Bias)

Sprogmodeller har en iboende fejlfinder-bias, hvor de belønnes for altid at finde nye teoretiske problemer, selv når kode er produktionsklar. `review`-skillen forhindrer denne uendelige løkke via en eksplicit trusselsmodel og proportionalitetsgrænse:

### 1. Trusselsmodel & Realistisk Kontekst
- **Aktør:** Koden udvikles af og til samarbejdsvillige AI-agenter og udviklere i et kontrolleret miljø.
- **Trussel:** Utilsigtede runtime-crashes, datatab, Git-forurening og kontraktbrud – **IKKE** ondsindede hackere, der forsøger jailbreaks eller shellcode-injektioner.
- **Udelukkelse:** Ignorer teoretiske edge-cases, mikro-optimeringer, hypotetisk allokator-teori og kode-æstetik, medmindre det direkte bryder med repositoryets standarder.

### 2. Tærskelkrav for Fund (Severity Threshold)
Rapportér **KUN** fejl, der opfylder mindst ét af følgende tre kriterier:
1. **Reelle Nedbrud:** Panics, ubehandlede fejl eller deadlocks under almindelig drift.
2. **Datatab & Git-forurening:** Filer committet uden for opgavens erklærede `scope` eller tavst overskrevne data.
3. **Kontraktbrud:** Eksplicitte brud på opgavens `acceptance_criteria` eller `Must NOT`-invarianter.

### 3. Frys Målstregen (Lukket Verifikation)
- **Runde 1 (Discovery Review):** Identificér reelle stopklodser iht. tærskelkravene.
- **Runde 2+ (Lukket Verifikation):** Revieweren må **KUN** verificere, om de aftalte punkter fra forrige runde er løst i den aktuelle diff. Det er strengt forbudt at flytte målstregen eller introducere nye review-kategorier/bagateller.
- **Obligatorisk Stopkriterie:** Hvis ingen fund overskrider tærskelkravene, SKAL revieweren returnere:  
  `STATUS: SHIP IT (Definition of Enough opfyldt)` uden supplerende forslag.

---

## 🧭 Den To-Aksede Review Model

Reviewet udføres altid mod opgavens specifikke **Git diff** (`git diff HEAD~1..HEAD` eller `git diff --cached`) eller isolerede feature-slice – **aldrig** mod et ustruktureret monolitisk helheds-dump.

### Akse 1: Specifikation & Invariant-opfyldelse
- [ ] **Acceptance Criteria:** Er samtlige kriterier i `tasks/<id>.md` opfyldt og empirisk bevist med tests?
- [ ] **Must NOT Overholdelse:** Er alle negative begrænsninger og arkitektoniske barrierer respekteret?
- [ ] **Scope Integritet:** Blev ændringerne holdt strengt inden for opgavens deklarerede `scope` uden utilsigtet drift?

### Akse 2: Kodestandarder, Fowler Smells & Arkitekturgrænser
- [ ] **Arkitekturgrænser:** Overholder kildekoden `[architecture]` i `gauntlet.toml`? (`xgauntlet check-architecture`).
- [ ] **Design Tokens & WAI 2.0:** Er alle UI-elementer fri for rå farver/størrelser, og overholder de tilgængelighedsstandarder?
- [ ] **Fowler Code Smells:**
  - *Large Class / Long Method:* Filer $\le 800$ linjer, metoder $\le 100$ linjer.
  - *Feature Envy:* Er logik placeret i de rigtige vertikale slices?
  - *Shotgun Surgery:* Er ændringer lokaliseret frem for spredt over mange moduler?
- [ ] **Mekanisk Først-Princip:** Hvis et fund kan tjekkes deterministisk (f.eks. linjelængder, ubrugte imports, versioner), skal det overdrages til `xgauntlet verify` / `check-architecture`, så LLM'en ikke spilder tid på det.
- [ ] **Linter & Typesikkerhed:** Passerer koden `cargo clippy --workspace --all-targets -- -D warnings` (eller stack-ækvivalent) med 0 advarsler?

---

## 📋 Review Rapport Format
Gennemgangen afsluttes med en klar rapport:
1. **Spec Verdict:** `[APPROVED | CHANGES_REQUESTED]`
2. **Architecture Verdict:** `[COMPLIANT | VIOLATIONS_FOUND]`
3. **Konklusion:** `STATUS: SHIP IT` (hvis godkendt) ELLER præcis prioriteret liste over reelle blokere (maks. 3–4 punkter).
