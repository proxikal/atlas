//! interpreter.rs — merged from 10 files + integration subtree (Phase Infra-02)

mod common;

use atlas_runtime::binder::Binder;
use atlas_runtime::compiler::Compiler;
use atlas_runtime::diagnostic::{Diagnostic, DiagnosticLevel};
use atlas_runtime::interpreter::Interpreter;
use atlas_runtime::lexer::Lexer;
use atlas_runtime::parser::Parser;
use atlas_runtime::security::SecurityContext;
use atlas_runtime::typechecker::generics::Monomorphizer;
use atlas_runtime::typechecker::TypeChecker;
use atlas_runtime::types::{Type, TypeParamDef};
use atlas_runtime::value::Value;
use atlas_runtime::Atlas;
use common::*;
use pretty_assertions::assert_eq;
use rstest::rstest;

// ============================================================================
// From interpreter_member_tests.rs
// ============================================================================

fn run_interpreter(source: &str) -> Result<String, String> {
    let mut lexer = Lexer::new(source);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();
    let mut binder = Binder::new();
    let (mut symbol_table, _) = binder.bind(&program);
    let mut typechecker = TypeChecker::new(&mut symbol_table);
    let _ = typechecker.check(&program);
    let mut interpreter = Interpreter::new();
    match interpreter.eval(&program, &SecurityContext::allow_all()) {
        Ok(value) => Ok(format!("{:?}", value)),
        Err(e) => Err(format!("{:?}", e)),
    }
}

// JSON as_string() Tests
#[rstest]
#[case(
    r#"let data: json = parseJSON("{\"name\":\"Alice\"}"); data["name"].as_string();"#,
    r#"String("Alice")"#
)]
#[case(r#"let data: json = parseJSON("{\"user\":{\"name\":\"Bob\"}}"); data["user"]["name"].as_string();"#, r#"String("Bob")"#)]
fn test_json_as_string(#[case] source: &str, #[case] expected: &str) {
    let result = run_interpreter(source).expect("Should succeed");
    assert_eq!(result, expected);
}

#[test]
fn test_json_as_string_error() {
    let result =
        run_interpreter(r#"let data: json = parseJSON("{\"age\":30}"); data["age"].as_string();"#);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Cannot extract string"));
}

// JSON as_number() Tests
#[rstest]
#[case(
    r#"let data: json = parseJSON("{\"age\":30}"); data["age"].as_number();"#,
    "Number(30.0)"
)]
#[case(
    r#"let data: json = parseJSON("{\"price\":19.99}"); data["price"].as_number();"#,
    "Number(19.99)"
)]
fn test_json_as_number(#[case] source: &str, #[case] expected: &str) {
    let result = run_interpreter(source).expect("Should succeed");
    assert_eq!(result, expected);
}

#[test]
fn test_json_as_number_error() {
    let result = run_interpreter(
        r#"let data: json = parseJSON("{\"name\":\"Alice\"}"); data["name"].as_number();"#,
    );
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Cannot extract number"));
}

// JSON as_bool() Tests
#[rstest]
#[case(
    r#"let data: json = parseJSON("{\"active\":true}"); data["active"].as_bool();"#,
    "Bool(true)"
)]
#[case(
    r#"let data: json = parseJSON("{\"disabled\":false}"); data["disabled"].as_bool();"#,
    "Bool(false)"
)]
fn test_json_as_bool(#[case] source: &str, #[case] expected: &str) {
    let result = run_interpreter(source).expect("Should succeed");
    assert_eq!(result, expected);
}

#[test]
fn test_json_as_bool_error() {
    let result =
        run_interpreter(r#"let data: json = parseJSON("{\"count\":5}"); data["count"].as_bool();"#);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Cannot extract bool"));
}

// JSON is_null() Tests
#[rstest]
#[case(
    r#"let data: json = parseJSON("{\"value\":null}"); data["value"].is_null();"#,
    "Bool(true)"
)]
#[case(
    r#"let data: json = parseJSON("{\"value\":\"text\"}"); data["value"].is_null();"#,
    "Bool(false)"
)]
#[case(
    r#"let data: json = parseJSON("{\"value\":42}"); data["value"].is_null();"#,
    "Bool(false)"
)]
fn test_json_is_null(#[case] source: &str, #[case] expected: &str) {
    let result = run_interpreter(source).expect("Should succeed");
    assert_eq!(result, expected);
}

// Complex Tests
#[test]
fn test_chained_methods() {
    let result = run_interpreter(
        r#"
        let data: json = parseJSON("{\"user\":{\"name\":\"Charlie\"}}");
        data["user"]["name"].as_string();
    "#,
    )
    .expect("Should succeed");
    assert_eq!(result, r#"String("Charlie")"#);
}

#[test]
fn test_method_in_expression() {
    let result = run_interpreter(
        r#"
        let data: json = parseJSON("{\"count\":5}");
        data["count"].as_number() + 10;
    "#,
    )
    .expect("Should succeed");
    assert_eq!(result, "Number(15.0)");
}

#[test]
fn test_method_in_conditional() {
    let result = run_interpreter(
        r#"
        let data: json = parseJSON("{\"enabled\":true}");
        var result: string = "no";
        if (data["enabled"].as_bool()) {
            result = "yes";
        };
        result;
    "#,
    )
    .expect("Should succeed");
    assert_eq!(result, r#"String("yes")"#);
}

#[test]
fn test_multiple_methods() {
    let result = run_interpreter(
        r#"
        let data: json = parseJSON("{\"a\":5,\"b\":10}");
        data["a"].as_number() + data["b"].as_number();
    "#,
    )
    .expect("Should succeed");
    assert_eq!(result, "Number(15.0)");
}

// Error Cases
#[test]
fn test_as_string_on_null() {
    let result =
        run_interpreter(r#"let data: json = parseJSON("{\"v\":null}"); data["v"].as_string();"#);
    assert!(result.is_err());
}

#[test]
fn test_as_number_on_null() {
    let result =
        run_interpreter(r#"let data: json = parseJSON("{\"v\":null}"); data["v"].as_number();"#);
    assert!(result.is_err());
}

#[test]
fn test_as_bool_on_null() {
    let result =
        run_interpreter(r#"let data: json = parseJSON("{\"v\":null}"); data["v"].as_bool();"#);
    assert!(result.is_err());
}

#[test]
fn test_as_string_on_object() {
    let result = run_interpreter(
        r#"let data: json = parseJSON("{\"obj\":{\"a\":1}}"); data["obj"].as_string();"#,
    );
    assert!(result.is_err());
}

#[test]
fn test_as_number_on_array() {
    let result = run_interpreter(
        r#"let data: json = parseJSON("{\"arr\":[1,2,3]}"); data["arr"].as_number();"#,
    );
    assert!(result.is_err());
}

// ============================================================================
// From nested_function_binding_tests.rs
// ============================================================================

fn parse_and_bind(source: &str) -> (Vec<String>, Vec<String>) {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, parse_diagnostics) = parser.parse();

    let mut binder = Binder::new();
    let (_symbol_table, bind_diagnostics) = binder.bind(&program);

    let parse_errors: Vec<String> = parse_diagnostics
        .iter()
        .map(|d| format!("{}: {}", d.code, d.message))
        .collect();

    let bind_errors: Vec<String> = bind_diagnostics
        .iter()
        .map(|d| format!("{}: {}", d.code, d.message))
        .collect();

    (parse_errors, bind_errors)
}

// ============================================================================
// Basic Nested Function Binding
// ============================================================================

#[test]
fn test_bind_nested_function_basic() {
    let source = r#"
        fn outer() -> number {
            fn helper(x: number) -> number {
                return x * 2;
            }
            return helper(21);
        }
    "#;

    let (parse_errors, bind_errors) = parse_and_bind(source);

    assert_eq!(parse_errors.len(), 0, "Parser errors: {:?}", parse_errors);
    assert_eq!(bind_errors.len(), 0, "Binder errors: {:?}", bind_errors);
}

#[test]
fn test_bind_multiple_nested_functions() {
    let source = r#"
        fn outer() -> number {
            fn add(a: number, b: number) -> number {
                return a + b;
            }
            fn multiply(a: number, b: number) -> number {
                return a * b;
            }
            return add(2, multiply(3, 4));
        }
    "#;

    let (parse_errors, bind_errors) = parse_and_bind(source);

    assert_eq!(parse_errors.len(), 0, "Parser errors: {:?}", parse_errors);
    assert_eq!(bind_errors.len(), 0, "Binder errors: {:?}", bind_errors);
}

#[test]
fn test_bind_deeply_nested_functions() {
    let source = r#"
        fn level1() -> number {
            fn level2() -> number {
                fn level3() -> number {
                    return 42;
                }
                return level3();
            }
            return level2();
        }
    "#;

    let (parse_errors, bind_errors) = parse_and_bind(source);

    assert_eq!(parse_errors.len(), 0, "Parser errors: {:?}", parse_errors);
    assert_eq!(bind_errors.len(), 0, "Binder errors: {:?}", bind_errors);
}

// ============================================================================
// Forward References
// ============================================================================

#[test]
fn test_bind_forward_reference_same_scope() {
    let source = r#"
        fn outer() -> number {
            fn first() -> number {
                return second();
            }
            fn second() -> number {
                return 42;
            }
            return first();
        }
    "#;

    let (parse_errors, bind_errors) = parse_and_bind(source);

    assert_eq!(parse_errors.len(), 0, "Parser errors: {:?}", parse_errors);
    // Binding should succeed - forward reference in same scope is allowed
    assert_eq!(bind_errors.len(), 0, "Binder errors: {:?}", bind_errors);
}

#[test]
fn test_bind_mutual_recursion_same_scope() {
    let source = r#"
        fn outer() -> number {
            fn isEven(n: number) -> bool {
                if (n == 0) {
                    return true;
                }
                return isOdd(n - 1);
            }
            fn isOdd(n: number) -> bool {
                if (n == 0) {
                    return false;
                }
                return isEven(n - 1);
            }
            return 0;
        }
    "#;

    let (parse_errors, bind_errors) = parse_and_bind(source);

    assert_eq!(parse_errors.len(), 0, "Parser errors: {:?}", parse_errors);
    assert_eq!(bind_errors.len(), 0, "Binder errors: {:?}", bind_errors);
}

// ============================================================================
// Shadowing
// ============================================================================

#[test]
fn test_bind_nested_function_shadows_global() {
    let source = r#"
        fn foo() -> number {
            return 1;
        }
        
        fn outer() -> number {
            fn foo() -> number {
                return 2;
            }
            return foo();
        }
    "#;

    let (parse_errors, bind_errors) = parse_and_bind(source);

    assert_eq!(parse_errors.len(), 0, "Parser errors: {:?}", parse_errors);
    assert_eq!(bind_errors.len(), 0, "Binder errors: {:?}", bind_errors);
}

#[test]
fn test_bind_nested_function_shadows_builtin() {
    let source = r#"
        fn outer() -> number {
            fn print(x: number) -> number {
                return x;
            }
            return print(42);
        }
    "#;

    let (parse_errors, bind_errors) = parse_and_bind(source);

    assert_eq!(parse_errors.len(), 0, "Parser errors: {:?}", parse_errors);
    // Nested functions CAN shadow builtins (unlike top-level functions)
    assert_eq!(bind_errors.len(), 0, "Binder errors: {:?}", bind_errors);
}

#[test]
fn test_bind_nested_function_shadows_outer_nested() {
    let source = r#"
        fn level1() -> number {
            fn helper() -> number {
                return 1;
            }
            fn level2() -> number {
                fn helper() -> number {
                    return 2;
                }
                return helper();
            }
            return level2();
        }
    "#;

    let (parse_errors, bind_errors) = parse_and_bind(source);

    assert_eq!(parse_errors.len(), 0, "Parser errors: {:?}", parse_errors);
    assert_eq!(bind_errors.len(), 0, "Binder errors: {:?}", bind_errors);
}

// ============================================================================
// Error Cases
// ============================================================================

#[test]
fn test_bind_redeclaration_same_scope() {
    let source = r#"
        fn outer() -> number {
            fn helper() -> number {
                return 1;
            }
            fn helper() -> number {
                return 2;
            }
            return 0;
        }
    "#;

    let (parse_errors, bind_errors) = parse_and_bind(source);

    assert_eq!(parse_errors.len(), 0, "Parser errors: {:?}", parse_errors);
    // Should have redeclaration error
    assert!(!bind_errors.is_empty(), "Expected redeclaration error");
    assert!(
        bind_errors.iter().any(|e| e.contains("AT2003")),
        "Expected AT2003 error, got: {:?}",
        bind_errors
    );
}

#[test]
fn test_bind_nested_function_in_if_block() {
    let source = r#"
        fn outer() -> number {
            if (true) {
                fn helper() -> number {
                    return 42;
                }
                return helper();
            }
            return 0;
        }
    "#;

    let (parse_errors, bind_errors) = parse_and_bind(source);

    assert_eq!(parse_errors.len(), 0, "Parser errors: {:?}", parse_errors);
    assert_eq!(bind_errors.len(), 0, "Binder errors: {:?}", bind_errors);
}

#[test]
fn test_bind_nested_function_in_while_block() {
    let source = r#"
        fn outer() -> number {
            var i: number = 0;
            while (i < 1) {
                fn helper() -> number {
                    return 42;
                }
                i++;
            }
            return 0;
        }
    "#;

    let (parse_errors, bind_errors) = parse_and_bind(source);

    assert_eq!(parse_errors.len(), 0, "Parser errors: {:?}", parse_errors);
    assert_eq!(bind_errors.len(), 0, "Binder errors: {:?}", bind_errors);
}

#[test]
fn test_bind_nested_function_in_for_block() {
    let source = r#"
        fn outer() -> number {
            for (var i: number = 0; i < 5; i++) {
                fn helper(x: number) -> number {
                    return x;
                }
            }
            return 0;
        }
    "#;

    let (parse_errors, bind_errors) = parse_and_bind(source);

    assert_eq!(parse_errors.len(), 0, "Parser errors: {:?}", parse_errors);
    assert_eq!(bind_errors.len(), 0, "Binder errors: {:?}", bind_errors);
}

#[test]
fn test_bind_nested_function_with_type_params() {
    let source = r#"
        fn outer<T>() -> number {
            fn inner<E>(x: E) -> number {
                return 42;
            }
            return inner(5);
        }
    "#;

    let (parse_errors, bind_errors) = parse_and_bind(source);

    assert_eq!(parse_errors.len(), 0, "Parser errors: {:?}", parse_errors);
    assert_eq!(bind_errors.len(), 0, "Binder errors: {:?}", bind_errors);
}

// ============================================================================
// From nested_function_interpreter_tests.rs
// ============================================================================

fn nested_run_interpreter(source: &str) -> Result<Value, String> {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();

    let mut binder = Binder::new();
    let (_symbol_table, _) = binder.bind(&program);

    let mut interpreter = Interpreter::new();
    interpreter
        .eval(&program, &SecurityContext::allow_all())
        .map_err(|e| format!("{:?}", e))
}

// ============================================================================
// Basic Nested Function Calls
// ============================================================================

#[test]
fn test_interp_nested_function_basic() {
    let source = r#"
        fn outer() -> number {
            fn helper(x: number) -> number {
                return x * 2;
            }
            return helper(21);
        }
        outer();
    "#;

    let result = nested_run_interpreter(source).unwrap();
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_interp_nested_function_multiple_params() {
    let source = r#"
        fn outer() -> number {
            fn add(a: number, b: number) -> number {
                return a + b;
            }
            return add(10, 32);
        }
        outer();
    "#;

    let result = nested_run_interpreter(source).unwrap();
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_interp_nested_function_string() {
    let source = r#"
        fn outer() -> string {
            fn greet(name: string) -> string {
                return "Hello, " + name;
            }
            return greet("World");
        }
        outer();
    "#;

    let result = nested_run_interpreter(source).unwrap();
    assert_eq!(result, Value::string("Hello, World"));
}

// ============================================================================
// Parameter Access
// ============================================================================

#[test]
fn test_interp_nested_function_params() {
    let source = r#"
        fn outer(x: number) -> number {
            fn double(y: number) -> number {
                return y * 2;
            }
            return double(x);
        }
        outer(21);
    "#;

    let result = nested_run_interpreter(source).unwrap();
    assert_eq!(result, Value::Number(42.0));
}

// ============================================================================
// Shadowing
// ============================================================================

#[test]
fn test_interp_nested_function_shadows_global() {
    let source = r#"
        fn foo() -> number {
            return 1;
        }

        fn outer() -> number {
            fn foo() -> number {
                return 42;
            }
            return foo();
        }
        outer();
    "#;

    let result = nested_run_interpreter(source).unwrap();
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_interp_nested_function_shadows_outer_nested() {
    let source = r#"
        fn level1() -> number {
            fn helper() -> number {
                return 1;
            }
            fn level2() -> number {
                fn helper() -> number {
                    return 42;
                }
                return helper();
            }
            return level2();
        }
        level1();
    "#;

    let result = nested_run_interpreter(source).unwrap();
    assert_eq!(result, Value::Number(42.0));
}

// ============================================================================
// Multiple Nesting Levels
// ============================================================================

#[test]
fn test_interp_deeply_nested_functions() {
    let source = r#"
        fn level1() -> number {
            fn level2() -> number {
                fn level3() -> number {
                    fn level4() -> number {
                        return 42;
                    }
                    return level4();
                }
                return level3();
            }
            return level2();
        }
        level1();
    "#;

    let result = nested_run_interpreter(source).unwrap();
    assert_eq!(result, Value::Number(42.0));
}

// ============================================================================
// Nested Functions Calling Each Other
// ============================================================================

#[test]
fn test_interp_nested_function_calling_nested() {
    let source = r#"
        fn outer() -> number {
            fn helper1() -> number {
                return 10;
            }
            fn helper2() -> number {
                return helper1() + 32;
            }
            return helper2();
        }
        outer();
    "#;

    let result = nested_run_interpreter(source).unwrap();
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_interp_nested_function_calling_outer() {
    let source = r#"
        fn global() -> number {
            return 40;
        }

        fn outer() -> number {
            fn nested() -> number {
                return global() + 2;
            }
            return nested();
        }
        outer();
    "#;

    let result = nested_run_interpreter(source).unwrap();
    assert_eq!(result, Value::Number(42.0));
}

// ============================================================================
// Void Functions
// ============================================================================

#[test]
fn test_interp_nested_function_void() {
    let source = r#"
        var result: number = 0;

        fn outer() -> void {
            fn setResult() -> void {
                result = 42;
            }
            setResult();
        }

        outer();
        result;
    "#;

    let result = nested_run_interpreter(source).unwrap();
    assert_eq!(result, Value::Number(42.0));
}

// ============================================================================
// Arrays
// ============================================================================

#[test]
fn test_interp_nested_function_array_param() {
    let source = r#"
        fn outer() -> number {
            fn sum(arr: number[]) -> number {
                return arr[0] + arr[1];
            }
            let nums: number[] = [10, 32];
            return sum(nums);
        }
        outer();
    "#;

    let result = nested_run_interpreter(source).unwrap();
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_interp_nested_function_array_return() {
    let source = r#"
        fn outer() -> number[] {
            fn makeArray() -> number[] {
                return [42, 100];
            }
            return makeArray();
        }
        outer()[0];
    "#;

    let result = nested_run_interpreter(source).unwrap();
    assert_eq!(result, Value::Number(42.0));
}

// ============================================================================
// Control Flow
// ============================================================================

#[test]
fn test_interp_nested_function_conditional() {
    let source = r#"
        fn outer() -> number {
            fn abs(x: number) -> number {
                if (x < 0) {
                    return -x;
                } else {
                    return x;
                }
            }
            return abs(-42);
        }
        outer();
    "#;

    let result = nested_run_interpreter(source).unwrap();
    assert_eq!(result, Value::Number(42.0));
}

// ============================================================================
// Nested Functions in Different Block Types
// ============================================================================

#[test]
fn test_interp_nested_function_in_if_block() {
    let source = r#"
        fn outer() -> number {
            if (true) {
                fn helper() -> number {
                    return 42;
                }
                return helper();
            }
            return 0;
        }
        outer();
    "#;

    let result = nested_run_interpreter(source).unwrap();
    assert_eq!(result, Value::Number(42.0));
}

// ============================================================================
// From nested_function_typecheck_tests.rs
// ============================================================================

fn nested_typecheck_source(source: &str) -> Vec<String> {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();

    let mut binder = Binder::new();
    let (mut symbol_table, _) = binder.bind(&program);

    let mut typechecker = TypeChecker::new(&mut symbol_table);
    let diagnostics = typechecker.check(&program);

    diagnostics
        .iter()
        .filter(|d| d.level == atlas_runtime::diagnostic::DiagnosticLevel::Error)
        .map(|d| format!("{}: {}", d.code, d.message))
        .collect()
}

fn nested_typecheck_source_with_warnings(source: &str) -> Vec<String> {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();

    let mut binder = Binder::new();
    let (mut symbol_table, _) = binder.bind(&program);

    let mut typechecker = TypeChecker::new(&mut symbol_table);
    let diagnostics = typechecker.check(&program);

    diagnostics
        .iter()
        .map(|d| format!("{}: {}", d.code, d.message))
        .collect()
}

// ============================================================================
// Basic Type Checking
// ============================================================================

#[test]
fn test_typecheck_nested_function_basic() {
    let source = r#"
        fn outer() -> number {
            fn helper(x: number) -> number {
                return x * 2;
            }
            return helper(21);
        }
    "#;

    let errors = nested_typecheck_source(source);
    assert_eq!(errors.len(), 0, "Type errors: {:?}", errors);
}

#[test]
fn test_typecheck_nested_function_multiple_params() {
    let source = r#"
        fn outer() -> number {
            fn add(a: number, b: number) -> number {
                return a + b;
            }
            return add(10, 20);
        }
    "#;

    let errors = nested_typecheck_source(source);
    assert_eq!(errors.len(), 0, "Type errors: {:?}", errors);
}

#[test]
fn test_typecheck_nested_function_different_types() {
    let source = r#"
        fn outer() -> string {
            fn greet(name: string) -> string {
                return "Hello, " + name;
            }
            return greet("World");
        }
    "#;

    let errors = nested_typecheck_source(source);
    assert_eq!(errors.len(), 0, "Type errors: {:?}", errors);
}

// ============================================================================
// Return Path Analysis
// ============================================================================

#[test]
fn test_typecheck_nested_function_missing_return() {
    let source = r#"
        fn outer() -> number {
            fn helper(x: number) -> number {
                let y: number = x * 2;
            }
            return 0;
        }
    "#;

    let errors = nested_typecheck_source(source);
    assert!(!errors.is_empty(), "Expected missing return error");
    assert!(
        errors.iter().any(|e| e.contains("AT3004")),
        "Expected AT3004 error, got: {:?}",
        errors
    );
}

#[test]
fn test_typecheck_nested_function_conditional_return() {
    let source = r#"
        fn outer() -> number {
            fn abs(x: number) -> number {
                if (x < 0) {
                    return -x;
                } else {
                    return x;
                }
            }
            return abs(-5);
        }
    "#;

    let errors = nested_typecheck_source(source);
    assert_eq!(errors.len(), 0, "Type errors: {:?}", errors);
}

#[test]
fn test_typecheck_nested_function_incomplete_return_paths() {
    let source = r#"
        fn outer() -> number {
            fn test(x: number) -> number {
                if (x > 0) {
                    return x;
                }
            }
            return 0;
        }
    "#;

    let errors = nested_typecheck_source(source);
    assert!(!errors.is_empty(), "Expected incomplete return paths error");
    assert!(
        errors.iter().any(|e| e.contains("AT3004")),
        "Expected AT3004 error, got: {:?}",
        errors
    );
}

// ============================================================================
// Type Errors
// ============================================================================

#[test]
fn test_typecheck_nested_function_wrong_param_type() {
    let source = r#"
        fn outer() -> number {
            fn double(x: number) -> number {
                return x * 2;
            }
            return double("not a number");
        }
    "#;

    let errors = nested_typecheck_source(source);
    assert!(!errors.is_empty(), "Expected type mismatch error");
    assert!(
        errors
            .iter()
            .any(|e| e.contains("AT3001") || e.contains("type")),
        "Expected type error, got: {:?}",
        errors
    );
}

#[test]
fn test_typecheck_nested_function_wrong_return_type() {
    let source = r#"
        fn outer() -> number {
            fn bad() -> number {
                return "wrong type";
            }
            return bad();
        }
    "#;

    let errors = nested_typecheck_source(source);
    assert!(!errors.is_empty(), "Expected type mismatch error");
    assert!(
        errors
            .iter()
            .any(|e| e.contains("AT3001") || e.contains("type")),
        "Expected type error, got: {:?}",
        errors
    );
}

#[test]
fn test_typecheck_nested_function_param_type_mismatch() {
    let source = r#"
        fn outer() -> void {
            fn process(x: number, y: string) -> void {
                print(str(x) + y);
            }
            process(42, 100);
        }
    "#;

    let errors = nested_typecheck_source(source);
    assert!(!errors.is_empty(), "Expected type mismatch error");
}

// ============================================================================
// Multiple Nesting Levels
// ============================================================================

#[test]
fn test_typecheck_deeply_nested_functions() {
    let source = r#"
        fn level1() -> number {
            fn level2() -> number {
                fn level3() -> number {
                    return 42;
                }
                return level3() + 1;
            }
            return level2() + 1;
        }
    "#;

    let errors = nested_typecheck_source(source);
    assert_eq!(errors.len(), 0, "Type errors: {:?}", errors);
}

#[test]
fn test_typecheck_nested_function_type_error_in_deep_nesting() {
    let source = r#"
        fn level1() -> number {
            fn level2() -> number {
                fn level3() -> number {
                    return "wrong";
                }
                return level3();
            }
            return level2();
        }
    "#;

    let errors = nested_typecheck_source(source);
    assert!(!errors.is_empty(), "Expected type error");
}

// ============================================================================
// Function Calls
// ============================================================================

#[test]
fn test_typecheck_nested_function_calling_nested() {
    let source = r#"
        fn outer() -> number {
            fn helper1() -> number {
                return 10;
            }
            fn helper2() -> number {
                return helper1() + 20;
            }
            return helper2();
        }
    "#;

    let errors = nested_typecheck_source(source);
    assert_eq!(errors.len(), 0, "Type errors: {:?}", errors);
}

#[test]
fn test_typecheck_nested_function_calling_outer() {
    let source = r#"
        fn global() -> number {
            return 100;
        }
        
        fn outer() -> number {
            fn nested() -> number {
                return global() + 1;
            }
            return nested();
        }
    "#;

    let errors = nested_typecheck_source(source);
    assert_eq!(errors.len(), 0, "Type errors: {:?}", errors);
}

// ============================================================================
// Type Parameters
// ============================================================================

#[test]
fn test_typecheck_nested_function_with_type_params() {
    let source = r#"
        fn outer<T>() -> number {
            fn inner<E>(x: E) -> number {
                return 42;
            }
            return inner(100);
        }
    "#;

    let errors = nested_typecheck_source(source);
    assert_eq!(errors.len(), 0, "Type errors: {:?}", errors);
}

// ============================================================================
// Void Return Type
// ============================================================================

#[test]
fn test_typecheck_nested_function_void_return() {
    let source = r#"
        fn outer() -> void {
            fn helper() -> void {
                print("test");
            }
            helper();
        }
    "#;

    let errors = nested_typecheck_source(source);
    assert_eq!(errors.len(), 0, "Type errors: {:?}", errors);
}

#[test]
fn test_typecheck_nested_function_void_no_return_required() {
    let source = r#"
        fn outer() -> void {
            fn helper() -> void {
                let x: number = 42;
            }
            helper();
        }
    "#;

    let errors = nested_typecheck_source(source);
    // Void functions don't require explicit return
    assert_eq!(errors.len(), 0, "Type errors: {:?}", errors);
}

// ============================================================================
// Array and Complex Types
// ============================================================================

#[test]
fn test_typecheck_nested_function_array_param() {
    let source = r#"
        fn outer() -> number {
            fn sum(arr: number[]) -> number {
                return arr[0] + arr[1];
            }
            let nums: number[] = [10, 20];
            return sum(nums);
        }
    "#;

    let errors = nested_typecheck_source(source);
    assert_eq!(errors.len(), 0, "Type errors: {:?}", errors);
}

#[test]
fn test_typecheck_nested_function_array_return() {
    let source = r#"
        fn outer() -> number[] {
            fn makeArray() -> number[] {
                return [1, 2, 3];
            }
            return makeArray();
        }
    "#;

    let errors = nested_typecheck_source(source);
    assert_eq!(errors.len(), 0, "Type errors: {:?}", errors);
}

// ============================================================================
// Unused Parameter Warnings (Should Work)
// ============================================================================

#[test]
fn test_typecheck_nested_function_unused_param_warning() {
    let source = r#"
        fn outer() -> number {
            fn helper(x: number, y: number) -> number {
                return x;
            }
            return helper(10, 20);
        }
    "#;

    let diagnostics = nested_typecheck_source_with_warnings(source);
    // Should have warning for unused parameter 'y'
    assert!(
        diagnostics.iter().any(|d| d.contains("AT2001")),
        "Expected unused parameter warning, got: {:?}",
        diagnostics
    );
}

#[test]
fn test_typecheck_nested_function_underscore_suppresses_warning() {
    let source = r#"
        fn outer() -> number {
            fn helper(_x: number) -> number {
                return 42;
            }
            return helper(10);
        }
    "#;

    let diagnostics = nested_typecheck_source_with_warnings(source);
    // Should NOT have warning for _x (underscore prefix suppresses)
    let unused_warnings = diagnostics.iter().filter(|d| d.contains("AT2001")).count();
    assert_eq!(
        unused_warnings, 0,
        "Should not warn about _x: {:?}",
        diagnostics
    );
}

// ============================================================================
// From scope_shadowing_tests.rs
// ============================================================================

fn bind_source(source: &str) -> Vec<Diagnostic> {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, lex_diags) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, parse_diags) = parser.parse();

    let mut binder = Binder::new();
    let (_table, bind_diags) = binder.bind(&program);

    let mut all_diags = Vec::new();
    all_diags.extend(lex_diags);
    all_diags.extend(parse_diags);
    all_diags.extend(bind_diags);
    all_diags
}

fn typecheck_source(source: &str) -> Vec<Diagnostic> {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, lex_diags) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, parse_diags) = parser.parse();

    let mut binder = Binder::new();
    let (mut table, bind_diags) = binder.bind(&program);

    let mut checker = TypeChecker::new(&mut table);
    let type_diags = checker.check(&program);

    let mut all_diags = Vec::new();
    all_diags.extend(lex_diags);
    all_diags.extend(parse_diags);
    all_diags.extend(bind_diags);
    all_diags.extend(type_diags);
    all_diags
}

fn assert_no_errors(diagnostics: &[Diagnostic]) {
    let errors: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    assert!(
        errors.is_empty(),
        "Expected no errors, got: {:?}",
        errors.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

fn assert_has_error(diagnostics: &[Diagnostic], code: &str) {
    let found = diagnostics.iter().any(|d| d.code == code);
    assert!(
        found,
        "Expected diagnostic with code {}, got: {:?}",
        code,
        diagnostics.iter().map(|d| &d.code).collect::<Vec<_>>()
    );
}

// ============================================================================
// Block Scoping - Valid Cases
// ============================================================================

#[rstest]
#[case::nested_block(r#"let x: number = 1; { let y: number = 2; let z = x + y; }"#)]
#[case::multiple_levels(
    r#"let a: number = 1; { let b: number = 2; { let c: number = 3; let sum = a + b + c; } }"#
)]
#[case::if_block(r#"let x: number = 1; if (x > 0) { let y: number = 2; }"#)]
#[case::while_block(r#"let i: number = 0; while (i < 10) { let temp: number = i; }"#)]
#[case::for_loop_init(r#"for (let i: number = 0; i < 10; i = i + 1) { let x = i; }"#)]
#[case::for_loop_body(r#"for (let i: number = 0; i < 10; i = i + 1) { let sum: number = 0; }"#)]
#[case::empty_block(r#"let x: number = 1; { } let y = x;"#)]
#[case::nested_empty(r#"{ { { } } }"#)]
fn test_valid_block_scoping(#[case] source: &str) {
    let diagnostics = bind_source(source);
    assert_no_errors(&diagnostics);
}

// ============================================================================
// Block Scoping - Out of Scope Errors
// ============================================================================

#[rstest]
#[case::block_var_out_of_scope(r#"{ let x: number = 1; } let y = x;"#, "AT2002")]
#[case::if_block_out_of_scope(
    r#"let x: number = 1; if (x > 0) { let y: number = 2; } let z = y;"#,
    "AT2002"
)]
#[case::while_block_out_of_scope(
    r#"let i: number = 0; while (i < 10) { let temp: number = i; } let x = temp;"#,
    "AT2002"
)]
#[case::for_init_out_of_scope(
    r#"for (let i: number = 0; i < 10; i = i + 1) { let x = i; } let y = i;"#,
    "AT2002"
)]
#[case::for_body_out_of_scope(
    r#"for (let i: number = 0; i < 10; i = i + 1) { let sum: number = 0; } let x = sum;"#,
    "AT2002"
)]
fn test_out_of_scope_errors(#[case] source: &str, #[case] expected_code: &str) {
    let diagnostics = bind_source(source);
    assert_has_error(&diagnostics, expected_code);
}

// ============================================================================
// Variable Shadowing - Allowed in Nested Scopes
// ============================================================================

#[rstest]
#[case::basic_shadowing(r#"let x: number = 1; { let x: string = "hello"; }"#)]
#[case::multiple_levels(
    r#"let x: number = 1; { let x: string = "level 1"; { let x: bool = true; } }"#
)]
#[case::param_shadowing(
    r#"fn foo(x: number) -> number { { let x: string = "shadow"; } return x; }"#
)]
#[case::if_block_shadow(r#"let x: number = 1; if (true) { let x: string = "shadow"; }"#)]
#[case::else_block_shadow(
    r#"let x: number = 1; if (false) { let y: number = 2; } else { let x: string = "shadow"; }"#
)]
#[case::while_shadow(r#"let i: number = 0; while (i < 10) { let i: string = "shadow"; }"#)]
#[case::for_shadow(
    r#"let i: number = 999; for (let i: number = 0; i < 10; i = i + 1) { let x = i; }"#
)]
#[case::shadow_restored(r#"let x: number = 1; { let x: string = "shadow"; } let y = x;"#)]
#[case::nested_fn_shadow(r#"fn outer(x: number) -> number { { let x: string = "shadow"; { let x: bool = true; } } return x; }"#)]
#[case::if_else_separate(
    r#"let x: number = 1; if (true) { let y: number = 2; } else { let y: string = "different"; }"#
)]
#[case::multiple_blocks(r#"{ let x: number = 1; } { let x: string = "different block"; }"#)]
#[case::loop_nested_blocks(r#"let i: number = 0; while (i < 10) { { let temp: number = i; } { let temp: string = "different"; } }"#)]
#[case::deeply_nested(r#"let a: number = 1; { let b: number = 2; { let c: number = 3; { let d: number = 4; { let e: number = 5; let sum = a + b + c + d + e; } } } }"#)]
#[case::different_types(r#"let x: number = 1; { let x: string = "string"; { let x: bool = true; { let x = [1, 2, 3]; } } }"#)]
fn test_valid_shadowing(#[case] source: &str) {
    let diagnostics = bind_source(source);
    assert_no_errors(&diagnostics);
}

// ============================================================================
// Redeclaration Errors - Same Scope
// ============================================================================

#[rstest]
#[case::same_scope(r#"let x: number = 1; let x: string = "redeclare";"#, "AT2003")]
#[case::in_block(
    r#"fn test() -> void { let x: number = 1; let x: string = "redeclare"; }"#,
    "AT2003"
)]
#[case::param_redecl(r#"fn foo(x: number, x: string) -> number { return 0; }"#, "AT2003")]
#[case::function_redecl(
    r#"fn foo() -> number { return 1; } fn foo() -> string { return "redeclare"; }"#,
    "AT2003"
)]
fn test_redeclaration_errors(#[case] source: &str, #[case] expected_code: &str) {
    let diagnostics = bind_source(source);
    assert_has_error(&diagnostics, expected_code);
}

#[test]
fn test_multiple_variable_redeclarations() {
    let diagnostics =
        bind_source(r#"let x: number = 1; let x: string = "second"; let x: bool = true;"#);
    // Should have multiple redeclaration errors
    let redecl_errors: Vec<_> = diagnostics.iter().filter(|d| d.code == "AT2003").collect();
    assert!(redecl_errors.len() >= 2);
}

// ============================================================================
// Function Parameter Cases
// ============================================================================

#[rstest]
#[case::param_shadow_allowed(
    r#"fn foo(x: number) -> number { { let x: string = "shadow"; } return x; }"#
)]
#[case::param_can_read(r#"fn double(x: number) -> number { let result = x * 2; return result; }"#)]
#[case::param_in_expr(r#"fn calculate(x: number, y: number) -> number { return x + y * 2; }"#)]
fn test_valid_parameter_usage(#[case] source: &str) {
    let diagnostics = bind_source(source);
    assert_no_errors(&diagnostics);
}

#[rstest]
#[case::immutable_assign(r#"fn foo(x: number) -> number { x = 10; return x; }"#)]
#[case::multiple_params(r#"fn add(a: number, b: number) -> number { a = a + 1; return a + b; }"#)]
fn test_parameter_immutability(#[case] source: &str) {
    // NOTE: Parameter immutability checking requires full type checking
    // This test documents the expected behavior
    let _diagnostics = typecheck_source(source);
    // Once AT3003 is implemented for parameters, add assertion:
    // assert_has_error(&_diagnostics, "AT3003");
}

// ============================================================================
// Function Scope
// ============================================================================

#[rstest]
#[case::access_params(r#"fn foo(x: number, y: string) -> number { let z = x; return z; }"#)]
#[case::call_other_fn(
    r#"fn helper() -> number { return 42; } fn main() -> number { return helper(); }"#
)]
#[case::hoisting(
    r#"fn main() -> number { return helper(); } fn helper() -> number { return 42; }"#
)]
#[case::use_prelude(r#"fn test() -> number { print("hello"); return 42; }"#)]
fn test_valid_function_scope(#[case] source: &str) {
    let diagnostics = bind_source(source);
    assert_no_errors(&diagnostics);
}

#[rstest]
#[case::undefined_var(r#"fn foo() -> number { return undefined_var; }"#, "AT2002")]
#[case::forward_ref(
    r#"let x: number = a + b; let a: number = 1; let b: number = 2;"#,
    "AT2002"
)]
#[case::self_ref(r#"let x: number = x + 1;"#, "AT2002")]
#[case::decl_order(r#"let x = y; let y: number = 1;"#, "AT2002")]
fn test_scope_errors(#[case] source: &str, #[case] expected_code: &str) {
    let diagnostics = bind_source(source);
    assert_has_error(&diagnostics, expected_code);
}

// ============================================================================
// Prelude Shadowing (Documented Behavior)
// ============================================================================

#[rstest]
#[case::shadow_print(r#"fn test() -> void { let print: number = 42; }"#)]
#[case::shadow_len(r#"fn test() -> void { let len: string = "shadowed"; }"#)]
fn test_prelude_shadowing(#[case] source: &str) {
    // NOTE: Prelude shadowing detection (AT1012) may not be fully implemented yet
    // This test documents the expected behavior
    // For now, just verify it binds without crashing
    let _diagnostics = bind_source(source);
    // Once AT1012 is implemented, add assertion:
    // assert_has_error(&_diagnostics, "AT1012");
}

// ============================================================================
// From pattern_matching_tests.rs
// ============================================================================

/// Helper to parse and type check code
fn typecheck(source: &str) -> (bool, Vec<String>) {
    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize();
    if !lex_diags.is_empty() {
        return (false, lex_diags.iter().map(|d| d.message.clone()).collect());
    }

    let mut parser = Parser::new(tokens);
    let (program, parse_diags) = parser.parse();
    if !parse_diags.is_empty() {
        return (
            false,
            parse_diags.iter().map(|d| d.message.clone()).collect(),
        );
    }

    let mut binder = Binder::new();
    let (mut symbol_table, bind_diags) = binder.bind(&program);
    if !bind_diags.is_empty() {
        return (
            false,
            bind_diags.iter().map(|d| d.message.clone()).collect(),
        );
    }

    let mut typechecker = TypeChecker::new(&mut symbol_table);
    let type_diags = typechecker.check(&program);

    let success = type_diags.is_empty();
    let messages = type_diags.iter().map(|d| d.message.clone()).collect();
    (success, messages)
}

// === Parser Tests: Match Expression Syntax ===

#[test]
fn test_parse_simple_match() {
    let source = r#"
        match x {
            1 => "one",
            2 => "two",
            _ => "other"
        }
    "#;
    let (_success, _) = typecheck(source);
    // May fail type checking (x undefined), but should parse
    // If we reach here, it parsed without panic
}

#[test]
fn test_parse_option_match() {
    let source = r#"
        fn test(opt: Option<number>) -> string {
            return match opt {
                Some(x) => "has value",
                None => "no value"
            };
        }
    "#;
    let (success, msgs) = typecheck(source);
    assert!(success, "Should type check: {:?}", msgs);
}

#[test]
fn test_parse_result_match() {
    let source = r#"
        fn test(res: Result<number, string>) -> string {
            return match res {
                Ok(val) => "success",
                Err(error) => "failure"
            };
        }
    "#;
    let (success, msgs) = typecheck(source);
    assert!(success, "Should type check: {:?}", msgs);
}

// === Pattern Type Tests ===

#[test]
fn test_literal_patterns() {
    let source = r#"
        fn test(x: number) -> string {
            return match x {
                0 => "zero",
                1 => "one",
                _ => "other"
            };
        }
    "#;
    let (success, msgs) = typecheck(source);
    assert!(success, "Literal patterns should work: {:?}", msgs);
}

#[test]
fn test_wildcard_pattern() {
    let source = r#"
        fn test(x: number) -> string {
            return match x {
                _ => "anything"
            };
        }
    "#;
    let (success, msgs) = typecheck(source);
    assert!(success, "Wildcard should work: {:?}", msgs);
}

#[test]
fn test_variable_binding_pattern() {
    let source = r#"
        fn test(x: number) -> number {
            return match x {
                value => value + 1
            };
        }
    "#;
    let (success, msgs) = typecheck(source);
    assert!(success, "Variable binding should work: {:?}", msgs);
}

#[test]
fn test_nested_constructor_patterns() {
    let source = r#"
        fn test(res: Result<Option<number>, string>) -> string {
            return match res {
                Ok(Some(x)) => "has value",
                Ok(None) => "no value",
                Err(e) => "error"
            };
        }
    "#;
    let (success, msgs) = typecheck(source);
    assert!(success, "Nested patterns should work: {:?}", msgs);
}

#[test]
fn test_array_patterns() {
    let source = r#"
        fn test(arr: number[]) -> string {
            return match arr {
                [] => "empty",
                [x] => "one",
                [x, y] => "two",
                _ => "many"
            };
        }
    "#;
    let (success, msgs) = typecheck(source);
    assert!(success, "Array patterns should work: {:?}", msgs);
}

// === Type Checking Tests ===

#[test]
fn test_pattern_type_mismatch() {
    let source = r#"
        fn test(x: number) -> string {
            return match x {
                "hello" => "string",
                _ => "other"
            };
        }
    "#;
    let (success, msgs) = typecheck(source);
    assert!(!success, "Should reject type mismatch");
    assert!(
        msgs.iter()
            .any(|m| m.contains("type mismatch") || m.contains("Pattern type")),
        "Should report type mismatch: {:?}",
        msgs
    );
}

#[test]
fn test_arm_type_mismatch() {
    let source = r#"
        fn test(x: number) -> string {
            return match x {
                0 => "zero",
                1 => 123,
                _ => "other"
            };
        }
    "#;
    let (success, msgs) = typecheck(source);
    assert!(!success, "Should reject mismatched arm types");
    assert!(
        msgs.iter().any(|m| m.contains("incompatible type")),
        "Should report arm type mismatch: {:?}",
        msgs
    );
}

#[test]
fn test_constructor_wrong_arity() {
    let source = r#"
        fn test(opt: Option<number>) -> string {
            return match opt {
                Some(x, y) => "wrong",
                None => "ok"
            };
        }
    "#;
    let (success, msgs) = typecheck(source);
    assert!(!success, "Should reject wrong arity");
    assert!(
        msgs.iter().any(|m| m.contains("expects 1 argument")),
        "Should report arity error: {:?}",
        msgs
    );
}

// === Exhaustiveness Tests ===

#[test]
fn test_option_exhaustive() {
    let source = r#"
        fn test(opt: Option<number>) -> string {
            return match opt {
                Some(x) => "has",
                None => "none"
            };
        }
    "#;
    let (success, msgs) = typecheck(source);
    assert!(
        success,
        "Complete Option match should be exhaustive: {:?}",
        msgs
    );
}

#[test]
fn test_option_non_exhaustive_missing_none() {
    let source = r#"
        fn test(opt: Option<number>) -> string {
            return match opt {
                Some(x) => "has"
            };
        }
    "#;
    let (success, msgs) = typecheck(source);
    assert!(!success, "Should reject non-exhaustive Option");
    assert!(
        msgs.iter()
            .any(|m| m.contains("Non-exhaustive") && m.contains("None")),
        "Should report missing None: {:?}",
        msgs
    );
}

#[test]
fn test_option_non_exhaustive_missing_some() {
    let source = r#"
        fn test(opt: Option<number>) -> string {
            return match opt {
                None => "none"
            };
        }
    "#;
    let (success, msgs) = typecheck(source);
    assert!(!success, "Should reject non-exhaustive Option");
    assert!(
        msgs.iter()
            .any(|m| m.contains("Non-exhaustive") && m.contains("Some")),
        "Should report missing Some: {:?}",
        msgs
    );
}

#[test]
fn test_result_exhaustive() {
    let source = r#"
        fn test(res: Result<number, string>) -> string {
            return match res {
                Ok(x) => "ok",
                Err(e) => "err"
            };
        }
    "#;
    let (success, msgs) = typecheck(source);
    assert!(
        success,
        "Complete Result match should be exhaustive: {:?}",
        msgs
    );
}

#[test]
fn test_result_non_exhaustive_missing_err() {
    let source = r#"
        fn test(res: Result<number, string>) -> string {
            return match res {
                Ok(x) => "ok"
            };
        }
    "#;
    let (success, msgs) = typecheck(source);
    assert!(!success, "Should reject non-exhaustive Result");
    assert!(
        msgs.iter()
            .any(|m| m.contains("Non-exhaustive") && m.contains("Err")),
        "Should report missing Err: {:?}",
        msgs
    );
}

#[test]
fn test_result_non_exhaustive_missing_ok() {
    let source = r#"
        fn test(res: Result<number, string>) -> string {
            return match res {
                Err(e) => "err"
            };
        }
    "#;
    let (success, msgs) = typecheck(source);
    assert!(!success, "Should reject non-exhaustive Result");
    assert!(
        msgs.iter()
            .any(|m| m.contains("Non-exhaustive") && m.contains("Ok")),
        "Should report missing Ok: {:?}",
        msgs
    );
}

#[test]
fn test_bool_exhaustive() {
    let source = r#"
        fn test(b: bool) -> string {
            return match b {
                true => "yes",
                false => "no"
            };
        }
    "#;
    let (success, msgs) = typecheck(source);
    assert!(
        success,
        "Complete bool match should be exhaustive: {:?}",
        msgs
    );
}

#[test]
fn test_bool_non_exhaustive() {
    let source = r#"
        fn test(b: bool) -> string {
            return match b {
                true => "yes"
            };
        }
    "#;
    let (success, msgs) = typecheck(source);
    assert!(!success, "Should reject non-exhaustive bool");
    assert!(
        msgs.iter()
            .any(|m| m.contains("Non-exhaustive") && m.contains("false")),
        "Should report missing false: {:?}",
        msgs
    );
}

#[test]
fn test_number_requires_wildcard() {
    let source = r#"
        fn test(x: number) -> string {
            return match x {
                0 => "zero",
                1 => "one"
            };
        }
    "#;
    let (success, msgs) = typecheck(source);
    assert!(!success, "Number match should require wildcard");
    assert!(
        msgs.iter().any(|m| m.contains("Non-exhaustive")),
        "Should report non-exhaustive: {:?}",
        msgs
    );
}

#[test]
fn test_wildcard_makes_exhaustive() {
    let source = r#"
        fn test(x: number) -> string {
            return match x {
                0 => "zero",
                _ => "other"
            };
        }
    "#;
    let (success, msgs) = typecheck(source);
    assert!(
        success,
        "Wildcard should make number match exhaustive: {:?}",
        msgs
    );
}

#[test]
fn test_variable_binding_makes_exhaustive() {
    let source = r#"
        fn test(opt: Option<number>) -> string {
            return match opt {
                value => "anything"
            };
        }
    "#;
    let (success, msgs) = typecheck(source);
    assert!(success, "Variable binding should be exhaustive: {:?}", msgs);
}

// === Variable Binding Tests ===

#[test]
fn test_pattern_variable_scope() {
    let source = r#"
        fn test(opt: Option<number>) -> number {
            return match opt {
                Some(x) => x + 1,
                None => 0
            };
        }
    "#;
    let (success, msgs) = typecheck(source);
    assert!(success, "Pattern variables should be in scope: {:?}", msgs);
}

#[test]
fn test_nested_pattern_variables() {
    let source = r#"
        fn test(res: Result<Option<number>, string>) -> number {
            return match res {
                Ok(Some(value)) => value,
                Ok(None) => 0,
                Err(msg) => 0
            };
        }
    "#;
    let (success, msgs) = typecheck(source);
    assert!(success, "Nested pattern variables should work: {:?}", msgs);
}

// === Edge Cases ===

#[test]
fn test_match_as_expression() {
    let source = r#"
        fn test(x: bool) -> string {
            let result = match x {
                true => "yes",
                false => "no"
            };
            return result;
        }
    "#;
    let (success, msgs) = typecheck(source);
    assert!(success, "Match should work as expression: {:?}", msgs);
}

#[test]
fn test_match_in_expression_context() {
    let source = r#"
        fn test(x: bool) -> string {
            return "Answer: " + match x {
                true => "yes",
                false => "no"
            };
        }
    "#;
    let (success, msgs) = typecheck(source);
    assert!(success, "Match in expression should work: {:?}", msgs);
}

#[test]
fn test_multiple_matches() {
    let source = r#"
        fn test(a: Option<number>, b: Result<string, number>) -> string {
            let x = match a {
                Some(n) => "has",
                None => "none"
            };
            let y = match b {
                Ok(s) => "ok",
                Err(e) => "err"
            };
            return x + y;
        }
    "#;
    let (success, msgs) = typecheck(source);
    assert!(success, "Multiple matches should work: {:?}", msgs);
}

// ============================================================================
// From assignment_target_tests.rs
// ============================================================================

fn parse_source(
    source: &str,
) -> (
    atlas_runtime::ast::Program,
    Vec<atlas_runtime::diagnostic::Diagnostic>,
) {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    parser.parse()
}

// ============================================================================
// Assignment Target Validation with Snapshots
// ============================================================================

#[rstest]
#[case("simple_name", "x = 42;")]
#[case("longer_name", "myVariable = 100;")]
#[case("expression_value", "x = 1 + 2 * 3;")]
#[case("array_index", "arr[0] = 42;")]
#[case("array_expression_index", "arr[i + 1] = 99;")]
#[case("nested_array", "matrix[i][j] = 5;")]
#[case("string_value", r#"name = "Alice";"#)]
#[case("boolean_value", "flag = true;")]
#[case("null_value", "value = null;")]
#[case("function_call_value", "result = foo();")]
#[case("array_literal_value", "items = [1, 2, 3];")]
fn test_assignment_targets(#[case] name: &str, #[case] source: &str) {
    let (program, diagnostics) = parse_source(source);

    assert_eq!(diagnostics.len(), 0, "Should parse without errors");
    assert_eq!(program.items.len(), 1, "Should have one statement");

    // Snapshot the AST to verify structure
    insta::assert_yaml_snapshot!(format!("assignment_{}", name), program.items[0]);
}

// ============================================================================
// Compound Assignment Operators
// ============================================================================

#[rstest]
#[case("add_assign", "x += 5;")]
#[case("sub_assign", "x -= 3;")]
#[case("mul_assign", "x *= 2;")]
#[case("div_assign", "x /= 4;")]
#[case("mod_assign", "x %= 10;")]
#[case("array_add_assign", "arr[i] += 1;")]
fn test_compound_assignments(#[case] name: &str, #[case] source: &str) {
    let (program, diagnostics) = parse_source(source);

    assert_eq!(diagnostics.len(), 0);
    insta::assert_yaml_snapshot!(format!("compound_{}", name), program.items[0]);
}

// ============================================================================
// Increment/Decrement Operators
// ============================================================================

#[rstest]
#[case("increment_name", "x++;")]
#[case("decrement_name", "x--;")]
#[case("increment_array", "arr[0]++;")]
#[case("decrement_array", "arr[i]--;")]
fn test_increment_decrement(#[case] name: &str, #[case] source: &str) {
    let (program, diagnostics) = parse_source(source);

    assert_eq!(diagnostics.len(), 0);
    insta::assert_yaml_snapshot!(format!("incdec_{}", name), program.items[0]);
}

// ============================================================================
// Multiple Assignments in Sequence
// ============================================================================

#[test]
fn test_multiple_assignments() {
    let source = "x = 1; y = 2; z = 3;";
    let (program, diagnostics) = parse_source(source);

    assert_eq!(diagnostics.len(), 0);
    assert_eq!(program.items.len(), 3);

    insta::assert_yaml_snapshot!("multiple_assignments", program);
}

// ============================================================================
// Complex Assignment Expressions
// ============================================================================

#[test]
fn test_chained_array_access_assignment() {
    let source = "matrix[row][col] = value;";
    let (program, diagnostics) = parse_source(source);

    assert_eq!(diagnostics.len(), 0);
    insta::assert_yaml_snapshot!("chained_array_assignment", program.items[0]);
}

// ============================================================================
// From test_for_in_edge_cases.rs
// ============================================================================

#[test]
fn test_for_in_large_array() {
    // Simplified: Use a smaller array to test iteration stability
    let source = r#"
        let arr: array = [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9,
            10, 11, 12, 13, 14, 15, 16, 17, 18, 19,
            20, 21, 22, 23, 24, 25, 26, 27, 28, 29
        ];

        var sum: number = 0;
        for item in arr {
            sum = sum + item;
        }

        sum
    "#;

    let runtime = Atlas::new();
    let result = runtime.eval(source);

    // Sum of 0..29 = 29 * 30 / 2 = 435
    assert_eq!(result.unwrap(), Value::Number(435.0));
}

#[test]
fn test_for_in_deeply_nested() {
    let source = r#"
        let arr3d: array = [
            [[1, 2], [3, 4]],
            [[5, 6], [7, 8]]
        ];

        var sum: number = 0;
        for layer in arr3d {
            for row in layer {
                for item in row {
                    sum = sum + item;
                }
            }
        }

        sum
    "#;

    let runtime = Atlas::new();
    let result = runtime.eval(source);

    assert_eq!(result.unwrap(), Value::Number(36.0), "Sum 1+2+..+8=36");
}

#[test]
fn test_for_in_with_array_iteration_count() {
    // Test that iteration count is correct
    let source = r#"
        let arr: array = [1, 2, 3, 4, 5];
        var count: number = 0;

        for item in arr {
            count = count + 1;
        }

        count
    "#;

    let runtime = Atlas::new();
    let result = runtime.eval(source);

    assert_eq!(
        result.unwrap(),
        Value::Number(5.0),
        "Should iterate 5 times"
    );
}

#[test]
fn test_for_in_with_early_return() {
    let source = r#"
        fn find_first_even(arr: array) -> number {
            for item in arr {
                if (item % 2 == 0) {
                    return item;
                }
            }
            return -1;
        }

        find_first_even([1, 3, 5, 8, 10])
    "#;

    let runtime = Atlas::new();
    let result = runtime.eval(source);

    assert_eq!(
        result.unwrap(),
        Value::Number(8.0),
        "Should return first even number"
    );
}

#[test]
fn test_for_in_with_complex_expressions() {
    let source = r#"
        let arr: array = [1, 2, 3, 4, 5];
        var sum_even: number = 0;
        var sum_odd: number = 0;

        for item in arr {
            if (item % 2 == 0) {
                sum_even = sum_even + item;
            } else {
                sum_odd = sum_odd + item;
            }
        }

        sum_even + sum_odd
    "#;

    let runtime = Atlas::new();
    let result = runtime.eval(source);

    assert_eq!(
        result.unwrap(),
        Value::Number(15.0),
        "Sum of all items = 15"
    );
}

#[test]
fn test_for_in_break_in_nested_loop() {
    let source = r#"
        let matrix: array = [[1, 2, 3], [4, 5, 6], [7, 8, 9]];
        var found: bool = false;

        for row in matrix {
            for item in row {
                if (item == 5) {
                    found = true;
                    break;
                }
            }
            if (found) {
                break;
            }
        }

        found
    "#;

    let runtime = Atlas::new();
    let result = runtime.eval(source);

    assert_eq!(result.unwrap(), Value::Bool(true));
}

#[test]
fn test_for_in_multiple_sequential() {
    let source = r#"
        let arr1: array = [1, 2, 3];
        let arr2: array = [4, 5, 6];
        var sum: number = 0;

        for item in arr1 {
            sum = sum + item;
        }

        for item in arr2 {
            sum = sum + item;
        }

        sum
    "#;

    let runtime = Atlas::new();
    let result = runtime.eval(source);

    assert_eq!(result.unwrap(), Value::Number(21.0), "Sum of 1..6 = 21");
}

#[test]
fn test_for_in_with_function_calls() {
    let source = r#"
        fn double(x: number) -> number {
            return x * 2;
        }

        let arr: array = [1, 2, 3];
        var sum: number = 0;

        for item in arr {
            sum = sum + double(item);
        }

        sum
    "#;

    let runtime = Atlas::new();
    let result = runtime.eval(source);

    assert_eq!(
        result.unwrap(),
        Value::Number(12.0),
        "Sum of doubled items = 12"
    );
}

#[test]
fn test_for_in_with_hashmap_keys() {
    let source = r#"
        let hmap: HashMap = hashMapNew();
        hashMapPut(hmap, "a", 1);
        hashMapPut(hmap, "b", 2);
        hashMapPut(hmap, "c", 3);

        let keys: array = hashMapKeys(hmap);
        var count: number = 0;

        for key in keys {
            count = count + 1;
        }

        count
    "#;

    let runtime = Atlas::new();
    let result = runtime.eval(source);

    assert_eq!(result.unwrap(), Value::Number(3.0));
}

#[test]
fn test_for_in_with_hashset() {
    let source = r#"
        let set: HashSet = hashSetNew();
        hashSetAdd(set, 10);
        hashSetAdd(set, 20);
        hashSetAdd(set, 30);

        let arr: array = hashSetToArray(set);
        var sum: number = 0;

        for item in arr {
            sum = sum + item;
        }

        sum
    "#;

    let runtime = Atlas::new();
    let result = runtime.eval(source);

    assert_eq!(result.unwrap(), Value::Number(60.0));
}

#[test]
fn test_for_in_with_result_early_return() {
    let source = r#"
        fn process(arr: array) -> number {
            var sum: number = 0;
            for item in arr {
                if (item < 0) {
                    return -999;
                }
                sum = sum + item;
            }
            return sum;
        }

        process([1, 2, 3])
    "#;

    let runtime = Atlas::new();
    let result = runtime.eval(source);

    assert_eq!(
        result.unwrap(),
        Value::Number(6.0),
        "Should return sum of positive numbers"
    );
}

#[test]
fn test_for_in_with_conditional_sum() {
    let source = r#"
        let arr: array = [1, -1, 2, -2, 3, -3];
        var pos_sum: number = 0;
        var neg_sum: number = 0;

        for num in arr {
            if (num > 0) {
                pos_sum = pos_sum + num;
            } else {
                neg_sum = neg_sum + num;
            }
        }

        pos_sum
    "#;

    let runtime = Atlas::new();
    let result = runtime.eval(source);

    assert_eq!(
        result.unwrap(),
        Value::Number(6.0),
        "Sum of positive values"
    );
}

#[test]
fn test_for_in_performance() {
    // Build a large array literal for performance testing
    let mut array_elements = Vec::new();
    for i in 0..1000 {
        array_elements.push(i.to_string());
    }
    let array_literal = format!("[{}]", array_elements.join(", "));

    let source = format!(
        r#"
        let arr: array = {};

        var sum: number = 0;
        for item in arr {{
            sum = sum + item;
        }}

        sum
    "#,
        array_literal
    );

    let start = std::time::Instant::now();
    let runtime = Atlas::new();
    let result = runtime.eval(&source);
    let duration = start.elapsed();

    assert!(result.is_ok());
    // Sum of 0..999 = 999 * 1000 / 2 = 499500
    assert_eq!(result.unwrap(), Value::Number(499500.0));
    assert!(
        duration.as_millis() < 2000,
        "Should complete in < 2s, took {}ms",
        duration.as_millis()
    );
}

// ============================================================================
// From test_for_in_semantic.rs
// ============================================================================

/// Helper to run full semantic analysis pipeline
fn analyze(source: &str) -> (bool, Vec<String>) {
    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize();
    if !lex_diags.is_empty() {
        return (false, lex_diags.iter().map(|d| d.message.clone()).collect());
    }

    let mut parser = Parser::new(tokens);
    let (program, parse_diags) = parser.parse();
    if !parse_diags.is_empty() {
        return (
            false,
            parse_diags.iter().map(|d| d.message.clone()).collect(),
        );
    }

    let mut binder = Binder::new();
    let (mut symbol_table, bind_diags) = binder.bind(&program);
    if !bind_diags.is_empty() {
        return (
            false,
            bind_diags.iter().map(|d| d.message.clone()).collect(),
        );
    }

    let mut typechecker = TypeChecker::new(&mut symbol_table);
    let type_diags = typechecker.check(&program);

    let success = type_diags.is_empty();
    let messages = type_diags.iter().map(|d| d.message.clone()).collect();
    (success, messages)
}

#[test]
fn test_for_in_binds_variable() {
    let source = r#"
        fn test() -> void {
            let arr = [1, 2, 3];
            for item in arr {
                print(item);
            }
        }
    "#;

    let (success, errors) = analyze(source);
    assert!(success, "Binder should handle for-in: {:?}", errors);
}

#[test]
fn test_for_in_type_checks_array() {
    let source = r#"
        fn test() -> void {
            let arr = [1, 2, 3];
            for item in arr {
                print(item);
            }
        }
    "#;

    let (success, errors) = analyze(source);
    assert!(
        success,
        "TypeChecker should accept array for-in: {:?}",
        errors
    );
}

#[test]
fn test_for_in_with_array_literal_type_check() {
    // Note: Using array literal directly works better than variables due to type inference limitations
    let source = r#"
        fn test() -> void {
            for item in [1, 2, 3] {
                print(item);
            }
        }
    "#;

    let (success, errors) = analyze(source);
    assert!(success, "Should accept array literal: {:?}", errors);
}

#[test]
fn test_for_in_variable_scoped() {
    let source = r#"
        fn test() -> void {
            let arr = [1, 2, 3];
            for item in arr {
                print(item);
            }
            print(item);
        }
    "#;

    let (success, errors) = analyze(source);
    assert!(!success, "Variable should not be accessible outside loop");
    assert!(
        errors
            .iter()
            .any(|e| e.contains("item") || e.contains("Undefined")),
        "Error should mention undefined variable: {:?}",
        errors
    );
}

#[test]
fn test_for_in_nested() {
    let source = r#"
        fn test() -> void {
            let matrix = [[1, 2], [3, 4]];
            for row in matrix {
                for item in row {
                    print(item);
                }
            }
        }
    "#;

    let (success, errors) = analyze(source);
    assert!(success, "Should handle nested for-in: {:?}", errors);
}

#[test]
fn test_for_in_with_break() {
    let source = r#"
        fn test() -> void {
            let arr = [1, 2, 3];
            for item in arr {
                if (item > 2) {
                    break;
                }
            }
        }
    "#;

    let (success, errors) = analyze(source);
    assert!(success, "Should allow break in for-in: {:?}", errors);
}

#[test]
fn test_for_in_with_continue() {
    let source = r#"
        fn test() -> void {
            let arr = [1, 2, 3];
            for item in arr {
                if (item == 2) {
                    continue;
                }
                print(item);
            }
        }
    "#;

    let (success, errors) = analyze(source);
    assert!(success, "Should allow continue in for-in: {:?}", errors);
}

#[test]
fn test_for_in_with_function_call() {
    let source = r#"
        fn getArray() -> array {
            return [1, 2, 3];
        }

        fn test() -> void {
            for item in getArray() {
                print(item);
            }
        }
    "#;

    let (success, errors) = analyze(source);
    assert!(
        success,
        "Should work with function call iterable: {:?}",
        errors
    );
}

#[test]
fn test_for_in_empty_array() {
    let source = r#"
        fn test() -> void {
            let arr = [];
            for item in arr {
                print(item);
            }
        }
    "#;

    let (success, errors) = analyze(source);
    assert!(success, "Should handle empty array: {:?}", errors);
}

#[test]
fn test_for_in_variable_shadowing() {
    let source = r#"
        fn test() -> void {
            let item = "outer";
            let arr = [1, 2, 3];
            for item in arr {
                print(item);
            }
            print(item);
        }
    "#;

    let (success, errors) = analyze(source);
    // This should succeed - the loop variable shadows the outer one
    // After the loop, 'item' refers to the outer variable again
    assert!(success, "Should allow variable shadowing: {:?}", errors);
}

// ============================================================================
// From integration/interpreter/arithmetic.rs
// ============================================================================

#[rstest]
#[case("1 + 2", 3.0)]
#[case("10 - 3", 7.0)]
#[case("4 * 5", 20.0)]
#[case("20 / 4", 5.0)]
#[case("10 % 3", 1.0)]
#[case("-42", -42.0)]
#[case("2 + 3 * 4 - 1", 13.0)]
#[case("(2 + 3) * 4", 20.0)]
fn test_arithmetic_operations(#[case] code: &str, #[case] expected: f64) {
    assert_eval_number(code, expected);
}

#[rstest]
#[case("10 / 0", "AT0005")]
#[case("10 % 0", "AT0005")]
#[case("0 / 0", "AT0005")]
#[case("-10 / 0", "AT0005")]
#[case("0 % 0", "AT0005")]
#[case("5 + (10 / 0)", "AT0005")]
fn test_divide_by_zero_errors(#[case] code: &str, #[case] error_code: &str) {
    assert_error_code(code, error_code);
}

#[rstest]
#[case("1e308 * 2.0", "AT0007")]
#[case("1.5e308 + 1.5e308", "AT0007")]
#[case("-1.5e308 - 1.5e308", "AT0007")]
#[case("1e308 / 1e-308", "AT0007")]
fn test_numeric_overflow(#[case] code: &str, #[case] error_code: &str) {
    assert_error_code(code, error_code);
}

#[test]
fn test_numeric_valid_large_numbers() {
    let runtime = Atlas::new();
    let code = r#"
        let x: number = 1e50;
        let y: number = 2e50;
        let z: number = x + y;
        z
    "#;

    match runtime.eval(code) {
        Ok(Value::Number(n)) => {
            assert!(n > 0.0);
            assert!(n.is_finite());
        }
        other => panic!("Expected valid large number, got {:?}", other),
    }
}

#[test]
fn test_numeric_multiplication_by_zero_valid() {
    assert_eval_number("let large: number = 1e200; large * 0", 0.0);
}

#[test]
fn test_numeric_negative_modulo() {
    let runtime = Atlas::new();
    match runtime.eval("-10 % 3") {
        Ok(Value::Number(n)) => {
            assert!(n.is_finite());
            std::assert_eq!(n, -1.0); // Rust's % preserves sign of left operand
        }
        other => panic!("Expected valid modulo result, got {:?}", other),
    }
}

#[test]
fn test_numeric_error_in_function() {
    let code = r#"
        fn compute(a: number) -> number {
            return a * a * a;
        }
        let big: number = 1e103;
        compute(big)
    "#;
    assert_error_code(code, "AT0007");
}

#[test]
fn test_numeric_error_propagation() {
    let code = r#"
        fn bad() -> number {
            return 1 / 0;
        }
        fn caller() -> number {
            return bad() + 5;
        }
        caller()
    "#;
    assert_error_code(code, "AT0005");
}

// ============================================================================
// From integration/interpreter/arrays.rs
// ============================================================================

#[test]
fn test_array_literal() {
    let code = r#"
        let arr: number[] = [1, 2, 3];
        arr[1]
    "#;
    assert_eval_number(code, 2.0);
}

#[test]
fn test_array_assignment() {
    let code = r#"
        let arr: number[] = [1, 2, 3];
        arr[1] = 99;
        arr[1]
    "#;
    assert_eval_number(code, 99.0);
}

#[test]
fn test_array_reference_semantics() {
    let code = r#"
        let arr1: number[] = [1, 2, 3];
        let arr2: number[] = arr1;
        arr1[0] = 42;
        arr2[0]
    "#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_empty_array() {
    let code = r#"
        let arr: number[] = [];
        len(arr)
    "#;
    assert_eval_number(code, 0.0);
}

#[test]
fn test_stdlib_len_array() {
    let code = r#"
        let arr: number[] = [1, 2, 3, 4];
        len(arr)
    "#;
    assert_eval_number(code, 4.0);
}

#[test]
fn test_nested_array_literal() {
    let code = r#"
        let arr: number[][] = [[1, 2], [3, 4]];
        arr[1][0]
    "#;
    assert_eval_number(code, 3.0);
}

#[test]
fn test_nested_array_mutation() {
    let code = r#"
        let arr: number[][] = [[1, 2], [3, 4]];
        arr[0][1] = 99;
        arr[0][1]
    "#;
    assert_eval_number(code, 99.0);
}

#[test]
fn test_array_whole_number_float_index() {
    let code = r#"
        let arr: number[] = [1, 2, 3];
        arr[1.0]
    "#;
    assert_eval_number(code, 2.0);
}

#[rstest]
#[case("let arr: number[] = [1, 2, 3]; arr[5]", "AT0006")]
#[case("let arr: number[] = [1, 2, 3]; arr[10] = 99; arr[0]", "AT0006")]
fn test_array_out_of_bounds(#[case] code: &str, #[case] error_code: &str) {
    assert_error_code(code, error_code);
}

#[rstest]
#[case("let arr: number[] = [1, 2, 3]; arr[-1]", "AT0103")]
#[case("let arr: number[] = [1, 2, 3]; arr[-1] = 99; arr[0]", "AT0103")]
#[case("let arr: number[] = [1, 2, 3]; arr[1.5]", "AT0103")]
#[case("let arr: number[] = [1, 2, 3]; arr[0.5] = 99; arr[0]", "AT0103")]
fn test_array_invalid_index(#[case] code: &str, #[case] error_code: &str) {
    assert_error_code(code, error_code);
}

#[test]
fn test_array_mutation_in_function() {
    let code = r#"
        fn modify(arr: number[]) -> void {
            arr[0] = 999;
        }
        let numbers: number[] = [1, 2, 3];
        modify(numbers);
        numbers[0]
    "#;
    assert_eval_number(code, 999.0);
}

#[test]
fn test_array_aliasing_multiple_aliases() {
    let code = r#"
        let arr1: number[] = [1, 2, 3];
        let arr2: number[] = arr1;
        let arr3: number[] = arr2;
        arr1[0] = 100;
        arr2[1] = 200;
        arr3[2] = 300;
        arr1[0] + arr2[1] + arr3[2]
    "#;
    assert_eval_number(code, 600.0);
}

#[test]
fn test_array_aliasing_nested_arrays() {
    let code = r#"
        let matrix: number[][] = [[1, 2], [3, 4]];
        let row: number[] = matrix[0];
        row[0] = 99;
        matrix[0][0]
    "#;
    assert_eval_number(code, 99.0);
}

#[test]
fn test_array_aliasing_identity_equality() {
    let code = r#"
        let arr1: number[] = [1, 2, 3];
        let arr2: number[] = arr1;
        arr1 == arr2
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_array_aliasing_different_arrays_not_equal() {
    let code = r#"
        let arr1: number[] = [1, 2, 3];
        let arr2: number[] = [1, 2, 3];
        arr1 == arr2
    "#;
    assert_eval_bool(code, false);
}

#[test]
fn test_array_aliasing_reassignment_breaks_link() {
    let code = r#"
        var arr1: number[] = [1, 2, 3];
        var arr2: number[] = arr1;
        arr2 = [10, 20, 30];
        arr2[0] = 99;
        arr1[0]
    "#;
    assert_eval_number(code, 1.0);
}

#[test]
fn test_array_sum_with_function() {
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
    assert_eval_number(code, 15.0);
}

// ============================================================================
// From integration/interpreter/control_flow.rs
// ============================================================================

#[test]
fn test_if_then() {
    let code = r#"
        var x: number = 0;
        if (true) {
            x = 42;
        }
        x
    "#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_if_else() {
    let code = r#"
        var x: number = 0;
        if (false) {
            x = 10;
        } else {
            x = 20;
        }
        x
    "#;
    assert_eval_number(code, 20.0);
}

#[test]
fn test_if_with_comparison() {
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
    assert_eval_number(code, 1.0);
}

#[test]
fn test_while_loop() {
    let code = r#"
        var i: number = 0;
        var sum: number = 0;
        while (i < 5) {
            sum = sum + i;
            i = i + 1;
        }
        sum
    "#;
    assert_eval_number(code, 10.0);
}

#[test]
fn test_while_loop_with_break() {
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
    assert_eval_number(code, 5.0);
}

#[test]
fn test_while_loop_with_continue() {
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
    assert_eval_number(code, 12.0);
}

#[test]
fn test_for_loop() {
    let code = r#"
        var sum: number = 0;
        for (let i: number = 0; i < 5; i = i + 1) {
            sum = sum + i;
        }
        sum
    "#;
    assert_eval_number(code, 10.0);
}

#[test]
fn test_for_loop_with_break() {
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
    assert_eval_number(code, 4.0);
}

#[test]
fn test_for_loop_with_continue() {
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
    assert_eval_number(code, 8.0);
}

#[test]
fn test_for_loop_with_increment() {
    let code = r#"
        var sum: number = 0;
        for (var i: number = 0; i < 5; i++) {
            sum += i;
        }
        sum
    "#;
    assert_eval_number(code, 10.0);
}

// ============================================================================
// From integration/interpreter/functions.rs
// ============================================================================

#[test]
fn test_function_definition_and_call() {
    let code = r#"
        fn add(a: number, b: number) -> number {
            return a + b;
        }
        add(3, 4)
    "#;
    assert_eval_number(code, 7.0);
}

#[test]
fn test_function_with_local_return() {
    let code = r#"
        fn foo(x: number) -> number {
            let y: number = x + 1;
            return y;
        }
        foo(5)
    "#;
    assert_eval_number(code, 6.0);
}

#[test]
fn test_function_with_early_return() {
    let code = r#"
        fn myAbs(x: number) -> number {
            if (x < 0) {
                return -x;
            }
            return x;
        }
        myAbs(-5)
    "#;
    assert_eval_number(code, 5.0);
}

#[test]
fn test_function_recursion() {
    let code = r#"
        fn factorial(n: number) -> number {
            if (n <= 1) {
                return 1;
            }
            return n * factorial(n - 1);
        }
        factorial(5)
    "#;
    assert_eval_number(code, 120.0);
}

#[test]
fn test_function_with_local_variables() {
    let code = r#"
        fn compute(x: number) -> number {
            let a: number = x + 1;
            let b: number = a * 2;
            return b - 1;
        }
        compute(5)
    "#;
    assert_eval_number(code, 11.0);
}

#[test]
fn test_function_nested_calls() {
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
    assert_eval_number(code, 11.0);
}

#[rstest]
#[case(
    "fn add(a: number, b: number) -> number { return a + b; } add(5)",
    "AT3005"
)]
#[case(
    "fn add(a: number, b: number) -> number { return a + b; } add(1, 2, 3)",
    "AT3005"
)]
fn test_function_wrong_arity(#[case] code: &str, #[case] error_code: &str) {
    assert_error_code(code, error_code);
}

#[test]
fn test_function_void_return() {
    let code = r#"
        var result: number = 0;
        fn set_result(x: number) -> void {
            result = x;
        }
        set_result(42);
        result
    "#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_function_no_parameters() {
    let code = r#"
        fn get_answer() -> number {
            return 42;
        }
        get_answer()
    "#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_function_multiple_parameters() {
    let code = r#"
        fn sum_four(a: number, b: number, c: number, d: number) -> number {
            return a + b + c + d;
        }
        sum_four(1, 2, 3, 4)
    "#;
    assert_eval_number(code, 10.0);
}

#[test]
fn test_function_call_stack_depth() {
    let code = r#"
        fn count_down(n: number) -> number {
            if (n <= 0) {
                return 0;
            }
            return n + count_down(n - 1);
        }
        count_down(5)
    "#;
    assert_eval_number(code, 15.0);
}

#[test]
fn test_function_local_variable_isolation() {
    let code = r#"
        var global: number = 100;
        fn modify_local() -> number {
            let global: number = 50;
            return global;
        }
        let result: number = modify_local();
        result + global
    "#;
    assert_eval_number(code, 150.0);
}

#[test]
fn test_function_mutually_recursive() {
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
    assert_eval_bool(code, true);
}

#[test]
fn test_fibonacci() {
    let code = r#"
        fn fib(n: number) -> number {
            if (n <= 1) {
                return n;
            }
            return fib(n - 1) + fib(n - 2);
        }
        fib(10)
    "#;
    assert_eval_number(code, 55.0);
}

#[test]
fn test_runtime_error_in_function_call() {
    let code = r#"
        fn divide(a: number, b: number) -> number {
            return a / b;
        }
        divide(10, 0)
    "#;
    assert_error_code(code, "AT0005");
}

// ============================================================================
// From integration/interpreter/logical.rs
// ============================================================================

#[rstest]
#[case("5 == 5", true)]
#[case("5 != 3", true)]
#[case("3 < 5", true)]
#[case("5 > 3", true)]
#[case("true && true", true)]
#[case("false || true", true)]
#[case("!false", true)]
#[case("true && false", false)]
#[case("false || false", false)]
#[case("!true", false)]
fn test_comparison_and_boolean_ops(#[case] code: &str, #[case] expected: bool) {
    assert_eval_bool(code, expected);
}

#[test]
fn test_variable_declaration_and_use() {
    let code = r#"
        let x: number = 42;
        x
    "#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_variable_assignment() {
    let code = r#"
        var x: number = 10;
        x = 20;
        x
    "#;
    assert_eval_number(code, 20.0);
}

#[test]
fn test_variable_arithmetic() {
    let code = r#"
        let a: number = 5;
        let b: number = 3;
        a + b
    "#;
    assert_eval_number(code, 8.0);
}

#[test]
fn test_block_scope() {
    let code = r#"
        let x: number = 1;
        if (true) {
            let x: number = 2;
            x;
        }
    "#;
    assert_eval_number(code, 2.0);
}

#[test]
fn test_function_scope() {
    let code = r#"
        var x: number = 10;
        fn foo(x: number) -> number {
            return x + 1;
        }
        foo(5)
    "#;
    assert_eval_number(code, 6.0);
}

// ============================================================================
// From integration/interpreter/strings.rs
// ============================================================================

#[test]
fn test_string_concatenation() {
    let code = r#"
        let s: string = "Hello, " + "World!";
        s
    "#;
    assert_eval_string(code, "Hello, World!");
}

// TODO: Enable when typechecker supports string indexing
#[test]
#[ignore]
fn test_string_indexing() {
    let code = r#"
        let s: string = "Hello";
        s[1]
    "#;
    assert_eval_string(code, "e");
}

#[test]
fn test_stdlib_len_string() {
    let code = r#"
        let s: string = "hello";
        len(s)
    "#;
    assert_eval_number(code, 5.0);
}

#[test]
fn test_stdlib_str() {
    let code = r#"
        let n: number = 42;
        str(n)
    "#;
    assert_eval_string(code, "42");
}

#[rstest]
#[case(r#"var x: number = 5; x++; x"#, 6.0)]
#[case(r#"var x: number = 10; x--; x"#, 9.0)]
#[case(r#"var x: number = 0; x++; x++; x++; x"#, 3.0)]
#[case(r#"var x: number = 10; x--; x--; x"#, 8.0)]
fn test_increment_decrement_basics(#[case] code: &str, #[case] expected: f64) {
    assert_eval_number(code, expected);
}

#[test]
fn test_increment_array_element() {
    let code = r#"
        let arr: number[] = [5, 10, 15];
        arr[0]++;
        arr[0]
    "#;
    assert_eval_number(code, 6.0);
}

#[test]
fn test_decrement_array_element() {
    let code = r#"
        let arr: number[] = [5, 10, 15];
        arr[2]--;
        arr[2]
    "#;
    assert_eval_number(code, 14.0);
}

#[test]
fn test_increment_in_loop() {
    let code = r#"
        var sum: number = 0;
        var i: number = 0;
        while (i < 5) {
            sum += i;
            i++;
        }
        sum
    "#;
    assert_eval_number(code, 10.0);
}

#[rstest]
#[case("let x: number = 5; x++; x", "AT3003")]
#[case("let x: number = 10; x += 5; x", "AT3003")]
fn test_immutable_mutation_errors(#[case] code: &str, #[case] error_code: &str) {
    assert_error_code(code, error_code);
}

#[rstest]
#[case("var x: number = 10; x += 5; x", 15.0)]
#[case("var x: number = 20; x -= 8; x", 12.0)]
#[case("var x: number = 7; x *= 3; x", 21.0)]
#[case("var x: number = 50; x /= 5; x", 10.0)]
#[case("var x: number = 17; x %= 5; x", 2.0)]
fn test_string_concat_assignments(#[case] code: &str, #[case] expected: f64) {
    assert_eval_number(code, expected);
}

#[test]
fn test_compound_chained() {
    let code = r#"
        var x: number = 10;
        x += 5;
        x *= 2;
        x -= 10;
        x
    "#;
    assert_eval_number(code, 20.0);
}

#[test]
fn test_compound_array_element() {
    let code = r#"
        let arr: number[] = [10, 20, 30];
        arr[1] += 5;
        arr[1]
    "#;
    assert_eval_number(code, 25.0);
}

#[test]
fn test_compound_divide_by_zero() {
    let code = r#"
        var x: number = 10;
        x /= 0;
        x
    "#;
    assert_error_code(code, "AT0005");
}
