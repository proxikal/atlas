//! Test command - run Atlas tests

use crate::testing::{TestReporter, TestRunner, TestSuite};
use anyhow::Result;
use colored::*;
use std::path::PathBuf;

/// Arguments for the test command
pub struct TestArgs {
    /// Filter tests by name pattern
    pub pattern: Option<String>,
    /// Run tests sequentially instead of parallel
    pub sequential: bool,
    /// Verbose output (show all test names)
    pub verbose: bool,
    /// Disable colored output
    pub no_color: bool,
    /// Test directory (defaults to current directory)
    pub dir: PathBuf,
    /// Output in JSON format
    pub json: bool,
}

impl Default for TestArgs {
    fn default() -> Self {
        Self {
            pattern: None,
            sequential: false,
            verbose: false,
            no_color: false,
            dir: PathBuf::from("."),
            json: false,
        }
    }
}

/// Run the test command
pub fn run(args: TestArgs) -> Result<()> {
    if args.no_color {
        colored::control::set_override(false);
    }

    if !args.json {
        println!("{}", "Discovering tests...".bold());
    }

    // Discover tests
    let mut suite = TestSuite::discover(&args.dir);

    // Report parse errors
    if !suite.parse_errors.is_empty() && !args.json {
        eprintln!();
        eprintln!("{}", "Parse errors in test files:".yellow().bold());
        for (path, error) in &suite.parse_errors {
            eprintln!("  {} {}", "●".yellow(), path.display());
            eprintln!("    {}", error.dimmed());
        }
        eprintln!();
    }

    // Apply filter if provided
    if let Some(pattern) = &args.pattern {
        suite = suite.filter(pattern);
    }

    if suite.is_empty() {
        if args.json {
            println!(
                "{}",
                serde_json::json!({
                    "tests": 0,
                    "passed": 0,
                    "failed": 0,
                    "message": "No tests found"
                })
            );
        } else {
            println!("{}", "No tests found.".yellow());
        }
        return Ok(());
    }

    if !args.json {
        println!(
            "Found {} test{}",
            suite.len().to_string().bold(),
            if suite.len() == 1 { "" } else { "s" }
        );
        println!();
    }

    // Run tests
    let runner = TestRunner::new().with_parallel(!args.sequential);
    let runs = runner.run(&suite);

    // Report results
    if args.json {
        let passed = runs.iter().filter(|r| r.result.is_pass()).count();
        let failed = runs.iter().filter(|r| r.result.is_fail()).count();

        let results: Vec<_> = runs
            .iter()
            .map(|r| {
                serde_json::json!({
                    "name": r.test.name,
                    "file": r.test.file.display().to_string(),
                    "line": r.test.line,
                    "passed": r.result.is_pass(),
                    "duration_ms": r.result.duration().as_millis(),
                })
            })
            .collect();

        println!(
            "{}",
            serde_json::json!({
                "tests": runs.len(),
                "passed": passed,
                "failed": failed,
                "results": results,
            })
        );
    } else {
        let reporter = TestReporter::new(args.verbose).with_no_color(args.no_color);
        reporter.report(&runs);
    }

    // Exit with code 1 if any tests failed
    let failed = runs.iter().any(|r| r.result.is_fail());
    if failed {
        std::process::exit(1);
    }

    if args.no_color {
        colored::control::unset_override();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_command_no_tests_found() {
        let dir = tempdir().unwrap();

        let args = TestArgs {
            dir: dir.path().to_path_buf(),
            ..Default::default()
        };

        // Should not panic, just report no tests
        let result = run(args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_command_discovers_and_runs() {
        let dir = tempdir().unwrap();

        fs::write(
            dir.path().join("test.at"),
            r#"
fn test_simple() {
    assert(true, "ok");
}
"#,
        )
        .unwrap();

        let args = TestArgs {
            dir: dir.path().to_path_buf(),
            verbose: true,
            no_color: true,
            ..Default::default()
        };

        let result = run(args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_command_filter() {
        let dir = tempdir().unwrap();

        fs::write(
            dir.path().join("test.at"),
            r#"
fn test_add() { assert(true, "ok"); }
fn test_sub() { assert(true, "ok"); }
"#,
        )
        .unwrap();

        let args = TestArgs {
            dir: dir.path().to_path_buf(),
            pattern: Some("add".to_string()),
            verbose: true,
            no_color: true,
            ..Default::default()
        };

        let result = run(args);
        assert!(result.is_ok());
    }
}
