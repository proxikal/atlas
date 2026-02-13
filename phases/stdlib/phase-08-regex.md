# Phase 08: Regular Expressions

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** String stdlib and Result types must exist.

**Verification:**
```bash
ls crates/atlas-runtime/src/stdlib/string.rs
ls crates/atlas-runtime/src/result_type.rs
cargo test stdlib_string
```

**What's needed:**
- String stdlib from stdlib/phase-01
- Result types from foundation/phase-09
- Regex engine (use regex crate)

**If missing:** Complete stdlib/phase-01 and foundation/phase-09 first

---

## Objective
Implement regular expression support with pattern matching, searching, and replacement - providing powerful text processing capabilities essential for any modern programming language.

## Files
**Create:** `crates/atlas-runtime/src/stdlib/regex.rs` (~700 lines)
**Update:** `crates/atlas-runtime/src/value.rs` (~100 lines Regex value)
**Update:** `Cargo.toml` (add regex dependency)
**Update:** `docs/stdlib.md` (~300 lines regex docs)
**Tests:** `crates/atlas-runtime/tests/regex_tests.rs` (~600 lines)

## Dependencies
- regex crate for engine
- String stdlib
- Result types for errors
- Array for match results

## Implementation

### Regex Construction
Create regex pattern from string. Compile pattern validating syntax. Cache compiled patterns. Return Result with compile errors. Support regex flags case-insensitive, multi-line, dot-all. Escape special characters function. Pre-compile common patterns. Pattern validation before compilation.

### Pattern Matching
Test string against pattern. is_match function returns boolean. find function returns first match with position and text. find_all function returns all matches. Capture groups extract substrings. Named capture groups. Match object with groups and positions. Lazy evaluation for performance.

### Search and Replace
Replace matched text with replacement string. replace_first replaces first match. replace_all replaces all matches. Replacement with capture group references. Replacement with function callback. Regex-based splitting strings. Match-based filtering.

### Match Results
Match object contains match details. Matched text accessor. Match start and end positions. Capture group access by index or name. Iteration over capture groups. Multiple match results as array. Efficient representation avoiding copies.

## Tests (TDD - Use rstest)
1. Compile valid pattern
2. Compile error invalid pattern
3. is_match returns true on match
4. is_match returns false no match
5. find returns first match
6. find_all returns all matches
7. Capture groups extraction
8. Named capture groups
9. replace_first single replacement
10. replace_all multiple replacements
11. Case-insensitive flag
12. Multi-line flag
13. Escape special characters
14. Complex pattern matching
15. Unicode handling

**Minimum test count:** 50 tests

## Acceptance
- Regex patterns compile
- Pattern matching works
- Search and replace functional
- Capture groups extract
- Flags modify behavior
- 50+ tests pass
- Documentation complete
- cargo test passes
