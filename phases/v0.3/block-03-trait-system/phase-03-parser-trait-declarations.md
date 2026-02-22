# Phase 03 — Parser: Trait Declarations

**Block:** 3 (Trait System)
**Depends on:** Phase 02 complete
**Estimated tests added:** 12–18

---

## Objective

Parse `trait Foo { fn method(param: Type) -> ReturnType; }` declarations into `TraitDecl`
AST nodes. Trait bodies contain only method signatures — `fn` headers ending in `;` instead
of a `{` block body. This is a new parser pattern.

---

## Current State (verified 2026-02-22)

`crates/atlas-runtime/src/parser/mod.rs`:
- `parse_item()` dispatches on first token: `Import`, `Export`, `Extern`, `Fn`, `Type`, else `Statement`
- `parse_function()` at line 95 — requires `{` block body (no semicolon-terminator path)
- No `parse_trait()` or `parse_trait_method_sig()` functions exist
- `TokenKind::Trait` added in Phase 01, `Item::Trait` added in Phase 02

---

## Changes

### `crates/atlas-runtime/src/parser/mod.rs`

**1. Extend `parse_item()` to recognize `trait`:**

```rust
fn parse_item(&mut self, doc_comment: Option<String>) -> Result<Item, ()> {
    if self.check(TokenKind::Import) {
        Ok(Item::Import(self.parse_import()?))
    } else if self.check(TokenKind::Export) {
        Ok(Item::Export(self.parse_export()?))
    } else if self.check(TokenKind::Extern) {
        Ok(Item::Extern(self.parse_extern()?))
    } else if self.check(TokenKind::Fn) {
        Ok(Item::Function(self.parse_function()?))
    } else if self.check(TokenKind::Type) {
        Ok(Item::TypeAlias(self.parse_type_alias(doc_comment)?))
    } else if self.check(TokenKind::Trait) {  // NEW
        Ok(Item::Trait(self.parse_trait()?))
    } else {
        Ok(Item::Statement(self.parse_statement()?))
    }
}
```

**2. Add `parse_trait()`:**

```rust
fn parse_trait(&mut self) -> Result<TraitDecl, ()> {
    let start_span = self.consume(TokenKind::Trait, "Expected 'trait'")?.span;

    let name_tok = self.consume_identifier("a trait name")?;
    let name = Identifier {
        name: name_tok.lexeme.clone(),
        span: name_tok.span,
    };

    // Optional type parameters: `trait Functor<T>`
    let type_params = self.parse_type_params()?;

    self.consume(TokenKind::LeftBrace, "Expected '{' after trait name")?;

    let mut methods = Vec::new();
    while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
        methods.push(self.parse_trait_method_sig()?);
    }

    let end_span = self.consume(TokenKind::RightBrace, "Expected '}' after trait body")?.span;

    Ok(TraitDecl {
        name,
        type_params,
        methods,
        span: start_span.merge(end_span),
    })
}
```

**3. Add `parse_trait_method_sig()`:**

```rust
/// Parse a method signature inside a trait body.
/// Syntax: `fn method_name<T>(param: Type, ...) -> ReturnType;`
/// Note: NO block body — terminated by `;`
fn parse_trait_method_sig(&mut self) -> Result<TraitMethodSig, ()> {
    let start_span = self.consume(TokenKind::Fn, "Expected 'fn' in trait body")?.span;

    let name_tok = self.consume_identifier("a method name")?;
    let name = Identifier {
        name: name_tok.lexeme.clone(),
        span: name_tok.span,
    };

    let type_params = self.parse_type_params()?;

    self.consume(TokenKind::LeftParen, "Expected '(' after method name")?;
    let params = self.parse_params()?;
    self.consume(TokenKind::RightParen, "Expected ')' after method parameters")?;

    self.consume(TokenKind::Arrow, "Expected '->' after method parameters")?;
    let return_type = self.parse_type_ref()?;

    let end_span = self.consume(TokenKind::Semicolon, "Expected ';' after trait method signature")?.span;

    Ok(TraitMethodSig {
        name,
        type_params,
        params,
        return_type,
        span: start_span.merge(end_span),
    })
}
```

**4. `parse_type_params()` — extract shared helper:**

Both `parse_function()` and `parse_trait()` need to parse `<T: Bound, U>` type parameters.
Extract the existing type param parsing from `parse_function()` into a shared
`parse_type_params() -> Result<Vec<TypeParam>, ()>` helper. This also applies to
`parse_type_alias()` which has the same inline logic.

Note: Phase 05 will extend `parse_type_params()` to handle `:` trait bounds. For now,
it parses the existing `extends` syntax only.

---

## Atlas Syntax Supported After This Phase

```atlas
// Empty trait
trait Marker { }

// Single method
trait Display {
    fn display(self: Display) -> string;
}

// Multiple methods
trait Comparable {
    fn compare(self: Comparable, other: Comparable) -> number;
    fn equals(self: Comparable, other: Comparable) -> bool;
}

// Generic trait
trait Container<T> {
    fn get(self: Container<T>, index: number) -> T;
    fn size(self: Container<T>) -> number;
}
```

---

## Tests

Add to `crates/atlas-runtime/tests/frontend_syntax.rs`:

```rust
#[test]
fn test_parse_empty_trait() {
    let (prog, diags) = parse_source("trait Marker { }");
    assert!(diags.is_empty());
    assert_eq!(prog.items.len(), 1);
    assert!(matches!(prog.items[0], Item::Trait(_)));
    if let Item::Trait(t) = &prog.items[0] {
        assert_eq!(t.name.name, "Marker");
        assert!(t.methods.is_empty());
    }
}

#[test]
fn test_parse_trait_single_method() {
    let src = "trait Display { fn display(self: Display) -> string; }";
    let (prog, diags) = parse_source(src);
    assert!(diags.is_empty());
    if let Item::Trait(t) = &prog.items[0] {
        assert_eq!(t.name.name, "Display");
        assert_eq!(t.methods.len(), 1);
        assert_eq!(t.methods[0].name.name, "display");
        assert_eq!(t.methods[0].params.len(), 1);
    }
}

#[test]
fn test_parse_trait_multiple_methods() {
    let src = "trait Comparable {
        fn compare(self: Comparable, other: Comparable) -> number;
        fn equals(self: Comparable, other: Comparable) -> bool;
    }";
    let (prog, diags) = parse_source(src);
    assert!(diags.is_empty());
    if let Item::Trait(t) = &prog.items[0] {
        assert_eq!(t.methods.len(), 2);
        assert_eq!(t.methods[0].name.name, "compare");
        assert_eq!(t.methods[1].name.name, "equals");
    }
}

#[test]
fn test_parse_generic_trait() {
    let src = "trait Container<T> { fn get(self: Container<T>, index: number) -> T; }";
    let (prog, diags) = parse_source(src);
    assert!(diags.is_empty());
    if let Item::Trait(t) = &prog.items[0] {
        assert_eq!(t.name.name, "Container");
        assert_eq!(t.type_params.len(), 1);
        assert_eq!(t.type_params[0].name, "T");
    }
}

#[test]
fn test_trait_method_requires_semicolon() {
    // Missing semicolon after method sig — parse error
    let src = "trait Foo { fn bar() -> number }";
    let (_, diags) = parse_source(src);
    assert!(!diags.is_empty(), "Missing semicolon should produce a diagnostic");
}

#[test]
fn test_trait_method_cannot_have_body() {
    // Trait method sigs have no body — `{` would be interpreted as next trait item start
    // This is a parse error for the block-as-method-body case
    let src = "trait Foo { fn bar() -> number { return 1; } }";
    let (_, diags) = parse_source(src);
    assert!(!diags.is_empty(), "Method body in trait declaration should fail");
}

#[test]
fn test_trait_coexists_with_functions() {
    let src = "trait Display { fn display(self: Display) -> string; }
               fn greet() -> string { return \"hello\"; }";
    let (prog, diags) = parse_source(src);
    assert!(diags.is_empty());
    assert_eq!(prog.items.len(), 2);
    assert!(matches!(prog.items[0], Item::Trait(_)));
    assert!(matches!(prog.items[1], Item::Function(_)));
}
```

---

## Acceptance Criteria

- [ ] `trait Name { }` parses to `Item::Trait(TraitDecl { methods: [] })`
- [ ] `trait Name { fn method(params) -> T; }` parses correctly
- [ ] Multiple methods in trait body all parse
- [ ] Generic traits `trait Foo<T>` parse with type params
- [ ] Missing `;` after method signature produces a diagnostic
- [ ] Method body `{ ... }` in trait is a parse error (no block allowed)
- [ ] Trait declarations coexist with other items in a file
- [ ] All existing parser tests pass (no regressions)
- [ ] `cargo clippy -- -D warnings` clean
- [ ] `cargo fmt` clean

---

## Notes

- Extracting `parse_type_params()` as a shared helper removes ~25 lines of duplication.
  Make sure all 3 call sites (function, trait, type alias) use the helper after extraction.
- Trait method params use the existing `parse_params()` helper — ownership annotations
  (`own`, `borrow`, `shared`) work automatically in trait method signatures.
- `void` return type: trait methods that return nothing should use `-> void` explicitly
  (same rule as regular functions). Atlas has no implicit void.
