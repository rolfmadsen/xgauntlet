---
type: Task Package
title: "Task 037: Policy Engine Command Tokenization and Fail-Closed Task Scope"
description: "Erstat naiv delstrengsmatching med struktureret kommandotokenisering og gør task-scope obligatorisk for alle filmodifikationer"
status: todo
generated: { by: process:xgauntlet-task-init, at: "2026-10-17T11:06:00Z" }
tags: [policy, wasm, security, tokenization, scope, fail-closed]
scope:
  - "crates/gauntlet-policy-engine/src/**"
  - "crates/xgauntlet-core/src/features/policy/**"
test_refs:
  - "crates/xgauntlet-core/tests/policy_engine_test.rs"
  - "crates/xgauntlet-core/tests/security_hardening_test.rs"
---

# Task 037: Policy Engine Command Tokenization and Fail-Closed Task Scope

**Status**: `TODO`  
**Intent**: `🐛 BUG FIX`  
**Oprettet**: `2026-10-17`  

## 🎯 Formål
Eliminere omgåelser af destruktive kommandoer ved at erstatte naiv strengmatching (`cmd.contains("git push")`) med struktureret tokenisering og argumentanalyse. Sikre at filmodifikationer (`WriteFile`, `EditFile`) altid kræver et eksplicit, validt opgavescope, således at tomme eller korrupte task-scopes fejler lukket (`Deny`).

## 📋 Acceptance Criteria
- [ ] **AC1 (Struktureret Shell-Tokenisering)**: Kommandostrenge i policy-motoren parses med ord- og argumenttokenisering (håndterer vilkårlige mellemrum, flagforskydninger som `git -C . push`, og aliases) frem for naive delstrenge.
- [ ] **AC2 (Negative Tests for Kommando-omgåelse)**: Tests beviser afvisning af destruktive kommandoer uanset whitespace (`git   push`), globale flag (`git --no-pager push origin main`, `git -C repo push`) og reordering.
- [ ] **AC3 (Fail-Closed Obligatorisk Task Scope)**: Hvis `allowed_scopes` og `allowed_test_targets` er tomme, afvises alle modifikationer af beskyttede filer med `Deny` (reason_code 4037).
- [ ] **AC4 (Fail-Closed Context Loading)**: `EnforcementContext::from_workspace()` må ikke sluge parsefejl med `.ok()`. Hvis en task-fil er korrupt eller mangler gyldigt scope, sættes `has_active_task = false` eller markeres som ugyldig, så der ikke opstår fri skriveadgang.

## 🚫 Must NOT
- Må IKKE tillade destruktive git-handlinger (`push`, `reset --hard`, `clean -f`) blot fordi kommandoen indeholder ekstra mellemrum eller globale flag.
- Må IKKE tillade vilkårlige filmodifikationer hvis opgavescopet er tomt eller udefineret.

## 📝 Revisions
- 2026-10-17: Oprettet som backlogopgave på baggrund af ChatGPT second opinion review (P1 fund).

## 🧪 Verifikation
- `cargo test -p gauntlet-policy-engine`
- `cargo test -p xgauntlet-core --test policy_engine_test`
- `cargo test -p xgauntlet-core --test security_hardening_test`
