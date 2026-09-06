---
type: Task Package
title: "Task 004: Evidence & Workspace Manifest Engine"
description: "Implementere deterministisk Git-blob/tree OID workspace manifest (immune overfor CRLF/LF), self-mutation invarianten, Tier 1 verifikationsrapporten (verification-report.json og evidence.md) samt drift-kontrol i crates/xgauntlet-core"
status: active
generated: { by: process:xgauntlet-task-init, at: "2026-09-06T17:30:00Z" }
tags: [evidence, manifest, git-oid, crlf-immunity, self-mutation, drift, verification-report, adr-0005]
---

# Task 004: Evidence & Workspace Manifest Engine

**Status**: `DONE`
**Intent**: `🚀 NEW FEATURE`
**Oprettet**: `2026-09-06`

## 🎯 Formål
Etablere Evidence & Workspace Manifest Engine (`features/evidence`) i `crates/xgauntlet-core` jf. [spec.md](spec.md), [CONTEXT.md](CONTEXT.md) og [ADR 0005](docs/adr/0005-two-tier-verification-and-attestation-model.md):
1. Implementere en deterministisk, længdepræfikset SHA-256 Workspace Manifest beregning med Git-blob/tree OID hashing, som er immun overfor cross-platform CRLF/LF linjeskift-variationer.
2. Sikre robust sondring mellem tekstfiler og binære filer (via null-byte inspektion `0x00` i de første 8KB), så binære filer aldrig fejlagtigt CRLF-normaliseres.
3. Beregne rå worktree content digest for realtidsdetektion af lokal kildedrift og in-place ændringer.
4. Implementere strikt symlink escape-kontrol, der afviser symbolske links, der peger uden for workspace-roden.
5. Implementere Self-Mutation Invarianten (`verify_self_mutation`), som mekanisk sammenligner pre- og post-verifikationsmanifester og afviser kørslen, hvis kildekode, tests eller konfiguration er blevet ændret under gauntlet-kørslen.
6. Implementere Drift Detection (`verify_workspace_state_match`), der validerer aktuelt kildetræ, policy, konfiguration og opgavekontrakter mod en eksisterende verifikationsrapport.
7. Implementere Tier 1 verifikationsrapportmotor (`VerificationReportEngine`), der genererer og parser Schema v2 `verification-report.json` samt formaterer `evidence.md`.
8. Tilføje `check-evidence` kommando til `xgauntlet-cli` for øjeblikkelig drift-kontrol i terminalen.

## 📋 Acceptance Criteria
- [x] `crates/xgauntlet-core/src/features/evidence/models.rs` definerer Schema v2 modellerne: `VerificationVerdict`, `CheckStatus`, `ExecutionOrigin`, `AttestationStatus`, `TrustDecision`, `CheckSummary`, `VcsMetadata`, `WorkspaceState`, `ExecutionMetadata`, `TaskContractSummary` og `VerificationReport`.
- [x] `crates/xgauntlet-core/src/features/evidence/manifest.rs` beregner deterministisk `CanonicalWorkspaceManifest` med `source_manifest_digest` (Git-blob OID baseret og CRLF/LF immun) og `source_content_digest` (rå worktree hash).
- [x] Tekstfiler med `\r\n` (CRLF) og `\n` (LF) producerer det nøjagtigt samme `source_manifest_digest`.
- [x] Binære filer bevares uændret uden modifikation af null-bytes eller linjeskift.
- [x] Symlink traversal uden for workspace roden afvises med `ManifestError::WorkspaceEscape`.
- [x] `crates/xgauntlet-core/src/features/evidence/invariant.rs` håndhæver Self-Mutation Invarianten og detekterer tilføjede, fjernede og ændrede filer mellem pre- og post-manifest.
- [x] `crates/xgauntlet-core/src/features/evidence/drift.rs` detekterer kildedrift, policy-drift, config-drift og task-drift mod rapporten med 16-tegns præfikstolerance.
- [x] `crates/xgauntlet-core/src/features/evidence/report.rs` håndterer serialisering og deserialisering af `verification-report.json` samt formatering af `evidence.md`.
- [x] `xgauntlet-cli` indeholder en fungerende `check-evidence` subcommand (`--workspace`, `--json`).
- [x] 100% grøn testsuite i `crates/xgauntlet-core/tests/evidence_manifest_test.rs` og samtlige workspace checks.

## 🚫 Must NOT
- Må IKKE introducere hardcodede hemmelige nøgler eller lokal HMAC-SHA256 signering jf. ADR 0005 (lokal verifikation er Tier 1 drift-kontrol, ikke en kryptografisk tillidsgrænse mod lokal bruger).
- Må IKKE fejle åbent ved symlink escapes eller korrupte/ulæselige filer.
- Må IKKE ændre eller beskadige eksisterende binære filer ved CRLF-normalisering.
- Må IKKE foretage utilsigtede git remote operationer (`git push`).
- Må IKKE introducere eksterne C-biblioteker eller runtime dæmoner.

## 📝 Revisions
- 2026-09-06: Oprettet som ACTIVE task for Evidence & Workspace Manifest Engine (Task 004).

## 🧪 Verifikation
- `cargo test -p xgauntlet-core --test evidence_manifest_test`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo fmt --check`
- `cargo run -p xgauntlet-cli -- check-spec -t 004-evidence-and-manifest-engine`
- `cargo run -p xgauntlet-cli -- check-evidence`
