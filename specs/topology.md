# Living Feature Specification: `topology`

**Bounded Context**: `xGauntlet Core & Governance Engine` ([CONTEXT.md](../CONTEXT.md))  
**Feature Path**: `crates/xgauntlet-core/src/features/topology/`  
**Type**: `Stateless AST-Analysis & Diagramming Feature`  
**Governing ADRs**: [ADR 0001](../docs/adr/0001-package-by-feature-architecture.md), [ADR 0013](../docs/adr/0013-feature-architecture-boundaries-and-living-specs.md)

---

## 🎯 Purpose & Capability Boundary
The `topology` feature provides sub-5ms deterministic code topology analysis, AST symbol scanning, and Mermaid class diagram generation without background daemons:
- **Topology Scanner (`parser.rs`)**: Scans workspace source files, packages (`Cargo.toml`, `package.json`), and AST symbols (`fn`, `struct`, `enum`, `interface`) into a deterministic `TopologyGraph`.
- **Field & Relation Extractor (`parser.rs`)**: Scans struct fields and enum variants in domain model files (`models.rs`), resolving inter-model type dependencies (`Direct`, `Option<T>`, `Vec<T>`).
- **Mermaid Exporter (`mermaid.rs`)**: Exports the topology graph into a deterministic, syntax-compliant Mermaid `classDiagram` with field types and relationship arrows (`*--` composition / `-->` association).
- **Blast Radius & Traversal (`traversal.rs`)**: Computes dependency shortest paths and topological impact analysis across components.

## 🚫 Invariants
- Must operate strictly under a sub-5ms cold start per file with zero external background daemons or language servers (Zero-Daemon Invariant).
- Must emit 100% deterministic, sorted Mermaid class diagram outputs to prevent non-reproducible git diffs.
- Must separate conceptual domain models from infrastructure boilerplate when `MermaidLevel::Conceptual` is requested.

## 📋 Relaterede Tasks
- [Task 021: AST Topology and Codebase Discovery Engine](../tasks/archive/021-ast-topology-and-codebase-discovery-engine.md)
- [Task 071: Domain AST Field & Relationship Extraction for Mermaid](../tasks/archive/071-domain-ast-mermaid-fields-and-relationships.md)
- [Task 074: Domain Relationship Completeness and Trait Realization](../tasks/archive/074-domain-relationship-completeness-and-trait-realization.md)
- [Task 075: Domain Orphan Resolution and AST Symbol Unification](../tasks/archive/075-domain-orphan-resolution-and-ast-symbol-unification.md)
