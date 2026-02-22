# Phase 01 — Token Additions: `trait` and `impl`

**Block:** 3 (Trait System)
**Depends on:** Block 2 complete ✅
**Estimated tests added:** 8–12

---

## Objective

Add `trait` and `impl` as keywords to the Atlas lexer. These are the two new reserved words
required by the trait system. `for` is already a keyword (for loops) and is reused in
`impl Trait for Type` — no new token needed for it.

---

## Current State (verified 2026-02-22)

`crates/atlas-runtime/src/token.rs`:
- `TokenKind` has 90+ variants
- Ownership keywords added in Block 2: `Own`, `Borrow`, `Shared` at lines 93–99
- Keyword map (`is_keyword`) at line ~211
- `as_str()` mapping at line ~250
- `Extends` already exists (line 89) for type param bounds
- **`Trait` and `Impl` are ABSENT** — confirmed by grep

---

## Changes

### `crates/atlas-runtime/src/token.rs`

1. Add to `TokenKind` enum (after the ownership annotation block):
   ```rust
   // Trait system (v0.3+)
   /// `trait` keyword
   Trait,
   /// `impl` keyword
   Impl,
   ```

2. Add to `is_keyword()` match arm:
   ```rust
   "trait" => Some(TokenKind::Trait),
   "impl" => Some(TokenKind::Impl),
   ```

3. Add to `as_str()` match arm:
   ```rust
   TokenKind::Trait => "trait",
   TokenKind::Impl => "impl",
   ```

4. Verify `trait` and `impl` are NOT in `is_identifier_start()` or any other exclusion list
   that would prevent them from being lexed as identifiers first (keyword promotion handles this
   automatically in the lexer — same pattern as `own`, `borrow`, `shared`).

---

## Tests

Add to the existing keyword test block in `token.rs` (inline `#[cfg(test)]` module):

```rust
#[test]
fn test_trait_impl_keywords() {
    assert_eq!(TokenKind::is_keyword("trait"), Some(TokenKind::Trait));
    assert_eq!(TokenKind::is_keyword("impl"), Some(TokenKind::Impl));
    assert_eq!(TokenKind::Trait.as_str(), "trait");
    assert_eq!(TokenKind::Impl.as_str(), "impl");
    assert_ne!(TokenKind::is_keyword("trait"), None);
    assert_ne!(TokenKind::is_keyword("impl"), None);
}

#[test]
fn test_trait_impl_not_identifiers() {
    // 'trait' and 'impl' must NOT be usable as variable names
    // (they lex as keywords, not identifiers)
    assert!(TokenKind::is_keyword("trait").is_some());
    assert!(TokenKind::is_keyword("impl").is_some());
}

#[test]
fn test_for_already_keyword() {
    // 'for' is already a keyword — no change needed
    assert_eq!(TokenKind::is_keyword("for"), Some(TokenKind::For));
}
```

Also add to the `frontend_syntax.rs` integration tests:

```rust
#[test]
fn test_trait_keyword_lexes_correctly() {
    // 'trait' as a keyword should not be usable as identifier
    // Using 'trait' as a variable name should fail to parse
    let atlas = Atlas::new();
    let result = atlas.eval("let trait = 1;");
    assert!(result.is_err(), "trait is a keyword, not an identifier");
}

#[test]
fn test_impl_keyword_lexes_correctly() {
    let atlas = Atlas::new();
    let result = atlas.eval("let impl = 1;");
    assert!(result.is_err(), "impl is a keyword, not an identifier");
}
```

---

## Acceptance Criteria

- [ ] `TokenKind::Trait` and `TokenKind::Impl` exist in the enum
- [ ] `is_keyword("trait")` returns `Some(TokenKind::Trait)`
- [ ] `is_keyword("impl")` returns `Some(TokenKind::Impl)`
- [ ] `Trait.as_str()` returns `"trait"`
- [ ] `Impl.as_str()` returns `"impl"`
- [ ] Using `trait` or `impl` as a variable name is a parse error
- [ ] All existing tests pass (no regressions)
- [ ] `cargo clippy -- -D warnings` clean
- [ ] `cargo fmt` clean

---

## Notes

- `for` is already `TokenKind::For` — the `impl Foo for TypeName` syntax uses the existing
  token. No new token needed.
- The lexer uses identifier promotion (lex as identifier, check keyword map) — `trait` and
  `impl` will follow the same path as all other keywords.
- Pattern: identical to how `own`, `borrow`, `shared` were added in Block 2 Phase 01.
