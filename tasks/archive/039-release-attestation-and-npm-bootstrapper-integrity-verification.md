---
type: Task Package
title: "Task 039: Native Release Attestation and Bootstrapper Integrity Verification"
description: "Implementer ADR 0005 Sigstore attestation med SHA-pinned actions/attest for distributionsarkiver i CI og kryptografisk forankret verifikation i npm bootstrapperen"
status: done
generated: { by: process:xgauntlet-task-init, at: "2026-10-17T11:36:00Z" }
tags: [attestation, security, sigstore, in-toto, npm, bootstrapper, adr-0005]
scope:
  - ".github/workflows/release.yml"
  - "package-lock.json"
  - "packages/cli/bin/xgauntlet.js"
  - "packages/cli/package.json"
  - "packages/cli/package-lock.json"
  - "packages/cli/test/**"
  - "crates/xgauntlet-core/src/features/evidence/**"
  - "crates/xgauntlet-cli/src/**"
  - "crates/xgauntlet-cli/tests/**"
test_refs:
  - "packages/cli/test/attestation.test.js"
  - "packages/cli/test/launcher.test.js"
  - "crates/xgauntlet-core/tests/evidence_manifest_test.rs"
  - "crates/xgauntlet-cli/tests/check_attestation_test.rs"
---

# Task 039: Native Release Attestation and Bootstrapper Integrity Verification

**Status**: `DONE`  
**Intent**: `🚀 NEW FEATURE` & `🛡️ SECURITY`  
**Oprettet**: `2026-10-17`  

## 🎯 Formål
Fuld implementering af ADR 0005's Tier 2 tillidskæde for udgivne release-artefakter:
1. Generere kryptografisk Sigstore OIDC attestering med den officielle SHA-pinnede `actions/attest` action for samtlige distribuerede release-arkiver (`.tar.gz` og `.zip`) i ét samlet kald via `subject-path`, bundet til commit SHA, workflow og repository OIDC identitet under mindst mulige workflow-rettigheder.
2. Distribuere det officielle attesteringsbundt (`attestation.bundle` fra actionens `bundle-path` output) som et downloadbart GitHub Release-aktiv sammen med arkiverne uden efterfølgende JSON-sammensyning eller ompakning.
3. Forankre bootstrapperens (`packages/cli/bin/xgauntlet.js`) integritetskontrol i en forud fastlagt trust policy med et **hårdkodet/betroet numerisk repository-ID** (må aldrig udledes dynamisk fra downloadkilden), der verificerer bundtet og arkivets digest mod Sigstore trusted roots før udpakning.
4. Etablere `check-attestation` i CLI'en med validering mod den definerede tillidspolitik (forventet numerisk repository-ID, repository-navn, workflow-sti, ref/commit, predicate-type og digest).

## 📋 Acceptance Criteria
- [x] **AC1 (CI Multi-Subject Archive Attestation & Workflow Artifact Transfer)**:
  - `.github/workflows/release.yml` beriges med et dedikeret attesterings-job (`attest-binaries`) med snævre rettigheder: `permissions: id-token: write, attestations: write, artifact-metadata: write, contents: read`.
  - Anvender `actions/attest` fastlåst til en verificeret fuld commit-SHA (ikke et mutable tag).
  - Eksplicit overførsel af distributionsfiler mellem GitHub Actions runners:
    1. `build-binaries`: uploader de fem færdige distributionsarkiver (`.tar.gz` og `.zip`) som workflow-artefakter via SHA-pinned `actions/upload-artifact`.
    2. `attest-binaries` (`needs: [gate-check-release, build-binaries]`): downloader netop disse arkiver via SHA-pinned `actions/download-artifact`, attesterer dem samlet i ét `actions/attest`-kald via `subject-path`, og uploader den uændrede bundle-fil fra `bundle-path`-outputtet som et workflow-artefakt via SHA-pinned `actions/upload-artifact`.
    3. `create-github-release` (`needs: [gate-check-release, build-binaries, attest-binaries]`): downloader både arkiverne og bundle-filen via `actions/download-artifact` og publicerer dem som release-aktiver uden genbygning eller ompakning.
- [x] **AC2 (Officiel `sigstore` Verifikator & Forud Forankret Repository-ID i Bootstrapper)**:
  - `packages/cli` anvender den officielle, vedligeholdte npm-pakke `sigstore` (der understøtter Sigstore-bundles og Node.js >= 18.17.0). Bootstrapperen må IKKE rulle sin egen kryptografiske bundle-verifikation fra bunden.
  - `packages/cli/bin/xgauntlet.js` downloader `attestation.bundle` sammen med platform-arkivet.
  - Verificerer bundtets kryptografiske signatur mod Sigstores betroede rødder via `sigstore.verify`.
  - Validerer certifikatets claims mod en streng, forud fastlagt trust policy:
    - Et **forud betroet, statisk forankret numerisk repository-ID** (`EXPECTED_REPOSITORY_ID` hårdkodet i koden; må ALDRIG udledes dynamisk fra download-URL eller release-endpoint).
    - Forventet repository-navn (`rolfmadsen/xgauntlet`).
    - Forventet release-workflow (`.github/workflows/release.yml`).
    - Forventet ref/tag/commit.
  - Matcher det downloadede arkivs beregnede SHA-256 digest mod det relevante subject i bundtet **før** udpakning.
- [x] **AC3 (Fail-Closed Ved Mismatch eller Ubetroet Identitet)**:
  - Hvis arkivets digest ikke findes blandt de attesterede subjects i bundtet, eller hvis attesteringen stammer fra et forkert repository-ID/workflow/ref, afbryder bootstrapperen omgående med en fatal sikkerhedsfejl og exit-kode 1 uden at udpakke eller eksekvere koden.
  - Dækkes af automatiserede negative tests:
    - Manipuleret arkiv-payload (digest mismatch).
    - Kryptografisk gyldigt Sigstore bundle udstedt til et andet repository-ID (afvises pga. repository-ID policy check, selvom signaturen er gyldig).
    - Kryptografisk gyldigt bundle udstedt af et andet workflow (afvises pga. workflow policy check).
    - Ugyldig eller udløbet signatur (afvises af `sigstore.verify`).
- [x] **AC4 (CLI Subcommand `check-attestation` med Numerisk Repo-ID Trust Policy)**: CLI'en udvides med `xgauntlet check-attestation` i overensstemmelse med ADR 0005. Kommandoen kontrollerer ikke blot attestationens syntaks, men håndhæver en eksplicit tillidspolitik (validerer unikt numerisk repository-ID, repository-sti, workflow-sti, ref/tag, predicate-type og artefakt-digest mod Sigstore trusted roots).
- [x] **AC5 (Eksplicit Adskillelse fra NPM Provenance)**: `npm publish --provenance` bevares for selve npm-pakken, men adskilles eksplicit fra attesteringen og integritetskontrollen af de distribuerede native arkiver.

## 🚫 Must NOT
- Må IKKE opfinde et custom JSON-samlingsformat for attestationer; skal anvende det officielle Sigstore bundle-output uændret fra `actions/attest`.
- Må IKKE implementere kryptografisk bundle-verifikation fra bunden i Node-bootstrapperen; skal anvende den officielle, vedligeholdte `sigstore` bibliotekspakke.
- Må IKKE udlede eller stole på et numerisk repository-ID hentet dynamisk fra den downloadkilde, hvis autenticitet skal verificeres.
- Må ALDRIG udpakke eller eksekvere en native binærfil hvis digest mismatch eller attesteringsfejl detekteres.
- Må IKKE tillade `create-github-release` at køre før `attest-binaries` er fuldført med et gyldigt attesteringsbundt.
- Må IKKE tildele `attestations: write` eller `id-token: write` globalt til hele workflowet, men kun snævert til det dedikerede attestation-job.
- Må IKKE genbygge eller ompakke arkiver efter attestationen er udført.

## 📝 Revisions
- 2026-10-17: Oprettet som backlogopgave på baggrund af ChatGPT second opinion review (P1 fund om tillidskæde, SHA-pinning, multi-subject bundle, uforanderligt repo-ID og DAG-afhængigheder).
- 2026-10-17: Præciseret eksplicit workflow-artefaktoverførsel (upload/download-artifact) og officiel `sigstore` verifikatorpakke med trust policy validering.
- 2026-10-17: Implementeret AC1-AC5 (SHA-pinned release.yml, verifyReleaseArchive med statisk forankret repo ID i bootstrapper, og check-attestation subcommand i CLI).

## 🧪 Verifikation
- `npm test --workspaces`
- `cargo test -p xgauntlet-core --test evidence_manifest_test`
- `cargo run -p xgauntlet-cli -- check-spec -t 039`
