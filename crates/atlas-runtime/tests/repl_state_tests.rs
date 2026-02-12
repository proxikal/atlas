//! REPL state persistence tests
//!
//! These tests verify that:
//! - Variable declarations persist across REPL inputs
//! - Function declarations persist across REPL inputs
//! - Errors do not reset REPL state
//! - Variables can be reassigned across inputs
//! - Functions can call other functions defined in previous inputs

use atlas_runtime::{ReplCore, Value};

/// Helper to evaluate and expect success
fn eval_ok(repl: &mut ReplCore, input: &str) -> Value {
    let result = repl.eval_line(input);
    if !result.diagnostics.is_empty() {
        panic!(
            "Expected success for input: {}\nGot diagnostics: {:?}",
            input, result.diagnostics
        );
    }
    result.value.unwrap_or(Value::Null)
}

/// Helper to evaluate and expect error
fn eval_err(repl: &mut ReplCore, input: &str) {
    let result = repl.eval_line(input);
    if result.diagnostics.is_empty() {
        panic!("Expected error for input: {}, but evaluation succeeded", input);
    }
}

#[test]
fn test_variable_persistence_across_inputs() {
    let mut repl = ReplCore::new();

    // Define variable in first input
    eval_ok(&mut repl, "let x = 42;");

    // Use variable in second input
    let value = eval_ok(&mut repl, "x;");
    assert_eq!(value, Value::Number(42.0));

    // Use variable in expression
    let value = eval_ok(&mut repl, "x + 8;");
    assert_eq!(value, Value::Number(50.0));
}

#[test]
fn test_mutable_variable_reassignment_across_inputs() {
    let mut repl = ReplCore::new();

    // Define mutable variable
    eval_ok(&mut repl, "var count = 0;");

    // Reassign in next input
    eval_ok(&mut repl, "count = count + 1;");

    // Verify new value
    let value = eval_ok(&mut repl, "count;");
    assert_eq!(value, Value::Number(1.0));

    // Reassign again
    eval_ok(&mut repl, "count = count + 10;");

    let value = eval_ok(&mut repl, "count;");
    assert_eq!(value, Value::Number(11.0));
}

#[test]
fn test_multiple_variables_persist() {
    let mut repl = ReplCore::new();

    // Define multiple variables across different inputs
    eval_ok(&mut repl, "let a = 1;");
    eval_ok(&mut repl, "let b = 2;");
    eval_ok(&mut repl, "let c = 3;");

    // Use all variables together
    let value = eval_ok(&mut repl, "a + b + c;");
    assert_eq!(value, Value::Number(6.0));
}

#[test]
fn test_function_persistence_across_inputs() {
    let mut repl = ReplCore::new();

    // Define function in first input
    eval_ok(&mut repl, "fn double(x: number) -> number { return x * 2; }");

    // Call function in second input
    let value = eval_ok(&mut repl, "double(21);");
    assert_eq!(value, Value::Number(42.0));

    // Call function with different argument
    let value = eval_ok(&mut repl, "double(100);");
    assert_eq!(value, Value::Number(200.0));
}

#[test]
fn test_function_calling_another_function() {
    let mut repl = ReplCore::new();

    // Define first function
    eval_ok(&mut repl, "fn add(a: number, b: number) -> number { return a + b; }");

    // Define second function that calls first
    eval_ok(&mut repl, "fn add_ten(x: number) -> number { return add(x, 10); }");

    // Call the second function
    let value = eval_ok(&mut repl, "add_ten(5);");
    assert_eq!(value, Value::Number(15.0));
}

#[test]
fn test_variable_used_in_function_definition() {
    let mut repl = ReplCore::new();

    // Define variable first
    eval_ok(&mut repl, "var multiplier = 3;");

    // Define function that uses the variable
    eval_ok(&mut repl, "fn scale(x: number) -> number { return x * multiplier; }");

    // Call function
    let value = eval_ok(&mut repl, "scale(10);");
    assert_eq!(value, Value::Number(30.0));

    // Change variable and call again
    eval_ok(&mut repl, "multiplier = 5;");
    let value = eval_ok(&mut repl, "scale(10);");
    assert_eq!(value, Value::Number(50.0));
}

#[test]
fn test_error_does_not_reset_state() {
    let mut repl = ReplCore::new();

    // Define variable
    eval_ok(&mut repl, "let x = 42;");

    // Cause an error (type error)
    eval_err(&mut repl, "let y: string = 123;");

    // Verify previous variable still exists
    let value = eval_ok(&mut repl, "x;");
    assert_eq!(value, Value::Number(42.0));

    // Define new variable after error
    eval_ok(&mut repl, "let z = 100;");

    // Both variables should exist
    let value = eval_ok(&mut repl, "x + z;");
    assert_eq!(value, Value::Number(142.0));
}

#[test]
fn test_runtime_error_does_not_reset_state() {
    let mut repl = ReplCore::new();

    // Define function and variable
    eval_ok(&mut repl, "let x = 10;");
    eval_ok(&mut repl, "fn divide(a: number, b: number) -> number { return a / b; }");

    // Cause runtime error (undefined variable)
    eval_err(&mut repl, "undefined_var + 1;");

    // Verify state still exists
    let value = eval_ok(&mut repl, "x;");
    assert_eq!(value, Value::Number(10.0));

    let value = eval_ok(&mut repl, "divide(20, 2);");
    assert_eq!(value, Value::Number(10.0));
}

#[test]
fn test_shadowing_in_nested_scope() {
    let mut repl = ReplCore::new();

    // Define variable at top level
    eval_ok(&mut repl, "let x = 1;");

    // Shadow in a nested scope (within a function)
    eval_ok(&mut repl, "fn test_shadow() -> number { let x = 2; return x; }");

    // Call function returns shadowed value
    let value = eval_ok(&mut repl, "test_shadow();");
    assert_eq!(value, Value::Number(2.0));

    // But top-level x is unchanged
    let value = eval_ok(&mut repl, "x;");
    assert_eq!(value, Value::Number(1.0));
}

#[test]
fn test_array_persistence() {
    let mut repl = ReplCore::new();

    // Define array
    eval_ok(&mut repl, "var arr = [1, 2, 3];");

    // Access array in next input
    let value = eval_ok(&mut repl, "arr[0];");
    assert_eq!(value, Value::Number(1.0));

    // Modify array
    eval_ok(&mut repl, "arr[1] = 99;");

    // Verify modification persisted
    let value = eval_ok(&mut repl, "arr[1];");
    assert_eq!(value, Value::Number(99.0));
}

#[test]
fn test_string_persistence() {
    let mut repl = ReplCore::new();

    // Define string variable
    eval_ok(&mut repl, r#"let message = "Hello";"#);

    // Use in next input
    let value = eval_ok(&mut repl, "message;");
    assert_eq!(value, Value::String("Hello".to_string().into()));

    // Concatenate in expression
    let value = eval_ok(&mut repl, r#"message + " World";"#);
    assert_eq!(value, Value::String("Hello World".to_string().into()));
}

#[test]
fn test_reset_clears_all_state() {
    let mut repl = ReplCore::new();

    // Define variable and function
    eval_ok(&mut repl, "let x = 42;");
    eval_ok(&mut repl, "fn foo() -> number { return x; }");

    // Verify they exist
    let value = eval_ok(&mut repl, "x;");
    assert_eq!(value, Value::Number(42.0));

    // Reset REPL
    repl.reset();

    // Variables should no longer exist
    eval_err(&mut repl, "x;");
    eval_err(&mut repl, "foo();");

    // But we can define new ones
    eval_ok(&mut repl, "let x = 100;");
    let value = eval_ok(&mut repl, "x;");
    assert_eq!(value, Value::Number(100.0));
}

#[test]
fn test_control_flow_with_persisted_variables() {
    let mut repl = ReplCore::new();

    // Define variable
    eval_ok(&mut repl, "let threshold = 10;");

    // Use in if statement in next input
    eval_ok(&mut repl, "var result = 0;");
    eval_ok(
        &mut repl,
        "if (threshold > 5) { result = 100; } else { result = 50; }",
    );

    let value = eval_ok(&mut repl, "result;");
    assert_eq!(value, Value::Number(100.0));
}

#[test]
fn test_loop_with_persisted_counter() {
    let mut repl = ReplCore::new();

    // Define counter
    eval_ok(&mut repl, "var counter = 0;");

    // Run loop that modifies counter
    eval_ok(&mut repl, "for (let i = 0; i < 5; i = i + 1) { counter = counter + 1; }");

    // Verify counter was updated
    let value = eval_ok(&mut repl, "counter;");
    assert_eq!(value, Value::Number(5.0));

    // Run another loop
    eval_ok(&mut repl, "for (let i = 0; i < 3; i = i + 1) { counter = counter + 2; }");

    // Counter should have accumulated
    let value = eval_ok(&mut repl, "counter;");
    assert_eq!(value, Value::Number(11.0));
}

#[test]
fn test_expression_result_vs_statement() {
    let mut repl = ReplCore::new();

    // Statement produces Null
    let value = eval_ok(&mut repl, "let x = 42;");
    assert_eq!(value, Value::Null);

    // Expression produces value
    let value = eval_ok(&mut repl, "x + 8;");
    assert_eq!(value, Value::Number(50.0));

    // Assignment produces Null
    eval_ok(&mut repl, "var y = x;");
    let value = eval_ok(&mut repl, "y = 100;");
    assert_eq!(value, Value::Null);

    // But variable is updated
    let value = eval_ok(&mut repl, "y;");
    assert_eq!(value, Value::Number(100.0));
}

#[test]
fn test_complex_state_persistence_scenario() {
    let mut repl = ReplCore::new();

    // Build up complex state across multiple inputs
    eval_ok(&mut repl, "var sum = 0;");
    eval_ok(&mut repl, "fn add_to_sum(n: number) -> void { sum = sum + n; }");
    eval_ok(&mut repl, "add_to_sum(10);");
    eval_ok(&mut repl, "add_to_sum(20);");
    eval_ok(&mut repl, "add_to_sum(30);");

    let value = eval_ok(&mut repl, "sum;");
    assert_eq!(value, Value::Number(60.0));

    // Define another function that uses sum
    eval_ok(&mut repl, "fn get_average(count: number) -> number { return sum / count; }");

    let value = eval_ok(&mut repl, "get_average(3);");
    assert_eq!(value, Value::Number(20.0));
}

#[test]
fn test_multiple_parse_errors_do_not_affect_state() {
    let mut repl = ReplCore::new();

    // Define valid state
    eval_ok(&mut repl, "let x = 1;");
    eval_ok(&mut repl, "let y = 2;");

    // Multiple parse errors
    eval_err(&mut repl, "let z = ;");
    eval_err(&mut repl, "fn foo(");
    eval_err(&mut repl, "if (true) }");

    // State should still be intact
    let value = eval_ok(&mut repl, "x + y;");
    assert_eq!(value, Value::Number(3.0));
}

#[test]
fn test_builtin_functions_always_available() {
    let mut repl = ReplCore::new();

    // Builtins should work from the start
    eval_ok(&mut repl, r#"print("Hello");"#);

    // Define variables
    eval_ok(&mut repl, r#"let message = "Test";"#);
    eval_ok(&mut repl, "let arr = [1, 2, 3];");

    // Builtins should still work with state
    eval_ok(&mut repl, "print(message);");
    eval_ok(&mut repl, "len(arr);");

    // After reset, builtins should still be available
    repl.reset();
    eval_ok(&mut repl, r#"print("After reset");"#);
}
