# Phase 01: Complete String API

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Value model must support string operations.

**Verification:**
```bash
grep -n "String(String)" crates/atlas-runtime/src/value.rs
grep -n "fn.*String" crates/atlas-runtime/src/stdlib/prelude.rs
```

**What's needed:**
- Value::String variant exists
- Basic prelude structure in place

**If missing:** Block and report issue - should exist from v0.1

---

## Objective
Implement complete string manipulation API with 18 functions covering core operations, search, transformation, and formatting.

## Files
**Create:** `crates/atlas-runtime/src/stdlib/string.rs` (~800 lines)
**Update:** `crates/atlas-runtime/src/stdlib/mod.rs` (add string module)
**Update:** `crates/atlas-runtime/src/stdlib/prelude.rs` (register functions)
**Tests:** `crates/atlas-runtime/tests/stdlib_string_tests.rs` (~600 lines)
**VM Tests:** `crates/atlas-runtime/tests/vm_stdlib_string_tests.rs` (~600 lines)

## Dependencies
- v0.1 complete (Value model, interpreter, VM, prelude system)
- docs/stdlib.md exists for updates
- Atlas-SPEC.md defines string semantics

## Implementation

### Core Operations (5 functions)
Implement split, join, trim, trimStart, trimEnd. Split divides strings by separator handling empty cases. Join combines arrays with separator. Trim functions remove whitespace Unicode-aware.

### Search Operations (3 functions)
Implement indexOf, lastIndexOf, includes. Find occurrences in strings returning indices or boolean. Handle empty search strings and not-found cases appropriately.

### Transformation (6 functions)
Implement toUpperCase, toLowerCase, substring, charAt, repeat, replace. Case conversion must be Unicode-aware. Substring validates UTF-8 boundaries. charAt returns grapheme clusters not bytes. Repeat limits count to prevent memory abuse. Replace handles first occurrence only in v0.2.

### Formatting (4 functions)
Implement padStart, padEnd, startsWith, endsWith. Padding fills to specified length. Prefix/suffix checking returns boolean. Handle edge cases like empty strings and multi-character fills.

### Architecture Notes
All functions take string references and return owned strings or Results. Prelude wrappers extract Value::String and convert results back. Use RuntimeError variants for different error types. Unicode handling uses Rust's standard methods respecting UTF-8.

### Prelude Registration
Register all 18 functions in prelude HashMap. Each gets wrapper function checking arity and types. Wrappers convert between Value and Rust types.

## Tests (TDD - Use rstest)

**String tests cover:**
1. Each function with basic inputs
2. Edge cases - empty strings, boundaries, invalid inputs
3. Unicode handling - multi-byte characters, emoji
4. Error conditions - wrong types, out of bounds
5. VM parity - identical results in both engines

**Use parameterized tests** for multiple input scenarios per function.

**Minimum test count:** 120 tests (60 interpreter, 60 VM)

## Integration Points
- Uses: Value enum from value.rs
- Uses: RuntimeError from error.rs
- Updates: prelude.rs with 18 function registrations
- Updates: docs/stdlib.md with API documentation
- Output: Complete string API available in Atlas programs

## Acceptance
- All 18 functions implemented and working
- Unicode handled correctly in all operations
- UTF-8 boundary validation in substring
- All edge cases handled with appropriate errors
- 120+ tests pass
- Interpreter/VM parity verified
- string.rs under 900 lines
- Test files under 700 lines each
- Documentation updated
- No clippy warnings
- cargo test passes
