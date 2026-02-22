# Phase 15 — LSP: Semantic Tokens + Hover for Traits

**Block:** 3 (Trait System)
**Depends on:** Phase 14 complete
**Estimated tests added:** 12–16

---

## Objective

Add LSP support for `trait` and `impl` declarations:
1. **Semantic tokens:** `trait`/`impl` keywords classified as `KEYWORD`; trait names
   classified as `TYPE`
2. **Hover:** hovering on `trait Name` shows the trait signature; hovering on
   `impl Trait for Type` shows which methods are implemented

---

## Current State (verified after Phase 14)

`crates/atlas-lsp/src/`:
- `semantic_tokens.rs` — classifies tokens; `own`/`borrow`/`shared` already classified as KEYWORD
- `hover.rs` — `find_keyword_hover()` handles individual keywords; ownership hover added in Block 2
- `token.rs`: `Trait` and `Impl` are keywords (Phase 01) — LSP should auto-classify as KEYWORD
  via existing keyword wildcard match (check the semantic token classifier logic)

---

## Investigation

```bash
grep -n "Own\|Borrow\|Shared\|keyword\|KEYWORD" \
  crates/atlas-lsp/src/semantic_tokens.rs | head -20
```

If `own`/`borrow`/`shared` were auto-classified as KEYWORD via a wildcard `is_keyword()` check,
then `trait` and `impl` will also be automatically classified (since Phase 01 added them to
`is_keyword()`). Verify this is the case — if yes, semantic tokens require only a test update.

---

## Changes

### `crates/atlas-lsp/src/semantic_tokens.rs`

If NOT auto-classified, add explicit classification:
```rust
TokenKind::Trait | TokenKind::Impl => SemanticTokenType::KEYWORD,
```

For trait names (identifiers that are trait declarations), classify as `TYPE`:
```rust
// When identifier is the name of a trait (follows `trait` keyword) → TYPE
// When identifier is the name in `impl X` → TYPE
// This requires context-aware classification based on preceding token
```

### `crates/atlas-lsp/src/hover.rs`

**Add trait keyword hover in `find_keyword_hover()`:**

```rust
"trait" => Some(
    "Declares a trait — a named set of method signatures that types can implement.\n\n\
     ```atlas\n\
     trait Display {\n    fn display(self: Display) -> string;\n}\n\
     ```"
    .to_string()
),
"impl" => Some(
    "Implements a trait for a type. All trait methods must be provided with matching signatures.\n\n\
     ```atlas\n\
     impl Display for number {\n    fn display(self: number) -> string { return str(self); }\n}\n\
     ```"
    .to_string()
),
```

**Add trait declaration hover:**

When the cursor is on a trait name in a `trait Foo { ... }` declaration, show:
```
(trait) Foo
Methods: display(self: Display) -> string
```

**Add impl block hover:**

When cursor is on the type name in `impl Display for number`:
```
(impl) number implements Display
Methods: display(self: number) -> string
```

---

## Implementation Plan

The hover provider walks the AST (`DocumentState`) to find the node at the cursor position.
This already happens for functions, variables, and ownership annotations.

For traits:
1. Check if the cursor is on an identifier in a `TraitDecl` node → show trait hover
2. Check if cursor is on the trait name in an `ImplBlock` → show impl hover
3. Check if cursor is on the type name in an `ImplBlock` → show "X implements Trait" info

These require the LSP document state to include `Item::Trait` and `Item::Impl` in the
AST it exposes. Verify `DocumentState` parses the full AST including trait/impl items.

---

## Tests

Add to `crates/atlas-lsp/tests/lsp_hover_tests.rs`:

```rust
#[tokio::test]
async fn test_hover_trait_keyword() {
    let (service, socket) = LspService::new(|client| AtlasLspServer::new(client));
    // ... standard LSP test setup ...
    // Open document with `trait Display { ... }`
    // Hover over `trait` keyword
    // Expect hover contains "Declares a trait"
}

#[tokio::test]
async fn test_hover_impl_keyword() {
    let (service, socket) = LspService::new(|client| AtlasLspServer::new(client));
    // Hover over `impl` keyword
    // Expect hover contains "Implements a trait"
}

#[tokio::test]
async fn test_hover_trait_name_in_declaration() {
    let (service, socket) = LspService::new(|client| AtlasLspServer::new(client));
    // Source: `trait Display { fn display(self: Display) -> string; }`
    // Hover over `Display` (the trait name)
    // Expect: "(trait) Display" + method list
}

#[tokio::test]
async fn test_hover_trait_name_in_impl_block() {
    let (service, socket) = LspService::new(|client| AtlasLspServer::new(client));
    // Source: `impl Display for number { ... }`
    // Hover over `Display` in the impl header
    // Expect: trait signature info
}
```

Add to `crates/atlas-lsp/tests/lsp_tokens_tests.rs`:

```rust
#[tokio::test]
async fn test_trait_keyword_semantic_token() {
    let (service, socket) = LspService::new(|client| AtlasLspServer::new(client));
    // Source: `trait Foo { }`
    // Check that `trait` token is classified as KEYWORD
    // Check that `Foo` token after `trait` is classified as TYPE (or at least not VARIABLE)
}

#[tokio::test]
async fn test_impl_keyword_semantic_token() {
    let (service, socket) = LspService::new(|client| AtlasLspServer::new(client));
    // Source: `impl Foo for number { }`
    // Check that `impl` token is classified as KEYWORD
    // Check that `for` token in impl context is classified as KEYWORD
}
```

---

## Acceptance Criteria

- [ ] `trait` keyword → `KEYWORD` semantic token
- [ ] `impl` keyword → `KEYWORD` semantic token
- [ ] `trait` hover returns description + syntax example
- [ ] `impl` hover returns description + syntax example
- [ ] Hover on trait name in declaration shows trait signature
- [ ] Hover on trait name in impl header shows trait definition
- [ ] All existing LSP tests pass (no regressions)
- [ ] `cargo clippy -- -D warnings` clean
- [ ] `cargo fmt` clean

---

## Notes

- Follow Block 2's LSP pattern exactly (Phases 14–15 added ownership LSP support).
- **No helper functions in LSP tests.** Each test creates its own server inline.
  This is a non-negotiable architectural rule from `atlas-lsp/src/CLAUDE.md`.
- If `trait`/`impl` are auto-classified as KEYWORD via the wildcard `is_keyword()` check,
  this phase mainly adds the hover text and tests. Semantic token classification may
  require only a test to verify the auto-behavior.
