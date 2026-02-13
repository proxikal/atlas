//! Integration tests for bytecode compiler features
//! Tests that code compiles AND executes correctly via VM

use atlas_runtime::compiler::Compiler;
use atlas_runtime::lexer::Lexer;
use atlas_runtime::parser::Parser;
use atlas_runtime::value::Value;
use atlas_runtime::vm::VM;

fn execute_source(source: &str) -> Result<Option<Value>, atlas_runtime::value::RuntimeError> {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, lex_diags) = lexer.tokenize();
    assert!(lex_diags.is_empty(), "Lexer errors: {:?}", lex_diags);

    let mut parser = Parser::new(tokens);
    let (program, parse_diags) = parser.parse();
    assert!(parse_diags.is_empty(), "Parser errors: {:?}", parse_diags);

    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&program).expect("Compilation failed");

    let mut vm = VM::new(bytecode);
    vm.run()
}

#[test]
fn test_array_index_assignment_execution() {
    let result = execute_source(r#"
        let arr = [1, 2, 3];
        arr[1] = 42;
        arr[1];
    "#);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(Value::Number(42.0)));
}

#[test]
fn test_compound_assignment_add_execution() {
    let result = execute_source(r#"
        let x = 10;
        x += 5;
        x;
    "#);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(Value::Number(15.0)));
}

#[test]
fn test_compound_assignment_sub_execution() {
    let result = execute_source(r#"
        let x = 10;
        x -= 3;
        x;
    "#);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(Value::Number(7.0)));
}

#[test]
fn test_compound_assignment_mul_execution() {
    let result = execute_source(r#"
        let x = 4;
        x *= 3;
        x;
    "#);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(Value::Number(12.0)));
}

#[test]
fn test_compound_assignment_div_execution() {
    let result = execute_source(r#"
        let x = 20;
        x /= 4;
        x;
    "#);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(Value::Number(5.0)));
}

#[test]
fn test_compound_assignment_mod_execution() {
    let result = execute_source(r#"
        let x = 17;
        x %= 5;
        x;
    "#);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(Value::Number(2.0)));
}

#[test]
fn test_increment_execution() {
    let result = execute_source(r#"
        let x = 5;
        x++;
        x;
    "#);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(Value::Number(6.0)));
}

#[test]
fn test_decrement_execution() {
    let result = execute_source(r#"
        let x = 5;
        x--;
        x;
    "#);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(Value::Number(4.0)));
}

#[test]
fn test_increment_on_array_element() {
    let result = execute_source(r#"
        let arr = [5, 10, 15];
        arr[1]++;
        arr[1];
    "#);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(Value::Number(11.0)));
}

// NOTE: Assignment expressions are NOT in v0.1 scope (Atlas-SPEC.md line 68)
// "Increment/decrement operators are statements, not expressions"
// These tests are for v0.2+ when assignment expressions are added
#[test]
#[ignore = "Assignment expressions not in v0.1 - planned for v0.2"]
fn test_short_circuit_and_false() {
    // Right side should not be evaluated
    let result = execute_source(r#"
        let x = 0;
        let result = false && (x = 1);
        x; // Should still be 0
    "#);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(Value::Number(0.0)));
}

#[test]
#[ignore = "Assignment expressions not in v0.1 - planned for v0.2"]
fn test_short_circuit_and_true() {
    // Right side should be evaluated
    let result = execute_source(r#"
        let x = 0;
        let result = true && (x = 1);
        x; // Should be 1
    "#);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(Value::Number(1.0)));
}

#[test]
#[ignore = "Assignment expressions not in v0.1 - planned for v0.2"]
fn test_short_circuit_or_true() {
    // Right side should not be evaluated
    let result = execute_source(r#"
        let x = 0;
        let result = true || (x = 1);
        x; // Should still be 0
    "#);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(Value::Number(0.0)));
}

#[test]
#[ignore = "Assignment expressions not in v0.1 - planned for v0.2"]
fn test_short_circuit_or_false() {
    // Right side should be evaluated
    let result = execute_source(r#"
        let x = 0;
        let result = false || (x = 1);
        x; // Should be 1
    "#);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(Value::Number(1.0)));
}

#[test]
fn test_user_function_simple() {
    let result = execute_source(r#"
        fn double(x: number) -> number {
            return x * 2;
        }
        double(21);
    "#);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(Value::Number(42.0)));
}

#[test]
fn test_user_function_with_multiple_params() {
    let result = execute_source(r#"
        fn add(a: number, b: number) -> number {
            return a + b;
        }
        add(10, 32);
    "#);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(Value::Number(42.0)));
}

#[test]
fn test_user_function_recursion() {
    let result = execute_source(r#"
        fn factorial(n: number) -> number {
            if (n <= 1) {
                return 1;
            }
            return n * factorial(n - 1);
        }
        factorial(5);
    "#);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(Value::Number(120.0)));
}

#[test]
fn test_user_function_with_local_variables() {
    let result = execute_source(r#"
        fn calculate(x: number) -> number {
            let y = x * 2;
            let z = y + 10;
            return z;
        }
        calculate(5);
    "#);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(Value::Number(20.0)));
}

#[test]
fn test_multiple_functions() {
    let result = execute_source(r#"
        fn double(x: number) -> number {
            return x * 2;
        }
        fn triple(x: number) -> number {
            return x * 3;
        }
        double(7) + triple(4);
    "#);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(Value::Number(26.0)));
}

#[test]
fn test_function_calling_function() {
    let result = execute_source(r#"
        fn add(a: number, b: number) -> number {
            return a + b;
        }
        fn addThree(a: number, b: number, c: number) -> number {
            return add(add(a, b), c);
        }
        addThree(10, 20, 12);
    "#);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(Value::Number(42.0)));
}
