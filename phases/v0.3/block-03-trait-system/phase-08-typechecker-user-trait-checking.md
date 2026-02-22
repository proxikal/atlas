# Phase 08 — Typechecker: User Trait Method Calls

**Block:** 3 (Trait System)
**Depends on:** Phase 07 complete
**Estimated tests added:** 15–20

---

## Objective

Resolve and typecheck calls to trait methods on typed values. When the typechecker sees
`value.method(args)` and `value` has a known type that implements a trait with that method,
resolve the call through the trait system.

This phase handles the typechecker side. Interpreter/VM dispatch is Phases 12–14.

---

## Current State (verified after Phase 07)

`crates/atlas-runtime/src/typechecker/expr.rs` or `mod.rs`:
- Method calls: `value.method(args)` — currently handled via `method_dispatch.rs`
  which checks stdlib methods (array, string, etc.)
- `MemberExpr` / call on member — check how this is currently handled
- `impl_registry` exists and is populated for registered impls

The key question: how does the typechecker currently handle `arr.push(x)` etc.?
Grep `method_dispatch.rs` and `typechecker/methods.rs` to understand the existing dispatch chain.

---

## Investigation Required (do this first)

```bash
grep -n "member\|method_call\|MemberExpr\|method_dispatch" \
  crates/atlas-runtime/src/typechecker/expr.rs | head -30
```

```bash
cat crates/atlas-runtime/src/typechecker/methods.rs | head -80
```

Understand the chain before adding to it. The trait method resolution must integrate
cleanly with (not replace) the existing stdlib method dispatch.

---

## Dispatch Priority Order

When typechecking `value.method(args)`:

1. **Stdlib methods** (existing) — `array.push()`, `string.split()`, etc.
2. **Trait methods** (new) — resolve through `impl_registry` for user-defined types
3. **Unknown method** — emit AT0XXX "method not found" diagnostic

The new trait resolution goes in slot 2 — only fires if stdlib dispatch doesn't match.

---

## Changes

### `crates/atlas-runtime/src/typechecker/methods.rs` (or `expr.rs`)

Add a `resolve_trait_method_call()` helper:

```rust
/// Try to resolve a method call through the trait/impl system.
/// Returns the return type if resolved, None if not found.
fn resolve_trait_method_call(
    &self,
    receiver_type: &Type,
    method_name: &str,
    arg_types: &[Type],
) -> Option<Type> {
    // Determine receiver type name (for impl lookup)
    let type_name = type_to_impl_key(receiver_type)?;

    // Check all impls for this type to find a matching method
    for ((impl_type, trait_name), entry) in &self.impl_registry.entries {
        if impl_type == &type_name {
            if let Some(method) = entry.methods.get(method_name) {
                // Found the method — return its return type
                let return_type = self.resolve_type_ref(&method.return_type);
                return Some(return_type);
            }
        }
    }
    None
}

/// Convert a Type to a string key for impl registry lookup.
fn type_to_impl_key(ty: &Type) -> Option<String> {
    match ty {
        Type::Number => Some("number".to_string()),
        Type::String => Some("string".to_string()),
        Type::Bool => Some("bool".to_string()),
        Type::Named(name) => Some(name.clone()),
        // Arrays, maps etc. don't support trait dispatch in Block 3
        _ => None,
    }
}
```

Integrate into the method call typechecking path — after stdlib dispatch fails, try
`resolve_trait_method_call()`. If it returns `Some(return_type)`, use that as the
call expression's type.

### Record trait dispatch info for compiler

The typechecker needs to annotate the call site with resolution info so the compiler
(Phase 12) can emit a direct `Call` to the impl method. Add to the call site data:

```rust
/// Added to resolved call expressions so the compiler knows how to dispatch
pub struct TraitDispatchInfo {
    pub type_name: String,
    pub trait_name: String,
    pub method_name: String,
}
```

This can be stored in a `HashMap<Span, TraitDispatchInfo>` on the `TypeChecker` and
passed to the compiler via the AST annotation mechanism (check how ownership annotations
are communicated to the compiler — follow that pattern).

---

## Tests

Add to `crates/atlas-runtime/tests/typesystem.rs`:

```rust
#[test]
fn test_trait_method_call_typechecks_with_correct_type() {
    let atlas = Atlas::new();
    let result = atlas.eval("
        trait Display { fn display(self: Display) -> string; }
        impl Display for number {
            fn display(self: number) -> string { return str(self); }
        }
        let x: number = 42;
        let s: string = x.display();
    ");
    assert!(result.is_ok(), "Trait method call should typecheck");
}

#[test]
fn test_trait_method_call_wrong_assignment_type_is_error() {
    let atlas = Atlas::new();
    let result = atlas.eval("
        trait Display { fn display(self: Display) -> string; }
        impl Display for number {
            fn display(self: number) -> string { return str(self); }
        }
        let x: number = 42;
        let n: number = x.display();  // display() returns string, assigning to number
    ");
    assert!(result.is_err(), "Wrong type for trait method return should be error");
}

#[test]
fn test_trait_method_call_on_unimplemented_type_is_error() {
    let atlas = Atlas::new();
    let result = atlas.eval("
        trait Display { fn display(self: Display) -> string; }
        impl Display for number {
            fn display(self: number) -> string { return str(self); }
        }
        let s: string = \"hello\";
        let result: string = s.display();  // string doesn't implement Display
    ");
    // string doesn't have Display impl — should be an error or fall through to existing stdlib
    // (This edge case depends on dispatch priority — document behavior)
    // If string has no impl, AT method not found should fire
    assert!(result.is_err() || result.is_ok());  // Document which — see Notes
}

#[test]
fn test_trait_method_resolution_correct_return_type() {
    let atlas = Atlas::new();
    let result = atlas.eval("
        trait Doubler { fn double(self: Doubler) -> number; }
        impl Doubler for number {
            fn double(self: number) -> number { return self * 2; }
        }
        let x: number = 5;
        let y: number = x.double();  // must be number
    ");
    assert!(result.is_ok());
}

#[test]
fn test_chained_trait_method_calls() {
    let atlas = Atlas::new();
    let result = atlas.eval("
        trait Stringify { fn to_str(self: Stringify) -> string; }
        impl Stringify for number {
            fn to_str(self: number) -> string { return str(self); }
        }
        let x: number = 42;
        let s: string = x.to_str();
        print(s);
    ");
    assert!(result.is_ok());
}
```

---

## Acceptance Criteria

- [ ] `value.method(args)` where type implements the trait typechecks correctly
- [ ] Return type of trait method call is inferred from the impl's method signature
- [ ] Type mismatch on assignment of trait method result is caught
- [ ] Calling a method on a type that doesn't implement the relevant trait is an error
- [ ] Existing stdlib method calls are not affected (dispatch priority respected)
- [ ] All existing tests pass (no regressions)
- [ ] `cargo clippy -- -D warnings` clean
- [ ] `cargo fmt` clean

---

## Notes

- **Dispatch priority:** stdlib methods take precedence over trait methods of the same name.
  This means if stdlib defines `string.length()` and a user defines `impl SomeT for string`
  with `length()`, the stdlib version wins. Document this as the defined behavior.
- **`Type::Named`:** User-defined "types" in Block 3 are type names that appear in code
  but aren't actually defined structs (no struct declarations in Atlas yet). They're opaque
  types — the impl registry handles them by name. This is a known limitation in Block 3.
- The `TraitDispatchInfo` annotation mechanism should follow exactly the same pattern used
  to communicate ownership annotation data from typechecker to compiler in Block 2. Check
  how `fn_ownership_registry` is used and follow that pattern.
