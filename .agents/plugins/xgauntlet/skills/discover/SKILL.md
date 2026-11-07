---
name: discover
description: Sokratisk sparring, udfordring af antagelser, problemrumsafgrænsning og Aristotelisk domæne-glossary i CONTEXT.md før specifikation og kodning.
---

# Sovereign Skill: discover

Fase 1 i xGauntlets Agentic SDLC: **Ideation & Context Alignment**.

Denne skill forvandler agenten fra en passiv "yes-man" til en proaktiv, intellektuel sparringspartner. Brug altid `discover`, når brugeren præsenterer en idé, et problem, en ny feature, en bug eller et refaktoreringsønske, før der oprettes en formel opgavefil.

---

## 🎯 De 4 Trin i Discover-Processen

### Trin 1: Formål & Afgrænsning (Problem Boundary)
- **Hvad er det reelle behov?** Adskil det underliggende forretnings- eller arkitekturbehov fra den foreslåede løsning.
- **Hvad er eksplicit udeladt (Non-Goals)?** Træk en skarp grænse for, hvad opgaven *ikke* skal løse, for at undgå scope creep.

### Trin 2: Udfordre Antagelser & Afdække Blinde Vinkler
- **Udfordr svage antagelser:** Undersøg trade-offs, performance-implikationer, state management kompleksitet og vedligeholdelsesbyrder.
- **Spørg sokratisk:** Stil 1-3 præcise, dybe spørgsmål for at nå til fælles forståelse.

### Trin 3: Aristotelisk Begrebsafklaring (`CONTEXT.md`)
- Hvert nyt eller uklart domænebegreb defineres jf. Aristoteles' metode:
  - **Genus Proximum:** Hvilken overordnet kategori tilhører begrebet?
  - **Differentia Specifica:** Hvad adskiller dette begreb specifikt fra alle andre begreber i samme kategori?
- Etabler og opdater tabellen i `CONTEXT.md`.

### Trin 4: Invarianter, Must NOT & ADR-Triggere
- **Must NOT:** Fastlæg negative begrænsninger, der under ingen omstændigheder må overtrædes (f.eks. Zero-Daemon, Zero Ambient Authority, forbud mod rå styling uden om design tokens).
- **ADR-triggere:** Hvis idéen introducerer et irreversibelt trade-off eller udfordrer en eksisterende beslutning i `docs/adr/`, skal en ny ADR formuleres.

---

## 🧭 DDD & Vertical Slice Taksonomi (Hvad, Hvorfor & Hvordan)
Under discovery placeres opgaven i `xGauntlet`s 4-niveaus domæne- og slice-taksonomi:
1. **Bounded Context (`CONTEXT.md`):** Den sproglige og arkitektoniske systemgrænse i løsningsrummet, hvor hvert begreb har én entydig betydning (*Aristoteles' genus et differentiam*).
2. **Domain (`src/features/<domain>/`):** Et fagligt delområde inden for konteksten (f.eks. `diagram`, `billing`, `export`).
3. **Slice (`src/features/<domain>/<slice>/`):** Den konkrete, autonome funktionelle kapabilitet (*Stateful UI-Slice* eller *Stateless Pipeline-Slice*) med Black-Box indkapsling (egen offentlig facade og nul direkte kobling til andre slices).
4. **ADR Gatekeeper-reglen (`docs/adr/` vs. `specs/<domain>/<slice>.md`):**
   - **Levende Slice-Specifikation (`specs/<domain>/<slice>.md`):** Beskriver *funktionel adfærd, UI-state, input og udgående Actions* for en konkret slice.
   - **ADR (`docs/adr/`):** Er **kun** for tværgående, irreversible arkitekturbeslutninger med reelle trade-offs. Skriv ALDRIG almindelige feature-krav som en ADR.

---

## 🏁 Output fra Discover
Når der er opnået enighed og fælles forståelse:
1. `CONTEXT.md` er opdateret med de nye domænebegreber.
2. Target `Domain` og `Slice` (`specs/<domain>/<slice>.md`) er identificeret som grundlag for `spec`-skillen, der syntetiserer en formel `tasks/<id>.md` opgavepakke.
