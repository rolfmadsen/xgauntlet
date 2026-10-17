---
type: Task Package
title: "Task 039: Native Release Attestation and Bootstrapper Integrity Verification"
description: "Implementer ADR 0005 Sigstore attestation med SHA-pinned actions/attest for distributionsarkiver i CI og kryptografisk forankret verifikation i npm bootstrapperen"
status: todo
generated: { by: process:xgauntlet-task-init, at: "2026-10-17T11:22:00Z" }
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
1. Generere kryptografisk Sigstore OIDC attestering med den officielle SHA-pinnede `actions/attest` action for samtlige distribuerede release-arkiver (`.tar.gz` og `.zip`), bundet til commit SHA, workflow og repository OIDC identitet under mindst mulige workflow-rettigheder.
2. Distribuere attesteringsbundtet (`attestation-bundle.json`) som release-aktiv og forankre bootstrapperens (`packages/cli/bin/xgauntlet.js`) integritetskontrol i en kryptografisk validering af bundtet og arkivets digest mod GitHub/Sigstore trusted roots før udpakning.
3. Etablere `check-attestation` i CLI'en med validering mod en defineret tillidspolitik (forventet repository, workflow, ref/commit, predicate-type og digest).

## 📋 Acceptance Criteria
- [ ] **AC1 (CI Distribution Archive Attestation med SHA-pinned `actions/attest`)**:
  - `.github/workflows/release.yml` beriges med et dedikeret attesterings-job med mindst mulige rettigheder: `permissions: id-token: write, attestations: write, artifact-metadata: write, contents: read`.
  - Anvender `actions/attest` fastlåst til en verificeret fuld commit-SHA (ikke et mutable tag).
  - Attesterer samtlige publicerede distributionsarkiver (`xgauntlet-linux-x64.tar.gz`, `xgauntlet-linux-arm64.tar.gz`, `xgauntlet-darwin-arm64.tar.gz`, `xgauntlet-darwin-x64.tar.gz`, `xgauntlet-win32-x64.zip`) og publicerer attesteringsbundtet som et downloadbart release-aktiv.
- [ ] **AC2 (Attesterings-forankret Arkiv-kontrol i Bootstrapper)**: `packages/cli/bin/xgauntlet.js` downloader attesteringsbundtet, verificerer dets kryptografiske signatur mod Sigstores betroede rødder, validerer trust claims (repo `rolfmadsen/xgauntlet`, release-workflow, tag/ref) og matcher det downloadede arkivs SHA-256 digest før udpakning.
- [ ] **AC3 (Fail-Closed Ved Mismatch eller Kompromittering)**: Hvis det downloadede arkiv ikke matcher den verificerede digest, eller hvis attesteringen mangler, er ugyldig eller stammer fra et uautoriseret workflow/repository, afbryder bootstrapperen omgående med en fatal sikkerhedsfejl og exit-kode 1 uden at udpakke eller eksekvere koden.
- [ ] **AC4 (CLI Subcommand `check-attestation` med Trust Policy)**: CLI'en udvides med `xgauntlet check-attestation` i overensstemmelse med ADR 0005. Kommandoen kontrollerer ikke blot attestationens syntaks, men håndhæver en eksplicit tillidspolitik (validerer repository, workflow-sti, ref/tag, predicate-type og artefakt-digest mod Sigstore trusted roots).
- [ ] **AC5 (Eksplicit Adskillelse fra NPM Provenance)**: `npm publish --provenance` bevares for selve npm-pakken, men adskilles eksplicit fra attesteringen og integritetskontrollen af de distribuerede native arkiver.

## 🚫 Must NOT
- Må IKKE stole på en ubekræftet checksum downloadet fra samme kompromitterbare GitHub release uden uafhængig kryptografisk forankring og trust policy validering (ADR 0005).
- Må ALDRIG udpakke eller eksekvere en native binærfil hvis digest mismatch eller attesteringsfejl detekteres.
- Må IKKE tildele `attestations: write` eller `id-token: write` globalt til hele workflowet, men kun snævert til det dedikerede attestation-job.

## 📝 Revisions
- 2026-10-17: Oprettet som backlogopgave på baggrund af ChatGPT second opinion review (P1 fund om tillidskæde, SHA-pinning, scoped permissions, og attesteringsbundt-distribution).

## 🧪 Verifikation
- `npm test --workspaces`
- `cargo test -p xgauntlet-core --test evidence_manifest_test`
- `cargo run -p xgauntlet-cli -- check-spec -t 039`
