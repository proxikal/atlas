# Phase 06a: Standard Library Integration Tests - Core

## 🚨 DEPENDENCIES - CHECK BEFORE STARTING

**REQUIRED:** All previous stdlib phases must be complete.

**Verification:**
```bash
# Verify all stdlib modules exist
ls crates/atlas-runtime/src/stdlib/string.rs
ls crates/atlas-runtime/src/stdlib/array.rs
ls crates/atlas-runtime/src/stdlib/math.rs
ls crates/atlas-runtime/src/stdlib/json.rs
ls crates/atlas-runtime/src/stdlib/types.rs
ls crates/atlas-runtime/src/stdlib/io.rs

# Check prelude has all functions registered
grep -c "prelude.insert" crates/atlas-runtime/src/stdlib/prelude.rs

# Verify existing unit tests pass
cargo test -p atlas-runtime string -- --exact --nocapture
cargo test -p atlas-runtime array -- --exact --nocapture
cargo test -p atlas-runtime math -- --exact --nocapture
cargo test -p atlas-runtime json -- --exact --nocapture
cargo test -p atlas-runtime file -- --exact --nocapture
```

**What's needed:**
- Phases 01-05 complete with all 84+ functions
- All unit tests passing
- Interpreter/VM parity verified per function

**If missing:** Complete previous phases first

---

## Objective

Create comprehensive cross-module integration tests for the standard library. Verify that functions from different modules work together correctly, test common usage patterns that combine multiple stdlib functions, and establish the foundation for real-world usage testing.

## Files

**Create:** `crates/atlas-runtime/tests/stdlib_integration_tests.rs` (~300 lines)

## Dependencies

- All stdlib phases 01-05 complete
- All unit tests passing
- rstest for parameterized tests
- Both interpreter and VM for parity testing

## Implementation

### 1. Test Infrastructure Setup

Create `tests/stdlib_integration_tests.rs` with:
- Helper functions for running Atlas code snippets in both engines
- Parity assertion helpers (assert both engines produce identical results)
- Test fixtures for common data structures
- Cleanup utilities for temporary files

### 2. String + Array Integration Tests

Test combinations like:
- `split()` → `map()` → `join()` pipelines
- `split()` → `filter()` → `length()` patterns
- Array of strings with `map(toUpper)`, `map(toLower)`
- `concat()` with `map()` transformations
- String manipulation in array context

**Example test scenarios:**
```atlas
// CSV-like processing
let line = "apple,banana,cherry"
let parts = split(line, ",")
let upper = map(parts, toUpper)
let result = join(upper, "|")
// result should be "APPLE|BANANA|CHERRY"

// Word filtering
let text = "the quick brown fox"
let words = split(text, " ")
let long = filter(words, fn(w) { length(w) > 3 })
// long should be ["quick", "brown"]
```

### 3. Array + Math Integration Tests

Test combinations like:
- `map()` with math functions (`abs`, `sqrt`, `round`)
- `reduce()` for sum/product calculations
- `filter()` with numeric predicates
- `sort()` with custom numeric comparisons
- Statistical operations on arrays (min, max, average)

**Example test scenarios:**
```atlas
// Calculate average
let numbers = [1, 2, 3, 4, 5]
let sum = reduce(numbers, fn(a, b) { a + b }, 0)
let avg = sum / length(numbers)
// avg should be 3

// Round all to nearest integer
let floats = [1.2, 2.7, 3.5, 4.1]
let rounded = map(floats, round)
// rounded should be [1, 3, 4, 4]
```

### 4. JSON + Type Integration Tests

Test combinations like:
- JSON parsing → type checking → property access
- Object construction → JSON serialization
- Array of objects → map/filter → JSON output
- Type validation before JSON operations
- Error handling for invalid JSON

**Example test scenarios:**
```atlas
// Parse and validate JSON
let json = parseJson('{"name":"Alice","age":30}')
if isObject(json) {
    let name = jsonGet(json, "name")
    let age = jsonGet(json, "age")
    // Verify name is string, age is number
}

// Build and serialize
let obj = jsonObject()
jsonSet(obj, "items", [1, 2, 3])
let str = jsonStringify(obj)
// str should be valid JSON
```

### 5. File + JSON Integration Tests

Test combinations like:
- Read file → parse JSON → process → write back
- File reading with string operations
- Temporary file workflows
- Error handling for missing files + JSON errors
- File I/O with array processing

**Example test scenarios:**
```atlas
// Read JSON file
let content = readFile("data.json")
let data = parseJson(content)
let processed = map(data, fn(item) { /* transform */ })
let output = jsonStringify(processed)
writeFile("output.json", output)
```

### 6. Cross-Function Parity Verification

For each integration test:
- Run identical code in interpreter
- Run identical code in VM
- Assert outputs are exactly equal
- Test edge cases in both engines
- Verify errors match in both engines

## Tests (TDD - Use rstest)

**Test categories (minimum 120 tests):**
1. String + Array combinations (30 tests)
   - split → map → join pipelines
   - split → filter → length patterns
   - String array transformations
   - concat with map operations
   - Edge cases (empty, single, special chars)

2. Array + Math combinations (30 tests)
   - map with math functions
   - reduce for calculations
   - filter with numeric predicates
   - sort with comparisons
   - Statistical operations

3. JSON + Type combinations (30 tests)
   - Parse → typecheck → access
   - Build → serialize workflows
   - Array of objects processing
   - Type validation patterns
   - Error handling

4. File + JSON combinations (20 tests)
   - Read → parse → process → write
   - File I/O with string ops
   - Temporary file workflows
   - Error handling
   - Round-trip serialization

5. Multi-step transformations (10 tests)
   - 3+ function chains
   - Complex data pipelines
   - Mixed module operations
   - Error propagation
   - Edge case handling

**All tests MUST verify interpreter/VM parity**

## Integration Points

- Uses: `stdlib/{string,array,math,json,types,io}.rs`
- Uses: Both interpreter and VM for parity
- Uses: `rstest` for parameterized tests
- Uses: `tempfile` for I/O tests
- Pattern: Integration tests in `tests/` directory
- Output: Foundation for phase 06b real-world tests

## Acceptance Criteria

- [ ] All 120+ integration tests pass
- [ ] Zero parity violations (100% identical output in both engines)
- [ ] All test categories implemented
- [ ] String + Array integration verified (30 tests)
- [ ] Array + Math integration verified (30 tests)
- [ ] JSON + Type integration verified (30 tests)
- [ ] File + JSON integration verified (20 tests)
- [ ] Multi-step transformations verified (10 tests)
- [ ] cargo test passes
- [ ] No clippy warnings
- [ ] Interpreter/VM parity: 100%

## Notes

**Testing protocol:**
- Write test function
- Run ONLY that test: `cargo test -p atlas-runtime test_function_name -- --exact`
- Iterate until passing
- Move to next test
- NEVER run full suite during development

**Parity verification:**
- Every test runs code in BOTH engines
- Assert outputs are identical
- No exceptions - 100% parity required

**This phase establishes the foundation for 06b (real-world patterns) and 06c (benchmarks + docs).**
