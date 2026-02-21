# Phase Correctness-04: Parity — Callback Intrinsics

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Correctness-03 complete. Value::Builtin variant exists.

**Verification:**
```bash
cargo check -p atlas-runtime 2>&1 | grep -c "error"  # must be 0
grep "Value::Builtin" crates/atlas-runtime/src/value.rs | wc -l  # > 0
```

---

## Objective

Interpreter/VM parity is not optional — it is a contract. Two concrete parity breaks exist in the callback-based intrinsics shared across both engines:

**Break 1 — NativeFunction callbacks:** The interpreter's `call_value` helper (used by every callback intrinsic) matches only `Value::Function` and `Value::Builtin`, returning `TypeError` for `Value::NativeFunction`. The VM's equivalent `vm_call_function_value` handles `NativeFunction` correctly. Passing a native Rust closure as a callback to `map()`, `filter()`, or any of the 17 intrinsics produces different results between engines.

**Break 2 — Callback type validation:** The interpreter validates the callback argument type before beginning iteration (returns early with a descriptive `TypeError`). The VM skips this validation and only fails when the callback is actually invoked. For invalid programs, the interpreter and VM produce different error messages and potentially different error types.

Both engines must be **identical** for all inputs, including invalid ones. Error messages for the same invalid program must match exactly. This phase makes both engines agree on every callback intrinsic.

---

## Files Changed

- `crates/atlas-runtime/src/interpreter/expr.rs` — add `Value::NativeFunction` arm to `call_value`; review all 17 intrinsic callback validation paths
- `crates/atlas-runtime/src/vm/mod.rs` — add callback type validation to all 17 `vm_intrinsic_*` functions to match interpreter behavior

---

## Dependencies

- Correctness-03 complete (`Value::Builtin` exists — call_value must handle it too)

---

## Implementation

### Step 1: Fix NativeFunction in interpreter call_value

In `interpreter/expr.rs`, `call_value` currently:
```rust
match func {
    Value::Function(func_ref) => { ... }
    Value::Builtin(name) => { ... }  // added in Correctness-03
    _ => Err(RuntimeError::TypeError { msg: "Expected function value".to_string(), span })
}
```

Add the missing arm:
```rust
Value::NativeFunction(native_fn) => native_fn(&args),
```

This is the same implementation as in `eval_call` and `vm_call_function_value`. All three call paths now handle NativeFunction identically.

### Step 2: Audit callback validation — establish canonical behavior

Read through all 17 callback intrinsics in both engines side-by-side. For each intrinsic, establish the canonical validation rule: what argument count is required, and what type must the callback argument be? Document this in a comment at the top of each intrinsic pair.

The canonical rule (to match interpreter behavior) is: validate callback type before beginning iteration. The interpreter already does this for the array intrinsics. The VM must match exactly.

### Step 3: Add callback validation to all 17 VM intrinsics

For each `vm_intrinsic_*` function that takes a callback argument, add validation before iteration. The validation pattern:

```rust
// Validate callback is callable
match &args[1] {
    Value::Function(_) | Value::Builtin(_) | Value::NativeFunction(_) => {}
    _ => return Err(RuntimeError::TypeError {
        msg: "map() second argument must be a function".to_string(),
        span,
    }),
}
```

The error message must match the interpreter's message for the same function exactly. Copy the message strings from the interpreter — do not paraphrase.

The 17 intrinsics to cover:
- Array: `map`, `filter`, `reduce`, `forEach`, `find`, `findIndex`, `flatMap`, `some`, `every`, `sort`, `sortBy`
- Result: `result_map`, `result_map_err`, `result_and_then`, `result_or_else`
- HashMap: `hashMapForEach`, `hashMapMap`, `hashMapFilter`
- HashSet: `hashSetForEach`, `hashSetMap`, `hashSetFilter`
- Regex: `regexReplaceWith`, `regexReplaceAllWith`

Wait — that is more than 17. Count every intrinsic in both engines and ensure every one is covered.

### Step 4: Align interpreter intrinsics with canonical rule

After establishing canonical behavior, review the interpreter intrinsics to confirm they ALL follow the same validation pattern. Some may validate, some may not — ensure they all do, consistently.

### Step 5: Write parity tests for each intrinsic

For each callback intrinsic, write a parity test that runs the same invalid program through both engines and asserts identical errors. Use the existing `assert_parity()` helper from `tests/common/mod.rs`.

```rust
#[test]
fn test_map_invalid_callback_parity() {
    assert_parity(r#"map([1,2,3], "not a function")"#);
    // Both engines must return the same error
}
```

Write at least one parity test per intrinsic.

---

## Tests

- `test_native_function_as_callback_interpreter` — passes a NativeFunction to `map()` in interpreter, returns correct result
- `test_native_function_as_callback_vm` — same for VM
- `test_native_function_as_callback_parity` — both produce same result
- One parity test per callback intrinsic for invalid callback argument
- All existing tests pass
- `cargo nextest run -p atlas-runtime` green

---

## Integration Points

- `interpreter/expr.rs` — `call_value` gains NativeFunction arm; intrinsics gain consistent validation
- `vm/mod.rs` — all `vm_intrinsic_*` functions gain callback validation
- `tests/interpreter.rs` or `tests/core.rs` — parity tests added

---

## Acceptance

- `call_value` in interpreter handles `Value::NativeFunction`
- All callback intrinsics in both engines validate callback type before iteration
- Error messages for invalid callbacks are **identical** between interpreter and VM
- Parity tests pass for all callback intrinsics (invalid callback argument case)
- Passing a `NativeFunction` as a callback produces correct results in both engines
- All existing tests pass
- Zero clippy warnings
- Commit: `fix(parity): Align callback intrinsics — NativeFunction support + validation parity`
