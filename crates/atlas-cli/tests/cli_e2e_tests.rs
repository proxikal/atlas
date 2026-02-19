//! End-to-end integration tests for CLI commands
//!
//! These tests verify the full pipeline for:
//! - `atlas run` - Execute source files
//! - `atlas build` - Compile to bytecode
//! - `atlas check` - Type check without execution
//!
//! Tests cover:
//! - Successful execution paths
//! - Error handling and exit codes
//! - File I/O operations
//! - Output formatting (JSON and human-readable)

use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

// ============================================================================
// Test Helpers
// ============================================================================

/// Create a temporary directory with a test file
fn create_test_file(filename: &str, content: &str) -> (TempDir, String) {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join(filename);
    fs::write(&file_path, content).unwrap();
    (temp_dir, file_path.to_str().unwrap().to_string())
}

/// Create a temporary Atlas project with atlas.toml and src/main.atlas
fn create_test_project(main_source: &str) -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();
    fs::create_dir(root.join("src")).unwrap();
    fs::write(
        root.join("atlas.toml"),
        r#"
[package]
name = "test-project"
version = "0.1.0"
"#,
    )
    .unwrap();
    fs::write(root.join("src/main.atlas"), main_source).unwrap();
    temp_dir
}

// ============================================================================
// atlas run - Success Cases
// ============================================================================

#[test]
fn test_run_simple_expression() {
    let (_dir, path) = create_test_file("test.atl", "42;");

    assert_cmd::cargo::cargo_bin_cmd!("atlas")
        .arg("run")
        .arg(&path)
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

#[test]
fn test_run_arithmetic() {
    let (_dir, path) = create_test_file("test.atl", "1 + 2 * 3;");

    assert_cmd::cargo::cargo_bin_cmd!("atlas")
        .arg("run")
        .arg(&path)
        .assert()
        .success()
        .stdout(predicate::str::contains("7"));
}

#[test]
fn test_run_string_output() {
    let (_dir, path) = create_test_file("test.atl", r#""hello world";"#);

    assert_cmd::cargo::cargo_bin_cmd!("atlas")
        .arg("run")
        .arg(&path)
        .assert()
        .success()
        .stdout(predicate::str::contains("hello world"));
}

#[test]
fn test_run_boolean_output() {
    let (_dir, path) = create_test_file("test.atl", "true;");

    assert_cmd::cargo::cargo_bin_cmd!("atlas")
        .arg("run")
        .arg(&path)
        .assert()
        .success()
        .stdout(predicate::str::contains("true"));
}

#[test]
fn test_run_null_no_output() {
    let (_dir, path) = create_test_file("test.atl", "null;");

    assert_cmd::cargo::cargo_bin_cmd!("atlas")
        .arg("run")
        .arg(&path)
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn test_run_variable_declaration_no_output() {
    let (_dir, path) = create_test_file("test.atl", "let x: number = 42;");

    assert_cmd::cargo::cargo_bin_cmd!("atlas")
        .arg("run")
        .arg(&path)
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn test_run_function_call() {
    let source = r#"
fn add(a: number, b: number) -> number {
    return a + b;
}
add(10, 20);
"#;
    let (_dir, path) = create_test_file("test.atl", source);

    assert_cmd::cargo::cargo_bin_cmd!("atlas")
        .arg("run")
        .arg(&path)
        .assert()
        .success()
        .stdout(predicate::str::contains("30"));
}

#[test]
fn test_run_array_literal() {
    let source = "[1, 2, 3];";
    let (_dir, path) = create_test_file("test.atl", source);

    assert_cmd::cargo::cargo_bin_cmd!("atlas")
        .arg("run")
        .arg(&path)
        .assert()
        .success()
        .stdout(predicate::str::contains("[1, 2, 3]"));
}

#[test]
fn test_run_array_access() {
    let source = r#"
let arr: number[] = [10, 20, 30];
arr[1];
"#;
    let (_dir, path) = create_test_file("test.atl", source);

    assert_cmd::cargo::cargo_bin_cmd!("atlas")
        .arg("run")
        .arg(&path)
        .assert()
        .success()
        .stdout(predicate::str::contains("20"));
}

#[test]
fn test_run_if_statement() {
    let source = r#"
if (true) {
    42;
}
"#;
    let (_dir, path) = create_test_file("test.atl", source);

    assert_cmd::cargo::cargo_bin_cmd!("atlas")
        .arg("run")
        .arg(&path)
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

#[test]
fn test_run_while_loop() {
    let source = r#"
var i: number = 0;
var sum: number = 0;
while (i < 5) {
    sum = sum + i;
    i = i + 1;
}
sum;
"#;
    let (_dir, path) = create_test_file("test.atl", source);

    assert_cmd::cargo::cargo_bin_cmd!("atlas")
        .arg("run")
        .arg(&path)
        .assert()
        .success()
        .stdout(predicate::str::contains("10"));
}

// ============================================================================
// atlas run - Error Cases
// ============================================================================

#[test]
fn test_run_missing_file() {
    assert_cmd::cargo::cargo_bin_cmd!("atlas")
        .arg("run")
        .arg("nonexistent.atl")
        .assert()
        .failure();
}

#[test]
fn test_run_parse_error() {
    let (_dir, path) = create_test_file("test.atl", "let x =");

    assert_cmd::cargo::cargo_bin_cmd!("atlas")
        .arg("run")
        .arg(&path)
        .assert()
        .failure();
}

#[test]
fn test_run_type_error() {
    let (_dir, path) = create_test_file("test.atl", r#"let x: number = "wrong";"#);

    assert_cmd::cargo::cargo_bin_cmd!("atlas")
        .arg("run")
        .arg(&path)
        .assert()
        .failure();
}

#[test]
fn test_run_type_error_function_call() {
    let source = r#"
fn greet(name: string) -> string {
    return name;
}
greet(42);
"#;
    let (_dir, path) = create_test_file("test.atl", source);

    assert_cmd::cargo::cargo_bin_cmd!("atlas")
        .arg("run")
        .arg(&path)
        .assert()
        .failure();
}

#[test]
fn test_run_undefined_variable() {
    let (_dir, path) = create_test_file("test.atl", "x;");

    assert_cmd::cargo::cargo_bin_cmd!("atlas")
        .arg("run")
        .arg(&path)
        .assert()
        .failure();
}

#[test]
fn test_run_json_flag_on_error() {
    let (_dir, path) = create_test_file("test.atl", r#"let x: number = "wrong";"#);

    assert_cmd::cargo::cargo_bin_cmd!("atlas")
        .arg("run")
        .arg(&path)
        .arg("--json")
        .assert()
        .failure()
        .stdout(predicate::str::contains("diag_version"));
}

// ============================================================================
// atlas build - Success Cases
// ============================================================================

#[test]
fn test_build_creates_bytecode_file() {
    let temp_dir = create_test_project("let x: number = 42;");
    let bytecode_path = temp_dir.path().join("target/debug/bin/test-project.atl.bc");

    assert_cmd::cargo::cargo_bin_cmd!("atlas")
        .current_dir(temp_dir.path())
        .arg("build")
        .assert()
        .success()
        .stdout(predicate::str::contains("Compiled"));

    assert!(bytecode_path.exists(), "Bytecode file should be created");
}

#[test]
fn test_build_with_function() {
    let source = r#"
fn add(a: number, b: number) -> number {
    return a + b;
}
"#;
    let temp_dir = create_test_project(source);
    let bytecode_path = temp_dir.path().join("target/debug/bin/test-project.atl.bc");

    assert_cmd::cargo::cargo_bin_cmd!("atlas")
        .current_dir(temp_dir.path())
        .arg("build")
        .assert()
        .success();

    assert!(bytecode_path.exists());
}

#[test]
fn test_build_with_disasm_flag() {
    let temp_dir = create_test_project("let x: number = 42;");

    assert_cmd::cargo::cargo_bin_cmd!("atlas")
        .current_dir(temp_dir.path())
        .arg("build")
        .arg("--verbose")
        .assert()
        .success()
        .stdout(predicate::str::contains("Compiled"));
}

#[test]
fn test_build_complex_program() {
    let source = r#"
fn factorial(n: number) -> number {
    if (n <= 1) {
        return 1;
    }
    return n * factorial(n - 1);
}

let result: number = factorial(5);
"#;
    let temp_dir = create_test_project(source);
    let bytecode_path = temp_dir.path().join("target/debug/bin/test-project.atl.bc");

    assert_cmd::cargo::cargo_bin_cmd!("atlas")
        .current_dir(temp_dir.path())
        .arg("build")
        .assert()
        .success();

    assert!(bytecode_path.exists());

    // Verify bytecode file is not empty
    let metadata = fs::metadata(&bytecode_path).unwrap();
    assert!(metadata.len() > 0, "Bytecode file should not be empty");
}

// ============================================================================
// atlas build - Error Cases
// ============================================================================

#[test]
fn test_build_missing_file() {
    assert_cmd::cargo::cargo_bin_cmd!("atlas")
        .current_dir(TempDir::new().unwrap().path())
        .arg("build")
        .assert()
        .failure();
}

#[test]
fn test_build_parse_error() {
    let temp_dir = create_test_project("let x =");

    assert_cmd::cargo::cargo_bin_cmd!("atlas")
        .current_dir(temp_dir.path())
        .arg("build")
        .assert()
        .failure();
}

#[test]
fn test_build_type_error() {
    let temp_dir = create_test_project(r#"let x: number = "wrong";"#);

    assert_cmd::cargo::cargo_bin_cmd!("atlas")
        .current_dir(temp_dir.path())
        .arg("build")
        .assert()
        .failure();
}

#[test]
fn test_build_json_flag_on_error() {
    let temp_dir = create_test_project(r#"let x: number = "wrong";"#);

    assert_cmd::cargo::cargo_bin_cmd!("atlas")
        .current_dir(temp_dir.path())
        .arg("build")
        .arg("--json")
        .assert()
        .failure()
        .stdout(predicate::str::contains("diag_version"));
}

// ============================================================================
// atlas check - Success Cases
// ============================================================================

#[test]
fn test_check_valid_program() {
    let (_dir, path) = create_test_file("test.atl", "let x: number = 42;");

    assert_cmd::cargo::cargo_bin_cmd!("atlas")
        .arg("check")
        .arg(&path)
        .assert()
        .success()
        .stdout(predicate::str::contains("No errors found"));
}

#[test]
fn test_check_complex_valid_program() {
    let source = r#"
fn add(a: number, b: number) -> number {
    return a + b;
}

let x: number = add(1, 2);
let arr: number[] = [1, 2, 3];
"#;
    let (_dir, path) = create_test_file("test.atl", source);

    assert_cmd::cargo::cargo_bin_cmd!("atlas")
        .arg("check")
        .arg(&path)
        .assert()
        .success()
        .stdout(predicate::str::contains("No errors found"));
}

// ============================================================================
// atlas check - Error Cases
// ============================================================================

#[test]
fn test_check_type_error() {
    let (_dir, path) = create_test_file("test.atl", r#"let x: number = "wrong";"#);

    assert_cmd::cargo::cargo_bin_cmd!("atlas")
        .arg("check")
        .arg(&path)
        .assert()
        .failure();
}

#[test]
fn test_check_parse_error() {
    let (_dir, path) = create_test_file("test.atl", "let x =");

    assert_cmd::cargo::cargo_bin_cmd!("atlas")
        .arg("check")
        .arg(&path)
        .assert()
        .failure();
}

#[test]
fn test_check_json_output() {
    let (_dir, path) = create_test_file("test.atl", "let x: number = 42;");

    assert_cmd::cargo::cargo_bin_cmd!("atlas")
        .arg("check")
        .arg(&path)
        .arg("--json")
        .assert()
        .success()
        .stdout(predicate::str::contains("No errors found"));
}

#[test]
fn test_check_json_output_with_error() {
    let (_dir, path) = create_test_file("test.atl", r#"let x: number = "wrong";"#);

    assert_cmd::cargo::cargo_bin_cmd!("atlas")
        .arg("check")
        .arg(&path)
        .arg("--json")
        .assert()
        .failure()
        .stdout(predicate::str::contains("diag_version"));
}

// ============================================================================
// Cross-command Integration Tests
// ============================================================================

#[test]
fn test_build_then_run_workflow() {
    // This test verifies that a built .atb file can be used
    // (though we don't have a command to run .atb files directly yet)
    let source = "let x: number = 42;";
    let temp_dir = create_test_project(source);
    let bytecode_path = temp_dir.path().join("target/debug/bin/test-project.atl.bc");

    // Build should succeed
    assert_cmd::cargo::cargo_bin_cmd!("atlas")
        .current_dir(temp_dir.path())
        .arg("build")
        .assert()
        .success();

    assert!(bytecode_path.exists());

    // Run should also succeed
    let source_path = temp_dir.path().join("src/main.atlas");
    assert_cmd::cargo::cargo_bin_cmd!("atlas")
        .arg("run")
        .arg(source_path.to_str().unwrap())
        .assert()
        .success();
}

#[test]
fn test_all_commands_handle_same_error() {
    let temp_dir = create_test_project(r#"let x: number = "wrong";"#);
    let source_path = temp_dir.path().join("src/main.atlas");

    // All commands should fail on the same type error
    assert_cmd::cargo::cargo_bin_cmd!("atlas")
        .arg("check")
        .arg(source_path.to_str().unwrap())
        .assert()
        .failure();

    assert_cmd::cargo::cargo_bin_cmd!("atlas")
        .current_dir(temp_dir.path())
        .arg("build")
        .assert()
        .failure();

    assert_cmd::cargo::cargo_bin_cmd!("atlas")
        .arg("run")
        .arg(source_path.to_str().unwrap())
        .assert()
        .failure();
}

#[test]
fn test_exit_code_consistency() {
    // Parse error
    let (_dir1, path1) = create_test_file("parse_error.atl", "let x =");
    let parse_err = assert_cmd::cargo::cargo_bin_cmd!("atlas")
        .arg("run")
        .arg(&path1)
        .output()
        .unwrap();
    assert!(!parse_err.status.success());

    // Type error
    let (_dir2, path2) = create_test_file("type_error.atl", r#"let x: number = "wrong";"#);
    let type_err = assert_cmd::cargo::cargo_bin_cmd!("atlas")
        .arg("run")
        .arg(&path2)
        .output()
        .unwrap();
    assert!(!type_err.status.success());

    // Both should have non-zero exit codes
    assert_eq!(parse_err.status.code(), type_err.status.code());
}
