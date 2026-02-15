# DR-006: HashMap Stdlib Architecture Adaptation

**Date:** 2026-02-15
**Status:** Accepted
**Component:** Standard Library - Collections

## Context

Phase-07a specification references `stdlib/prelude.rs` for function registration, but this file doesn't exist. Current stdlib architecture uses `stdlib/mod.rs` with `is_builtin()` and `call_builtin()` pattern for all stdlib functions.

## Decision

**Adapt HashMap implementation to existing `stdlib/mod.rs` architecture:**

**Function registration pattern:**
```rust
// In stdlib/mod.rs
pub fn is_builtin(name: &str) -> bool {
    matches!(
        name,
        // Existing functions...
        // HashMap functions
        | "hashMapNew" | "hashMapFromEntries"
        | "hashMapPut" | "hashMapGet" | "hashMapRemove"
        | "hashMapHas" | "hashMapSize" | "hashMapIsEmpty"
        | "hashMapClear" | "hashMapKeys" | "hashMapValues" | "hashMapEntries"
    )
}

pub fn call_builtin(
    name: &str,
    args: &[Value],
    call_span: Span,
    security: &SecurityContext,
) -> Result<Value, RuntimeError> {
    match name {
        // Existing functions...
        "hashMapNew" => collections::hashmap::new_map(args, call_span),
        "hashMapPut" => collections::hashmap::put(args, call_span),
        // ... etc
    }
}
```

**File structure:**
- `stdlib/collections/mod.rs` - Module exports
- `stdlib/collections/hash.rs` - HashKey and hash infrastructure
- `stdlib/collections/hashmap.rs` - AtlasHashMap + function implementations

## Rationale

**Consistency with existing pattern:**
- All stdlib functions (string, array, math, json, io, types) use is_builtin/call_builtin
- Creating separate prelude.rs would fragment stdlib architecture
- Developers expect single registration point in stdlib/mod.rs

**Maintainability:**
- Single place to see all stdlib functions
- Easier to search and verify function registration
- Clear pattern for future collection types (HashSet, Queue, Stack)

**AI-first principle:**
- Consistent pattern easier for AI to learn and extend
- No need to remember multiple registration mechanisms
- Clear from codebase analysis (no special cases)

## Alternatives Considered

- **Create prelude.rs as specified:** Rejected - introduces architectural inconsistency, fragments stdlib registration
- **Move all functions to prelude.rs:** Rejected - massive refactoring, no clear benefit
- **Create collections/prelude.rs:** Rejected - partial solution, still inconsistent

## Consequences

- ✅ **Benefits:** Consistent with 100% of existing stdlib functions
- ✅ **Benefits:** Single source of truth for function registration
- ✅ **Benefits:** Clear pattern for HashSet/Queue/Stack in future phases
- ✅ **Benefits:** Easier for AI code generation (one pattern, not two)
- ⚠️  **Trade-offs:** Phase-07a spec not followed literally (adapted intelligently)
- ⚠️  **Trade-offs:** Slightly larger stdlib/mod.rs file (acceptable - still manageable)

## Implementation Notes

**Phase:** `stdlib/phase-07a-hash-infrastructure-hashmap.md`

**Files adapted:**
- Create: `stdlib/collections/mod.rs` (module exports)
- Create: `stdlib/collections/hash.rs` (HashKey, compute_hash)
- Create: `stdlib/collections/hashmap.rs` (AtlasHashMap, functions)
- Update: `stdlib/mod.rs` (is_builtin + call_builtin, NOT create prelude.rs)
- Update: `value.rs` (HashMap variant + RuntimeError::UnhashableType)

**Registration count:**
- 12 HashMap functions in is_builtin match
- 12 HashMap functions in call_builtin match
- Consistent with string (18), array (13), math (18), json (9)

## References

- Related: DR-005 (Collection API design)
- Pattern: `stdlib/string.rs`, `stdlib/array.rs`, `stdlib/math.rs` (existing examples)
- Spec: Phase-07a (adapted for existing architecture)
