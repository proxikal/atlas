# Phase 05 — Parser: Trait Bounds on Generics

**Block:** 3 (Trait System)
**Depends on:** Phase 04 complete
**Estimated tests added:** 10–14

---

## Objective

Parse `:` trait bounds on generic type parameters: `fn foo<T: Copy>(x: T)`.
Populate `TypeParam.trait_bounds: Vec<TraitBound>`. Support multiple bounds
with `+`: `fn foo<T: Copy + Display>(x: T)`.

---

## Current State (verified 2026-02-22)

`crates/atlas-runtime/src/parser/mod.rs`:
- `parse_type_params()` (extracted in Phase 03) handles `<T>` and `<T extends Bound>`
- `TypeParam.bound: Option<TypeRef>` uses `Extends` token
- `TypeParam.trait_bounds: Vec<TraitBound>` added in Phase 02, always `vec![]` so far
- `TokenKind::Colon` — need to verify this exists

Check: does `TokenKind::Colon` exist?

```bash
grep -n "Colon" crates/atlas-runtime/src/token.rs
```

If `Colon` doesn't exist, add it to `token.rs` first in this phase.

---

## Changes

### `crates/atlas-runtime/src/token.rs` (if needed)

If `TokenKind::Colon` doesn't exist:
```rust
/// `:` — used in trait bounds `T: Trait`
Colon,
```
Add to `as_str()`: `TokenKind::Colon => ":"`.
Add to lexer character dispatch: `':' => self.make_token(TokenKind::Colon)`.

### `crates/atlas-runtime/src/parser/mod.rs`

**Extend `parse_type_params()` to handle `:` bounds:**

```rust
fn parse_type_params(&mut self) -> Result<Vec<TypeParam>, ()> {
    let mut type_params = Vec::new();
    if self.match_token(TokenKind::Less) {
        loop {
            let start_span = self.peek().span;
            let name_tok = self.consume_identifier("a type parameter name")?;
            let name = name_tok.lexeme.clone();
            let name_span = name_tok.span;

            // Existing: `extends` type-level bound
            let mut bound = None;
            if self.match_token(TokenKind::Extends) {
                bound = Some(self.parse_type_ref()?);
            }

            // NEW: `:` trait bounds (one or more, separated by `+`)
            let mut trait_bounds = Vec::new();
            if self.match_token(TokenKind::Colon) {
                loop {
                    let bound_start = self.peek().span;
                    let trait_name_tok = self.consume_identifier("a trait name")?;
                    let bound_end = trait_name_tok.span;
                    trait_bounds.push(TraitBound {
                        trait_name: trait_name_tok.lexeme.clone(),
                        span: bound_start.merge(bound_end),
                    });
                    if !self.match_token(TokenKind::Plus) {
                        break;
                    }
                }
            }

            type_params.push(TypeParam {
                name,
                bound,
                trait_bounds,
                span: start_span.merge(name_span),
            });

            if !self.match_token(TokenKind::Comma) {
                break;
            }
        }
        self.consume(TokenKind::Greater, "Expected '>' after type parameters")?;
    }
    Ok(type_params)
}
```

**Note on `+` token:** Check if `TokenKind::Plus` exists (it's used for arithmetic `+`).
It likely exists. If not, add it. Use the existing `Plus` token for `+` in trait bounds.

---

## Atlas Syntax Supported After This Phase

```atlas
// Single trait bound
fn copy_it<T: Copy>(x: T) -> T {
    return x;
}

// Multiple bounds
fn show_copy<T: Copy + Display>(x: T) -> string {
    return x.display();
}

// Multiple type params with bounds
fn pair_display<T: Display, U: Display>(a: T, b: U) -> string {
    return a.display() + b.display();
}

// Bound on existing generic function
fn identity<T>(x: T) -> T { return x; }      // unchanged, no bounds
fn safe_copy<T: Copy>(x: T) -> T { return x; } // with bound
```

---

## Tests

Add to `crates/atlas-runtime/tests/frontend_syntax.rs`:

```rust
#[test]
fn test_parse_type_param_single_trait_bound() {
    let src = "fn foo<T: Copy>(x: T) -> T { return x; }";
    let (prog, diags) = parse_source(src);
    assert!(diags.is_empty());
    if let Item::Function(f) = &prog.items[0] {
        assert_eq!(f.type_params.len(), 1);
        assert_eq!(f.type_params[0].name, "T");
        assert_eq!(f.type_params[0].trait_bounds.len(), 1);
        assert_eq!(f.type_params[0].trait_bounds[0].trait_name, "Copy");
    }
}

#[test]
fn test_parse_type_param_multiple_trait_bounds() {
    let src = "fn foo<T: Copy + Display>(x: T) -> string { return x.display(); }";
    let (prog, diags) = parse_source(src);
    assert!(diags.is_empty());
    if let Item::Function(f) = &prog.items[0] {
        assert_eq!(f.type_params[0].trait_bounds.len(), 2);
        assert_eq!(f.type_params[0].trait_bounds[0].trait_name, "Copy");
        assert_eq!(f.type_params[0].trait_bounds[1].trait_name, "Display");
    }
}

#[test]
fn test_parse_multiple_type_params_with_bounds() {
    let src = "fn pair<T: Display, U: Display>(a: T, b: U) -> string { return \"\"; }";
    let (prog, diags) = parse_source(src);
    assert!(diags.is_empty());
    if let Item::Function(f) = &prog.items[0] {
        assert_eq!(f.type_params.len(), 2);
        assert_eq!(f.type_params[0].trait_bounds[0].trait_name, "Display");
        assert_eq!(f.type_params[1].trait_bounds[0].trait_name, "Display");
    }
}

#[test]
fn test_parse_type_param_no_bound_unchanged() {
    // Existing unbounded type params still work
    let src = "fn identity<T>(x: T) -> T { return x; }";
    let (prog, diags) = parse_source(src);
    assert!(diags.is_empty());
    if let Item::Function(f) = &prog.items[0] {
        assert_eq!(f.type_params[0].trait_bounds.len(), 0);
        assert!(f.type_params[0].bound.is_none());
    }
}

#[test]
fn test_parse_extends_bound_still_works() {
    // Existing `extends` bound syntax is preserved
    let src = "fn foo<T extends number>(x: T) -> T { return x; }";
    let (prog, diags) = parse_source(src);
    assert!(diags.is_empty());
    if let Item::Function(f) = &prog.items[0] {
        assert!(f.type_params[0].bound.is_some());
        assert_eq!(f.type_params[0].trait_bounds.len(), 0);
    }
}

#[test]
fn test_parse_trait_method_with_bounded_type_param() {
    let src = "trait Printer { fn print<T: Display>(value: T) -> void; }";
    let (prog, diags) = parse_source(src);
    assert!(diags.is_empty());
    if let Item::Trait(t) = &prog.items[0] {
        let method = &t.methods[0];
        assert_eq!(method.type_params[0].trait_bounds[0].trait_name, "Display");
    }
}

#[test]
fn test_parse_impl_method_with_bounded_type_param() {
    let src = "
        trait Printer { fn print<T: Display>(value: T) -> void; }
        impl Printer for ConsolePrinter {
            fn print<T: Display>(value: T) -> void { }
        }
    ";
    let (prog, diags) = parse_source(src);
    assert!(diags.is_empty());
    if let Item::Impl(ib) = &prog.items[1] {
        let method = &ib.methods[0];
        assert_eq!(method.type_params[0].trait_bounds[0].trait_name, "Display");
    }
}
```

---

## Acceptance Criteria

- [ ] `<T: TraitName>` parses into `TypeParam.trait_bounds` with one entry
- [ ] `<T: Trait1 + Trait2>` parses into two `TraitBound` entries
- [ ] Multiple type params each get their own `trait_bounds`
- [ ] Existing `<T>` (no bounds) still works — `trait_bounds` is empty
- [ ] Existing `<T extends Bound>` still works — `bound` field populated as before
- [ ] Trait method and impl method type params both support `:` bounds
- [ ] All existing generic function tests pass (no regressions)
- [ ] `cargo clippy -- -D warnings` clean
- [ ] `cargo fmt` clean

---

## Notes

- `+` for multiple bounds uses `TokenKind::Plus` — same token as arithmetic `+`.
  No ambiguity: within `< >` after a trait name, `+` can only mean "another bound."
- If `Colon` token doesn't exist, this phase adds it. Verify against `token.rs` first.
- The `extends` bound and `:` trait bounds are complementary, not alternatives.
  A type param could theoretically have both, but the typechecker will validate
  their interaction (Phase 09).
- Trait bounds in impl method type params (`fn method<T: Copy>`) are parsed correctly
  but typechecker doesn't enforce them yet — enforcement is Phase 10.
