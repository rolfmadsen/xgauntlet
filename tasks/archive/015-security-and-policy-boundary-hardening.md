---
type: Task Package
title: "Task 015: P0 Security & Policy Boundary Hardening"
description: "Hærdning af xGauntlet sikkerhedsgrænser: Fail-closed task resolution, kanoniske WorkspaceRelativePath stier, WasmPolicyEngine integration i adaptere, forsvar mod kommandokædning, typesikker serde i WASM-motor, udvidet anti-tamper manifest og eksakt drift detection jf. spec.md, ADR 0003, ADR 0006 og ADR 0007"
status: done
generated: { by: process:xgauntlet-task-init, at: "2026-09-07T18:17:00Z" }
tags: [security, policy, wasm, task-contract, path-traversal, fail-closed, anti-tamper, adr-0006, adr-0007]
---

# Task 015: P0 Security & Policy Boundary Hardening

**Status**: `DONE`
**Intent**: `🔄 REFACTOR`
**Oprettet**: `2026-09-07`

## 🎯 Formål
Lukke samtlige kritiske huller mellem det specificerede sikkerhedsniveau og den faktiske implementation i xGauntlet jf. [spec.md](spec.md), [ADR 0003](docs/adr/0003-surgical-gatekeeper-and-no-remote-push.md), [ADR 0005](docs/adr/0005-two-tier-verification-and-attestation-model.md), [ADR 0006](docs/adr/0006-multi-harness-policy-adapter-contract.md) og [ADR 0007](docs/adr/0007-local-transparent-supervisor-and-wasm-verifier.md):
1. **Fail-Closed Task Binding**:
   - Eliminere syntetisk `default-task` fallback i `features/gauntlet/pipeline.rs`.
   - Sikre at `resolve_task_contract` fejler hårdt hvis opgaven ikke findes, er ugyldig eller mangler acceptkriterier.
   - Fjerne utilsigtet fallback til aktiv opgave når en eksplicit opgave `--task <id>` ikke findes.
2. **Kanonisk Stistyring og Workspace Containment**:
   - Implementere `WorkspaceRelativePath` primitive der afviser stitraversering (`..`), absolutte stier (`/etc/passwd`), Windows drive letters og UNC-stier.
   - Håndhæve workspace containment før en `CapabilityRequest` oprettes i adapterne.
   - Sikre at naive strengpræfikser i policy engine (`docs/`, `tasks/`) ikke kan misbruges til at slippe ud af dokumentationszonen.
3. **Faktisk WasmPolicyEngine Eksekvering**:
   - Forbinde `WasmPolicyEngine` direkte til adapter-evalueringen i `HarnessAdapter::evaluate_invocation`, så Wasmtime in-memory evaluering faktisk håndhæver Zero Ambient Authority i produktion.
4. **Struktureret Kommandokontrol & Forsvar mod Kommandokædning**:
   - Blokkere shell-kædning (`&&`, `;`, `|`) bag ufarlige kommandopræfikser (`git status && curl ...`), medmindre der foreligger aktiv opgaveautorisation.
5. **Typesikker Serde-deserialisering i WASM Security Boundary**:
   - Erstatte hjemmelavet JSON-strengparser i `gauntlet-policy-engine` med typesikker `serde` / `serde_json` med fail-closed semantik på malformed input.
6. **Udvidet Anti-Tamper & Fail-Closed Evidens**:
   - Inkludere `.agents/`, `.github/`, `CONTEXT.md` og `CODING_STANDARDS.md` i `DEFAULT_SCOPES`.
   - Verificere `policy_digest`, `config_digest` og `task_digest` i `verify_self_mutation`.
   - Sikre at `compute_digest_of_files` returnerer en `Result` og fejler lukket ved ulæselige filer.
   - Fjerne 16-tegns præfiks-tolerance i `drift.rs` til fordel for eksakt lighed.

## 📋 Acceptance Criteria
- [x] `resolve_task_contract` returnerer en eksplicit fejl når specificeret opgave ikke findes eller ingen opgave er aktiv, og falder aldrig tilbage på en anden opgave eller syntetisk kontrakt.
- [x] `execute_gauntlet_pipeline` fejler lukket med verdict `FAILED`, hvis opgavekontrakten mangler eller er ugyldig.
- [x] `WorkspaceRelativePath` er implementeret og afviser `..`, absolutte stier (`/`, `C:\`), UNC-stier og workspace escapes.
- [x] Alle harness adaptere (`Antigravity`, `Claude Code`, `Codex`) afviser path traversal angreb (fx `docs/../../.github/workflows/release.yml` og `/etc/passwd`) med `DENY` og exit code 1.
- [x] `HarnessAdapter::evaluate_invocation` eksekverer den indlejrede `WasmPolicyEngine` in-memory frem for in-process reference evaluator.
- [x] Kommandoer med kædningsoperatorer (`&&`, `;`, `|`) kan ikke omgås via hvidlistede præfikser (`git status`, `cargo test`) og afvises i read-only/uden aktiv opgave.
- [x] `gauntlet-policy-engine` deserialiserer requests og context typesikkert via `serde_json`, og afviser enhver syntaktisk fejl med `DENY` (grundkode 4037).
- [x] `DEFAULT_SCOPES` i manifest motoren inkluderer `.agents`, `.github`, `CONTEXT.md` og `CODING_STANDARDS.md`.
- [x] `verify_self_mutation` validerer `policy_digest`, `config_digest` og `task_digest` ud over kildefilsmanifestet.
- [x] `compute_digest_of_files` returnerer `Result<String, ManifestError>` og fejler hårdt hvis en fil ikke kan læses.
- [x] `digest_matches` i `drift.rs` kræver 100% eksakt streng-lighed uden præfiks-tolerance.
- [x] Ny adversarial regression testsuite (`tests/security_hardening_test.rs`) og udvidet paritetstestsuite beviser alle sikkerhedsinvarianter.
- [x] 100% grøn testsuite på tværs af workspacet (`cargo test --workspace`).

## 🚫 Must NOT
- Må IKKE redesigne den overordnede arkitektur (Harness -> Adapter -> CapabilityRequest -> WasmPolicyEngine -> Verification).
- Må IKKE introducere eksterne dæmoner eller runtime-sockets.
- Må IKKE tillade syntetiske opgaver (`default-task`) eller fail-open tilstande.
- Må IKKE udføre remote git publication (`git push`).
- Må IKKE slække på sikkerhedskriterier for at få eksisterende tests til at passere.

## 📝 Revisions
- 2026-09-07: Oprettet som aktiv opgave efter uafhængigt arkitekturreview (Task 015).
- 2026-09-07: Alle 13 acceptkriterier implementeret, verificeret med adversarial suite og forseglet (DONE).

## 🧪 Verifikation
- `cargo test -p xgauntlet-core --test security_hardening_test`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo fmt --check`
- `cargo run -p xgauntlet-cli -- check-spec -t 015-security-and-policy-boundary-hardening`
