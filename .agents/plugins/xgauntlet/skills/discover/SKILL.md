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

## 🏁 Output fra Discover
Når der er opnået enighed og fælles forståelse:
1. `CONTEXT.md` er opdateret med de nye domænebegreber.
2. Der er formuleret et præcist grundlag for `spec`-skillen, som kan syntetisere en formel `tasks/<id>.md` opgavepakke.
