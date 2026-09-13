---
name: domain-modeling
description: Socratic domain modeling and ubiquitous glossary extraction for CONTEXT.md and domain ADRs.
---

# Domain Modeling & Ubiquitous Language

Guide the user and agent in establishing clean operational boundaries and formal domain terminology.

## Workflow
1. **Identify Core Domain Entities**: Uncover ubiquitous terminology, core entities, and invariant rules before code is written.
2. **Aristotelian Definitions**: Formalize definitions in `CONTEXT.md` using genus-differentia format (`<Term> is a <Genus> that <Differentia>`).
3. **Operational Boundaries**: Clarify what is inside domain scope and what is explicitly excluded.
4. **Architectural Decisions**: When domain invariants require persistent trade-offs, formalize them as ADRs in `docs/adr/`.
