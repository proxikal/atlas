# Phase 07: Type Checker — Call-Site Ownership Checking

**Block:** 2 (Ownership Syntax)
**Depends on:** Phase 06 (annotations stored in function types)
**Complexity:** medium-high
**Files to modify:**
- `crates/atlas-runtime/src/typechecker/expr.rs`
- `crates/atlas-runtime/src/typechecker/mod.rs` (if call resolution lives there)

## Summary

The type checker validates ownership at call sites: you cannot pass a `shared<T>` value
to a parameter expecting `own`, and you cannot pass a plain value to a `shared` parameter
(it must be wrapped). This is the compile-time diagnostic layer for ownership misuse.

## Current State

Verified: Call expression type checking is in `typechecker/expr.rs`. It validates argument
count and types but has no ownership awareness. After Phase 06, the resolved function type
carries ownership per parameter.

## Requirements

1. **`own` parameter call-site check:**
   - When calling `fn f(own x: T)`, the argument must be a value the caller CAN give up.
   - For v0.3 (no static liveness analysis), this check is: the argument expression must
     NOT be annotated as a borrowed value (i.e., not the result of dereffing a `borrow` param).
   - Full "binding invalidated after call" check is v0.4 (static dataflow). For v0.3, emit
     a warning if the argument is a `borrow` parameter being passed to an `own` param:
     `AT_BORROW_TO_OWN: "passing borrowed value to 'own' parameter — ownership cannot transfer"`.

2. **`shared` parameter call-site check:**
   - Passing a non-`Shared<T>` value to a `shared` parameter: error
     `AT_NON_SHARED_TO_SHARED: "expected shared<T> value for 'shared' parameter"`.
   - The value's type must be `Value::SharedValue(_)` (i.e., wrapped in `Shared<T>`).

3. **`borrow` parameter call-site check:**
   - Any value can be borrowed — no restriction on caller's value type.
   - No diagnostic emitted for `borrow` parameters at call sites (it is the safest mode).

4. **Annotation mismatch in higher-order functions:**
   - When a function value is passed as an argument where a function type is expected,
     ownership annotations on the function type's parameters must match. This is a best-effort
     check for v0.3; exact enforcement is v0.4.

## Acceptance Criteria

- [ ] Passing `borrow`-annotated value to `own` param emits `AT_BORROW_TO_OWN` warning
- [ ] Passing non-`shared<T>` value to `shared` param emits `AT_NON_SHARED_TO_SHARED` error
- [ ] Passing any value to `borrow` param emits no diagnostic
- [ ] Passing correctly-typed value to `own` param emits no diagnostic
- [ ] All existing type checker call-site tests continue to pass
- [ ] `cargo nextest run -p atlas-runtime` 100% passing

## Tests Required

```rust
#[test]
fn test_typechecker_borrow_to_own_warning() {
    // fn consumer(own data: array<number>) -> void
    // fn caller(borrow data: array<number>) -> void { consumer(data); }
    // expect AT_BORROW_TO_OWN warning
}

#[test]
fn test_typechecker_non_shared_to_shared_error() {
    // fn register(shared handler: array<number>) -> void
    // let arr = [1, 2, 3];
    // register(arr);  — arr is not shared<T>
    // expect AT_NON_SHARED_TO_SHARED error
}

#[test]
fn test_typechecker_borrow_param_accepts_any_value() {
    // fn read(borrow data: array<number>) -> number
    // let arr = [1, 2, 3];
    // read(arr);  — OK, no diagnostic
}

#[test]
fn test_typechecker_own_param_accepts_owned_value() {
    // fn consume(own data: array<number>) -> void
    // let arr = [1, 2, 3];
    // consume(arr);  — OK, arr is owned by caller
}
```

## Notes

- The "binding invalidated after call" semantic (caller cannot use `arr` after `consume(arr)`)
  is v0.4 static dataflow. v0.3 only catches the type-level mismatches listed above.
- This is intentional — it gives Atlas programs useful ownership diagnostics in v0.3 without
  requiring the full liveness analysis that v0.4 will add.
- New diagnostic codes: `AT_BORROW_TO_OWN` (warning), `AT_NON_SHARED_TO_SHARED` (error).
  Add these to the diagnostic registry in `diagnostic.rs`.
