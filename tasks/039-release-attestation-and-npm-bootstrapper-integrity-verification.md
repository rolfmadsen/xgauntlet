---
type: Task Package
title: "Task 039: Native Release Attestation and Bootstrapper Integrity Verification"
description: "Implementer ADR 0005 Sigstore attestation for native binærfiler i CI og kryptografisk forankret digest-verifikation i npm bootstrapperen"
status: todo
generated: { by: process:xgauntlet-task-init, at: "2026-10-17T11:08:00Z" }
tags: [attestation, security, sigstore, in-toto, npm, bootstrapper, adr-0005]
scope:
  - ".github/workflows/release.yml"
  - "packages/cli/bin/xgauntlet.js"
  - "crates/xgauntlet-core/src/features/evidence/**"
  - "crates/xgauntlet-cli/src/**"
test_refs:
  - "packages/cli/test/bootstrapper_integrity.test.js"
  - "crates/xgauntlet-core/tests/evidence_manifest_test.rs"
---

# Task 039: Native Release Attestation and Bootstrapper Integrity Verification

**Status**: `TODO`  
**Intent**: `🚀 NEW FEATURE` & `🛡️ SECURITY`  
**Oprettet**: `2026-10-17`  

## 🎯 Formål
Fuld implementering af ADR 0005's tillidskæde for release-artefakter:
1. Generere kryptografisk Sigstore OIDC attestering (`actions/attest` / in-toto / DSSE) for samtlige kompilerede native binærfiler i CI.
2. Forankre bootstrapperens (`packages/cli/bin/xgauntlet.js`) integritetskontrol i den betroede attestering frem for uverificerede downloads fra samme kompromitterbare kilde.
3. Sikre at release-processen håndhæver tre-dimensionel tillidsbeslutning (`verification_result == PASSED AND attestation_status == VALID AND trust_decision == ACCEPTED`).

## 📋 Acceptance Criteria
- [ ] **AC1 (CI Native Artifact Attestation)**: `.github/workflows/release.yml` beriges med officiel `actions/attest-build-provenance` for samtlige byggede platform-binærer (Linux x64/arm64, macOS arm64/x64, Windows x64), der binder dem til commit SHA og repository OIDC identitet.
- [ ] **AC2 (Attesterings-forankret Digest i Bootstrapper)**: `packages/cli/bin/xgauntlet.js` verificerer den downloadede binærs SHA-256 digest mod en autoritativ ledger eller attesteringsbundt før udpakning og eksekvering.
- [ ] **AC3 (Fail-Closed Ved Mismatch eller Kompromittering)**: Hvis det downloadede arkiv ikke matcher den attesterede digest, eller hvis digesten ikke kan verificeres, afbryder bootstrapperen omgående med en fatal sikkerhedsfejl og exit-kode 1 uden at eksekvere binæren.
- [ ] **AC4 (CLI Subcommand `check-attestation`)**: CLI'en udvides med `xgauntlet check-attestation` i overensstemmelse med ADR 0005, der validerer in-toto attestationer mod Sigstore trusted roots.
- [ ] **AC5 (NPM Provenance Adskillelse)**: `npm publish --provenance` bevares for selve npm-pakken, men adskilles eksplicit fra den uafhængige attestering og verifikation af native binaries.

## 🚫 Must NOT
- Må IKKE stole på en ubekræftet checksum downloadet fra samme kompromitterbare GitHub release uden uafhængig kryptografisk forankring (ADR 0005).
- Må ALDRIG eksekvere en native binærfil hvis digest mismatch eller attesteringsfejl detekteres.

## 📝 Revisions
- 2026-10-17: Oprettet som backlogopgave på baggrund af ChatGPT second opinion review (P1 fund om tillidskæde og bootstrapper supply-chain).

## 🧪 Verifikation
- `npm test --workspaces`
- `cargo test -p xgauntlet-core --test evidence_manifest_test`
- `cargo run -p xgauntlet-cli -- check-spec -t 039`
