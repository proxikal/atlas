# Phase 05: Value Equality Semantics

**Block:** 1 (Memory Model)
**Depends on:** Phases 02, 03 complete (all collection variants migrated)

---

## Objective

Audit and finalize the complete `PartialEq` implementation for `Value`. After Phases 02–03
replace `Arc::ptr_eq` with content equality for arrays and collections, this phase:
1. Verifies every arm of `Value`'s `PartialEq` is semantically correct
2. Removes all remaining `Arc::ptr_eq` hacks from `value.rs`
3. Documents the equality contract in code comments
4. Ensures the equality semantics are consistent across interpreter and VM

---

## Current State (verified 2026-02-21)

`value.rs` PartialEq uses `Arc::ptr_eq` for:
- `Value::Array` (line 165) ← fixed in Phase 02
- `Value::NativeFunction` (line 171) ← KEEP — functions compare by identity
- `Value::HashMap` (line 179) ← fixed in Phase 03
- `Value::HashSet` (line 181) ← fixed in Phase 03
- `Value::Queue` (line 183) ← fixed in Phase 03
- `Value::Stack` (line 185) ← fixed in Phase 03
- `Value::Regex` (line 187) — needs decision
- `Value::HttpRequest` (line 191) — needs decision
- `Value::HttpResponse` (line 193) — needs decision
- `Value::Future` (line 195) — reference equality (correct)
- `Value::TaskHandle` (line 197) — reference equality (correct)
- `Value::ChannelSender` (line 199) — reference equality (correct)
- `Value::ChannelReceiver` (line 201) — reference equality (correct)
- `Value::AsyncMutex` (line 203) — reference equality (correct)

---

## Decisions for Ambiguous Variants

### `Value::Regex`
Two regexes with the same pattern string are semantically equal.
**Decision:** Compare by pattern string: `a.as_str() == b.as_str()`

### `Value::HttpRequest` / `Value::HttpResponse`
HTTP objects are data — two requests with identical fields are equal.
**Decision:** Derive or implement `PartialEq` on `HttpRequest`/`HttpResponse` structs,
then compare by content: `a.as_ref() == b.as_ref()`

### `Value::NativeFunction`
Native functions are closures — no meaningful content equality. Identity is correct.
**Decision:** KEEP `Arc::ptr_eq` for `NativeFunction`. This is intentional and documented.

### `Value::Future` / `TaskHandle` / `ChannelSender` / `ChannelReceiver` / `AsyncMutex`
These are live runtime objects. Identity equality is correct — two different channels
with no messages are NOT the same channel.
**Decision:** KEEP `Arc::ptr_eq` for all async runtime variants.

### `Value::SharedValue`
Uses `Arc::ptr_eq` (added in Phase 04). Reference equality is correct for `Shared<T>`.
**Decision:** KEEP `Arc::ptr_eq` as established in Phase 04.

---

## Implementation

### 1. Update `PartialEq` arms for Regex, HttpRequest, HttpResponse

For Regex:
```rust
(Value::Regex(a), Value::Regex(b)) => a.as_str() == b.as_str(),
```

For HttpRequest/HttpResponse:
- Check if `HttpRequest` and `HttpResponse` derive `PartialEq` in `stdlib/http.rs`
- If not, add `#[derive(PartialEq)]` or implement manually
- Then: `(Value::HttpRequest(a), Value::HttpRequest(b)) => a.as_ref() == b.as_ref()`

### 2. Add documenting comments to PartialEq

```rust
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            // --- Value types: content equality ---
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Null, Value::Null) => true,
            (Value::Array(a), Value::Array(b)) => a == b,        // CoW: content
            (Value::HashMap(a), Value::HashMap(b)) => a == b,    // CoW: content
            (Value::HashSet(a), Value::HashSet(b)) => a == b,    // CoW: content
            (Value::Queue(a), Value::Queue(b)) => a == b,        // CoW: content
            (Value::Stack(a), Value::Stack(b)) => a == b,        // CoW: content
            (Value::Regex(a), Value::Regex(b)) => a.as_str() == b.as_str(),
            (Value::DateTime(a), Value::DateTime(b)) => a == b,
            (Value::HttpRequest(a), Value::HttpRequest(b)) => a.as_ref() == b.as_ref(),
            (Value::HttpResponse(a), Value::HttpResponse(b)) => a.as_ref() == b.as_ref(),
            // --- Reference types: identity equality ---
            // NativeFunction: closures have no meaningful content equality
            (Value::NativeFunction(a), Value::NativeFunction(b)) => Arc::ptr_eq(a, b),
            // SharedValue: reference semantics — same allocation = same reference
            (Value::SharedValue(a), Value::SharedValue(b)) => a == b,
            // Async runtime objects: live resources, identity is the only meaningful equality
            (Value::Future(a), Value::Future(b)) => Arc::ptr_eq(a, b),
            (Value::TaskHandle(a), Value::TaskHandle(b)) => Arc::ptr_eq(a, b),
            (Value::ChannelSender(a), Value::ChannelSender(b)) => Arc::ptr_eq(a, b),
            (Value::ChannelReceiver(a), Value::ChannelReceiver(b)) => Arc::ptr_eq(a, b),
            (Value::AsyncMutex(a), Value::AsyncMutex(b)) => Arc::ptr_eq(a, b),
            // Different variants are never equal
            _ => false,
        }
    }
}
```

### 3. Verify `Hash` consistency

In Rust, `a == b` must imply `hash(a) == hash(b)`. Check if `Value` implements `Hash`.
If it does, every `PartialEq` change requires a corresponding `Hash` update.

If `Value` does NOT implement `Hash` (likely — it contains floats), no action needed.

---

## Tests

```rust
#[test]
fn array_equality_by_content_not_identity() {
    let a = Value::Array(ValueArray::from_vec(vec![Value::Number(1.0)]));
    let b = Value::Array(ValueArray::from_vec(vec![Value::Number(1.0)]));
    assert_eq!(a, b); // different allocations, same content
}

#[test]
fn array_inequality_after_mutation() {
    let a = Value::Array(ValueArray::from_vec(vec![Value::Number(1.0)]));
    let mut b = a.clone();
    if let Value::Array(ref mut arr) = b {
        arr.push(Value::Number(2.0));
    }
    assert_ne!(a, b);
}

#[test]
fn regex_equality_by_pattern() {
    use regex::Regex;
    let a = Value::Regex(Arc::new(Regex::new(r"\d+").unwrap()));
    let b = Value::Regex(Arc::new(Regex::new(r"\d+").unwrap()));
    assert_eq!(a, b);
}

#[test]
fn native_function_inequality_different_closures() {
    use std::sync::Arc;
    let f1: crate::value::NativeFn = Arc::new(|_| Ok(Value::Null));
    let f2: crate::value::NativeFn = Arc::new(|_| Ok(Value::Null));
    let a = Value::NativeFunction(f1);
    let b = Value::NativeFunction(f2);
    assert_ne!(a, b); // different closures — identity inequality
}
```

---

## Acceptance Criteria

- [ ] All `Arc::ptr_eq` removed from collection variants in `value.rs`
- [ ] `Regex` equality compares by pattern string
- [ ] `HttpRequest`/`HttpResponse` equality compares by content
- [ ] `NativeFunction` and async runtime variants retain `ptr_eq` (documented)
- [ ] All equality unit tests pass
- [ ] No `Hash` inconsistency (audit complete)
