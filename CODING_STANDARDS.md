# Coding Standards: Polyglot (Rust & TypeScript & React)

This repository employs a **polyglot multi-stack architecture**. All subsystems adhere to authoritative language style guides harmonized under unified architectural invariants.

---

## 1. Transversal Engineering & Architectural Principles

All code across all languages in this repository adheres to these core craftsmanship invariants:

- **Uncle Bob Clean Architecture & TDD:** Follow the strict Red $\to$ Green $\to$ Refactor cycle. Write black-box acceptance tests first, prove they fail with expected behavior, write the minimal implementation, and refactor while assertions remain frozen.
- **Package-by-Feature (Screaming Architecture):** Colocate related domain models, business logic, schemas, services, and tests within cohesive feature directories. Architecture must be strictly acyclic.
- **Fail-Closed Security & Verification:** In security, authorization, and verification boundaries, unexpected or undefined states MUST fail closed (deny/abort) rather than swallow exceptions or fall through.
- **Invariant Property Testing:** Verify mathematical and business invariants (round-trip serialization, idempotence, monotonicity) using property-based testing (`hypothesis`, `fast-check`, `proptest`).
- **Evidence-First Development:** Line-by-line review is backed by executable evidence. Verification reports and test suites must be 100% green with zero mutation survivors.
- **Clean Documentation:** Document *Why* (invariants, architectural boundaries, failure modes) rather than restating *What* code lines do. Avoid noisy comments.

---

## 2. Rust Standards
Subsystems written in Rust follow the **Official Rust API Guidelines**:

### Naming & Case Conventions (C-CASE)
- **Naming Conventions:**
  - `UpperCamelCase` for types and traits (e.g., `VerificationReport`, `TaskContract`).
  - `snake_case` for functions, methods, and modules (e.g., `execute_verify`, `parse_task_file`).
  - `SCREAMING_SNAKE_CASE` for constants and statics.
- **Constructors:** Use `new()` or `with_capacity()` as standard constructor names.

### Error Handling & Invariants (C-GOOD-ERR)
- **Error Handling:**
  - `unwrap()` and `expect()` are strictly forbidden in production code. Use `?` operator for clean error propagation.
  - Libraries must define structured, strongly-typed errors implementing `std::error::Error` (via `thiserror`). Application binaries may use `anyhow` for top-level context.
  - Panics are acceptable only in test assertions and invariant property tests.

### Ergonomics & Ownership (C-CONV, C-GENERIC)
- **Borrowing over Cloning:** Pass shared references (`&str`, `&[T]`) instead of owned types (`&String`, `&Vec<T>`) in function arguments.
- **Standard Conversions:** Implement standard conversion traits (`From`, `TryFrom`, `AsRef`) where natural conversions exist.
- **Newtype Pattern (C-NEWTYPE):** Wrap primitive types in lightweight domain structs (e.g., `struct UserId(String);`) to prevent *Primitive Obsession*.

### Documentation & Tests (C-DOC)
- **Rustdoc Documentation Standard:** All public items MUST have `///` doc comments detailing purpose, `# Arguments`, `# Returns`, `# Errors`, `# Panics`, and `# Examples`.
- **Clippy Strictness:** Code must pass `cargo clippy -- -D warnings` with zero warnings.

### Concrete Rust DO / DON'T Examples

#### ❌ DON'T (Anti-pattern: Production `unwrap()`, excessive cloning, primitive obsession)
```rust
// ❌ unwrap in production, taking &String instead of &str, cloning everywhere
pub fn fetch_user_name(id: &String) -> String {
    let db = open_connection().unwrap(); // ❌ Panic in production
    let user = db.query(id.clone()).unwrap();
    user.name
}
```

#### ✅ DO (Idiomatic: Newtype pattern, `thiserror`, borrowing, standard doc comments)
```rust
use std::path::Path;
use thiserror::Error;

/// Structured domain error hierarchy.
#[derive(Debug, Error)]
pub enum UserError {
    #[error("User '{0}' not found")]
    NotFound(String),
    #[error("Database connection failed: {0}")]
    ConnectionFailed(String),
}

/// Strongly-typed User identifier preventing primitive obsession.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserId(String);

impl UserId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
```

---

## 3. TypeScript & React Standards
Subsystems written in TypeScript & React follow the **Google TypeScript Style Guide & React**:

### Type Safety & TypeScript Disciplines
- **Strict Mode:** Code must compile with `strict: true` and zero compiler warnings.
- **Full Typecheck & Project References:** In modern TypeScript/Vite/Solution architectures (where root `tsconfig.json` contains `"files": []` and `"references": [...]`), blind `tsc --noEmit` exits with code 0 without checking source files. Verification layers MUST use `tsc -b` (build mode) or `tsc --noEmit -p tsconfig.app.json` to ensure full type validation across all composite sub-projects.
- **No `any`:** `any` is strictly prohibited. Use `unknown` combined with type narrowing, type predicates (`is`), or validation libraries (`zod`) at runtime I/O boundaries.
- **Interfaces vs Types:** Use `interface` for public API object shapes and extensible contracts; use `type` for unions, intersections, tuple types, and utility types.
- **Discriminated Unions:** Model state machines and mutually exclusive states using discriminated unions (e.g. `{ status: 'success'; data: T } | { status: 'error'; error: Error }`) rather than parallel optional boolean flags.

### React & Component Architecture
- **Functional Components:** All components must be pure functional components with explicit props interfaces (`interface ButtonProps { ... }`).
- **Custom Hooks for Logic:** JSX templates must remain declarative presentation layers. Extract non-trivial business logic, asynchronous state, and side-effects into custom hooks (`use[Feature]`).
- **Component File Budget:** Components should stay focused and ideally under 150 lines. Decompose complex UIs into smaller, single-responsibility sub-components.
- **Immutability First:** Prefer `const` over `let`. Never mutate props or state objects directly; use shallow copies or immutable updates.

### Documentation & TSDoc Standards
- **TSDoc Documentation Standard:** All exported functions, hooks, interfaces, and component props MUST be documented with TSDoc tags (`@param`, `@returns`, `@throws`, `@example`).
- **Self-Documenting Types:** Do not write comments that merely rephrase type signatures. Document semantic invariants and edge-case behavior.

### Concrete TypeScript DO / DON'T Examples

#### ❌ DON'T (Anti-pattern: `any`, bloated component with inline async side-effects)
```tsx
// ❌ any type, mutable let, unhandled async in render
export default function UserCard(props: any) {
  let [data, setData] = React.useState<any>(null);
  React.useEffect(() => {
    fetch('/api/user/' + props.id).then(r => r.json()).then(d => setData(d));
  }, [props.id]);
  return <div>{data?.name}</div>;
}
```

#### ✅ DO (Idiomatic: Typed props interface, custom hook, TSDoc, discriminated union)
```tsx
import React from 'react';

/** State model for asynchronous user profile loading. */
export type UserState =
  | { status: 'idle' | 'loading' }
  | { status: 'success'; profile: UserProfile }
  | { status: 'error'; error: Error };

export interface UserProfile {
  readonly id: string;
  readonly name: string;
  readonly email: string;
}

export interface UserCardProps {
  /** The unique user identifier to display. */
  readonly userId: string;
  /** Optional callback fired when the profile card is clicked. */
  readonly onSelect?: (userId: string) => void;
}

/**
 * Custom hook to manage user profile fetching and lifecycle state.
 *
 * @param userId - Unique identifier for the user.
 * @returns Discriminated union state representing loading, success, or error.
 */
export function useUserProfile(userId: string): UserState {
  const [state, setState] = React.useState<UserState>({ status: 'idle' });

  React.useEffect(() => {
    let isMounted = true;
    setState({ status: 'loading' });

    fetch(`/api/users/${encodeURIComponent(userId)}`)
      .then((res) => {
        if (!res.ok) throw new Error(`Failed to load user: ${res.statusText}`);
        return res.json();
      })
      .then((profile: UserProfile) => {
        if (isMounted) setState({ status: 'success', profile });
      })
      .catch((error: Error) => {
        if (isMounted) setState({ status: 'error', error });
      });

    return () => {
      isMounted = false;
    };
  }, [userId]);

  return state;
}

/**
 * Presentational component rendering user profile details.
 */
export const UserCard: React.FC<UserCardProps> = ({ userId, onSelect }) => {
  const state = useUserProfile(userId);

  if (state.status === 'loading') return <div>Loading profile...</div>;
  if (state.status === 'error') return <div role="alert">{state.error.message}</div>;
  if (state.status !== 'success') return null;

  return (
    <article onClick={() => onSelect?.(userId)} className="user-card">
      <h3>{state.profile.name}</h3>
      <p>{state.profile.email}</p>
    </article>
  );
};
```

---

## 4. Cross-Stack Boundary & Interop Invariants
In polyglot repositories where multiple languages communicate (e.g. TypeScript frontend + Python/Rust backend):

- **Explicit Schema Contracts:** All boundary APIs (REST HTTP, GraphQL, WebSockets, IPC, CLI JSON) MUST be governed by machine-readable, versioned schema definitions (OpenAPI, JSON Schema, Protobuf).
- **Zero Untyped JSON Bridges:** Untyped dictionary/object mappings across boundaries are prohibited. Payloads must be validated with runtime validators (`zod` in TypeScript, Pydantic in Python, `serde` with strong types in Rust).
- **Uniform Error Envelope:** All boundary endpoints must emit a standardized error envelope (`{ "error": { "code": string, "message": string, "details": object } }`).
- **Deterministic Serialization:** Invariant round-trip tests must guarantee that data serialized in one stack deserializes identically in other stacks without precision or field loss.
