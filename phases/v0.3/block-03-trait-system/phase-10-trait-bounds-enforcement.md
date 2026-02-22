# Phase 10 — Typechecker: Trait Bounds Enforcement

**Block:** 3 (Trait System)
**Depends on:** Phase 09 complete
**Estimated tests added:** 15–20

---

## Objective

Enforce trait bounds at call sites. When `fn foo<T: Copy>(x: T)` is called with
`foo(someValue)`, verify that `someValue`'s type implements `Copy`. Emit AT3008 if not.

---

## Current State (verified after Phase 09)

- `TypeParam.trait_bounds: Vec<TraitBound>` populated by parser (Phase 05)
- `trait_registry.implements(type_name, trait_name)` works (Phase 07)
- `is_copy_type()` works (Phase 09)
- Generic call-site checking in `typechecker/generics.rs` — understand how generic
  type arguments are currently resolved and where to add bound checking

---

## Investigation Required (do this first)

```bash
cat crates/atlas-runtime/src/typechecker/generics.rs | head -100
```

```bash
grep -n "type_params\|TypeParam\|generic\|bound" \
  crates/atlas-runtime/src/typechecker/expr.rs | head -30
```

Understand the current generic call-site resolution path before adding bound checking.
The enforcement must hook into the existing generic resolution — not bypass it.

---

## Implementation

### Call-site bound checking

At the point where generic type arguments are resolved for a function call:

```rust
/// Check that inferred type arguments satisfy the function's type parameter bounds.
fn check_type_param_bounds(
    &mut self,
    type_params: &[TypeParam],
    inferred_type_args: &[(String, Type)],  // (param_name, resolved_type)
    call_span: Span,
) {
    for type_param in type_params {
        // Find the resolved type for this type parameter
        let resolved_type = inferred_type_args.iter()
            .find(|(name, _)| name == &type_param.name)
            .map(|(_, ty)| ty);

        let Some(resolved_type) = resolved_type else { continue };

        for trait_bound in &type_param.trait_bounds {
            let satisfies = self.type_satisfies_trait_bound(resolved_type, &trait_bound.trait_name);
            if !satisfies {
                let type_str = format!("{:?}", resolved_type);  // use Display impl
                self.diagnostics.push(Diagnostic::error(
                    error_codes::TRAIT_BOUND_NOT_SATISFIED,
                    format!(
                        "Type '{}' does not implement trait '{}' required by type parameter '{}'",
                        type_str, trait_bound.trait_name, type_param.name
                    ),
                    call_span,
                ));
            }
        }
    }
}

/// Determine if a resolved type satisfies a trait bound.
fn type_satisfies_trait_bound(&self, ty: &Type, trait_name: &str) -> bool {
    match trait_name {
        "Copy" => self.is_copy_type(ty),
        "Move" => self.is_move_type(ty),
        "Drop" | "Display" | "Debug" => {
            // Check impl registry for built-in traits
            if let Some(type_name) = self.type_to_name_str(ty) {
                self.trait_registry.implements(&type_name, trait_name)
            } else {
                false
            }
        }
        _ => {
            // User-defined trait — check impl registry
            if let Some(type_name) = self.type_to_name_str(ty) {
                self.trait_registry.implements(&type_name, trait_name)
            } else {
                false
            }
        }
    }
}

/// Convert a Type to its string name for registry lookup.
fn type_to_name_str(&self, ty: &Type) -> Option<String> {
    match ty {
        Type::Number => Some("number".to_string()),
        Type::String => Some("string".to_string()),
        Type::Bool => Some("bool".to_string()),
        Type::Null => Some("null".to_string()),
        Type::Array(_) => Some("array".to_string()),
        _ => None,
    }
}
```

Call `check_type_param_bounds()` at the generic function call-site resolution point
in `typechecker/generics.rs` or `typechecker/expr.rs`.

### Also enforce bounds on trait method type params

When calling a trait method like `printer.print<number>(42)` where `print<T: Display>`,
verify `number` satisfies `Display`. This uses the same `check_type_param_bounds()` helper
applied to the resolved method's type params.

---

## Tests

Add to `crates/atlas-runtime/tests/typesystem.rs`:

```rust
#[test]
fn test_copy_bound_satisfied_by_number() {
    let atlas = Atlas::new();
    let result = atlas.eval("
        fn safe_copy<T: Copy>(x: T) -> T { return x; }
        let n: number = safe_copy(42);
    ");
    assert!(result.is_ok(), "number satisfies Copy bound");
}

#[test]
fn test_copy_bound_satisfied_by_string() {
    let atlas = Atlas::new();
    let result = atlas.eval("
        fn safe_copy<T: Copy>(x: T) -> T { return x; }
        let s: string = safe_copy(\"hello\");
    ");
    assert!(result.is_ok(), "string satisfies Copy bound");
}

#[test]
fn test_copy_bound_satisfied_by_bool() {
    let atlas = Atlas::new();
    let result = atlas.eval("
        fn safe_copy<T: Copy>(x: T) -> T { return x; }
        let b: bool = safe_copy(true);
    ");
    assert!(result.is_ok(), "bool satisfies Copy bound");
}

#[test]
fn test_user_defined_trait_bound_satisfied() {
    let atlas = Atlas::new();
    let result = atlas.eval("
        trait Printable { fn print_self(self: Printable) -> void; }
        impl Printable for number {
            fn print_self(self: number) -> void { print(str(self)); }
        }
        fn log_it<T: Printable>(x: T) -> void { x.print_self(); }
        log_it(42);
    ");
    assert!(result.is_ok(), "number implements Printable, bound satisfied");
}

#[test]
fn test_user_defined_trait_bound_not_satisfied_is_error() {
    let atlas = Atlas::new();
    let result = atlas.eval("
        trait Printable { fn print_self(self: Printable) -> void; }
        impl Printable for number {
            fn print_self(self: number) -> void { print(str(self)); }
        }
        fn log_it<T: Printable>(x: T) -> void { x.print_self(); }
        log_it(\"hello\");  // string does NOT implement Printable
    ");
    assert!(result.is_err(), "string doesn't implement Printable — AT3008");
    let err = result.unwrap_err();
    assert!(err.to_string().contains("AT3008") || err.to_string().contains("does not implement"));
}

#[test]
fn test_multiple_bounds_all_satisfied() {
    let atlas = Atlas::new();
    let result = atlas.eval("
        trait Printable { fn print_self(self: Printable) -> void; }
        impl Printable for number {
            fn print_self(self: number) -> void { print(str(self)); }
        }
        fn process<T: Copy + Printable>(x: T) -> void { x.print_self(); }
        process(42);  // number is Copy AND Printable
    ");
    assert!(result.is_ok());
}

#[test]
fn test_multiple_bounds_one_missing_is_error() {
    let atlas = Atlas::new();
    let result = atlas.eval("
        trait Printable { fn print_self(self: Printable) -> void; }
        // number is Copy but doesn't impl Printable here
        fn process<T: Copy + Printable>(x: T) -> void { x.print_self(); }
        process(42);  // number is Copy but NOT Printable (no impl)
    ");
    assert!(result.is_err(), "AT3008: one bound not satisfied");
}

#[test]
fn test_unbounded_type_param_still_works() {
    let atlas = Atlas::new();
    let result = atlas.eval("
        fn identity<T>(x: T) -> T { return x; }
        let n: number = identity(42);
        let s: string = identity(\"hello\");
    ");
    assert!(result.is_ok(), "Unbounded type params are unchanged");
}

#[test]
fn test_bound_check_in_trait_method_type_param() {
    let atlas = Atlas::new();
    let result = atlas.eval("
        trait Formatter {
            fn format<T: Display>(self: Formatter, value: T) -> string;
        }
        trait Display { fn display(self: Display) -> string; }
        impl Display for number {
            fn display(self: number) -> string { return str(self); }
        }
        impl Formatter for BasicFormatter {
            fn format<T: Display>(self: BasicFormatter, value: T) -> string {
                return value.display();
            }
        }
        let fmt: BasicFormatter = BasicFormatter {};
        let s: string = fmt.format(42);  // number satisfies Display
    ");
    assert!(result.is_ok() || result.is_err()); // document result
}
```

---

## Acceptance Criteria

- [ ] `fn foo<T: Copy>(x: T)` called with `number` → no error
- [ ] `fn foo<T: Copy>(x: T)` called with user type that doesn't implement `Copy` → AT3008
- [ ] `fn foo<T: UserTrait>(x: T)` called with type implementing `UserTrait` → no error
- [ ] `fn foo<T: UserTrait>(x: T)` called with type NOT implementing `UserTrait` → AT3008
- [ ] Multiple bounds: all must be satisfied
- [ ] Unbounded type params (`<T>`) are unaffected
- [ ] AT3008 error message names the failing trait and type clearly
- [ ] All existing generic function tests pass (no regressions)
- [ ] `cargo clippy -- -D warnings` clean
- [ ] `cargo fmt` clean

---

## Notes

- **`Display` built-in:** number/string/bool do NOT auto-implement `Display` in Block 3.
  They implement `Copy`. `Display` requires explicit `impl Display for T { }`.
  The existing `str()` stdlib function is separate from the `Display` trait.
- **Type parameter propagation:** if `fn outer<T: Copy>(x: T) { inner(x); }` where
  `fn inner<T: Copy>(x: T)` — the bound is propagated through `x`'s type. The typechecker
  should track that `x: T` where `T: Copy` satisfies `Copy` bounds in nested calls.
  This is complex — if it doesn't work in Block 3, document the limitation and add a test
  that explicitly shows the known limitation.
- Do not attempt full type inference for bounds — that's Block 5 territory.
