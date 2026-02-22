# Phase 15: LSP — Ownership Annotation Completion

**Block:** 2 (Ownership Syntax)
**Depends on:** Phase 14 (semantic tokens/hover working, LSP ownership-aware)
**Complexity:** low
**Files to modify:**
- `crates/atlas-lsp/src/completion.rs`

## Summary

When writing a function signature, the LSP suggests `own`, `borrow`, `shared` as completion
items in parameter annotation position. AI code generation uses LSP completion — this
directly serves the AI-first goal.

## Current State

Verified: `completion.rs:30` has a function snippet:
`"fn ${1:name}(${2:params}) -> ${3:type} {\n\t${4}\n}"`

`completion.rs:182-193` builds parameter lists in completion items from function types.

There is currently no completion for ownership keywords inside parameter lists.

## Requirements

1. **Trigger context:** When the cursor is inside a parameter list `fn foo(|)` or after
   a comma `fn foo(x: number, |)`, and the user has typed the beginning of `own`, `borrow`,
   or `shared`, offer these as completion items.

2. **Completion items to add:**
   ```
   own     — "Move semantics: caller's binding is invalidated after call"
   borrow  — "Immutable reference: caller retains ownership, no mutation"
   shared  — "Shared reference: Arc<T> semantics, requires shared<T> value"
   ```
   Each with `CompletionItemKind::Keyword` and an appropriate `documentation` field.

3. **Snippet support:** After completing `own`, insert `own ${1:name}: ${2:Type}` if
   the editor supports snippet completion.

4. **No false triggering:** Do NOT suggest these keywords outside parameter context
   (e.g., in expression position, in type position).

5. **In `completion.rs:182`** (parameter rendering in function completions): when building
   the completion text for a function with known ownership annotations, include them:
   `"own data: array<number>"` instead of `"data: array<number>"`.

## Acceptance Criteria

- [ ] Typing `own` in parameter position triggers `own` completion suggestion
- [ ] Typing `borrow` in parameter position triggers `borrow` completion suggestion
- [ ] Typing `shared` in parameter position triggers `shared` completion suggestion
- [ ] Completion documentation explains each annotation's semantics
- [ ] Function completions show ownership annotations in parameter text
- [ ] Keywords are NOT suggested in expression or type position
- [ ] `cargo nextest run -p atlas-lsp` 100% passing

## Tests Required

```rust
#[tokio::test]
async fn test_completion_suggests_own_in_param_position() {
    // Document: fn f(o|) -> void { }   (cursor after 'o' inside params)
    // Request completion
    // Verify 'own' in completion list with KEYWORD kind
}

#[tokio::test]
async fn test_completion_no_ownership_in_expression_position() {
    // Document: fn f() -> void { let x = o|; }
    // Request completion at cursor
    // Verify 'own'/'borrow'/'shared' are NOT in completion list
}

#[tokio::test]
async fn test_function_completion_shows_ownership_in_params() {
    // Known function: fn process(own data: array<number>) -> void
    // Request completion for `process`
    // Verify completion detail text includes "own data: array<number>"
}
```

## Notes

- The completion context detection (am I inside a parameter list?) is the trickiest part.
  Use position-based heuristics from the token stream if full AST position tracking is
  unavailable — check if the cursor is between `(` and `)` of a `fn` declaration.
- This phase directly impacts AI code generation quality: AI editors using the LSP will
  get prompted with ownership annotations when writing function signatures.
