---
type: Task Package
title: "Task 039: Native Release Attestation and Bootstrapper Integrity Verification"
description: "Implementer ADR 0005 Sigstore attestation med SHA-pinned actions/attest for distributionsarkiver i CI og kryptografisk forankret verifikation i npm bootstrapperen"
status: todo
generated: { by: process:xgauntlet-task-init, at: "2026-10-17T11:31:00Z" }
tags: [attestation, security, sigstore, in-toto, npm, bootstrapper, adr-0005]
scope:
  - ".github/workflows/release.yml"
  - "packages/cli/bin/xgauntlet.js"
  - "packages/cli/package.json"
  - "packages/cli/package-lock.json"
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
2. Etablere en deterministisk pipeline-arbejdsgang der indsamler attestation-bundles fra runneren (`${RUNNER_TEMP}/created_attestation_paths.txt`) og uploader `attestation-bundle.json` som et downloadbart GitHub Release-aktiv sammen med arkiverne uden efterfølgende ompakning.
3. Forankre bootstrapperens (`packages/cli/bin/xgauntlet.js`) integritetskontrol i en kryptografisk validering af bundtet og arkivets digest mod GitHub/Sigstore trusted roots før udpakning, bundet til numerisk repository-ID for at forhindre genbrug af repository-navn.
4. Etablere `check-attestation` i CLI'en med validering mod en defineret tillidspolitik (forventet numerisk repository-ID, repository-navn, workflow-sti, ref/commit, predicate-type og digest).

## 📋 Acceptance Criteria
- [ ] **AC1 (CI Distribution Archive Attestation med SHA-pinned `actions/attest` & Asset Upload)**:
  - `.github/workflows/release.yml` beriges med et dedikeret attesterings-job med mindst mulige rettigheder: `permissions: id-token: write, attestations: write, artifact-metadata: write, contents: read`.
  - Anvender `actions/attest` fastlåst til en verificeret fuld commit-SHA (ikke et mutable tag).
  - Modtager distributionsarkiverne fra `build-binaries`, attesterer dem, indsamler de genererede bundle-filer fra `${RUNNER_TEMP}/created_attestation_paths.txt`, samler dem i et autoritativt `attestation-bundle.json` workflow-artefakt, og videregiver dem til `create-github-release` som vedhæfter bundtet som release-aktiv sammen med de uændrede arkiver.
- [ ] **AC2 (Attesterings-forankret Arkiv-kontrol i Bootstrapper med Repository-ID)**: `packages/cli/bin/xgauntlet.js` downloader attesteringsbundtet sammen med det valgte platform-arkiv, verificerer dets kryptografiske signatur mod Sigstores betroede rødder, og validerer trust claims mod det forventede numeriske repository-ID, repository-navn (`rolfmadsen/xgauntlet`), release-workflow og tag/commit, samt matcher arkivets SHA-256 digest før udpakning.
- [ ] **AC3 (Fail-Closed Ved Mismatch eller Kompromittering)**: Hvis det downloadede arkiv ikke matcher den verificerede digest, eller hvis attesteringen mangler, er ugyldig eller stammer fra et forkert repository-ID, afbryder bootstrapperen omgående med en fatal sikkerhedsfejl og exit-kode 1 uden at udpakke eller eksekvere koden.
- [ ] **AC4 (CLI Subcommand `check-attestation` med Numerisk Repo-ID Trust Policy)**: CLI'en udvides med `xgauntlet check-attestation` i overensstemmelse med ADR 0005. Kommandoen kontrollerer ikke blot attestationens syntaks, men håndhæver en eksplicit tillidspolitik (validerer unikt numerisk repository-ID, repository-sti, workflow-sti, ref/tag, predicate-type og artefakt-digest mod Sigstore trusted roots).
- [ ] **AC5 (Eksplicit Adskillelse fra NPM Provenance)**: `npm publish --provenance` bevares for selve npm-pakken, men adskilles eksplicit fra attesteringen og integritetskontrollen af de distribuerede native arkiver.

## 🚫 Must NOT
- Må IKKE stole på en ubekræftet checksum downloadet fra samme kompromitterbare GitHub release uden uafhængig kryptografisk forankring og trust policy validering mod numerisk repository-ID (ADR 0005).
- Må ALDRIG udpakke eller eksekvere en native binærfil hvis digest mismatch eller attesteringsfejl detekteres.
- Må IKKE tildele `attestations: write` eller `id-token: write` globalt til hele workflowet, men kun snævert til det dedikerede attestation-job.
- Må IKKE genbygge eller ompakke arkiver efter attestationen er udført.

## 📝 Revisions
- 2026-10-17: Oprettet som backlogopgave på baggrund af ChatGPT second opinion review (P1 fund om tillidskæde, SHA-pinning, scoped permissions, bundle-distribution og numerisk repository-ID kontrol).

## 🧪 Verifikation
- `npm test --workspaces`
- `cargo test -p xgauntlet-core --test evidence_manifest_test`
- `cargo run -p xgauntlet-cli -- check-spec -t 039`
