# Phase 02: Migrate Value::Array Variant

**Block:** 1 (Memory Model)
**Depends on:** Phase 01 complete (ValueArray type exists and compiles)

---

## Objective

Replace `Value::Array(Arc<Mutex<Vec<Value>>>)` with `Value::Array(ValueArray)` in `value.rs`.
Update `PartialEq`, `Display`, `Debug`, and the `From` impls on `Value`. After this phase,
`Arc<Mutex<Vec<Value>>>` is gone from `value.rs`. The interpreter and VM will have compile
errors — those are fixed in Phases 08–14.

---

## Current State (verified 2026-02-21)

`value.rs` line 34: `Array(Arc<Mutex<Vec<Value>>>),`

`value.rs` PartialEq (lines ~160–170): `(Value::Array(a), Value::Array(b)) => Arc::ptr_eq(a, b),`

`Display` for Value: iterates `a.lock().unwrap()` for array formatting.

`Clone` is derived — `Arc<Mutex<...>>` clone is a cheap refcount bump (same as ValueArray).

---

## Implementation

### 1. Replace the variant

In `value.rs`, change:
```rust
/// Array value (reference-counted, mutable through Mutex)
Array(Arc<Mutex<Vec<Value>>>),
```
to:
```rust
/// Array value (copy-on-write, value semantics)
Array(ValueArray),
```

Remove the `Arc` and `Mutex` imports if they are now unused (check — other variants still use them).

### 2. Update `PartialEq`

Change:
```rust
(Value::Array(a), Value::Array(b)) => Arc::ptr_eq(a, b),
```
to:
```rust
(Value::Array(a), Value::Array(b)) => a == b,
```
`ValueArray::PartialEq` compares contents — this is the semantic change.

### 3. Update `Display`

Old:
```rust
Value::Array(arr) => {
    let arr = arr.lock().unwrap();
    let items: Vec<String> = arr.iter().map(|v| format!("{}", v)).collect();
    write!(f, "[{}]", items.join(", "))
}
```
New:
```rust
Value::Array(arr) => {
    let items: Vec<String> = arr.iter().map(|v| format!("{}", v)).collect();
    write!(f, "[{}]", items.join(", "))
}
```

### 4. Update Debug (if manually implemented)

Same pattern — remove `.lock().unwrap()`.

### 5. Update any `From` / `Into` impls for `Value::Array`

Search `value.rs` for `Value::Array(Arc::new(Mutex::new(` — replace with `Value::Array(ValueArray::from_vec(`.

### 6. Update `value.rs` helper constructors

If `value.rs` has a `Value::array(items: Vec<Value>) -> Value` convenience constructor, update it:
```rust
pub fn array(items: Vec<Value>) -> Value {
    Value::Array(ValueArray::from_vec(items))
}
```

---

## Expected Compile Errors After This Phase

The following crates/files will have compile errors — expected, will be fixed in later phases:

- `interpreter/expr.rs` — all sites that do `.lock().unwrap()` on an array value
- `interpreter/stmt.rs` — same
- `vm/dispatch.rs` — same
- `vm/mod.rs` — same
- `stdlib/array.rs` — all array stdlib functions
- `stdlib/collections/hashmap.rs` — `extract_array` returning `Arc<Mutex<Vec<Value>>>`
- `stdlib/datetime.rs` — returning `Arc<Mutex<Vec<Value>>>`

**These are expected. Do not fix them in this phase.**

Build target for this phase: `cargo check -p atlas-runtime` reports errors ONLY in the
files listed above. `value.rs` itself compiles cleanly.

---

## Tests

The unit tests from Phase 01 (`cow_type_tests`) must continue to pass.

Add one integration test in `value.rs` test module:
```rust
#[test]
fn value_array_clone_is_independent() {
    let a = Value::Array(ValueArray::from_vec(vec![Value::Number(1.0)]));
    let mut b = a.clone();
    if let Value::Array(ref mut arr) = b {
        arr.push(Value::Number(2.0));
    }
    // a should still have len 1
    if let Value::Array(ref arr) = a {
        assert_eq!(arr.len(), 1);
    }
}

#[test]
fn value_array_equality_is_by_content() {
    let a = Value::Array(ValueArray::from_vec(vec![Value::Number(1.0)]));
    let b = Value::Array(ValueArray::from_vec(vec![Value::Number(1.0)]));
    assert_eq!(a, b); // content equal, different allocation
}
```

---

## Acceptance Criteria

- [ ] `Value::Array` variant uses `ValueArray` (no `Arc<Mutex<Vec<Value>>>` in `value.rs`)
- [ ] `PartialEq` for `Value::Array` uses content comparison
- [ ] `Display` formats arrays without `.lock()`
- [ ] `cargo check -p atlas-runtime` errors are ONLY in the expected files listed above
- [ ] Phase 01 unit tests still pass (run `cargo test -p atlas-runtime value.rs`)
- [ ] No `Arc<Mutex<Vec<Value>>>` remains in `value.rs`
