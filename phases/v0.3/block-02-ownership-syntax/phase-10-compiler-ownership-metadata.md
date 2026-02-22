# Phase 10: Compiler — Ownership Metadata in Bytecode

**Block:** 2 (Ownership Syntax)
**Depends on:** Phase 05 (build green), Phase 06 (annotations in type system)
**Complexity:** medium
**Files to modify:**
- `crates/atlas-runtime/src/compiler/mod.rs`
- `crates/atlas-runtime/src/value.rs` (`FunctionRef` struct)
- `crates/atlas-runtime/src/bytecode/` (if bytecode serialization must change)

## Summary

Propagate ownership annotations from `FunctionDecl.params` through the compiler into
`FunctionRef` and the bytecode, so the VM has access to ownership information at runtime.
Without this, Phase 11/12 (VM enforcement) have no data to work with.

## Current State

Verified: `FunctionRef` in `value.rs:464`:
```rust
pub struct FunctionRef {
    pub name: String,
    pub arity: usize,
    pub bytecode_offset: usize,
    pub local_count: usize,
}
```
The compiler creates `FunctionRef` at `compiler/mod.rs:199, 280`. Param ownership is not
stored anywhere in the bytecode or `FunctionRef`.

## Requirements

1. **Add ownership info to `FunctionRef`:**
   ```rust
   pub struct FunctionRef {
       pub name: String,
       pub arity: usize,
       pub bytecode_offset: usize,
       pub local_count: usize,
       // Ownership annotation per parameter, in parameter order.
       // None = unannotated (value type copy semantics)
       pub param_ownership: Vec<Option<OwnershipAnnotation>>,
       pub return_ownership: Option<OwnershipAnnotation>,
   }
   ```

2. **Populate during compilation:** In `compiler/mod.rs` where `FunctionRef` is constructed,
   populate `param_ownership` from `func.params[i].ownership` for each param.

3. **Backward compatibility:** All existing `FunctionRef` construction sites that predate
   Block 2 (builtins, stdlib, etc.) use `param_ownership: vec![]` and `return_ownership: None`.
   This is correct — stdlib functions are unannotated.

4. **Bytecode serialization:** `OwnershipAnnotation` must be serializable. Add serialization
   support to `bytecode/serialize.rs` for `Option<OwnershipAnnotation>` per parameter.
   Map: `None → 0`, `Some(Own) → 1`, `Some(Borrow) → 2`, `Some(Shared) → 3`.

5. **`ClosureRef`** inherits from `FunctionRef` — no extra work needed there.

## Acceptance Criteria

- [ ] `FunctionRef.param_ownership` exists and is populated during compilation
- [ ] `FunctionRef.return_ownership` exists and is populated during compilation
- [ ] Compiling `fn process(own data: array<number>) -> void` produces `FunctionRef`
      with `param_ownership[0] = Some(Own)`
- [ ] Compiling unannotated functions produces `param_ownership: vec![]` / `[]`
- [ ] Bytecode serialization/deserialization round-trips ownership correctly
- [ ] All existing compiler tests continue to pass
- [ ] `cargo nextest run -p atlas-runtime` 100% passing

## Tests Required

```rust
#[test]
fn test_compiler_emits_own_annotation() {
    // Compile: fn process(own data: array<number>) -> void { }
    // Extract FunctionRef from bytecode constants
    // Verify param_ownership[0] == Some(Own)
}

#[test]
fn test_compiler_emits_mixed_annotations() {
    // fn f(own a: number, borrow b: string, c: bool) -> void
    // param_ownership == [Some(Own), Some(Borrow), None]
}

#[test]
fn test_compiler_unannotated_function() {
    // fn f(x: number) -> number { return x; }
    // param_ownership == [None]
}

#[test]
fn test_bytecode_round_trips_ownership() {
    // compile → serialize → deserialize → verify FunctionRef ownership preserved
}
```

## Notes

- `OwnershipAnnotation` needs `Serialize`/`Deserialize` from Phase 02. Verify they derive
  correctly when running this phase.
- The bytecode format change is backward-incompatible for serialized bytecode files. This
  is acceptable — Atlas does not guarantee bytecode stability in v0.x.
