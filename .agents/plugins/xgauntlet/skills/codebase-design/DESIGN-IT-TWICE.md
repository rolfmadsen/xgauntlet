# Design It Twice

When exploring alternative interfaces for a chosen deepening candidate, use this parallel sub-agent pattern. Based on "Design It Twice" (John Ousterhout): your first idea is unlikely to be the best.

Uses the vocabulary in [SKILL.md](./SKILL.md): **module**, **interface**, **seam**, **adapter**, **leverage**.

## Process

### 1. Frame the problem space
Before spawning sub-agents, clarify the problem space for the chosen candidate:
- The constraints any new interface would need to satisfy.
- The dependencies it would rely on, and which category they fall into (see [DEEPENING.md](./DEEPENING.md)).
- A rough illustrative code sketch to ground the constraints.

### 2. Spawn sub-agents
Spawn 3+ sub-agents in parallel. Each must produce a **radically different** interface for the deepened module:
- **Agent 1**: "Minimize the interface: aim for 1–3 entry points max. Maximise leverage per entry point."
- **Agent 2**: "Maximise flexibility: support multiple use cases and extensions cleanly."
- **Agent 3**: "Optimise for the most common caller: make the default path trivial."
- **Agent 4** (if applicable): "Design around ports & adapters for cross-seam dependencies."

Each sub-agent outputs:
1. Interface (types, methods, params, plus invariants, ordering, error modes).
2. Usage example showing how callers use it.
3. What the implementation hides behind the seam.
4. Dependency strategy and adapters.
5. Trade-offs: where leverage is high, where it is thin.

### 3. Present and compare
Present designs sequentially, then compare them in prose:
- Contrast by **depth** (leverage at the interface).
- Contrast by **locality** (where change concentrates).
- Contrast by **seam placement**.

Give your own opinionated recommendation on which design is strongest and why.
