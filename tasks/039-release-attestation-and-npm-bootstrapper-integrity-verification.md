---
type: Task Package
title: "Task 039: Native Release Attestation and Bootstrapper Integrity Verification"
description: "Implementer ADR 0005 Sigstore attestation med actions/attest@v4 for distributionsarkiver i CI og kryptografisk forankret verifikation i npm bootstrapperen"
status: todo
generated: { by: process:xgauntlet-task-init, at: "2026-10-17T11:18:00Z" }
tags: [attestation, security, sigstore, in-toto, npm, bootstrapper, adr-0005]
scope:
  - ".github/workflows/release.yml"
  - "packages/cli/bin/xgauntlet.js"
  - "packages/cli/test/**"
  - "crates/xgauntlet-core/src/features/evidence/**"
  - "crates/xgauntlet-cli/src/**"
test_refs:
  - "packages/cli/test/launcher.test.js"
  - "crates/xgauntlet-core/tests/evidence_manifest_test.rs"
---

# Task 039: Native Release Attestation and Bootstrapper Integrity Verification

**Status**: `TODO`  
**Intent**: `🚀 NEW FEATURE` & `🛡️ SECURITY`  
**Oprettet**: `2026-10-17`  

## 🎯 Formål
Fuld implementering af ADR 0005's Tier 2 tillidskæde for udgivne release-artefakter:
1. Generere kryptografisk Sigstore OIDC attestering med den officielle `actions/attest@v4` action for samtlige distribuerede release-arkiver (`.tar.gz` og `.zip`), bundet til commit SHA, workflow og repository OIDC identitet.
2. Forankre bootstrapperens (`packages/cli/bin/xgauntlet.js`) integritetskontrol i en verificeret digest før udpakning, så kompromitterede downloads fra samme release-kilde afvises.
3. Etablere `check-attestation` i CLI'en med validering mod en defineret tillidspolitik (forventet repository, workflow, ref/commit, predicate-type og digest).

## 📋 Acceptance Criteria
- [ ] **AC1 (CI Distribution Archive Attestation med `actions/attest@v4`)**: `.github/workflows/release.yml` beriges med `actions/attest@v4` for samtlige publicerede distributionsarkiver (`xgauntlet-linux-x64.tar.gz`, `xgauntlet-linux-arm64.tar.gz`, `xgauntlet-darwin-arm64.tar.gz`, `xgauntlet-darwin-x64.tar.gz`, `xgauntlet-win32-x64.zip`), der binder dem til commit SHA og repository OIDC identitet.
- [ ] **AC2 (Attesterings-forankret Arkiv-kontrol i Bootstrapper)**: `packages/cli/bin/xgauntlet.js` verificerer det downloadede arkivs SHA-256 digest mod et autoritativt, verificeret digest-register før udpakning.
- [ ] **AC3 (Fail-Closed Ved Mismatch eller Kompromittering)**: Hvis det downloadede arkiv ikke matcher den verificerede digest, eller hvis digesten ikke kan valideres, afbryder bootstrapperen omgående med en fatal sikkerhedsfejl og exit-kode 1 uden at udpakke eller eksekvere koden.
- [ ] **AC4 (CLI Subcommand `check-attestation` med Trust Policy)**: CLI'en udvides med `xgauntlet check-attestation` i overensstemmelse med ADR 0005. Kommandoen kontrollerer ikke blot attestationens syntaks, men håndhæver en eksplicit tillidspolitik (validerer repository, workflow-sti, ref/tag, predicate-type og artefakt-digest mod Sigstore trusted roots).
- [ ] **AC5 (Eksplicit Adskillelse fra NPM Provenance)**: `npm publish --provenance` bevares for selve npm-pakken, men adskilles eksplicit fra attesteringen og integritetskontrollen af de distribuerede native arkiver.

## 🚫 Must NOT
- Må IKKE stole på en ubekræftet checksum downloadet fra samme kompromitterbare GitHub release uden uafhængig kryptografisk forankring eller trust policy (ADR 0005).
- Må ALDRIG udpakke eller eksekvere en native binærfil hvis digest mismatch eller attesteringsfejl detekteres.

## 📝 Revisions
- 2026-10-17: Oprettet som backlogopgave på baggrund af ChatGPT second opinion review (P1 fund om tillidskæde, `actions/attest@v4` og bootstrapper supply-chain).

## 🧪 Verifikation
- `npm test --workspaces`
- `cargo test -p xgauntlet-core --test evidence_manifest_test`
- `cargo run -p xgauntlet-cli -- check-spec -t 039`
