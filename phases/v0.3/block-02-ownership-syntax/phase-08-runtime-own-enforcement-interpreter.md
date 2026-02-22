# Phase 08: Runtime — `own` Enforcement in Interpreter (Debug Mode)

**Block:** 2 (Ownership Syntax)
**Depends on:** Phase 06 (annotations on function types), Phase 05 (build green)
**Complexity:** medium
**Files to modify:**
- `crates/atlas-runtime/src/interpreter/mod.rs`

## Summary

In debug mode, the interpreter enforces `own` semantics at runtime: when a value is passed
to an `own` parameter, the caller's binding is marked as "consumed" (moved). Any subsequent
read of that binding in the same call frame produces a runtime error. This is the behavioral
ground truth that v0.4's static verifier must match.

## Current State

Verified: `interpreter/mod.rs:631` iterates `func.params` to bind arguments to local scope:
```rust
for (i, param) in func.params.iter().enumerate() {
    scope.insert(param.name.name.clone(), (arg.clone(), true));
}
```
There is no concept of "consumed" bindings. After this phase, a consumed binding still exists
in scope but attempts to read it produce a runtime error in debug mode.

## Requirements

1. **"Consumed" binding state.** Extend scope entries from `(Value, bool)` to a type that
   can also carry a "consumed" flag. Options:
   - Add a `consumed: bool` to the scope value tuple → `(Value, bool, bool)` where the
     second bool is mutability and third is consumed. Simple but verbose.
   - Introduce `enum ScopeEntry { Value(Value, bool), Consumed(String) }` where the String
     is the variable name for the error message. Cleaner.
   Choose the approach that minimizes blast radius to the rest of the interpreter.

2. **On `own` parameter binding:** After binding the argument to the callee's scope, mark
   the CALLER'S binding for the same variable as consumed:
   ```
   // pseudocode
   if param.ownership == Some(Own) {
       caller_scope.mark_consumed(arg_var_name);
   }
   ```
   This only applies when the argument is a direct variable reference (not a literal or
   expression result — those have no binding to consume).

3. **On variable read of a consumed binding:** When the interpreter reads a variable that
   is in "consumed" state, check `cfg!(debug_assertions)`:
   - **Debug mode:** `panic!` or return `Err` with message:
     `"use of moved value: '{name}' was passed to 'own' parameter and is no longer valid"`
   - **Release mode:** No check (zero cost).

4. **Only applies in debug mode.** Use `#[cfg(debug_assertions)]` guards so release builds
   have zero overhead.

## Acceptance Criteria

- [ ] After `consume(arr)` where `consume` has `own arr: array<number>`, accessing `arr`
      in the caller's scope produces a runtime error in debug mode
- [ ] In release mode (no debug_assertions), the same code runs without error (no check)
- [ ] Passing a literal to an `own` param does not attempt to consume any binding
- [ ] Passing an expression result to an `own` param does not attempt to consume any binding
- [ ] `borrow` and unannotated params do NOT mark the caller's binding as consumed
- [ ] All existing interpreter tests continue to pass
- [ ] `cargo nextest run -p atlas-runtime` 100% passing

## Tests Required

```rust
#[test]
fn test_own_param_consumes_binding_debug() {
    // Atlas source:
    // fn consume(own data: array<number>) -> void { }
    // let arr = [1, 2, 3];
    // consume(arr);
    // arr;  ← should error in debug mode
    let result = eval_debug(src);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("use of moved value"));
}

#[test]
fn test_borrow_param_does_not_consume() {
    // fn read(borrow data: array<number>) -> void { }
    // let arr = [1, 2, 3];
    // read(arr);
    // arr;  ← should be fine
}

#[test]
fn test_own_literal_arg_no_consume() {
    // fn consume(own data: array<number>) -> void { }
    // consume([1, 2, 3]);  ← literal, no binding to consume — OK
}
```

## Notes

- The "consumed" check is debug-assertions only. This is identical to how Rust's overflow
  checks and index bounds checks work — they exist in debug, are elided in release.
- v0.4 will make this a compile-time error instead of runtime. The runtime behavior here
  is the behavioral spec that the static verifier must replicate.
- Parity with VM is required — Phase 11 implements the same enforcement in the VM.
