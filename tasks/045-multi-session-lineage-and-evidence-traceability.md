---
type: Task Package
title: "Task 045: Multi-Session Lineage & Evidence Traceability"
description: "Udvide Two-Tier Evidensmodellen med en kryptografisk, append-only Session Lineage Trail i verification-report.json og evidence.md"
status: active
generated: { by: process:antigravity-task-init, at: "2026-10-25T11:25:00Z" }
tags: [evidence, multi-session, lineage, traceability, two-tier-model, adr-0005, audit-trail]
scope: ["crates/xgauntlet-core/src/features/evidence/**", "crates/xgauntlet-core/src/features/checkpoint/**", "crates/xgauntlet-core/src/features/gauntlet/**", "crates/xgauntlet-core/src/lib.rs", "crates/xgauntlet-cli/**"]
test_refs: ["crates/xgauntlet-core/tests/session_lineage_test.rs"]
---

# Task 045: Multi-Session Lineage & Evidence Traceability

**Status**: `ACTIVE`  
**Intent**: `🚀 NEW FEATURE`  
**Oprettet**: `2026-10-25`  

## 🎯 Formål
Formålet med denne opgave er at udvide xGauntlets **Two-Tier Verification & Attestation Model ([ADR 0005](file:///home/rolfmadsen/Github/xgauntlet/docs/adr/0005-two-tier-verification-and-attestation-model.md))** med uafviselig sporbarhed på tværs af flere sessioner (**Session Lineage**).

I takt med at opgaver afvikles i isolerede, rene sessioner (f.eks. *System Architect* i `SPEC`-fasen, *TDD Craftsman* i `IMPLEMENTATION`-fasen, og *Code Reviewer* i `AUDIT`-fasen), er der behov for et ubrudt revisionsspor. Dette spor skal dokumentere præcis hvilke AI-roller og sessioner, der har bidraget til kildekoden, hvilke lokale checkpoints der blev forseglet, og at invariant-beviserne er bevaret uden manipulation.

Opgaven etablerer en append-only kryptografisk `SessionLineageTrail`, som integreres direkte i `verification-report.json` og visualiseres i `evidence.md`.

## 📋 Acceptance Criteria
- [ ] **AC1 (Session Lineage Datamodel)**:
  - Oprette modulet `crates/xgauntlet-core/src/features/evidence/lineage.rs`:
    - `SessionLineageEntry`:
      - `session_nonce: String`: Unik sessions-identifikator.
      - `role: String`: Den aktive AI-ingeniørrolle (f.eks. "System Architect", "TDD Craftsman", "Code Reviewer").
      - `phase: String`: Fasen ved registrering (`SPEC`, `RED`, `GREEN`, `REFACTOR`, `AUDIT`, `DONE`).
      - `commit_oid: String`: Det lokale Git commit SHA for checkpointet.
      - `timestamp: String`: ISO-8601 tidsstempel.
      - `digest: String`: SHA-256 hash-chaining af forrige digest + aktuelle tilstand for tamper-evident verifikation.
    - `SessionLineageTrail`: Vektor af `SessionLineageEntry` med metoder til append, serialisering og integritetskontrol.
- [ ] **AC2 (Integration i Checkpoint Engine)**:
  - Opdatere `crates/xgauntlet-core/src/features/checkpoint/engine.rs`:
    - Ved hvert succesfuldt fase-checkpoint (`run_checkpoint`) tilføjes automatisk et nyt `SessionLineageEntry` til opgavens evidensrapport.
    - Sikre at commit OID, rolle og fase fanges deterministisk.
- [ ] **AC3 (Tamper-Evident Hash Chaining)**:
  - Implementere kædet integritetskontrol: Hvert lineage-stempel hasher dataene fra det forrige stempel (Genesis $\to$ Red $\to$ Green $\to$ Refactor $\to$ Done), så manipulation af historiske sessionsposter afvises af `xgauntlet check-evidence`.
- [ ] **AC4 (Evidence Markdown Visualisering)**:
  - Opdatere rapportgenereringen i `evidence.md`:
    - Indsætte en sektion: `### 📜 Multi-Session Audit Lineage` med en Markdown-tabel over opgavens samlede livscyklus:
      `| Session | Rolle | Fase | Commit | Tidsstempel | Status |`
- [ ] **AC5 (CLI Undersøgelse & Formatering)**:
  - Udvide CLI med `xgauntlet evidence lineage [-t <task>]` til hurtig terminal-inspektion af en opgaves sessionshistorik.
  - Støtte `--json` flag for maskinel pipeline-integration.
- [ ] **AC6 (Testsuite & Verifikation)**:
  - Oprette `crates/xgauntlet-core/tests/session_lineage_test.rs` med dækning af:
    - Oprettelse og append af lineage-poster.
    - Verifikation af hash-chaining integritet.
    - Afvisning ved manipuleret sessionshistorik.
  - 100% grøn testsuite (`cargo test --workspace`).
  - Verificere at `cargo run -p xgauntlet-cli -- check-spec -t 045` validerer med 0 fejl.

## 🚫 Must NOT
- Må IKKE bryde bagudkompatibilitet med eksisterende `verification-report.json` filer; lineage-feltet skal være valgfrit (`Option<SessionLineageTrail>`) ved deserialisering af ældre rapporter.
- Må IKKE udføre remote netværkskald eller afhænge af eksterne attestationstjenester lokalt; al chaining skal være matematisk deterministisk på den lokale maskine jf. ADR 0005.

## 📝 Revisions
- 2026-10-25: Task oprettet for at sikre uafviselig sporbarhed på tværs af flersessions-opgaver.
- 2026-10-25: SPEC-fase initialiseret; domænebegreber i CONTEXT.md og spec.md synkroniseret.

## 🧪 Verifikation
- `cargo test -p xgauntlet-core --test session_lineage_test`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo run -p xgauntlet-cli -- check-spec -t 045`
