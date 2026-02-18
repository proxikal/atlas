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
