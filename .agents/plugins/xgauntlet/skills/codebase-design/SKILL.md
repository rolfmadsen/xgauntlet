---
name: codebase-design
description: Clean Architecture & Deep Modules design adhering to Package-by-Feature (Screaming Architecture) and John Ousterhout's philosophy of software design.
---

# Codebase Design & Deep Modules

Design software systems around deep modules with simple interfaces and rich hidden implementations.

## Design Principles
1. **Package-by-Feature**: Group code by business capabilities and feature domains rather than technical layer stereotypes.
2. **Deep Modules**: Strive for modules that provide powerful functionality through a simple, cohesive API surface.
3. **Information Hiding**: Keep implementation details, state management, and private helpers strictly encapsulated.
4. **Zero Ambient Authority**: Capabilities must be passed explicitly; avoid hidden ambient global state.
