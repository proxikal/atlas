# Phase 04: Parser — Ownership Annotations on Return Types

**Block:** 2 (Ownership Syntax)
**Depends on:** Phase 03 (param annotations working)
**Complexity:** low
**Files to modify:**
- `crates/atlas-runtime/src/parser/mod.rs`

## Summary

Parse `own` / `borrow` before a function return type and store it in
`FunctionDecl.return_ownership`. This allows signatures like `fn allocate(size: number) -> own Buffer`.

## Current State

Verified: After the parameter list, the parser currently expects `->` followed by a `TypeRef`.
`FunctionDecl.return_ownership` field exists after Phase 02.

Syntax to support:
```atlas
fn allocate(size: number) -> own Buffer
fn first(borrow arr: array<number>) -> borrow number
fn identity(x: number) -> number   // unannotated — no change
```

## Requirements

1. After parsing `->` in a function declaration, peek at the next token:
   - If `TokenKind::Own` → consume, set `return_ownership = Some(Own)`
   - If `TokenKind::Borrow` → consume, set `return_ownership = Some(Borrow)`
   - Otherwise → `return_ownership = None`
   Then parse the `TypeRef` as normal.

2. `Shared` is NOT valid as a return ownership annotation — you cannot return a `shared`
   value directly; callers receive a `shared<T>` typed value instead. Emit parse error if
   `Shared` appears in return annotation position.

3. Store resolved `return_ownership` in `FunctionDecl`.

## Acceptance Criteria

- [ ] `fn allocate(size: number) -> own Buffer` parses with `return_ownership = Some(Own)`
- [ ] `fn peek(borrow arr: array<number>) -> borrow number` parses with
      `return_ownership = Some(Borrow)`
- [ ] `fn normal() -> number` parses with `return_ownership = None` (no regression)
- [ ] `fn bad() -> shared number` produces a parse error with clear message
- [ ] All existing parser tests continue to pass
- [ ] `cargo nextest run -p atlas-runtime` 100% passing

## Tests Required

```rust
#[test]
fn test_parse_own_return_type() {
    let src = "fn allocate(size: number) -> own Buffer { }";
    let ast = parse(src).unwrap();
    // verify FunctionDecl.return_ownership == Some(Own)
    // verify FunctionDecl.return_type == TypeRef::Named("Buffer", ...)
}

#[test]
fn test_parse_borrow_return_type() { /* -> borrow number */ }

#[test]
fn test_parse_shared_return_type_is_error() {
    let src = "fn bad() -> shared number { }";
    assert!(parse(src).is_err());
}

#[test]
fn test_unannotated_return_type_unchanged() {
    let src = "fn f() -> number { return 1; }";
    // return_ownership == None
}
```

## Notes

- The `Buffer` type referenced in examples does not exist yet in v0.3 — use `number`
  or `string` as stand-ins in tests. The parser is type-agnostic; it doesn't validate
  whether `Buffer` is a known type.
- `-> own` on a function returning a primitive (number, bool, string) is semantically
  meaningless but syntactically valid. The type-checker (Phase 06) will warn on this.
