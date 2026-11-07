# ADR Format

ADRs live in `docs/adr/` and use sequential numbering: `0001-slug.md`, `0002-slug.md`, etc.

Create the `docs/adr/` directory lazily: only when the first ADR is needed.

## Template

```md
# {Short title of the decision}

{1-3 sentences: what's the context, what did we decide, and why.}
```

That's it. An ADR can be a single paragraph. The value is in recording *that* a decision was made and *why*, not in filling out sections.

## ADR Lifecycle & Frontmatter (`check-adr` & `check-release`)

Every ADR in `docs/adr/` uses OKF v0.2 YAML frontmatter with a deterministic 4-state lifecycle:
- `status: proposed` — Under discussion; not yet enforced or required in release docs.
- `status: accepted` — Active architectural decision; **must** be referenced in `README.md` or `spec.md` (`xgauntlet check-release`).
- `status: superseded` — Replaced by a newer ADR; **must** specify `superseded_by: "<new-adr-file.md>"` in frontmatter (`MISSING_ADR_SUPERSEDED_BY` if omitted). Exempt from `README.md`/`spec.md` release coverage.
- `status: deprecated` — Retired decision no longer applicable; exempt from `README.md`/`spec.md` release coverage.

```yaml
---
type: Architectural Decision Record
title: "ADR 0012: Example Decision"
status: superseded
superseded_by: "0015-new-decision.md"
generated: { by: human:architect, at: "2026-11-07T10:00:00Z" }
tags: [architecture]
---
```

> **ADR Gatekeeper Rule:** Functional feature specifications belong in `specs/<domain>/<slice>.md`, never in `docs/adr/`.

## Optional sections

Only include these when they add genuine value. Most ADRs won't need them.

- **Status** frontmatter (`proposed | accepted | deprecated | superseded` with `superseded_by`): useful when decisions are revisited
- **Considered Options**: only when the rejected alternatives are worth remembering
- **Consequences**: only when non-obvious downstream effects need to be called out

## Numbering

Scan `docs/adr/` for the highest existing number and increment by one.

## When to offer an ADR

All three of these must be true:

1. **Hard to reverse**: the cost of changing your mind later is meaningful
2. **Surprising without context**: a future reader will look at the code and wonder "why on earth did they do it this way?"
3. **The result of a real trade-off**: there were genuine alternatives and you picked one for specific reasons

If a decision is easy to reverse, skip it: you'll just reverse it. If it's not surprising, nobody will wonder why. If there was no real alternative, there's nothing to record beyond "we did the obvious thing."
