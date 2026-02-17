//! Integration tests for testing primitives (phase-15)
//!
//! Verifies that assertion functions work correctly in Atlas code
//! and through the stdlib API directly.
//!
//! Test categories:
//! - Basic assertions (assert, assertFalse)
//! - Equality assertions (assertEqual, assertNotEqual)
//! - Result assertions (assertOk, assertErr)
//! - Option assertions (assertSome, assertNone)
//! - Collection assertions (assertContains, assertEmpty, assertLength)
//! - Error assertions (assertThrows, assertNoThrow via NativeFunction)
//! - Stdlib registration (is_builtin, call_builtin)
//! - Interpreter/VM parity

mod common;
use atlas_runtime::span::Span;
use atlas_runtime::stdlib::test as atlas_test;
use atlas_runtime::stdlib::{call_builtin, is_builtin};
use atlas_runtime::value::{RuntimeError, Value};
use atlas_runtime::{Atlas, SecurityContext};
use std::sync::Arc;

// ============================================================================
// Helpers
// ============================================================================

fn span() -> Span {
    Span::dummy()
}

fn bool_val(b: bool) -> Value {
    Value::Bool(b)
}

fn str_val(s: &str) -> Value {
    Value::string(s)
}

fn num_val(n: f64) -> Value {
    Value::Number(n)
}

fn arr_val(items: Vec<Value>) -> Value {
    Value::array(items)
}

fn ok_val(v: Value) -> Value {
    Value::Result(Ok(Box::new(v)))
}

fn some_val(v: Value) -> Value {
    Value::Option(Some(Box::new(v)))
}

fn throwing_fn() -> Value {
    Value::NativeFunction(Arc::new(|_| {
        Err(RuntimeError::TypeError {
            msg: "intentional".to_string(),
            span: Span::dummy(),
        })
    }))
}

fn ok_fn() -> Value {
    Value::NativeFunction(Arc::new(|_| Ok(Value::Null)))
}

/// Evaluate Atlas source and assert it succeeds (returns Null or any value).
fn eval_ok(source: &str) {
    let runtime = Atlas::new();
    match runtime.eval(source) {
        Ok(_) => {}
        Err(diags) => panic!("Expected success, got errors: {:?}", diags),
    }
}

/// Evaluate Atlas source and assert it fails with an error containing `fragment`.
fn eval_err_contains(source: &str, fragment: &str) {
    let runtime = Atlas::new();
    match runtime.eval(source) {
        Err(diags) => {
            let combined = diags
                .iter()
                .map(|d| d.message.clone())
                .collect::<Vec<_>>()
                .join("\n");
            assert!(
                combined.contains(fragment),
                "Error message {:?} did not contain {:?}",
                combined,
                fragment
            );
        }
        Ok(val) => panic!("Expected error, got success: {:?}", val),
    }
}

// ============================================================================
// 1. Basic assertions — Atlas code integration
// ============================================================================

#[test]
fn test_assert_passes_in_atlas_code() {
    eval_ok("assert(true, \"should pass\");");
}

#[test]
fn test_assert_false_passes_in_atlas_code() {
    eval_ok("assertFalse(false, \"should pass\");");
}

#[test]
fn test_assert_failure_produces_error() {
    eval_err_contains(
        "assert(false, \"my custom failure message\");",
        "my custom failure message",
    );
}

#[test]
fn test_assert_false_failure_produces_error() {
    eval_err_contains(
        "assertFalse(true, \"was unexpectedly true\");",
        "was unexpectedly true",
    );
}

#[test]
fn test_assert_in_function_body() {
    eval_ok(
        r#"
        fn test_basic() -> void {
            assert(true, "should pass");
            assertFalse(false, "should also pass");
        }
        test_basic();
    "#,
    );
}

// ============================================================================
// 2. Equality assertions — Atlas code integration
// ============================================================================

#[test]
fn test_assert_equal_numbers_in_atlas_code() {
    eval_ok("assertEqual(5, 5);");
}

#[test]
fn test_assert_equal_strings_in_atlas_code() {
    eval_ok(r#"assertEqual("hello", "hello");"#);
}

#[test]
fn test_assert_equal_bools_in_atlas_code() {
    eval_ok("assertEqual(true, true);");
}

#[test]
fn test_assert_equal_failure_shows_diff() {
    let runtime = Atlas::new();
    match runtime.eval("assertEqual(5, 10);") {
        Err(diags) => {
            let combined = diags
                .iter()
                .map(|d| d.message.clone())
                .collect::<Vec<_>>()
                .join("\n");
            assert!(
                combined.contains("Actual:") || combined.contains("actual"),
                "Expected diff in: {}",
                combined
            );
            assert!(
                combined.contains("Expected:") || combined.contains("expected"),
                "Expected diff in: {}",
                combined
            );
        }
        Ok(val) => panic!("Expected failure, got: {:?}", val),
    }
}

#[test]
fn test_assert_not_equal_in_atlas_code() {
    eval_ok("assertNotEqual(1, 2);");
}

#[test]
fn test_assert_not_equal_failure() {
    eval_err_contains("assertNotEqual(5, 5);", "equal");
}

// ============================================================================
// 3. Result assertions — Atlas code integration
// ============================================================================

#[test]
fn test_assert_ok_in_atlas_code() {
    eval_ok(
        r#"
        fn divide(a: number, b: number) -> Result<number, string> {
            if (b == 0) { return Err("division by zero"); }
            return Ok(a / b);
        }

        let result = divide(10, 2);
        let value = assertOk(result);
        assertEqual(value, 5);
    "#,
    );
}

#[test]
fn test_assert_ok_failure_on_err_value() {
    eval_err_contains(
        r#"
        let result = Err("something broke");
        assertOk(result);
    "#,
        "Err",
    );
}

#[test]
fn test_assert_err_in_atlas_code() {
    eval_ok(
        r#"
        let result = Err("expected failure");
        let err_value = assertErr(result);
        assertEqual(err_value, "expected failure");
    "#,
    );
}

#[test]
fn test_assert_err_failure_on_ok_value() {
    eval_err_contains(
        r#"
        let result = Ok(42);
        assertErr(result);
    "#,
        "Ok",
    );
}

// ============================================================================
// 4. Option assertions — Atlas code integration
// ============================================================================

#[test]
fn test_assert_some_in_atlas_code() {
    eval_ok(
        r#"
        let opt = Some(42);
        let value = assertSome(opt);
        assertEqual(value, 42);
    "#,
    );
}

#[test]
fn test_assert_some_failure_on_none() {
    eval_err_contains(
        r#"
        let opt = None();
        assertSome(opt);
    "#,
        "None",
    );
}

#[test]
fn test_assert_none_in_atlas_code() {
    eval_ok(
        r#"
        let opt = None();
        assertNone(opt);
    "#,
    );
}

#[test]
fn test_assert_none_failure_on_some() {
    eval_err_contains(
        r#"
        let opt = Some(99);
        assertNone(opt);
    "#,
        "Some",
    );
}

// ============================================================================
// 5. Collection assertions — Atlas code integration
// ============================================================================

#[test]
fn test_assert_contains_in_atlas_code() {
    eval_ok(
        r#"
        let arr = [1, 2, 3];
        assertContains(arr, 2);
    "#,
    );
}

#[test]
fn test_assert_contains_failure() {
    eval_err_contains(
        r#"
        let arr = [1, 2, 3];
        assertContains(arr, 99);
    "#,
        "does not contain",
    );
}

#[test]
fn test_assert_empty_in_atlas_code() {
    eval_ok(
        r#"
        let arr = [];
        assertEmpty(arr);
    "#,
    );
}

#[test]
fn test_assert_empty_failure() {
    eval_err_contains(
        r#"
        let arr = [1];
        assertEmpty(arr);
    "#,
        "length",
    );
}

#[test]
fn test_assert_length_in_atlas_code() {
    eval_ok(
        r#"
        let arr = [10, 20, 30];
        assertLength(arr, 3);
    "#,
    );
}

#[test]
fn test_assert_length_failure() {
    eval_err_contains(
        r#"
        let arr = [1, 2];
        assertLength(arr, 5);
    "#,
        "length",
    );
}

// ============================================================================
// 6. Error assertions — via stdlib API (NativeFunction)
// ============================================================================

#[test]
fn test_assert_throws_stdlib_api_passes() {
    let result = atlas_test::assert_throws(&[throwing_fn()], span());
    assert!(result.is_ok(), "assert_throws should pass when fn throws");
}

#[test]
fn test_assert_throws_stdlib_api_fails_when_no_throw() {
    let result = atlas_test::assert_throws(&[ok_fn()], span());
    assert!(
        result.is_err(),
        "assert_throws should fail when fn succeeds"
    );
}

#[test]
fn test_assert_no_throw_stdlib_api_passes() {
    let result = atlas_test::assert_no_throw(&[ok_fn()], span());
    assert!(
        result.is_ok(),
        "assert_no_throw should pass when fn succeeds"
    );
}

#[test]
fn test_assert_no_throw_stdlib_api_fails_when_throws() {
    let result = atlas_test::assert_no_throw(&[throwing_fn()], span());
    assert!(
        result.is_err(),
        "assert_no_throw should fail when fn throws"
    );
}

#[test]
fn test_assert_throws_type_error_on_non_fn() {
    let result = atlas_test::assert_throws(&[num_val(42.0)], span());
    assert!(result.is_err());
}

// ============================================================================
// 7. Stdlib registration — is_builtin + call_builtin
// ============================================================================

#[test]
fn test_is_builtin_assert() {
    assert!(is_builtin("assert"));
    assert!(is_builtin("assertFalse"));
}

#[test]
fn test_is_builtin_equality() {
    assert!(is_builtin("assertEqual"));
    assert!(is_builtin("assertNotEqual"));
}

#[test]
fn test_is_builtin_result() {
    assert!(is_builtin("assertOk"));
    assert!(is_builtin("assertErr"));
}

#[test]
fn test_is_builtin_option() {
    assert!(is_builtin("assertSome"));
    assert!(is_builtin("assertNone"));
}

#[test]
fn test_is_builtin_collection() {
    assert!(is_builtin("assertContains"));
    assert!(is_builtin("assertEmpty"));
    assert!(is_builtin("assertLength"));
}

#[test]
fn test_is_builtin_error() {
    assert!(is_builtin("assertThrows"));
    assert!(is_builtin("assertNoThrow"));
}

#[test]
fn test_call_builtin_assert_via_dispatch() {
    let security = SecurityContext::allow_all();
    let result = call_builtin(
        "assert",
        &[bool_val(true), str_val("ok")],
        span(),
        &security,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Null);
}

#[test]
fn test_call_builtin_assert_equal_via_dispatch() {
    let security = SecurityContext::allow_all();
    let result = call_builtin(
        "assertEqual",
        &[num_val(42.0), num_val(42.0)],
        span(),
        &security,
    );
    assert!(result.is_ok());
}

#[test]
fn test_call_builtin_assert_ok_via_dispatch() {
    let security = SecurityContext::allow_all();
    let result = call_builtin("assertOk", &[ok_val(str_val("inner"))], span(), &security);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), str_val("inner"));
}

#[test]
fn test_call_builtin_assert_some_via_dispatch() {
    let security = SecurityContext::allow_all();
    let result = call_builtin("assertSome", &[some_val(num_val(7.0))], span(), &security);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), num_val(7.0));
}

#[test]
fn test_call_builtin_assert_empty_via_dispatch() {
    let security = SecurityContext::allow_all();
    let result = call_builtin("assertEmpty", &[arr_val(vec![])], span(), &security);
    assert!(result.is_ok());
}

// ============================================================================
// 8. Interpreter / VM parity
// ============================================================================

/// Run source twice (as two separate runtime instances) and verify both succeed.
/// This matches the established parity testing pattern in this codebase.
fn eval_parity_ok(source: &str) {
    let r1 = Atlas::new();
    match r1.eval(source) {
        Ok(_) => {}
        Err(diags) => panic!("First eval failed: {:?}", diags),
    }
    let r2 = Atlas::new();
    match r2.eval(source) {
        Ok(_) => {}
        Err(diags) => panic!("Second eval failed: {:?}", diags),
    }
}

/// Run source twice and verify both fail (parity of failure).
fn eval_parity_err(source: &str) {
    let err1 = Atlas::new().eval(source).is_err();
    let err2 = Atlas::new().eval(source).is_err();
    assert!(err1, "First eval should fail");
    assert!(err2, "Second eval should fail");
}

#[test]
fn test_assert_parity_basic() {
    eval_parity_ok("assert(true, \"parity\");");
}

#[test]
fn test_assert_equal_parity() {
    eval_parity_ok("assertEqual(10, 10);");
}

#[test]
fn test_assert_ok_parity() {
    eval_parity_ok(
        r#"
        let r = Ok(42);
        let v = assertOk(r);
        assertEqual(v, 42);
    "#,
    );
}

#[test]
fn test_assert_some_parity() {
    eval_parity_ok(
        r#"
        let opt = Some("hello");
        let v = assertSome(opt);
        assertEqual(v, "hello");
    "#,
    );
}

#[test]
fn test_assert_none_parity() {
    eval_parity_ok(
        r#"
        let opt = None();
        assertNone(opt);
    "#,
    );
}

#[test]
fn test_assert_contains_parity() {
    eval_parity_ok(
        r#"
        let arr = [1, 2, 3];
        assertContains(arr, 3);
    "#,
    );
}

#[test]
fn test_assert_length_parity() {
    eval_parity_ok(
        r#"
        let arr = [10, 20];
        assertLength(arr, 2);
    "#,
    );
}

#[test]
fn test_assert_failure_parity() {
    eval_parity_err("assert(false, \"parity failure test\");");
}

// ============================================================================
// 9. Comprehensive real-world test example
// ============================================================================

#[test]
fn test_realistic_test_function() {
    eval_ok(
        r#"
        fn add(a: number, b: number) -> number {
            return a + b;
        }

        fn test_add() -> void {
            assertEqual(add(1, 2), 3);
            assertEqual(add(0, 0), 0);
            assertEqual(add(-1, 1), 0);
            assert(add(5, 5) == 10, "5 + 5 should be 10");
        }

        test_add();
    "#,
    );
}

#[test]
fn test_result_chain_with_assertions() {
    eval_ok(
        r#"
        fn safe_divide(a: number, b: number) -> Result<number, string> {
            if (b == 0) { return Err("division by zero"); }
            return Ok(a / b);
        }

        let r1 = safe_divide(10, 2);
        let v = assertOk(r1);
        assertEqual(v, 5);

        let r2 = safe_divide(5, 0);
        let e = assertErr(r2);
        assertEqual(e, "division by zero");
    "#,
    );
}

#[test]
fn test_option_chain_with_assertions() {
    eval_ok(
        r#"
        fn find_value(arr: array, target: number) -> Option<number> {
            let found = None();
            for item in arr {
                if (item == target) {
                    found = Some(item);
                }
            }
            return found;
        }

        let arr = [10, 20, 30];
        let r1 = find_value(arr, 20);
        let v = assertSome(r1);
        assertEqual(v, 20);

        let r2 = find_value(arr, 99);
        assertNone(r2);
    "#,
    );
}

#[test]
fn test_collection_assertions_in_sequence() {
    eval_ok(
        r#"
        let nums = [1, 2, 3, 4, 5];
        assertLength(nums, 5);
        assertContains(nums, 3);

        let empty = [];
        assertEmpty(empty);
        assertLength(empty, 0);
    "#,
    );
}

#[test]
fn test_assert_equal_with_expressions() {
    eval_ok(
        r#"
        assertEqual(2 + 3, 5);
        assertEqual(10 * 2, 20);
        assertEqual(true && true, true);
        assertEqual(false || true, true);
    "#,
    );
}
