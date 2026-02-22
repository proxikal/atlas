# Phase 09: Runtime — `shared` Enforcement in Interpreter (Debug Mode)

**Block:** 2 (Ownership Syntax)
**Depends on:** Phase 08 (own enforcement pattern established)
**Complexity:** low
**Files to modify:**
- `crates/atlas-runtime/src/interpreter/mod.rs`

## Summary

When the interpreter calls a function with a `shared` annotated parameter, assert at runtime
(debug mode) that the argument IS a `Value::SharedValue(_)`. If not, emit a clear error.
This enforces the contract that `shared` params require explicit reference semantics.

## Current State

Verified: `interpreter/mod.rs:631` — param binding loop. After Phase 08 the loop is
ownership-aware. This phase adds one more check: `shared` params must receive a `SharedValue`.

The `Value::SharedValue(Shared<Box<Value>>)` variant exists (from Block 1). Verified in
`value.rs:457–459`.

## Requirements

1. In the param binding loop (same location as Phase 08), for each param where
   `ownership == Some(Shared)`:
   - In debug mode (`#[cfg(debug_assertions)]`): assert the argument is `Value::SharedValue(_)`.
     On failure: `Err` with message:
     `"ownership violation: parameter '{name}' expects shared<T> but received {type_name}"`
   - In release mode: no check.

2. This check fires BEFORE the value is bound to the callee's scope, so the error
   message shows the caller's type correctly.

3. Inverse check: if the argument IS a `Value::SharedValue(_)` and the param is annotated
   `own` or `borrow`, emit a warning in debug mode:
   `"passing shared<T> value to '{own|borrow}' parameter — consider using the 'shared' annotation"`
   This is advisory only, not a hard error.

## Acceptance Criteria

- [ ] Passing `array<number>` (non-shared) to `shared` param produces runtime error (debug)
- [ ] Passing `Value::SharedValue(_)` to `shared` param succeeds with no error
- [ ] Passing `Value::SharedValue(_)` to `own` param emits advisory warning (debug)
- [ ] In release mode, no check is performed (zero overhead)
- [ ] All existing interpreter tests continue to pass
- [ ] `cargo nextest run -p atlas-runtime` 100% passing

## Tests Required

```rust
#[test]
fn test_shared_param_rejects_plain_value_debug() {
    // fn register(shared handler: array<number>) -> void { }
    // let arr = [1, 2, 3];
    // register(arr);  ← arr is not shared<T>
    // expect: ownership violation error
}

#[test]
fn test_shared_param_accepts_shared_value() {
    // let arr = shared([1, 2, 3]);   ← creates Shared<T>
    // fn register(shared handler: array<number>) -> void { }
    // register(arr);  ← OK
}

#[test]
fn test_shared_value_to_own_param_advisory() {
    // let arr = shared([1, 2, 3]);
    // fn consume(own handler: array<number>) -> void { }
    // consume(arr);  ← advisory warning in debug mode
}
```

## Notes

- The `shared(value)` constructor syntax for creating `Value::SharedValue` must already
  work (it was implemented in Block 1 Phase 16+). Verify this at execution time.
- Both Phase 08 and Phase 09 are interpreter-only. VM counterparts are Phase 11 and 12.
  Parity test is Phase 13.
