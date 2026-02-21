# Phase v02-completion-03: Stdlib Core Hardening (string, array, math, json, types)

## Dependencies

**Required:** v0.2 main branch clean

**Verification:**
```bash
cargo nextest run -p atlas-runtime 2>&1 | tail -3   # must show 0 failures
ls crates/atlas-runtime/src/stdlib/string.rs        # must exist
ls crates/atlas-runtime/src/stdlib/array.rs         # must exist
ls crates/atlas-runtime/src/stdlib/math.rs          # must exist
ls crates/atlas-runtime/src/stdlib/json.rs          # must exist
ls crates/atlas-runtime/src/stdlib/types.rs         # must exist
```

---

## Objective

The v0.2 post-mortem identified that 15-20% of stdlib functions have shallow implementations that may not handle edge cases correctly. This phase audits and hardens the five most-used stdlib modules — `string`, `array`, `math`, `json`, `types` — by systematically testing every documented function with edge case inputs and fixing any incorrect behavior found.

---

## Files

**Update (fixes only — no new functions):**
- `crates/atlas-runtime/src/stdlib/string.rs` (~424 lines) — fix edge cases found
- `crates/atlas-runtime/src/stdlib/array.rs` (~330 lines) — fix edge cases found
- `crates/atlas-runtime/src/stdlib/math.rs` (~483 lines) — fix edge cases found
- `crates/atlas-runtime/src/stdlib/json.rs` (~640 lines) — fix edge cases found
- `crates/atlas-runtime/src/stdlib/types.rs` (~844 lines) — fix edge cases found

**Create:**
- `crates/atlas-runtime/tests/stdlib/mod.rs` — test module entry (if not exists)
- `crates/atlas-runtime/tests/stdlib/string_hardening.rs` — ~200 lines, 35+ tests
- `crates/atlas-runtime/tests/stdlib/array_hardening.rs` — ~150 lines, 25+ tests
- `crates/atlas-runtime/tests/stdlib/math_hardening.rs` — ~150 lines, 25+ tests
- `crates/atlas-runtime/tests/stdlib/json_hardening.rs` — ~120 lines, 20+ tests

**Total new code:** ~650 lines tests + fixes as needed
**Minimum test count:** 60 tests across all modules

---

## Implementation Notes

**Audit process (REQUIRED before writing tests):**
For each module:
1. Read every public function in the source file
2. Identify the documented behavior from `docs/specification/stdlib.md` or inline docs
3. Write test for: happy path, empty input, boundary values, wrong type arg (if applicable), very large input

**String functions to audit (read `string.rs` first):**
Focus edge cases on: `split` (empty separator, empty string), `replace` (replace-all vs first), `pad_start`/`pad_end` (pad wider than string), `trim` variants (string of all whitespace), `substring` (negative index, out of bounds), `char_at` (out of bounds), `repeat` (repeat 0 times), `index_of` (not found returns), `starts_with`/`ends_with` (empty needle), `to_upper`/`to_lower` (already correct case), `format` function (missing args, extra args)

**Array functions to audit (read `array.rs` first):**
Focus edge cases on: `push`/`pop` on empty array, `slice` (negative start, end > length, start > end), `splice` (at index 0, at last index, delete more than exist), `filter`/`map`/`reduce` (on empty array), `find`/`find_index` (not found), `sort` (already sorted, single element, empty, mixed types), `reverse` (empty, single), `concat` (empty arrays), `flatten` (nested empty arrays), `unique` (all duplicates, no duplicates), `zip` (different lengths)

**Math functions to audit (read `math.rs` first):**
Focus edge cases on: `sqrt` (negative number), `pow` (0^0, negative exponent), `log`/`log2`/`log10` (0, negative), `floor`/`ceil`/`round` (negative numbers, already integer), `min`/`max` (single element, equal elements), `abs` (negative, positive, zero), `clamp` (value below min, above max, equal to bounds), trig functions (values that produce NaN or Infinity → should be runtime error AT0007)

**JSON functions to audit (read `json.rs` first):**
Focus edge cases on: `json_parse` (malformed JSON, empty string, just `null`, deeply nested), `json_stringify` (circular reference detection, null values, arrays with nulls), `json_get` on missing key (should return null, not error), nested `json_get` on non-object (should return null), `json_set` (set on non-object), `json_keys`/`json_values` on non-object (error vs empty)

**Types functions to audit (read `types.rs` first):**
Focus edge cases on: `type_of` for all Value variants (including Option, Result, Array, Object, Function, Builtin), `is_number`/`is_string`/`is_bool`/`is_null`/`is_array`/`is_object`/`is_function` — verify they return false for all non-matching types, `to_number` (string that's not a number, bool, null, array), `to_string` (all types), `to_bool` (all types — Atlas has no truthiness but `to_bool` should be explicit)

**Critical requirements:**
- Every fix must be accompanied by a test that fails before the fix and passes after
- Do NOT add new functions — hardening only
- Do NOT change documented behavior — only fix incorrect implementations
- If a function is "works for common case" and fixing edge cases would change its signature, document the limitation instead and add a regression test for the common case

**Error handling:**
- Math operations producing NaN/Infinity → RuntimeError with AT0007
- Out-of-bounds access → RuntimeError with appropriate code
- Type mismatch in stdlib call → RuntimeError with AT-prefix code

---

## Tests (TDD Approach)

**String hardening:** (35 tests)
- 3 tests per function × ~12 functions = ~36 tests covering edge cases

**Array hardening:** (25 tests)
- 2-3 tests per function × ~10 functions = ~25 tests

**Math hardening:** (25 tests)
- 2-3 tests per function × ~10 functions = ~25 tests (including error cases for invalid inputs)

**JSON hardening:** (20 tests)
- 2-3 tests per function × ~7 functions = ~20 tests

**Minimum test count:** 60 tests (may be higher — write as many as needed to cover all edge cases)

**Parity requirement:** All tests run in both interpreter and VM with identical results.

**Test approach:**
- Use `rstest` `#[case(...)]` matrices for input/output pairs
- Use `eval_ok` / `vm_eval_ok` helpers to run in both engines
- Test error cases with `eval_err` / `vm_eval_err` — verify error code matches expected

---

## Acceptance Criteria

- ✅ Every documented function in string, array, math, json, types modules tested with edge cases
- ✅ All discovered bugs fixed (documented in commit message with: function name, bug description, fix)
- ✅ 60+ new tests created across the 5 modules
- ✅ All tests pass in both interpreter and VM
- ✅ No behavior changes to documented correct behavior (only bug fixes)
- ✅ No clippy warnings
- ✅ `cargo nextest run -p atlas-runtime` passes

---

## References

**Specifications:** `docs/specification/stdlib.md`
**Related phases:** v02-completion-04 (extended stdlib modules — builds on same methodology)
