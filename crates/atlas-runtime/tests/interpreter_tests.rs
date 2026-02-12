//! Interpreter integration tests
//!
//! Tests for arithmetic, conditionals, loops, and functions using the interpreter.
//! Phase: numbererpreter/phase-01-interpreter-core.md

use atlas_runtime::{Atlas, Value};

// ============================================================================
// Arithmetic Tests
// ============================================================================

#[test]
fn test_arithmetic_addition() {
    let runtime = Atlas::new();
    let result = runtime.eval("1 + 2");

    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 3.0),
        _ => panic!("Expected Number(3.0), got {:?}", result),
    }
}

#[test]
fn test_arithmetic_subtraction() {
    let runtime = Atlas::new();
    let result = runtime.eval("10 - 3");

    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 7.0),
        _ => panic!("Expected Number(7.0), got {:?}", result),
    }
}

#[test]
fn test_arithmetic_multiplication() {
    let runtime = Atlas::new();
    let result = runtime.eval("4 * 5");

    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 20.0),
        _ => panic!("Expected Number(20.0), got {:?}", result),
    }
}

#[test]
fn test_arithmetic_division() {
    let runtime = Atlas::new();
    let result = runtime.eval("20 / 4");

    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 5.0),
        _ => panic!("Expected Number(5.0), got {:?}", result),
    }
}

#[test]
fn test_arithmetic_modulo() {
    let runtime = Atlas::new();
    let result = runtime.eval("10 % 3");

    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 1.0),
        _ => panic!("Expected Number(1.0), got {:?}", result),
    }
}

#[test]
fn test_arithmetic_negation() {
    let runtime = Atlas::new();
    let result = runtime.eval("-42");

    match result {
        Ok(Value::Number(n)) => assert_eq!(n, -42.0),
        _ => panic!("Expected Number(-42.0), got {:?}", result),
    }
}

#[test]
fn test_arithmetic_complex_expression() {
    let runtime = Atlas::new();
    let result = runtime.eval("2 + 3 * 4 - 1");

    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 13.0),
        _ => panic!("Expected Number(13.0), got {:?}", result),
    }
}

#[test]
fn test_arithmetic_parentheses() {
    let runtime = Atlas::new();
    let result = runtime.eval("(2 + 3) * 4");

    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 20.0),
        _ => panic!("Expected Number(20.0), got {:?}", result),
    }
}

// ============================================================================
// Variable Tests
// ============================================================================

#[test]
fn test_variable_declaration_and_use() {
    let runtime = Atlas::new();
    let code = r#"
        let x: number = 42;
        x
    "#;

    match runtime.eval(code) {
        Ok(Value::Number(n)) => assert_eq!(n, 42.0),
        _ => panic!("Expected Number(42.0)"),
    }
}

#[test]
fn test_variable_assignment() {
    let runtime = Atlas::new();
    let code = r#"
        var x: number = 10;
        x = 20;
        x
    "#;

    match runtime.eval(code) {
        Ok(Value::Number(n)) => assert_eq!(n, 20.0),
        _ => panic!("Expected Number(20.0)"),
    }
}

#[test]
fn test_variable_arithmetic() {
    let runtime = Atlas::new();
    let code = r#"
        let a: number = 5;
        let b: number = 3;
        a + b
    "#;

    match runtime.eval(code) {
        Ok(Value::Number(n)) => assert_eq!(n, 8.0),
        _ => panic!("Expected Number(8.0)"),
    }
}

// ============================================================================
// Comparison and Boolean Tests
// ============================================================================

#[test]
fn test_comparison_equal() {
    let runtime = Atlas::new();
    let result = runtime.eval("5 == 5");

    match result {
        Ok(Value::Bool(b)) => assert!(b),
        _ => panic!("Expected Bool(true)"),
    }
}

#[test]
fn test_comparison_not_equal() {
    let runtime = Atlas::new();
    let result = runtime.eval("5 != 3");

    match result {
        Ok(Value::Bool(b)) => assert!(b),
        _ => panic!("Expected Bool(true)"),
    }
}

#[test]
fn test_comparison_less_than() {
    let runtime = Atlas::new();
    let result = runtime.eval("3 < 5");

    match result {
        Ok(Value::Bool(b)) => assert!(b),
        _ => panic!("Expected Bool(true)"),
    }
}

#[test]
fn test_comparison_greater_than() {
    let runtime = Atlas::new();
    let result = runtime.eval("5 > 3");

    match result {
        Ok(Value::Bool(b)) => assert!(b),
        _ => panic!("Expected Bool(true)"),
    }
}

#[test]
fn test_boolean_and() {
    let runtime = Atlas::new();
    let result = runtime.eval("true && true");

    match result {
        Ok(Value::Bool(b)) => assert!(b),
        _ => panic!("Expected Bool(true)"),
    }
}

#[test]
fn test_boolean_or() {
    let runtime = Atlas::new();
    let result = runtime.eval("false || true");

    match result {
        Ok(Value::Bool(b)) => assert!(b),
        _ => panic!("Expected Bool(true)"),
    }
}

#[test]
fn test_boolean_not() {
    let runtime = Atlas::new();
    let result = runtime.eval("!false");

    match result {
        Ok(Value::Bool(b)) => assert!(b),
        _ => panic!("Expected Bool(true)"),
    }
}

// ============================================================================
// Conditional Tests (If/Else)
// ============================================================================

#[test]
fn test_if_then() {
    let runtime = Atlas::new();
    let code = r#"
        var x: number = 0;
        if (true) {
            x = 42;
        }
        x
    "#;

    match runtime.eval(code) {
        Ok(Value::Number(n)) => assert_eq!(n, 42.0),
        _ => panic!("Expected Number(42.0)"),
    }
}

#[test]
fn test_if_else() {
    let runtime = Atlas::new();
    let code = r#"
        var x: number = 0;
        if (false) {
            x = 10;
        } else {
            x = 20;
        }
        x
    "#;

    match runtime.eval(code) {
        Ok(Value::Number(n)) => assert_eq!(n, 20.0),
        _ => panic!("Expected Number(20.0)"),
    }
}

#[test]
fn test_if_with_comparison() {
    let runtime = Atlas::new();
    let code = r#"
        let x: number = 5;
        var result: number = 0;
        if (x > 3) {
            result = 1;
        } else {
            result = 2;
        }
        result
    "#;

    match runtime.eval(code) {
        Ok(Value::Number(n)) => assert_eq!(n, 1.0),
        _ => panic!("Expected Number(1.0)"),
    }
}

// ============================================================================
// Loop Tests (While)
// ============================================================================

#[test]
fn test_while_loop() {
    let runtime = Atlas::new();
    let code = r#"
        var i: number = 0;
        var sum: number = 0;
        while (i < 5) {
            sum = sum + i;
            i = i + 1;
        }
        sum
    "#;

    match runtime.eval(code) {
        Ok(Value::Number(n)) => assert_eq!(n, 10.0), // 0 + 1 + 2 + 3 + 4
        _ => panic!("Expected Number(10.0)"),
    }
}

#[test]
fn test_while_loop_with_break() {
    let runtime = Atlas::new();
    let code = r#"
        var i: number = 0;
        while (i < 10) {
            if (i == 5) {
                break;
            }
            i = i + 1;
        }
        i
    "#;

    match runtime.eval(code) {
        Ok(Value::Number(n)) => assert_eq!(n, 5.0),
        _ => panic!("Expected Number(5.0)"),
    }
}

#[test]
fn test_while_loop_with_continue() {
    let runtime = Atlas::new();
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
        sum
    "#;

    match runtime.eval(code) {
        Ok(Value::Number(n)) => assert_eq!(n, 12.0), // 1 + 2 + 4 + 5 (skips 3)
        _ => panic!("Expected Number(12.0)"),
    }
}

// ============================================================================
// Loop Tests (For)
// ============================================================================

#[test]
fn test_for_loop() {
    let runtime = Atlas::new();
    let code = r#"
        var sum: number = 0;
        for (let i: number = 0; i < 5; i = i + 1) {
            sum = sum + i;
        }
        sum
    "#;

    match runtime.eval(code) {
        Ok(Value::Number(n)) => assert_eq!(n, 10.0),
        _ => panic!("Expected Number(10.0)"),
    }
}

#[test]
fn test_for_loop_with_break() {
    let runtime = Atlas::new();
    let code = r#"
        var result: number = 0;
        for (let i: number = 0; i < 10; i = i + 1) {
            if (i == 5) {
                break;
            }
            result = i;
        }
        result
    "#;

    match runtime.eval(code) {
        Ok(Value::Number(n)) => assert_eq!(n, 4.0),
        _ => panic!("Expected Number(4.0)"),
    }
}

#[test]
fn test_for_loop_with_continue() {
    let runtime = Atlas::new();
    let code = r#"
        var sum: number = 0;
        for (let i: number = 0; i < 5; i = i + 1) {
            if (i == 2) {
                continue;
            }
            sum = sum + i;
        }
        sum
    "#;

    match runtime.eval(code) {
        Ok(Value::Number(n)) => assert_eq!(n, 8.0), // 0 + 1 + 3 + 4 (skips 2)
        _ => panic!("Expected Number(8.0)"),
    }
}

// ============================================================================
// Function Tests
// ============================================================================

#[test]
fn test_function_definition_and_call() {
    let runtime = Atlas::new();
    let code = r#"
        fn add(a: number, b: number) -> number {
            return a + b;
        }
        add(3, 4)
    "#;

    match runtime.eval(code) {
        Ok(Value::Number(n)) => assert_eq!(n, 7.0),
        _ => panic!("Expected Number(7.0)"),
    }
}

#[test]
fn test_function_with_no_return() {
    let runtime = Atlas::new();
    let code = r#"
        fn foo(x: number) -> number {
            let y: number = x + 1;
            return y;
        }
        foo(5)
    "#;

    match runtime.eval(code) {
        Ok(Value::Number(n)) => assert_eq!(n, 6.0),
        _ => panic!("Expected Number(6.0)"),
    }
}

#[test]
fn test_function_with_early_return() {
    let runtime = Atlas::new();
    let code = r#"
        fn abs(x: number) -> number {
            if (x < 0) {
                return -x;
            }
            return x;
        }
        abs(-5)
    "#;

    match runtime.eval(code) {
        Ok(Value::Number(n)) => assert_eq!(n, 5.0),
        _ => panic!("Expected Number(5.0)"),
    }
}

#[test]
fn test_function_recursion() {
    let runtime = Atlas::new();
    let code = r#"
        fn factorial(n: number) -> number {
            if (n <= 1) {
                return 1;
            }
            return n * factorial(n - 1);
        }
        factorial(5)
    "#;

    match runtime.eval(code) {
        Ok(Value::Number(n)) => assert_eq!(n, 120.0),
        _ => panic!("Expected Number(120.0)"),
    }
}

#[test]
fn test_function_with_local_variables() {
    let runtime = Atlas::new();
    let code = r#"
        fn compute(x: number) -> number {
            let a: number = x + 1;
            let b: number = a * 2;
            return b - 1;
        }
        compute(5)
    "#;

    match runtime.eval(code) {
        Ok(Value::Number(n)) => assert_eq!(n, 11.0), // ((5 + 1) * 2) - 1 = 11
        _ => panic!("Expected Number(11.0)"),
    }
}

// ============================================================================
// Array Tests
// ============================================================================

#[test]
fn test_array_literal() {
    let runtime = Atlas::new();
    let code = r#"
        let arr: number[] = [1, 2, 3];
        arr[1]
    "#;

    match runtime.eval(code) {
        Ok(Value::Number(n)) => assert_eq!(n, 2.0),
        _ => panic!("Expected Number(2.0)"),
    }
}

#[test]
fn test_array_assignment() {
    let runtime = Atlas::new();
    let code = r#"
        let arr: number[] = [1, 2, 3];
        arr[1] = 99;
        arr[1]
    "#;

    match runtime.eval(code) {
        Ok(Value::Number(n)) => assert_eq!(n, 99.0),
        _ => panic!("Expected Number(99.0)"),
    }
}

#[test]
fn test_array_reference_semantics() {
    let runtime = Atlas::new();
    let code = r#"
        let arr1: number[] = [1, 2, 3];
        let arr2: number[] = arr1;
        arr1[0] = 42;
        arr2[0]
    "#;

    match runtime.eval(code) {
        Ok(Value::Number(n)) => assert_eq!(n, 42.0),
        _ => panic!("Expected Number(42.0)"),
    }
}

// ============================================================================
// Array Error Tests
// ============================================================================

#[test]
fn test_array_out_of_bounds_read() {
    let runtime = Atlas::new();
    let code = r#"
        let arr: number[] = [1, 2, 3];
        arr[5]
    "#;

    match runtime.eval(code) {
        Err(_) => {}, // Expected runtime error
        Ok(val) => panic!("Expected OutOfBounds error, got {:?}", val),
    }
}

#[test]
fn test_array_out_of_bounds_write() {
    let runtime = Atlas::new();
    let code = r#"
        let arr: number[] = [1, 2, 3];
        arr[10] = 99;
        arr[0]
    "#;

    match runtime.eval(code) {
        Err(_) => {}, // Expected runtime error
        Ok(val) => panic!("Expected OutOfBounds error, got {:?}", val),
    }
}

#[test]
fn test_array_negative_index_read() {
    let runtime = Atlas::new();
    let code = r#"
        let arr: number[] = [1, 2, 3];
        arr[-1]
    "#;

    match runtime.eval(code) {
        Err(_) => {}, // Expected runtime error
        Ok(val) => panic!("Expected InvalidIndex error, got {:?}", val),
    }
}

#[test]
fn test_array_negative_index_write() {
    let runtime = Atlas::new();
    let code = r#"
        let arr: number[] = [1, 2, 3];
        arr[-1] = 99;
        arr[0]
    "#;

    match runtime.eval(code) {
        Err(_) => {}, // Expected runtime error
        Ok(val) => panic!("Expected InvalidIndex error, got {:?}", val),
    }
}

#[test]
fn test_array_fractional_index_read() {
    let runtime = Atlas::new();
    let code = r#"
        let arr: number[] = [1, 2, 3];
        arr[1.5]
    "#;

    match runtime.eval(code) {
        Err(_) => {}, // Expected runtime error
        Ok(val) => panic!("Expected InvalidIndex error, got {:?}", val),
    }
}

#[test]
fn test_array_fractional_index_write() {
    let runtime = Atlas::new();
    let code = r#"
        let arr: number[] = [1, 2, 3];
        arr[0.5] = 99;
        arr[0]
    "#;

    match runtime.eval(code) {
        Err(_) => {}, // Expected runtime error
        Ok(val) => panic!("Expected InvalidIndex error, got {:?}", val),
    }
}

#[test]
fn test_array_whole_number_float_index() {
    let runtime = Atlas::new();
    let code = r#"
        let arr: number[] = [1, 2, 3];
        arr[1.0]
    "#;

    // 1.0 is a valid index (whole number)
    match runtime.eval(code) {
        Ok(Value::Number(n)) => assert_eq!(n, 2.0),
        _ => panic!("Expected Number(2.0) - 1.0 is a valid whole number index"),
    }
}

#[test]
fn test_array_mutation_in_function() {
    let runtime = Atlas::new();
    let code = r#"
        fn modify(arr: number[]) -> void {
            arr[0] = 999;
        }

        let numbers: number[] = [1, 2, 3];
        modify(numbers);
        numbers[0]
    "#;

    match runtime.eval(code) {
        Ok(Value::Number(n)) => assert_eq!(n, 999.0),
        _ => panic!("Expected Number(999.0) - array should be mutated"),
    }
}

#[test]
fn test_empty_array() {
    let runtime = Atlas::new();
    let code = r#"
        let arr: number[] = [];
        len(arr)
    "#;

    match runtime.eval(code) {
        Ok(Value::Number(n)) => assert_eq!(n, 0.0),
        _ => panic!("Expected Number(0.0)"),
    }
}

#[test]
fn test_nested_array_literal() {
    let runtime = Atlas::new();
    let code = r#"
        let arr: number[][] = [[1, 2], [3, 4]];
        arr[1][0]
    "#;

    match runtime.eval(code) {
        Ok(Value::Number(n)) => assert_eq!(n, 3.0),
        _ => panic!("Expected Number(3.0)"),
    }
}

#[test]
fn test_nested_array_mutation() {
    let runtime = Atlas::new();
    let code = r#"
        let arr: number[][] = [[1, 2], [3, 4]];
        arr[0][1] = 99;
        arr[0][1]
    "#;

    match runtime.eval(code) {
        Ok(Value::Number(n)) => assert_eq!(n, 99.0),
        _ => panic!("Expected Number(99.0)"),
    }
}

// ============================================================================
// Function Call Tests (Phase 05)
// ============================================================================

#[test]
fn test_function_nested_calls() {
    let runtime = Atlas::new();
    let code = r#"
        fn add(a: number, b: number) -> number {
            return a + b;
        }

        fn multiply(x: number, y: number) -> number {
            return x * y;
        }

        fn compute(n: number) -> number {
            return add(multiply(n, 2), 5);
        }

        compute(3)
    "#;

    match runtime.eval(code) {
        Ok(Value::Number(n)) => assert_eq!(n, 11.0), // (3 * 2) + 5 = 11
        _ => panic!("Expected Number(11.0)"),
    }
}

#[test]
fn test_function_wrong_arity_too_few() {
    let runtime = Atlas::new();
    let code = r#"
        fn add(a: number, b: number) -> number {
            return a + b;
        }
        add(5)
    "#;

    match runtime.eval(code) {
        Err(_) => {}, // Expected runtime error for wrong arity
        Ok(val) => panic!("Expected arity error, got {:?}", val),
    }
}

#[test]
fn test_function_wrong_arity_too_many() {
    let runtime = Atlas::new();
    let code = r#"
        fn add(a: number, b: number) -> number {
            return a + b;
        }
        add(1, 2, 3)
    "#;

    match runtime.eval(code) {
        Err(_) => {}, // Expected runtime error for wrong arity
        Ok(val) => panic!("Expected arity error, got {:?}", val),
    }
}

#[test]
fn test_function_void_return() {
    let runtime = Atlas::new();
    let code = r#"
        var result: number = 0;

        fn set_result(x: number) -> void {
            result = x;
        }

        set_result(42);
        result
    "#;

    match runtime.eval(code) {
        Ok(Value::Number(n)) => assert_eq!(n, 42.0),
        _ => panic!("Expected Number(42.0)"),
    }
}

#[test]
fn test_function_no_parameters() {
    let runtime = Atlas::new();
    let code = r#"
        fn get_answer() -> number {
            return 42;
        }
        get_answer()
    "#;

    match runtime.eval(code) {
        Ok(Value::Number(n)) => assert_eq!(n, 42.0),
        _ => panic!("Expected Number(42.0)"),
    }
}

#[test]
fn test_function_multiple_parameters() {
    let runtime = Atlas::new();
    let code = r#"
        fn sum_four(a: number, b: number, c: number, d: number) -> number {
            return a + b + c + d;
        }
        sum_four(1, 2, 3, 4)
    "#;

    match runtime.eval(code) {
        Ok(Value::Number(n)) => assert_eq!(n, 10.0),
        _ => panic!("Expected Number(10.0)"),
    }
}

#[test]
fn test_function_call_stack_depth() {
    let runtime = Atlas::new();
    let code = r#"
        fn count_down(n: number) -> number {
            if (n <= 0) {
                return 0;
            }
            return n + count_down(n - 1);
        }
        count_down(5)
    "#;

    match runtime.eval(code) {
        Ok(Value::Number(n)) => assert_eq!(n, 15.0), // 5 + 4 + 3 + 2 + 1 = 15
        _ => panic!("Expected Number(15.0)"),
    }
}

#[test]
fn test_function_local_variable_isolation() {
    let runtime = Atlas::new();
    let code = r#"
        var global: number = 100;

        fn modify_local() -> number {
            let global: number = 50;
            return global;
        }

        let result: number = modify_local();
        result + global
    "#;

    match runtime.eval(code) {
        Ok(Value::Number(n)) => assert_eq!(n, 150.0), // 50 + 100 = 150
        _ => panic!("Expected Number(150.0)"),
    }
}

#[test]
fn test_function_return_early_from_nested() {
    let runtime = Atlas::new();
    let code = r#"
        fn find_first_positive(a: number, b: number, c: number) -> number {
            if (a > 0) {
                return a;
            }
            if (b > 0) {
                return b;
            }
            if (c > 0) {
                return c;
            }
            return -1;
        }
        find_first_positive(-5, -3, 7)
    "#;

    match runtime.eval(code) {
        Ok(Value::Number(n)) => assert_eq!(n, 7.0),
        _ => panic!("Expected Number(7.0)"),
    }
}

#[test]
fn test_function_mutually_recursive() {
    let runtime = Atlas::new();
    let code = r#"
        fn is_even(n: number) -> bool {
            if (n == 0) {
                return true;
            }
            return is_odd(n - 1);
        }

        fn is_odd(n: number) -> bool {
            if (n == 0) {
                return false;
            }
            return is_even(n - 1);
        }

        is_even(4)
    "#;

    match runtime.eval(code) {
        Ok(Value::Bool(b)) => assert!(b),
        _ => panic!("Expected Bool(true)"),
    }
}

// ============================================================================
// String Tests
// ============================================================================

#[test]
fn test_string_concatenation() {
    let runtime = Atlas::new();
    let code = r#"
        let s: string = "Hello, " + "World!";
        s
    "#;

    match runtime.eval(code) {
        Ok(Value::String(s)) => assert_eq!(s.as_ref(), "Hello, World!"),
        _ => panic!("Expected String"),
    }
}

// TODO: Enable when typechecker supports string indexing
// String indexing is implemented in the interpreter, but the typechecker
// doesn't allow it yet. This will be addressed in a future phase.
#[test]
#[ignore]
fn test_string_indexing() {
    let runtime = Atlas::new();
    let code = r#"
        let s: string = "Hello";
        s[1]
    "#;

    match runtime.eval(code) {
        Ok(Value::String(s)) => assert_eq!(s.as_ref(), "e"),
        _ => panic!("Expected String('e')"),
    }
}

// ============================================================================
// Stdlib Function Tests
// ============================================================================

#[test]
fn test_stdlib_len_string() {
    let runtime = Atlas::new();
    let code = r#"
        let s: string = "hello";
        len(s)
    "#;

    match runtime.eval(code) {
        Ok(Value::Number(n)) => assert_eq!(n, 5.0),
        _ => panic!("Expected Number(5.0)"),
    }
}

#[test]
fn test_stdlib_len_array() {
    let runtime = Atlas::new();
    let code = r#"
        let arr: number[] = [1, 2, 3, 4];
        len(arr)
    "#;

    match runtime.eval(code) {
        Ok(Value::Number(n)) => assert_eq!(n, 4.0),
        _ => panic!("Expected Number(4.0)"),
    }
}

#[test]
fn test_stdlib_str() {
    let runtime = Atlas::new();
    let code = r#"
        let n: number = 42;
        str(n)
    "#;

    match runtime.eval(code) {
        Ok(Value::String(s)) => assert_eq!(s.as_ref(), "42"),
        _ => panic!("Expected String('42')"),
    }
}

// ============================================================================
// Scope and Shadowing Tests
// ============================================================================

#[test]
fn test_block_scope() {
    let runtime = Atlas::new();
    let code = r#"
        let x: number = 1;
        if (true) {
            let x: number = 2;
            x;
        }
    "#;

    match runtime.eval(code) {
        Ok(Value::Number(n)) => assert_eq!(n, 2.0),
        _ => panic!("Expected Number(2.0)"),
    }
}

#[test]
fn test_function_scope() {
    let runtime = Atlas::new();
    let code = r#"
        var x: number = 10;
        fn foo(x: number) -> number {
            return x + 1;
        }
        foo(5)
    "#;

    match runtime.eval(code) {
        Ok(Value::Number(n)) => assert_eq!(n, 6.0),
        _ => panic!("Expected Number(6.0)"),
    }
}

// ============================================================================
// Complex Integration Tests
// ============================================================================

#[test]
fn test_fibonacci() {
    let runtime = Atlas::new();
    let code = r#"
        fn fib(n: number) -> number {
            if (n <= 1) {
                return n;
            }
            return fib(n - 1) + fib(n - 2);
        }
        fib(10)
    "#;

    match runtime.eval(code) {
        Ok(Value::Number(n)) => assert_eq!(n, 55.0),
        _ => panic!("Expected Number(55.0)"),
    }
}

#[test]
fn test_array_sum_with_function() {
    let runtime = Atlas::new();
    let code = r#"
        fn sum_array(arr: number[]) -> number {
            var total: number = 0;
            var i: number = 0;
            while (i < len(arr)) {
                total = total + arr[i];
                i = i + 1;
            }
            return total;
        }

        let numbers: number[] = [1, 2, 3, 4, 5];
        sum_array(numbers)
    "#;

    match runtime.eval(code) {
        Ok(Value::Number(n)) => assert_eq!(n, 15.0),
        _ => panic!("Expected Number(15.0)"),
    }
}
