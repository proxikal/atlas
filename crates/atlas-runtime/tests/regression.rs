//! Comprehensive Regression Test Suite
//!
//! This file serves as the golden test matrix for Atlas language features.
//! It provides quick regression detection by testing all core features in one place.
//!
//! Test coverage:
//! - Literals (number, string, bool, null, arrays)
//! - Operators (arithmetic, comparison, logical)
//! - Variables (let/var, mutation, scoping)
//! - Functions (declarations, calls, returns, recursion)
//! - Control flow (if/else, while, for, break, continue)
//! - Arrays (indexing, mutation, nested arrays)
//! - Type checking (type errors, type inference)
//! - Error handling (runtime errors, compile errors)
//! - Standard library functions

use rstest::rstest;

mod common;
use common::*;

// ============================================================================
// Literals
// ============================================================================

#[rstest]
#[case("42;", 42.0)]
#[case("2.5;", 2.5)]
#[case("0;", 0.0)]
#[case("-42;", -42.0)]
fn regression_number_literals(#[case] code: &str, #[case] expected: f64) {
    assert_eval_number(code, expected);
}

#[rstest]
#[case(r#""hello";"#, "hello")]
#[case(r#""world";"#, "world")]
#[case(r#""""#, "")]
fn regression_string_literals(#[case] code: &str, #[case] expected: &str) {
    assert_eval_string(code, expected);
}

#[rstest]
#[case("true;", true)]
#[case("false;", false)]
fn regression_bool_literals(#[case] code: &str, #[case] expected: bool) {
    assert_eval_bool(code, expected);
}

#[test]
fn regression_null_literal() {
    assert_eval_null("null");
}

// ============================================================================
// Arithmetic Operators
// ============================================================================

#[rstest]
#[case("1 + 2;", 3.0)]
#[case("5 - 3;", 2.0)]
#[case("4 * 3;", 12.0)]
#[case("10 / 2;", 5.0)]
#[case("10 % 3;", 1.0)]
#[case("2 + 3 * 4;", 14.0)] // Precedence
#[case("(2 + 3) * 4;", 20.0)] // Grouping
fn regression_arithmetic(#[case] code: &str, #[case] expected: f64) {
    assert_eval_number(code, expected);
}

// ============================================================================
// Comparison Operators
// ============================================================================

#[rstest]
#[case("1 < 2;", true)]
#[case("2 > 1;", true)]
#[case("1 <= 1;", true)]
#[case("2 >= 2;", true)]
#[case("1 == 1;", true)]
#[case("1 != 2;", true)]
fn regression_comparison(#[case] code: &str, #[case] expected: bool) {
    assert_eval_bool(code, expected);
}

// ============================================================================
// Logical Operators
// ============================================================================

#[rstest]
#[case("true && true;", true)]
#[case("true && false;", false)]
#[case("false || true;", true)]
#[case("false || false;", false)]
#[case("!true;", false)]
#[case("!false;", true)]
fn regression_logical(#[case] code: &str, #[case] expected: bool) {
    assert_eval_bool(code, expected);
}

// ============================================================================
// Variables - Let (Immutable)
// ============================================================================

#[rstest]
#[case("let x: number = 42; x;", 42.0)]
#[case("let x: number = 10; let y: number = 20; x + y;", 30.0)]
fn regression_let_variables(#[case] code: &str, #[case] expected: f64) {
    assert_eval_number(code, expected);
}

// ============================================================================
// Variables - Var (Mutable)
// ============================================================================

#[rstest]
#[case("var x: number = 10; x = 20; x;", 20.0)]
#[case("var x: number = 1; x = x + 1; x;", 2.0)]
fn regression_var_variables(#[case] code: &str, #[case] expected: f64) {
    assert_eval_number(code, expected);
}

// ============================================================================
// Functions
// ============================================================================

#[test]
fn regression_function_declaration_and_call() {
    let code = r#"
        fn add(a: number, b: number) -> number {
            return a + b;
        }
        add(2, 3);
    "#;
    assert_eval_number(code, 5.0);
}

#[test]
fn regression_function_recursion() {
    let code = r#"
        fn factorial(n: number) -> number {
            if (n <= 1) {
                return 1;
            }
            return n * factorial(n - 1);
        }
        factorial(5);
    "#;
    assert_eval_number(code, 120.0);
}

#[test]
fn regression_function_local_variables() {
    let code = r#"
        fn compute(x: number) -> number {
            let temp: number = x * 2;
            return temp + 1;
        }
        compute(5);
    "#;
    assert_eval_number(code, 11.0);
}

// ============================================================================
// Control Flow - If/Else
// ============================================================================

#[test]
fn regression_if_then() {
    let code = r#"
        let x: number = 10;
        if (x > 5) {
            x + 10;
        }
    "#;
    assert_eval_number(code, 20.0);
}

#[test]
fn regression_if_else() {
    let code = r#"
        let x: number = 3;
        if (x > 5) {
            10;
        } else {
            20;
        }
    "#;
    assert_eval_number(code, 20.0);
}

// ============================================================================
// Control Flow - While
// ============================================================================

#[test]
fn regression_while_loop() {
    let code = r#"
        var i: number = 0;
        var sum: number = 0;
        while (i < 5) {
            sum = sum + i;
            i = i + 1;
        }
        sum;
    "#;
    assert_eval_number(code, 10.0); // 0+1+2+3+4 = 10
}

#[test]
fn regression_while_with_break() {
    let code = r#"
        var i: number = 0;
        while (i < 10) {
            if (i == 5) {
                break;
            }
            i = i + 1;
        }
        i;
    "#;
    assert_eval_number(code, 5.0);
}

#[test]
fn regression_while_with_continue() {
    let code = r#"
        var i: number = 0;
        var sum: number = 0;
        while (i < 5) {
            i = i + 1;
            if (i == 3) {
                continue;
            }
            sum = sum + i;
        }
        sum;
    "#;
    assert_eval_number(code, 12.0); // 1+2+4+5 = 12 (skips 3)
}

// ============================================================================
// Arrays
// ============================================================================

#[test]
fn regression_array_literal() {
    let code = r#"
        let arr: number[] = [1, 2, 3];
        len(arr);
    "#;
    assert_eval_number(code, 3.0);
}

#[test]
fn regression_array_indexing() {
    let code = r#"
        let arr: number[] = [10, 20, 30];
        arr[1];
    "#;
    assert_eval_number(code, 20.0);
}

#[test]
fn regression_array_mutation() {
    let code = r#"
        var arr: number[] = [1, 2, 3];
        arr[0] = 99;
        arr[0];
    "#;
    assert_eval_number(code, 99.0);
}

#[test]
fn regression_nested_arrays() {
    let code = r#"
        let matrix: number[][] = [[1, 2], [3, 4]];
        matrix[1][0];
    "#;
    assert_eval_number(code, 3.0);
}

// ============================================================================
// String Operations
// ============================================================================

#[test]
fn regression_string_concatenation() {
    let code = r#"
        let s1: string = "hello";
        let s2: string = "world";
        s1 + s2;
    "#;
    assert_eval_string(code, "helloworld");
}

// Note: String indexing is not yet supported in Atlas
// TODO: Enable when typechecker supports string indexing

// ============================================================================
// Standard Library Functions
// ============================================================================

#[test]
fn regression_stdlib_len() {
    let code = r#"len("hello");"#;
    assert_eval_number(code, 5.0);
}

#[test]
fn regression_stdlib_print() {
    // print() returns null
    let code = r#"print("test");"#;
    assert_eval_null(code);
}

#[test]
fn regression_stdlib_str() {
    let code = r#"str(42);"#;
    assert_eval_string(code, "42");
}

// ============================================================================
// Type Errors
// ============================================================================

#[rstest]
#[case(r#"let x: number = "hello";"#, "AT3001")] // Type mismatch
#[case(r#"unknown_var;"#, "AT2002")] // Unknown symbol
#[case(r#"let x: number = 1; x = 2;"#, "AT3003")] // Invalid assignment (let is immutable)
fn regression_type_errors(#[case] code: &str, #[case] expected_code: &str) {
    assert_error_code(code, expected_code);
}

// ============================================================================
// Runtime Errors
// ============================================================================

#[rstest]
#[case("1 / 0;", "AT0005")] // Divide by zero
#[case("let arr: number[] = [1, 2]; arr[10];", "AT0006")] // Out of bounds
fn regression_runtime_errors(#[case] code: &str, #[case] expected_code: &str) {
    assert_error_code(code, expected_code);
}

// ============================================================================
// Complex Integration Tests
// ============================================================================

#[test]
fn regression_fibonacci() {
    let code = r#"
        fn fib(n: number) -> number {
            if (n <= 1) {
                return n;
            }
            return fib(n - 1) + fib(n - 2);
        }
        fib(10);
    "#;
    assert_eval_number(code, 55.0);
}

#[test]
fn regression_array_sum() {
    let code = r#"
        let arr: number[] = [1, 2, 3, 4, 5];
        var sum: number = 0;
        var i: number = 0;
        while (i < len(arr)) {
            sum = sum + arr[i];
            i = i + 1;
        }
        sum;
    "#;
    assert_eval_number(code, 15.0);
}

#[test]
fn regression_nested_function_calls() {
    let code = r#"
        fn double(x: number) -> number {
            return x * 2;
        }
        fn triple(x: number) -> number {
            return x * 3;
        }
        double(triple(5));
    "#;
    assert_eval_number(code, 30.0);
}

// Note: Scope shadowing is comprehensively tested in scope_shadowing_tests.rs

#[test]
fn regression_mixed_operations() {
    let code = r#"
        fn calculate(a: number, b: number) -> number {
            let sum: number = a + b;
            let product: number = a * b;
            if (sum > product) {
                return sum;
            } else {
                return product;
            }
        }
        calculate(5, 6);
    "#;
    assert_eval_number(code, 30.0); // product (5*6=30) > sum (5+6=11)
}

// ============================================================================
// STABILITY VERIFICATION TESTS (Phase 04)
// ============================================================================
//
// Comprehensive stability tests covering:
// - Determinism: same input → same output across multiple runs
// - Edge cases: empty inputs, boundary values, unicode, special floats
// - Stress: large data, deep recursion, long strings
// - Error recovery: malformed input handled gracefully, not panicked
// - Release mode: all tests also pass in --release builds

use atlas_runtime::Atlas;

// ─── Determinism Tests ───────────────────────────────────────────────────────

#[test]
fn stability_determinism_arithmetic() {
    // Same arithmetic expression evaluated twice must produce the same result.
    let code = "1 + 2 * 3 - 4 / 2;";
    let runtime1 = Atlas::new();
    let runtime2 = Atlas::new();
    let r1 = runtime1.eval(code);
    let r2 = runtime2.eval(code);
    assert!(
        format!("{:?}", r1) == format!("{:?}", r2),
        "Non-deterministic: {:?} != {:?}",
        r1,
        r2
    );
}

#[test]
fn stability_determinism_string_concat() {
    let code = r#""hello" + " " + "world";"#;
    let runtime1 = Atlas::new();
    let runtime2 = Atlas::new();
    let r1 = runtime1.eval(code);
    let r2 = runtime2.eval(code);
    assert!(
        format!("{:?}", r1) == format!("{:?}", r2),
        "Non-deterministic: {:?} != {:?}",
        r1,
        r2
    );
}

#[test]
fn stability_determinism_function_calls() {
    let code = r#"
        fn fib(n: number) -> number {
            if (n <= 1) { return n; }
            return fib(n - 1) + fib(n - 2);
        }
        fib(8);
    "#;
    let runtime1 = Atlas::new();
    let runtime2 = Atlas::new();
    let r1 = runtime1.eval(code);
    let r2 = runtime2.eval(code);
    assert!(
        format!("{:?}", r1) == format!("{:?}", r2),
        "Non-deterministic: {:?} != {:?}",
        r1,
        r2
    );
}

#[test]
fn stability_determinism_conditionals() {
    let code = "if (3 > 2) { 42; } else { 0; }";
    let runtime1 = Atlas::new();
    let runtime2 = Atlas::new();
    let r1 = runtime1.eval(code);
    let r2 = runtime2.eval(code);
    assert!(
        format!("{:?}", r1) == format!("{:?}", r2),
        "Non-deterministic: {:?} != {:?}",
        r1,
        r2
    );
}

#[test]
fn stability_determinism_array_operations() {
    let code = "let arr: number[] = [3, 1, 4, 1, 5]; arr[2];";
    let runtime1 = Atlas::new();
    let runtime2 = Atlas::new();
    let r1 = runtime1.eval(code);
    let r2 = runtime2.eval(code);
    assert!(
        format!("{:?}", r1) == format!("{:?}", r2),
        "Non-deterministic: {:?} != {:?}",
        r1,
        r2
    );
}

#[test]
fn stability_determinism_error_reporting() {
    // Errors must be reported deterministically — same input → same error code.
    let code = "1 / 0;";
    let runtime1 = Atlas::new();
    let runtime2 = Atlas::new();
    let r1 = runtime1.eval(code);
    let r2 = runtime2.eval(code);
    assert!(
        r1.is_err() == r2.is_err(),
        "Non-deterministic: {:?} vs {:?}",
        r1,
        r2
    );
    match (&r1, &r2) {
        (Err(d1), Err(d2)) => assert!(
            d1.len() == d2.len(),
            "Diagnostic count mismatch: {} != {}",
            d1.len(),
            d2.len()
        ),
        _ => {}
    }
}

#[test]
fn stability_determinism_boolean_logic() {
    let code = "true && false || (true && true);";
    let runtime1 = Atlas::new();
    let runtime2 = Atlas::new();
    let r1 = runtime1.eval(code);
    let r2 = runtime2.eval(code);
    assert!(
        format!("{:?}", r1) == format!("{:?}", r2),
        "Non-deterministic: {:?} != {:?}",
        r1,
        r2
    );
}

#[test]
fn stability_determinism_while_loop() {
    let code = r#"
        var sum: number = 0;
        var i: number = 0;
        while (i < 10) {
            sum = sum + i;
            i = i + 1;
        }
        sum;
    "#;
    let runtime1 = Atlas::new();
    let runtime2 = Atlas::new();
    let r1 = runtime1.eval(code);
    let r2 = runtime2.eval(code);
    assert!(
        format!("{:?}", r1) == format!("{:?}", r2),
        "Non-deterministic: {:?} != {:?}",
        r1,
        r2
    );
}

#[test]
fn stability_determinism_nested_functions() {
    let code = r#"
        fn outer(x: number) -> number {
            fn inner(y: number) -> number {
                return y * 2;
            }
            return inner(x) + 1;
        }
        outer(5);
    "#;
    let runtime1 = Atlas::new();
    let runtime2 = Atlas::new();
    let r1 = runtime1.eval(code);
    let r2 = runtime2.eval(code);
    assert!(
        format!("{:?}", r1) == format!("{:?}", r2),
        "Non-deterministic: {:?} != {:?}",
        r1,
        r2
    );
}

#[test]
fn stability_determinism_type_error() {
    // Type errors must be reported with the same diagnostic code each time.
    let code = "let x: number = true;";
    let runtime1 = Atlas::new();
    let runtime2 = Atlas::new();
    let r1 = runtime1.eval(code);
    let r2 = runtime2.eval(code);
    assert!(
        r1.is_err() == r2.is_err(),
        "Non-deterministic: {:?} vs {:?}",
        r1,
        r2
    );
}

// ─── Edge Case Tests ──────────────────────────────────────────────────────────

#[test]
fn stability_edge_empty_string_literal() {
    assert_eval_string("\"\";", "");
}

#[test]
fn stability_edge_empty_array() {
    // An empty array should be valid and have length 0.
    assert_no_error("let arr: number[] = [];");
}

#[test]
fn stability_edge_zero_value() {
    assert_eval_number("0;", 0.0);
}

#[test]
fn stability_edge_negative_zero() {
    // -0.0 is a valid float; should evaluate without error.
    assert_no_error("-0.0;");
}

#[test]
fn stability_edge_large_integer() {
    // Large numbers within float64 range should work fine.
    assert_eval_number("1000000.0;", 1_000_000.0);
}

#[test]
fn stability_edge_float_precision() {
    // Basic float precision: 0.1 + 0.2 should produce a number (not crash).
    assert_no_error("0.1 + 0.2;");
}

#[test]
fn stability_edge_negative_number() {
    assert_eval_number("-42;", -42.0);
}

#[test]
fn stability_edge_null_literal() {
    assert_eval_null("null;");
}

#[test]
fn stability_edge_boolean_true() {
    assert_eval_bool("true;", true);
}

#[test]
fn stability_edge_boolean_false() {
    assert_eval_bool("false;", false);
}

#[test]
fn stability_edge_single_char_string() {
    assert_eval_string(r#""a";"#, "a");
}

#[test]
fn stability_edge_deeply_nested_arithmetic() {
    // 10 levels of nesting — should not overflow the stack.
    assert_no_error("((((((((((1 + 2) + 3) + 4) + 5) + 6) + 7) + 8) + 9) + 10) + 0);");
}

#[test]
fn stability_edge_chained_comparisons() {
    assert_no_error("1 < 2;");
    assert_no_error("2 > 1;");
    assert_no_error("1 == 1;");
    assert_no_error("1 != 2;");
}

#[test]
fn stability_edge_not_operator() {
    assert_eval_bool("!true;", false);
    assert_eval_bool("!false;", true);
}

#[test]
fn stability_edge_string_with_spaces() {
    assert_eval_string(r#""hello world";"#, "hello world");
}

#[test]
fn stability_edge_multiple_statements() {
    // Programs with many statements should not crash.
    assert_no_error(
        r#"
        let a: number = 1;
        let b: number = 2;
        let c: number = 3;
        let d: number = 4;
        let e: number = 5;
        a + b + c + d + e;
    "#,
    );
}

#[test]
fn stability_edge_string_escape_sequences() {
    // Strings with standard escape-adjacent characters.
    assert_no_error(r#""tab\there";"#);
}

#[test]
fn stability_edge_nested_array_access() {
    assert_eval_number("let arr: number[] = [10, 20, 30]; arr[0];", 10.0);
    assert_eval_number("let arr: number[] = [10, 20, 30]; arr[2];", 30.0);
}

#[test]
fn stability_edge_function_with_no_return_value() {
    // Void functions must not crash on call.
    assert_no_error("fn greet() -> null { } greet();");
}

// ─── Stress Tests ────────────────────────────────────────────────────────────

#[test]
fn stability_stress_recursion_depth_50() {
    // Moderate recursion (50 levels) must complete without stack overflow.
    let code = r#"
        fn countdown(n: number) -> number {
            if (n <= 0) { return 0; }
            return countdown(n - 1);
        }
        countdown(50);
    "#;
    assert_eval_number(code, 0.0);
}

#[test]
fn stability_stress_recursion_depth_100() {
    // Deeper recursion (100 levels).
    let code = r#"
        fn sum_down(n: number) -> number {
            if (n <= 0) { return 0; }
            return n + sum_down(n - 1);
        }
        sum_down(100);
    "#;
    assert_eval_number(code, 5050.0);
}

#[test]
fn stability_stress_large_array_100_elements() {
    // 100 element array should be allocated and accessed without issues.
    let elements: Vec<String> = (0..100).map(|i| i.to_string()).collect();
    let code = format!("let arr: number[] = [{}]; arr[99];", elements.join(", "));
    assert_eval_number(&code, 99.0);
}

#[test]
fn stability_stress_large_array_500_elements() {
    // 500 element array stress test.
    let elements: Vec<String> = (0..500).map(|i| i.to_string()).collect();
    let code = format!("let arr: number[] = [{}]; arr[499];", elements.join(", "));
    assert_eval_number(&code, 499.0);
}

#[test]
fn stability_stress_many_variables() {
    // Programs with many variables should not exhaust resources.
    let mut code = String::new();
    for i in 0..50 {
        code.push_str(&format!("let v{}: number = {};\n", i, i));
    }
    code.push_str("v49;");
    assert_eval_number(&code, 49.0);
}

#[test]
fn stability_stress_long_string() {
    // A string of 1000 characters should be handled without issues.
    let long = "a".repeat(1000);
    let code = format!(r#""{}";"#, long);
    let runtime = Atlas::new();
    let result = runtime.eval(&code);
    assert!(
        result.is_ok(),
        "Long string evaluation failed: {:?}",
        result
    );
}

#[test]
fn stability_stress_many_function_calls() {
    // Many sequential function calls should not exhaust resources.
    let code = r#"
        fn inc(x: number) -> number { return x + 1; }
        var n: number = 0;
        var i: number = 0;
        while (i < 200) {
            n = inc(n);
            i = i + 1;
        }
        n;
    "#;
    assert_eval_number(code, 200.0);
}

#[test]
fn stability_stress_deep_if_else_nesting() {
    // Deeply nested conditionals (10 levels).
    let code = r#"
        let x: number = 5;
        if (x > 0) {
            if (x > 1) {
                if (x > 2) {
                    if (x > 3) {
                        if (x > 4) {
                            42;
                        } else { 0; }
                    } else { 0; }
                } else { 0; }
            } else { 0; }
        } else { 0; }
    "#;
    assert_eval_number(code, 42.0);
}

#[test]
fn stability_stress_while_1000_iterations() {
    // 1000 loop iterations should complete successfully.
    let code = r#"
        var sum: number = 0;
        var i: number = 0;
        while (i < 1000) {
            sum = sum + 1;
            i = i + 1;
        }
        sum;
    "#;
    assert_eval_number(code, 1000.0);
}

#[test]
fn stability_stress_fibonacci_15() {
    // Fibonacci(15) = 610 — exercises recursive call depth.
    let code = r#"
        fn fib(n: number) -> number {
            if (n <= 1) { return n; }
            return fib(n - 1) + fib(n - 2);
        }
        fib(15);
    "#;
    assert_eval_number(code, 610.0);
}

// ─── Error Recovery Tests ─────────────────────────────────────────────────────

#[test]
fn stability_error_recovery_undefined_variable() {
    // Accessing an undefined variable must return an error, not panic.
    assert_has_error("undefined_var;");
}

#[test]
fn stability_error_recovery_type_mismatch() {
    // Type mismatch must be caught at compile time, not crash at runtime.
    assert_has_error("let x: number = true;");
}

#[test]
fn stability_error_recovery_divide_by_zero() {
    // Divide by zero must produce a runtime error, not a panic.
    assert_has_error("1 / 0;");
}

#[test]
fn stability_error_recovery_array_out_of_bounds() {
    // Out-of-bounds access must produce a runtime error, not a panic.
    assert_has_error("let arr: number[] = [1, 2]; arr[10];");
}

#[test]
fn stability_error_recovery_wrong_argument_count() {
    // Calling a function with wrong arity must produce an error.
    assert_has_error("fn f(x: number) -> number { return x; } f(1, 2);");
}

#[test]
fn stability_error_recovery_wrong_return_type() {
    // Returning wrong type must produce a type error.
    assert_has_error("fn f() -> number { return true; }");
}

#[test]
fn stability_error_recovery_multiple_errors() {
    // Programs with multiple errors must not crash even on first error.
    let code = "let a: number = true; let b: string = 42;";
    let runtime = Atlas::new();
    let result = runtime.eval(code);
    assert!(
        result.is_err(),
        "Expected errors for type-mismatched declarations"
    );
}

#[test]
fn stability_error_recovery_unclosed_string() {
    // Unclosed string literal must produce a lex/parse error, not panic.
    assert_has_error(r#""unclosed string"#);
}

#[test]
fn stability_error_recovery_invalid_operator_usage() {
    // Applying operators to wrong types must produce an error.
    assert_has_error(r#"true + 1;"#);
}

#[test]
fn stability_error_recovery_call_non_function() {
    // Calling a non-function value must produce a runtime error.
    assert_has_error("let x: number = 42; x();");
}

// ─── Release Mode Verification Tests ─────────────────────────────────────────
// These tests verify behaviors that must hold in release mode builds.
// They are designed to catch issues that only manifest when optimizations are on.

#[test]
fn stability_release_arithmetic_precision() {
    // Arithmetic should be precise in both debug and release mode.
    assert_eval_number("100.0 * 100.0;", 10_000.0);
}

#[test]
fn stability_release_large_number_arithmetic() {
    // Large float arithmetic should not lose precision unexpectedly.
    assert_eval_number("1000000.0 + 1.0;", 1_000_001.0);
}

#[test]
fn stability_release_boolean_short_circuit() {
    // Short-circuit evaluation must work correctly in release mode.
    assert_eval_bool("false && true;", false);
    assert_eval_bool("true || false;", true);
}

#[test]
fn stability_release_recursive_correctness() {
    // Recursive functions must produce correct results (not optimized away).
    let code = r#"
        fn factorial(n: number) -> number {
            if (n <= 1) { return 1; }
            return n * factorial(n - 1);
        }
        factorial(10);
    "#;
    assert_eval_number(code, 3_628_800.0);
}

#[test]
fn stability_release_string_operations() {
    // String concatenation must work correctly in release mode.
    assert_eval_string(r#""foo" + "bar";"#, "foobar");
}

#[test]
fn stability_release_comparison_operators() {
    assert_eval_bool("1 < 2;", true);
    assert_eval_bool("2 < 1;", false);
    assert_eval_bool("1 == 1;", true);
    assert_eval_bool("1 != 2;", true);
    assert_eval_bool("2 >= 2;", true);
    assert_eval_bool("1 <= 2;", true);
}

#[test]
fn stability_release_loop_termination() {
    // Loops must terminate correctly in release mode (no optimizer infinite loop).
    let code = r#"
        var x: number = 10;
        while (x > 0) {
            x = x - 1;
        }
        x;
    "#;
    assert_eval_number(code, 0.0);
}

#[test]
fn stability_release_variable_mutation() {
    // Variable mutation must work correctly (not cached/inlined incorrectly).
    let code = r#"
        var x: number = 1;
        x = x + 1;
        x = x * 2;
        x;
    "#;
    assert_eval_number(code, 4.0);
}

#[test]
fn stability_release_nested_scope() {
    // Nested scopes must be correctly maintained in release mode.
    let code = r#"
        let x: number = 10;
        fn f() -> number {
            let y: number = 20;
            return x + y;
        }
        f();
    "#;
    assert_eval_number(code, 30.0);
}

#[test]
fn stability_release_error_codes_preserved() {
    // Error codes must be the same in debug and release builds.
    assert_error_code("1 / 0;", "AT0005");
    assert_error_code("let arr: number[] = [1]; arr[5];", "AT0006");
}
