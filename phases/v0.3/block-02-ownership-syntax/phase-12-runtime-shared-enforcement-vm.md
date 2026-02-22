# Phase 12: Runtime — `shared` Enforcement in VM (Debug Mode)

**Block:** 2 (Ownership Syntax)
**Depends on:** Phase 09 (interpreter shared enforcement), Phase 10 (ownership in FunctionRef)
**Complexity:** low
**Files to modify:**
- `crates/atlas-runtime/src/vm/mod.rs`

## Summary

Mirror Phase 09's interpreter `shared` enforcement in the VM. When the VM calls a function
with a `shared` parameter, assert in debug mode that the argument is `Value::SharedValue(_)`.
Same behavior, different engine.

## Current State

Verified: VM call handling in `vm/mod.rs`. After Phase 10, `FunctionRef.param_ownership`
is available. After Phase 11, the debug-mode ownership check infrastructure exists in the VM.

## Requirements

1. In the VM call handler, for each argument at position `i` where
   `func_ref.param_ownership[i] == Some(Shared)`:
   - Debug mode: assert the argument is `Value::SharedValue(_)`.
     On failure: runtime error
     `"ownership violation: parameter '{name}' expects shared<T> but received {type_name}"`
   - Release mode: no check.

2. Advisory warning (debug) when `Value::SharedValue(_)` is passed to `own`/`borrow` param:
   same behavior as interpreter Phase 09.

3. All checks are `#[cfg(debug_assertions)]` only.

## Acceptance Criteria

- [ ] VM produces same `shared` ownership error as interpreter for identical source
- [ ] Passing plain value to `shared` param → runtime error (debug mode VM)
- [ ] Passing `SharedValue` to `shared` param → no error
- [ ] In release mode: no check performed
- [ ] All existing VM tests continue to pass
- [ ] `cargo nextest run -p atlas-runtime` 100% passing

## Tests Required

```rust
#[test]
fn test_vm_shared_param_rejects_plain_value() {
    // Same source as Phase 09 interpreter test, run through VM
}

#[test]
fn test_vm_shared_param_accepts_shared_value() {
    // Same as Phase 09 acceptance case, VM mode
}
```

## Notes

- These checks are truly minimal — just pattern match `Value::SharedValue(_)` on the
  argument before binding it to the callee's frame. No complex infrastructure needed.
- After this phase, both engines enforce `own` and `shared` ownership contracts at runtime
  (debug mode). Phase 13 verifies they produce identical behavior.
