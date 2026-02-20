//! Test runner - execute discovered tests

use crate::testing::discovery::{TestFunction, TestSuite};
use atlas_runtime::api::{ExecutionMode, Runtime};
use rayon::prelude::*;
use std::fs;
use std::time::{Duration, Instant};

/// Result of running a single test
#[derive(Debug, Clone)]
pub enum TestResult {
    /// Test passed successfully
    Pass { duration: Duration },
    /// Test failed with an error message
    Fail { error: String, duration: Duration },
    /// Test timed out (future feature)
    #[allow(dead_code)]
    Timeout { duration: Duration },
}

impl TestResult {
    /// Check if this result is a pass
    pub fn is_pass(&self) -> bool {
        matches!(self, TestResult::Pass { .. })
    }

    /// Check if this result is a failure
    pub fn is_fail(&self) -> bool {
        matches!(self, TestResult::Fail { .. } | TestResult::Timeout { .. })
    }

    /// Get the duration of this test
    pub fn duration(&self) -> Duration {
        match self {
            TestResult::Pass { duration } => *duration,
            TestResult::Fail { duration, .. } => *duration,
            TestResult::Timeout { duration } => *duration,
        }
    }
}

/// A completed test run
#[derive(Debug, Clone)]
pub struct TestRun {
    /// The test that was run
    pub test: TestFunction,
    /// Result of running the test
    pub result: TestResult,
}

/// Test runner with configuration
pub struct TestRunner {
    /// Whether to run tests in parallel
    parallel: bool,
    /// Timeout for individual tests
    #[allow(dead_code)]
    timeout: Duration,
}

impl Default for TestRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl TestRunner {
    /// Create a new test runner with default settings
    pub fn new() -> Self {
        Self {
            parallel: true,
            timeout: Duration::from_secs(30),
        }
    }

    /// Set whether to run tests in parallel
    pub fn with_parallel(mut self, parallel: bool) -> Self {
        self.parallel = parallel;
        self
    }

    /// Set the timeout for individual tests
    #[allow(dead_code)]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Run all tests in the suite
    pub fn run(&self, suite: &TestSuite) -> Vec<TestRun> {
        if self.parallel {
            self.run_parallel(suite)
        } else {
            self.run_sequential(suite)
        }
    }

    /// Run tests sequentially
    fn run_sequential(&self, suite: &TestSuite) -> Vec<TestRun> {
        suite
            .tests
            .iter()
            .map(|test| self.run_single_test(test))
            .collect()
    }

    /// Run tests in parallel using rayon
    fn run_parallel(&self, suite: &TestSuite) -> Vec<TestRun> {
        suite
            .tests
            .par_iter()
            .map(|test| self.run_single_test(test))
            .collect()
    }

    /// Run a single test
    fn run_single_test(&self, test: &TestFunction) -> TestRun {
        let start = Instant::now();

        // Load the test file
        let source = match fs::read_to_string(&test.file) {
            Ok(s) => s,
            Err(e) => {
                return TestRun {
                    test: test.clone(),
                    result: TestResult::Fail {
                        error: format!("Failed to read test file: {}", e),
                        duration: start.elapsed(),
                    },
                };
            }
        };

        // Create isolated runtime for this test (using VM mode for state persistence)
        let mut runtime = Runtime::new(ExecutionMode::Interpreter);

        // Execute the file to define functions
        if let Err(e) = runtime.eval(&source) {
            return TestRun {
                test: test.clone(),
                result: TestResult::Fail {
                    error: format!("Failed to load test file: {}", e),
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::NamedTempFile;

    fn create_test_file(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::with_suffix(".at").unwrap();
        write!(file, "{}", content).unwrap();
        file
    }

    #[test]
    fn test_runner_pass() {
        let file = create_test_file(
            r#"
fn test_simple() {
    assert(true, "should pass");
}
"#,
        );

        let test = TestFunction {
            name: "test_simple".to_string(),
            file: file.path().to_path_buf(),
            line: 2,
        };

        let runner = TestRunner::new();
        let result = runner.run_single_test(&test);

        assert!(result.result.is_pass());
    }

    #[test]
    fn test_runner_fail() {
        let file = create_test_file(
            r#"
fn test_failing() {
    assert(false, "should fail");
}
"#,
        );

        let test = TestFunction {
            name: "test_failing".to_string(),
            file: file.path().to_path_buf(),
            line: 2,
        };

        let runner = TestRunner::new();
        let result = runner.run_single_test(&test);

        assert!(result.result.is_fail());
    }

    #[test]
    fn test_runner_missing_file() {
        let test = TestFunction {
            name: "test_missing".to_string(),
            file: PathBuf::from("/nonexistent/file.at"),
            line: 1,
        };

        let runner = TestRunner::new();
        let result = runner.run_single_test(&test);

        assert!(result.result.is_fail());
        if let TestResult::Fail { error, .. } = result.result {
            assert!(error.contains("Failed to read"));
        }
    }

    #[test]
    fn test_runner_sequential() {
        let file = create_test_file(
            r#"
fn test_one() { assert(true, "ok"); }
fn test_two() { assert(true, "ok"); }
"#,
        );

        let suite = TestSuite {
            tests: vec![
                TestFunction {
                    name: "test_one".to_string(),
                    file: file.path().to_path_buf(),
                    line: 2,
                },
                TestFunction {
                    name: "test_two".to_string(),
                    file: file.path().to_path_buf(),
                    line: 3,
                },
            ],
            parse_errors: Vec::new(),
        };

        let runner = TestRunner::new().with_parallel(false);
        let results = runner.run(&suite);

        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.result.is_pass()));
    }

    #[test]
    fn test_runner_parallel() {
        let file = create_test_file(
            r#"
fn test_a() { assert(true, "ok"); }
fn test_b() { assert(true, "ok"); }
fn test_c() { assert(true, "ok"); }
"#,
        );

        let suite = TestSuite {
            tests: vec![
                TestFunction {
                    name: "test_a".to_string(),
                    file: file.path().to_path_buf(),
                    line: 2,
                },
                TestFunction {
                    name: "test_b".to_string(),
                    file: file.path().to_path_buf(),
                    line: 3,
                },
                TestFunction {
                    name: "test_c".to_string(),
                    file: file.path().to_path_buf(),
                    line: 4,
                },
            ],
            parse_errors: Vec::new(),
        };

        let runner = TestRunner::new().with_parallel(true);
        let results = runner.run(&suite);

        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| r.result.is_pass()));
    }

    #[test]
    fn test_result_duration() {
        let pass = TestResult::Pass {
            duration: Duration::from_millis(100),
        };
        assert_eq!(pass.duration(), Duration::from_millis(100));

        let fail = TestResult::Fail {
            error: "error".to_string(),
            duration: Duration::from_millis(50),
        };
        assert_eq!(fail.duration(), Duration::from_millis(50));
    }
}
