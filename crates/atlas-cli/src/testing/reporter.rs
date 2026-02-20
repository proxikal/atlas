//! Test reporter - display test results

use crate::testing::runner::{TestResult, TestRun};
use colored::*;
use std::io::{self, Write};
use std::time::Duration;

/// Test reporter with output configuration
pub struct TestReporter {
    /// Show detailed output for each test
    verbose: bool,
    /// Disable colored output
    no_color: bool,
}

impl Default for TestReporter {
    fn default() -> Self {
        Self::new(false)
    }
}

impl TestReporter {
    /// Create a new test reporter
    pub fn new(verbose: bool) -> Self {
        Self {
            verbose,
            no_color: false,
        }
    }

    /// Disable colored output
    pub fn with_no_color(mut self, no_color: bool) -> Self {
        self.no_color = no_color;
        self
    }

    /// Report test results
    pub fn report(&self, runs: &[TestRun]) {
        if self.no_color {
            colored::control::set_override(false);
        }

        // Print individual test results
        for run in runs {
            self.print_test_result(run);
        }

        // Newline before summary if not verbose (dots need newline)
        if !self.verbose && !runs.is_empty() {
            println!();
        }

        // Print summary
        println!();
        self.print_summary(runs);

        // Print failed test details
        self.print_failures(runs);

        // Reset color override
        if self.no_color {
            colored::control::unset_override();
        }
    }

    /// Print a single test result
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
                    let _ = io::stdout().flush();
                }
            }
            TestResult::Fail { error: _, duration } => {
                if self.verbose {
                    println!(
                        "{} {} ({:.2?})",
                        "FAIL".red().bold(),
                        run.test.name,
                        duration
                    );
                } else {
                    print!("{}", "F".red().bold());
                    let _ = io::stdout().flush();
                }
            }
            TestResult::Timeout { duration } => {
                if self.verbose {
                    println!(
                        "{} {} (timeout after {:.2?})",
                        "TIMEOUT".yellow().bold(),
                        run.test.name,
                        duration
                    );
                } else {
                    print!("{}", "T".yellow().bold());
                    let _ = io::stdout().flush();
                }
            }
        }
    }

    /// Print summary statistics
    fn print_summary(&self, runs: &[TestRun]) {
        let total = runs.len();
        let passed = runs.iter().filter(|r| r.result.is_pass()).count();
        let failed = runs.iter().filter(|r| r.result.is_fail()).count();

        let total_duration: Duration = runs.iter().map(|r| r.result.duration()).sum();

        println!("{}", "─".repeat(50));

        let status = if failed > 0 {
            "FAILED".red().bold()
        } else {
            "PASSED".green().bold()
        };

        println!(
            "Test result: {} | {} total, {} passed, {} failed",
            status,
            total.to_string().bold(),
            passed.to_string().green().bold(),
            if failed > 0 {
                failed.to_string().red().bold()
            } else {
                failed.to_string().normal()
            }
        );
        println!("Time: {:.2?}", total_duration);
    }

    /// Print details of failed tests
    fn print_failures(&self, runs: &[TestRun]) {
        let failures: Vec<_> = runs.iter().filter(|r| r.result.is_fail()).collect();

        if failures.is_empty() {
            return;
        }

        println!();
        println!("{}", "Failures:".red().bold());
        println!();

        for run in failures {
            println!(
                "  {} {}:{}",
                "●".red(),
                run.test.file.display(),
                run.test.line
            );
            println!("    {}", run.test.name.bold());

            if let TestResult::Fail { error, .. } = &run.result {
                for line in error.lines() {
                    println!("      {}", line.dimmed());
                }
            } else if let TestResult::Timeout { duration } = &run.result {
                println!("      {} after {:.2?}", "Timed out".yellow(), duration);
            }
            println!();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::discovery::TestFunction;
    use std::path::PathBuf;
    use std::time::Duration;

    fn make_pass(name: &str) -> TestRun {
        TestRun {
            test: TestFunction {
                name: name.to_string(),
                file: PathBuf::from("test.at"),
                line: 1,
            },
            result: TestResult::Pass {
                duration: Duration::from_millis(10),
            },
        }
    }

    fn make_fail(name: &str, error: &str) -> TestRun {
        TestRun {
            test: TestFunction {
                name: name.to_string(),
                file: PathBuf::from("test.at"),
                line: 1,
            },
            result: TestResult::Fail {
                error: error.to_string(),
                duration: Duration::from_millis(5),
            },
        }
    }

    #[test]
    fn test_reporter_all_pass() {
        let runs = vec![make_pass("test_one"), make_pass("test_two")];

        let reporter = TestReporter::new(true).with_no_color(true);
        // Just verify it doesn't panic
        reporter.report(&runs);
    }

    #[test]
    fn test_reporter_with_failures() {
        let runs = vec![
            make_pass("test_pass"),
            make_fail("test_fail", "assertion failed"),
        ];

        let reporter = TestReporter::new(true).with_no_color(true);
        // Just verify it doesn't panic
        reporter.report(&runs);
    }

    #[test]
    fn test_reporter_quiet_mode() {
        let runs = vec![make_pass("test_one"), make_pass("test_two")];

        let reporter = TestReporter::new(false).with_no_color(true);
        // Just verify it doesn't panic (quiet mode prints dots)
        reporter.report(&runs);
    }

    #[test]
    fn test_reporter_empty() {
        let runs: Vec<TestRun> = vec![];

        let reporter = TestReporter::new(true).with_no_color(true);
        reporter.report(&runs);
    }
}
