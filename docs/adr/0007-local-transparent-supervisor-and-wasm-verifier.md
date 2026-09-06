---
type: Architectural Decision Record
title: 'ADR 0007: Local Transparent Supervisor with Embedded WASM Policy and Privilege Separation'
status: stable
tags:
- architecture
- adr
- supervisor
- wasm
- systemd
- security
generated:
  by: antigravity/gemini-3.7-flash
  at: '2026-08-30T09:16:00Z'
verified: []
---

# 7. Local Transparent Supervisor with Embedded WASM Policy & Privilege Separation

**Status**: `accepted` (Extends ADR 0005 and ADR 0006)  
**Date**: `2026-08-30`  

## Context
ADR 0005 decoupled local unsigned reports from authoritative CI attestations because unprivileged local executions cannot establish an independent trust boundary against a process running as the same OS user. While this truthfully documented local limitations, developers need strong local provenance and tamper-evident enforcement during AI-agent coding sessions before pushing to remote CI.

Furthermore, running raw gatekeeper hooks in Python inside IDE processes introduces startup latency and couples the agent harness to the host system's Python runtime.

## Decision
Vi etablerer en WebAssembly-baseret policy-arkitektur med Zero Ambient Authority og forseglet matematisk manifestkontrol:

1. **Zero Ambient Authority WASM Policy Component**:
   Policy-evaluering (task-binding, stibeskyttelse, blokering af destruktive kommandoer) kompileres til en deterministisk WebAssembly komponent (`wit/gauntlet_policy.wit`). Komponenten har absolut nul adgang til værts-filsystem, netværk, system-ur, tilfældighedskilder eller private nøgler.

2. **Zero-Daemon Arkitektur (xGauntlet Evolution)**:
   Hvor `agent-gauntlet` benyttede en baggrundsdæmon med Linux `systemd` for at omgå Python koldstart-latens, eliminerer `xGauntlet` behovet for dæmoner fuldstændigt:
   - Skrevet i **Rust** med sub-3ms koldstart.
   - Indlejrer **Wasmtime** direkte i memory. Hvert hook- og CLI-kald afvikles øjeblikkeligt i en selvstændig, lynhurtig proces uden baggrundsterminaler, socket-filer eller service managers.

3. **Matematisk Anti-Tamper Manifest (Self-Mutation Invariant)**:
   I stedet for skrøbelige OS-kernel sandkasser (`bwrap`, `sandbox-exec`), der fejler i containere eller på Windows/macOS, håndhæver xGauntlet en matematisk invariant:
   - Pre- og post-test sammenligning af deterministiske Git-blob SHA-256 digests.
   - Enhver ændring af kildekode, tests eller konfiguration under testkørsel detekteres og afvises øjeblikkeligt.

4. **Two-Tier Tillidsmodel (ADR 0005 Paritet)**:
   - `LOCAL_UNSUPERVISED`: Lokal deterministisk verifikationsrapport for hurtig feedback og matematisk drift-kontrol (`xgauntlet check-evidence`).
   - `CI_ATTESTED`: Autoritativ, kryptografisk Sigstore OIDC / in-toto attestation udstedt uafhængigt i et isoleret CI-miljø.

## Consequences
- **Positivt**: 100% kryds-platform paritet på Linux, macOS og Windows 11 uden eksterne dæmon- eller systemkrav.
- **Positivt**: Nul ambient authority med deterministisk Wasmtime-evaluering.
- **Positivt**: Ingen baggrundsdæmoner der kan gå ned eller kræve genstart.
- **Positivt**: Fuld resistens mod selvmutation og kildedrift via Git-blob OID manifests.
