# Phase 12: Stdlib — Array Module

**Block:** 1 (Memory Model)
**Depends on:** Phase 02 complete (Value::Array is ValueArray)

---

## Objective

Rewrite `crates/atlas-runtime/src/stdlib/array.rs` to use the `ValueArray` API throughout.
This is the largest single-file change in Block 1 — array.rs has the most `.lock()` sites.
Every array stdlib function must be updated.

---

## Current State (verified 2026-02-21)

`stdlib/array.rs` uses `Arc<Mutex<Vec<Value>>>` throughout. The file has the highest
concentration of `.lock().unwrap()` calls (estimated ~50+). After Phase 02,
`Value::Array(ValueArray)` is in place — array.rs will have compile errors at every
function that touches arrays.

---

## Pattern to Apply

### Extract array from Value (read)
```rust
// OLD:
fn stdlib_array_len(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::Array(arr) => {
            let guard = arr.lock().unwrap();
            Ok(Value::Number(guard.len() as f64))
        }
        _ => Err(RuntimeError::type_error("expected array"))
    }
}

// NEW:
fn stdlib_array_len(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::Array(arr) => Ok(Value::Number(arr.len() as f64)),
        _ => Err(RuntimeError::type_error("expected array"))
    }
}
```

### Extract array from Value (mutation — requires owned Value)
```rust
// OLD:
fn stdlib_array_push(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::Array(arr) => {
            arr.lock().unwrap().push(args[1].clone());
            Ok(Value::Null)
        }
        _ => Err(...)
    }
}

// NEW (mutation requires &mut — stdlib functions take &[Value], so clone first):
fn stdlib_array_push(args: &[Value]) -> Result<Value, RuntimeError> {
    match args[0].clone() {
        Value::Array(mut arr) => {
            arr.push(args[1].clone());  // CoW triggers if arr is shared
            Ok(Value::Array(arr))       // return the (possibly new) array
        }
        _ => Err(...)
    }
}
```

**Critical design note:** Stdlib functions receive `&[Value]`. They cannot take `&mut Value`.
Mutation functions must:
1. Clone the `Value::Array` (cheap — refcount bump)
2. Call mutation on the cloned `ValueArray` (CoW triggers if the original was aliased)
3. Return the new `Value::Array`

The caller (interpreter or VM) is responsible for storing the returned value back to the
variable. This is the correct CoW model — stdlib returns the new state, caller updates.

---

## Functions to Update

All stdlib array functions in `array.rs`. Common ones:
- `array_len`, `array_push`, `array_pop`
- `array_get`, `array_set`, `array_slice`
- `array_concat`, `array_join`, `array_reverse`
- `array_sort`, `array_map`, `array_filter`, `array_reduce`
- `array_find`, `array_find_index`, `array_includes`
- `array_flatten`, `array_zip`, `array_unzip`
- `array_first`, `array_last`, `array_rest`
- `array_fill`, `array_range`, `array_unique`

Read the actual function list from `array.rs` before implementing — don't assume.

---

## Return Value Convention Change

**Before:** `push` returned `Value::Null` (mutated in-place through Arc).
**After:** `push` returns the modified `Value::Array`.

This is a **semantic change** that must be propagated:
1. The interpreter must use the return value of `push` and store it back
2. The VM must use the return value of `push` and update the stack/variable
3. Any test that does `push(arr, x)` and then reads `arr` must be reviewed

This is intentional — it makes array operations purely functional at the stdlib level,
which is correct for value semantics.

---

## Tests

After this phase, all array stdlib tests must pass:
```
cargo nextest run -p atlas-runtime -- stdlib::array
```

Key test: verify `push` returns the new array and the original is unaffected.

---

## Acceptance Criteria

- [ ] All functions in `stdlib/array.rs` use `ValueArray` API
- [ ] Mutation functions return `Value::Array` (not `Value::Null`)
- [ ] No `.lock().unwrap()` in `stdlib/array.rs`
- [ ] All array stdlib tests pass
- [ ] `cargo nextest run -p atlas-runtime -- stdlib::array` 100% pass
