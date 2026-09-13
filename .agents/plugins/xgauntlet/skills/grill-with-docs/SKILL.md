---
name: grill-with-docs
description: A relentless interview to sharpen a plan or design, which also creates docs (ADR's and glossary in CONTEXT.md) as decisions crystallise.
---

# Grill With Docs

A relentless interview to sharpen a plan or design, which actively challenges terminology, updates `CONTEXT.md` inline, and records Architectural Decision Records (ADRs) as decisions crystallise.

## 1. Grilling (The Design Tree)

Interview the user relentlessly until you reach a shared understanding. Map this as a **design tree**: every decision branches into the decisions that hang off it.

Work the tree in **rounds**. The **frontier** is every decision whose prerequisites are already settled: the questions you can ask _now_ without guessing at answers you haven't heard yet. Ask the whole frontier in one round: number each question and give your recommended answer. Then wait for the user's answers before the next round.

Format a round like so:

```
❓ **Q1** - **<question title>**: <question body, might be multiple paragraphs, including multiple choices>

➡️ <your recommended answer>

---

❓ **Q2** - **<question title>**: <question body, might be multiple paragraphs, including multiple choices>

➡️ <your recommended answer>
```

Finding _facts_ is your job, never the user's. Look up codebase facts and inspect files directly without blocking the user.

## 2. Active Domain Modeling During Grilling

### Challenge against the glossary
When the user uses a term that conflicts with existing language in `CONTEXT.md`, call it out immediately:
> *"Your glossary defines 'cancellation' as X, but you seem to mean Y. Which is it?"*

### Sharpen fuzzy language
When the user uses vague or overloaded terms, propose a precise canonical term:
> *"You're saying 'account': do you mean the Customer or the User? Those are different things."*

### Update CONTEXT.md inline
When a term is resolved, update `CONTEXT.md` right there. Don't batch these up: capture them as they happen.
`CONTEXT.md` should be totally devoid of implementation details. It is a glossary and nothing else.

### Offer ADRs sparingly
Only offer to create an ADR when all three are true:
1. **Hard to reverse**: the cost of changing your mind later is meaningful.
2. **Surprising without context**: a future reader will wonder *"why did they do it this way?"*
3. **The result of a real trade-off**: there were genuine alternatives and you picked one for specific reasons.

If any of the three is missing, skip the ADR. Number sequentially in `docs/adr/000X-slug.md`.

## Cockpit Status
Whenever active in an xGauntlet-enabled workspace, format the top of your responses with the Cockpit Task HUD:
```text
> ### 🛡️ [Task: <Task Title / ID>] `[<Task Type>: SPEC / GRILL]`
> **Status**: `Phase: SPEC` | `Gauntlet: PENDING` | `Git: <branch>@<oid> • <clean | dirty>`
> **Progress**: `Criteria: X/Y [■■□□□]` | `Scope: <affected crates/paths>`
> **Links**: 📋 [Task](tasks/) • 📄 [Spec](spec.md) • 📖 [Glossary](CONTEXT.md) • 🏛️ [ADR](docs/adr/README.md) • 🧪 [Evidence](evidence.md)
> 💡 **Next Action:** <kort beskrivelse af næste umiddelbare handling>
```
