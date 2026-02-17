//! Process management tests (Phase-12)
//!
//! Tests for command execution, environment variables, and process control.

use atlas_runtime::{Atlas, SecurityContext, Value};

/// Helper to evaluate code expecting success
fn eval_ok(code: &str) -> Value {
    let security = SecurityContext::allow_all();
    let runtime = Atlas::new_with_security(security);
    runtime.eval(code).unwrap()
}

// ============================================================================
// Command Execution Tests
// ============================================================================

#[test]
fn test_exec_simple_command() {
    // Test executing a simple command (echo on Unix, similar on Windows)
    let code = if cfg!(target_os = "windows") {
        r#"exec(["cmd", "/C", "echo", "hello"])"#
    } else {
        r#"exec(["echo", "hello"])"#
    };

    let result = eval_ok(code);
    // Should return Result<object, string>
    assert!(matches!(result, Value::Result(_)));
}

#[test]
fn test_shell_command() {
    let code = if cfg!(target_os = "windows") {
        r#"shell("echo hello")"#
    } else {
        r#"shell("echo hello")"#
    };

    let result = eval_ok(code);
    // Should return Result<object, string>
    assert!(matches!(result, Value::Result(_)));
}

// ============================================================================
// Environment Variable Tests
// ============================================================================

#[test]
fn test_set_get_env() {
    let code = r#"
        setEnv("TEST_VAR_ATLAS", "test_value");
        getEnv("TEST_VAR_ATLAS")
    "#;
    let result = eval_ok(code);
    match result {
        Value::String(s) => assert_eq!(&*s, "test_value"),
        other => panic!("Expected String, got {:?}", other),
    }
}

#[test]
fn test_get_env_nonexistent() {
    let code = r#"getEnv("NONEXISTENT_VAR_ATLAS_12345")"#;
    let result = eval_ok(code);
    assert!(matches!(result, Value::Null));
}

#[test]
fn test_unset_env() {
    let code = r#"
        setEnv("TEST_VAR_UNSET", "value");
        unsetEnv("TEST_VAR_UNSET");
        getEnv("TEST_VAR_UNSET")
    "#;
    let result = eval_ok(code);
    assert!(matches!(result, Value::Null));
}

#[test]
fn test_list_env() {
    let code = r#"listEnv()"#;
    let result = eval_ok(code);
    // Should return an object (JsonValue)
    assert!(matches!(result, Value::JsonValue(_)));
}

// ============================================================================
// Working Directory Tests
// ============================================================================

#[test]
fn test_get_cwd() {
    let code = r#"getCwd()"#;
    let result = eval_ok(code);
    // Should return a string
    assert!(matches!(result, Value::String(_)));
}

// ============================================================================
// Process Info Tests
// ============================================================================

#[test]
fn test_get_pid() {
    let code = r#"getPid()"#;
    let result = eval_ok(code);
    // Should return a number
    match result {
        Value::Number(n) => assert!(n > 0.0),
        other => panic!("Expected Number, got {:?}", other),
    }
}

// ============================================================================
// Security Tests
// ============================================================================

#[test]
fn test_exec_requires_permission() {
    let code = r#"exec("ls")"#;
    // Default context denies all
    let security = SecurityContext::new();
    let runtime = Atlas::new_with_security(security);
    let result = runtime.eval(code);
    // Should fail due to permission denial
    assert!(result.is_err());
}

#[test]
fn test_env_requires_permission() {
    let code = r#"getEnv("PATH")"#;
    // Default context denies all
    let security = SecurityContext::new();
    let runtime = Atlas::new_with_security(security);
    let result = runtime.eval(code);
    // Should fail due to permission denial
    assert!(result.is_err());
}
