# Living Feature Specification: `adapters`

**Bounded Context**: `xGauntlet Core & Governance Engine` ([CONTEXT.md](../CONTEXT.md))  
**Feature Path**: `crates/xgauntlet-core/src/features/adapters/`  
**Type**: `Stateless Multi-Harness Adapter & Hook Normalization Feature`  
**Governing ADRs**: [ADR 0001](../docs/adr/0001-package-by-feature-architecture.md), [ADR 0004](../docs/adr/0004-harness-adapter-slices.md), [ADR 0006](../docs/adr/0006-multi-harness-policy-adapter-contract.md), [ADR 0013](../docs/adr/0013-feature-architecture-boundaries-and-living-specs.md), [ADR 0014](../docs/adr/0014-worktree-isolated-multi-agent-and-subagent-supervision.md)

---

## 🎯 Purpose & Capability Boundary
The `adapters` feature translates vendor-specific AI harness tool payloads, lifecycle hooks, and plugin manifests into canonical xGauntlet `CapabilityRequest` operations and fail-closed `AdapterHookVerdict` decisions across all supported harnesses:
- **Antigravity (`antigravity/`)**: `PreToolUse` gatekeeper normalization, `PreInvocation` cockpit telemetry + deduplication, `wrap_response` 5-line blockquote HUD, and `.agents/hooks.json` scaffolding.
- **Claude Code (`claude_code/`)**: `PreToolUse` / `PostToolUse` hook contract, tool payload normalization, and `.claude/settings.json` scaffolding.
- **OpenAI Codex (`codex/`)**: `PreToolUse` / `PostToolUse` hook contract, function/patch normalization, and `.codex/hooks.json` scaffolding.
- **Mistral Vibe (`mistral/`)**: `pre_tool` / `post_tool` TOML hook contract and tool payload normalization.
- **Shared Adapter Contract (`models.rs`, `workspace.rs`, `delegation.rs`)**: Payload-aware git worktree resolution (`resolve_effective_workspace` verifying matching git common-dir), unified multi-harness agent delegation classification (`classify_delegation_tool` → `ToolActionType::Delegate`), path sanitization (`WorkspaceRelativePath`), trusted `EnforcementContext` derivation, and embedded WASM policy evaluation.

## 🚫 Invariants
- Never trust caller-supplied authority parameters (`has_active_task`, `read_only`, `allowed_scopes`) from hook payloads ([ADR 0006](../docs/adr/0006-multi-harness-policy-adapter-contract.md)).
- Always fail closed on empty, malformed, or unrecognized tool payloads (`ToolActionType::Other` → `4038`).
- Never resolve workspace boundaries outside valid repository/worktree roots or allow `..` traversal escapes.
- Keep `PreToolUse` hook evaluation sub-5ms without background daemons or network calls.

## 📋 Relaterede Tasks
- [Task 088: Worktree-Isolated Multi-Agent & Subagent Supervision](../tasks/088-worktree-isolated-multi-agent-supervision.md)
