---
name: release
description: Release readiness audit, changelog verifikation (CHANGELOG.md), ADR dækning og multi-digest integritetsforsegling.
---

# Sovereign Skill: release

Fase 7 i xGauntlets Agentic SDLC: **Release Readiness & Session Handoff**.

Sikrer, at opgaven eller versionen er 100% klar til udgivelse eller integration, at der ikke findes uhåndteret gæld, og at sessionen afsluttes med en ren håndoverdragelse uden "context rot".

---

## 🏁 Trin i Release-Processen

### 1. Release Readiness Audit
Kør xGauntlets indbyggede release-tjek:
```bash
xgauntlet check-release
```
Validerer:
- **CHANGELOG.md:** Er de udførte ændringer dokumenteret under den korrekte sektion jf. Keep a Changelog standarden?
- **ADR Reference Dækning:** Er samtlige ADRs i `docs/adr/` refereret i `README.md` eller `spec.md`?
- **Manifest Integritet:** Stemmer `Cargo.toml` / `package.json` versioner overens på tværs af biblioteker?

### 2. Multi-Layer Gauntlet & Evidensforsegling
Kør fuld verifikation med evidenslagring:
```bash
xgauntlet verify --task <id> --save
```
Bekræft:
- `lint` [PASSED]
- `types` [PASSED]
- `unit` [PASSED]
- `invariants` [PASSED]
- `architecture` [PASSED]
- `evidence.md` og `verification-report.json` er opdateret.

### 3. Frys Opgavepakken (Status: DONE)
- Marker opgaven som færdig i `tasks/<id>.md`:
  - Frontmatter: `status: done`
  - Body: `**Status**: DONE`
- Kør det afsluttende phase checkpoint (genererer automatisk `🏁 SESSION HANDOFF` kortet):
```bash
xgauntlet checkpoint --phase done -t <id>
```

### 4. Clean Worktree Guarantee & To-Niveau Session Handoff
Kontroller at Git-træet er 100% rent:
```bash
git status
```

#### Niveau A: Almindelig Task-Afslutning (Hverdags-synkronisering)
Ved afslutning af et standard arbejds-slice eller en task, skal der **IKKE** oprettes Git tags.
`xgauntlet checkpoint --phase done` eller `xgauntlet handoff` genererer deterministisk det officielle Markdown-kort med den eksakte push-kommando:
```bash
xgauntlet handoff
```
Viser:
```text
> ### 🏁 SESSION HANDOFF • <task_id>
> **Status**: `TASK: DONE` | **Evidens**: `FORSEGLET (Two-Tier Model)` | **Næste Rolle**: `<role>`
> **Git State**: `<branch>@<oid>` • `clean`
> 💡 *Start venligst en frisk chat-session for at bevare et skarpt kontekstvindue uden context rot.*
>
> 📋 **Kopiér og indsæt følgende starter-prompt i en ny chat:**
> ```text
> Hej! Vi skal i denne session fortsætte arbejdet efter afslutning af task <short_task>.
> Læs venligst CONTEXT.md og spec.md samt opgavelisten i tasks/ for at afklare næste opgave.
> ```
>
> 🚀 **Synkroniser dine commits til remote:**
> ```bash
> git push origin <branch>
> ```
```

#### Niveau B: Eksplicit Release Session (Når en ny version skal frigives)
KUN når brugeren specifikt anmoder om at udgive en version eller milepæl:
1. **Deterministisk SemVer Bumping & Manifest Sync:**
   Kør xGauntlets release manager (evaluerer SemVer 2.0.0, opdaterer manifester og roterer CHANGELOG.md):
   ```bash
   xgauntlet release bump patch   # for bagudkompatible fixes
   xgauntlet release bump minor   # for additive, bagudkompatible features
   xgauntlet release bump major   # for inkompatible API/kontrakt-brud
   ```
   (Eller med specifik version: `xgauntlet release prepare 1.0.0`)
2. **Kør Verifikation af Release-Klarhed:**
   ```bash
   xgauntlet check-release
   ```
3. **Udfør De Autoritative Release Git-Kommandoer:**
   Værktøjet udskriver de præcise kommandoer til brugeren:
   ```bash
   git add Cargo.toml Cargo.lock CHANGELOG.md
   git commit -m "chore(release): prepare v<version>"
   git tag -a v<version> -m "Release v<version>"
   git push origin <branch> --tags
   ```
