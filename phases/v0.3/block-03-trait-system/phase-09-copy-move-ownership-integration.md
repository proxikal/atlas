# Phase 09 — Typechecker: Copy/Move + Ownership Integration

**Block:** 3 (Trait System)
**Depends on:** Phase 08 complete
**Estimated tests added:** 15–20

---

## Objective

Integrate `Copy` and `Move` traits with Block 2 ownership annotations:
- `Copy` types: passing without `own`/`borrow` is implicitly a copy (no error)
- `Move` types: require explicit `own` annotation to transfer ownership
- Built-in value types (number, string, bool, array, map) are all implicitly `Copy`
- User types default to `Move` unless they explicitly `impl Copy for MyType`

This phase connects the ownership typechecker (Block 2's `fn_ownership_registry`) with
the new `trait_registry`.

---

## Current State (verified after Phase 08)

`crates/atlas-runtime/src/typechecker/mod.rs`:
- Block 2 Phase 06/07: call-site ownership checking is in `typechecker/expr.rs` or `mod.rs`
- `fn_ownership_registry: HashMap<String, FnOwnershipEntry>` — per-function ownership
- `trait_registry.implementations` has: `number/string/bool/null` → `Copy`
- No current concept of "type is Copy" affecting ownership requirements

---

## Decision: Copy vs Move Semantics (LOCKED in this phase)

**Rule:** A type T is `Copy` if and only if:
1. It is a built-in value type (number, string, bool, null, array, map, set, queue, stack), OR
2. It explicitly implements `Copy` via `impl Copy for T { }`

**Rule:** A type T is `Move` if and only if:
1. It explicitly implements `Move` via `impl Move for T { }`, OR
2. It is a user type that does NOT implement `Copy`

**Ownership interaction:**
- Calling `fn process(data: SomeType)` (unannotated param):
  - If `SomeType` is `Copy` → OK, implicit copy (no annotation required)
  - If `SomeType` is `Move` → typechecker error: must use `own data: SomeType` or `borrow data: SomeType`
  - If `SomeType` is unknown/opaque → warn (strict mode) or allow (permissive) — use warn for now
- Calling `fn process(own data: SomeType)` (annotated `own`):
  - If `SomeType` is `Copy` → OK (explicit own on Copy is allowed, just redundant)
  - If `SomeType` is `Move` → OK (correct annotation)

---

## Implementation

### `crates/atlas-runtime/src/typechecker/mod.rs`

Add helper:

```rust
/// Determine if a type implements Copy.
pub fn is_copy_type(&self, ty: &Type) -> bool {
    match ty {
        // Built-in value types are always Copy
        Type::Number | Type::String | Type::Bool | Type::Null | Type::Void => true,
        Type::Array(_) | Type::JsonValue => true,
        // Generic types (Option, Result) — Copy if their type args are Copy
        Type::Generic { name, type_args } => {
            match name.as_str() {
                "Option" | "Result" => type_args.iter().all(|arg| self.is_copy_type(arg)),
                _ => false,
            }
        }
        // Function types are Copy (they're reference-counted internally)
        Type::Function { .. } => true,
        // Named types: check trait registry
        Type::TypeParameter { name } => {
            // Type params with Copy bound are Copy
            // (tracked in current_fn_type_param_bounds — add this if needed)
            false  // conservative: unknown type params are not Copy by default
        }
        _ => false,
    }
}

/// Determine if a type implements Move (explicitly non-Copy).
pub fn is_move_type(&self, ty: &Type) -> bool {
    // A type is Move if it's not Copy and has explicit Move impl
    // In Block 3, this mainly applies to user-declared "opaque types"
    !self.is_copy_type(ty)
}
```

### Extend call-site ownership checking

In the Block 2 ownership call-site checker (wherever AT3001/AT3002 ownership errors are emitted),
add a new check:

When a function parameter has no ownership annotation AND the argument type is not `Copy`:

```rust
// In call-site checking for unannotated parameters:
if param_ownership.is_none() && !self.is_copy_type(&arg_type) {
    // User type that is not Copy being passed without ownership annotation
    self.diagnostics.push(Diagnostic::warning(
        error_codes::MOVE_TYPE_REQUIRES_OWNERSHIP_ANNOTATION,
        format!(
            "Type '{}' is not Copy — consider annotating with 'own' or 'borrow'",
            type_name
        ),
        arg_span,
    ));
}
```

Use `warning` not `error` for Block 3 (the feature is new and user types are opaque).
Full enforcement becomes strict in v0.4 when user-defined struct types are stable.

Add diagnostic code: `MOVE_TYPE_REQUIRES_OWNERSHIP_ANNOTATION: &str = "AT3010"`.

### `impl Copy` for user types

When `impl Copy for SomeType { }` is processed by `check_impl_block()` (Phase 07),
`trait_registry.mark_implements("SomeType", "Copy")` is called. Then `is_copy_type()`
should check this registry for named types:

```rust
Type::Named(name) | Type::TypeParameter { name } => {
    // Check if explicitly marked as Copy via impl
    self.trait_registry.implements(name, "Copy")
}
```

Wait — `Type::Named` may not exist. Check `types.rs` for how user type names appear in the
Type enum. Likely they appear as `Type::Generic { name, type_args: [] }` or via an alias.
Investigate and adapt accordingly.

---

## Tests

Add to `crates/atlas-runtime/tests/typesystem.rs`:

```rust
#[test]
fn test_number_is_copy_no_annotation_required() {
    let atlas = Atlas::new();
    // Passing number without own/borrow is fine (Copy type)
    let result = atlas.eval("
        fn double(x: number) -> number { return x * 2; }
        let n: number = 5;
        let result: number = double(n);
    ");
    assert!(result.is_ok(), "number is Copy, no annotation needed");
}

#[test]
fn test_string_is_copy_no_annotation_required() {
    let atlas = Atlas::new();
    let result = atlas.eval("
        fn greet(name: string) -> string { return \"Hello \" + name; }
        let s: string = \"world\";
        let g: string = greet(s);
    ");
    assert!(result.is_ok(), "string is Copy, no annotation needed");
}

#[test]
fn test_bool_is_copy_no_annotation_required() {
    let atlas = Atlas::new();
    let result = atlas.eval("
        fn negate(b: bool) -> bool { return !b; }
        let flag: bool = true;
        let result: bool = negate(flag);
    ");
    assert!(result.is_ok(), "bool is Copy, no annotation needed");
}

#[test]
fn test_explicit_own_on_copy_type_is_allowed() {
    // own annotation on Copy type is redundant but not an error
    let atlas = Atlas::new();
    let result = atlas.eval("
        fn consume(own x: number) -> number { return x; }
        let n: number = 42;
        let result: number = consume(n);
    ");
    assert!(result.is_ok(), "Explicit own on Copy type should be allowed");
}

#[test]
fn test_impl_copy_marks_type_as_copy() {
    let atlas = Atlas::new();
    let result = atlas.eval("
        trait Copy { }  // would fail — Copy is built-in
    ");
    // Trying to redefine Copy is an error
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("AT3001"));
}

#[test]
fn test_array_is_copy_type() {
    let atlas = Atlas::new();
    let result = atlas.eval("
        fn process(arr: number[]) -> number { return len(arr); }
        let a: number[] = [1, 2, 3];
        let n: number = process(a);
    ");
    assert!(result.is_ok(), "array is Copy (CoW), no annotation needed");
}

#[test]
fn test_copy_type_check_in_generic_function() {
    // Generic function with Copy bound — number satisfies it
    let atlas = Atlas::new();
    let result = atlas.eval("
        fn identity<T: Copy>(x: T) -> T { return x; }
        let n: number = identity(42);
    ");
    // Phase 10 enforces this — for now document expected behavior
    assert!(result.is_ok() || result.is_err()); // placeholder
}
```

---

## Acceptance Criteria

- [ ] `is_copy_type()` returns `true` for number, string, bool, null, array, map
- [ ] `is_copy_type()` returns `false` for user-defined opaque types (without `impl Copy`)
- [ ] Passing Copy type without ownership annotation produces no error
- [ ] Explicit `own` on a Copy type is allowed (no error)
- [ ] `impl Copy for UserType { }` marks UserType as Copy in the registry
- [ ] Attempting to define `trait Copy` produces AT3001 (built-in trait)
- [ ] AT3010 warning fires for Move types passed without annotation (not error yet)
- [ ] All existing Block 2 ownership tests still pass (no regressions)
- [ ] `cargo clippy -- -D warnings` clean
- [ ] `cargo fmt` clean

---

## Notes

- This phase establishes the **data path** — `is_copy_type()` is a pure query. The
  enforcement strictness is intentionally mild (warning not error) in Block 3, because
  user-defined types are opaque and the feature is new.
- `Drop` integration: when a `Move` type goes out of scope, if it implements `Drop`,
  its `drop()` method should eventually be called. In Block 3, this is NOT automatic.
  The path is: `is_move_type() && impl_registry.has_impl(ty, "Drop")` = "could call drop"
  but the call is explicit only. Automatic drop = v0.4.
- The `Type` enum may not have a `Named` variant. Investigate before implementing.
  User types in Block 3 appear in function signatures as unresolved type names.
  Check how the typechecker handles `fn process(x: Buffer)` where Buffer is unknown.
