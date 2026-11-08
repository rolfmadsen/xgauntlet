# Living Slice Specification: `topology`

**Bounded Context**: `xGauntlet Core & Governance Engine` ([CONTEXT.md](../CONTEXT.md))  
**Slice Path**: `crates/xgauntlet-core/src/features/topology/`  
**Type**: `Stateless AST-Analysis & Diagramming Slice`  
**Governing ADRs**: [ADR 0001](../docs/adr/0001-package-by-feature-architecture.md), [ADR 0012](../docs/adr/0012-vertical-slice-boundaries-and-adr-lifecycle.md)

---

## 🎯 Purpose & Capability Boundary
The `topology` slice provides sub-5ms deterministic code topology analysis, AST symbol scanning, and Mermaid class diagram generation without background daemons:
- **Topology Scanner (`parser.rs`)**: Scans workspace source files, packages (`Cargo.toml`, `package.json`), and AST symbols (`fn`, `struct`, `enum`, `interface`) into a deterministic `TopologyGraph`.
- **Field & Relation Extractor (`parser.rs`)**: Scans struct fields and enum variants in domain model files (`models.rs`), resolving inter-model type dependencies (`Direct`, `Option<T>`, `Vec<T>`).
- **Mermaid Exporter (`mermaid.rs`)**: Exports the topology graph into a deterministic, syntax-compliant Mermaid `classDiagram` with field types and relationship arrows (`*--` composition / `-->` association).
- **Blast Radius & Traversal (`traversal.rs`)**: Computes dependency shortest paths and topological impact analysis across components.

## 🚫 Invariants
- Must operate strictly under a sub-5ms cold start per file with zero external background daemons or language servers (Zero-Daemon Invariant).
- Must emit 100% deterministic, sorted Mermaid class diagram outputs to prevent non-reproducible git diffs.
- Must separate conceptual domain models from infrastructure boilerplate when `MermaidLevel::Conceptual` is requested.
