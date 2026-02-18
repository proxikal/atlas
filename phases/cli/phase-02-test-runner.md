# Phase 02: Test Runner (Discovery, Execution, Reporting)

## 🚨 BLOCKERS - CHECK BEFORE STARTING

**REQUIRED:**
- CLI from v0.1 with command infrastructure
- Stdlib testing primitives (Stdlib/phase-15)

**Verification:**
```bash
# CLI exists
ls crates/atlas-cli/src/commands/
cargo run --bin atlas -- --help

# Testing primitives exist
grep -n "fn assert\|fn assertEqual" crates/atlas-runtime/src/stdlib/test.rs
cargo nextest run -p atlas-runtime -E 'test(test_primitives)'
```

**What's needed:**
- CLI from v0.1 with command structure
- Parser for discovering test functions
- Runtime for executing tests
- Configuration system from foundation/phase-04
- **Stdlib/phase-15 (testing primitives) MUST be complete**

**If missing:** Complete Stdlib/phase-15 first (provides assertions)

---

## Objective

Implement `atlas test` command - a full-featured test runner following the Rust (`cargo test`) and Go (`go test`) model. Discovers test functions, executes them in parallel, reports results. Uses stdlib testing primitives (Stdlib/phase-15) for assertions.

**Design Philosophy:** CLI orchestration layer (like `cargo test`), not a framework. Stdlib provides assertions, CLI provides discovery/execution/reporting.

---

## Architecture Decision

**Following: cargo test / go test Model**

```
┌────────────────────────────────────────┐
│ atlas test (THIS PHASE)                │
│ ┌────────────────────────────────────┐ │
│ │ 1. Discovery: Find test_* functions│ │
│ │ 2. Filter: atlas test test_foo     │ │
│ │ 3. Execute: Run tests in parallel  │ │
│ │ 4. Report: Pass/fail, timing, etc. │ │
│ └────────────────────────────────────┘ │
└───────────┬────────────────────────────┘
            │ uses
┌───────────▼────────────────────────────┐
│ Stdlib: assert(), assertEqual(), etc.  │  (Stdlib/phase-15)
└────────────────────────────────────────┘
```

**What this phase does:**
- ✅ Test discovery (find test functions)
- ✅ Test execution (run discovered tests)
- ✅ Parallel execution (tokio/rayon)
- ✅ Test filtering (`atlas test test_foo`)
- ✅ Test reporting (pass/fail, timing, colors)
- ✅ Exit codes (0 = pass, 1 = fail)
- ✅ CLI interface

**What this phase does NOT do:**
- ❌ Define assertions (Stdlib/phase-15)
- ❌ Implement assert logic (Stdlib/phase-15)

---

## Files

**Create:** `crates/atlas-cli/src/commands/test.rs` (~500 lines)
**Create:** `crates/atlas-cli/src/testing/mod.rs` (~100 lines)
**Create:** `crates/atlas-cli/src/testing/discovery.rs` (~300 lines)
**Create:** `crates/atlas-cli/src/testing/runner.rs` (~400 lines)
**Create:** `crates/atlas-cli/src/testing/reporter.rs` (~300 lines)
**Update:** `crates/atlas-cli/src/main.rs` (~10 lines - register command)
**Tests:** `crates/atlas-cli/tests/test_runner_integration.rs` (~400 lines)

**Total: ~2000 lines**

---

## Dependencies

- CLI from v0.1 (command infrastructure)
- **Stdlib/phase-15 (testing primitives) - CRITICAL**
- Parser for finding test functions
- Runtime for executing Atlas code
- Configuration system (foundation/phase-04)
- Optional: tokio for parallel execution

---

## Implementation

### GATE -1: Sanity Check ✅

```bash
cargo clean
cargo check -p atlas-cli
cargo check -p atlas-runtime

# Verify stdlib testing primitives exist
cargo nextest run -p atlas-runtime -E 'test(test_primitives)'
```

---

### GATE 0: Design Test Discovery Strategy

**Test function conventions (like Rust/Go):**

**Option A: Naming convention** (like Go)
```atlas
fn test_addition() -> void {
    assertEqual(2 + 2, 4);
}

fn test_division() -> void {
    let result = divide(10, 2);
    assertEqual(result, 5);
}
```

**Option B: Attribute annotation** (like Rust)
```atlas
@test
fn addition_works() -> void {
    assertEqual(2 + 2, 4);
}
```

**Recommendation: Start with naming convention (simpler), add attributes later.**

**Discovery algorithm:**
1. Parse all `.atl` files in project
2. Find functions starting with `test_`
3. Verify they take no parameters
4. Verify they return void
5. Build test list with file:line locations

**Acceptance:**
- ✅ Strategy defined
- ✅ Follows Rust/Go patterns
- ✅ Simple and clear

---

### ⚠️ API NOTE for GATE 1

Atlas Lexer and Parser do NOT return `Result`. Their actual API is:
```rust
let mut lexer = Lexer::new(&source);
let (tokens, lex_diags) = lexer.tokenize();  // returns (Vec<Token>, Vec<Diagnostic>)
let mut parser = Parser::new(tokens);
let (ast, parse_diags) = parser.parse();     // returns (Program, Vec<Diagnostic>)
```
Adapt the code examples below accordingly — they use `.map_err(...)` for illustration only.

### GATE 1: Implement Test Discovery

**File:** `crates/atlas-cli/src/testing/discovery.rs`

```rust
use crate::parser::{Parser, Lexer};
use std::path::{Path, PathBuf};
use std::fs;

#[derive(Debug, Clone)]
pub struct TestFunction {
    pub name: String,
    pub file: PathBuf,
    pub line: usize,
}

#[derive(Debug)]
pub struct TestSuite {
    pub tests: Vec<TestFunction>,
}

impl TestSuite {
    pub fn discover(root: &Path) -> Result<Self, String> {
        let mut tests = Vec::new();

        // Walk directory tree finding .atl files
        for entry in walkdir::WalkDir::new(root)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            if path.extension() == Some(std::ffi::OsStr::new("atl")) {
                // Parse file and find test functions
                if let Ok(file_tests) = discover_tests_in_file(path) {
                    tests.extend(file_tests);
                }
            }
        }

        Ok(TestSuite { tests })
    }

    pub fn filter(&self, pattern: &str) -> Self {
        let filtered = self.tests
            .iter()
            .filter(|t| t.name.contains(pattern))
            .cloned()
            .collect();

        TestSuite { tests: filtered }
    }
}

fn discover_tests_in_file(path: &Path) -> Result<Vec<TestFunction>, String> {
    let source = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize()
        .map_err(|e| format!("Lexer error in {}: {:?}", path.display(), e))?;

    let mut parser = Parser::new(tokens);
    let ast = parser.parse()
        .map_err(|e| format!("Parse error in {}: {:?}", path.display(), e))?;

    let mut tests = Vec::new();

    // Walk AST finding functions starting with "test_"
    for stmt in &ast.statements {
        if let Stmt::Function { name, params, return_type, .. } = stmt {
            if name.starts_with("test_") {
                // Verify test function signature
                if !params.is_empty() {
                    eprintln!("Warning: {} takes parameters, skipping", name);
                    continue;
                }

                if return_type != &Type::Void {
                    eprintln!("Warning: {} doesn't return void, skipping", name);
                    continue;
                }

                tests.push(TestFunction {
                    name: name.clone(),
                    file: path.to_path_buf(),
                    line: stmt.span().start.line,
                });
            }
        }
    }

    Ok(tests)
}
```

**Test:**
```bash
cargo nextest run -p atlas-cli -E 'test(test_discovery)'
```

**Acceptance:**
- ✅ Discovers test_* functions
- ✅ Finds tests in multiple files
- ✅ Filters by name pattern
- ✅ Validates test signatures

---

### GATE 2: Implement Test Runner

**File:** `crates/atlas-cli/src/testing/runner.rs`

```rust
use crate::testing::discovery::{TestFunction, TestSuite};
use atlas_runtime::Atlas;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub enum TestResult {
    Pass { duration: Duration },
    Fail { error: String, duration: Duration },
    Timeout { duration: Duration },
}

#[derive(Debug)]
pub struct TestRun {
    pub test: TestFunction,
    pub result: TestResult,
}

pub struct TestRunner {
    parallel: bool,
    timeout: Duration,
}

impl TestRunner {
    pub fn new() -> Self {
        Self {
            parallel: true,
            timeout: Duration::from_secs(30),
        }
    }

    pub fn with_parallel(mut self, parallel: bool) -> Self {
        self.parallel = parallel;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn run(&self, suite: &TestSuite) -> Vec<TestRun> {
        if self.parallel {
            self.run_parallel(suite)
        } else {
            self.run_sequential(suite)
        }
    }

    fn run_sequential(&self, suite: &TestSuite) -> Vec<TestRun> {
        suite.tests
            .iter()
            .map(|test| self.run_single_test(test))
            .collect()
    }

    fn run_parallel(&self, suite: &TestSuite) -> Vec<TestRun> {
        use rayon::prelude::*;

        suite.tests
            .par_iter()
            .map(|test| self.run_single_test(test))
            .collect()
    }

    fn run_single_test(&self, test: &TestFunction) -> TestRun {
        let start = Instant::now();

        // Load file and execute just this test function
        let source = std::fs::read_to_string(&test.file)
            .expect("Failed to read test file");

        // Create isolated runtime for this test
        let mut runtime = Atlas::new();

        // Execute file (defines functions)
        if let Err(e) = runtime.eval(&source) {
            return TestRun {
                test: test.clone(),
                result: TestResult::Fail {
                    error: format!("Failed to load test: {}", e),
                    duration: start.elapsed(),
                },
            };
        }

        // Call the test function
        let test_call = format!("{}();", test.name);

        match runtime.eval(&test_call) {
            Ok(_) => TestRun {
                test: test.clone(),
                result: TestResult::Pass {
                    duration: start.elapsed(),
                },
            },
            Err(e) => TestRun {
                test: test.clone(),
                result: TestResult::Fail {
                    error: e.to_string(),
                    duration: start.elapsed(),
                },
            },
        }
    }
}
```

**Test:**
```bash
cargo nextest run -p atlas-cli -E 'test(test_runner)'
```

**Acceptance:**
- ✅ Runs individual tests
- ✅ Runs tests in parallel
- ✅ Runs tests sequentially
- ✅ Isolates test execution
- ✅ Captures pass/fail results
- ✅ Measures timing

---

### GATE 3: Implement Test Reporter

**File:** `crates/atlas-cli/src/testing/reporter.rs`

```rust
use crate::testing::runner::{TestRun, TestResult};
use colored::*;

pub struct TestReporter {
    verbose: bool,
}

impl TestReporter {
    pub fn new(verbose: bool) -> Self {
        Self { verbose }
    }

    pub fn report(&self, runs: &[TestRun]) {
        // Print progress during test run
        for run in runs {
            self.print_test_result(run);
        }

        println!();
        self.print_summary(runs);
    }

    fn print_test_result(&self, run: &TestRun) {
        match &run.result {
            TestResult::Pass { duration } => {
                if self.verbose {
                    println!(
                        "{} {} ({:.2?})",
                        "PASS".green().bold(),
                        run.test.name,
                        duration
                    );
                } else {
                    print!("{}", ".".green());
                }
            }
            TestResult::Fail { error, duration } => {
                println!(
                    "{} {} ({:.2?})",
                    "FAIL".red().bold(),
                    run.test.name,
                    duration
                );
                if self.verbose {
                    println!("  {}", error.red());
                }
            }
            TestResult::Timeout { duration } => {
                println!(
                    "{} {} (timeout after {:.2?})",
                    "TIMEOUT".yellow().bold(),
                    run.test.name,
                    duration
                );
            }
        }
    }

    fn print_summary(&self, runs: &[TestRun]) {
        let total = runs.len();
        let passed = runs.iter()
            .filter(|r| matches!(r.result, TestResult::Pass { .. }))
            .count();
        let failed = runs.iter()
            .filter(|r| matches!(r.result, TestResult::Fail { .. }))
            .count();

        let total_duration: std::time::Duration = runs.iter()
            .map(|r| match &r.result {
                TestResult::Pass { duration } => *duration,
                TestResult::Fail { duration, .. } => *duration,
                TestResult::Timeout { duration } => *duration,
            })
            .sum();

        println!("─────────────────────────────────────");
        println!(
            "Tests: {} total, {} passed, {} failed",
            total.to_string().bold(),
            passed.to_string().green().bold(),
            failed.to_string().red().bold()
        );
        println!("Time:  {:.2?}", total_duration);

        if failed > 0 {
            println!();
            println!("{}", "Failed tests:".red().bold());
            for run in runs {
                if matches!(run.result, TestResult::Fail { .. }) {
                    println!("  • {}", run.test.name);
                    if let TestResult::Fail { error, .. } = &run.result {
                        for line in error.lines() {
                            println!("    {}", line.dimmed());
                        }
                    }
                }
            }
        }
    }
}
```

**Test:**
```bash
cargo nextest run -p atlas-cli -E 'test(test_reporter)'
```

**Acceptance:**
- ✅ Reports test results
- ✅ Colorized output
- ✅ Summary statistics
- ✅ Timing information
- ✅ Verbose/quiet modes

---

### GATE 4: Implement CLI Command

**File:** `crates/atlas-cli/src/commands/test.rs`

```rust
use clap::Args;
use crate::testing::{TestSuite, TestRunner, TestReporter};
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct TestCommand {
    /// Filter tests by name pattern
    #[arg(value_name = "PATTERN")]
    pattern: Option<String>,

    /// Run tests sequentially instead of parallel
    #[arg(long)]
    sequential: bool,

    /// Verbose output (show all test names)
    #[arg(short, long)]
    verbose: bool,

    /// Test directory (defaults to current directory)
    #[arg(long, default_value = ".")]
    dir: PathBuf,
}

impl TestCommand {
    pub fn execute(&self) -> anyhow::Result<()> {
        println!("{}", "Discovering tests...".bold());

        // Discover tests
        let mut suite = TestSuite::discover(&self.dir)?;

        // Apply filter if provided
        if let Some(pattern) = &self.pattern {
            suite = suite.filter(pattern);
        }

        if suite.tests.is_empty() {
            println!("{}", "No tests found.".yellow());
            return Ok(());
        }

        println!(
            "Found {} test{}",
            suite.tests.len(),
            if suite.tests.len() == 1 { "" } else { "s" }
        );
        println!();

        // Run tests
        let runner = TestRunner::new()
            .with_parallel(!self.sequential);

        let runs = runner.run(&suite);

        // Report results
        let reporter = TestReporter::new(self.verbose);
        reporter.report(&runs);

        // Exit with code 1 if any tests failed
        let failed = runs.iter()
            .any(|r| matches!(r.result, TestResult::Fail { .. }));

        if failed {
            std::process::exit(1);
        }

        Ok(())
    }
}
```

**Register in main.rs:**
```rust
#[derive(Subcommand)]
enum Commands {
    Run(commands::run::RunCommand),
    Test(commands::test::TestCommand),  // Add this
    // ... other commands
}

match &cli.command {
    Commands::Test(cmd) => cmd.execute()?,
    // ...
}
```

**Test:**
```bash
cargo build --bin atlas
./target/debug/atlas test --help
```

**Acceptance:**
- ✅ CLI command registered
- ✅ Help text displays
- ✅ Options work (--verbose, --sequential, pattern)

---

### GATE 5: Create Integration Tests

**Create test fixtures:**
```bash
mkdir -p crates/atlas-cli/tests/fixtures/sample_tests
```

**File:** `crates/atlas-cli/tests/fixtures/sample_tests/math_tests.atl`
```atlas
fn test_addition() -> void {
    assertEqual(2 + 2, 4);
    assertEqual(10 + 5, 15);
}

fn test_subtraction() -> void {
    assertEqual(10 - 5, 5);
    assertEqual(100 - 1, 99);
}

fn test_multiplication() -> void {
    assertEqual(3 * 4, 12);
}

fn test_division() -> void {
    assertEqual(10 / 2, 5);
}
```

**File:** `crates/atlas-cli/tests/test_runner_integration.rs`
```rust
use assert_cmd::Command;

#[test]
fn test_runner_discovers_tests() {
    let mut cmd = Command::cargo_bin("atlas").unwrap();
    cmd.arg("test")
        .arg("--dir")
        .arg("tests/fixtures/sample_tests")
        .arg("--verbose");

    cmd.assert()
        .success()
        .stdout(predicates::str::contains("test_addition"))
        .stdout(predicates::str::contains("test_subtraction"))
        .stdout(predicates::str::contains("PASS"));
}

#[test]
fn test_runner_filters_by_pattern() {
    let mut cmd = Command::cargo_bin("atlas").unwrap();
    cmd.arg("test")
        .arg("test_add")
        .arg("--dir")
        .arg("tests/fixtures/sample_tests");

    cmd.assert()
        .success()
        .stdout(predicates::str::contains("1 test"));
}

#[test]
fn test_runner_fails_on_assertion_failure() {
    // Create test file with failing assertion
    // ...

    let mut cmd = Command::cargo_bin("atlas").unwrap();
    cmd.arg("test")
        .arg("--dir")
        .arg("tests/fixtures/failing_tests");

    cmd.assert()
        .failure()  // Exit code 1
        .stdout(predicates::str::contains("FAIL"));
}
```

**Test:**
```bash
cargo nextest run -p atlas-cli -E 'test(test_runner_integration)'
```

**Acceptance:**
- ✅ Integration tests pass
- ✅ Test discovery works
- ✅ Filtering works
- ✅ Pass/fail detection works
- ✅ Exit codes correct

---

### GATE 6: Test Real-World Usage

**Create a real test file:**
```atlas
fn test_result_type() -> void {
    fn divide(a: number, b: number) -> Result<number, string> {
        if (b == 0) { return Err("division by zero"); }
        return Ok(a / b);
    }

    let result = divide(10, 2);
    let value = assertOk(result);
    assertEqual(value, 5);
}

fn test_option_type() -> void {
    fn find_first_even(arr: array) -> Option<number> {
        for (let i = 0; i < len(arr); i = i + 1) {
            if (arr[i] % 2 == 0) {
                return Some(arr[i]);
            }
        }
        return None;
    }

    let result = find_first_even([1, 3, 5, 8, 10]);
    let value = assertSome(result);
    assertEqual(value, 8);
}
```

**Run:**
```bash
atlas test --verbose
```

**Acceptance:**
- ✅ Real tests run successfully
- ✅ Uses stdlib assertions
- ✅ Result/Option integration works
- ✅ Output is clear and helpful

---

### GATE 7: Documentation

**Create:** `docs/testing.md`

```markdown
# Testing in Atlas

Atlas provides built-in testing support following the Rust/Go model.

## Writing Tests

Test functions start with `test_` prefix:

```atlas
fn test_addition() -> void {
    assertEqual(2 + 2, 4);
}

fn test_division() -> void {
    let result = divide(10, 2);
    assertEqual(result, 5);
}
```

## Running Tests

```bash
# Run all tests
atlas test

# Run specific test
atlas test test_addition

# Filter by pattern
atlas test test_div

# Verbose output
atlas test --verbose

# Sequential execution
atlas test --sequential
```

## Assertions

See stdlib documentation for available assertions:
- `assert(condition, message)`
- `assertEqual(actual, expected)`
- `assertOk(result)`, `assertErr(result)`
- `assertSome(option)`, `assertNone(option)`
- And more...

## Exit Codes

- `0` - All tests passed
- `1` - One or more tests failed
```

**Acceptance:**
- ✅ Documentation complete
- ✅ Examples clear
- ✅ Usage explained

---

### GATE 8: Clippy & Format

```bash
cargo clippy -p atlas-cli -- -D warnings
cargo fmt -p atlas-cli
```

**Acceptance:**
- ✅ Zero clippy warnings
- ✅ Code formatted

---

## Acceptance Criteria

**ALL must be met:**

1. ✅ Test discovery works (finds test_* functions)
2. ✅ Test execution works (runs tests)
3. ✅ Parallel execution works
4. ✅ Sequential execution works
5. ✅ Test filtering works
6. ✅ Test reporting works (colorized, timing)
7. ✅ CLI command works (`atlas test`)
8. ✅ Exit codes correct (0 = pass, 1 = fail)
9. ✅ Uses stdlib assertions (Stdlib/phase-15)
10. ✅ Integration tests pass
11. ✅ Real-world usage works
12. ✅ Documentation complete
13. ✅ Zero clippy warnings

---

## Handoff

**Commit message:**
```
feat(cli): Add test runner command - phase-02

Implements `atlas test` following Rust (cargo test) and Go (go test) model.

**Architecture:**
- CLI: Test discovery, execution, reporting (this phase)
- Stdlib: Assertions (Stdlib/phase-15)
- Clean separation like cargo test

**Features:**
- Test discovery (test_* functions)
- Parallel execution (rayon)
- Test filtering (atlas test pattern)
- Colorized reporting
- Timing information
- Exit codes (0 = pass, 1 = fail)

**Usage:**
```bash
atlas test                  # Run all tests
atlas test test_foo         # Filter by name
atlas test --verbose        # Show all test names
atlas test --sequential     # No parallel execution
```

**Tests:**
- Integration tests pass
- Real Atlas tests run successfully
- Uses stdlib assertions correctly

**Dependencies:**
- Requires Stdlib/phase-15 (testing primitives)

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
```

**Update STATUS.md:**
- CLI: Mark phase-02 complete
- Note: "Test runner (Rust/Go model)"

---

## Notes

**Why this approach:**
- Follows cargo test / go test model
- Clean separation: CLI orchestration, stdlib primitives
- No duplication with Stdlib/phase-15
- World-class standard

**Integration with stdlib:**
- Uses assert(), assertEqual(), etc. from Stdlib/phase-15
- Doesn't define assertions itself
- Clean dependency

**Future enhancements:**
- Attribute-based test marking (`@test`)
- Code coverage reporting
- Test fixtures and setup/teardown
- Property-based testing integration

**Time estimate:** 8-10 hours (full test runner implementation)
