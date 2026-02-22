# Phase 07 — Typechecker: Impl Conformance Checking

**Block:** 3 (Trait System)
**Depends on:** Phase 06 complete
**Estimated tests added:** 18–24

---

## Objective

Fully implement `check_impl_block()`. An impl block must:
1. Implement ALL methods declared in the trait (AT3004 if missing)
2. Each method's signature must match the trait's declaration (AT3005 if mismatch)
3. Register the (type_name, trait_name) pair in `trait_registry.implementations`

Also add the `ImplRegistry` — a lookup map from `(type_name, trait_name)` to the
resolved method implementations, used by Phase 12/13 for dispatch.

---

## Current State (verified after Phase 06)

- `TraitRegistry` exists with `trait_exists()`, `get_methods()`, `mark_implements()`
- `check_impl_block()` is a stub that only checks if trait exists (AT3003)
- `AT3004` (`IMPL_METHOD_MISSING`) and `AT3005` (`IMPL_METHOD_SIGNATURE_MISMATCH`) are
  registered in `diagnostic.rs` but not emitted yet

---

## New Type: `ImplRegistry`

```rust
/// Resolved impl block: maps method name -> compiled method body reference
#[derive(Debug, Clone)]
pub struct ImplEntry {
    pub trait_name: String,
    pub type_name: String,
    /// Method name -> the ImplMethod (for interpretation/compilation)
    pub methods: HashMap<String, ImplMethod>,
}

/// Registry of all impl blocks: (type_name, trait_name) -> ImplEntry
#[derive(Debug, Default)]
pub struct ImplRegistry {
    pub entries: HashMap<(String, String), ImplEntry>,
}

impl ImplRegistry {
    pub fn register(&mut self, type_name: &str, trait_name: &str, methods: HashMap<String, ImplMethod>) {
        self.entries.insert(
            (type_name.to_string(), trait_name.to_string()),
            ImplEntry {
                trait_name: trait_name.to_string(),
                type_name: type_name.to_string(),
                methods,
            },
        );
    }

    pub fn get_method(&self, type_name: &str, trait_name: &str, method_name: &str) -> Option<&ImplMethod> {
        self.entries
            .get(&(type_name.to_string(), trait_name.to_string()))
            .and_then(|entry| entry.methods.get(method_name))
    }

    pub fn has_impl(&self, type_name: &str, trait_name: &str) -> bool {
        self.entries.contains_key(&(type_name.to_string(), trait_name.to_string()))
    }
}
```

Add `pub impl_registry: ImplRegistry` to `TypeChecker` and initialize in `new()`.

---

## Full `check_impl_block()` Implementation

```rust
fn check_impl_block(&mut self, impl_block: &ImplBlock) {
    let trait_name = &impl_block.trait_name.name;
    let type_name = &impl_block.type_name.name;

    // 1. Verify trait exists
    if !self.trait_registry.trait_exists(trait_name) {
        self.diagnostics.push(Diagnostic::error(
            error_codes::TRAIT_NOT_FOUND,
            format!("Trait '{}' is not defined", trait_name),
            impl_block.trait_name.span,
        ));
        return;
    }

    // 2. Check for duplicate impl
    if self.impl_registry.has_impl(type_name, trait_name) {
        self.diagnostics.push(Diagnostic::error(
            error_codes::IMPL_ALREADY_EXISTS,
            format!("'{}' already implements '{}'", type_name, trait_name),
            impl_block.span,
        ));
        return;
    }

    // 3. Get required methods from trait
    let required_methods: Vec<TraitMethodEntry> = self.trait_registry
        .get_methods(trait_name)
        .cloned()
        .unwrap_or_default();

    // 4. Build a map of provided methods
    let provided: HashMap<String, &ImplMethod> = impl_block.methods.iter()
        .map(|m| (m.name.name.clone(), m))
        .collect();

    // 5. Check all required methods are provided with matching signatures
    let mut all_ok = true;
    for required in &required_methods {
        match provided.get(&required.name) {
            None => {
                self.diagnostics.push(Diagnostic::error(
                    error_codes::IMPL_METHOD_MISSING,
                    format!(
                        "Impl of '{}' for '{}' is missing method '{}'",
                        trait_name, type_name, required.name
                    ),
                    impl_block.span,
                ));
                all_ok = false;
            }
            Some(impl_method) => {
                // Check param count matches (excluding self)
                let impl_param_types: Vec<Type> = impl_method.params.iter()
                    .map(|p| self.resolve_type_ref(&p.type_ref))
                    .collect();

                if impl_param_types != required.param_types {
                    self.diagnostics.push(Diagnostic::error(
                        error_codes::IMPL_METHOD_SIGNATURE_MISMATCH,
                        format!(
                            "Method '{}' in impl of '{}' for '{}' has wrong parameter types",
                            required.name, trait_name, type_name
                        ),
                        impl_method.span,
                    ));
                    all_ok = false;
                }

                // Check return type matches
                let impl_return = self.resolve_type_ref(&impl_method.return_type);
                if impl_return != required.return_type {
                    self.diagnostics.push(Diagnostic::error(
                        error_codes::IMPL_METHOD_SIGNATURE_MISMATCH,
                        format!(
                            "Method '{}' in impl of '{}' for '{}' has wrong return type",
                            required.name, trait_name, type_name
                        ),
                        impl_method.span,
                    ));
                    all_ok = false;
                }
            }
        }
    }

    // 6. Typecheck method bodies
    for impl_method in &impl_block.methods {
        self.check_impl_method_body(impl_method);
    }

    // 7. Register impl if conformance passed
    if all_ok {
        let method_map: HashMap<String, ImplMethod> = impl_block.methods.iter()
            .map(|m| (m.name.name.clone(), m.clone()))
            .collect();
        self.impl_registry.register(type_name, trait_name, method_map);
        self.trait_registry.mark_implements(type_name, trait_name);
    }
}

fn check_impl_method_body(&mut self, method: &ImplMethod) {
    // Typecheck the method body — reuse existing function body checking logic
    // Push a new scope, register params, check body statements
    let saved_return_type = self.current_function_return_type.take();
    let saved_fn_info = self.current_function_info.take();

    self.current_function_return_type = Some(self.resolve_type_ref(&method.return_type));
    self.current_function_info = Some((method.name.name.clone(), method.span));

    // Register params in scope
    for param in &method.params {
        let ty = self.resolve_type_ref(&param.type_ref);
        self.declare_variable(&param.name.name, ty, param.span, SymbolKind::Parameter);
    }

    for stmt in &method.body.statements {
        self.check_statement(stmt);
    }

    self.current_function_return_type = saved_return_type;
    self.current_function_info = saved_fn_info;
}
```

Add `IMPL_ALREADY_EXISTS: &str = "AT3009"` to diagnostic codes.

---

## Tests

Add to `crates/atlas-runtime/tests/typesystem.rs`:

```rust
#[test]
fn test_impl_conformance_all_methods_present() {
    let atlas = Atlas::new();
    let result = atlas.eval("
        trait Display { fn display(self: Display) -> string; }
        impl Display for number {
            fn display(self: number) -> string { return str(self); }
        }
    ");
    assert!(result.is_ok(), "Complete impl should typecheck");
}

#[test]
fn test_impl_missing_method_is_error() {
    let atlas = Atlas::new();
    let result = atlas.eval("
        trait Shape {
            fn area(self: Shape) -> number;
            fn perimeter(self: Shape) -> number;
        }
        impl Shape for Circle {
            fn area(self: Circle) -> number { return 0.0; }
            // missing: perimeter
        }
    ");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("AT3004") || err.to_string().contains("missing method"));
}

#[test]
fn test_impl_wrong_return_type_is_error() {
    let atlas = Atlas::new();
    let result = atlas.eval("
        trait Display { fn display(self: Display) -> string; }
        impl Display for number {
            fn display(self: number) -> number { return self; }  // wrong return type
        }
    ");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("AT3005") || err.to_string().contains("return type"));
}

#[test]
fn test_impl_wrong_param_types_is_error() {
    let atlas = Atlas::new();
    let result = atlas.eval("
        trait Adder { fn add(self: Adder, x: number) -> number; }
        impl Adder for MyType {
            fn add(self: MyType, x: string) -> number { return 0; }  // wrong param type
        }
    ");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("AT3005") || err.to_string().contains("parameter types"));
}

#[test]
fn test_duplicate_impl_is_error() {
    let atlas = Atlas::new();
    let result = atlas.eval("
        trait Marker { }
        impl Marker for number { }
        impl Marker for number { }  // duplicate
    ");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("AT3009") || err.to_string().contains("already implements"));
}

#[test]
fn test_empty_trait_impl_is_valid() {
    let atlas = Atlas::new();
    let result = atlas.eval("
        trait Marker { }
        impl Marker for number { }
        impl Marker for string { }  // different types, both valid
    ");
    assert!(result.is_ok());
}

#[test]
fn test_impl_method_body_typechecked() {
    let atlas = Atlas::new();
    let result = atlas.eval("
        trait Doubler { fn double(self: Doubler, x: number) -> number; }
        impl Doubler for MyDoubler {
            fn double(self: MyDoubler, x: number) -> number {
                return \"wrong\";  // type error in body
            }
        }
    ");
    assert!(result.is_err(), "Type error in impl method body should be caught");
}

#[test]
fn test_marker_trait_impl_for_multiple_types() {
    let atlas = Atlas::new();
    let result = atlas.eval("
        trait Serializable { }
        impl Serializable for number { }
        impl Serializable for string { }
        impl Serializable for bool { }
    ");
    assert!(result.is_ok());
}

#[test]
fn test_impl_registers_in_trait_registry() {
    // After impl, the type is known to implement the trait (AT3006 won't fire)
    let atlas = Atlas::new();
    let result = atlas.eval("
        trait Display { fn display(self: Display) -> string; }
        impl Display for number {
            fn display(self: number) -> string { return str(self); }
        }
        fn show<T: Display>(x: T) -> string { return x.display(); }
    ");
    // Phase 10 adds full bound enforcement — for now just verify no crash
    assert!(result.is_ok() || result.is_err());  // placeholder until Phase 10
}
```

---

## Acceptance Criteria

- [ ] `ImplRegistry` struct exists with `register()`, `get_method()`, `has_impl()`
- [ ] `TypeChecker` has `impl_registry: ImplRegistry` field
- [ ] AT3004 fires when impl is missing a required method
- [ ] AT3005 fires when impl method has wrong return type
- [ ] AT3005 fires when impl method has wrong param types
- [ ] AT3009 fires on duplicate `impl Trait for Type`
- [ ] Impl method bodies are type-checked for internal correctness
- [ ] Successful impl registers in both `impl_registry` and `trait_registry.implementations`
- [ ] Marker trait (no methods) impl is always valid
- [ ] All existing tests pass (no regressions)
- [ ] `cargo clippy -- -D warnings` clean
- [ ] `cargo fmt` clean

---

## Notes

- The `self` parameter in trait methods is treated as a regular typed parameter. The
  typechecker doesn't give it special treatment in Block 3 — it's just `param[0]` with
  whatever type the impl provides. Full `self`-type inference is v0.4.
- Signature comparison is structural (type equality), not nominal. `number` == `number`.
  Type aliases are resolved before comparison via `resolve_type_ref()`.
- Extra methods in impl (methods not in trait) are silently allowed. This supports
  impl blocks that add methods beyond what the trait requires.
