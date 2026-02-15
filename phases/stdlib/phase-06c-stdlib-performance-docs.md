# Phase 06c: Standard Library Performance & Documentation

## 🚨 DEPENDENCIES - CHECK BEFORE STARTING

**REQUIRED:** Phases 06a and 06b must be complete.

**Verification:**
```bash
# Verify phase 06a and 06b test files exist and pass
ls crates/atlas-runtime/tests/stdlib_integration_tests.rs
ls crates/atlas-runtime/tests/stdlib_real_world_tests.rs
cargo test -p atlas-runtime stdlib_integration_tests -- --nocapture
cargo test -p atlas-runtime stdlib_real_world_tests -- --nocapture

# Verify all stdlib modules still working
cargo test -p atlas-runtime string -- --exact --nocapture
cargo test -p atlas-runtime array -- --exact --nocapture
```

**What's needed:**
- Phases 06a and 06b complete with all tests passing
- All unit tests still passing
- Interpreter/VM parity verified

**If missing:** Complete phases 06a and 06b first

---

## Objective

Establish performance baselines for stdlib operations, create comprehensive benchmarks to detect regressions, perform systematic parity verification for all 84+ functions, complete the API reference documentation, and create a usage guide. This phase completes the stdlib integration testing and documentation.

## Files

**Create:** `crates/atlas-runtime/benches/stdlib_benchmarks.rs` (~400 lines)
**Update:** `crates/atlas-runtime/Cargo.toml` (add criterion dev-dependency)
**Update:** `docs/api/stdlib.md` (complete API reference with all functions)
**Create:** `docs/guides/stdlib-usage-guide.md` (~500 lines)
**Create:** `crates/atlas-runtime/tests/stdlib_parity_verification.rs` (~500 lines)

## Dependencies

- Phases 06a and 06b complete
- Criterion for benchmarking
- All stdlib modules functional
- Both interpreter and VM for systematic parity testing

## Implementation

### 1. Add Criterion Dependency

Update `Cargo.toml`:
```toml
[dev-dependencies]
criterion = "0.5"

[[bench]]
name = "stdlib_benchmarks"
harness = false
```

### 2. Performance Benchmarks

Create benchmarks for key operations in `benches/stdlib_benchmarks.rs`:

**String operations:**
- `split()` with 1000 elements
- `join()` with 1000 elements
- `replace()` on large strings
- `trim()`, `toUpper()`, `toLower()` on various sizes
- Substring operations

**Array operations:**
- `map()` on 10K elements
- `filter()` on 10K elements
- `reduce()` on 10K elements
- `sort()` on 1000 numbers
- `concat()` large arrays
- `indexOf()`, `contains()` searches

**Math operations:**
- Basic arithmetic (10K operations)
- `sqrt()`, `pow()`, `abs()` (10K calls)
- Trigonometric functions (10K calls)

**JSON operations:**
- Parse large JSON documents (1KB, 10KB, 100KB)
- Stringify large objects
- Nested object access (100 deep)
- Array serialization (1000 elements)

**File I/O:**
- Read 1MB file
- Write 1MB file
- Multiple small reads (100 files)
- Multiple small writes (100 files)

**Benchmark targets:**
- String split/join: < 100μs for 1000 elements
- Array map/filter: < 1ms for 10K elements
- Sort: < 500μs for 1000 numbers
- JSON parse: < 500μs for 10KB
- File read 1MB: < 5ms

**Use criterion groups for organization:**
```rust
use criterion::{criterion_group, criterion_main, Criterion};

fn string_benchmarks(c: &mut Criterion) {
    c.bench_function("split_1000_elements", |b| {
        b.iter(|| {
            // Run Atlas code: split(string, ",")
        });
    });
}

criterion_group!(benches, string_benchmarks, array_benchmarks, ...);
criterion_main!(benches);
```

### 3. Systematic Parity Verification

Create `tests/stdlib_parity_verification.rs` to systematically test EVERY stdlib function in both engines:

**Test structure:**
```rust
#[rstest]
#[case::string_length("length", "hello", vec!["5"])]
#[case::string_upper("toUpper", "hello", vec!["HELLO"])]
// ... all 84+ functions
fn test_stdlib_function_parity(
    #[case] function_name: &str,
    #[case] atlas_code: &str,
    #[case] expected: Vec<&str>,
) {
    // Run in interpreter
    let interp_result = run_interpreter(atlas_code);

    // Run in VM
    let vm_result = run_vm(atlas_code);

    // Assert identical
    assert_eq!(interp_result, vm_result);
    assert_eq!(interp_result, expected);
}
```

**Coverage:**
- All 18 string functions
- All 21 array functions
- All 18 math functions + 5 constants
- All 17 JSON functions
- All 10 file I/O functions
- All type checking functions
- Edge cases for each function
- Error cases for each function

**Minimum 130 parity tests (one per function + edge cases)**

### 4. Complete API Reference

Update `docs/api/stdlib.md` with complete reference for all 84+ functions:

**For each function, include:**
- Function signature with types
- Description (1-2 sentences)
- Parameters with descriptions
- Return value description
- Example usage (Atlas code)
- Edge cases and special behavior
- Error conditions
- Performance characteristics (if notable)
- Related functions

**Example format:**
```markdown
### `split(string, separator) -> [string]`

Splits a string into an array of substrings using the given separator.

**Parameters:**
- `string`: The string to split
- `separator`: The separator string (can be multi-character)

**Returns:** Array of strings (empty array if input is empty)

**Examples:**
```atlas
split("a,b,c", ",")  // ["a", "b", "c"]
split("hello", "")   // ["h", "e", "l", "l", "o"]
split("", ",")       // []
```

**Edge cases:**
- Empty string returns empty array
- Empty separator splits into individual characters
- Separator not found returns array with original string

**Errors:** None (always succeeds)

**Performance:** O(n) where n is string length

**See also:** `join`, `trim`, `replace`
```

### 5. Usage Guide

Create `docs/guides/stdlib-usage-guide.md` with:

**Sections:**
1. **Introduction** - Overview of stdlib organization
2. **Common Patterns** - Frequently used function combinations
3. **String Processing** - Working with text
4. **Array Operations** - Data manipulation and transformation
5. **Mathematical Computations** - Numeric processing
6. **JSON Handling** - Structured data
7. **File I/O** - Reading and writing files
8. **Type Checking** - Runtime type validation
9. **Error Handling** - Best practices with Result types
10. **Performance Tips** - Efficient stdlib usage
11. **Integration Examples** - Real-world programs
12. **Migration Guide** - From other languages

**Example pattern documentation:**
```markdown
## Common Pattern: CSV Processing

Reading, parsing, and processing CSV files:

```atlas
// Read and parse
let csv = readFile("data.csv")
let lines = split(csv, "\n")
let header = first(lines)
let rows = rest(lines)

// Process each row
let data = map(rows, fn(row) {
    let fields = split(row, ",")
    // Transform into structured data
    fields
})

// Filter and aggregate
let filtered = filter(data, fn(row) { /* criteria */ })
let total = reduce(filtered, fn(sum, row) { sum + row }, 0)
```

**See:** Real-world examples in `tests/stdlib_real_world_tests.rs`
```

## Tests (TDD - Use rstest)

**Benchmarks (15 benchmarks):**
- String operations (3 benchmarks)
- Array operations (5 benchmarks)
- Math operations (2 benchmarks)
- JSON operations (3 benchmarks)
- File I/O (2 benchmarks)

**Parity verification (130+ tests):**
- All 84+ functions tested in both engines
- Edge cases for each function
- Error cases for each function
- Systematic coverage

**Documentation verification:**
- All functions documented in API reference
- All common patterns in usage guide
- All examples tested and verified

## Integration Points

- Uses: All stdlib modules from phases 01-05
- Uses: Phase 06a and 06b test infrastructure
- Uses: Criterion for benchmarking
- Uses: Both interpreter and VM for parity
- Creates: Complete stdlib documentation
- Output: Production-ready stdlib with verified performance and complete docs

## Acceptance Criteria

- [ ] All 15 benchmarks implemented and running
- [ ] All benchmarks meet performance targets
- [ ] All 130+ parity tests pass with 100% identical output
- [ ] Every stdlib function verified in both engines
- [ ] Complete API reference in `docs/api/stdlib.md`
- [ ] Usage guide created with all sections
- [ ] All examples in docs tested and verified
- [ ] cargo bench runs successfully
- [ ] cargo test passes
- [ ] No clippy warnings
- [ ] Interpreter/VM parity: 100%
- [ ] Code coverage for stdlib >90%

## Commands

**Run benchmarks:**
```bash
cargo bench --bench stdlib_benchmarks
```

**Run parity verification:**
```bash
cargo test -p atlas-runtime stdlib_parity_verification -- --nocapture
```

**Run all stdlib tests:**
```bash
cargo test -p atlas-runtime stdlib -- --nocapture
```

## Notes

**Testing protocol:**
- Write benchmark or test function
- Run ONLY that test: `cargo test -p atlas-runtime test_function_name -- --exact`
- For benchmarks: `cargo bench --bench stdlib_benchmarks -- benchmark_name`
- Iterate until passing/meeting targets
- Move to next test
- NEVER run full suite during development

**Performance focus:**
- Establish baselines, not optimizations
- Detect regressions in future
- Identify hot paths for later optimization
- Document expected performance

**Documentation focus:**
- Complete reference for all functions
- Practical examples
- Real-world patterns
- Migration guidance

**This phase completes stdlib integration testing and establishes stdlib as production-ready with verified performance and complete documentation.**
