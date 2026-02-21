# Phase 16: Stdlib Return Value Propagation (Interpreter + VM)

**Block:** 1 (Memory Model)
**Depends on:** Phases 07, 10, 12 complete (interpreter mutations, VM mutations, stdlib array done)

---

## Objective

Phase 12 changed array mutation stdlib functions (push, pop, sort, etc.) to return the
modified `Value::Array` instead of `Value::Null`. This is correct for value semantics —
but the interpreter and VM must be updated to USE the return value and store it back to
the variable. Without this phase, `arr.push(x)` would return a new array that gets
discarded, leaving the original variable unchanged.

---

## The Problem

In the old model:
```
arr.push(x)
→ stdlib_push(arr, x)
→ arr.lock().unwrap().push(x)  // mutates in-place through Arc
→ returns Null
→ interpreter ignores return value ← fine, because mutation happened in-place
```

In the new model:
```
arr.push(x)
→ stdlib_push(arr, x)
→ clones arr, pushes to clone, returns new Value::Array
→ interpreter must store return value back to 'arr' ← NOT currently happening
```

---

## What Needs to Change

### Interpreter: method call dispatch

When the interpreter evaluates `arr.push(x)` (a method call on a value):
1. It evaluates `arr` → gets a `Value::Array`
2. It calls the stdlib function with `[arr, x]`
3. **Currently:** discards the result
4. **Required:** stores the result back to the `arr` variable

```rust
// In interpreter/expr.rs, method call handling:
// After calling stdlib function:
let result = call_stdlib(method_name, &args)?;
// NEW: if the method is a mutating collection method, store result back
if is_mutating_method(method_name) {
    env.set(receiver_name, result.clone());
}
```

**Better approach:** The stdlib function always returns the new value. The interpreter
always stores it back. This is cleaner than tracking "mutating vs. non-mutating" methods.

Pattern to adopt: for any method call on a collection (Array, HashMap, etc.), after
calling the stdlib function, rebind the receiver variable to the return value.

### VM: opcode for method calls

The VM has a `CallMethod` (or similar) opcode. After this opcode executes:
- Pop the return value from the stack
- If the call target was a local variable containing a collection, update that local
  to the return value

Check `vm/dispatch.rs` for the method call opcode and update accordingly.

---

## Method Categories

**Mutating array methods** (return new array):
`push`, `pop`, `sort`, `reverse`, `insert`, `remove`, `set`, `fill`, `extend`, `truncate`

**Non-mutating array methods** (return other types):
`len`, `get`, `includes`, `find`, `find_index`, `join`, `slice`, `concat`, `map`, `filter`, `reduce`

Note: `map`, `filter`, `reduce` return NEW arrays — they don't mutate the receiver.
The receiver variable should NOT be overwritten with their result.

**Rule:** Only overwrite the receiver variable when the called method's name is in the
mutating set. For non-mutating methods, return value is the result of the expression.

---

## Tests

```atlas
// push stores result back to variable:
let arr = [1, 2, 3]
arr.push(4)
assert(arr == [1, 2, 3, 4])  // variable updated

// pop stores result back AND returns popped element:
let arr = [1, 2, 3]
let popped = arr.pop()
assert(arr == [1, 2])
assert(popped == 3)

// non-mutating: receiver unchanged
let arr = [3, 1, 2]
let sorted = arr.sort()
assert(arr == [3, 1, 2])   // original unchanged
assert(sorted == [1, 2, 3]) // sorted is the new array
```

---

## Acceptance Criteria

- [ ] `arr.push(x)` updates `arr` in the calling scope (interpreter)
- [ ] `arr.pop()` updates `arr` and returns the popped element (interpreter)
- [ ] Same behavior in VM
- [ ] Non-mutating methods (`sort`, `filter`, `map`) do NOT modify receiver
- [ ] All push/pop/sort tests pass in both engines
- [ ] Interpreter and VM produce identical output (parity)
