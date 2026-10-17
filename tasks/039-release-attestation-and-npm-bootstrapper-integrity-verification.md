---
type: Task Package
title: "Task 039: Native Release Attestation and Bootstrapper Integrity Verification"
description: "Implementer ADR 0005 Sigstore attestation med SHA-pinned actions/attest for distributionsarkiver i CI og kryptografisk forankret verifikation i npm bootstrapperen"
status: todo
generated: { by: process:xgauntlet-task-init, at: "2026-10-17T11:36:00Z" }
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
1. Generere kryptografisk Sigstore OIDC attestering med den officielle SHA-pinnede `actions/attest` action for samtlige distribuerede release-arkiver (`.tar.gz` og `.zip`) i ét samlet kald via `subject-path`, bundet til commit SHA, workflow og repository OIDC identitet under mindst mulige workflow-rettigheder.
2. Distribuere det officielle attesteringsbundt (`attestation.bundle` fra actionens `bundle-path` output) som et downloadbart GitHub Release-aktiv sammen med arkiverne uden efterfølgende JSON-sammensyning eller ompakning.
3. Forankre bootstrapperens (`packages/cli/bin/xgauntlet.js`) integritetskontrol i en forud fastlagt trust policy med et **hårdkodet/betroet numerisk repository-ID** (må aldrig udledes dynamisk fra downloadkilden), der verificerer bundtet og arkivets digest mod Sigstore trusted roots før udpakning.
4. Etablere `check-attestation` i CLI'en med validering mod den definerede tillidspolitik (forventet numerisk repository-ID, repository-navn, workflow-sti, ref/commit, predicate-type og digest).

## 📋 Acceptance Criteria
- [ ] **AC1 (CI Multi-Subject Archive Attestation med SHA-pinned `actions/attest`)**:
  - `.github/workflows/release.yml` beriges med et dedikeret attesterings-job (`attest-binaries`) med snævre rettigheder: `permissions: id-token: write, attestations: write, artifact-metadata: write, contents: read`.
  - Anvender `actions/attest` fastlåst til en verificeret fuld commit-SHA (ikke et mutable tag).
  - Modtager alle fem distributionsarkiver fra `build-binaries` og attesterer dem samlet i ét `actions/attest`-kald via `subject-path`. Det resulterende Sigstore-bundt (`bundle-path`) overføres uændret som `attestation.bundle` til release-jobbet.
  - `create-github-release` afhænger eksplicit af attesteringen (`needs: [gate-check-release, build-binaries, attest-binaries]`) og vedhæfter `attestation.bundle` som release-aktiv sammen med de uændrede arkiver.
- [ ] **AC2 (Forud Forankret Repository-ID & Sigstore Verifikation i Bootstrapper)**:
  - `packages/cli/bin/xgauntlet.js` downloader `attestation.bundle` sammen med platform-arkivet.
  - Verificerer bundtets kryptografiske signatur mod Sigstores betroede rødder.
  - Validerer trust claims mod et **forud betroet, statisk forankret numerisk repository-ID** (må ALDRIG udledes dynamisk fra release-endpointet), samt forventet repository-navn (`rolfmadsen/xgauntlet`), release-workflow og commit/tag.
  - Matcher det downloadede arkivs SHA-256 digest mod bundtets attesterede subjects **før** udpakning.
- [ ] **AC3 (Fail-Closed Ved Mismatch eller Kompromittering)**: Hvis arkivets digest ikke findes blandt de attesterede subjects i bundtet, eller hvis attesteringen stammer fra et forkert repository-ID/workflow, afbryder bootstrapperen omgående med en fatal sikkerhedsfejl og exit-kode 1 uden at udpakke eller eksekvere koden. Dækkes af negative tests med gyldig attestering fra et fremmed repository-ID.
- [ ] **AC4 (CLI Subcommand `check-attestation` med Numerisk Repo-ID Trust Policy)**: CLI'en udvides med `xgauntlet check-attestation` i overensstemmelse med ADR 0005. Kommandoen kontrollerer ikke blot attestationens syntaks, men håndhæver en eksplicit tillidspolitik (validerer unikt numerisk repository-ID, repository-sti, workflow-sti, ref/tag, predicate-type og artefakt-digest mod Sigstore trusted roots).
- [ ] **AC5 (Eksplicit Adskillelse fra NPM Provenance)**: `npm publish --provenance` bevares for selve npm-pakken, men adskilles eksplicit fra attesteringen og integritetskontrollen af de distribuerede native arkiver.

## 🚫 Must NOT
- Må IKKE opfinde et custom JSON-samlingsformat for attestationer; skal anvende det officielle Sigstore bundle-output uændret fra `actions/attest`.
- Må IKKE udlede eller stole på et numerisk repository-ID hentet dynamisk fra den downloadkilde, hvis autenticitet skal verificeres.
- Må ALDRIG udpakke eller eksekvere en native binærfil hvis digest mismatch eller attesteringsfejl detekteres.
- Må IKKE tillade `create-github-release` at køre før `attest-binaries` er fuldført med et gyldigt attesteringsbundt.
- Må IKKE tildele `attestations: write` eller `id-token: write` globalt til hele workflowet, men kun snævert til det dedikerede attestation-job.
- Må IKKE genbygge eller ompakke arkiver efter attestationen er udført.

## 📝 Revisions
- 2026-10-17: Oprettet som backlogopgave på baggrund af ChatGPT second opinion review (P1 fund om tillidskæde, SHA-pinning, multi-subject bundle, uforanderligt repo-ID og DAG-afhængigheder).

## 🧪 Verifikation
- `npm test --workspaces`
- `cargo test -p xgauntlet-core --test evidence_manifest_test`
- `cargo run -p xgauntlet-cli -- check-spec -t 039`
