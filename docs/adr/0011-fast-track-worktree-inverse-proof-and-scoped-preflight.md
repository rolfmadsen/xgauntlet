---
type: Architectural Decision Record
title: 'ADR 0011: Fast-Track Bugfix Protocol, Worktree Inverse Proof, and Scoped Preflight Execution'
status: accepted
tags:
- fast-track
- bugfix
- inverse-proof
- worktree
- preflight
- tdd
- governance
generated:
  by: process:antigravity-task-init
  at: '2026-11-01T18:35:00Z'
verified: []
---

# 11. Fast-Track Bugfix Protocol, Worktree Inverse Proof, and Scoped Preflight Execution

**Status**: `accepted` (Extends ADR 0003, ADR 0005, and ADR 0009)  
**Date**: `2026-11-01`  

## Context
Efter introduktionen af multi-layer gauntlets og fuld workspace-verifikation i xGauntlet v0.10.x, observeres betydelig udviklingsfriktion og unødig ventetid under iterative TDD-cyklusser i større Rust-projekter:
1. **Uscopet Preflight ved TDD Checkpoints**: Hvert `checkpoint` (`-p red`, `-p green`, `-p refactor`) eksekverede hele workspace-testsuiten (f.eks. 34 test-binarier, >50s CPU-tid) i stedet for kun opgavens deklarerede `test_refs`.
2. **Ceremonielt Overhead ved Bounded Bugfixes**: Små afgrænsede fejlrettelser på 1–3 filer blev tvunget igennem alle 6 faser med fulde builds og commits, selv når fejlen og regressionstesten var simple.
3. **Risiko for Born-Green Tests uden RED Commit**: Hvis en bugfix implementeres i ét samlet trin med regressionstesten, mangler beviset for, at testen faktisk fangede fejlen og ikke var en tautologi ("født grøn").
4. **Destruktiv Workspace Manipulation**: Forsøg på at teste mod tidligere kode via `git stash` eller `git checkout` i det aktive working tree risikerer tab af uncommitted arbejde eller korruption af udviklerens editor-tilstand.

## Decision
Vi etablerer to komplementære verifikations- og optimeringsmekanismer:

### 1. Scoped Pre-flight i TDD Inner Loop (`red`, `green`, `refactor`)
- Under de iterative TDD faser (`red`, `green`, `refactor`) afvikler `xgauntlet checkpoint` udelukkende de specifikke tests, der er deklareret i opgavens `test_refs` frontmatter.
- En dedikeret **Rust Target Resolver** kortlægger test-stierne til de korrekte `cargo test`-argumenter (`--test <target>`, `--test <harness> <mod>`, eller `--lib <module>`).
- Fuld multi-layer verifikation (`gauntlet.toml`) forbliver ufravigelig og komplet ved `xgauntlet verify` og `xgauntlet checkpoint -p done`.

### 2. Fast-Track Bugfix Lane (`intent: bug`)
- Opgaver med `intent: bug` / `🐛 BUG FIX`, hvor `scope.len() <= 3` og uden berøring af `docs/adr/` eller `gauntlet.toml`, kvalificerer automatisk til Fast-Track.
- Fast-Track tillader et forkortet 2-trins flow: `RED → DONE` (hvor `SPEC`- og `REFACTOR`-commits er valgfrie).
- Alternativt tillades et 1-trins atomisk flow via **Worktree Inverse Proof**:
  - `xgauntlet` opretter en isoleret midlertidig `git worktree` i OS temp directory baseret på uændret `HEAD`.
  - Kun ændringerne fra `test_refs` (regressionstesten) kopieres over i worktreen, mens produktionskoden forbliver på `HEAD`.
  - Regressionstesten afvikles mod `HEAD`. Hvis den passerer uden produktionsfixet, afvises opgaven prompte som `BORN_GREEN_TEST`.
  - Hvis testen fejler mod `HEAD` (fejlen er reproduceret mekanisk) OG samtlige checks passerer i det aktive workspace, godkendes og forsegles evidensen som `worktree_inverse_proof`.
  - Worktreen fjernes deterministisk under alle omstændigheder (succes, fejl, panic).

### 3. Invarianter
- Må ALDRIG udføre `git stash`, `git checkout` eller `git reset` i udviklerens aktive working tree under inverse proof.
- Må ALDRIG slække på verifikationskravene ved `DONE`.
- Må ALDRIG tillade Fast-Track for opgaver med `scope.len() > 3` eller opgaver der ændrer `docs/adr/` eller `gauntlet.toml`.
