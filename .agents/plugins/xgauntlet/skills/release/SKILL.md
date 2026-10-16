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

### 4. Clean Worktree Guarantee & Session Handoff
Kontroller at Git-træet er 100% rent:
```bash
git status
```
Vis det officielle `🏁 SESSION HANDOFF` kort i agent-svaret med starter-prompt til den næste session for at forhindre context rot.
