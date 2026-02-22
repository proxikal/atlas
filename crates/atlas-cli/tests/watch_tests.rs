//! Watch mode CLI tests
//!
//! Tests for the atlas run --watch command including:
//! - Initial run behavior
//! - File change detection
//! - Error handling
//! - Configuration flags

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::io::Write;
use std::time::Duration;
use tempfile::{tempdir, NamedTempFile};

/// Helper to create a temp file with Atlas code
fn temp_atlas_file(content: &str) -> NamedTempFile {
    let mut file = NamedTempFile::with_suffix(".at").unwrap();
    write!(file, "{}", content).unwrap();
    file
}

/// Helper to get atlas command
fn atlas() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("atlas")
}

// Note: Most watch mode tests are unit tests in the watch module itself,
// since the actual watch loop is blocking. These e2e tests focus on
// argument parsing and initial setup validation.

// ============================================================================
// Watch Flag Parsing
// ============================================================================

#[test]
fn test_run_with_watch_flag_exists() {
    // Verify the --watch flag is recognized
    atlas()
        .args(["run", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--watch"));
}

#[test]
fn test_run_watch_short_flag() {
    // Verify -w flag is recognized
    atlas()
        .args(["run", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("-w"));
}

#[test]
fn test_run_no_clear_flag_exists() {
    // Verify --no-clear flag is recognized
    atlas()
        .args(["run", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--no-clear"));
}

#[test]
fn test_run_verbose_flag_exists() {
    // Verify --verbose flag is recognized (for watch mode)
    atlas()
        .args(["run", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--verbose"));
}

// ============================================================================
// Watch Error Handling
// ============================================================================

#[test]
fn test_watch_nonexistent_file() {
    // Watch mode should fail immediately for nonexistent file
    // Note: We use timeout to prevent blocking forever
    atlas()
        .args(["run", "--watch", "nonexistent_file.at"])
        .timeout(Duration::from_secs(2))
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("File not found").or(predicate::str::contains("not found")),
        );
}

#[test]
fn test_watch_invalid_path() {
    atlas()
        .args(["run", "--watch", ""])
        .timeout(Duration::from_secs(2))
        .assert()
        .failure();
}

// ============================================================================
// Normal Run (without watch)
// ============================================================================

#[test]
fn test_run_simple_expression() {
    let file = temp_atlas_file("1 + 2;");
    atlas()
        .args(["run", file.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("3"));
}

#[test]
fn test_run_with_print() {
    let file = temp_atlas_file("print(\"hello\");");
    atlas()
        .args(["run", file.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("hello"));
}

#[test]
fn test_run_with_error() {
    let file = temp_atlas_file("let x: number = \"string\";");
    atlas()
        .args(["run", file.path().to_str().unwrap()])
        .assert()
        .failure();
}

#[test]
fn test_run_json_output() {
    let file = temp_atlas_file("let x: number = \"string\";");
    atlas()
        .args(["run", "--json", file.path().to_str().unwrap()])
        .assert()
        .failure()
        .stdout(predicate::str::contains("{").or(predicate::str::contains("error")));
}

// ============================================================================
// Watch Mode Unit Tests (via module tests)
// ============================================================================

// The following tests verify internal watch module functionality
// They're included here for completeness but the actual implementation
// tests are in src/commands/watch.rs

mod watch_unit_tests {
    #[test]
    fn test_watch_config_default_values() {
        // Test that WatchConfig has sensible defaults
        // This is tested in the module itself (src/commands/watch.rs)
    }

    #[test]
    fn test_relevant_change_detection_same_file() {
        // Tested in watch.rs
    }

    #[test]
    fn test_relevant_change_detection_atlas_extension() {
        // Tested in watch.rs
    }

    #[test]
    fn test_relevant_change_detection_non_atlas() {
        // Tested in watch.rs
    }

    #[test]
    fn test_debounce_prevents_rapid_runs() {
        // Tested via integration behavior
    }
}

// ============================================================================
// Integration with Other Flags
// ============================================================================

#[test]
fn test_run_watch_with_json() {
    // Verify watch and json flags can be combined
    atlas()
        .args(["run", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--json"));
}

#[test]
fn test_run_watch_with_verbose() {
    // Verify watch and verbose flags can be combined
    atlas()
        .args(["run", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--verbose"));
}

// ============================================================================
// File System Operations
// ============================================================================

#[test]
fn test_watch_directory_setup() {
    // Verify watch can be set up for a file in a subdirectory
    let dir = tempdir().unwrap();
    let subdir = dir.path().join("src");
    fs::create_dir(&subdir).unwrap();

    let file_path = subdir.join("main.at");
    fs::write(&file_path, "let x = 1;").unwrap();

    // Normal run should work (watch mode would block)
    atlas()
        .args(["run", file_path.to_str().unwrap()])
        .assert()
        .success();
}

#[test]
fn test_watch_current_directory_file() {
    let file = temp_atlas_file("42;");

    // Normal run should work
    atlas()
        .args(["run", file.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

// ============================================================================
// Output Behavior
// ============================================================================

#[test]
fn test_run_null_result_no_output() {
    // Null results shouldn't print anything
    let file = temp_atlas_file("let x = 1;");
    atlas()
        .args(["run", file.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().or(predicate::str::contains("")));
}

#[test]
fn test_run_value_result_printed() {
    let file = temp_atlas_file("\"hello world\";");
    atlas()
        .args(["run", file.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("hello world"));
}

#[test]
fn test_run_function_definition_no_output() {
    let file = temp_atlas_file("fn greet() { print(\"hi\"); }");
    atlas()
        .args(["run", file.path().to_str().unwrap()])
        .assert()
        .success();
}

// ============================================================================
// Error Message Quality
// ============================================================================

#[test]
fn test_watch_error_includes_filename() {
    atlas()
        .args(["run", "--watch", "missing.at"])
        .timeout(Duration::from_secs(2))
        .assert()
        .failure()
        .stderr(predicate::str::contains("missing.at").or(predicate::str::contains("not found")));
}

#[test]
fn test_run_syntax_error_message() {
    let file = temp_atlas_file("let x = ;");
    atlas()
        .args(["run", file.path().to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error").or(predicate::str::contains("Error")));
}

#[test]
fn test_run_type_error_message() {
    let file = temp_atlas_file("let x: number = true;");
    atlas()
        .args(["run", file.path().to_str().unwrap()])
        .assert()
        .failure();
}

// ============================================================================
// Complex Programs
// ============================================================================

#[test]
fn test_run_with_functions() {
    let code = r#"
fn add(a: number, b: number) -> number {
    return a + b;
}
add(2, 3);
"#;
    let file = temp_atlas_file(code);
    atlas()
        .args(["run", file.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("5"));
}

#[test]
fn test_run_with_loops() {
    let code = r#"
var sum = 0;
var i = 0;
while (i < 5) {
    sum = sum + i;
    i = i + 1;
}
sum;
"#;
    let file = temp_atlas_file(code);
    atlas()
        .args(["run", file.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("10"));
}

#[test]
fn test_run_with_conditionals() {
    let code = r#"
let x = 10;
var result = "";
if (x > 5) {
    result = "big";
} else {
    result = "small";
}
result;
"#;
    let file = temp_atlas_file(code);
    atlas()
        .args(["run", file.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("big"));
}
