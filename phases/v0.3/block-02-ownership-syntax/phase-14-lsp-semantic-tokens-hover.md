# Phase 14: LSP — Semantic Tokens + Ownership Hover

**Block:** 2 (Ownership Syntax)
**Depends on:** Phase 06 (annotations in type system, LSP uses typechecker output)
**Complexity:** medium
**Files to modify:**
- `crates/atlas-lsp/src/semantic_tokens.rs`
- `crates/atlas-lsp/src/hover.rs`

## Summary

Two LSP improvements: (1) `own`, `borrow`, `shared` keyword tokens get their own semantic
token type so editors highlight them distinctly. (2) Function parameter hover shows ownership
annotation, turning `(parameter) data: array<number>` into
`(own parameter) data: array<number>` where annotated.

## Current State

Verified:
- `semantic_tokens.rs` — handles keyword token classification. `Own`, `Borrow`, `Shared`
  are new keywords after Phase 01; they need to be emitted as the `keyword` semantic token
  type (or a custom `modifier` if the LSP supports it).
- `hover.rs:267` — current hover for parameters:
  `hover.push_str(&format!("(parameter) {}: {:?}", symbol.name, symbol.ty))`
  After Block 2, this should include ownership when present.

## Requirements

### Semantic Tokens

1. In `semantic_tokens.rs`, the token classifier must recognize `TokenKind::Own`,
   `TokenKind::Borrow`, `TokenKind::Shared` and emit them as `SemanticTokenType::KEYWORD`.
   This is likely already handled if the classifier emits `KEYWORD` for all `TokenKind`
   that `is_keyword()` returns `Some(_)` — verify and add if not.

2. If the LSP uses `SemanticTokenModifier`, consider adding an `ownership` modifier
   for these three keywords to allow editor themes to distinguish them from control-flow
   keywords. This is a nice-to-have; the hard requirement is that they are not emitted
   as `variable` or `identifier` tokens.

### Hover

3. In `hover.rs`, when rendering a parameter symbol's hover text, check if the resolved
   function type has ownership for this parameter (from Phase 06's `TypedParam`):
   - `Some(Own)` → `"(own parameter) {name}: {type}"`
   - `Some(Borrow)` → `"(borrow parameter) {name}: {type}"`
   - `Some(Shared)` → `"(shared parameter) {name}: {type}"`
   - `None` → `"(parameter) {name}: {type}"` (unchanged)

4. Function signature hover (when hovering over a function call) should show annotations:
   `fn process(own data: array<number>) -> void`
   instead of:
   `fn process(data: array<number>) -> void`

## Acceptance Criteria

- [ ] `own`, `borrow`, `shared` tokens receive `KEYWORD` semantic token type in LSP output
- [ ] Parameter hover shows ownership annotation prefix when annotation is present
- [ ] Parameter hover unchanged for unannotated params (no regression)
- [ ] Function signature in call-site hover shows ownership annotations
- [ ] LSP tests pass (follow atlas-lsp inline-server test pattern — no helper functions)
- [ ] `cargo nextest run -p atlas-lsp` 100% passing

## Tests Required

Follow `atlas-lsp` testing pattern (inline server creation, no helper functions):

```rust
#[tokio::test]
async fn test_own_param_shows_in_hover() {
    // Create inline LSP server
    // Open document: fn process(own data: array<number>) -> void { }
    // Hover over `data` in the parameter list
    // Verify hover text contains "(own parameter) data: array<number>"
}

#[tokio::test]
async fn test_unannotated_param_hover_unchanged() {
    // fn f(x: number) -> void { }
    // Hover over x
    // Verify hover text is "(parameter) x: number" — no ownership prefix
}

#[tokio::test]
async fn test_own_keyword_is_keyword_semantic_token() {
    // Document: fn f(own x: number) -> void { }
    // Get semantic tokens
    // Verify `own` token has KEYWORD type
}
```

## Notes

- Check `crates/atlas-lsp/tests/` and existing test files before writing new tests.
  Do NOT create a new test file if a suitable domain file exists — add to it.
- LSP tests use inline server creation pattern. Never add helper functions to LSP tests.
  See auto-memory `testing-patterns.md` for the exact pattern.
