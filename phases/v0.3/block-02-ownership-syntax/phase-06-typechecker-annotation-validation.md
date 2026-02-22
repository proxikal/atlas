# Phase 06: Type Checker — Annotation Validation on Function Declarations

**Block:** 2 (Ownership Syntax)
**Depends on:** Phase 03 + Phase 04 (annotations reach the AST)
**Complexity:** medium
**Files to modify:**
- `crates/atlas-runtime/src/typechecker/mod.rs`
- `crates/atlas-runtime/src/typechecker/expr.rs` (if function type construction lives there)

## Summary

The type checker learns about ownership annotations. It stores them in the function's type
signature, validates consistency, and emits diagnostics for obvious misuse (e.g., `own` on
a primitive type that has no notion of exclusive ownership).

## Current State

Verified: The type checker processes `FunctionDecl` params at `typechecker/mod.rs:202–230`.
It maps each `Param` to a resolved `AtlasType` and builds a function type. The `ownership`
field on `Param` is currently ignored (it didn't exist before this block).

## Requirements

1. **Store annotations in function type.** When the type checker builds a function's
   parameter type list, attach ownership information. Define a `TypedParam` or extend the
   existing parameter representation in the type environment:
   ```rust
   pub struct TypedParam {
       pub name: String,
       pub ty: AtlasType,
       pub ownership: Option<OwnershipAnnotation>,
   }
   ```
   Store this in `FunctionType` (or equivalent) so later phases can query it.

2. **Validate annotations on declaration:**
   - `own` on a primitive (`number`, `bool`, `string`): emit warning diagnostic
     `AT_OWN_ON_PRIMITIVE` — these are always copied, annotation has no effect.
   - `borrow` on a `shared<T>` type: emit warning — `shared<T>` already has reference
     semantics; `borrow shared<T>` is redundant.
   - Duplicate annotations (malformed AST): emit error.

3. **Store return ownership** on the resolved function type as well.

4. The type checker does NOT yet validate call sites — that is Phase 07. This phase
   only validates the function declaration itself.

## Acceptance Criteria

- [ ] Ownership annotations on params are visible in the resolved function type
- [ ] `own` annotation on `number` param produces a warning diagnostic
- [ ] `borrow` annotation on `shared<T>` param produces a warning diagnostic
- [ ] Valid annotations (`own data: array<number>`) produce no diagnostics
- [ ] `return_ownership` is stored on the resolved function type
- [ ] All existing type checker tests continue to pass
- [ ] `cargo nextest run -p atlas-runtime` 100% passing

## Tests Required

```rust
#[test]
fn test_typechecker_stores_own_annotation() {
    // typecheck: fn process(own data: array<number>) -> void
    // resolve function type, verify param[0].ownership == Some(Own)
}

#[test]
fn test_typechecker_warns_own_on_primitive() {
    // fn bad(own x: number) -> void
    // expect warning diagnostic AT_OWN_ON_PRIMITIVE
}

#[test]
fn test_typechecker_accepts_own_on_array() {
    // fn process(own data: array<number>) -> void — no diagnostic
}

#[test]
fn test_typechecker_accepts_borrow_annotation() {
    // fn read(borrow data: array<number>) -> number — no diagnostic
}
```

## Notes

- The full type for a function signature in Atlas should now carry ownership per parameter.
  This is the type-system representation that Phase 07 (call-site checking) queries.
- Do not block on Block 3 trait system for this phase. Ownership annotations are independent
  of traits — `Copy`/`Move` traits (Block 3) refine ownership inference, but the explicit
  annotation system is self-contained.
- Diagnostic codes to introduce: `AT_OWN_ON_PRIMITIVE` (warning), `AT_BORROW_ON_SHARED` (warning).
