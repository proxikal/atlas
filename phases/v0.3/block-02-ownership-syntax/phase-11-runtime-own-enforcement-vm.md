# Phase 11: Runtime — `own` Enforcement in VM (Debug Mode)

**Block:** 2 (Ownership Syntax)
**Depends on:** Phase 08 (interpreter own enforcement), Phase 10 (ownership in FunctionRef)
**Complexity:** medium
**Files to modify:**
- `crates/atlas-runtime/src/vm/mod.rs`

## Summary

Mirror Phase 08's interpreter `own` enforcement in the VM. When the VM calls a function
with `own` parameters, it marks the caller's local slot as "consumed" in debug mode.
Subsequent reads of that slot produce a runtime error. Parity with interpreter is required.

## Current State

Verified: The VM's function call handling uses `FunctionRef` from the bytecode constants.
After Phase 10, `FunctionRef.param_ownership` carries ownership per parameter.

The VM manages local variables as a stack frame of `Value` slots, not a HashMap. The
"consumed" state mechanism must be adapted to the VM's stack-based model.

## Requirements

1. **Consumed slot representation (debug only).** In debug builds, maintain a parallel
   `Vec<bool>` per call frame tracking which local slots are consumed. Length = `local_count`.
   In release builds, this vec is not allocated (conditional compilation).

2. **On `Call` opcode with `own` param:** For each argument at position `i` where
   `func_ref.param_ownership[i] == Some(Own)`:
   - If the argument was loaded via `GetLocal(slot)` or `GetGlobal(name)`, mark that
     slot/global as consumed in the CALLER'S frame (debug mode only).
   - If the argument is a literal or computed expression, no consume-marking needed.

3. **On `GetLocal`/`GetGlobal` for consumed slot (debug mode):**
   Check if the slot is consumed. If yes: runtime error
   `"use of moved value: local[{slot}] was passed to 'own' parameter"`.

4. **Release mode:** All consumed-slot tracking is `#[cfg(debug_assertions)]` only.

## Acceptance Criteria

- [ ] VM produces same ownership violation error as interpreter for same Atlas source
- [ ] `own` param call consumes the caller's local slot (debug mode)
- [ ] Reading consumed slot → runtime error (debug mode)
- [ ] In release mode, no overhead
- [ ] All existing VM tests continue to pass
- [ ] `cargo nextest run -p atlas-runtime` 100% passing

## Tests Required

```rust
#[test]
fn test_vm_own_consumes_local() {
    // Same Atlas source as Phase 08 test — VM mode
    // Verify same error, same message format
}

#[test]
fn test_vm_own_borrow_identical_to_interpreter() {
    // Run same source through interpreter AND VM
    // Both must produce identical error or identical success
}
```

## Notes

- The `GetLocal` instruction in the VM takes a slot index — use this as the consumed-slot
  key. For `GetGlobal`, use the name string.
- Tracking which argument came from which slot requires inspecting the bytecode sequence
  before the Call opcode. At the time of execution, the args are already on the stack — the
  VM needs to track this during argument evaluation, not at the Call instruction itself.
  One approach: emit a `MarkConsumed(slot)` opcode (debug builds only) immediately after
  each argument push for an `own` parameter. Phase 10 can emit these opcodes; Phase 11 executes them.
  Alternative: do the slot-tracking at the Call site by storing "origin slot" metadata on Values
  during debug builds. Either approach is valid — choose the one with less blast radius.
