# Phase 04: Implement Shared\<T\> Wrapper

**Block:** 1 (Memory Model)
**Depends on:** Phase 01 complete (ValueArray/ValueMap established the CoW pattern)

---

## Objective

Introduce `Shared<T>` — the explicit reference semantics wrapper for Atlas values.
When a user writes `shared<Buffer>`, they are opting into reference semantics: mutations
through any alias are visible to all aliases. This is the escape hatch from CoW. It uses
`Arc<Mutex<T>>` intentionally — the one place where reference semantics are *desired*.

Add a `Value::Shared` variant so the runtime can represent a shared-reference value.

---

## Current State (verified 2026-02-21)

No `Shared<T>` type exists anywhere in the codebase. The spec at
`docs/specification/memory-model.md` defines it but it has not been implemented.

---

## Implementation

### 1. Define `Shared<T>` in `value.rs`

```rust
use std::sync::{Arc, Mutex};

/// Explicit reference semantics wrapper.
///
/// `Shared<T>` opts into reference semantics: all clones point to the same underlying
/// value. Mutation through any clone is visible to all other clones. This is the
/// intentional escape hatch from CoW — used when the program explicitly requests
/// shared mutable state (e.g., `shared<Buffer>`).
///
/// Contrast with `ValueArray` which uses `Arc<Vec<Value>>` + CoW: mutations on a
/// `ValueArray` clone never affect the original. Mutations on a `Shared<T>` always do.
#[derive(Clone, Debug)]
pub struct Shared<T>(Arc<Mutex<T>>);

impl<T> Shared<T> {
    pub fn new(value: T) -> Self {
        Shared(Arc::new(Mutex::new(value)))
    }

    /// Acquire the lock and apply a read function.
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        let guard = self.0.lock().expect("Shared<T> lock poisoned");
        f(&*guard)
    }

    /// Acquire the lock and apply a mutation function.
    pub fn with_mut<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let mut guard = self.0.lock().expect("Shared<T> lock poisoned");
        f(&mut *guard)
    }

    /// Returns true if this is the only reference to the inner value.
    pub fn is_exclusively_owned(&self) -> bool {
        Arc::strong_count(&self.0) == 1
    }
}

impl<T: PartialEq> PartialEq for Shared<T> {
    fn eq(&self, other: &Self) -> bool {
        // Pointer equality — two Shared<T> are equal only if they are the same allocation.
        // This matches reference semantics: two different `shared<T>` variables with the
        // same contents are NOT equal unless they are the same reference.
        Arc::ptr_eq(&self.0, &other.0)
    }
}
```

**Note on `PartialEq` for `Shared<T>`:** Uses pointer equality deliberately. This is the
only place in the codebase where `ptr_eq` is correct — shared references are equal only
if they are the same reference, not just the same content. This is documented in the
memory model spec.

### 2. Add `Value::Shared` variant

In the `Value` enum:
```rust
/// Explicitly shared reference — reference semantics (see Shared<T>).
/// Mutations are visible to all aliases. Used for `shared<T>` annotated values.
SharedValue(Shared<Box<Value>>),
```

The inner type is `Box<Value>` to avoid recursive size issues with `Mutex<Value>`.

### 3. Update `PartialEq` for `Value::Shared`

```rust
(Value::SharedValue(a), Value::SharedValue(b)) => a == b,
```
Delegates to `Shared<T>::PartialEq` which uses `Arc::ptr_eq` — correct.

### 4. Update `Display` for `Value::Shared`

```rust
Value::SharedValue(s) => {
    s.with(|v| write!(f, "shared({})", v))
}
```

### 5. Update `Clone` for `Value::Shared`

`Clone` is derived. `Shared<T>` clone bumps the refcount — both aliases point to the
same allocation. This is the desired behavior for reference semantics.

---

## Tests

```rust
#[cfg(test)]
mod shared_tests {
    use super::*;

    #[test]
    fn shared_mutation_visible_through_all_aliases() {
        let s = Shared::new(42i64);
        let s2 = s.clone();
        s.with_mut(|v| *v = 100);
        assert_eq!(s2.with(|v| *v), 100); // mutation visible through s2
    }

    #[test]
    fn shared_equality_is_reference_not_content() {
        let a: Shared<i64> = Shared::new(42);
        let b: Shared<i64> = Shared::new(42); // same content, different allocation
        let c = a.clone(); // same allocation as a
        assert_ne!(a, b); // different references — not equal
        assert_eq!(a, c); // same reference — equal
    }

    #[test]
    fn value_shared_clone_shares_mutation() {
        let original = Value::SharedValue(Shared::new(Box::new(Value::Number(1.0))));
        let alias = original.clone();
        if let Value::SharedValue(ref s) = original {
            s.with_mut(|v| **v = Value::Number(99.0));
        }
        if let Value::SharedValue(ref s) = alias {
            s.with(|v| assert_eq!(**v, Value::Number(99.0)));
        }
    }
}
```

---

## Acceptance Criteria

- [ ] `Shared<T>` type defined with `with`, `with_mut`, `is_exclusively_owned` API
- [ ] `PartialEq` uses `Arc::ptr_eq` (reference equality, not content equality)
- [ ] `Value::SharedValue` variant added
- [ ] `Display` for `Value::SharedValue` compiles
- [ ] All 3 `Shared` unit tests pass
- [ ] `cargo build -p atlas-runtime` passes (or check-level with known errors from Phase 02)
