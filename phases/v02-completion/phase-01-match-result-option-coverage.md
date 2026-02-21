# Phase v02-completion-01: match / Result / Option — Coverage and Correctness

## Dependencies

**Required:** v0.2 main branch clean (all tests passing)

**Verification:**
```bash
cargo nextest run -p atlas-runtime 2>&1 | tail -3   # must show 0 failures
ls crates/atlas-runtime/src/interpreter/expr.rs      # eval_match must exist
ls crates/atlas-runtime/src/compiler/expr.rs         # compile_match must exist
ls crates/atlas-runtime/src/typechecker/expr.rs      # check_match must exist
```

**If missing:** Do not proceed — base is broken.

---

## Objective

`match`, `Result<T,E>`, and `Option<T>` are implemented across all layers (parser, binder, typechecker, interpreter, compiler) but have no dedicated integration test file. There are only ~258 scattered references across broader test files. This phase creates a comprehensive dedicated test suite that exercises every pattern type, every engine, and every edge case — and fixes any bugs discovered in the process.

---

## Files

**Create:**
- `crates/atlas-runtime/tests/pattern_matching.rs` — ~350 lines, 50+ test cases

**Update (bug fixes only — minimal, targeted):**
- `crates/atlas-runtime/src/interpreter/expr.rs` — fix any bugs found during testing
- `crates/atlas-runtime/src/compiler/expr.rs` — fix any bugs found during testing
- `crates/atlas-runtime/src/typechecker/expr.rs` — fix any exhaustiveness edge cases

**Total new code:** ~350 lines test + fixes as needed
**Minimum test count:** 50 tests

---

## Dependencies (Components)

- `Value::Option`, `Value::Result` (existing — `value.rs`)
- `eval_match`, `try_match_pattern` (existing — `interpreter/expr.rs`)
- `compile_match`, `compile_pattern_check` (existing — `compiler/expr.rs`)
- `check_match`, `check_exhaustiveness` (existing — `typechecker/expr.rs`)

---

## Implementation Notes

**Key patterns to analyze (BEFORE writing any tests):**
- Read `interpreter/expr.rs:505-658` — understand `eval_match` and `try_match_constructor` fully
- Read `compiler/expr.rs:301-447` — understand how bytecode match works
- Read `typechecker/expr.rs:873-960` — understand exhaustiveness logic
- Check `tests/interpreter.rs`, `tests/vm.rs`, `tests/typesystem.rs` — enumerate what's already covered

**Discovery process (run each test category as you write it, fix before continuing):**
1. Write test → run → observe → fix if broken → next test
2. Do NOT batch 50 tests and run at end — discover bugs early

**Critical requirements:**
- Every test MUST run both interpreter and VM, verify identical output (parity)
- Exhaustiveness errors must produce correct error codes (AT3027 for Option, AT3028 for Result)
- Pattern variable bindings must be immutable in both engines
- Nested patterns must work: `Ok(Some(value))`, `Err(None)`, etc.

**Error handling:**
- AT3027 — Non-exhaustive Option match (missing Some or None)
- AT3028 — Non-exhaustive Result match (missing Ok or Err)
- AT3020 — Empty match expression
- AT3021 — Incompatible arm types

---

## Tests (TDD Approach)

**Literal patterns:** (8 tests)
1. Match on number literal: `match 42 { 42 => "yes", _ => "no" }`
2. Match on string literal: `match "hi" { "hi" => 1, _ => 0 }`
3. Match on bool true/false
4. Match on null literal
5. Multiple literal arms with fallthrough to wildcard
6. Literal that doesn't match → wildcard catches it
7. First-match-wins: duplicate literals, first wins
8. No match without wildcard → exhaustiveness error (type error)

**Variable binding patterns:** (6 tests)
1. Simple variable binding: `match 5 { x => x + 1 }` → 6
2. Variable binding is immutable (cannot reassign in arm)
3. Variable binding scoped to arm body only
4. Variable in nested position (not just top-level)
5. Wildcard `_` binds nothing, matches anything
6. Variable shadows outer scope (binding takes precedence)

**Constructor patterns — Option:** (10 tests)
1. `Some(x)` matches `Some(42)` → binds x = 42
2. `None` matches `None` → no bindings
3. Nested: `Some(Some(x))` matches `Some(Some(99))` → x = 99
4. Exhaustiveness: match Option with only `Some` → AT3027
5. Exhaustiveness: match Option with only `None` → AT3027
6. Exhaustiveness: wildcard makes Option exhaustive
7. Type of bound value: `Some(x)` where option is `Option<string>` → x is string
8. `Some` arm runs for Some, `None` arm runs for None
9. Result from stdlib function used in match
10. Option in function return position

**Constructor patterns — Result:** (10 tests)
1. `Ok(x)` matches `Ok(42)` → binds x = 42
2. `Err(e)` matches `Err("msg")` → binds e = "msg"
3. Nested: `Ok(Ok(x))` matches `Ok(Ok(1))` → x = 1
4. Exhaustiveness: match Result with only `Ok` → AT3028
5. Exhaustiveness: match Result with only `Err` → AT3028
6. Exhaustiveness: wildcard makes Result exhaustive
7. Error type can be any type (string, number)
8. Real use-case: function returning Result, caller matches it
9. Chain: `Ok(val)` → compute something, `Err(e)` → return Err
10. Result in function return position

**Array patterns:** (6 tests)
1. Empty array `[]` matches `[]`, not `[1]`
2. Single element `[x]` matches `[42]` → x = 42
3. Two elements `[a, b]` matches `[1, 2]` → a=1, b=2
4. Length mismatch fails → next arm tried
5. Nested array: `[[x]]` matches `[[5]]` → x = 5
6. Array pattern with wildcard element: `[_, x]`

**Exhaustiveness and type checking:** (6 tests)
1. All arms must return same type — type mismatch → AT3021
2. Match is an expression — result usable in assignment
3. Bool exhaustiveness: `true` + `false` → exhaustive without wildcard
4. Number match with wildcard → exhaustive (numbers aren't enumerable)
5. String match with wildcard → exhaustive
6. Empty match body → AT3020

**Parity — identical output interpreter vs VM:** (4 tests)
1. Simple literal match — both return same value
2. Constructor match — both return same bound value
3. Nested pattern — both return same result
4. Side-effecting arm (print) — both produce same output

**Minimum test count:** 50 tests

**Parity requirement:** All tests run in both interpreter and VM with identical results.

**Test approach:**
- Use `rstest` `#[case(...)]` for parameterized cases where patterns are similar
- Use existing `eval_ok` / `eval_err` / `vm_eval` / `vm_eval_ok` helpers from `common/`
- Add parity helper: runs source in both engines, asserts identical result

---

## Acceptance Criteria

- ✅ `crates/atlas-runtime/tests/pattern_matching.rs` created with 50+ tests
- ✅ All pattern types tested: Literal, Wildcard, Variable, Constructor (Some/None/Ok/Err), Array
- ✅ Exhaustiveness checking verified for Option and Result
- ✅ Interpreter/VM parity verified for all test cases
- ✅ Any bugs discovered during testing are fixed (documented in commit message)
- ✅ Error codes AT3020, AT3021, AT3027, AT3028 all produce correct diagnostics
- ✅ All 50+ tests pass in both engines
- ✅ No clippy warnings
- ✅ `cargo nextest run -p atlas-runtime` passes

---

## References

**Specifications:** `docs/specification/types.md` (Pattern Matching section)
**Related phases:** v02-completion-02 (guard clauses + OR patterns — builds on this)
