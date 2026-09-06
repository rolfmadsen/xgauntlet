---
type: Architecture Documentation Index
title: "Architecture Decision Records (ADRs)"
description: "Oversigt over projektets arkitekturbeslutninger og ADR-governance"
status: stable
generated: { by: process:agent-gauntlet-init, at: "2026-08-23T12:00:00Z" }
tags: [adr, architecture, index, okf]
---

# Architecture Decision Records (ADRs)

Dette katalog indeholder Architecture Decision Records (ADR) for `xGauntlet`.

- **Strict ADR Adherence**: AI-agenter og udviklere SKAL følge samtlige accepterede ADR'er i `docs/adr/`.
- **Active Sparring on Conflicts**: Hvis et forslag eller en prompt strider mod gældende ADR'er, SKAL agenten gøre opmærksom på konflikten og afklare beslutningen først.
- **Lazy Creation**: Nye ADR'er oprettes kun ved irreversible, ikke-trivielle arkitekturvalg.

## 📋 ADR Indeks

| ADR | Titel | Status | Kernebeslutning |
|---|---|---|---|
| [ADR 0001](0001-package-by-feature-architecture.md) | Package-by-Feature Architecture | `ACCEPTED` | Kildekode opdeles vertikalt i feature-slices under `crates/xgauntlet-core/src/features/`. |
| [ADR 0002](0002-cryptographic-evidence-authority.md) | Cryptographic Evidence Authority | `SUPERSEDED` | Erstattet af ADR 0005. Viser hvorfor lokale HMAC-nøgler fejler under samme OS-bruger. |
| [ADR 0003](0003-surgical-gatekeeper-and-no-remote-push.md) | Surgical Gatekeeper & No Remote Push | `ACCEPTED` | Fail-closed beskyttelse af kildekode uden aktiv task. Hård blokering af `git push`. |
| [ADR 0004](0004-harness-adapter-slices.md) | Vertical Slice Harness Adapters | `ACCEPTED` | Hver AI-harness (Antigravity, Claude Code, Codex) er en autonom vertikal slice. |
| [ADR 0005](0005-two-tier-verification-and-attestation-model.md) | Two-Tier Evidence and Trust Boundary | `ACCEPTED` | Lokal drift-kontrol (Tier 1) adskilles fra autoritativ Sigstore OIDC attestering i CI (Tier 2). |
| [ADR 0006](0006-multi-harness-policy-adapter-contract.md) | Multi-Harness Policy Adapter Contract | `ACCEPTED` | Strikt adskillelse af `CapabilityRequest` og `EnforcementContext`. Zero cryptographic authority. |
| [ADR 0007](0007-local-transparent-supervisor-and-wasm-verifier.md) | Zero-Daemon WASM Policy Verifier | `ACCEPTED` | Nul ambient authority via in-memory Wasmtime og matematisk pre/post manifestkontrol. |
