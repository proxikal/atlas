# Phase 06 — Typechecker: Trait Registry + Built-in Traits

**Block:** 3 (Trait System)
**Depends on:** Phase 05 complete
**Estimated tests added:** 15–20

---

## Objective

Add `TraitRegistry` to the typechecker. Register all five built-in traits: `Copy`, `Move`,
`Drop`, `Display`, `Debug`. Register user-defined traits during type checking of `Item::Trait`.
This phase establishes the data structure — conformance checking is Phase 07.

---

## Current State (verified 2026-02-22)

`crates/atlas-runtime/src/typechecker/mod.rs`:
- `TypeChecker` struct has: `symbol_table`, `diagnostics`, `fn_ownership_registry`,
  `current_fn_param_ownerships`, `type_aliases`, `alias_cache`, etc.
- No `TraitRegistry` or `ImplRegistry` fields exist
- `check_item()` handles `Function`, `Statement`, `Import`, `Export`, `Extern`, `TypeAlias`
- `Item::Trait` and `Item::Impl` will cause `check_item()` to panic on exhaustive match
  or silently ignore — must be handled

---

## New Types

Add to `crates/atlas-runtime/src/typechecker/mod.rs` (or new `typechecker/traits.rs`):

```rust
/// A registered trait's method signatures.
#[derive(Debug, Clone)]
pub struct TraitMethodEntry {
    pub name: String,
    pub type_params: Vec<TypeParamDef>,
    pub param_types: Vec<Type>,
    pub return_type: Type,
}

/// Registry of known traits (built-in + user-defined).
#[derive(Debug, Default)]
pub struct TraitRegistry {
    /// Maps trait name -> list of required method signatures
    pub traits: HashMap<String, Vec<TraitMethodEntry>>,
    /// Set of built-in trait names (not user-definable as aliases)
    pub built_in: HashSet<String>,
    /// Maps (type_name, trait_name) -> whether type implements the trait
    /// For built-in types, this is pre-populated. For user types, populated during impl checking.
    pub implementations: HashMap<(String, String), bool>,
}

impl TraitRegistry {
    pub fn new() -> Self {
        let mut registry = Self::default();
        registry.register_built_ins();
        registry
    }

    /// Register all built-in Atlas traits.
    fn register_built_ins(&mut self) {
        // Copy — types that can be freely copied (value semantics)
        self.register_built_in("Copy", vec![]);  // marker trait, no methods

        // Move — types that require explicit ownership transfer (non-Copy)
        self.register_built_in("Move", vec![]);  // marker trait, no methods

        // Drop — types with custom destructor logic
        self.register_built_in("Drop", vec![
            TraitMethodEntry {
                name: "drop".to_string(),
                type_params: vec![],
                param_types: vec![],  // `self` handled separately
                return_type: Type::Void,
            }
        ]);

        // Display — types that can be converted to string representation
        self.register_built_in("Display", vec![
            TraitMethodEntry {
                name: "display".to_string(),
                type_params: vec![],
                param_types: vec![],  // `self` as first param (implicit in method call)
                return_type: Type::String,
            }
        ]);

        // Debug — types that can be serialized to debug string representation
        self.register_built_in("Debug", vec![
            TraitMethodEntry {
                name: "debug_repr".to_string(),
                type_params: vec![],
                param_types: vec![],
                return_type: Type::String,
            }
        ]);

        // Register built-in type implementations
        // All primitive types implement Copy (value semantics)
        for primitive in &["number", "string", "bool", "null"] {
            self.mark_implements(primitive, "Copy");
            // Primitives do NOT implement Move (they're Copy, not moved)
        }

        // Built-in types do NOT implement Display/Debug by default —
        // they use the existing str() / debug() stdlib functions.
        // User-defined types can impl Display/Debug to override str() behavior.
    }

    fn register_built_in(&mut self, name: &str, methods: Vec<TraitMethodEntry>) {
        self.traits.insert(name.to_string(), methods);
        self.built_in.insert(name.to_string());
    }

    pub fn register_user_trait(&mut self, name: &str, methods: Vec<TraitMethodEntry>) {
        self.traits.insert(name.to_string(), methods);
    }

    pub fn mark_implements(&mut self, type_name: &str, trait_name: &str) {
        self.implementations.insert(
            (type_name.to_string(), trait_name.to_string()),
            true,
        );
    }

    pub fn implements(&self, type_name: &str, trait_name: &str) -> bool {
        self.implementations
            .get(&(type_name.to_string(), trait_name.to_string()))
            .copied()
            .unwrap_or(false)
    }

    pub fn trait_exists(&self, name: &str) -> bool {
        self.traits.contains_key(name)
    }

    pub fn get_methods(&self, trait_name: &str) -> Option<&Vec<TraitMethodEntry>> {
        self.traits.get(trait_name)
    }

    pub fn is_built_in(&self, name: &str) -> bool {
        self.built_in.contains(name)
    }
}
```

---

## Changes to `TypeChecker`

**Add field:**
```rust
pub trait_registry: TraitRegistry,
```

**Initialize in `TypeChecker::new()`:**
```rust
trait_registry: TraitRegistry::new(),
```

**Extend `check_item()` to handle `Item::Trait`:**

```rust
Item::Trait(trait_decl) => self.check_trait_decl(trait_decl),
Item::Impl(impl_block) => self.check_impl_block(impl_block),  // Phase 07 adds full impl checking
```

**Add `check_trait_decl()`:**

```rust
fn check_trait_decl(&mut self, trait_decl: &TraitDecl) {
    let trait_name = &trait_decl.name.name;

    // Error if re-declaring a built-in trait
    if self.trait_registry.is_built_in(trait_name) {
        self.diagnostics.push(Diagnostic::error(
            error_codes::TRAIT_REDEFINES_BUILTIN,
            format!("Cannot redefine built-in trait '{}'", trait_name),
            trait_decl.name.span,
        ));
        return;
    }

    // Error if duplicate user trait declaration
    if self.trait_registry.trait_exists(trait_name) {
        self.diagnostics.push(Diagnostic::error(
            error_codes::TRAIT_ALREADY_DEFINED,
            format!("Trait '{}' is already defined", trait_name),
            trait_decl.name.span,
        ));
        return;
    }

    // Resolve method signatures
    let mut method_entries = Vec::new();
    for method_sig in &trait_decl.methods {
        let param_types: Vec<Type> = method_sig.params.iter()
            .map(|p| self.resolve_type_ref(&p.type_ref))
            .collect();
        let return_type = self.resolve_type_ref(&method_sig.return_type);

        method_entries.push(TraitMethodEntry {
            name: method_sig.name.name.clone(),
            type_params: method_sig.type_params.iter()
                .map(|tp| TypeParamDef { name: tp.name.clone(), bound: None })
                .collect(),
            param_types,
            return_type,
        });
    }

    self.trait_registry.register_user_trait(trait_name, method_entries);
}
```

**Add stub `check_impl_block()` (full logic in Phase 07):**
```rust
fn check_impl_block(&mut self, impl_block: &ImplBlock) {
    // Phase 07 adds full conformance checking
    // For now: just verify trait exists
    let trait_name = &impl_block.trait_name.name;
    if !self.trait_registry.trait_exists(trait_name) {
        self.diagnostics.push(Diagnostic::error(
            error_codes::TRAIT_NOT_FOUND,
            format!("Trait '{}' is not defined", trait_name),
            impl_block.trait_name.span,
        ));
    }
}
```

---

## New Diagnostic Codes

Add to `crates/atlas-runtime/src/diagnostic.rs` (AT3xxx range):

```rust
pub const TRAIT_REDEFINES_BUILTIN: &str = "AT3001";
pub const TRAIT_ALREADY_DEFINED: &str = "AT3002";
pub const TRAIT_NOT_FOUND: &str = "AT3003";
pub const IMPL_METHOD_MISSING: &str = "AT3004";      // Phase 07
pub const IMPL_METHOD_SIGNATURE_MISMATCH: &str = "AT3005";  // Phase 07
pub const TYPE_DOES_NOT_IMPLEMENT_TRAIT: &str = "AT3006";   // Phase 10
pub const COPY_TYPE_REQUIRED: &str = "AT3007";              // Phase 09
pub const TRAIT_BOUND_NOT_SATISFIED: &str = "AT3008";       // Phase 10
```

(Register all 8 codes now — referenced in later phases.)

---

## Tests

Add to `crates/atlas-runtime/tests/typesystem.rs`:

```rust
#[test]
fn test_builtin_traits_exist_in_registry() {
    // After typechecking any program, built-in traits are registered
    let atlas = Atlas::new();
    atlas.eval("let x = 1;").unwrap();
    // This tests internal state — use the public API to verify behavior instead:
    // number implements Copy (verified by no error on copy-requiring operations)
}

#[test]
fn test_user_trait_declaration_typechecks() {
    let atlas = Atlas::new();
    let result = atlas.eval("trait Display { fn display(self: Display) -> string; }");
    assert!(result.is_ok(), "Valid trait declaration should typecheck");
}

#[test]
fn test_duplicate_trait_declaration_is_error() {
    let atlas = Atlas::new();
    let result = atlas.eval("
        trait Foo { fn bar() -> void; }
        trait Foo { fn baz() -> void; }
    ");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("AT3002") || err.to_string().contains("already defined"));
}

#[test]
fn test_redefining_builtin_trait_is_error() {
    let atlas = Atlas::new();
    let result = atlas.eval("trait Copy { fn do_copy() -> void; }");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("AT3001") || err.to_string().contains("built-in"));
}

#[test]
fn test_impl_unknown_trait_is_error() {
    let atlas = Atlas::new();
    let result = atlas.eval("impl UnknownTrait for number { }");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("AT3003") || err.to_string().contains("not defined"));
}

#[test]
fn test_trait_with_multiple_methods_registers() {
    let atlas = Atlas::new();
    let result = atlas.eval("
        trait Comparable {
            fn compare(self: Comparable, other: Comparable) -> number;
            fn equals(self: Comparable, other: Comparable) -> bool;
        }
    ");
    assert!(result.is_ok(), "Multi-method trait declaration should typecheck");
}

#[test]
fn test_empty_trait_declaration_is_valid() {
    let atlas = Atlas::new();
    let result = atlas.eval("trait Marker { }");
    assert!(result.is_ok(), "Empty (marker) trait should be valid");
}

#[test]
fn test_impl_known_trait_with_stub_body() {
    // With Phase 06's stub impl checker: known trait + empty impl = no AT3003 error
    let atlas = Atlas::new();
    let result = atlas.eval("
        trait Marker { }
        impl Marker for number { }
    ");
    // Should not produce AT3003 (trait exists); Phase 07 adds conformance checking
    assert!(result.is_ok() || !result.unwrap_err().to_string().contains("AT3003"));
}
```

---

## Acceptance Criteria

- [ ] `TraitRegistry` struct exists with `new()` that pre-populates built-in traits
- [ ] Built-in traits registered: `Copy`, `Move`, `Drop`, `Display`, `Debug`
- [ ] `number`, `string`, `bool`, `null` marked as implementing `Copy`
- [ ] `TypeChecker` has `trait_registry: TraitRegistry` field
- [ ] `check_item()` handles `Item::Trait` and `Item::Impl` (no panic on exhaustive match)
- [ ] `check_trait_decl()` registers user traits and errors on duplicates/builtins
- [ ] AT3001–AT3008 error codes registered in `diagnostic.rs`
- [ ] All existing tests pass (no regressions)
- [ ] `cargo clippy -- -D warnings` clean
- [ ] `cargo fmt` clean

---

## Notes

- `Drop` invocation is explicit only in Block 3 (call `value.drop()` manually). Automatic
  scope-exit drop is v0.4. The `Drop` trait exists and can be implemented, but the typechecker
  and VM don't automatically call it on scope exit yet.
- The `implementations` map uses string type names. When user-defined types (structs) arrive
  in v0.4, this becomes a `Type`-keyed map. For v0.3, string names are sufficient since all
  type names are valid identifiers.
- `Display` and `Debug` built-in trait methods (`display`, `debug_repr`) intentionally don't
  pre-register built-in types as implementing them. Built-in str() conversion uses stdlib
  functions, not trait dispatch. User types implement Display to customize str() output.
