# Phase 02 — AST Nodes: TraitDecl, ImplBlock, TraitBound

**Block:** 3 (Trait System)
**Depends on:** Phase 01 complete
**Estimated tests added:** 10–15

---

## Objective

Add `TraitDecl`, `ImplBlock`, and `TraitBound` AST nodes. Extend `Item` enum with
`Trait` and `Impl` variants. Add `trait_bounds: Vec<TraitBound>` to `TypeParam`.

These are data-only changes — the parser (Phase 03–05) will fill them in.

---

## Current State (verified 2026-02-22)

`crates/atlas-runtime/src/ast.rs`:
- `Item` enum has: `Function`, `Statement`, `Import`, `Export`, `Extern`, `TypeAlias`
- `TypeParam` struct has: `name: String`, `bound: Option<TypeRef>`, `span: Span`
- No `TraitDecl`, `ImplBlock`, or `TraitBound` exist anywhere in the file
- Insta snapshot tests for `ast_dump` exist — changes to `FunctionDecl` or `TypeParam`
  will require snapshot updates (lesson from Block 2 Phase 14)

---

## New Types

### `TraitBound` — a single trait constraint on a type parameter

```rust
/// A trait bound on a type parameter: `T: TraitName`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraitBound {
    /// The trait name (e.g., "Copy", "Display", "MyTrait")
    pub trait_name: String,
    pub span: Span,
}
```

### `TraitMethodSig` — a method signature inside a `trait` declaration (no body)

```rust
/// A method signature in a trait declaration.
/// Has no body — the body is in the `ImplBlock`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraitMethodSig {
    pub name: Identifier,
    pub type_params: Vec<TypeParam>,
    pub params: Vec<Param>,
    pub return_type: TypeRef,
    pub span: Span,
}
```

### `TraitDecl` — a `trait Foo { fn method(...) -> T; }` declaration

```rust
/// A trait declaration.
///
/// Syntax: `trait Foo { fn method(self: Foo, arg: T) -> R; }`
///
/// Trait bodies contain only method signatures (no implementations).
/// Implementations live in `ImplBlock`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraitDecl {
    pub name: Identifier,
    /// Type parameters for generic traits (e.g., `trait Functor<T>`)
    pub type_params: Vec<TypeParam>,
    pub methods: Vec<TraitMethodSig>,
    pub span: Span,
}
```

### `ImplMethod` — a method implementation inside an `impl` block

```rust
/// A method implementation inside an `impl` block.
/// Identical to `FunctionDecl` but scoped to an impl.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImplMethod {
    pub name: Identifier,
    pub type_params: Vec<TypeParam>,
    pub params: Vec<Param>,
    pub return_type: TypeRef,
    pub body: Block,
    pub span: Span,
}
```

### `ImplBlock` — an `impl Foo for TypeName { ... }` block

```rust
/// An impl block.
///
/// Syntax: `impl TraitName for TypeName { fn method(...) { ... } }`
///
/// `trait_name` is the trait being implemented (e.g., "Display").
/// `type_name` is the type implementing the trait (e.g., "Buffer").
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImplBlock {
    pub trait_name: Identifier,
    /// Type arguments applied to the trait (e.g., `impl Functor<number> for MyType`)
    pub trait_type_args: Vec<TypeRef>,
    pub type_name: Identifier,
    pub methods: Vec<ImplMethod>,
    pub span: Span,
}
```

---

## Modifications to Existing Types

### `Item` enum — add two variants

```rust
pub enum Item {
    Function(FunctionDecl),
    Statement(Stmt),
    Import(ImportDecl),
    Export(ExportDecl),
    Extern(ExternDecl),
    TypeAlias(TypeAliasDecl),
    // NEW:
    Trait(TraitDecl),
    Impl(ImplBlock),
}
```

### `TypeParam` struct — add `trait_bounds` field

```rust
pub struct TypeParam {
    pub name: String,
    /// Optional type-level bound (e.g., `T extends number`) — existing field
    pub bound: Option<TypeRef>,
    // NEW:
    /// Trait bounds on this type parameter (e.g., `T: Copy + Display`)
    pub trait_bounds: Vec<TraitBound>,
    pub span: Span,
}
```

**Default for existing construction sites:** `trait_bounds: vec![]` — add this to all
existing `TypeParam { ... }` construction sites in the codebase to avoid breaking builds.

---

## Snapshot Impact

The `TypeParam` struct change will break insta snapshots if any snapshot contains
`TypeParam` output. Grep for `TypeParam` in `tests/snapshots/` and update proactively.

```bash
grep -rl "TypeParam\|type_params" crates/atlas-runtime/tests/snapshots/
```

Run `cargo test` with `UPDATE_EXPECT=1` or `INSTA_UPDATE=always` to regenerate snapshots
after the struct change.

---

## Span Helper

Add `span()` helpers to new types following the existing pattern:

```rust
impl TraitDecl {
    pub fn span(&self) -> Span { self.span }
}
impl ImplBlock {
    pub fn span(&self) -> Span { self.span }
}
```

---

## Tests

Add to `ast.rs` inline `#[cfg(test)]` module:

```rust
#[test]
fn test_trait_decl_construction() {
    let decl = TraitDecl {
        name: Identifier { name: "Display".to_string(), span: Span::new(6, 13) },
        type_params: vec![],
        methods: vec![TraitMethodSig {
            name: Identifier { name: "display".to_string(), span: Span::new(20, 27) },
            type_params: vec![],
            params: vec![],
            return_type: TypeRef::Named("string".to_string(), Span::new(32, 38)),
            span: Span::new(17, 39),
        }],
        span: Span::new(0, 40),
    };
    assert_eq!(decl.name.name, "Display");
    assert_eq!(decl.methods.len(), 1);
    assert_eq!(decl.methods[0].name.name, "display");
}

#[test]
fn test_impl_block_construction() {
    let impl_block = ImplBlock {
        trait_name: Identifier { name: "Display".to_string(), span: Span::new(5, 12) },
        trait_type_args: vec![],
        type_name: Identifier { name: "Buffer".to_string(), span: Span::new(17, 23) },
        methods: vec![],
        span: Span::new(0, 30),
    };
    assert_eq!(impl_block.trait_name.name, "Display");
    assert_eq!(impl_block.type_name.name, "Buffer");
}

#[test]
fn test_trait_bound_construction() {
    let bound = TraitBound {
        trait_name: "Copy".to_string(),
        span: Span::new(3, 7),
    };
    assert_eq!(bound.trait_name, "Copy");
}

#[test]
fn test_type_param_with_trait_bounds() {
    let param = TypeParam {
        name: "T".to_string(),
        bound: None,
        trait_bounds: vec![
            TraitBound { trait_name: "Copy".to_string(), span: Span::new(3, 7) },
        ],
        span: Span::new(0, 7),
    };
    assert_eq!(param.trait_bounds.len(), 1);
    assert_eq!(param.trait_bounds[0].trait_name, "Copy");
}

#[test]
fn test_item_enum_trait_variant() {
    let decl = TraitDecl {
        name: Identifier { name: "Foo".to_string(), span: Span::new(6, 9) },
        type_params: vec![],
        methods: vec![],
        span: Span::new(0, 12),
    };
    let item = Item::Trait(decl);
    assert!(matches!(item, Item::Trait(_)));
}

#[test]
fn test_item_enum_impl_variant() {
    let impl_block = ImplBlock {
        trait_name: Identifier { name: "Foo".to_string(), span: Span::new(5, 8) },
        trait_type_args: vec![],
        type_name: Identifier { name: "Bar".to_string(), span: Span::new(13, 16) },
        methods: vec![],
        span: Span::new(0, 19),
    };
    let item = Item::Impl(impl_block);
    assert!(matches!(item, Item::Impl(_)));
}
```

---

## Acceptance Criteria

- [ ] `TraitBound`, `TraitMethodSig`, `TraitDecl`, `ImplMethod`, `ImplBlock` structs compile
- [ ] `Item::Trait` and `Item::Impl` variants exist
- [ ] `TypeParam.trait_bounds: Vec<TraitBound>` field exists
- [ ] All existing `TypeParam` construction sites updated with `trait_bounds: vec![]`
- [ ] Insta snapshots updated if needed
- [ ] All unit tests pass
- [ ] `cargo clippy -- -D warnings` clean
- [ ] `cargo fmt` clean

---

## Notes

- `ImplMethod` is intentionally separate from `FunctionDecl` even though they're structurally
  similar. Impl methods have a different semantic context (they belong to an impl block, not
  top-level scope). Deduplicating them would create tight coupling between AST nodes.
- `trait_bounds` is `Vec<TraitBound>` not `Option<TraitBound>` — allows `T: Copy + Display`
  multi-bound syntax in future phases without an AST change.
- `bound: Option<TypeRef>` (existing `extends`) remains untouched. Two separate mechanisms:
  `extends` = type-level inheritance (existing), `:` = trait bounds (new).
