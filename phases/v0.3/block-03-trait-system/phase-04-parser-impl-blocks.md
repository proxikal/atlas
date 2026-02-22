# Phase 04 — Parser: Impl Blocks

**Block:** 3 (Trait System)
**Depends on:** Phase 03 complete
**Estimated tests added:** 12–18

---

## Objective

Parse `impl TraitName for TypeName { fn method(...) { ... } }` blocks into `ImplBlock`
AST nodes. Impl method bodies ARE required (full function bodies, unlike trait signatures).

---

## Current State (verified 2026-02-22)

`crates/atlas-runtime/src/parser/mod.rs`:
- `parse_item()` extended in Phase 03 to handle `Trait`
- `TokenKind::Impl` added in Phase 01, `Item::Impl(ImplBlock)` added in Phase 02
- `for` is already `TokenKind::For`
- `parse_function()` handles function bodies — `parse_impl_method()` will reuse the body
  parsing portion of `parse_function()`

---

## Changes

### `crates/atlas-runtime/src/parser/mod.rs`

**1. Extend `parse_item()` to recognize `impl`:**

```rust
} else if self.check(TokenKind::Impl) {  // NEW
    Ok(Item::Impl(self.parse_impl_block()?))
} else {
    Ok(Item::Statement(self.parse_statement()?))
}
```

**2. Add `parse_impl_block()`:**

```rust
fn parse_impl_block(&mut self) -> Result<ImplBlock, ()> {
    let start_span = self.consume(TokenKind::Impl, "Expected 'impl'")?.span;

    // Parse trait name: `impl TraitName`
    let trait_name_tok = self.consume_identifier("a trait name")?;
    let trait_name = Identifier {
        name: trait_name_tok.lexeme.clone(),
        span: trait_name_tok.span,
    };

    // Optional type args on trait: `impl Functor<number> for MyType`
    let trait_type_args = if self.check(TokenKind::Less) {
        self.parse_type_arg_list()?
    } else {
        vec![]
    };

    // `for TypeName`
    self.consume(TokenKind::For, "Expected 'for' after trait name in impl block")?;

    let type_name_tok = self.consume_identifier("a type name")?;
    let type_name = Identifier {
        name: type_name_tok.lexeme.clone(),
        span: type_name_tok.span,
    };

    self.consume(TokenKind::LeftBrace, "Expected '{' after type name in impl block")?;

    let mut methods = Vec::new();
    while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
        methods.push(self.parse_impl_method()?);
    }

    let end_span = self.consume(TokenKind::RightBrace, "Expected '}' after impl body")?.span;

    Ok(ImplBlock {
        trait_name,
        trait_type_args,
        type_name,
        methods,
        span: start_span.merge(end_span),
    })
}
```

**3. Add `parse_impl_method()`:**

```rust
/// Parse a method implementation inside an impl block.
/// Syntax: `fn method_name<T>(param: Type) -> ReturnType { body }`
/// This is identical to a function declaration but stored in an ImplBlock.
fn parse_impl_method(&mut self) -> Result<ImplMethod, ()> {
    let start_span = self.consume(TokenKind::Fn, "Expected 'fn' in impl body")?.span;

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

    // Impl methods REQUIRE a body
    let body = self.parse_block()?;
    let end_span = body.span;

    Ok(ImplMethod {
        name,
        type_params,
        params,
        return_type,
        body,
        span: start_span.merge(end_span),
    })
}
```

**4. `parse_type_arg_list()` — helper for `<number, string>` in impl trait type args:**

Check if this helper already exists from generics parsing. If not, add a small helper that
parses a `<TypeRef, TypeRef, ...>` list and returns `Vec<TypeRef>`. This is shared between
impl blocks and generic type expressions.

---

## Atlas Syntax Supported After This Phase

```atlas
// Simple impl
trait Display {
    fn display(self: Display) -> string;
}

impl Display for number {
    fn display(self: number) -> string {
        return str(self);
    }
}

// Impl with multiple methods
trait Shape {
    fn area(self: Shape) -> number;
    fn perimeter(self: Shape) -> number;
}

impl Shape for Circle {
    fn area(self: Circle) -> number {
        return 3.14159 * self.radius * self.radius;
    }
    fn perimeter(self: Circle) -> number {
        return 2.0 * 3.14159 * self.radius;
    }
}

// Impl for generic trait
impl Container<number> for NumberList {
    fn get(self: NumberList, index: number) -> number {
        return self.items[index];
    }
    fn size(self: NumberList) -> number {
        return len(self.items);
    }
}
```

---

## Tests

Add to `crates/atlas-runtime/tests/frontend_syntax.rs`:

```rust
#[test]
fn test_parse_simple_impl_block() {
    let src = "
        trait Display { fn display(self: Display) -> string; }
        impl Display for number {
            fn display(self: number) -> string { return str(self); }
        }
    ";
    let (prog, diags) = parse_source(src);
    assert!(diags.is_empty());
    assert_eq!(prog.items.len(), 2);
    assert!(matches!(prog.items[1], Item::Impl(_)));
    if let Item::Impl(ib) = &prog.items[1] {
        assert_eq!(ib.trait_name.name, "Display");
        assert_eq!(ib.type_name.name, "number");
        assert_eq!(ib.methods.len(), 1);
        assert_eq!(ib.methods[0].name.name, "display");
    }
}

#[test]
fn test_parse_impl_with_multiple_methods() {
    let src = "
        trait Shape {
            fn area(self: Shape) -> number;
            fn perimeter(self: Shape) -> number;
        }
        impl Shape for Circle {
            fn area(self: Circle) -> number { return 0.0; }
            fn perimeter(self: Circle) -> number { return 0.0; }
        }
    ";
    let (prog, diags) = parse_source(src);
    assert!(diags.is_empty());
    if let Item::Impl(ib) = &prog.items[1] {
        assert_eq!(ib.methods.len(), 2);
    }
}

#[test]
fn test_parse_impl_generic_trait() {
    let src = "
        trait Container<T> { fn size(self: Container<T>) -> number; }
        impl Container<number> for NumberList {
            fn size(self: NumberList) -> number { return 0; }
        }
    ";
    let (prog, diags) = parse_source(src);
    assert!(diags.is_empty());
    if let Item::Impl(ib) = &prog.items[1] {
        assert_eq!(ib.trait_name.name, "Container");
        assert_eq!(ib.trait_type_args.len(), 1);
    }
}

#[test]
fn test_parse_impl_requires_for_keyword() {
    let src = "impl Display number { fn display(self: number) -> string { return \"\"; } }";
    let (_, diags) = parse_source(src);
    assert!(!diags.is_empty(), "Missing 'for' keyword should produce a diagnostic");
}

#[test]
fn test_parse_impl_method_requires_body() {
    // Impl methods must have a body (unlike trait signatures)
    let src = "trait T { fn m() -> void; } impl T for X { fn m() -> void; }";
    let (_, diags) = parse_source(src);
    assert!(!diags.is_empty(), "Missing method body in impl should produce a diagnostic");
}

#[test]
fn test_parse_impl_empty_body() {
    // Impl with zero methods is valid (e.g., marker trait impl)
    let src = "trait Marker { } impl Marker for number { }";
    let (prog, diags) = parse_source(src);
    assert!(diags.is_empty());
    if let Item::Impl(ib) = &prog.items[1] {
        assert!(ib.methods.is_empty());
    }
}

#[test]
fn test_parse_impl_with_owned_params() {
    // Ownership annotations work in impl methods (reuses parse_params)
    let src = "
        trait Processor { fn process(own self: Processor, own data: number) -> number; }
        impl Processor for MyProc {
            fn process(own self: MyProc, own data: number) -> number { return data; }
        }
    ";
    let (prog, diags) = parse_source(src);
    assert!(diags.is_empty());
    if let Item::Impl(ib) = &prog.items[1] {
        let param = &ib.methods[0].params[1]; // 'data' param
        assert_eq!(param.ownership, Some(OwnershipAnnotation::Own));
    }
}
```

---

## Acceptance Criteria

- [ ] `impl Trait for Type { fn method(...) { body } }` parses to `Item::Impl(ImplBlock)`
- [ ] `trait_name`, `type_name`, and `methods` are correctly populated
- [ ] Generic trait type args `impl Trait<T> for Type` parse correctly
- [ ] Missing `for` keyword produces a parse diagnostic
- [ ] Impl method without body produces a parse diagnostic
- [ ] Empty impl body `{ }` is valid
- [ ] Ownership annotations on impl method params work
- [ ] Trait + impl declarations coexist in same file
- [ ] All existing tests pass (no regressions)
- [ ] `cargo clippy -- -D warnings` clean
- [ ] `cargo fmt` clean

---

## Notes

- `parse_impl_method()` is structurally almost identical to `parse_function()`. Do NOT
  merge them — impl methods have different semantic context and the slight duplication is
  correct here. Future phases may add impl-specific attributes.
- The `for` in `impl Trait for Type` uses `TokenKind::For` (the loop keyword). This works
  because `impl` token uniquely identifies the context — no ambiguity with `for` loops.
- Impl blocks at the top level of a file only (not nested inside functions). The parser
  only calls `parse_impl_block()` from `parse_item()`.
