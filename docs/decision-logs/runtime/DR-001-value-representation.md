# DR-001: Shared Value Enum with Reference Counting

**Date:** 2024-01-18
**Status:** Accepted
**Component:** Runtime Model

## Context
Runtime needed to define how values are represented in memory and shared between interpreter and VM.

## Decision
Single shared `Value` enum across interpreter and VM:
- Reference counting (`Rc<T>` for single-threaded, `Arc<T>` for potential threading)
- No garbage collector in v0.1
- Strings immutable, arrays mutable and shared by reference
- Function arguments passed by value with shared references for heap types

## Rationale
**Code reuse:** Both engines use identical value representation, reducing duplication and ensuring consistent behavior.

**Memory safety:** Rust's `Rc<T>` provides automatic memory management without GC complexity.

**Performance:** Reference counting has predictable performance (no GC pauses).

## Alternatives Considered
- **Separate value types per engine:** Rejected - doubles maintenance burden, risk of behavior divergence
- **Garbage collector:** Rejected - adds complexity for v0.1, reference counting sufficient
- **Copy-on-write for arrays:** Rejected - complicates mutation semantics, reference semantics clearer

## Consequences
- ✅ **Benefits:** Single source of truth for value representation
- ✅ **Benefits:** Predictable memory behavior (no GC pauses)
- ✅ **Benefits:** Memory safe (Rust guarantees)
- ⚠️  **Trade-offs:** Reference cycles possible (not an issue in v0.1 without closures/objects)
- ❌ **Costs:** Reference counting overhead (minimal in practice)

## Implementation Notes
- `Value` enum in `crates/atlas-runtime/src/value.rs`
- Heap types use `Rc<String>` and `Rc<RefCell<Vec<Value>>>`
- Both interpreter and VM import same `Value` type

## References
- Spec: `docs/specification/runtime-spec.md`
- Implementation: `docs/implementation/02-core-types.md`
- Related: DR-002 (Security Context)
