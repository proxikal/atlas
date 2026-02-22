# Phase 01: Ownership Keyword Tokens

**Block:** 2 (Ownership Syntax)
**Depends on:** Block 1 complete ✅
**Complexity:** low
**Files to modify:**
- `crates/atlas-runtime/src/token.rs`
- `crates/atlas-runtime/src/lexer/mod.rs`

## Summary

Add `own`, `borrow`, `shared` as reserved keywords in the lexer. These are the three ownership
annotation tokens that all subsequent Block 2 phases depend on.

## Current State

Verified: `token.rs` has a `// Keywords` section with `True`, `False`, `Let`, `Var`, `Fn`,
`If`, `Else`, `While`, `For`, `In`, `Return` etc. The keywords `own`, `borrow`, `shared` are
NOT present — any identifier named `own`, `borrow`, or `shared` currently lexes as `Ident`.

The lexer `is_keyword()` function (used in identifier → keyword promotion) has no entry for
these three words.

## Requirements

1. Add `TokenKind::Own`, `TokenKind::Borrow`, `TokenKind::Shared` to the keyword section of
   `token.rs`.
2. In `lexer/mod.rs` `is_keyword()` match arm: map `"own"` → `Own`, `"borrow"` → `Borrow`,
   `"shared"` → `Shared`.
3. In the `as_str()` / `Display` impl for `TokenKind`: map back `Own` → `"own"`, etc.
4. Add these keywords to the existing keyword unit tests in `token.rs`.

## Acceptance Criteria

- [ ] `TokenKind::Own`, `TokenKind::Borrow`, `TokenKind::Shared` exist and derive
      `Debug, Clone, PartialEq`
- [ ] Lexer produces `Own` token when source contains the identifier `own`
- [ ] Lexer produces `Borrow` token when source contains the identifier `borrow`
- [ ] Lexer produces `Shared` token when source contains the identifier `shared`
- [ ] `as_str()` round-trips each token back to the keyword string
- [ ] Existing keyword tests continue to pass
- [ ] `cargo nextest run -p atlas-runtime` 100% passing
- [ ] `cargo clippy -p atlas-runtime -- -D warnings` clean
- [ ] `cargo fmt --check -p atlas-runtime` clean

## Tests Required

In the existing keyword unit test block in `token.rs`:
```rust
assert_eq!(TokenKind::is_keyword("own"), Some(TokenKind::Own));
assert_eq!(TokenKind::is_keyword("borrow"), Some(TokenKind::Borrow));
assert_eq!(TokenKind::is_keyword("shared"), Some(TokenKind::Shared));
assert_eq!(TokenKind::Own.as_str(), "own");
assert_eq!(TokenKind::Borrow.as_str(), "borrow");
assert_eq!(TokenKind::Shared.as_str(), "shared");
```

In a lexer integration test — verify full token stream:
```rust
// "fn process(own data: number) -> number" lexes with Own token at correct position
```

## Notes

- These three keywords were previously valid identifiers. Any Atlas source that used `own`,
  `borrow`, or `shared` as variable names will now fail to parse. There are no such usages
  in the current test suite (verified: grep found zero hits).
- `shared` keyword for PARAMETER annotations is distinct from `shared<T>` TYPE syntax —
  the type syntax uses `Generic { name: "shared", ... }` in the AST. The lexer keyword
  only fires in identifier position; the parser decides context.
