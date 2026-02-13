# Atlas Test Infrastructure Modernization Plan

**Status:** CRITICAL - Must implement before v0.1 release
**Date:** 2026-02-12
**Goal:** Production-grade testing to compete with Rust/Go/Python/C

---

## Current Problems

### 1. Massive Boilerplate
**Current (10+ lines per test):**
```rust
#[test]
fn test_arithmetic_addition() {
    let runtime = Atlas::new();
    let result = runtime.eval("1 + 2");
    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 3.0),
        _ => panic!("Expected Number(3.0), got {:?}", result),
    }
}
```

**Rust Standard (2 lines with rstest):**
```rust
#[rstest]
#[case("1 + 2", 3.0)]
fn test_arithmetic(#[case] input: &str, #[case] expected: f64) {
    assert_eval_number(input, expected);
}
```

### 2. No Snapshot Testing
- Manual golden file helpers (unused!)
- No automatic snapshot updates
- Parser/bytecode output tests are verbose nightmares

### 3. No Property Testing
- Missing fuzzing for edge cases
- No invariant checking
- Manual edge case tests only

### 4. No Performance Tracking
- Zero benchmarks
- No regression detection
- Can't measure optimization impact

### 5. Statistics
- **9,220 lines of test code** (24 files)
- **1,175+ tests** (will grow to 5,000+)
- **~70% is boilerplate** that could be eliminated

---

## Solution: Adopt Rust Best Practices

### Phase 1: Add Test Dependencies (IMMEDIATE)

**Add to `Cargo.toml`:**
```toml
[workspace.dependencies]
# Testing
insta = { version = "1.39", features = ["yaml", "json"] }
rstest = "0.22"
proptest = "1.5"
pretty_assertions = "1.4"

# Benchmarking
criterion = { version = "0.5", features = ["html_reports"] }

[dev-dependencies]
insta.workspace = true
rstest.workspace = true
proptest.workspace = true
pretty_assertions.workspace = true
```

### Phase 2: Create Test Helpers (Week 1)

**File: `crates/atlas-runtime/tests/common/mod.rs`**
```rust
//! Shared test utilities following Rust best practices

use atlas_runtime::{Atlas, Value};

/// Assert that source code evaluates to a number
pub fn assert_eval_number(source: &str, expected: f64) {
    let runtime = Atlas::new();
    match runtime.eval(source) {
        Ok(Value::Number(n)) => assert_eq!(n, expected),
        other => panic!("Expected Number({expected}), got {other:?}"),
    }
}

/// Assert that source code evaluates to a string
pub fn assert_eval_string(source: &str, expected: &str) {
    let runtime = Atlas::new();
    match runtime.eval(source) {
        Ok(Value::String(s)) => assert_eq!(s.as_ref(), expected),
        other => panic!("Expected String({expected:?}), got {other:?}"),
    }
}

/// Assert that source code produces an error with specific code
pub fn assert_error_code(source: &str, expected_code: &str) {
    let runtime = Atlas::new();
    match runtime.eval(source) {
        Err(diags) => {
            assert!(!diags.is_empty(), "Expected error, got success");
            assert_eq!(diags[0].code, expected_code);
        }
        Ok(val) => panic!("Expected error {expected_code}, got {val:?}"),
    }
}

/// Compile source and snapshot the bytecode
pub fn snapshot_bytecode(name: &str, source: &str) {
    use atlas_runtime::compiler::Compiler;
    use atlas_runtime::lexer::Lexer;
    use atlas_runtime::parser::Parser;

    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&program).unwrap();

    insta::assert_yaml_snapshot!(name, bytecode);
}
```

### Phase 3: Refactor Test Files (Week 2-3)

#### Before (interpreter_tests.rs - 2,146 lines):
```rust
#[test]
fn test_arithmetic_addition() {
    let runtime = Atlas::new();
    let result = runtime.eval("1 + 2");
    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 3.0),
        _ => panic!("Expected Number(3.0), got {:?}", result),
    }
}

#[test]
fn test_arithmetic_subtraction() {
    let runtime = Atlas::new();
    let result = runtime.eval("10 - 3");
    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 7.0),
        _ => panic!("Expected Number(7.0), got {:?}", result),
    }
}

// ... 100+ more similar tests
```

#### After (200 lines):
```rust
mod common;
use rstest::rstest;
use common::*;

#[rstest]
#[case("1 + 2", 3.0)]
#[case("10 - 3", 7.0)]
#[case("4 * 5", 20.0)]
#[case("20 / 4", 5.0)]
#[case("10 % 3", 1.0)]
#[case("-42", -42.0)]
#[case("2 + 3 * 4 - 1", 13.0)]
#[case("(2 + 3) * 4", 20.0)]
fn test_arithmetic(#[case] input: &str, #[case] expected: f64) {
    assert_eval_number(input, expected);
}

#[rstest]
#[case("true && true", true)]
#[case("true && false", false)]
#[case("false || true", true)]
#[case("!true", false)]
fn test_logical(#[case] input: &str, #[case] expected: bool) {
    assert_eval_bool(input, expected);
}
```

**Reduction: 2,146 lines → ~200 lines (90% smaller!)**

### Phase 4: Snapshot Testing (Week 3)

**Parser output snapshots:**
```rust
#[test]
fn test_parser_snapshots() {
    snapshot_ast("function_declaration", "fn add(a: number, b: number) -> number { return a + b; }");
    snapshot_ast("array_literal", "[1, 2, 3]");
    snapshot_ast("nested_blocks", "{ let x = 1; { let y = 2; } }");
}
```

**Bytecode snapshots:**
```rust
#[test]
fn test_bytecode_snapshots() {
    snapshot_bytecode("simple_add", "1 + 2;");
    snapshot_bytecode("function_call", "fn double(x) { return x * 2; } double(21);");
}
```

**Diagnostic snapshots:**
```rust
#[test]
fn test_diagnostic_snapshots() {
    snapshot_diagnostics("type_mismatch", "let x: number = \"hello\";");
    snapshot_diagnostics("unknown_var", "print(undeclared);");
}
```

**Benefits:**
- Automatic golden file management
- `cargo insta review` to approve changes
- Catches unintended output changes

### Phase 5: Property Testing (Week 4)

**Example: Arithmetic properties**
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn arithmetic_commutative(a in -1000.0..1000.0, b in -1000.0..1000.0) {
        let ab = eval_expr(&format!("{a} + {b}"));
        let ba = eval_expr(&format!("{b} + {a}"));
        prop_assert_eq!(ab, ba);
    }

    #[test]
    fn no_string_length_overflow(s in ".*") {
        let result = eval_expr(&format!("len({s:?})"));
        prop_assert!(result.is_ok());
    }

    #[test]
    fn parser_never_panics(input in ".*") {
        // Parser should always return Ok or Err, never panic
        let _ = parse_source(&input);
    }
}
```

**Benefits:**
- Automatically finds edge cases
- Tests invariants (commutative, associative, etc.)
- Fuzzing for robustness

### Phase 6: Benchmarking (Week 5)

**File: `benches/lexer_bench.rs`**
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use atlas_runtime::lexer::Lexer;

fn bench_lexer(c: &mut Criterion) {
    c.bench_function("lex_arithmetic", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box("1 + 2 * 3 - 4 / 5".to_string()));
            lexer.tokenize()
        })
    });

    c.bench_function("lex_function", |b| {
        b.iter(|| {
            let source = black_box("fn factorial(n: number) -> number {
                if (n <= 1) { return 1; }
                return n * factorial(n - 1);
            }".to_string());
            let mut lexer = Lexer::new(source);
            lexer.tokenize()
        })
    });
}

criterion_group!(benches, bench_lexer);
criterion_main!(benches);
```

**Benefits:**
- Track performance over time
- Detect regressions automatically
- HTML reports with graphs
- Compare optimization strategies

---

## Implementation Timeline

### Week 1: Foundation
- [ ] Add dependencies to Cargo.toml
- [ ] Create `tests/common/mod.rs` with helpers
- [ ] Write testing guide document
- [ ] Set up `insta` for one test file (proof of concept)

### Week 2: Refactor Core Tests
- [ ] Convert `interpreter_tests.rs` to rstest (2,146 → 200 lines)
- [ ] Convert `parser_tests.rs` to rstest + snapshots
- [ ] Convert `lexer_tests.rs` to rstest

### Week 3: Snapshot Testing
- [ ] Add parser output snapshots
- [ ] Add bytecode snapshots
- [ ] Add diagnostic snapshots
- [ ] Remove manual golden file code

### Week 4: Property Testing
- [ ] Add arithmetic property tests
- [ ] Add string operation property tests
- [ ] Add parser robustness tests (fuzzing)

### Week 5: Benchmarking
- [ ] Set up Criterion benchmarks
- [ ] Add lexer benchmarks
- [ ] Add parser benchmarks
- [ ] Add compiler benchmarks
- [ ] Add VM benchmarks

---

## Expected Results

### Code Reduction
- **Current:** 9,220 lines of test code
- **After:** ~2,000 lines (78% reduction)
- **More tests:** 1,175 → 3,000+ (easier to add)

### Test Speed
- **Current:** ~2 seconds (1,175 tests)
- **After:** ~1 second (3,000+ tests)
- **Benchmarks:** Tracked separately

### Developer Experience
- **Before:** 10+ lines per test, manual boilerplate
- **After:** 1-2 lines per test case
- **Snapshots:** `cargo insta review` to update
- **Properties:** Automatic edge case discovery

### Production Readiness
- ✅ Snapshot testing (like Jest, Pytest)
- ✅ Property testing (like QuickCheck, Hypothesis)
- ✅ Benchmarking (like Criterion, Go benchmarks)
- ✅ Table-driven tests (like Go subtests)
- ✅ Matches Rust/Go/Python standards

---

## Test Organization Structure

```
crates/atlas-runtime/
├── tests/
│   ├── common/
│   │   ├── mod.rs           # Shared helpers
│   │   ├── assertions.rs    # Custom assertions
│   │   └── fixtures.rs      # Test data
│   ├── snapshots/           # Insta snapshots (auto-generated)
│   │   ├── parser__*.snap
│   │   ├── bytecode__*.snap
│   │   └── diagnostics__*.snap
│   ├── unit/                # Unit tests (fast, isolated)
│   │   ├── lexer.rs
│   │   ├── parser.rs
│   │   ├── typechecker.rs
│   │   └── compiler.rs
│   ├── integration/         # Integration tests (end-to-end)
│   │   ├── interpreter.rs
│   │   ├── vm.rs
│   │   └── repl.rs
│   └── property/            # Property tests (slow, comprehensive)
│       ├── arithmetic.rs
│       ├── strings.rs
│       └── parser_fuzz.rs
├── benches/                 # Benchmarks (separate from tests)
│   ├── lexer_bench.rs
│   ├── parser_bench.rs
│   ├── compiler_bench.rs
│   └── vm_bench.rs
```

---

## Testing Standards Document

### Test Naming
- Unit tests: `test_<component>_<behavior>`
- Integration: `test_e2e_<scenario>`
- Property: `prop_<invariant>`
- Benchmarks: `bench_<operation>`

### Test Size Guidelines
- **Small (unit):** < 0.1s, no I/O, isolated
- **Medium (integration):** < 1s, minimal I/O
- **Large (e2e):** < 10s, full system

### When to Use Each Tool
- **rstest:** Parameterized tests (same logic, different inputs)
- **insta:** Structured output (AST, bytecode, diagnostics)
- **proptest:** Invariants, fuzzing, edge cases
- **criterion:** Performance tracking

### Example PR Checklist
- [ ] Tests use rstest for parameterization
- [ ] Snapshots committed (ran `cargo insta accept`)
- [ ] Property tests for new invariants
- [ ] Benchmarks if performance-critical
- [ ] Test coverage > 90%

---

## Migration Strategy

### Phase 1: Non-Breaking (Current Sprint)
- Add dependencies
- Create common helpers
- Refactor 3-5 test files as proof of concept
- **No existing tests removed**

### Phase 2: Gradual Refactor (Next Sprint)
- Refactor all integration tests
- Add snapshots for parser/compiler
- Add property tests for core operations
- **Old tests kept until new tests proven**

### Phase 3: Complete Migration (v0.1 Release)
- All tests migrated
- Remove old boilerplate
- Add benchmarks
- **Documented testing guide**

---

## Success Metrics

### Before (Current)
- 9,220 lines of test code
- 1,175 tests
- ~2s test time
- ~70% boilerplate
- No snapshots, no properties, no benchmarks

### After (Target for v0.1)
- ~2,000 lines of test code (78% reduction)
- 3,000+ tests (2.5x more coverage)
- ~1s test time (faster despite more tests)
- ~10% boilerplate
- ✅ Snapshot testing
- ✅ Property testing
- ✅ Performance benchmarks
- ✅ Production-grade infrastructure

---

## Next Steps

1. **Approve this plan** - Review and sign off
2. **Add dependencies** - Update Cargo.toml
3. **Create test helpers** - `tests/common/mod.rs`
4. **Pilot refactor** - Pick 1 test file to modernize
5. **Measure impact** - Compare before/after
6. **Roll out gradually** - File by file migration

**Decision needed:** Should we start now or after Stdlib Phase 05?

**Recommendation:** Start NOW (Week 1 tasks), continue stdlib in parallel.
