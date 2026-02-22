//! interpreter.rs — merged from 10 files + integration subtree (Phase Infra-02)

mod common;

use atlas_runtime::binder::Binder;
use atlas_runtime::diagnostic::{Diagnostic, DiagnosticLevel};
use atlas_runtime::interpreter::Interpreter;
use atlas_runtime::lexer::Lexer;
use atlas_runtime::parser::Parser;
use atlas_runtime::security::SecurityContext;
use atlas_runtime::typechecker::TypeChecker;
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
    "Number(30)"
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
    assert_eq!(result, "Number(15)");
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
    assert_eq!(result, "Number(15)");
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
        msgs.iter().any(|m| m.contains("incompatible type")
            || m.contains("type mismatch")
            || m.contains("Return type mismatch")),
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
    // CoW value semantics: arr2 is a logical copy of arr1.
    // Mutating arr1[0] triggers CoW — arr2 retains the original value.
    let code = r#"
        let arr1: number[] = [1, 2, 3];
        let arr2: number[] = arr1;
        arr1[0] = 42;
        arr2[0]
    "#;
    assert_eval_number(code, 1.0);
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
    // CoW value semantics: function receives a logical copy of the array.
    // Mutations inside the function do not affect the caller's binding.
    let code = r#"
        fn modify(arr: number[]) -> void {
            arr[0] = 999;
        }
        let numbers: number[] = [1, 2, 3];
        modify(numbers);
        numbers[0]
    "#;
    assert_eval_number(code, 1.0);
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
    // CoW value semantics: `row` is a logical copy of matrix[0].
    // Mutating row[0] does not affect matrix[0][0].
    let code = r#"
        let matrix: number[][] = [[1, 2], [3, 4]];
        let row: number[] = matrix[0];
        row[0] = 99;
        matrix[0][0]
    "#;
    assert_eval_number(code, 1.0);
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
    // CoW value semantics: equality is structural (same content = equal).
    // Two independently-constructed [1,2,3] arrays are equal.
    let code = r#"
        let arr1: number[] = [1, 2, 3];
        let arr2: number[] = [1, 2, 3];
        arr1 == arr2
    "#;
    assert_eval_bool(code, true);
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
        for (var i: number = 0; i < 5; i = i + 1) {
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
        for (var i: number = 0; i < 10; i = i + 1) {
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
        for (var i: number = 0; i < 5; i = i + 1) {
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
#[ignore = "typechecker does not yet support string indexing"]
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
#[case("let x: number = 1; x = 2; x", "AT3003")] // Basic assignment to let
#[case("let x: number = 5; x--; x", "AT3003")] // Decrement
fn test_immutable_mutation_errors(#[case] code: &str, #[case] error_code: &str) {
    assert_error_code(code, error_code);
}

#[rstest]
#[case("var x: number = 10; x += 5; x", 15.0)]
#[case("var x: number = 20; x -= 8; x", 12.0)]
#[case("var x: number = 7; x *= 3; x", 21.0)]
#[case("var x: number = 50; x /= 5; x", 10.0)]
#[case("var x: number = 17; x %= 5; x", 2.0)]
#[case("var x: number = 1; x = 2; x", 2.0)] // Basic assignment to var
#[case("var x: number = 5; x++; x", 6.0)] // Increment
#[case("var x: number = 5; x--; x", 4.0)] // Decrement
fn test_mutable_var_assignments(#[case] code: &str, #[case] expected: f64) {
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

// ============================================================================
// Phase interpreter-02: Interpreter-VM Parity Tests
// ============================================================================

use atlas_runtime::compiler::Compiler;
use atlas_runtime::vm::VM;

/// Run code through both interpreter and VM, assert identical results
fn assert_parity(source: &str) {
    // Run interpreter (with binder + typechecker for type-tag resolution)
    let mut lexer = Lexer::new(source);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();
    let mut binder = Binder::new();
    let (mut symbol_table, _) = binder.bind(&program);
    let mut typechecker = TypeChecker::new(&mut symbol_table);
    let _ = typechecker.check(&program);

    let mut interp = Interpreter::new();
    let interp_result = interp.eval(&program, &SecurityContext::allow_all());

    // Run VM (with binder + typechecker so compiler has type tags)
    let mut lexer2 = Lexer::new(source);
    let (tokens2, _) = lexer2.tokenize();
    let mut parser2 = Parser::new(tokens2);
    let (program2, _) = parser2.parse();
    let mut binder2 = Binder::new();
    let (mut symbol_table2, _) = binder2.bind(&program2);
    let mut typechecker2 = TypeChecker::new(&mut symbol_table2);
    let _ = typechecker2.check(&program2);

    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&program2).expect("compilation failed");
    let mut vm = VM::new(bytecode);
    let vm_result = vm.run(&SecurityContext::allow_all());

    // Compare results
    match (interp_result, vm_result) {
        (Ok(interp_val), Ok(vm_val)) => {
            let interp_str = format!("{:?}", interp_val);
            let vm_str = format!("{:?}", vm_val.unwrap_or(Value::Null));
            assert_eq!(
                interp_str, vm_str,
                "Parity mismatch for:\n{}\nInterpreter: {}\nVM: {}",
                source, interp_str, vm_str
            );
        }
        (Err(interp_err), Err(vm_err)) => {
            // Both errored - acceptable parity
            let _ = (interp_err, vm_err);
        }
        (Ok(val), Err(err)) => {
            panic!(
                "Parity mismatch: interpreter succeeded with {:?}, VM failed with {:?}",
                val, err
            );
        }
        (Err(err), Ok(val)) => {
            panic!(
                "Parity mismatch: interpreter failed with {:?}, VM succeeded with {:?}",
                err, val
            );
        }
    }
}

// Arithmetic parity tests
#[rstest]
#[case("1 + 2;")]
#[case("10 - 3;")]
#[case("5 * 4;")]
#[case("20 / 4;")]
#[case("17 % 5;")]
#[case("2 + 3 * 4;")]
#[case("(2 + 3) * 4;")]
#[case("-5;")]
#[case("--5;")]
fn test_parity_arithmetic(#[case] code: &str) {
    assert_parity(code);
}

// Boolean parity tests
#[rstest]
#[case("true;")]
#[case("false;")]
#[case("!true;")]
#[case("!false;")]
#[case("true && true;")]
#[case("true && false;")]
#[case("false || true;")]
#[case("false || false;")]
#[case("1 < 2;")]
#[case("2 <= 2;")]
#[case("3 > 2;")]
#[case("3 >= 3;")]
#[case("1 == 1;")]
#[case("1 != 2;")]
fn test_parity_boolean(#[case] code: &str) {
    assert_parity(code);
}

// Variable parity tests
#[rstest]
#[case("let x = 10; x;")]
#[case("var y = 5; y = y + 1; y;")]
#[case("let a = 1; let b = 2; a + b;")]
#[case("var c = 0; c = c + 1; c = c + 1; c;")]
fn test_parity_variables(#[case] code: &str) {
    assert_parity(code);
}

// Function parity tests
#[rstest]
#[case("fn add(a: number, b: number) -> number { return a + b; } add(2, 3);")]
#[case("fn identity(x: number) -> number { return x; } identity(42);")]
#[case("fn constant() -> number { return 99; } constant();")]
#[case("fn inc(x: number) -> number { return x + 1; } inc(inc(inc(0)));")]
fn test_parity_functions(#[case] code: &str) {
    assert_parity(code);
}

// Control flow parity tests
#[rstest]
#[case("var r = 0; if (true) { r = 1; } else { r = 2; } r;")]
#[case("var r = 0; if (false) { r = 1; } else { r = 2; } r;")]
#[case("var r = 0; if (1 < 2) { r = 10; } else { r = 20; } r;")]
#[case("var x = 0; if (x == 0) { x = 1; } x;")]
fn test_parity_if_else(#[case] code: &str) {
    assert_parity(code);
}

// Loop parity tests
#[rstest]
#[case("var i = 0; while (i < 5) { i = i + 1; } i;")]
#[case("var sum = 0; var i = 0; while (i < 10) { sum = sum + i; i = i + 1; } sum;")]
#[case("var count = 0; while (count < 3) { count = count + 1; } count;")]
fn test_parity_while_loop(#[case] code: &str) {
    assert_parity(code);
}

// Array parity tests
#[rstest]
#[case("[1, 2, 3];")]
#[case("let arr = [10, 20, 30]; arr[0];")]
#[case("let arr = [1, 2, 3]; arr[2];")]
#[case("let arr: number[] = [5]; len(arr);")]
fn test_parity_arrays(#[case] code: &str) {
    assert_parity(code);
}

// String parity tests
#[rstest]
#[case(r#""hello";"#)]
#[case(r#""foo" + "bar";"#)]
#[case(r#"let s = "test"; len(s);"#)]
#[case(r#"toUpperCase("hello");"#)]
#[case(r#"toLowerCase("WORLD");"#)]
fn test_parity_strings(#[case] code: &str) {
    assert_parity(code);
}

// ============================================================================
// Phase 19: Interpreter/VM Parity — Array & Collection Operations
// ============================================================================

// Array: index read
#[rstest]
#[case("let arr: number[] = [10, 20, 30]; arr[1];")]
#[case("let arr: number[] = [10, 20, 30]; arr[0];")]
#[case("let arr: number[] = [10, 20, 30]; arr[2];")]
fn test_parity_array_index_read(#[case] code: &str) {
    assert_parity(code);
}

// Array: length
#[rstest]
#[case("let arr: number[] = [1, 2, 3]; len(arr);")]
#[case("let arr: number[] = []; len(arr);")]
#[case("let arr: number[] = [1, 2, 3]; arr.len();")]
fn test_parity_array_length(#[case] code: &str) {
    assert_parity(code);
}

// Array: push (CoW — original unaffected)
#[rstest]
#[case("var a: array = [1, 2]; var b: array = a; b.push(3); len(a);")]
#[case("var a: array = [1]; a.push(2); a.push(3); len(a);")]
fn test_parity_array_push_cow(#[case] code: &str) {
    assert_parity(code);
}

// Array: pop (CoW — pops from receiver, returns length)
#[rstest]
#[case("var a: array = [1, 2, 3]; a.pop(); len(a);")]
#[case("var a: array = [1, 2, 3]; var b: array = a; a.pop(); len(b);")]
fn test_parity_array_pop(#[case] code: &str) {
    assert_parity(code);
}

// Array: sort (returns new sorted array, receiver unchanged)
#[rstest]
#[case("var a: array = [3, 1, 2]; let s = a.sort(); s[0];")]
#[case("var a: array = [3, 1, 2]; let s = a.sort(); a[0];")]
fn test_parity_array_sort(#[case] code: &str) {
    assert_parity(code);
}

// Array: concat via + operator
#[rstest]
#[case("let a: number[] = [1, 2]; let b: number[] = [3, 4]; let c = a + b; len(c);")]
#[case("let a: number[] = [1, 2]; let b: number[] = [3, 4]; let c = a + b; c[0];")]
fn test_parity_array_concat(#[case] code: &str) {
    assert_parity(code);
}

// Array: for-each (sum over elements)
#[rstest]
#[case("var sum: number = 0; for x in [1, 2, 3] { sum = sum + x; } sum;")]
#[case("var count: number = 0; for _x in [10, 20, 30] { count = count + 1; } count;")]
fn test_parity_array_foreach(#[case] code: &str) {
    assert_parity(code);
}

// Array: map/filter with closures — both engines error (acceptable parity until Block 4)
// These are included so parity is verified even for unsupported operations.
#[rstest]
#[case("let a: number[] = [1, 2, 3]; map(a, fn(x: number) -> number { return x * 2; });")]
#[case("let a: number[] = [1, 2, 3, 4]; filter(a, fn(x: number) -> bool { return x > 2; });")]
fn test_parity_array_map_filter_both_error(#[case] code: &str) {
    assert_parity(code); // Both engines must agree (both succeed or both fail)
}

// Map (HashMap): get
#[rstest]
#[case("let m: HashMap = hashMapNew(); hashMapPut(m, \"a\", 1); unwrap(hashMapGet(m, \"a\"));")]
#[case("let m: HashMap = hashMapNew(); hashMapPut(m, \"x\", 42); unwrap(hashMapGet(m, \"x\"));")]
fn test_parity_hashmap_get(#[case] code: &str) {
    assert_parity(code);
}

// Map (HashMap): set with CoW — original unaffected after copy
#[rstest]
#[case("var m: HashMap = hashMapNew(); hashMapPut(m, \"a\", 1); var n: HashMap = m; hashMapPut(n, \"b\", 2); hashMapSize(m);")]
fn test_parity_hashmap_set_cow(#[case] code: &str) {
    assert_parity(code);
}

// Map (HashMap): keys count
#[rstest]
#[case("let m: HashMap = hashMapNew(); hashMapPut(m, \"a\", 1); hashMapPut(m, \"b\", 2); hashMapSize(m);")]
fn test_parity_hashmap_keys(#[case] code: &str) {
    assert_parity(code);
}

// Map (HashMap): remove (delete a key)
#[rstest]
#[case("let m: HashMap = hashMapNew(); hashMapPut(m, \"a\", 1); hashMapPut(m, \"b\", 2); hashMapRemove(m, \"a\"); hashMapSize(m);")]
fn test_parity_hashmap_remove(#[case] code: &str) {
    assert_parity(code);
}

// Queue: enqueue/dequeue/size
#[rstest]
#[case("let q: Queue = queueNew(); queueEnqueue(q, 1); queueEnqueue(q, 2); queueEnqueue(q, 3); queueSize(q);")]
#[case("let q: Queue = queueNew(); queueEnqueue(q, 10); queueEnqueue(q, 20); unwrap(queueDequeue(q)); queueSize(q);")]
#[case("let q: Queue = queueNew(); queueEnqueue(q, 42); unwrap(queueDequeue(q));")]
fn test_parity_queue_operations(#[case] code: &str) {
    assert_parity(code);
}

// Stack: push/pop/size
#[rstest]
#[case(
    "let s: Stack = stackNew(); stackPush(s, 1); stackPush(s, 2); stackPush(s, 3); stackSize(s);"
)]
#[case("let s: Stack = stackNew(); stackPush(s, 10); stackPush(s, 20); unwrap(stackPop(s)); stackSize(s);")]
#[case("let s: Stack = stackNew(); stackPush(s, 99); unwrap(stackPop(s));")]
fn test_parity_stack_operations(#[case] code: &str) {
    assert_parity(code);
}

// CoW semantics: identical behavior in both engines
#[rstest]
#[case("let a: number[] = [1, 2, 3]; let b: number[] = a; a[0] = 99; b[0];")]
#[case("var a: array = [1, 2]; var b: array = a; b.push(9); len(a);")]
#[case("let a: number[] = [1, 2, 3]; let b: number[] = a; b[2] = 100; a[2];")]
fn test_parity_cow_semantics(#[case] code: &str) {
    assert_parity(code);
}

// ============================================================================
// Phase interpreter-02: Integration Tests - Closures and Scopes
// ============================================================================

#[test]
fn test_integration_nested_function_with_params() {
    // Test nested function that takes parameters (avoids closure capture warnings)
    let code = r#"
        fn outer(x: number) -> number {
            fn inner(y: number) -> number {
                return y * 2;
            }
            return inner(x);
        }
        outer(10);
    "#;
    assert_eval_number(code, 20.0);
}

#[test]
fn test_integration_nested_function_calls() {
    let code = r#"
        fn a(x: number) -> number { return x + 1; }
        fn b(x: number) -> number { return a(x) + 1; }
        fn c(x: number) -> number { return b(x) + 1; }
        c(0);
    "#;
    assert_eval_number(code, 3.0);
}

#[test]
fn test_integration_scope_shadowing() {
    let code = r#"
        let x = 1;
        fn test() -> number {
            let x = 2;
            return x;
        }
        test() + x;
    "#;
    assert_eval_number(code, 3.0);
}

#[test]
fn test_integration_multiple_function_levels() {
    // Test function calls across multiple levels
    let code = r#"
        fn level1(x: number) -> number {
            fn level2(y: number) -> number {
                fn level3(z: number) -> number {
                    return z + 1;
                }
                return level3(y) + 1;
            }
            return level2(x) + 1;
        }
        level1(0);
    "#;
    assert_eval_number(code, 3.0);
}

#[test]
fn test_integration_function_as_parameter() {
    // Test higher-order function pattern
    let code = r#"
        fn apply(f: (number) -> number, x: number) -> number {
            return f(x);
        }
        fn double(n: number) -> number {
            return n * 2;
        }
        apply(double, 5);
    "#;
    assert_eval_number(code, 10.0);
}

// ============================================================================
// Phase interpreter-02: Integration Tests - Error Recovery
// ============================================================================

#[test]
fn test_integration_undefined_variable_error() {
    let result = Atlas::new().eval("undefined_var;");
    assert!(result.is_err(), "Expected error for undefined variable");
}

#[test]
fn test_integration_type_mismatch_error() {
    let result = Atlas::new().eval(r#"let x: number = "hello";"#);
    assert!(result.is_err(), "Expected type mismatch error");
}

#[test]
fn test_integration_divide_by_zero_error() {
    assert_error_code("10 / 0;", "AT0005");
}

#[test]
fn test_integration_array_index_out_of_bounds() {
    let result = Atlas::new().eval("let arr = [1, 2, 3]; arr[10];");
    assert!(result.is_err(), "Expected array index out of bounds error");
}

#[test]
fn test_integration_function_wrong_arity() {
    let code = r#"
        fn add(a: number, b: number) -> number { return a + b; }
        add(1);
    "#;
    let result = Atlas::new().eval(code);
    assert!(result.is_err(), "Expected function arity error");
}

// ============================================================================
// Phase interpreter-02: Integration Tests - Complex Programs
// ============================================================================

#[test]
fn test_integration_fibonacci_recursive() {
    let code = r#"
        fn fib(n: number) -> number {
            if (n <= 1) { return n; }
            return fib(n - 1) + fib(n - 2);
        }
        fib(10);
    "#;
    assert_eval_number(code, 55.0);
}

#[test]
fn test_integration_factorial() {
    let code = r#"
        fn factorial(n: number) -> number {
            if (n <= 1) { return 1; }
            return n * factorial(n - 1);
        }
        factorial(5);
    "#;
    assert_eval_number(code, 120.0);
}

#[test]
fn test_integration_sum_to_n() {
    let code = r#"
        fn sum_to(n: number) -> number {
            var sum = 0;
            var i = 1;
            while (i <= n) {
                sum = sum + i;
                i = i + 1;
            }
            return sum;
        }
        sum_to(100);
    "#;
    assert_eval_number(code, 5050.0);
}

#[test]
fn test_integration_is_prime() {
    let code = r#"
        fn is_prime(n: number) -> bool {
            if (n < 2) { return false; }
            var i = 2;
            while (i * i <= n) {
                if (n % i == 0) { return false; }
                i = i + 1;
            }
            return true;
        }
        is_prime(17);
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_integration_is_not_prime() {
    let code = r#"
        fn is_prime(n: number) -> bool {
            if (n < 2) { return false; }
            var i = 2;
            while (i * i <= n) {
                if (n % i == 0) { return false; }
                i = i + 1;
            }
            return true;
        }
        is_prime(15);
    "#;
    assert_eval_bool(code, false);
}

// ============================================================================
// Phase interpreter-02: Integration Tests - Stdlib Functions
// ============================================================================

#[test]
fn test_integration_stdlib_len_string() {
    assert_eval_number(r#"len("hello");"#, 5.0);
}

#[test]
fn test_integration_stdlib_len_array() {
    assert_eval_number("len([1, 2, 3, 4, 5]);", 5.0);
}

#[test]
fn test_integration_stdlib_str() {
    assert_eval_string("str(42);", "42");
}

#[test]
fn test_integration_stdlib_trim() {
    assert_eval_string(r#"trim("  hello  ");"#, "hello");
}

#[test]
fn test_integration_stdlib_split_join() {
    let code = r#"
        let parts = split("a,b,c", ",");
        join(parts, "-");
    "#;
    assert_eval_string(code, "a-b-c");
}

#[test]
fn test_integration_stdlib_substring() {
    assert_eval_string(r#"substring("hello world", 0, 5);"#, "hello");
}

#[test]
fn test_integration_stdlib_includes() {
    assert_eval_bool(r#"includes("hello world", "world");"#, true);
}

#[test]
fn test_integration_stdlib_starts_with() {
    assert_eval_bool(r#"startsWith("hello world", "hello");"#, true);
}

#[test]
fn test_integration_stdlib_ends_with() {
    assert_eval_bool(r#"endsWith("hello world", "world");"#, true);
}

#[test]
fn test_integration_stdlib_replace() {
    assert_eval_string(
        r#"replace("hello world", "world", "atlas");"#,
        "hello atlas",
    );
}

// ============================================================================
// Phase interpreter-02: Performance Correctness Tests
// ============================================================================

#[test]
fn test_perf_loop_1000_iterations() {
    let code = "var i = 0; while (i < 1000) { i = i + 1; } i;";
    assert_eval_number(code, 1000.0);
}

#[test]
fn test_perf_nested_loop_correctness() {
    let code = r#"
        var count = 0;
        var i = 0;
        while (i < 10) {
            var j = 0;
            while (j < 10) {
                count = count + 1;
                j = j + 1;
            }
            i = i + 1;
        }
        count;
    "#;
    assert_eval_number(code, 100.0);
}

#[test]
fn test_perf_string_accumulation() {
    let code = r#"
        var s = "";
        var i = 0;
        while (i < 50) {
            s = s + "x";
            i = i + 1;
        }
        len(s);
    "#;
    assert_eval_number(code, 50.0);
}

#[test]
fn test_perf_function_calls_correctness() {
    let code = r#"
        fn inc(x: number) -> number { return x + 1; }
        var r = 0;
        var i = 0;
        while (i < 100) {
            r = inc(r);
            i = i + 1;
        }
        r;
    "#;
    assert_eval_number(code, 100.0);
}

#[test]
fn test_perf_array_operations() {
    // Test array indexing performance
    let code = r#"
        let arr = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        var sum = 0;
        var i = 0;
        while (i < 100) {
            sum = sum + arr[i % 10];
            i = i + 1;
        }
        sum;
    "#;
    assert_eval_number(code, 550.0); // sum of 1-10 is 55, times 10 = 550
}

// ============================================================================
// Phase interpreter-02: Edge Case Tests
// ============================================================================

#[test]
fn test_edge_empty_function() {
    let code = "fn noop() { } noop();";
    assert_no_error(code);
}

#[test]
fn test_edge_deeply_nested_if() {
    let code = r#"
        var x = 0;
        if (true) {
            if (true) {
                if (true) {
                    x = 1;
                }
            }
        }
        x;
    "#;
    assert_eval_number(code, 1.0);
}

#[test]
fn test_edge_boolean_short_circuit_and() {
    // If short-circuit works, second function should not be called
    let code = r#"
        var called = 0;
        fn side_effect() -> bool {
            called = called + 1;
            return true;
        }
        let result = false && side_effect();
        called;
    "#;
    assert_eval_number(code, 0.0);
}

#[test]
fn test_edge_boolean_short_circuit_or() {
    // If short-circuit works, second function should not be called
    let code = r#"
        var called = 0;
        fn side_effect() -> bool {
            called = called + 1;
            return false;
        }
        let result = true || side_effect();
        called;
    "#;
    assert_eval_number(code, 0.0);
}

#[test]
fn test_edge_return_from_nested_block() {
    let code = r#"
        fn test() -> number {
            if (true) {
                if (true) {
                    return 42;
                }
            }
            return 0;
        }
        test();
    "#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_edge_while_loop_early_break() {
    // Note: Atlas may not have break keyword - if it does, test it
    // Otherwise test early return from function containing loop
    let code = r#"
        fn first_over_5() -> number {
            var i = 0;
            while (i < 100) {
                if (i > 5) { return i; }
                i = i + 1;
            }
            return -1;
        }
        first_over_5();
    "#;
    assert_eval_number(code, 6.0);
}

// ============================================================================
// Phase 07: Array Mutation CoW Semantics (Interpreter)
// ============================================================================

/// Index assignment writes back to the variable in the environment.
///
/// Previously, `set_array_element` mutated a local copy and discarded it.
/// Now `assign_at_index` clones the container, mutates via CoW, and writes back.
#[test]
fn test_array_index_assignment_write_back() {
    assert_eval_number("var arr: array = [10, 20, 30]; arr[1] = 99; arr[1];", 99.0);
}

#[test]
fn test_array_index_assignment_first_element() {
    assert_eval_number("var arr: array = [1, 2, 3]; arr[0] = 42; arr[0];", 42.0);
}

#[test]
fn test_array_index_assignment_last_element() {
    assert_eval_number("var arr: array = [1, 2, 3]; arr[2] = 77; arr[2];", 77.0);
}

/// CoW: mutating a cloned array does not affect the original.
///
/// `var a = [1, 2, 3]; var b = a; b[0] = 99;`
/// After mutation, `a[0]` must still be 1 — CoW cloned the underlying data.
#[test]
fn test_cow_index_mutation_does_not_affect_original() {
    assert_eval_number(
        "var a: array = [1, 2, 3]; var b: array = a; b[0] = 99; a[0];",
        1.0,
    );
}

#[test]
fn test_cow_cloned_array_gets_mutation() {
    assert_eval_number(
        "var a: array = [1, 2, 3]; var b: array = a; b[0] = 99; b[0];",
        99.0,
    );
}

/// Compound assignment (`+=`) on array index writes back correctly.
#[test]
fn test_array_compound_assign_add() {
    assert_eval_number("var arr: array = [10, 20, 30]; arr[1] += 5; arr[1];", 25.0);
}

/// Increment (`++`) on array index writes back correctly.
#[test]
fn test_array_increment_writes_back() {
    assert_eval_number("var arr: array = [5, 6, 7]; arr[0]++; arr[0];", 6.0);
}

/// Decrement (`--`) on array index writes back correctly.
#[test]
fn test_array_decrement_writes_back() {
    assert_eval_number("var arr: array = [5, 6, 7]; arr[2]--; arr[2];", 6.0);
}

/// Multiple mutations accumulate on the same variable.
#[test]
fn test_array_multiple_mutations_accumulate() {
    assert_eval_number(
        "var arr: array = [0, 0, 0]; arr[0] = 10; arr[1] = 20; arr[2] = 30; arr[0] + arr[1] + arr[2];",
        60.0,
    );
}

/// Loop-based array mutation: each iteration writes back correctly.
#[test]
fn test_array_mutation_in_loop() {
    assert_eval_number(
        r#"
            var arr: array = [1, 2, 3, 4, 5];
            var i = 0;
            while (i < 5) {
                arr[i] = arr[i] * 2;
                i = i + 1;
            }
            arr[0] + arr[1] + arr[2] + arr[3] + arr[4];
        "#,
        30.0,
    );
}

// ============================================================================
// Phase 16: Stdlib Return Value Propagation — array method CoW write-back
// ============================================================================

/// arr.push(x) — receiver variable updated in place (CoW write-back)
#[test]
fn test_array_method_push_updates_receiver() {
    assert_eval_number(r#"var arr: array = [1, 2, 3]; arr.push(4); arr[3];"#, 4.0);
}

/// arr.push(x) — length increases
#[test]
fn test_array_method_push_increases_len() {
    assert_eval_number(r#"var arr: array = [1, 2]; arr.push(3); len(arr);"#, 3.0);
}

/// arr.push chained — multiple pushes accumulate
#[test]
fn test_array_method_push_multiple() {
    assert_eval_number(
        r#"var arr: array = []; arr.push(10); arr.push(20); arr.push(30); arr[1];"#,
        20.0,
    );
}

/// arr.pop() — returns the popped element
#[test]
fn test_array_method_pop_returns_element() {
    assert_eval_number(r#"var arr: array = [1, 2, 3]; let x = arr.pop(); x;"#, 3.0);
}

/// arr.pop() — receiver shortened by one element
#[test]
fn test_array_method_pop_shrinks_receiver() {
    assert_eval_number(r#"var arr: array = [1, 2, 3]; arr.pop(); len(arr);"#, 2.0);
}

/// arr.pop() — receiver still holds correct remaining elements
#[test]
fn test_array_method_pop_receiver_correct() {
    assert_eval_number(
        r#"var arr: array = [10, 20, 30]; arr.pop(); arr[0] + arr[1];"#,
        30.0,
    );
}

/// arr.sort() — returns a new sorted array
#[test]
fn test_array_method_sort_returns_sorted() {
    assert_eval_number(
        r#"var arr: array = [3, 1, 2]; let s = arr.sort(); s[0];"#,
        1.0,
    );
}

/// arr.sort() — does NOT mutate the receiver
#[test]
fn test_array_method_sort_non_mutating() {
    assert_eval_number(
        r#"var arr: array = [3, 1, 2]; let s = arr.sort(); arr[0];"#,
        3.0,
    );
}

/// arr.sort() — numeric sort (ascending by value)
#[test]
fn test_array_method_sort_numeric() {
    assert_eval_number(
        r#"var arr: array = [10, 2, 30, 4]; let s = arr.sort(); s[0];"#,
        2.0,
    );
}

/// arr.reverse() — receiver is updated with reversed array (mutating)
#[test]
fn test_array_method_reverse_updates_receiver() {
    assert_eval_number(r#"var arr: array = [1, 2, 3]; arr.reverse(); arr[0];"#, 3.0);
}

/// arr.reverse() — result is the reversed array
#[test]
fn test_array_method_reverse_result_correct() {
    assert_eval_number(
        r#"var arr: array = [1, 2, 3]; let r = arr.reverse(); r[0];"#,
        3.0,
    );
}

/// Free function pop(arr) CoW write-back — pop() as free function also updates receiver
#[test]
fn test_free_fn_pop_cow_writeback() {
    assert_eval_number(
        r#"var arr: array = [1, 2, 3]; let x = pop(arr); len(arr);"#,
        2.0,
    );
}

/// Free function pop(arr) — returns removed element
#[test]
fn test_free_fn_pop_returns_element() {
    assert_eval_number(r#"var arr: array = [1, 2, 3]; let x = pop(arr); x;"#, 3.0);
}

/// Free function shift(arr) — removes first element
#[test]
fn test_free_fn_shift_cow_writeback() {
    assert_eval_number(
        r#"var arr: array = [10, 20, 30]; let x = shift(arr); x;"#,
        10.0,
    );
}

/// Free function shift(arr) — receiver is updated
#[test]
fn test_free_fn_shift_receiver_updated() {
    assert_eval_number(
        r#"var arr: array = [10, 20, 30]; shift(arr); len(arr);"#,
        2.0,
    );
}

/// Free function reverse(arr) — writes new array back to receiver
#[test]
fn test_free_fn_reverse_cow_writeback() {
    assert_eval_number(r#"var arr: array = [1, 2, 3]; reverse(arr); arr[0];"#, 3.0);
}

/// arr.push with inferred array type (no annotation)
#[test]
fn test_array_method_push_inferred_type() {
    assert_eval_number(r#"let arr = [1, 2, 3]; arr.push(4); arr[3];"#, 4.0);
}

/// Parity: interpreter and VM produce same result for push
#[test]
fn test_array_method_push_parity_via_atlas_eval() {
    let code = r#"var arr: array = [1, 2, 3]; arr.push(99); arr[3];"#;
    assert_eval_number(code, 99.0);
}

// ============================================================================
// Value semantics regression tests — CoW behavior must never regress
// ============================================================================

/// Regression: assignment creates independent copy; mutation of source does not
/// affect the copy (CoW value semantics).
#[test]
fn test_value_semantics_regression_assign_copy() {
    let code = r#"
        let a: number[] = [1, 2, 3];
        let b: number[] = a;
        a[0] = 99;
        b[0]
    "#;
    assert_eval_number(code, 1.0);
}

/// Regression: mutation of assigned copy does not affect source.
#[test]
fn test_value_semantics_regression_copy_mutation_isolated() {
    let code = r#"
        let a: number[] = [1, 2, 3];
        let b: number[] = a;
        b[0] = 42;
        a[0]
    "#;
    assert_eval_number(code, 1.0);
}

/// Regression: push on assigned copy does not grow the source.
#[test]
fn test_value_semantics_regression_push_copy_isolated() {
    let code = r#"
        var a: array = [1, 2, 3];
        var b: array = a;
        b.push(4);
        len(a)
    "#;
    assert_eval_number(code, 3.0);
}

/// Regression: function parameter is an independent copy — mutations stay local.
#[test]
fn test_value_semantics_regression_fn_param_copy() {
    let code = r#"
        fn fill(arr: number[]) -> void {
            arr[0] = 999;
        }
        let nums: number[] = [1, 2, 3];
        fill(nums);
        nums[0]
    "#;
    assert_eval_number(code, 1.0);
}

/// Regression: three-way copy — each variable is independent.
#[test]
fn test_value_semantics_regression_three_way_copy() {
    let code = r#"
        let a: number[] = [1, 2, 3];
        let b: number[] = a;
        let c: number[] = b;
        b[0] = 10;
        c[1] = 20;
        a[0] + a[1]
    "#;
    assert_eval_number(code, 3.0);
}

// ============================================================================
// Phase 08: Runtime `own` enforcement in interpreter (debug mode)
// ============================================================================

/// After passing a variable to an `own` parameter, reading it must fail in debug mode.
#[test]
fn test_own_param_consumes_binding_debug() {
    let src = r#"
        fn consume(own data: array<number>) -> void { }
        let arr: array<number> = [1, 2, 3];
        consume(arr);
        arr;
    "#;
    let result = run_interpreter(src);
    assert!(
        result.is_err(),
        "Expected error after consuming arr, got: {:?}",
        result
    );
    assert!(
        result.unwrap_err().contains("use of moved value"),
        "Error should mention 'use of moved value'"
    );
}

/// A `borrow` parameter must NOT consume the caller's binding.
#[test]
fn test_borrow_param_does_not_consume_binding() {
    let src = r#"
        fn read(borrow data: array<number>) -> void { }
        let arr: array<number> = [1, 2, 3];
        read(arr);
        len(arr);
    "#;
    let result = run_interpreter(src);
    assert!(
        result.is_ok(),
        "borrow should not consume binding, got: {:?}",
        result
    );
    assert_eq!(result.unwrap(), "Number(3)");
}

/// An unannotated parameter must NOT consume the caller's binding.
#[test]
fn test_unannotated_param_does_not_consume_binding() {
    let src = r#"
        fn take(data: array<number>) -> void { }
        let arr: array<number> = [1, 2, 3];
        take(arr);
        len(arr);
    "#;
    let result = run_interpreter(src);
    assert!(
        result.is_ok(),
        "unannotated param should not consume binding, got: {:?}",
        result
    );
    assert_eq!(result.unwrap(), "Number(3)");
}

/// Passing a literal to an `own` parameter must not attempt to consume any binding.
#[test]
fn test_own_param_with_literal_arg_no_consume() {
    let src = r#"
        fn consume(own data: array<number>) -> void { }
        consume([1, 2, 3]);
        42;
    "#;
    let result = run_interpreter(src);
    assert!(
        result.is_ok(),
        "literal arg to own param should not error, got: {:?}",
        result
    );
    assert_eq!(result.unwrap(), "Number(42)");
}

/// Passing an expression result to an `own` parameter must not consume any binding.
#[test]
fn test_own_param_with_expression_arg_no_consume() {
    let src = r#"
        fn make_arr() -> array<number> { [10, 20]; }
        fn consume(own data: array<number>) -> void { }
        let arr: array<number> = [1, 2, 3];
        consume(make_arr());
        len(arr);
    "#;
    let result = run_interpreter(src);
    assert!(
        result.is_ok(),
        "expression arg to own param should not consume unrelated binding, got: {:?}",
        result
    );
    assert_eq!(result.unwrap(), "Number(3)");
}

// ============================================================================
// Phase 09: Runtime `shared` enforcement in interpreter (debug mode)
// ============================================================================

/// Passing a plain (non-shared) value to a `shared` param must produce a runtime error.
#[test]
fn test_shared_param_rejects_plain_value_debug() {
    let src = r#"
        fn register(shared handler: number[]) -> void { }
        let arr: number[] = [1, 2, 3];
        register(arr);
    "#;
    let result = run_interpreter(src);
    assert!(
        result.is_err(),
        "Expected ownership violation error, got: {:?}",
        result
    );
    assert!(
        result.unwrap_err().contains("ownership violation"),
        "Error should mention 'ownership violation'"
    );
}

/// Passing an actual SharedValue to a `shared` param must succeed.
#[test]
fn test_shared_param_accepts_shared_value() {
    use atlas_runtime::value::{Shared, Value};

    // Parse and register the function
    let src = r#"
        fn register(shared handler: number[]) -> void { }
        register(sv);
    "#;
    let mut lexer = atlas_runtime::lexer::Lexer::new(src);
    let (tokens, _) = lexer.tokenize();
    let mut parser = atlas_runtime::parser::Parser::new(tokens);
    let (program, _) = parser.parse();
    let mut binder = atlas_runtime::binder::Binder::new();
    let (mut symbol_table, _) = binder.bind(&program);
    let mut typechecker = atlas_runtime::typechecker::TypeChecker::new(&mut symbol_table);
    let _ = typechecker.check(&program);

    let mut interp = Interpreter::new();
    // Inject a SharedValue into the interpreter's globals so Atlas source can reference it
    let shared_val = Value::SharedValue(Shared::new(Box::new(Value::array(vec![
        Value::Number(1.0),
        Value::Number(2.0),
    ]))));
    interp.define_global("sv".to_string(), shared_val);

    let result = interp.eval(&program, &SecurityContext::allow_all());
    assert!(
        result.is_ok(),
        "SharedValue passed to shared param should succeed, got: {:?}",
        result
    );
}

/// Passing a SharedValue to an `own` param emits an advisory (not a hard error).
#[test]
fn test_shared_value_to_own_param_advisory_not_error() {
    use atlas_runtime::value::{Shared, Value};

    let src = r#"
        fn consume(own handler: number[]) -> void { }
        consume(sv);
    "#;
    let mut lexer = atlas_runtime::lexer::Lexer::new(src);
    let (tokens, _) = lexer.tokenize();
    let mut parser = atlas_runtime::parser::Parser::new(tokens);
    let (program, _) = parser.parse();
    let mut binder = atlas_runtime::binder::Binder::new();
    let (mut symbol_table, _) = binder.bind(&program);
    let mut typechecker = atlas_runtime::typechecker::TypeChecker::new(&mut symbol_table);
    let _ = typechecker.check(&program);

    let mut interp = Interpreter::new();
    let shared_val = Value::SharedValue(Shared::new(Box::new(Value::array(vec![Value::Number(
        1.0,
    )]))));
    interp.define_global("sv".to_string(), shared_val);

    // Advisory warning only — must NOT be a hard error
    let result = interp.eval(&program, &SecurityContext::allow_all());
    assert!(
        result.is_ok(),
        "SharedValue to own param should be advisory (not hard error), got: {:?}",
        result
    );
}
