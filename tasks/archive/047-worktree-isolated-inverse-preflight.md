---
type: Task Package
title: "Task 047: Worktree-Isolated Inverse Preflight"
description: "Eliminere datatab-risiko i inverse preflight ved at køre baseline-testen i en isoleret git worktree i stedet for at overskrive brugerens working tree"
status: done
generated: { by: process:antigravity-task-init, at: "2026-10-25T19:17:00Z" }
tags: [checkpoint, provenance, inverse-preflight, data-safety, git-worktree, fail-closed, adr-0003]
scope:
  - "crates/xgauntlet-core/src/features/checkpoint/**"
  - "tasks/047-worktree-isolated-inverse-preflight.md"
test_refs:
  - "crates/xgauntlet-core/tests/inverse_preflight_isolation_test.rs"
---

# Task 047: Worktree-Isolated Inverse Preflight

**Status**: `DONE`
**Intent**: `🐛 BUG FIX`
**Oprettet**: `2026-10-25`

## 🎯 Formål
Code review af `origin/main..HEAD` (fund 3) viste, at `verify_inverse_test_failure` i
`checkpoint/provenance.rs` overskriver ucommittede produktionsfiler i brugerens working tree med
`HEAD`-indhold og kun gendanner via `Drop`. Ved Ctrl-C, timeout eller SIGKILL går brugerens
GREEN-ændringer tabt, og gendannelsesfejl ignoreres tavst (`let _ = fs::write`).

Yderligere defekter fundet under verifikation:
- Produktionsfiler, der er slettet i working tree, genopstår efter preflight (de backuppes ikke, men skrives fra `HEAD`).
- Utrackede nye produktionsfiler indgår ikke i `git diff HEAD` og forbliver derfor i "baseline", så baseline ikke er ren.

Løsning: Baseline-testen køres i en midlertidig `git worktree add --detach <tmp> HEAD`, hvori kun
ændrede/nye testfiler kopieres ind. Brugerens working tree røres aldrig.

**Udenfor scope (opfølgning):** Tilskrivning af preflight-fejl pr. test-identifier (review 🟡 #3).

## 📋 Acceptance Criteria
- [x] **AC1 (Working tree urørt)**: Under kørsel af testlaget i inverse preflight har ændrede produktionsfiler i brugerens workspace fortsat deres working-tree-indhold (observeret af testlaget selv via absolut sti).
- [x] **AC2 (Isoleret eksekvering & oprydning)**: Testlaget eksekveres med en arbejdsmappe forskellig fra workspace, og efter preflight eksisterer den midlertidige mappe ikke, og `git worktree list` viser kun hoved-worktree.
- [x] **AC3 (Ingen genopstandelse)**: En produktionsfil slettet i working tree eksisterer stadig ikke efter preflight.
- [x] **AC4 (Ren baseline)**: Utrackede nye produktionsfiler indgår ikke i baseline; en ny test, der kun består med en utracket ny produktionsfil, verificeres som `InversePreflightVerified`.
- [x] **AC5 (Fail-closed opsætning)**: Kan isoleret baseline ikke etableres (f.eks. repo uden `HEAD`), returneres `CheckpointError` i stedet for tavs succes.
- [x] **AC6 (Regression)**: Eksisterende tests i `provenance_engine_test.rs` og `checkpoint_engine_test.rs` består uændret.

## 🚫 Must NOT
- Må IKKE skrive til, slette eller omdøbe filer i brugerens working tree (bortset fra git-interne worktree-metadata og ignorerede build-artefakter).
- Må IKKE bruge `git stash`, `git reset --hard`, `git checkout -- <path>` eller `git clean -f` mod brugerens working tree.
- Må IKKE foretage remote publication (`git push`).
- Må IKKE ændre den offentlige signatur af `verify_inverse_test_failure`.
- Må IKKE efterlade worktree-registreringer ved normal afslutning eller fejl.

## 📝 Revisions
- 2026-10-25: Task oprettet på baggrund af code review af `origin/main..HEAD` (fund 3).
- 2026-10-25: Implementeret IsolatedWorktree mod HEAD, verificeret mod AC1-AC6 via inverse_preflight_isolation_test.

## 🧪 Verifikation
- `cargo test -p xgauntlet-core --test inverse_preflight_isolation_test`
- `cargo test -p xgauntlet-core --test provenance_engine_test`
- `cargo run -p xgauntlet-cli -- check-spec -t 047-worktree-isolated-inverse-preflight`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
