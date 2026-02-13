# Phase 03: Complete Math API

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Value model must support numbers with IEEE 754 semantics.

**Verification:**
```bash
grep -n "Number(f64)" crates/atlas-runtime/src/value.rs
grep -n "is_nan\|is_infinite" crates/atlas-runtime/src/
```

**What's needed:**
- Value::Number(f64) exists
- NaN/Infinity handled in operations

**If missing:** Should exist from v0.1

---

## Objective
Implement complete math library with 18 functions and 5 constants covering basic operations, trigonometry, exponentials, and utilities with full IEEE 754 compliance.

## Files
**Create:** `crates/atlas-runtime/src/stdlib/math.rs` (~700 lines)
**Update:** `crates/atlas-runtime/src/stdlib/mod.rs` (add math module)
**Update:** `crates/atlas-runtime/src/stdlib/prelude.rs` (register functions + constants)
**Update:** `Cargo.toml` (add rand crate)
**Tests:** `crates/atlas-runtime/tests/stdlib_math_tests.rs` (~500 lines)
**VM Tests:** `crates/atlas-runtime/tests/vm_stdlib_math_tests.rs` (~500 lines)

## Dependencies
- v0.1 complete with Number type
- Atlas-SPEC.md defines numeric semantics
- Rust std::f64 for platform libm delegation
- rand crate for random()

## Implementation

### Basic Operations (6 functions)
Implement abs, floor, ceil, round, min, max. Absolute value handles signed zero and infinities. Rounding functions preserve special values. Round uses ties-to-even banker's rounding. Min/max propagate NaN correctly.

### Exponential/Power (3 functions)
Implement sqrt, pow, log. Square root handles negative inputs. Power function has special cases for zero exponents and infinite bases. Natural logarithm handles domain edge cases.

### Trigonometry (6 functions)
Implement sin, cos, tan, asin, acos, atan. All use radians. Inverse functions validate domain restrictions. All propagate special values per IEEE 754.

### Utilities (3 functions)
Implement clamp, sign, random. Clamp restricts values to range with validation. Sign returns -1/0/1 preserving signed zero. Random generates uniform distribution in [0, 1) using thread_rng.

### Constants
Expose PI, E, SQRT2, LN2, LN10 as global values in prelude.

### Architecture Notes
Delegate all operations to Rust std::f64 methods which use platform libm. Follow IEEE 754 exactly - NaN propagates, infinities handled, signed zero preserved, domain errors return NaN not panic.

## Tests (TDD - Use rstest)

**Math tests cover:**
1. Basic functionality for each function
2. Special values - NaN, infinity, signed zero
3. Domain edges - negative sqrt, out of range asin
4. Rounding behavior - ties to even
5. Random distribution - multiple calls differ, in range
6. Constants accuracy
7. VM parity

**Minimum test count:** 120 tests (60 interpreter, 60 VM)

## Integration Points
- Uses: Value::Number(f64)
- Uses: RuntimeError::ValueError for domain errors
- Updates: Cargo.toml with rand = "0.8"
- Updates: prelude.rs with 18 functions + 5 constants
- Updates: docs/stdlib.md
- Output: Complete math library

## Acceptance
- All 18 functions + 5 constants implemented
- IEEE 754 compliance verified
- Ties-to-even rounding in round()
- Domain errors return NaN
- Random uniform in [0, 1)
- 120+ tests pass
- Interpreter/VM parity verified
- math.rs under 800 lines
- Test files under 600 lines each
- Documentation updated
- No clippy warnings
- cargo test passes
