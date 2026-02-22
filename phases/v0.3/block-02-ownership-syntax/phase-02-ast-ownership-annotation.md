# Phase 02: AST — OwnershipAnnotation + Param Update

**Block:** 2 (Ownership Syntax)
**Depends on:** Phase 01 (keyword tokens exist)
**Complexity:** medium
**Files to modify:**
- `crates/atlas-runtime/src/ast.rs`

## Summary

Define the `OwnershipAnnotation` enum and add an `ownership` field to `Param` and a
`return_ownership` field to `FunctionDecl`. This is the AST layer that all subsequent phases
(parser, typechecker, compiler, runtime) operate on.

## Current State

Verified: `Param` struct is:
```rust
pub struct Param {
    pub name: Identifier,
    pub type_ref: TypeRef,
    pub span: Span,
}
```

`FunctionDecl` has `return_type: TypeRef` but no ownership annotation on the return type.

`Param` is constructed in exactly two places:
- `src/parser/mod.rs:145`
- `tests/frontend_integration.rs:1635, 1643`

## Requirements

1. Define `OwnershipAnnotation` enum in `ast.rs`:
   ```rust
   #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
   pub enum OwnershipAnnotation {
       Own,     // own param: T  — move semantics, caller binding invalidated
       Borrow,  // borrow param: T — immutable reference, caller retains ownership
       Shared,  // shared param: T — Arc<Mutex<T>>, explicit shared mutability
   }
   ```

2. Add `ownership: Option<OwnershipAnnotation>` field to `Param`:
   ```rust
   pub struct Param {
       pub name: Identifier,
       pub type_ref: TypeRef,
       pub ownership: Option<OwnershipAnnotation>,  // None = unannotated (value type copy)
       pub span: Span,
   }
   ```

3. Add `return_ownership: Option<OwnershipAnnotation>` field to `FunctionDecl`:
   ```rust
   pub struct FunctionDecl {
       // ... existing fields ...
       pub return_ownership: Option<OwnershipAnnotation>,  // None = unannotated
       // ...
   }
   ```

4. Update the `FunctionDecl` default/test helper at `ast.rs:694` to include
   `return_ownership: None`.

5. Update the doc comment on `Param` to explain the ownership field.

## Acceptance Criteria

- [ ] `OwnershipAnnotation` enum exists with `Own`, `Borrow`, `Shared` variants
- [ ] `OwnershipAnnotation` derives `Debug, Clone, PartialEq, Serialize, Deserialize`
- [ ] `Param.ownership: Option<OwnershipAnnotation>` field exists
- [ ] `FunctionDecl.return_ownership: Option<OwnershipAnnotation>` field exists
- [ ] Existing code that constructs `Param { name, type_ref, span }` fails to compile
      (intentional — Phase 05 fixes all construction sites)
- [ ] `cargo build -p atlas-runtime` after Phase 05 is the verification gate
- [ ] All existing ast.rs unit tests pass (or are updated for new struct fields)

## Tests Required

Unit test in `ast.rs`:
```rust
#[test]
fn test_ownership_annotation_variants() {
    assert_eq!(OwnershipAnnotation::Own, OwnershipAnnotation::Own);
    assert_ne!(OwnershipAnnotation::Own, OwnershipAnnotation::Borrow);
    // Clone and Debug work
    let ann = OwnershipAnnotation::Shared;
    let _ = format!("{:?}", ann.clone());
}

#[test]
fn test_param_with_ownership() {
    let param = Param {
        name: Identifier { name: "data".to_string(), span: Span::new(0, 4) },
        type_ref: TypeRef::Named("number".to_string(), Span::new(6, 12)),
        ownership: Some(OwnershipAnnotation::Own),
        span: Span::new(0, 12),
    };
    assert_eq!(param.ownership, Some(OwnershipAnnotation::Own));
}
```

## Notes

- `ownership: None` on a `Param` means "unannotated" — the type-checker phase will
  determine behavior by type category (value type = implicit copy, resource type = error).
- This phase will cause compilation failures at every `Param { name, type_ref, span }`
  construction site. This is expected and correct. Phase 05 fixes them all systematically.
  Execute Phase 02 immediately before Phase 05 to minimize the window.
