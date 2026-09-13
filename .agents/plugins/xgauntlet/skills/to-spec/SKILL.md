---
name: to-spec
description: Synthesize system philosophy, macro architecture, Package-by-Feature feature modules, and multi-layer verification contracts into spec.md.
---

# Specification Synthesis (`spec.md`)

Formalize the repository's macro system specification, system-wide invariants, and capabilities.

## Standard `spec.md` Structure
1. `# Specification: <System / Feature Name>`
2. `## 🎯 Philosophy & Core Capabilities`: Overordnede systemegenskaber og domæneprincipper.
3. `## 📐 Architecture & Feature Modules`: Modul- og pakkestruktur (`Package-by-Feature`).
4. `## 🚫 Must NOT (System Invariants)`: Globale sikkerheds- og arkitektur-invarianter, der gælder på tværs af alle opgaver.
5. `## 🧪 Multi-Layer Verification Contracts`: Makro-verifikationskriterier og test-dækning.

Validate with `xgauntlet check-spec` before proceeding to implementation.
