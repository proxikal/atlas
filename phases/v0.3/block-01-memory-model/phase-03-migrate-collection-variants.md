# Phase 03: Migrate Collection Variants to CoW

**Block:** 1 (Memory Model)
**Depends on:** Phase 01 complete (ValueArray/ValueMap types exist)

---

## Objective

Replace the four `Arc<Mutex<AtlasXxx>>` collection variants in `Value` with CoW wrappers.
`HashMap`, `HashSet`, `Queue`, `Stack` all currently use `Arc<Mutex<...>>` — the same
aliasing/deadlock problem as arrays. This phase creates CoW wrappers for each and swaps
the variants. The internal `AtlasHashMap` / `AtlasHashSet` / `AtlasQueue` / `AtlasStack`
types are NOT changed — only their wrapping changes.

---

## Current State (verified 2026-02-21)

`value.rs` lines 48–54:
```rust
HashMap(Arc<Mutex<crate::stdlib::collections::hashmap::AtlasHashMap>>),
HashSet(Arc<Mutex<crate::stdlib::collections::hashset::AtlasHashSet>>),
Queue(Arc<Mutex<crate::stdlib::collections::queue::AtlasQueue>>),
Stack(Arc<Mutex<crate::stdlib::collections::stack::AtlasStack>>),
```

`PartialEq` for these variants uses `Arc::ptr_eq` (lines 179–185).

`hashset.rs` lines 316, 340 use `Arc::ptr_eq` for set identity comparison.

---

## Implementation

### 1. Create CoW wrapper for each collection type

Add to `value.rs` (following the same pattern as `ValueArray`):

```rust
/// Copy-on-write wrapper for AtlasHashMap
#[derive(Clone, Debug, Default)]
pub struct ValueHashMap(Arc<crate::stdlib::collections::hashmap::AtlasHashMap>);

impl ValueHashMap {
    pub fn new() -> Self {
        ValueHashMap(Arc::new(crate::stdlib::collections::hashmap::AtlasHashMap::new()))
    }

    pub fn inner(&self) -> &crate::stdlib::collections::hashmap::AtlasHashMap {
        &self.0
    }

    pub fn inner_mut(&mut self) -> &mut crate::stdlib::collections::hashmap::AtlasHashMap {
        Arc::make_mut(&mut self.0)
    }

    pub fn is_exclusively_owned(&self) -> bool {
        Arc::strong_count(&self.0) == 1
    }
}

impl PartialEq for ValueHashMap {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_ref() == other.0.as_ref()
    }
}
```

Repeat for `ValueHashSet`, `ValueQueue`, `ValueStack` — same structure, different inner type.

**Note:** Each collection's inner type must implement `PartialEq` for this to compile.
Check each `AtlasXxx` type in `stdlib/collections/`. If `PartialEq` is missing, derive it
or implement it for the inner type. Do not skip — equality by pointer identity is wrong.

### 2. Replace variants in `Value`

```rust
HashMap(ValueHashMap),
HashSet(ValueHashSet),
Queue(ValueQueue),
Stack(ValueStack),
```

### 3. Update `PartialEq` in `Value`

Replace `Arc::ptr_eq` calls:
```rust
(Value::HashMap(a), Value::HashMap(b)) => a == b,
(Value::HashSet(a), Value::HashSet(b)) => a == b,
(Value::Queue(a), Value::Queue(b)) => a == b,
(Value::Stack(a), Value::Stack(b)) => a == b,
```

### 4. Update `Display` for collections

Remove `.lock().unwrap()` from any Display impls for these variants.
The `inner()` accessor provides read access without locking.

### 5. Fix `hashset.rs` `Arc::ptr_eq` calls

`hashset.rs` lines 316 and 340 compare `Arc<Mutex<AtlasHashSet>>` by pointer for
set-intersection/difference identity optimization. After this phase, the parameter types
change. Update the comparison to use content equality or remove the shortcut:
```rust
// Before: if Arc::ptr_eq(&set_a, &set_b) { ... }
// After: if set_a == set_b { ... }  — or simply remove the shortcut
```

### 6. Update `extract_xxx` helper functions in stdlib/collections/

Each collection module has a pattern like:
```rust
fn extract_hashmap(value: &Value, span: Span) -> Result<Arc<Mutex<AtlasHashMap>>, RuntimeError>
```
These will have compile errors after the variant change. Update return types to use the new wrappers. However — defer full fixup to Phases 16–19. In this phase, only update the function signatures to compile; the caller sites can remain broken temporarily.

---

## Tests

Add unit tests alongside Phase 01's cow_type_tests:

```rust
#[test]
fn value_hashmap_cow_insert_does_not_affect_clone() {
    let mut a = ValueHashMap::new();
    a.inner_mut().insert("x".to_string(), Value::Number(1.0));
    let b = a.clone();
    a.inner_mut().insert("y".to_string(), Value::Number(2.0));
    // b should still have 1 key
    assert_eq!(b.inner().len(), 1);
}

#[test]
fn value_collection_equality_by_content() {
    let mut a = ValueHashMap::new();
    a.inner_mut().insert("k".to_string(), Value::Number(1.0));
    let mut b = ValueHashMap::new();
    b.inner_mut().insert("k".to_string(), Value::Number(1.0));
    assert_eq!(a, b);
}
```

---

## Acceptance Criteria

- [ ] All four collection variants use CoW wrappers (no `Arc<Mutex<AtlasXxx>>` in `value.rs`)
- [ ] `PartialEq` for all four variants uses content comparison
- [ ] `Arc::ptr_eq` removed from `value.rs` for all collection variants
- [ ] `hashset.rs` `Arc::ptr_eq` calls updated (lines 316, 340)
- [ ] `cargo check -p atlas-runtime` reports no new errors beyond what Phase 02 left
- [ ] CoW unit tests pass
