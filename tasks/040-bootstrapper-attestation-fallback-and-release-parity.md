---
type: Task Package
title: "Task 040: Bootstrapper Attestation Fallback and Dual-Asset Release Parity"
description: "Implementer transparent attestation bundle fallback (.bundle og .json) i bootstrapperen og dual-asset udgivelse i GitHub Actions for gnidningsfri npm installation"
status: active
generated: { by: process:xgauntlet-task-init, at: "2026-10-17T15:06:28Z" }
tags: [task-lifecycle, intent, scaffolding, rust]
scope:
  - "crates/xgauntlet-cli/src/**"
  - "packages/cli/bin/xgauntlet.js"
  - "packages/cli/test/**"
  - ".github/workflows/release.yml"
test_refs:
  - "crates/xgauntlet-cli/tests/check_attestation_test.rs"
  - "packages/cli/test/attestation.test.js"
---

# Task 040: Bootstrapper Attestation Fallback and Dual-Asset Release Parity

**Status**: `ACTIVE`
**Intent**: `🐛 BUG FIX`
**Oprettet**: `2026-10-17`

## 🎯 Formål
Fuldstændig eliminering af 404-fejl under `npm install -g xgauntlet` og første kørsel:
1. Gøre bootstrapperen (`packages/cli/bin/xgauntlet.js`) resilient over for både `attestation.bundle` og `attestation.json` via transparent fallback.
2. Forhindre at mislykkede downloads efterlader tomme cachemapper i `~/.cache/xgauntlet/vX.Y.Z/`.
3. Sikre at release-pipelinen (`.github/workflows/release.yml`) uploader både `attestation.bundle` og `attestation.json` som release-aktiver.

## 📋 Acceptance Criteria
- [ ] **AC1 (Dual-Candidate Attestation Fetching)**: `packages/cli/bin/xgauntlet.js` implementerer en dedikeret `fetchAttestationBundle`-hjælper (eller ækvivalent), der prøver `attestation.bundle` og transparent falder tilbage på `attestation.json` (og omvendt), så installationen lykkes uanset hvilket filnavn GitHub Release serverer.
- [ ] **AC2 (Ingen tomme cache-mapper ved downloadfejl)**: `downloadAndCacheBinary` fjerner eller rydder op i målmappen (`cacheDir`), hvis download eller verifikation fejler, så ufuldstændige downloads ikke forurener cache-træet.
- [ ] **AC3 (Dual-Asset Release i CI)**: `.github/workflows/release.yml` klargør og uploader det attesterede bundt som både `attestation.bundle` og `attestation.json` i `attest-binaries` og `create-github-release`.
- [ ] **AC4 (Regressions- og Paritetstests)**: `packages/cli/test/attestation.test.js` udvides med tests for fallback-håndteringen og workflow-filens dual-asset konfiguration.

## 🚫 Must NOT
- Må IKKE fjerne eller svække Sigstore kryptografisk verifikation eller trust policy kravene (repo ID, workflow, ref).
- Må IKKE bryde eksisterende arkitektur-invarianter eller introducere baggrunds-dæmoner.
- Må IKKE udføre remote publication handlinger (`git push`).

## 📝 Revisions
- 2026-10-17: Oprettet Task 040 specifikation for attestation fallback og dual-asset release paritet.

## 🧪 Verifikation
- `npm --prefix packages/cli test`
- `cargo run -p xgauntlet-cli -- check-spec -t 040`
