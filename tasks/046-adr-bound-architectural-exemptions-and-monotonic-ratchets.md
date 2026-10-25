---
type: Task Package
title: "Task 046: ADR-Bound Architectural Exemptions and Monotonic Ratchet Governance"
description: "Enforce mandatory ADR attribution on architecture exemptions and prevent autonomous ratchet ceiling inflation"
status: done
generated: { by: process:xgauntlet-task-init, at: "2026-10-25T17:16:04Z" }
tags: [architecture, exemptions, adr, ratchet, fail-closed, governance]
scope:
  - "crates/xgauntlet-core/src/features/architecture/**"
  - "crates/xgauntlet-core/src/features/config/**"
  - "crates/xgauntlet-cli/**"
  - "gauntlet.toml"
  - "docs/adr/**"
  - "README.md"
  - "spec.md"
test_refs:
  - "crates/xgauntlet-core/tests/architecture_gate_test.rs"
---

# Task 046: ADR-Bound Architectural Exemptions and Monotonic Ratchet Governance

**Status**: `DONE`
**Intent**: `🚀 NEW FEATURE`
**Oprettet**: `2026-10-25`

## 🎯 Formål
Forhindre at arkitekturgrænser og ratchet-lofter udvandes gennem uautoriserede eller vilkårlige udvidelser af `[[architecture.exemptions]]`. Etablere obligatorisk binding til accepterede ADR'er i `docs/adr/` og håndhæve fail-closed verifikation i gatekeeperen.

## 📋 Acceptance Criteria
- [x] **AC1 (ADR 0010 Formulering & Indeksering)**: Formulere ADR 0010 i `docs/adr/0010-adr-bound-architectural-exemptions-and-monotonic-ratchets.md` og indeksere i `docs/adr/README.md`, `README.md` og `spec.md`.
- [x] **AC2 (ArchitectureExemption Schema & ADR Attribution)**: Udvide `ArchitectureExemption` med `pub adr: String`, og opdatere TOML loader (`loader.rs`) og serialization (`render_toml`) til at understøtte `adr = "docs/adr/..."`.
- [x] **AC3 (Fail-Closed ADR Verification i Architecture Gatekeeper)**: I `run_architecture_check` verificere:
  - Exemption uden `adr` udløser `ArchitectureFindingCategory::ExemptionMissingAdr` (`report.is_valid = false`).
  - Exemption med manglende ADR-fil udløser `ArchitectureFindingCategory::ExemptionAdrNotFound`.
  - Exemption med en ADR, der ikke har `status: accepted` eller `status: stable`, udløser `ArchitectureFindingCategory::ExemptionAdrNotAccepted`.
- [x] **AC4 (Monotone Ratchet-Invarianter)**: Håndhæve at ratchet-lofter ikke kan hæves uden godkendt ADR og at gatekeeperen fail-closed validerer lofterne mod filstørrelser og ADR-begrundelser.
- [x] **AC5 (Opdatering af gauntlet.toml)**: Forankre de 3 eksisterende exemptions i `gauntlet.toml` til henholdsvis ADR 0010 (for `main.rs`) og ADR 0004 (for `harness_contracts_test.rs` og `harness_adapters_test.rs`).
- [x] **AC6 (Testsuite & Multi-Layer Gauntlet)**: Skrive dækkende integrationstests i `architecture_gate_test.rs` og verificere at hele workspace består (`cargo test --workspace`, `cargo clippy`, `cargo fmt`, `check-architecture`, `check-spec`).

## 🚫 Must NOT
- Må IKKE tillade udokumenterede undtagelser i `gauntlet.toml` uden gyldig ADR.
- Må IKKE bryde bagudkompatibilitet for eksisterende serde serialisering / deserialisering.
- Må IKKE foretage remote publication (`git push`).
- Må IKKE introducere dæmoner eller eksterne sockets.

## 📝 Revisions
- 2026-10-25: Task oprettet som ACTIVE for Task 046: ADR-Bound Architectural Exemptions and Monotonic Ratchet Governance.
- 2026-10-25: Task gennemført og forseglet som DONE efter succesfuld gauntlet-verifikation.

## 🧪 Verifikation
- `cargo test -p xgauntlet-core --test architecture_gate_test`
- `cargo run -p xgauntlet-cli -- check-spec -t 046-adr-bound-architectural-exemptions-and-monotonic-ratchets`
- `cargo run -p xgauntlet-cli -- check-architecture`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
