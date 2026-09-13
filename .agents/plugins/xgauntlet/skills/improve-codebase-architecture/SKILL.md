---
name: improve-codebase-architecture
description: Refactor modules, eliminate architectural smells, optimize dependency graphs, and generate interactive architectural reports.
---

# Architectural Refactoring & Smells Elimination

Systematic approach to improving code architecture without breaking verified behavioral contracts.

## Workflow
1. **Smell Identification**: Identify architectural code smells (divergent change, shotgun surgery, cyclic dependencies, primitive obsession).
2. **Preserve Contracts**: Keep all public assertions and verification suites completely frozen during refactoring.
3. **Incremental Restructuring**: Perform localized extractions and module boundary realignments.
4. **Gauntlet Verification**: Run `xgauntlet verify` to prove zero behavioral drift.
