# Phase 01: Define CoW Value Types

**Block:** 1 (Memory Model)
**Depends on:** None (Block 1 foundation)
**Estimated complexity:** Medium

---

## Objective

Introduce `ValueArray` and `ValueMap` as new types in `value.rs` alongside the existing
`Arc<Mutex<Vec<Value>>>` representation. Both new types must compile. No existing code is
changed in this phase — the old variants remain. This is the "add before remove" strategy
that prevents a big-bang rewrite from breaking the build for 20 phases.

---

## Current State (verified 2026-02-21)

`value.rs` line 34:
```rust
Array(Arc<Mutex<Vec<Value>>>),
```

`value.rs` line 48–54:
```rust
HashMap(Arc<Mutex<crate::stdlib::collections::hashmap::AtlasHashMap>>),
HashSet(Arc<Mutex<crate::stdlib::collections::hashset::AtlasHashSet>>),
Queue(Arc<Mutex<crate::stdlib::collections::queue::AtlasQueue>>),
Stack(Arc<Mutex<crate::stdlib::collections::stack::AtlasStack>>),
```

No `ValueArray`, `ValueMap`, or `Shared<T>` types exist anywhere in the codebase.

---

## Implementation

### 1. Add `ValueArray` type to `value.rs`

```rust
/// Copy-on-write array. Cheap to clone (refcount bump).
/// Mutations on a shared array clone the inner Vec first (Arc::make_mut).
#[derive(Clone, Debug)]
pub struct ValueArray(Arc<Vec<Value>>);

impl ValueArray {
    pub fn new() -> Self {
        ValueArray(Arc::new(Vec::new()))
    }

    pub fn from_vec(v: Vec<Value>) -> Self {
        ValueArray(Arc::new(v))
    }

    /// Read access — no clone needed.
    pub fn as_slice(&self) -> &[Value] {
        &self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get element by index — returns reference into inner Vec.
    pub fn get(&self, index: usize) -> Option<&Value> {
        self.0.get(index)
    }

    /// Mutating access — triggers CoW if Arc is shared.
    pub fn push(&mut self, value: Value) {
        Arc::make_mut(&mut self.0).push(value);
    }

    pub fn pop(&mut self) -> Option<Value> {
        Arc::make_mut(&mut self.0).pop()
    }

    pub fn set(&mut self, index: usize, value: Value) -> bool {
        let inner = Arc::make_mut(&mut self.0);
        if index < inner.len() {
            inner[index] = value;
            true
        } else {
            false
        }
    }

    pub fn insert(&mut self, index: usize, value: Value) {
        Arc::make_mut(&mut self.0).insert(index, value);
    }

    pub fn remove(&mut self, index: usize) -> Value {
        Arc::make_mut(&mut self.0).remove(index)
    }

    pub fn truncate(&mut self, len: usize) {
        Arc::make_mut(&mut self.0).truncate(len);
    }

    pub fn extend(&mut self, iter: impl IntoIterator<Item = Value>) {
        Arc::make_mut(&mut self.0).extend(iter);
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Value> {
        self.0.iter()
    }

    /// Returns true if this array is the sole owner (no other clones).
    /// Used by the VM to decide whether to mutate in-place or CoW-copy.
    pub fn is_exclusively_owned(&self) -> bool {
        Arc::strong_count(&self.0) == 1
    }

    /// Convert to owned Vec — clones only if shared.
    pub fn into_vec(self) -> Vec<Value> {
        Arc::try_unwrap(self.0).unwrap_or_else(|arc| (*arc).clone())
    }

    /// Expose inner Arc for cases that need to check sharing (e.g., equality).
    pub fn arc(&self) -> &Arc<Vec<Value>> {
        &self.0
    }
}

impl Default for ValueArray {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for ValueArray {
    fn eq(&self, other: &Self) -> bool {
        // Value equality — compare contents, not pointer identity
        self.0.as_slice() == other.0.as_slice()
    }
}

impl std::ops::Index<usize> for ValueArray {
    type Output = Value;
    fn index(&self, index: usize) -> &Value {
        &self.0[index]
    }
}

impl From<Vec<Value>> for ValueArray {
    fn from(v: Vec<Value>) -> Self {
        ValueArray::from_vec(v)
    }
}

impl FromIterator<Value> for ValueArray {
    fn from_iter<I: IntoIterator<Item = Value>>(iter: I) -> Self {
        ValueArray(Arc::new(iter.into_iter().collect()))
    }
}
```

### 2. Add `ValueMap` type to `value.rs`

```rust
use std::collections::HashMap;

/// Copy-on-write string-keyed map. Cheap to clone (refcount bump).
/// Mutations clone the inner HashMap if shared (Arc::make_mut).
#[derive(Clone, Debug, Default)]
pub struct ValueMap(Arc<HashMap<String, Value>>);

impl ValueMap {
    pub fn new() -> Self {
        ValueMap(Arc::new(HashMap::new()))
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.0.get(key)
    }

    pub fn insert(&mut self, key: String, value: Value) {
        Arc::make_mut(&mut self.0).insert(key, value);
    }

    pub fn remove(&mut self, key: &str) -> Option<Value> {
        Arc::make_mut(&mut self.0).remove(key)
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.0.contains_key(key)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, String, Value> {
        self.0.iter()
    }

    pub fn keys(&self) -> std::collections::hash_map::Keys<'_, String, Value> {
        self.0.keys()
    }

    pub fn values(&self) -> std::collections::hash_map::Values<'_, String, Value> {
        self.0.values()
    }

    pub fn is_exclusively_owned(&self) -> bool {
        Arc::strong_count(&self.0) == 1
    }
}

impl PartialEq for ValueMap {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_ref() == other.0.as_ref()
    }
}

impl From<HashMap<String, Value>> for ValueMap {
    fn from(m: HashMap<String, Value>) -> Self {
        ValueMap(Arc::new(m))
    }
}
```

### 3. Verify build

At the end of this phase, `cargo build -p atlas-runtime` must pass with zero errors.
The old `Array(Arc<Mutex<Vec<Value>>>)` variant is still present — no conflicts yet.

---

## Tests

Add to `crates/atlas-runtime/src/value.rs` (unit tests module):

```rust
#[cfg(test)]
mod cow_type_tests {
    use super::*;

    #[test]
    fn value_array_cow_push_does_not_affect_clone() {
        let mut a = ValueArray::from_vec(vec![Value::Number(1.0)]);
        let b = a.clone();
        a.push(Value::Number(2.0));
        assert_eq!(a.len(), 2);
        assert_eq!(b.len(), 1); // b is unaffected
    }

    #[test]
    fn value_array_in_place_mutation_when_exclusive() {
        let mut a = ValueArray::from_vec(vec![Value::Number(1.0)]);
        assert!(a.is_exclusively_owned());
        a.push(Value::Number(2.0)); // no copy — exclusive owner
        assert_eq!(a.len(), 2);
    }

    #[test]
    fn value_array_equality_by_content() {
        let a = ValueArray::from_vec(vec![Value::Number(1.0), Value::Number(2.0)]);
        let b = ValueArray::from_vec(vec![Value::Number(1.0), Value::Number(2.0)]);
        assert_eq!(a, b); // same content, different Arc
    }

    #[test]
    fn value_map_cow_insert_does_not_affect_clone() {
        let mut a = ValueMap::new();
        a.insert("x".to_string(), Value::Number(1.0));
        let b = a.clone();
        a.insert("y".to_string(), Value::Number(2.0));
        assert_eq!(a.len(), 2);
        assert_eq!(b.len(), 1); // b is unaffected
    }

    #[test]
    fn value_map_equality_by_content() {
        let mut a = ValueMap::new();
        a.insert("k".to_string(), Value::Number(42.0));
        let mut b = ValueMap::new();
        b.insert("k".to_string(), Value::Number(42.0));
        assert_eq!(a, b);
    }
}
```

---

## Acceptance Criteria

- [ ] `ValueArray` compiles with full API (new, push, pop, set, get, iter, CoW semantics)
- [ ] `ValueMap` compiles with full API
- [ ] `PartialEq` for both types uses value comparison, not pointer identity
- [ ] All 5 CoW unit tests pass
- [ ] `cargo build -p atlas-runtime` passes
- [ ] No existing tests broken (old `Value::Array` variant still present)
