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
- Commit det afsluttende phase checkpoint:
```bash
git commit -m "chore(<id>): seal evidence and mark task DONE"
```

### 4. Clean Worktree Guarantee & To-Niveau Session Handoff
Kontroller at Git-træet er 100% rent:
```bash
git status
```

#### Niveau A: Almindelig Task-Afslutning (Hverdags-synkronisering)
Ved afslutning af et standard arbejds-slice eller en task, skal der **IKKE** oprettes Git tags.
Vis det officielle `🏁 SESSION HANDOFF` kort med den rene push-kommando:
```text
> ### 🏁 SESSION HANDOFF • <task_id>
> **Status**: `TASK: DONE` | **Evidens**: `FORSEGLET`
> 
> 📋 Kopiér og kør for at synkronisere dine lokale commits:
> ```bash
> git push origin <branch>
> ```
> 💡 Start en frisk chat-session for næste opgave.
```

#### Niveau B: Eksplicit Release Session (Når en ny version skal frigives)
KUN når brugeren specifikt anmoder om at udgive en version eller milepæl:
1. **Evaluer Kontrakt-Kompatibilitet (SemVer 2.0.0):**
   - **MAJOR:** Inkompatible API-ændringer, fjernede funktioner eller brydende schema/konfigurationsændringer.
   - **MINOR:** Nye additive, bagudkompatible funktioner eller moduler.
   - **PATCH:** Bagudkompatible fejlrettelser og interne refaktoriseringer.
2. **Bekræft med Brugeren:** Fremlæg den foreslåede versionsstigning og afvent bekræftelse.
3. **Opdater Manifests:** Bump version i `Cargo.toml`, `package.json`, osv.
4. **Vis Udgivelses- og Tag-Kommandoer:**
```bash
git push origin <branch>
git tag -a v<version> -m "Release v<version>: <kort titel>"
git push origin --tags
```
