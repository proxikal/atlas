# Phase v02-completion-02: match — Guard Clauses and OR Patterns

## Dependencies

**Required:** v02-completion-01 complete (50+ passing match tests)

**Verification:**
```bash
cargo nextest run -p atlas-runtime --test pattern_matching   # must pass
grep "guard" crates/atlas-runtime/src/ast.rs               # must output nothing (pre-condition)
```

**If missing:** Complete phase-01 first. Do not add new syntax on top of untested foundations.

---

## Objective

The spec documents guard clauses (`pattern if condition => expr`) and OR patterns (`0 | 1 | 2 => expr`) as features of the match system, but neither exists anywhere in the codebase — not in the AST, parser, binder, typechecker, or engines. This phase adds both features completely across all 6 layers: AST → parser → binder → typechecker → interpreter → VM compiler.

---

## Files

**Update:**
- `crates/atlas-runtime/src/ast.rs` — add `guard: Option<Box<Expr>>` to `MatchArm`; add `Pattern::Or` variant
- `crates/atlas-runtime/src/parser/expr.rs` — parse `if <expr>` guard after pattern; parse `|` between patterns
- `crates/atlas-runtime/src/binder.rs` — visit guard expressions; visit OR pattern sub-patterns
- `crates/atlas-runtime/src/typechecker/expr.rs` — guard must be `bool`; OR sub-patterns must match scrutinee type; exhaustiveness aware of OR patterns
- `crates/atlas-runtime/src/interpreter/expr.rs` — eval guard before accepting arm; try each OR sub-pattern
- `crates/atlas-runtime/src/compiler/expr.rs` — emit guard check bytecode (JumpIfFalse on guard); compile OR pattern as sequential sub-pattern checks

**Create:**
- (no new files — all changes are to existing files)

**Tests (add to existing `tests/pattern_matching.rs`):**
- ~150 additional lines, 30+ new test cases for guard clauses and OR patterns

**Total new code:** ~300 lines across 6 files + ~150 lines tests
**Minimum test count:** 30 new tests (80+ total in pattern_matching.rs)

---

## Dependencies (Components)

- `MatchArm` struct (existing — `ast.rs`, needs `guard` field)
- `Pattern` enum (existing — `ast.rs`, needs `Or` variant)
- Parser expression parsing (existing — `parser/expr.rs`)
- Binder visitor (existing — `binder.rs`)
- TypeChecker `check_match` (existing — `typechecker/expr.rs`)
- Interpreter `eval_match` (existing — `interpreter/expr.rs`)
- Compiler `compile_match` (existing — `compiler/expr.rs`)

---

## Implementation Notes

**AST changes (do first — everything else depends on this):**
```rust
// In MatchArm:
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Box<Expr>>,   // NEW: pattern if <guard> => body
    pub body: Expr,
    pub span: Span,
}

// In Pattern enum:
pub enum Pattern {
    // ... existing variants ...
    Or(Vec<Pattern>, Span),         // NEW: 0 | 1 | 2
}
```

**Parser changes:**
- After parsing a pattern, if next token is `if`, parse the guard expression
- OR patterns: parse primary pattern, then loop `|` + pattern while `|` is next
  - OR patterns bind tighter than `=>` but looser than constructor args
  - `Ok(x) | Err(x)` — both sub-patterns bind same variable name `x`
  - `0 | 1 | 2` — no bindings in any sub-pattern

**Binder changes:**
- Visit `arm.guard` if present (it's an expression — use existing `visit_expr`)
- Visit all sub-patterns in `Pattern::Or` (each sub-pattern visited individually)
- OR patterns: all sub-patterns must bind the same set of variable names

**Typechecker changes:**
- Guard: `check_expr(guard)` must return `Type::Bool`; error if not (AT3029 — guard must be bool)
- OR sub-patterns: each sub-pattern checked independently against scrutinee type
- Exhaustiveness: an OR pattern covering `Some | None` = exhaustive for Option; an OR pattern covering `Ok | Err` = exhaustive for Result

**Interpreter changes (eval_match):**
```rust
// After checking if pattern matched:
if let Some(bindings) = self.try_match_pattern(&arm.pattern, &scrutinee) {
    // NEW: check guard if present
    if let Some(guard_expr) = &arm.guard {
        self.push_scope();
        for (name, val) in &bindings { /* bind */ }
        let guard_result = self.eval_expr(guard_expr)?;
        self.pop_scope();
        if guard_result != Value::Bool(true) {
            continue; // Guard failed — try next arm
        }
    }
    // ... existing: push scope, bind, eval body ...
}
```

**Compiler changes (compile_match):**
- After matching pattern (success flag on stack), if arm has guard:
  - Emit conditional: `JumpIfFalse` to next arm if guard evaluates to false
  - Guard expression compiled in scope where pattern bindings are locals
- OR patterns: compile as `compile_pattern_check(sub1) || compile_pattern_check(sub2)`
  - Use `Or` bytecode logic: try sub1, if false try sub2, if any succeeds → matched

**Error handling:**
- AT3029 — guard expression must be `bool`, e.g., `pattern if 42 => ...` is an error
- Guard failure is NOT an error — it means "this arm didn't match, try next"

**Critical requirements:**
- Guard failure must try the NEXT arm, not terminate the match
- All OR sub-patterns must bind the same variable names (for type safety)
- Parity: guard evaluation must behave identically in interpreter and VM

---

## Tests (TDD Approach)

**Guard clauses:** (15 tests)
1. Basic guard: `match x { n if n > 0 => "pos", _ => "non-pos" }` → works
2. Guard false: guard fails → next arm tried
3. Guard false: no subsequent arm → non-exhaustive error
4. Guard uses bound variable: `Some(x) if x > 10 => x` → x available in guard
5. Guard type error: `n if 42` → AT3029 (guard must be bool)
6. Guard accesses outer scope: `match val { x if x == limit => "hit" }`
7. Multiple guarded arms: first matching guard wins
8. Guard with function call: `x if is_valid(x) => process(x)`
9. Guarded arm + unguarded wildcard: `n if n > 0 => "pos", _ => "other"`
10. Guard with boolean expression: `x if x > 0 && x < 10 => "single digit"`
11. Guard on constructor: `Ok(v) if v != 0 => v, Ok(_) => 0, Err(e) => -1`
12. Exhaustiveness with guarded arms: guarded arm does NOT satisfy exhaustiveness
13. Guard parity: interpreter and VM produce identical result
14. Guard with side effects (function call): executes at most once
15. Multiple bound variables in guard: `[a, b] if a < b => ...`

**OR patterns:** (15 tests)
1. Basic OR: `match x { 0 | 1 => "small", _ => "big" }` → works
2. Three-way OR: `0 | 1 | 2 => "tiny"` → matches any of the three
3. OR with variable: `Ok(x) | Err(x) => x` — same binding name in both arms
4. OR binding type mismatch: `Ok(x) | Err(x)` where T ≠ E → type error
5. OR exhaustiveness: `Some(_) | None =>` without other arms → exhaustive for Option
6. OR exhaustiveness: `Ok(_) | Err(_) =>` → exhaustive for Result
7. OR does not match: `0 | 1` when value is 2 → falls to next arm
8. OR string patterns: `"yes" | "y" | "true" => true, _ => false`
9. OR with wildcard in sub-pattern: `0 | _` → always matches (wildcard in OR)
10. OR parity: interpreter and VM produce same result
11. OR does not bind when no binding in sub-pattern: `0 | 1 => "zero or one"`
12. Nested OR: `Ok(0 | 1) => "small ok"` — OR inside constructor arg
13. OR in first arm, wildcard in second: order preserved (first match wins)
14. OR pattern + guard: `0 | 1 if x > 0 => ...` — guard applies to full OR pattern
15. All-literal OR covers bool: `true | false =>` → exhaustive for bool

**Minimum test count:** 30 new tests (80+ total in pattern_matching.rs)

**Parity requirement:** All tests run in both interpreter and VM with identical results.

---

## Acceptance Criteria

- ✅ `guard: Option<Box<Expr>>` added to `MatchArm` in `ast.rs`
- ✅ `Pattern::Or(Vec<Pattern>, Span)` added to `Pattern` enum
- ✅ Parser handles `pattern if <expr> => body` syntax
- ✅ Parser handles `pattern | pattern | pattern => body` syntax
- ✅ Binder visits guard expressions and OR sub-patterns
- ✅ Typechecker: guard must be bool (AT3029 on violation)
- ✅ Typechecker: OR sub-patterns must match scrutinee type
- ✅ Typechecker: exhaustiveness is aware of OR patterns
- ✅ Interpreter: guard failure tries next arm correctly
- ✅ Compiler: guard and OR patterns work correctly in bytecode
- ✅ 30+ new tests pass (80+ total in pattern_matching.rs)
- ✅ 100% interpreter/VM parity for all new tests
- ✅ No clippy warnings
- ✅ `cargo nextest run -p atlas-runtime` passes

---

## References

**Specifications:** `docs/specification/types.md` (Pattern Matching section — guard clauses and OR patterns documented as current limitations)
**Related phases:** v02-completion-01 (prerequisite), v03 language features
