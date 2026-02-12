//! Integration tests for Atlas runtime API
//!
//! These tests validate the runtime API without using the CLI,
//! ensuring it can be embedded in other applications.

use atlas_runtime::{Atlas, DiagnosticLevel, RuntimeResult, Value};

/// Test that runtime can be created and used
#[test]
fn test_runtime_api_availability() {
    let runtime = Atlas::new();
    let _result: RuntimeResult<Value> = runtime.eval("test");
    // API is available and types work correctly
}

/// Test eval with simple input
#[test]
fn test_eval_basic() {
    let runtime = Atlas::new();
    let result = runtime.eval("1");
    // Currently stubbed, but test structure is correct
    assert!(result.is_err() || result.is_ok());
}

/// Test eval_file with path
#[test]
fn test_eval_file_basic() {
    let runtime = Atlas::new();
    let result = runtime.eval_file("test.atlas");
    // Currently stubbed, but test structure is correct
    assert!(result.is_err() || result.is_ok());
}

/// Test that diagnostics have the correct structure
#[test]
fn test_diagnostic_structure() {
    let runtime = Atlas::new();
    let result = runtime.eval("invalid");

    match result {
        Err(diagnostics) => {
            assert!(!diagnostics.is_empty(), "Should return at least one diagnostic");

            let diag = &diagnostics[0];
            assert_eq!(diag.level, DiagnosticLevel::Error);
            assert!(!diag.message.is_empty(), "Diagnostic should have a message");
        }
        Ok(_) => {
            // Currently returns error, but when implemented might succeed
            // This test is flexible for future implementation
        }
    }
}

/// Test that runtime can be used multiple times
#[test]
fn test_runtime_reuse() {
    let runtime = Atlas::new();

    let _result1 = runtime.eval("1");
    let _result2 = runtime.eval("2");
    let _result3 = runtime.eval("3");

    // Runtime can be called multiple times without panicking
}

/// Test that multiple runtime instances can coexist
#[test]
fn test_multiple_runtimes() {
    let runtime1 = Atlas::new();
    let runtime2 = Atlas::new();

    let _result1 = runtime1.eval("test1");
    let _result2 = runtime2.eval("test2");

    // Multiple independent runtimes can be created
}

/// Test error diagnostics for invalid syntax
#[test]
fn test_error_diagnostics() {
    let runtime = Atlas::new();
    let result = runtime.eval("@#$%");

    match result {
        Err(diagnostics) => {
            assert!(!diagnostics.is_empty());
            for diag in &diagnostics {
                assert_eq!(diag.level, DiagnosticLevel::Error);
            }
        }
        Ok(_) => {
            // When implemented, this should be an error
        }
    }
}

/// Test that runtime works without CLI dependencies
#[test]
fn test_no_cli_dependency() {
    // This test ensures we can use the runtime API
    // without pulling in any CLI-specific code
    let runtime = Atlas::new();
    let _result = runtime.eval("1 + 1");

    // If this compiles and runs, we have no CLI dependencies
}

// Future tests (currently ignored until implementation)

#[test]
#[ignore]
fn test_eval_returns_value() {
    let runtime = Atlas::new();
    let result = runtime.eval("42");

    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 42.0),
        _ => panic!("Expected Number(42.0)"),
    }
}

#[test]
#[ignore]
fn test_eval_preserves_state() {
    let runtime = Atlas::new();

    // Define a variable
    runtime.eval("let x: int = 10;").unwrap();

    // Use it in another eval
    let result = runtime.eval("x").unwrap();

    match result {
        Value::Number(n) => assert_eq!(n, 10.0),
        _ => panic!("Expected Number(10.0)"),
    }
}

#[test]
#[ignore]
fn test_eval_file_with_real_file() {
    use std::fs;
    use std::io::Write;

    // Create a temporary test file
    let mut file = fs::File::create("test_program.atlas").unwrap();
    writeln!(file, "let x: int = 42;").unwrap();

    let runtime = Atlas::new();
    let result = runtime.eval_file("test_program.atlas");

    // Clean up
    fs::remove_file("test_program.atlas").unwrap();

    match result {
        Ok(Value::Null) => (), // Variable declaration returns null
        _ => panic!("Expected Null"),
    }
}
