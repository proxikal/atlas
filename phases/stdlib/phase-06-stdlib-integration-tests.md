# Phase 06: Standard Library Integration Tests

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** All previous stdlib phases must be complete.

**Verification:**
```bash
ls crates/atlas-runtime/src/stdlib/{string,array,math,json,types,io}.rs
grep -c "prelude.insert" crates/atlas-runtime/src/stdlib/prelude.rs
cargo test stdlib
```

**What's needed:**
- Phases 01-05 complete with all 60+ functions
- All unit tests passing
- Interpreter/VM parity verified per function

**If missing:** Complete previous phases first

---

## Objective
Comprehensive integration testing of complete standard library - verify all 60+ functions work together correctly, test real-world usage patterns, establish performance baselines, and ensure zero parity violations.

## Files
**Create:** `crates/atlas-runtime/tests/stdlib_integration_tests.rs` (~800 lines)
**Create:** `crates/atlas-runtime/tests/stdlib_real_world_tests.rs` (~600 lines)
**Create:** `crates/atlas-runtime/benches/stdlib_benchmarks.rs` (~400 lines)
**Update:** `Cargo.toml` (add criterion for benchmarks)
**Update:** `docs/stdlib.md` (complete API reference)
**Create:** `docs/stdlib-usage-guide.md` (~500 lines)

## Dependencies
- All stdlib phases 01-05 complete
- All unit tests passing
- Criterion for benchmarking

## Implementation

### Cross-Function Integration Tests
Test combinations of stdlib functions working together. String+Array pipelines. Array+Math aggregations. JSON+Type conversions. File+JSON data processing. Complex multi-step transformations verifying interoperability.

### Real-World Usage Patterns
Implement realistic Atlas programs using stdlib. CSV processing with split/map/filter. JSON API response handling. Log file analysis. Data transformation pipelines. Plugin systems with sandboxing. Expression evaluators with context.

### Performance Benchmarks
Establish baseline performance for key operations. String split/join with 1000 elements. Array map/filter on 10K elements. Sort 1000 numbers. JSON parse large documents. File read 1MB. Set targets and detect regressions.

### Parity Verification
Systematically verify every function produces identical results in both interpreter and VM. Test all edge cases in both engines. No parity violations tolerated.

### Documentation
Complete API reference with signatures, descriptions, examples, edge cases, errors for all 60+ functions. Create usage guide with common patterns, best practices, performance tips, integration examples.

## Tests (TDD - Use rstest)

**Integration test categories:**
1. String+Array combinations
2. Array+Math pipelines
3. JSON+Type conversions
4. File+JSON workflows
5. Complex multi-step scenarios
6. CSV/log/JSON real-world programs
7. Performance benchmarks
8. Systematic parity checks for all functions

**Minimum test count:** 400+ tests plus 15 benchmarks

## Integration Points
- Uses: All stdlib modules from phases 01-05
- Uses: Both interpreter and VM for parity
- Uses: Criterion for benchmarks
- Uses: Tempfile for I/O tests
- Updates: docs/stdlib.md with complete API
- Creates: Usage guide with practical examples
- Output: Verified stdlib ready for production

## Acceptance
- All 400+ integration tests pass
- Real-world programs run correctly
- Zero parity violations across all 60+ functions
- All performance benchmarks meet targets
- Complete API documentation
- Usage guide with examples
- Benchmarks run: cargo bench
- Code coverage >90% for stdlib
- Final stdlib count: 60+ functions working
- All tests pass: cargo test
- No clippy warnings
