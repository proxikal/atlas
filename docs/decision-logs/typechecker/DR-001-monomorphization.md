# DR-001: Monomorphization for Generic Types

**Date:** 2025-01-10
**Status:** Accepted
**Component:** Type System - Generics

## Context
Atlas v0.2 adds generic types (`Option<T>`, `Result<T,E>`). Strategy needed for compiling generics to bytecode.

## Decision
**Monomorphization (Rust-style):** Generate specialized code for each type instantiation.

**Implementation:**
- Monomorphizer caches specialized instances: `(function_name, type_args) -> substitution_map`
- Name mangling for VM dispatch: `identity<number>` → `identity$number`
- Type inference determines concrete types at compile time
- Both interpreter and VM use same monomorphization infrastructure
- Interpreter can stay polymorphic (tracks values), VM requires bytecode generation per instance

## Rationale
**Performance:** Zero runtime overhead - specialized code for each type.

**Type safety:** Full type information available at compile time.

**Proven approach:** Rust and C++ use monomorphization successfully in production.

**Debuggability:** Each specialization is standalone code - easier to debug than type-erased versions.

## Alternatives Considered
- **Type erasure (Java-style):** Rejected - loses type information at runtime, worse performance, casts needed
- **Runtime dispatch (Go-style):** Rejected - requires interface boxing, slower execution, dynamic overhead
- **Template-only (C++-style):** Rejected - code bloat without caching, harder to debug, compilation explosion

## Consequences
- ✅ **Benefits:** Zero runtime overhead
- ✅ **Benefits:** Full type safety at compile time
- ✅ **Benefits:** Easy debugging (each specialization standalone)
- ✅ **Benefits:** Proven in production (Rust, C++)
- ⚠️  **Trade-offs:** Code bloat for many instantiations (mitigated by caching)
- ⚠️  **Trade-offs:** Longer compile times for generic-heavy code
- ❌ **Costs:** More complex compiler implementation

## Implementation Notes
**Status:** Infrastructure complete (BLOCKER 02-C). Full pipeline in BLOCKER 02-D (`Option<T>`, `Result<T,E>`).

**Caching strategy:**
- Key: `(function_name, Vec<Type>)`
- Value: Substitution map for type parameters
- Reuse specializations across compilation units

**Name mangling:**
- `identity<number>` → `identity$number`
- `identity<string>` → `identity$string`
- `Result<number, string>` → `Result$number$string`

## References
- Spec: `docs/specification/language-semantics.md` (Generics section)
- Phase: `phases/v0.2/blockers/02-D-generic-types.md`
- Related: DR-001 (Type System)
