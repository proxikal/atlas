# Phase 07: Interpreter — Array Mutation Paths + CoW Trigger

**Block:** 1 (Memory Model)
**Depends on:** Phase 06 complete (read paths fixed in interpreter)

---

## Objective

Fix all array *mutation* operations in the interpreter. This is the semantically critical
phase — CoW must be triggered correctly. The key rule:

> When the interpreter mutates an array, it must own a mutable reference to that array's
> data. If the `ValueArray` is shared (cloned and aliased), `Arc::make_mut` will clone
> the inner `Vec` before mutating. If it's exclusively owned, mutation is in-place.

This happens automatically via `ValueArray`'s mutation API (`push`, `pop`, `set`, etc.)
which all call `Arc::make_mut` internally. The interpreter does NOT need to call
`Arc::make_mut` directly — it just needs to call the right `ValueArray` methods on a
`&mut ValueArray`.

---

## The Critical Pattern

The challenge: the interpreter stores values in its environment as `Value`. To mutate
an array inside a `Value`, you need `&mut Value`, then `&mut ValueArray`.

```rust
// OLD (with Mutex — mutation through shared reference):
if let Value::Array(arr) = env.get("x") {
    arr.lock().unwrap().push(new_item); // pushes through the Arc
}

// NEW (CoW — mutation requires &mut):
if let Some(Value::Array(arr)) = env.get_mut("x") {
    arr.push(new_item); // CoW triggered if arr is shared
}
```

If the interpreter doesn't have `get_mut` on its environment, this phase may need to
add it. Read `interpreter/mod.rs` to understand the environment data structure before
implementing.

---

## Common Mutation Patterns

### Array push (most common)
```rust
// Expression: arr.push(item)
// OLD:
if let Value::Array(arr) = &array_val {
    arr.lock().unwrap().push(item_val);
}
// NEW — needs &mut Value::Array:
if let Value::Array(arr) = &mut array_val {
    arr.push(item_val);
}
```

### Array index assignment (`arr[i] = val`)
```rust
// OLD:
if let Value::Array(arr) = &array_val {
    let mut guard = arr.lock().unwrap();
    guard[index] = val;
}
// NEW:
if let Value::Array(arr) = &mut array_val {
    arr.set(index, val);
}
```

### Array pop
```rust
// OLD: arr.lock().unwrap().pop()
// NEW: arr.pop()  (on &mut ValueArray)
```

### Array built-in methods (sort, reverse, etc.)
Check if interpreter handles these directly or delegates to stdlib.
If stdlib: Phase 15 handles them.
If interpreter: update here.

---

## Environment Mutability Analysis

Before implementing, read `interpreter/mod.rs` to answer:
1. How does the interpreter store local variables? (HashMap? Vec? custom Env struct?)
2. Can it get `&mut Value` for a named variable?
3. Is there a `get_mut` or equivalent?

If the environment doesn't support `get_mut`, the mutation model needs to be:
```rust
let mut val = env.get("arr").clone(); // clone the Value (cheap for ValueArray — refcount bump)
if let Value::Array(ref mut arr) = val {
    arr.push(item); // CoW triggers if the original was aliased
}
env.set("arr", val); // put the (potentially new) Value back
```

This is the standard CoW update loop. It correctly handles aliasing: if `arr` had 2
references before the push, the clone produces a third, then `Arc::make_mut` sees
strong_count > 1 and clones the inner Vec. After `env.set`, the original reference
count drops. Aliased copies remain unchanged. Correct behavior.

---

## Tests

These tests specifically verify CoW semantics are working end-to-end through the interpreter:

```atlas
// Test: push to array does not affect original
let a = [1, 2, 3]
let b = a           // b is a clone of a (cheap)
b.push(4)           // CoW triggers: b gets its own copy
assert(a == [1, 2, 3])  // a unaffected
assert(b == [1, 2, 3, 4])
```

```atlas
// Test: index mutation does not affect original
let a = [1, 2, 3]
let b = a
b[0] = 99
assert(a[0] == 1)   // a unaffected
assert(b[0] == 99)
```

```atlas
// Test: function cannot mutate caller's array
fn append(arr: [number], x: number) -> [number] {
    arr.push(x)
    arr
}
let original = [1, 2]
let result = append(original, 3)
assert(original == [1, 2])     // original unaffected
assert(result == [1, 2, 3])
```

These tests go in `crates/atlas-runtime/tests/` in the appropriate domain test file.

---

## Acceptance Criteria

- [ ] All array mutation sites in `interpreter/` use `ValueArray` mutation API
- [ ] No `.lock().unwrap()` remains on `Value::Array` in `interpreter/`
- [ ] CoW semantics verified: push/pop/set on a cloned array does not affect the original
- [ ] Index assignment CoW test passes
- [ ] Function-cannot-mutate-caller test passes
- [ ] `cargo nextest run -p atlas-runtime` passes all interpreter array tests
