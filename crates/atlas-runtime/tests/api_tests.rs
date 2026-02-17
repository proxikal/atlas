//! Integration tests for Runtime API
//!
//! Tests the public embedding API for creating runtimes, evaluating code,
//! calling functions, and managing global variables.

use atlas_runtime::api::{EvalError, ExecutionMode, Runtime};
use atlas_runtime::Value;
use std::sync::Arc;

// Runtime Creation Tests

#[test]
fn test_runtime_creation_interpreter() {
    let runtime = Runtime::new(ExecutionMode::Interpreter);
    assert_eq!(runtime.mode(), ExecutionMode::Interpreter);
}

#[test]
fn test_runtime_creation_vm() {
    let runtime = Runtime::new(ExecutionMode::VM);
    assert_eq!(runtime.mode(), ExecutionMode::VM);
}

#[test]
fn test_runtime_with_custom_security() {
    use atlas_runtime::SecurityContext;
    let security = SecurityContext::allow_all();
    let runtime = Runtime::new_with_security(ExecutionMode::Interpreter, security);
    assert_eq!(runtime.mode(), ExecutionMode::Interpreter);
}

// Basic eval() Tests - Interpreter Mode

#[test]
fn test_eval_number_literal_interpreter() {
    let mut runtime = Runtime::new(ExecutionMode::Interpreter);
    let result = runtime.eval("42").unwrap();
    assert!(matches!(result, Value::Number(n) if n == 42.0));
}

#[test]
fn test_eval_string_literal_interpreter() {
    let mut runtime = Runtime::new(ExecutionMode::Interpreter);
    let result = runtime.eval("\"hello\"").unwrap();
    assert!(matches!(result, Value::String(s) if s.as_ref() == "hello"));
}

#[test]
fn test_eval_bool_true_interpreter() {
    let mut runtime = Runtime::new(ExecutionMode::Interpreter);
    let result = runtime.eval("true").unwrap();
    assert!(matches!(result, Value::Bool(true)));
}

#[test]
fn test_eval_bool_false_interpreter() {
    let mut runtime = Runtime::new(ExecutionMode::Interpreter);
    let result = runtime.eval("false").unwrap();
    assert!(matches!(result, Value::Bool(false)));
}

#[test]
fn test_eval_null_interpreter() {
    let mut runtime = Runtime::new(ExecutionMode::Interpreter);
    let result = runtime.eval("null").unwrap();
    assert!(matches!(result, Value::Null));
}

#[test]
fn test_eval_arithmetic_interpreter() {
    let mut runtime = Runtime::new(ExecutionMode::Interpreter);
    let result = runtime.eval("1 + 2").unwrap();
    assert!(matches!(result, Value::Number(n) if n == 3.0));
}

#[test]
fn test_eval_string_concat_interpreter() {
    let mut runtime = Runtime::new(ExecutionMode::Interpreter);
    let result = runtime.eval("\"hello\" + \" \" + \"world\"").unwrap();
    assert!(matches!(result, Value::String(s) if s.as_ref() == "hello world"));
}

#[test]
fn test_eval_comparison_interpreter() {
    let mut runtime = Runtime::new(ExecutionMode::Interpreter);
    let result = runtime.eval("5 > 3").unwrap();
    assert!(matches!(result, Value::Bool(true)));
}

#[test]
fn test_eval_logical_and_interpreter() {
    let mut runtime = Runtime::new(ExecutionMode::Interpreter);
    let result = runtime.eval("true && false").unwrap();
    assert!(matches!(result, Value::Bool(false)));
}

#[test]
fn test_eval_logical_or_interpreter() {
    let mut runtime = Runtime::new(ExecutionMode::Interpreter);
    let result = runtime.eval("true || false").unwrap();
    assert!(matches!(result, Value::Bool(true)));
}

// Basic eval() Tests - VM Mode

#[test]
fn test_eval_number_literal_vm() {
    let mut runtime = Runtime::new(ExecutionMode::VM);
    let result = runtime.eval("42").unwrap();
    assert!(matches!(result, Value::Number(n) if n == 42.0));
}

#[test]
fn test_eval_string_literal_vm() {
    let mut runtime = Runtime::new(ExecutionMode::VM);
    let result = runtime.eval("\"hello\"").unwrap();
    assert!(matches!(result, Value::String(s) if s.as_ref() == "hello"));
}

#[test]
fn test_eval_bool_true_vm() {
    let mut runtime = Runtime::new(ExecutionMode::VM);
    let result = runtime.eval("true").unwrap();
    assert!(matches!(result, Value::Bool(true)));
}

#[test]
fn test_eval_bool_false_vm() {
    let mut runtime = Runtime::new(ExecutionMode::VM);
    let result = runtime.eval("false").unwrap();
    assert!(matches!(result, Value::Bool(false)));
}

#[test]
fn test_eval_null_vm() {
    let mut runtime = Runtime::new(ExecutionMode::VM);
    let result = runtime.eval("null").unwrap();
    assert!(matches!(result, Value::Null));
}

#[test]
fn test_eval_arithmetic_vm() {
    let mut runtime = Runtime::new(ExecutionMode::VM);
    let result = runtime.eval("1 + 2").unwrap();
    assert!(matches!(result, Value::Number(n) if n == 3.0));
}

#[test]
fn test_eval_string_concat_vm() {
    let mut runtime = Runtime::new(ExecutionMode::VM);
    let result = runtime.eval("\"hello\" + \" \" + \"world\"").unwrap();
    assert!(matches!(result, Value::String(s) if s.as_ref() == "hello world"));
}

#[test]
fn test_eval_comparison_vm() {
    let mut runtime = Runtime::new(ExecutionMode::VM);
    let result = runtime.eval("5 > 3").unwrap();
    assert!(matches!(result, Value::Bool(true)));
}

#[test]
fn test_eval_logical_and_vm() {
    let mut runtime = Runtime::new(ExecutionMode::VM);
    let result = runtime.eval("true && false").unwrap();
    assert!(matches!(result, Value::Bool(false)));
}

#[test]
fn test_eval_logical_or_vm() {
    let mut runtime = Runtime::new(ExecutionMode::VM);
    let result = runtime.eval("true || false").unwrap();
    assert!(matches!(result, Value::Bool(true)));
}

// State Persistence Tests - Interpreter
// Note: Cross-eval state persistence requires persistent symbol tables
// This is a future enhancement. For v0.2 phase-01, use single-eval programs
// or set_global/get_global for programmatic state management.

#[test]
fn test_single_eval_with_multiple_statements() {
    let mut runtime = Runtime::new(ExecutionMode::Interpreter);
    // Variables defined and used in the same eval() work fine
    let result = runtime
        .eval("var x: number = 1; var y: number = 2; x + y")
        .unwrap();
    assert!(matches!(result, Value::Number(n) if n == 3.0));
}

#[test]
fn test_function_definition_and_call_single_eval() {
    let mut runtime = Runtime::new(ExecutionMode::Interpreter);
    // Define and call function in the same eval()
    let result = runtime
        .eval("fn add(x: number, y: number) -> number { return x + y; } add(10, 20)")
        .unwrap();
    assert!(matches!(result, Value::Number(n) if n == 30.0));
}

#[test]
fn test_function_multiple_calls_single_eval() {
    let mut runtime = Runtime::new(ExecutionMode::Interpreter);
    let result = runtime
        .eval("fn square(x: number) -> number { return x * x; } square(3) + square(4)")
        .unwrap();
    assert!(matches!(result, Value::Number(n) if n == 25.0));
}

// State Persistence Tests - VM

#[test]
fn test_global_variable_persistence_vm() {
    let mut runtime = Runtime::new(ExecutionMode::VM);
    runtime.eval("var x: number = 42;").unwrap();
    // Note: VM doesn't persist state yet in this phase
    // This test documents current limitation
}

#[test]
fn test_function_definition_persistence_vm() {
    let mut runtime = Runtime::new(ExecutionMode::VM);
    runtime
        .eval("fn add(x: number, y: number) -> number { return x + y; }")
        .unwrap();
    // Note: VM doesn't persist state yet in this phase
}

// Error Handling Tests

#[test]
fn test_eval_parse_error_missing_semicolon() {
    let mut runtime = Runtime::new(ExecutionMode::Interpreter);
    let result = runtime.eval("let x: number =");
    assert!(matches!(result, Err(EvalError::ParseError(_))));
}

#[test]
fn test_eval_parse_error_invalid_syntax() {
    let mut runtime = Runtime::new(ExecutionMode::Interpreter);
    let result = runtime.eval("@#$%");
    assert!(matches!(result, Err(EvalError::ParseError(_))));
}

#[test]
fn test_eval_type_error_wrong_type() {
    let mut runtime = Runtime::new(ExecutionMode::Interpreter);
    let result = runtime.eval("let x: number = \"hello\";");
    assert!(matches!(result, Err(EvalError::TypeError(_))));
}

#[test]
fn test_eval_type_error_arithmetic_on_string() {
    let mut runtime = Runtime::new(ExecutionMode::Interpreter);
    let result = runtime.eval("\"hello\" - \"world\"");
    assert!(matches!(result, Err(EvalError::TypeError(_))));
}

#[test]
fn test_eval_runtime_error_divide_by_zero() {
    let mut runtime = Runtime::new(ExecutionMode::Interpreter);
    let result = runtime.eval("1 / 0");
    assert!(matches!(result, Err(EvalError::RuntimeError(_))));
}

#[test]
fn test_eval_type_error_undefined_variable() {
    let mut runtime = Runtime::new(ExecutionMode::Interpreter);
    let result = runtime.eval("nonexistent");
    // Undefined variables are caught at binding/typecheck phase
    assert!(result.is_err());
}

// call() Function Tests

#[test]
fn test_call_builtin_print() {
    let mut runtime = Runtime::new(ExecutionMode::Interpreter);
    let result = runtime.call("print", vec![Value::Number(42.0)]).unwrap();
    assert!(matches!(result, Value::Null));
}

#[test]
fn test_call_builtin_str() {
    let mut runtime = Runtime::new(ExecutionMode::Interpreter);
    let result = runtime.call("str", vec![Value::Number(42.0)]).unwrap();
    assert!(matches!(result, Value::String(s) if s.as_ref() == "42"));
}

#[test]
fn test_call_builds_on_eval() {
    let mut runtime = Runtime::new(ExecutionMode::Interpreter);
    // call() uses eval() internally, so functions must be defined inline
    let result = runtime.call("str", vec![Value::Number(42.0)]).unwrap();
    assert!(matches!(result, Value::String(s) if s.as_ref() == "42"));
}

// get_global/set_global Tests - Interpreter Mode

#[test]
fn test_set_global_number_interpreter() {
    let mut runtime = Runtime::new(ExecutionMode::Interpreter);
    runtime.set_global("x", Value::Number(42.0));
    let value = runtime.get_global("x").unwrap();
    assert!(matches!(value, Value::Number(n) if n == 42.0));
}

#[test]
fn test_set_global_string_interpreter() {
    let mut runtime = Runtime::new(ExecutionMode::Interpreter);
    runtime.set_global("message", Value::String(Arc::new("hello".to_string())));
    let value = runtime.get_global("message").unwrap();
    assert!(matches!(value, Value::String(s) if s.as_ref() == "hello"));
}

#[test]
fn test_set_global_bool_interpreter() {
    let mut runtime = Runtime::new(ExecutionMode::Interpreter);
    runtime.set_global("flag", Value::Bool(true));
    let value = runtime.get_global("flag").unwrap();
    assert!(matches!(value, Value::Bool(true)));
}

#[test]
fn test_set_global_null_interpreter() {
    let mut runtime = Runtime::new(ExecutionMode::Interpreter);
    runtime.set_global("nothing", Value::Null);
    let value = runtime.get_global("nothing").unwrap();
    assert!(matches!(value, Value::Null));
}

#[test]
fn test_get_global_nonexistent_interpreter() {
    let runtime = Runtime::new(ExecutionMode::Interpreter);
    let value = runtime.get_global("nonexistent");
    assert!(value.is_none());
}

#[test]
fn test_set_global_and_get_global_roundtrip() {
    let mut runtime = Runtime::new(ExecutionMode::Interpreter);
    runtime.set_global("x", Value::Number(100.0));
    let value = runtime.get_global("x").unwrap();
    assert!(matches!(value, Value::Number(n) if n == 100.0));
    // Note: Using set_global'd variables in eval() requires symbol table persistence (future phase)
}

#[test]
fn test_set_global_overwrite_interpreter() {
    let mut runtime = Runtime::new(ExecutionMode::Interpreter);
    runtime.set_global("x", Value::Number(10.0));
    runtime.set_global("x", Value::Number(20.0));
    let value = runtime.get_global("x").unwrap();
    assert!(matches!(value, Value::Number(n) if n == 20.0));
}

// get_global/set_global Tests - VM Mode (current limitations)

#[test]
fn test_get_global_vm_returns_none() {
    let runtime = Runtime::new(ExecutionMode::VM);
    // VM mode doesn't support direct global access yet
    let value = runtime.get_global("x");
    assert!(value.is_none());
}

// Mode Parity Tests

#[test]
fn test_parity_arithmetic_expression() {
    let mut interp = Runtime::new(ExecutionMode::Interpreter);
    let mut vm = Runtime::new(ExecutionMode::VM);

    let expr = "((10 + 5) * 2) - 3";
    let interp_result = interp.eval(expr).unwrap();
    let vm_result = vm.eval(expr).unwrap();

    assert!(matches!(interp_result, Value::Number(n) if n == 27.0));
    assert!(matches!(vm_result, Value::Number(n) if n == 27.0));
}

#[test]
fn test_parity_string_operations() {
    let mut interp = Runtime::new(ExecutionMode::Interpreter);
    let mut vm = Runtime::new(ExecutionMode::VM);

    let expr = "\"hello\" + \" \" + \"world\"";
    let interp_result = interp.eval(expr).unwrap();
    let vm_result = vm.eval(expr).unwrap();

    assert!(matches!(interp_result, Value::String(s) if s.as_ref() == "hello world"));
    assert!(matches!(vm_result, Value::String(s) if s.as_ref() == "hello world"));
}

#[test]
fn test_parity_boolean_logic() {
    let mut interp = Runtime::new(ExecutionMode::Interpreter);
    let mut vm = Runtime::new(ExecutionMode::VM);

    let expr = "(true && false) || (false || true)";
    let interp_result = interp.eval(expr).unwrap();
    let vm_result = vm.eval(expr).unwrap();

    assert!(matches!(interp_result, Value::Bool(true)));
    assert!(matches!(vm_result, Value::Bool(true)));
}

// Complex Program Tests

#[test]
fn test_complex_program_with_control_flow_interpreter() {
    let mut runtime = Runtime::new(ExecutionMode::Interpreter);
    let program = r#"
        fn factorial(n: number) -> number {
            if (n <= 1) {
                return 1;
            } else {
                return n * factorial(n - 1);
            }
        }
        factorial(5)
    "#;
    let result = runtime.eval(program).unwrap();
    assert!(matches!(result, Value::Number(n) if n == 120.0));
}

#[test]
fn test_complex_program_with_loops_interpreter() {
    let mut runtime = Runtime::new(ExecutionMode::Interpreter);
    let program = r#"
        var sum: number = 0;
        for (var i: number = 1; i <= 10; i = i + 1) {
            sum = sum + i;
        }
        sum
    "#;
    let result = runtime.eval(program).unwrap();
    assert!(matches!(result, Value::Number(n) if n == 55.0));
}

#[test]
fn test_multiple_function_definitions_single_eval() {
    let mut runtime = Runtime::new(ExecutionMode::Interpreter);
    // Define all functions in a single eval()
    let result = runtime
        .eval(
            r#"
            fn add(x: number, y: number) -> number { return x + y; }
            fn sub(x: number, y: number) -> number { return x - y; }
            fn mul(x: number, y: number) -> number { return x * y; }
            add(10, sub(20, mul(2, 3)))
        "#,
        )
        .unwrap();
    assert!(matches!(result, Value::Number(n) if n == 24.0));
}
