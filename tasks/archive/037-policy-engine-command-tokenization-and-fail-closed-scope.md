---
type: Task Package
title: "Task 037: Policy Engine Command Tokenization and Fail-Closed Task Scope"
description: "Erstat naiv delstrengsmatching med struktureret kommandotokenisering, afvisning af wrappers, og obligatorisk task-scope for alle filmodifikationer"
status: done
generated: { by: process:xgauntlet-task-init, at: "2026-10-17T11:21:00Z" }
tags: [policy, wasm, security, tokenization, scope, fail-closed]
scope:
  - "crates/gauntlet-policy-engine/src/**"
  - "crates/xgauntlet-core/src/features/policy/**"
  - "crates/xgauntlet-core/src/features/tasks/**"
test_refs:
  - "crates/xgauntlet-core/tests/policy_engine_test.rs"
  - "crates/xgauntlet-core/tests/security_hardening_test.rs"
---

# Task 037: Policy Engine Command Tokenization and Fail-Closed Task Scope

**Status**: `DONE`  
**Intent**: `🐛 BUG FIX`  
**Oprettet**: `2026-10-17`  

## 🎯 Formål
Eliminere omgåelser af destruktive kommandoer ved at erstatte naiv delstrengsmatching (`cmd.contains("git push")`) med struktureret tokenisering af executable og argumenter, samt afvisning af wrapper-eksekveringer (`env`, `bash -c`, `sh -c`, `sudo`). Sikre at alle filmodifikationer (`WriteFile`, `EditFile`) håndhæver fail-closed validering mod et eksplicit og gyldigt opgavescope, således at tomme eller korrupte task-scopes resulterer i en faktisk `Deny`-beslutning, og `allowed_test_targets` aldrig giver utilsigtet skriveadgang til kildekodefiler.

## 📋 Acceptance Criteria
- [x] **AC1 (Struktureret Kommandotokenisering & Wrapper-Afvisning)**: Kommandostrenge parses til executable og argument-tokens. Kommandoer der forsøger at omgå kontrol via eksekverings-wrappers (`env`, `sudo`, `eval`, `bash -c`, `sh -c`, `xargs`) eller subprocess-kæder afvises fail-closed med `Deny`.
- [x] **AC2 (Negative Tests for Destruktiv Kommando-omgåelse)**: Tests beviser afvisning af destruktive git-handlinger uanset whitespace (`git   push`), globale flag (`git --no-pager push origin main`, `git -C repo push`, `git -c foo=bar push`), aliaser eller flagreordering.
- [x] **AC3 (Fail-Closed Obligatorisk Task Scope & Test Scope Isolering)**:
  - Hvis både `allowed_scopes` og `allowed_test_targets` er tomme eller udefinerede, afvises alle modifikationer af filer med `Deny` (reason_code 4037).
  - Hvis `allowed_scopes` er tom, men `allowed_test_targets` indeholder test-stier, giver det KUN skriveadgang til de specifikke test-filer; modifikation af kildefiler afvises altid strengt med `Deny`.
  - Testen skal bevise selve den resulterende `PolicyDecision::Deny`, ikke blot interne flag.
- [x] **AC4 (Fail-Closed Context Loading)**: `EnforcementContext::from_workspace()` må ikke sluge parsefejl med `.ok()`. Hvis en task-fil er korrupt eller ikke kan parses, sættes `has_active_task = false` og `allowed_scopes` efterlades tom, så enhver modificerende operation efterfølgende afvises med `Deny`.

## 🚫 Must NOT
- Må IKKE tillade destruktive git-handlinger (`push`, `reset --hard`, `clean -f`) blot fordi kommandoen indeholder ekstra mellemrum, globale flag eller wrapper-binærer.
- Må IKKE tillade kildekode-modifikationer hvis opgavescopet for kildekode er tomt eller udefineret, uanset om test-targets er angivet.

## 📝 Revisions
- 2026-10-17: Oprettet som backlogopgave på baggrund af ChatGPT second opinion review (P1 fund om kommandobypasses, tomt scope og test-scope afgrænsning).
- 2026-10-17: Løst og verificeret via Evidence-First TDD. Kommandotokenisering implementeret, wrappers afvist, fail-closed obligatorisk scope og test-scope isolering håndhævet samt unswallowed parsefejl i from_workspace. Markeret som DONE.

## 🧪 Verifikation
- `cargo test -p gauntlet-policy-engine`
- `cargo test -p xgauntlet-core --test policy_engine_test`
- `cargo test -p xgauntlet-core --test security_hardening_test`
- `cargo run -p xgauntlet-cli -- check-spec -t 037`
