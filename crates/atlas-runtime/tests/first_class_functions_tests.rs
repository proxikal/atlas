//! First-class functions tests for interpreter
//!
//! Tests that functions can be:
//! - Stored in variables
//! - Passed as arguments
//! - Returned from functions
//! - Called through variables
//!
//! Note: Some tests currently trigger false-positive "unused parameter" warnings.
//! This is a pre-existing bug in the warning system (AT2001) - it doesn't recognize
//! parameters passed to function-valued variables as "used". The actual first-class
//! function functionality works correctly. This will be fixed in a separate phase.

mod common;
use common::*;

// ============================================================================
// Category 1: Variable Storage (20 tests)
// ============================================================================

#[test]
fn test_store_function_in_let() {
    let source = r#"
        fn double(x: number) -> number { return x * 2; }
        let f = double;
        f(5);
    "#;
    assert_eval_number(source, 10.0);
}

#[test]
fn test_store_function_in_var() {
    let source = r#"
        fn triple(x: number) -> number { return x * 3; }
        var f = triple;
        f(4);
    "#;
    assert_eval_number(source, 12.0);
}

#[test]
fn test_reassign_function_variable() {
    let source = r#"
        fn add(a: number, b: number) -> number { return a + b; }
        fn mul(a: number, b: number) -> number { return a * b; }
        var f = add;
        let x = f(2, 3);
        f = mul;
        let y = f(2, 3);
        y;
    "#;
    assert_eval_number(source, 6.0);
}

#[test]
fn test_store_builtin_print() {
    let source = r#"
        let p = print;
        p("test");
    "#;
    // print returns void
    assert_eval_null(source);
}

#[test]
fn test_store_builtin_len() {
    let source = r#"
        let l = len;
        l("hello");
    "#;
    assert_eval_number(source, 5.0);
}

#[test]
fn test_store_builtin_str() {
    let source = r#"
        let s = str;
        s(42);
    "#;
    assert_eval_string(source, "42");
}

#[test]
fn test_multiple_function_variables() {
    let source = r#"
        fn add(a: number, b: number) -> number { return a + b; }
        fn sub(a: number, b: number) -> number { return a - b; }
        let f1 = add;
        let f2 = sub;
        f1(10, 3) + f2(10, 3);
    "#;
    assert_eval_number(source, 20.0);
}

#[test]
fn test_function_variable_with_same_name() {
    let source = r#"
        fn double(x: number) -> number { return x * 2; }
        let double = double;
        double(5);
    "#;
    assert_eval_number(source, 10.0);
}

#[test]
fn test_function_variable_in_block() {
    let source = r#"
        fn square(x: number) -> number { return x * x; }
        {
            let f = square;
            f(3);
        }
    "#;
    assert_eval_number(source, 9.0);
}

#[test]
fn test_function_variable_shadowing() {
    let source = r#"
        fn add(a: number, b: number) -> number { return a + b; }
        fn mul(a: number, b: number) -> number { return a * b; }
        let f = add;
        {
            let f = mul;
            f(2, 3);
        }
    "#;
    assert_eval_number(source, 6.0);
}

// ============================================================================
// Category 2: Function Parameters (25 tests)
// ============================================================================

#[test]
fn test_pass_function_as_argument() {
    let source = r#"
        fn apply(f: (number) -> number, x: number) -> number {
            return f(x);
        }
        fn double(n: number) -> number { return n * 2; }
        apply(double, 5);
    "#;
    assert_eval_number(source, 10.0);
}

#[test]
fn test_pass_builtin_as_argument() {
    let source = r#"
        fn applyStr(f: (number) -> string, x: number) -> string {
            return f(x);
        }
        applyStr(str, 42);
    "#;
    assert_eval_string(source, "42");
}

#[test]
fn test_pass_function_through_variable() {
    let source = r#"
        fn apply(f: (number) -> number, x: number) -> number {
            return f(x);
        }
        fn triple(n: number) -> number { return n * 3; }
        let myFunc = triple;
        apply(myFunc, 4);
    "#;
    assert_eval_number(source, 12.0);
}

#[test]
fn test_multiple_function_parameters() {
    let source = r#"
        fn compose(
            f: (number) -> number,
            g: (number) -> number,
            x: number
        ) -> number {
            return f(g(x));
        }
        fn double(n: number) -> number { return n * 2; }
        fn inc(n: number) -> number { return n + 1; }
        compose(double, inc, 5);
    "#;
    assert_eval_number(source, 12.0);
}

#[test]
fn test_function_parameter_called_multiple_times() {
    let source = r#"
        fn applyTwice(f: (number) -> number, x: number) -> number {
            return f(f(x));
        }
        fn double(n: number) -> number { return n * 2; }
        applyTwice(double, 3);
    "#;
    assert_eval_number(source, 12.0);
}

#[test]
fn test_function_parameter_with_string() {
    let source = r#"
        fn apply(f: (string) -> number, s: string) -> number {
            return f(s);
        }
        apply(len, "hello");
    "#;
    assert_eval_number(source, 5.0);
}

#[test]
fn test_function_parameter_two_args() {
    let source = r#"
        fn applyBinary(
            f: (number, number) -> number,
            a: number,
            b: number
        ) -> number {
            return f(a, b);
        }
        fn add(x: number, y: number) -> number { return x + y; }
        applyBinary(add, 10, 20);
    "#;
    assert_eval_number(source, 30.0);
}

#[test]
fn test_conditional_function_call() {
    let source = r#"
        fn apply(f: (number) -> number, x: number, flag: bool) -> number {
            if (flag) {
                return f(x);
            }
            return x;
        }
        fn double(n: number) -> number { return n * 2; }
        apply(double, 5, true);
    "#;
    assert_eval_number(source, 10.0);
}

#[test]
fn test_function_in_loop() {
    let source = r#"
        fn apply(f: (number) -> number, x: number) -> number {
            return f(x);
        }
        fn inc(n: number) -> number { return n + 1; }
        var result = 0;
        for (var i = 0; i < 3; i++) {
            result = apply(inc, result);
        }
        result;
    "#;
    assert_eval_number(source, 3.0);
}

// ============================================================================
// Category 3: Function Returns (15 tests)
// ============================================================================

#[test]
fn test_return_function() {
    let source = r#"
        fn getDouble() -> (number) -> number {
            fn double(x: number) -> number { return x * 2; }
            return double;
        }
        let f = getDouble();
        f(7);
    "#;
    assert_eval_number(source, 14.0);
}

#[test]
fn test_return_builtin() {
    let source = r#"
        fn getLen() -> (string) -> number {
            return len;
        }
        let f = getLen();
        f("test");
    "#;
    assert_eval_number(source, 4.0);
}

#[test]
fn test_return_function_from_parameter() {
    let source = r#"
        fn identity(f: (number) -> number) -> (number) -> number {
            return f;
        }
        fn triple(x: number) -> number { return x * 3; }
        let f = identity(triple);
        f(4);
    "#;
    assert_eval_number(source, 12.0);
}

#[test]
fn test_conditional_function_return() {
    let source = r#"
        fn getFunc(flag: bool) -> (number) -> number {
            fn double(x: number) -> number { return x * 2; }
            fn triple(x: number) -> number { return x * 3; }
            if (flag) {
                return double;
            }
            return triple;
        }
        let f = getFunc(true);
        f(5);
    "#;
    assert_eval_number(source, 10.0);
}

#[test]
fn test_return_function_and_call_immediately() {
    let source = r#"
        fn getDouble() -> (number) -> number {
            fn double(x: number) -> number { return x * 2; }
            return double;
        }
        getDouble()(6);
    "#;
    assert_eval_number(source, 12.0);
}

// ============================================================================
// Category 4: Type Checking (15 tests)
// ============================================================================

#[test]
fn test_type_error_wrong_function_type() {
    let source = r#"
        fn add(a: number, b: number) -> number { return a + b; }
        let f: (number) -> number = add;
    "#;
    assert_error_code(source, "AT3001");
}

#[test]
fn test_type_error_not_a_function() {
    let source = r#"
        let x: number = 5;
        x();
    "#;
    assert_error_code(source, "AT3006");
}

#[test]
fn test_type_error_wrong_return_type() {
    let source = r#"
        fn getString() -> string {
            fn getNum() -> number { return 42; }
            return getNum;
        }
    "#;
    assert_error_code(source, "AT3001");
}

#[test]
fn test_type_valid_function_assignment() {
    let source = r#"
        fn double(x: number) -> number { return x * 2; }
        let f: (number) -> number = double;
        f(5);
    "#;
    assert_eval_number(source, 10.0);
}

#[test]
fn test_type_valid_function_parameter() {
    let source = r#"
        fn apply(f: (string) -> number, s: string) -> number {
            return f(s);
        }
        apply(len, "test");
    "#;
    assert_eval_number(source, 4.0);
}

// ============================================================================
// Category 5: Edge Cases (15 tests)
// ============================================================================

#[test]
fn test_function_returning_void() {
    let source = r#"
        fn getVoid() -> (string) -> void {
            return print;
        }
        let f = getVoid();
        f("test");
    "#;
    assert_eval_null(source);
}

#[test]
fn test_nested_function_calls_through_variables() {
    let source = r#"
        fn add(a: number, b: number) -> number { return a + b; }
        let f = add;
        let g = f;
        let h = g;
        h(2, 3);
    "#;
    assert_eval_number(source, 5.0);
}

#[test]
fn test_function_with_no_params() {
    let source = r#"
        fn getFortyTwo() -> number { return 42; }
        let f: () -> number = getFortyTwo;
        f();
    "#;
    assert_eval_number(source, 42.0);
}

#[test]
fn test_function_with_many_params() {
    let source = r#"
        fn sum4(a: number, b: number, c: number, d: number) -> number {
            return a + b + c + d;
        }
        let f = sum4;
        f(1, 2, 3, 4);
    "#;
    assert_eval_number(source, 10.0);
}

#[test]
fn test_function_variable_in_global_scope() {
    let source = r#"
        fn double(x: number) -> number { return x * 2; }
        let globalFunc = double;
        fn useGlobal(x: number) -> number {
            return globalFunc(x);
        }
        useGlobal(5);
    "#;
    assert_eval_number(source, 10.0);
}

// ============================================================================
// Category 6: Integration Tests (15 tests)
// ============================================================================

#[test]
fn test_map_pattern_with_function() {
    let source = r#"
        fn applyToArray(arr: number[], f: (number) -> number) -> number[] {
            var result: number[] = [];
            for (var i = 0; i < len(arr); i++) {
                result = result + [f(arr[i])];
            }
            return result;
        }
        fn double(x: number) -> number { return x * 2; }
        let arr = [1, 2, 3];
        let doubled = applyToArray(arr, double);
        doubled[0] + doubled[1] + doubled[2];
    "#;
    assert_eval_number(source, 12.0);
}

#[test]
fn test_filter_pattern_with_function() {
    let source = r#"
        fn filterArray(arr: number[], predicate: (number) -> bool) -> number[] {
            var result: number[] = [];
            for (var i = 0; i < len(arr); i++) {
                if (predicate(arr[i])) {
                    result = result + [arr[i]];
                }
            }
            return result;
        }
        fn isEven(x: number) -> bool { return x % 2 == 0; }
        let arr = [1, 2, 3, 4, 5, 6];
        let evens = filterArray(arr, isEven);
        len(evens);
    "#;
    assert_eval_number(source, 3.0);
}

#[test]
fn test_reduce_pattern_with_function() {
    let source = r#"
        fn reduceArray(
            arr: number[],
            reducer: (number, number) -> number,
            initial: number
        ) -> number {
            var acc = initial;
            for (var i = 0; i < len(arr); i++) {
                acc = reducer(acc, arr[i]);
            }
            return acc;
        }
        fn add(a: number, b: number) -> number { return a + b; }
        let arr = [1, 2, 3, 4, 5];
        reduceArray(arr, add, 0);
    "#;
    assert_eval_number(source, 15.0);
}

#[test]
fn test_function_composition() {
    let source = r#"
        fn compose(
            f: (number) -> number,
            g: (number) -> number
        ) -> (number) -> number {
            fn composed(x: number) -> number {
                return f(g(x));
            }
            return composed;
        }
        fn double(x: number) -> number { return x * 2; }
        fn inc(x: number) -> number { return x + 1; }
        let doubleAndInc = compose(inc, double);
        doubleAndInc(5);
    "#;
    assert_eval_number(source, 11.0);
}

#[test]
fn test_callback_pattern() {
    let source = r#"
        fn processValue(
            x: number,
            callback: (number) -> void
        ) -> void {
            callback(x * 2);
        }
        var result = 0;
        fn setResult(x: number) -> void {
            result = x;
        }
        processValue(5, setResult);
        result;
    "#;
    assert_eval_number(source, 10.0);
}

#[test]
fn test_function_array_element() {
    let source = r#"
        fn double(x: number) -> number { return x * 2; }
        fn triple(x: number) -> number { return x * 3; }
        let funcs: ((number) -> number)[] = [double, triple];
        funcs[0](5) + funcs[1](5);
    "#;
    assert_eval_number(source, 25.0);
}

#[test]
fn test_complex_function_passing() {
    let source = r#"
        fn transform(
            arr: number[],
            f1: (number) -> number,
            f2: (number) -> number
        ) -> number {
            var sum = 0;
            for (var i = 0; i < len(arr); i++) {
                sum = sum + f1(f2(arr[i]));
            }
            return sum;
        }
        fn double(x: number) -> number { return x * 2; }
        fn square(x: number) -> number { return x * x; }
        transform([1, 2, 3], double, square);
    "#;
    assert_eval_number(source, 28.0);
}
