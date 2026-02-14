//! VM tests for Option<T> and Result<T,E>
//!
//! BLOCKER 02-D: Built-in Generic Types
//!
//! These tests verify VM parity with interpreter for Option and Result support.
//! Tests mirror option_result_tests.rs to ensure identical behavior.

mod common;
use common::*;

// ============================================================================
// Option<T> Tests
// ============================================================================

#[test]
fn test_option_is_some() {
    assert_eval_bool("is_some(Some(42))", true);
    assert_eval_bool("is_some(None())", false);
}

#[test]
fn test_option_is_none() {
    assert_eval_bool("is_none(None())", true);
    assert_eval_bool("is_none(Some(42))", false);
}

#[test]
fn test_option_unwrap_number() {
    assert_eval_number("unwrap(Some(42))", 42.0);
}

#[test]
fn test_option_unwrap_string() {
    assert_eval_string(r#"unwrap(Some("hello"))"#, "hello");
}

#[test]
fn test_option_unwrap_bool() {
    assert_eval_bool("unwrap(Some(true))", true);
}

#[test]
fn test_option_unwrap_null() {
    assert_eval_null("unwrap(Some(null))");
}

#[test]
fn test_option_unwrap_or_some() {
    assert_eval_number("unwrap_or(Some(42), 0)", 42.0);
}

#[test]
fn test_option_unwrap_or_none() {
    assert_eval_number("unwrap_or(None(), 99)", 99.0);
}

#[test]
fn test_option_unwrap_or_string() {
    assert_eval_string(r#"unwrap_or(Some("hello"), "default")"#, "hello");
    assert_eval_string(r#"unwrap_or(None(), "default")"#, "default");
}

#[test]
fn test_option_nested() {
    assert_eval_number("unwrap(unwrap(Some(Some(42))))", 42.0);
}

// ============================================================================
// Result<T,E> Tests
// ============================================================================

#[test]
fn test_result_is_ok() {
    assert_eval_bool("is_ok(Ok(42))", true);
    assert_eval_bool(r#"is_ok(Err("failed"))"#, false);
}

#[test]
fn test_result_is_err() {
    assert_eval_bool(r#"is_err(Err("failed"))"#, true);
    assert_eval_bool("is_err(Ok(42))", false);
}

#[test]
fn test_result_unwrap_ok_number() {
    assert_eval_number("unwrap(Ok(42))", 42.0);
}

#[test]
fn test_result_unwrap_ok_string() {
    assert_eval_string(r#"unwrap(Ok("success"))"#, "success");
}

#[test]
fn test_result_unwrap_ok_null() {
    assert_eval_null("unwrap(Ok(null))");
}

#[test]
fn test_result_unwrap_or_ok() {
    assert_eval_number("unwrap_or(Ok(42), 0)", 42.0);
}

#[test]
fn test_result_unwrap_or_err() {
    assert_eval_number(r#"unwrap_or(Err("failed"), 99)"#, 99.0);
}

#[test]
fn test_result_unwrap_or_string() {
    assert_eval_string(r#"unwrap_or(Ok("success"), "default")"#, "success");
    assert_eval_string(r#"unwrap_or(Err(404), "default")"#, "default");
}

// ============================================================================
// Mixed Option/Result Tests
// ============================================================================

#[test]
fn test_option_and_result_together() {
    let code = r#"
        let opt = Some(42);
        let res = Ok(99);
        unwrap(opt) + unwrap(res)
    "#;
    assert_eval_number(code, 141.0);
}

#[test]
fn test_option_in_conditional() {
    let code = r#"
        let opt = Some(42);
        if (is_some(opt)) {
            unwrap(opt);
        } else {
            0;
        }
    "#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_result_in_conditional() {
    let code = r#"
        let res = Ok(42);
        if (is_ok(res)) {
            unwrap(res);
        } else {
            0;
        }
    "#;
    assert_eval_number(code, 42.0);
}

// ============================================================================
// Complex Tests
// ============================================================================

#[test]
fn test_option_chain() {
    let code = r#"
        let a = Some(10);
        let b = Some(20);
        let c = Some(30);
        unwrap(a) + unwrap(b) + unwrap(c)
    "#;
    assert_eval_number(code, 60.0);
}

#[test]
fn test_result_chain() {
    let code = r#"
        let a = Ok(10);
        let b = Ok(20);
        let c = Ok(30);
        unwrap(a) + unwrap(b) + unwrap(c)
    "#;
    assert_eval_number(code, 60.0);
}

#[test]
fn test_option_unwrap_or_with_none_chain() {
    let code = r#"
        let a = None();
        let b = None();
        unwrap_or(a, 5) + unwrap_or(b, 10)
    "#;
    assert_eval_number(code, 15.0);
}

#[test]
fn test_result_unwrap_or_with_err_chain() {
    let code = r#"
        let a = Err("fail1");
        let b = Err("fail2");
        unwrap_or(a, 5) + unwrap_or(b, 10)
    "#;
    assert_eval_number(code, 15.0);
}
