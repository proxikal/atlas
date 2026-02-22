# Phase 03: Parser — Ownership Annotations on Parameters

**Block:** 2 (Ownership Syntax)
**Depends on:** Phase 01 (tokens), Phase 02 (AST), Phase 05 (constructor fixup)
**Complexity:** medium
**Files to modify:**
- `crates/atlas-runtime/src/parser/mod.rs`

## Summary

Teach the parser to recognize `own`, `borrow`, `shared` before a parameter name and store the
annotation in `Param.ownership`. Unannotated parameters parse exactly as before — no breaking
change to existing Atlas programs.

## Current State

Verified: Parameter parsing in `parser/mod.rs:145`. The parser currently reads:
```
param_name : type_ref
```
After this phase it reads:
```
[own | borrow | shared]? param_name : type_ref
```

The keyword tokens `Own`, `Borrow`, `Shared` exist after Phase 01.
The `Param.ownership` field exists after Phase 02/05.

## Requirements

1. In the function parameter parsing loop (around line 145), before consuming the parameter
   name identifier, peek at the current token:
   - If `TokenKind::Own` → consume token, set `ownership = Some(OwnershipAnnotation::Own)`
   - If `TokenKind::Borrow` → consume token, set `ownership = Some(OwnershipAnnotation::Borrow)`
   - If `TokenKind::Shared` → consume token, set `ownership = Some(OwnershipAnnotation::Shared)`
   - Otherwise → `ownership = None` (no change to existing parse path)

2. Store the resolved `ownership` in the `Param` struct.

3. The same logic applies in BOTH places params are parsed:
   - Named function declarations (`fn foo(own data: Buffer)`)
   - Any anonymous function or closure parameter parsing (Block 4 will add these, but
     the ownership parser should already handle them if they share the same parse path)

4. Parser error: if `own`/`borrow`/`shared` token appears in param position but is NOT
   followed by a valid identifier, emit a meaningful diagnostic:
   `"Expected parameter name after ownership annotation 'own'"`.

## Acceptance Criteria

- [ ] `fn process(own data: number) -> number` parses with `param.ownership = Some(Own)`
- [ ] `fn read(borrow data: number) -> number` parses with `param.ownership = Some(Borrow)`
- [ ] `fn share(shared data: number) -> number` parses with `param.ownership = Some(Shared)`
- [ ] `fn normal(data: number) -> number` parses with `param.ownership = None` (no regression)
- [ ] Multiple params with mixed annotations parse correctly:
      `fn mixed(own a: number, borrow b: string, c: bool)`
- [ ] Missing identifier after annotation produces a parse error (not a panic)
- [ ] All existing parser tests continue to pass
- [ ] `cargo nextest run -p atlas-runtime` 100% passing

## Tests Required

Add to `parser/mod.rs` test block or `tests/frontend_integration.rs`:

```rust
#[test]
fn test_parse_own_param() {
    let src = "fn process(own data: number) -> number { return data; }";
    let ast = parse(src).unwrap();
    // extract FunctionDecl, verify params[0].ownership == Some(Own)
}

#[test]
fn test_parse_borrow_param() { /* borrow */ }

#[test]
fn test_parse_shared_param() { /* shared */ }

#[test]
fn test_parse_mixed_ownership_params() {
    // fn f(own a: number, borrow b: string, c: bool) -> void
    // a=Own, b=Borrow, c=None
}

#[test]
fn test_parse_unannotated_param_unchanged() {
    // fn f(x: number) -> number — ownership is None
}
```

## Notes

- `shared` as a parameter annotation (`shared data: Buffer`) is NOT the same as the
  `shared<T>` generic type. The parser sees `Shared` keyword in param prefix position vs.
  identifier `shared` followed by `<` in type position. These are unambiguous contexts.
- Do NOT add ownership annotation parsing to `extern fn` declarations — those use a different
  param struct (`ExternTypeAnnotation`) and are outside Block 2 scope.
