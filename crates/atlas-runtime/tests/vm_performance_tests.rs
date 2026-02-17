//! VM Performance Regression Tests
//!
//! These tests verify that VM optimizations don't break correctness.
//! Each test exercises a specific optimization path and validates results.

use atlas_runtime::compiler::Compiler;
use atlas_runtime::lexer::Lexer;
use atlas_runtime::parser::Parser;
use atlas_runtime::security::SecurityContext;
use atlas_runtime::value::Value;
use atlas_runtime::vm::VM;
use std::time::Instant;

fn vm_eval(source: &str) -> Option<Value> {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&program).expect("Compilation failed");
    let mut vm = VM::new(bytecode);
    vm.run(&SecurityContext::allow_all())
        .expect("VM execution failed")
}

fn vm_number(source: &str) -> f64 {
    match vm_eval(source) {
        Some(Value::Number(n)) => n,
        other => panic!("Expected Number, got {:?}", other),
    }
}

fn vm_string(source: &str) -> String {
    match vm_eval(source) {
        Some(Value::String(s)) => (*s).clone(),
        other => panic!("Expected String, got {:?}", other),
    }
}

fn vm_bool(source: &str) -> bool {
    match vm_eval(source) {
        Some(Value::Bool(b)) => b,
        other => panic!("Expected Bool, got {:?}", other),
    }
}

// ============================================================================
// Arithmetic optimization correctness (tests 1-8)
// ============================================================================

#[test]
fn test_arithmetic_add_loop_correctness() {
    let result =
        vm_number("let sum = 0; let i = 1; while (i <= 100) { sum = sum + i; i = i + 1; } sum;");
    assert_eq!(result, 5050.0);
}

#[test]
fn test_arithmetic_sub_correctness() {
    let result = vm_number(
        "let result = 1000; let i = 0; while (i < 10) { result = result - i; i = i + 1; } result;",
    );
    assert_eq!(result, 955.0);
}

#[test]
fn test_arithmetic_mul_correctness() {
    let result = vm_number(
        "let result = 1; let i = 1; while (i <= 10) { result = result * i; i = i + 1; } result;",
    );
    assert_eq!(result, 3628800.0);
}

#[test]
fn test_arithmetic_div_correctness() {
    let result = vm_number("let r = 1000000; r = r / 10; r = r / 10; r = r / 10; r;");
    assert_eq!(result, 1000.0);
}

#[test]
fn test_arithmetic_mod_correctness() {
    let result = vm_number(
        "let count = 0; let i = 0; while (i < 100) { if (i % 3 == 0) { count = count + 1; } i = i + 1; } count;",
    );
    assert_eq!(result, 34.0);
}

#[test]
fn test_arithmetic_negate_correctness() {
    let result = vm_number("let x = 42; -x;");
    assert_eq!(result, -42.0);
}

#[test]
fn test_arithmetic_chained_expression() {
    let result = vm_number("1 + 2 * 3 - 4 + 5;");
    assert_eq!(result, 8.0);
}

#[test]
fn test_arithmetic_mixed_operations() {
    let result = vm_number("let a = 10; let b = 3; a * b + a / b - a % b;");
    assert!((result - 32.333333333333336).abs() < 1e-10);
}

// ============================================================================
// Function call optimization correctness (tests 9-16)
// ============================================================================

#[test]
fn test_function_simple_call() {
    let result = vm_number("fn add(a: number, b: number) -> number { return a + b; } add(3, 4);");
    assert_eq!(result, 7.0);
}

#[test]
fn test_function_recursive_fibonacci() {
    let result = vm_number(
        "fn fib(n: number) -> number { if (n <= 1) { return n; } return fib(n - 1) + fib(n - 2); } fib(10);",
    );
    assert_eq!(result, 55.0);
}

#[test]
fn test_function_nested_calls() {
    let result = vm_number(
        "fn double(x: number) -> number { return x * 2; } fn triple(x: number) -> number { return x * 3; } fn compute(x: number) -> number { return double(triple(x)); } compute(5);",
    );
    assert_eq!(result, 30.0);
}

#[test]
fn test_function_many_calls_loop() {
    let result = vm_number(
        "fn increment(x: number) -> number { return x + 1; } let r = 0; let i = 0; while (i < 100) { r = increment(r); i = i + 1; } r;",
    );
    assert_eq!(result, 100.0);
}

#[test]
fn test_function_return_value() {
    let result = vm_number(
        "fn max(a: number, b: number) -> number { if (a > b) { return a; } return b; } max(10, 20);",
    );
    assert_eq!(result, 20.0);
}

#[test]
fn test_function_multiple_args() {
    let result = vm_number(
        "fn sum3(a: number, b: number, c: number) -> number { return a + b + c; } sum3(10, 20, 30);",
    );
    assert_eq!(result, 60.0);
}

#[test]
fn test_function_recursive_factorial() {
    let result = vm_number(
        "fn fact(n: number) -> number { if (n <= 1) { return 1; } return n * fact(n - 1); } fact(10);",
    );
    assert_eq!(result, 3628800.0);
}

#[test]
fn test_function_call_in_expression() {
    let result =
        vm_number("fn square(x: number) -> number { return x * x; } square(3) + square(4);");
    assert_eq!(result, 25.0);
}

// ============================================================================
// Loop optimization correctness (tests 17-24)
// ============================================================================

#[test]
fn test_loop_simple_counting() {
    let result = vm_number("let i = 0; while (i < 1000) { i = i + 1; } i;");
    assert_eq!(result, 1000.0);
}

#[test]
fn test_loop_accumulation() {
    let result =
        vm_number("let sum = 0; let i = 1; while (i <= 1000) { sum = sum + i; i = i + 1; } sum;");
    assert_eq!(result, 500500.0);
}

#[test]
fn test_loop_nested() {
    let result = vm_number(
        "let count = 0; let i = 0; while (i < 50) { let j = 0; while (j < 50) { count = count + 1; j = j + 1; } i = i + 1; } count;",
    );
    assert_eq!(result, 2500.0);
}

#[test]
fn test_loop_with_conditionals() {
    let result = vm_number(
        "let evens = 0; let i = 0; while (i < 100) { if (i % 2 == 0) { evens = evens + 1; } i = i + 1; } evens;",
    );
    assert_eq!(result, 50.0);
}

#[test]
fn test_loop_variable_update() {
    let result = vm_number(
        "let a = 0; let b = 1; let i = 0; while (i < 20) { let temp = a + b; a = b; b = temp; i = i + 1; } b;",
    );
    assert_eq!(result, 10946.0);
}

#[test]
fn test_loop_large_iteration() {
    let result =
        vm_number("let sum = 0; let i = 0; while (i < 10000) { sum = sum + i; i = i + 1; } sum;");
    assert_eq!(result, 49995000.0);
}

#[test]
fn test_loop_function_call_inside() {
    let result = vm_number(
        "fn square(x: number) -> number { return x * x; } let sum = 0; let i = 1; while (i <= 10) { sum = sum + square(i); i = i + 1; } sum;",
    );
    assert_eq!(result, 385.0);
}

#[test]
fn test_loop_deeply_nested() {
    let result = vm_number(
        "let count = 0; let i = 0; while (i < 10) { let j = 0; while (j < 10) { let k = 0; while (k < 10) { count = count + 1; k = k + 1; } j = j + 1; } i = i + 1; } count;",
    );
    assert_eq!(result, 1000.0);
}

// ============================================================================
// Array optimization correctness (tests 25-30)
// ============================================================================

#[test]
fn test_array_creation_and_access() {
    let result = vm_number("let arr = [10, 20, 30, 40, 50]; arr[0] + arr[4];");
    assert_eq!(result, 60.0);
}

#[test]
fn test_array_index_in_loop() {
    let result = vm_number(
        "let arr = [1, 2, 3, 4, 5]; let sum = 0; let i = 0; while (i < 5) { sum = sum + arr[i]; i = i + 1; } sum;",
    );
    assert_eq!(result, 15.0);
}

#[test]
fn test_array_set_index() {
    let result = vm_number(
        "let arr = [0, 0, 0]; arr[0] = 10; arr[1] = 20; arr[2] = 30; arr[0] + arr[1] + arr[2];",
    );
    assert_eq!(result, 60.0);
}

#[test]
fn test_array_element_sum() {
    let result = vm_number(
        "let arr = [10, 20, 30, 40, 50]; let sum = 0; let i = 0; while (i < 5) { sum = sum + arr[i]; i = i + 1; } sum;",
    );
    assert_eq!(result, 150.0);
}

#[test]
fn test_array_large_creation() {
    let result = vm_number(
        "let arr = [1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20]; let sum = 0; let i = 0; while (i < 20) { sum = sum + arr[i]; i = i + 1; } sum;",
    );
    assert_eq!(result, 210.0);
}

#[test]
fn test_array_modification_in_loop() {
    let result = vm_number(
        "let arr = [1, 2, 3, 4, 5]; let i = 0; while (i < 5) { arr[i] = arr[i] * 2; i = i + 1; } arr[0] + arr[1] + arr[2] + arr[3] + arr[4];",
    );
    assert_eq!(result, 30.0);
}

// ============================================================================
// Stack operation correctness (tests 31-34)
// ============================================================================

#[test]
fn test_stack_deep_expression() {
    let result = vm_number("((((1 + 2) * 3) + 4) * 5);");
    assert_eq!(result, 65.0);
}

#[test]
fn test_stack_pop_semantics() {
    let result = vm_number("let x = 1; let y = 2; let z = x + y; z;");
    assert_eq!(result, 3.0);
}

#[test]
fn test_stack_dup_pattern() {
    let result = vm_number("let arr = [1, 2, 3]; arr[0] = arr[0] + 10; arr[0];");
    assert_eq!(result, 11.0);
}

#[test]
fn test_stack_complex_expression_chain() {
    let result = vm_number(
        "let a = 1; let b = 2; let c = 3; let d = 4; (a + b) * (c + d) - (a * b + c * d);",
    );
    assert_eq!(result, 7.0);
}

// ============================================================================
// Comparison and equality optimization (tests 35-38)
// ============================================================================

#[test]
fn test_comparison_less_greater() {
    let result = vm_number(
        "let count = 0; if (1 < 2) { count = count + 1; } if (2 > 1) { count = count + 1; } if (1 <= 1) { count = count + 1; } if (2 >= 2) { count = count + 1; } count;",
    );
    assert_eq!(result, 4.0);
}

#[test]
fn test_equality_check() {
    let result = vm_number(
        "let count = 0; if (1 == 1) { count = count + 1; } if (1 != 2) { count = count + 1; } count;",
    );
    assert_eq!(result, 2.0);
}

#[test]
fn test_boolean_not() {
    let result = vm_bool("let x = true; !x;");
    assert!(!result);
}

#[test]
fn test_comparison_in_loop() {
    let result = vm_number(
        "let max_val = 0; let i = 0; while (i < 100) { if (i > max_val) { max_val = i; } i = i + 1; } max_val;",
    );
    assert_eq!(result, 99.0);
}

// ============================================================================
// String optimization correctness (tests 39-40)
// ============================================================================

#[test]
fn test_string_concat_correctness() {
    let result = vm_string(r#""hello" + " " + "world";"#);
    assert_eq!(result, "hello world");
}

#[test]
fn test_string_concat_loop() {
    let result =
        vm_string(r#"let s = ""; let i = 0; while (i < 5) { s = s + "a"; i = i + 1; } s;"#);
    assert_eq!(result, "aaaaa");
}

// ============================================================================
// Dispatch table correctness (tests 41-44)
// ============================================================================

#[test]
fn test_dispatch_null() {
    assert!(matches!(vm_eval("null;"), Some(Value::Null)));
}

#[test]
fn test_dispatch_true() {
    assert_eq!(vm_bool("true;"), true);
}

#[test]
fn test_dispatch_false() {
    assert_eq!(vm_bool("false;"), false);
}

#[test]
fn test_dispatch_number() {
    assert_eq!(vm_number("42;"), 42.0);
}

// ============================================================================
// Performance smoke tests (tests 45-48)
// ============================================================================

#[test]
fn test_perf_large_loop_completes() {
    let start = Instant::now();
    let result =
        vm_number("let sum = 0; let i = 0; while (i < 50000) { sum = sum + i; i = i + 1; } sum;");
    let elapsed = start.elapsed();
    assert_eq!(result, 1249975000.0);
    assert!(elapsed.as_secs() < 5, "Loop took too long: {:?}", elapsed);
}

#[test]
fn test_perf_recursive_fib_completes() {
    let start = Instant::now();
    let result = vm_number(
        "fn fib(n: number) -> number { if (n <= 1) { return n; } return fib(n - 1) + fib(n - 2); } fib(20);",
    );
    let elapsed = start.elapsed();
    assert_eq!(result, 6765.0);
    assert!(elapsed.as_secs() < 10, "Fib took too long: {:?}", elapsed);
}

#[test]
fn test_perf_nested_loops_complete() {
    let start = Instant::now();
    let result = vm_number(
        "let count = 0; let i = 0; while (i < 100) { let j = 0; while (j < 100) { count = count + 1; j = j + 1; } i = i + 1; } count;",
    );
    let elapsed = start.elapsed();
    assert_eq!(result, 10000.0);
    assert!(
        elapsed.as_secs() < 5,
        "Nested loops took too long: {:?}",
        elapsed
    );
}

#[test]
fn test_perf_function_calls_complete() {
    let start = Instant::now();
    let result = vm_number(
        "fn add(a: number, b: number) -> number { return a + b; } let sum = 0; let i = 0; while (i < 10000) { sum = add(sum, 1); i = i + 1; } sum;",
    );
    let elapsed = start.elapsed();
    assert_eq!(result, 10000.0);
    assert!(
        elapsed.as_secs() < 5,
        "Function calls took too long: {:?}",
        elapsed
    );
}
