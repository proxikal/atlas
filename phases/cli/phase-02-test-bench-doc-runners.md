# Phase 02: Test Runner, Benchmark Runner, Doc Generator

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** CLI from v0.1 with command infrastructure.

**Verification:**
```bash
ls crates/atlas-cli/src/commands/
cargo run --bin atlas -- --help
cargo test
```

**What's needed:**
- CLI from v0.1 with command structure
- Parser for discovering test functions
- Runtime for executing tests
- Configuration system from foundation/phase-04

**If missing:** CLI should exist from v0.1

---

## Objective
Implement built-in test runner discovering and executing test functions, benchmark runner measuring performance with statistical analysis, and documentation generator extracting doc comments producing markdown or HTML documentation.

## Files
**Create:** `crates/atlas-cli/src/commands/test.rs` (~500 lines)
**Create:** `crates/atlas-cli/src/commands/bench.rs` (~400 lines)
**Create:** `crates/atlas-cli/src/commands/doc.rs` (~400 lines)
**Create:** `crates/atlas-cli/src/testing/runner.rs` (~300 lines)
**Create:** `crates/atlas-cli/src/testing/reporter.rs` (~200 lines)
**Create:** `crates/atlas-cli/src/benching/runner.rs` (~300 lines)
**Create:** `crates/atlas-cli/src/benching/stats.rs` (~200 lines)
**Create:** `crates/atlas-cli/src/docs/extractor.rs` (~300 lines)
**Create:** `crates/atlas-cli/src/docs/generator.rs` (~300 lines)
**Tests:** `crates/atlas-cli/tests/test_runner_tests.rs` (~300 lines)
**Tests:** `crates/atlas-cli/tests/bench_runner_tests.rs` (~200 lines)
**Tests:** `crates/atlas-cli/tests/doc_generator_tests.rs` (~200 lines)

## Dependencies
- CLI from v0.1
- Parser for function discovery
- Runtime for test execution
- Configuration system

## Implementation

### Test Discovery
Implement test function discovery in source files. Parse source files finding all functions. Identify test functions by naming convention starting with test_. Extract test function names and locations. Support test modules and nested tests. Build test suite with all discovered tests. Report test count before execution.

### Test Execution
Execute discovered tests collecting results. Create isolated environment per test. Execute each test function. Catch assertion failures and errors. Record pass or fail status. Measure test execution time. Support parallel test execution. Collect all test results. Report summary with pass/fail counts.

### Test Filtering
Support test filtering by name patterns. Accept filter argument with glob pattern. Match test names against pattern. Execute only matching tests. Report filtered test count. Support multiple filter patterns. Support exclude patterns.

### Test Reporter
Implement test result reporting with clear output. Show progress during test execution. Display dots or test names per test. Report failures with error messages and locations. Show test execution times. Generate summary with total, passed, failed counts. Exit with non-zero status if any tests fail. Support verbose mode showing all test output.

### Benchmark Discovery
Discover benchmark functions by naming convention starting with bench_. Parse source files finding benchmark functions. Extract benchmark names and locations. Build benchmark suite with all discovered benchmarks. Report benchmark count before execution.

### Benchmark Execution
Execute benchmarks measuring performance. Run each benchmark multiple iterations for statistical validity. Measure execution time per iteration. Collect timing samples. Calculate statistical measures mean median standard deviation. Detect and report outliers. Support warmup iterations. Compare against baseline if available.

### Benchmark Statistics
Calculate comprehensive statistics for benchmark results. Compute mean execution time. Compute median for outlier resistance. Calculate standard deviation showing variance. Compute min and max times. Calculate percentiles 95th 99th. Format statistics readably with units. Compare against previous runs showing improvements or regressions.

### Benchmark Reporter
Report benchmark results with statistical analysis. Display benchmark name and measured times. Show mean median and standard deviation. Format times with appropriate units nanoseconds to seconds. Generate comparison report if baseline exists. Highlight significant changes. Support JSON output for tooling integration.

### Documentation Extraction
Extract documentation from source files. Parse source files identifying doc comments starting with three slashes. Associate doc comments with following function, type, or module. Extract comment text removing comment markers. Parse markdown in doc comments. Build documentation structure with hierarchy. Handle code examples in doc comments.

### Documentation Generation
Generate documentation from extracted comments. Create markdown files per module. Generate HTML with styling and navigation. Include function signatures and parameters. Format code examples with syntax highlighting. Generate table of contents. Create index of all documented items. Support search functionality. Generate cross-references between items.

### CLI Integration
Integrate test, bench, and doc commands into CLI. Add test subcommand with filter and parallel flags. Add bench subcommand with baseline and iterations flags. Add doc subcommand with output format flags. Parse command arguments and flags. Execute appropriate runner with configuration. Report results to user. Exit with appropriate status codes.

## Tests (TDD - Use rstest)

**Test runner tests:**
1. Discover test functions
2. Execute passing tests
3. Execute failing tests
4. Test filtering by pattern
5. Parallel execution
6. Result reporting
7. Summary generation
8. Exit status codes
9. Isolated environments
10. Timing measurement

**Benchmark runner tests:**
1. Discover benchmark functions
2. Execute benchmarks
3. Statistical calculation
4. Timing accuracy
5. Iteration count
6. Baseline comparison
7. Result reporting
8. JSON output
9. Outlier detection
10. Warmup iterations

**Doc generator tests:**
1. Extract doc comments
2. Parse markdown
3. Generate markdown output
4. Generate HTML output
5. Code example handling
6. Cross-reference generation
7. Table of contents
8. Index generation
9. Module hierarchy
10. Function signature formatting

**Minimum test count:** 80 tests (30 test, 25 bench, 25 doc)

## Integration Points
- Uses: CLI from v0.1
- Uses: Parser for discovery
- Uses: Runtime for execution
- Creates: Test runner infrastructure
- Creates: Benchmark runner infrastructure
- Creates: Doc generator infrastructure
- Output: Professional development tools

## Acceptance
- atlas test discovers and runs all test functions
- Test filtering works with patterns
- Parallel test execution functional
- Test results clearly reported
- atlas bench runs benchmarks with statistics
- Benchmark statistics accurate mean median stddev
- Baseline comparison shows improvements
- atlas doc generates documentation from comments
- Doc output in markdown and HTML formats
- Documentation includes signatures and examples
- 80+ tests pass 30 test 25 bench 25 doc
- All commands have comprehensive help
- Exit status codes appropriate
- No clippy warnings
- cargo test passes
