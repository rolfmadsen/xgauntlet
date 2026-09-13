---
name: to-spec
description: Turn the current conversation into a spec and publish it to spec.md or the project issue tracker: no interview, just synthesis of what you've already discussed.
---

# To Spec

This skill takes the current conversation context and codebase understanding and produces a spec. Do NOT interview the user; just synthesize what you already know.

## Process

1. Explore the repo to understand the current state of the codebase, if you haven't already. Use the project's domain glossary vocabulary (`CONTEXT.md`) throughout the spec, and respect any ADRs in `docs/adr/` in the area you're touching.

2. Sketch out the seams at which you're going to test the feature. Existing seams should be preferred to new ones. Use the highest seam possible. If new seams are needed, propose them at the highest point you can. The fewer seams across the codebase, the better - the ideal number is one. Check with the user that these seams match their expectations.

3. Write the spec using the template below into `spec.md` (macro system spec) or publish it to the project issue tracker.

<spec-template>

# Specification: <Feature Name>

## Problem Statement

The problem that the user is facing, from the user's perspective.

## Solution

The solution to the problem, from the user's perspective.

## User Stories

A numbered list of user stories. Each user story should be in the format of:

1. As an <actor>, I want a <feature>, so that <benefit>

<user-story-example>
1. As a developer, I want to verify specifications against implementation automatically, so that regressions are caught before PR merge.
</user-story-example>

This list of user stories should be extensive and cover all aspects of the feature.

## Implementation Decisions

A list of implementation decisions that were made. This can include:

- The modules that will be built/modified
- The interfaces of those modules that will be modified
- Technical clarifications from the developer
- Architectural decisions
- Schema changes
- API contracts
- Specific interactions

Do NOT include specific file paths or volatile code snippets (they go stale fast).
Exception: if a prototype produced a snippet that encodes a decision more precisely than prose can (state machine, reducer, schema, type shape), inline it within the relevant decision and note briefly that it came from a prototype. Trim to the decision-rich parts, not a working demo, just the important bits.

## 🚫 Must NOT (System Invariants)

Negative constraints and architectural barriers that must never be broken under any circumstances (e.g. Zero Ambient Authority, Zero-Daemon, no external sockets, no secrets in logs).

## Testing Decisions

A list of testing decisions that were made. Include:

- A description of what makes a good test (only test external behavior, not implementation details)
- Which modules and seams will be tested
- Prior art for the tests (i.e. similar types of tests in the codebase)

## Out of Scope

A description of the things that are explicitly out of scope for this spec.

## Further Notes

Any further notes or trade-offs about the feature.

</spec-template>
