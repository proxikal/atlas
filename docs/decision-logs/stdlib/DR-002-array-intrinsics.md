# DR-002: Array API - Intrinsics vs Stdlib Split

**Date:** 2025-02-05
**Status:** Accepted
**Component:** Standard Library - Arrays

## Context
Array API implementation needed strategy for callback-based functions (map, filter, reduce) vs pure functions (concat, slice).

## Decision
**Split array functions by callback requirements:**

**Pure functions (10):** Implemented in `stdlib/array.rs`
- pop, shift, unshift, reverse, concat, flatten, indexOf, lastIndexOf, includes, slice

**Callback intrinsics (11):** Implemented in interpreter/VM directly
- map, filter, reduce, forEach, find, findIndex, flatMap, some, every, sort, sortBy

## Rationale
**Callback functions need runtime execution context:** To invoke user code, callbacks require access to interpreter/VM internals.

**Clean stdlib interface:** Each engine uses native calling mechanism without complex abstraction layer.

**Industry precedent:**
- V8 (JavaScript): `Array.prototype.map/filter/reduce` implemented as C++ runtime intrinsics
- CPython: `map()`, `filter()` implemented as builtin types in C
- Rust: Iterator methods like `map/filter` are compiler intrinsics for optimization

## Alternatives Considered
- **Create function-caller trait:** Rejected - adds complexity without benefit for 2-engine architecture in same codebase
- **All functions as intrinsics:** Rejected - pure functions don't need runtime context, belongs in stdlib
- **Callback wrapper abstraction:** Rejected - over-engineering for dual-engine design

## Consequences
- ✅ **Benefits:** Maintains clean stdlib interface
- ✅ **Benefits:** Each engine uses native calling mechanism (simple, direct)
- ✅ **Benefits:** Matches production compiler patterns (V8, CPython, Rust)
- ⚠️  **Trade-offs:** Code in two places (already Atlas's dual-engine architecture)
- ❌ **Costs:** None significant (architecture already dual-engine)

## Implementation Notes
**Phase:** `stdlib/phase-02-complete-array-api.md` (v0.2)

**Pure functions location:** `crates/atlas-runtime/src/stdlib/array.rs`

**Intrinsics location:**
- Interpreter: `crates/atlas-runtime/src/interpreter.rs` (builtin handlers)
- VM: `crates/atlas-runtime/src/vm.rs` (opcode handlers)

**Parity requirement:** Both engines implement identical callback intrinsics with identical test coverage.

## References
- Spec: `docs/specification/language-semantics.md` (Array section)
- API: `docs/api/stdlib.md` (Array module)
- Related: DR-001 (Value Representation)
