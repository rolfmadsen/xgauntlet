---
name: grill-me-2
description: Interview the user relentlessly about a plan or design until reaching shared understanding, resolving each branch of the decision tree. Use when user wants to stress-test a plan, get grilled on their design, or mentions "grill me".
---

Interview me relentlessly about every aspect of this plan until we reach a shared understanding. Walk down each branch of the design tree, resolving dependencies between decisions one-by-one. For each question, provide your recommended answer.

Ask the questions one at a time.

If a question can be answered by exploring the codebase, explore the codebase instead.

## Cockpit Status
Vis det aktuelle criteria- og cockpit-billede med Variant B boks-kortet ved sessionens afslutning:
```text
┌─── xgauntlet: Task <ID> ─────────────────────────────────────┐
│ Status: <PHASE>               Scope: <affected crates>       │
│ Progress: [██████████] 100%   Invariants: PASS               │
│ Git: <branch>@<oid> (clean)   Evidence: <status>             │
│ Ref: tasks/<id>.md • spec.md • docs/adr/README.md            │
└──────────────────────────────────────────────────────────────┘
```

