# Phase 14: Stdlib — HashSet, Queue, Stack Modules

**Block:** 1 (Memory Model)
**Depends on:** Phase 03 complete (ValueHashSet/ValueQueue/ValueStack exist)

---

## Objective

Update the three remaining collection stdlib modules:
- `stdlib/collections/hashset.rs`
- `stdlib/collections/queue.rs`
- `stdlib/collections/stack.rs`

Same pattern as Phase 13: remove `Arc<Mutex>` extraction, use CoW wrapper `.inner()` /
`.inner_mut()`, return modified value for mutations.

---

## Current State (verified 2026-02-21)

`hashset.rs`: `extract_hashset` at line 374, `Arc::ptr_eq` at lines 316 and 340.
`queue.rs`: `extract_queue` at line 178 returning `Arc<Mutex<AtlasQueue>>`.
`stack.rs`: `extract_stack` at line 175 returning `Arc<Mutex<AtlasStack>>`.

---

## hashset.rs Specific: Set Identity Optimization

Lines 316 and 340 use `Arc::ptr_eq` as an optimization for intersection/difference:
```rust
// If both sets are the same Arc, result is trivially known
if Arc::ptr_eq(&set_a, &set_b) { ... }
```

After Phase 03, sets use `ValueHashSet`. The optimization can be preserved:
```rust
// Check if same underlying allocation (sets are identical)
if std::ptr::eq(set_a.arc().as_ptr(), set_b.arc().as_ptr()) { ... }
```
Or simply remove the optimization — correctness over micro-optimization.
Recommendation: remove it. The test suite will tell us if it matters for performance.

## Implementation Pattern (same for all three)

```rust
// Extract helper (read):
fn extract_queue_ref(value: &Value, span: Span) -> Result<&ValueQueue, RuntimeError> {
    match value {
        Value::Queue(q) => Ok(q),
        _ => Err(RuntimeError::type_error("expected Queue", span))
    }
}

// Mutation function:
fn queue_enqueue(args: &[Value]) -> Result<Value, RuntimeError> {
    let mut val = args[0].clone();
    if let Value::Queue(ref mut q) = val {
        q.inner_mut().enqueue(args[1].clone());
    }
    Ok(val)
}
```

---

## Tests

```
cargo nextest run -p atlas-runtime -- stdlib::collections
```

All 4 collection stdlib test suites must pass after this phase.

---

## Acceptance Criteria

- [ ] `hashset.rs`, `queue.rs`, `stack.rs` all use CoW wrapper API
- [ ] `Arc::ptr_eq` optimization in `hashset.rs` removed or updated
- [ ] No `.lock().unwrap()` in any of the three modules
- [ ] Mutation functions return modified `Value::HashSet` / `Value::Queue` / `Value::Stack`
- [ ] All collection stdlib tests pass
