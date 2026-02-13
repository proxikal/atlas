# Phase 15: Built-in Testing Framework

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Reflection API and assertions must be possible.

**Verification:**
```bash
ls crates/atlas-runtime/src/reflect/mod.rs
cargo test reflect
grep -n "assert" crates/atlas-runtime/src/stdlib/
```

**What's needed:**
- Reflection API from foundation/phase-12
- Module system from foundation/phase-06
- Result types from foundation/phase-09
- Function discovery capabilities

**If missing:** Complete foundation phases 06, 09, and 12 first

---

## Objective
Implement built-in testing framework with test discovery, assertions, mocking, and reporting - providing first-class testing support like Rust's built-in tests or Jest enabling comprehensive test-driven development in Atlas.

## Files
**Create:** `crates/atlas-runtime/src/testing/mod.rs` (~800 lines)
**Create:** `crates/atlas-runtime/src/testing/runner.rs` (~600 lines)
**Create:** `crates/atlas-runtime/src/testing/assertions.rs` (~500 lines)
**Create:** `crates/atlas-runtime/src/testing/mocking.rs` (~400 lines)
**Create:** `crates/atlas-runtime/src/stdlib/test.rs` (~600 lines)
**Update:** `crates/atlas-cli/src/commands/test.rs` (~400 lines integration)
**Create:** `docs/testing-framework.md` (~800 lines)
**Tests:** `crates/atlas-runtime/tests/testing_framework_tests.rs` (~700 lines)

## Dependencies
- Reflection API for test discovery
- Module system for test organization
- Result types for test outcomes
- CLI integration for test runner
- Function metadata

## Implementation

### Test Declaration and Discovery
Define tests with test keyword or annotation. Test functions take no parameters. Test discovery scans modules for test functions. Name-based discovery with test prefix. Attribute-based discovery with test annotation. Group tests in modules. Skip tests with skip annotation. Focus tests with only annotation. Test metadata tags, descriptions.

### Assertion Library
Comprehensive assertion functions. assert_eq for equality with nice diff. assert_ne for inequality. assert_true and assert_false for booleans. assert_ok and assert_err for Results. assert_some and assert_none for nullables. assert_contains for collections. assert_throws for exceptions. Custom assertions with predicates. Assertion failure messages with context.

### Test Runner
Execute discovered tests. Run tests in isolation. Parallel test execution option. Sequential execution for order-dependent. Filter tests by name or tag. Run only failed tests. Timeout per test. Setup and teardown hooks. Before and after each test. Module-level setup and teardown. Shared test fixtures.

### Test Output and Reporting
Progress indicator during test run. Pass/fail status for each test. Failure details with assertion messages. Stack traces for failures. Summary statistics total, passed, failed, skipped. Execution time per test and total. Colorized output for terminal. JSON output for CI integration. JUnit XML format option. Code coverage reporting (future).

### Mocking and Stubbing
Mock objects for testing. Mock function creation. Stub return values. Verify function calls. Call count assertions. Argument matching. Spy on function calls. Restore original functions. Mock modules and dependencies. Automatic cleanup after tests.

### Test Organization
Test modules separate from source. Test files with _test suffix. Unit tests inline with source. Integration tests in tests directory. Test suites grouping related tests. Nested test organization. Shared test utilities. Test data fixtures. Snapshot testing for outputs.

### Async Test Support
Support async test functions. Await completion of async tests. Timeout for async tests. Parallel async test execution. Mock async functions. Test async error handling. Integration with async runtime.

### Property-Based Testing
Generate random test inputs. Property verification on generated data. Shrinking failing inputs. Configurable input generators. Seed for reproducible tests. Number of iterations setting. QuickCheck-style testing.

## Tests (TDD - Use rstest)

**Test discovery tests:**
1. Discover tests by name prefix
2. Discover tests by annotation
3. Skip tests with skip annotation
4. Focus tests with only annotation
5. Test metadata extraction
6. Module-level test grouping

**Assertion tests:**
1. assert_eq with equal values
2. assert_eq failure with diff
3. assert_ne inequality
4. assert_true and assert_false
5. assert_ok for Result
6. assert_err for Result error
7. assert_contains for arrays
8. Custom assertion predicates

**Test runner tests:**
1. Run all discovered tests
2. Run tests in parallel
3. Filter tests by name
4. Run only failed tests
5. Test timeout enforcement
6. Setup and teardown execution
7. Test isolation
8. Shared fixtures

**Test output tests:**
1. Progress indicator
2. Pass/fail status display
3. Failure details
4. Summary statistics
5. Execution time reporting
6. Colorized terminal output
7. JSON output format
8. JUnit XML format

**Mocking tests:**
1. Create mock function
2. Stub return value
3. Verify function called
4. Call count assertion
5. Argument matching
6. Spy on function
7. Restore original function
8. Automatic cleanup

**Test organization tests:**
1. Test modules discovered
2. Test file suffix recognition
3. Integration test separation
4. Test suites grouping
5. Shared utilities
6. Snapshot testing

**Async test tests:**
1. Run async test function
2. Async test timeout
3. Parallel async tests
4. Mock async function
5. Async error handling

**Property-based tests:**
1. Generate random inputs
2. Verify property holds
3. Shrink failing input
4. Reproducible with seed
5. Configurable iterations

**Integration tests:**
1. CLI test runner integration
2. Real test suite execution
3. CI integration with exit codes
4. Coverage reporting
5. Test-driven development workflow

**Minimum test count:** 80 tests

## Integration Points
- Uses: Reflection API from foundation/phase-12
- Uses: Module system from foundation/phase-06
- Uses: Result types from foundation/phase-09
- Uses: CLI framework from v0.1
- Creates: Testing framework
- Creates: Assertion library
- Creates: Mock system
- Output: First-class testing support

## Acceptance
- Test discovery works
- Assertion library comprehensive
- Test runner executes tests
- Output reporting clear
- Mocking and stubbing functional
- Test organization supported
- Async tests work
- Property-based testing available
- 80+ tests pass
- CLI integration complete
- Documentation with examples
- TDD workflow enabled
- No clippy warnings
- cargo test passes
