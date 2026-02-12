//! Comprehensive tests for function return analysis
//!
//! Tests cover:
//! - All code paths must return for non-void/non-null functions
//! - If/else branch return analysis
//! - Nested control flow return analysis
//! - Early returns
//! - Functions that don't need to return (void/null)
//! - Missing return diagnostics (AT3004)

use atlas_runtime::binder::Binder;
use atlas_runtime::diagnostic::{Diagnostic, DiagnosticLevel};
use atlas_runtime::lexer::Lexer;
use atlas_runtime::parser::Parser;
use atlas_runtime::typechecker::TypeChecker;

fn typecheck_source(source: &str) -> Vec<Diagnostic> {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, lex_diags) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, parse_diags) = parser.parse();

    let mut binder = Binder::new();
    let (table, bind_diags) = binder.bind(&program);

    let mut checker = TypeChecker::new(&table);
    let type_diags = checker.check(&program);

    // Combine all diagnostics
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
    assert!(
        !diagnostics.is_empty(),
        "Expected at least one diagnostic with code {}",
        code
    );
    let found = diagnostics.iter().any(|d| d.code == code);
    assert!(
        found,
        "Expected diagnostic with code {}, got: {:?}",
        code,
        diagnostics.iter().map(|d| &d.code).collect::<Vec<_>>()
    );
}

// ========== Functions That Always Return ==========

#[test]
fn test_simple_return() {
    let diagnostics = typecheck_source(
        r#"
        fn getNumber() -> number {
            return 42;
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

#[test]
fn test_return_expression() {
    let diagnostics = typecheck_source(
        r#"
        fn add(a: number, b: number) -> number {
            return a + b;
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

#[test]
fn test_return_after_statements() {
    let diagnostics = typecheck_source(
        r#"
        fn calculate(x: number) -> number {
            let y: number = x * 2;
            let z: number = y + 10;
            return z;
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

#[test]
fn test_early_return() {
    let diagnostics = typecheck_source(
        r#"
        fn abs(x: number) -> number {
            if (x < 0) {
                return -x;
            }
            return x;
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

// ========== Missing Return Errors ==========

#[test]
fn test_missing_return_error() {
    let diagnostics = typecheck_source(
        r#"
        fn getNumber() -> number {
        }
    "#,
    );
    assert_has_error(&diagnostics, "AT3004"); // Not all code paths return
}

#[test]
fn test_missing_return_with_statements() {
    let diagnostics = typecheck_source(
        r#"
        fn calculate(x: number) -> number {
            let y: number = x * 2;
            let z: number = y + 10;
        }
    "#,
    );
    assert_has_error(&diagnostics, "AT3004"); // Not all code paths return
}

#[test]
fn test_missing_return_string_function() {
    let diagnostics = typecheck_source(
        r#"
        fn getMessage() -> string {
            let msg: string = "hello";
        }
    "#,
    );
    assert_has_error(&diagnostics, "AT3004"); // Not all code paths return
}

#[test]
fn test_missing_return_bool_function() {
    let diagnostics = typecheck_source(
        r#"
        fn isPositive(x: number) -> bool {
            let result: bool = x > 0;
        }
    "#,
    );
    assert_has_error(&diagnostics, "AT3004"); // Not all code paths return
}

// ========== If/Else Return Analysis ==========

#[test]
fn test_if_else_both_return() {
    let diagnostics = typecheck_source(
        r#"
        fn abs(x: number) -> number {
            if (x < 0) {
                return -x;
            } else {
                return x;
            }
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

#[test]
fn test_if_else_only_if_returns_error() {
    let diagnostics = typecheck_source(
        r#"
        fn test(x: number) -> number {
            if (x > 0) {
                return x;
            } else {
                let y: number = 1;
            }
        }
    "#,
    );
    assert_has_error(&diagnostics, "AT3004"); // else branch doesn't return
}

#[test]
fn test_if_else_only_else_returns_error() {
    let diagnostics = typecheck_source(
        r#"
        fn test(x: number) -> number {
            if (x > 0) {
                let y: number = 1;
            } else {
                return x;
            }
        }
    "#,
    );
    assert_has_error(&diagnostics, "AT3004"); // if branch doesn't return
}

#[test]
fn test_if_without_else_returns_error() {
    let diagnostics = typecheck_source(
        r#"
        fn test(x: number) -> number {
            if (x > 0) {
                return x;
            }
        }
    "#,
    );
    assert_has_error(&diagnostics, "AT3004"); // no else branch
}

#[test]
fn test_if_without_else_then_return() {
    let diagnostics = typecheck_source(
        r#"
        fn test(x: number) -> number {
            if (x > 0) {
                return x * 2;
            }
            return x;
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

// ========== Nested If/Else Return Analysis ==========

#[test]
fn test_nested_if_else_all_return() {
    let diagnostics = typecheck_source(
        r#"
        fn classify(x: number) -> number {
            if (x > 0) {
                if (x > 10) {
                    return 2;
                } else {
                    return 1;
                }
            } else {
                return 0;
            }
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

#[test]
fn test_nested_if_missing_inner_return() {
    let diagnostics = typecheck_source(
        r#"
        fn test(x: number) -> number {
            if (x > 0) {
                if (x > 10) {
                    return 2;
                } else {
                    let y: number = 1;
                }
            } else {
                return 0;
            }
        }
    "#,
    );
    assert_has_error(&diagnostics, "AT3004"); // inner else doesn't return
}

#[test]
fn test_nested_if_missing_outer_return() {
    let diagnostics = typecheck_source(
        r#"
        fn test(x: number) -> number {
            if (x > 0) {
                if (x > 10) {
                    return 2;
                } else {
                    return 1;
                }
            } else {
                let y: number = 0;
            }
        }
    "#,
    );
    assert_has_error(&diagnostics, "AT3004"); // outer else doesn't return
}

#[test]
fn test_deeply_nested_all_return() {
    let diagnostics = typecheck_source(
        r#"
        fn classify(x: number, y: number) -> number {
            if (x > 0) {
                if (y > 0) {
                    if (x > y) {
                        return 1;
                    } else {
                        return 2;
                    }
                } else {
                    return 3;
                }
            } else {
                return 4;
            }
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

// ========== Void Functions (Don't Need Return) ==========

// NOTE: 'void' as a return type may not be fully supported in the parser yet.
// Functions that don't need to return a value can use any return type and
// the compiler will check that all paths return appropriately.

// ========== Multiple Returns in Same Block ==========

#[test]
fn test_multiple_early_returns() {
    let diagnostics = typecheck_source(
        r#"
        fn classify(x: number) -> number {
            if (x < 0) {
                return -1;
            }
            if (x == 0) {
                return 0;
            }
            return 1;
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

#[test]
fn test_unreachable_code_after_return() {
    let diagnostics = typecheck_source(
        r#"
        fn test() -> number {
            return 42;
            let x: number = 1;
        }
    "#,
    );
    // This should work (unreachable code is a warning, not an error)
    assert_no_errors(&diagnostics);
}

// ========== Returns in Loops ==========

#[test]
fn test_return_in_while_loop_not_sufficient() {
    let diagnostics = typecheck_source(
        r#"
        fn test(x: number) -> number {
            while (x > 0) {
                return x;
            }
        }
    "#,
    );
    assert_has_error(&diagnostics, "AT3004"); // loop might not execute
}

#[test]
fn test_return_after_loop() {
    let diagnostics = typecheck_source(
        r#"
        fn sum(n: number) -> number {
            let s: number = 0;
            let i: number = 0;
            while (i < n) {
                s = s + i;
                i = i + 1;
            }
            return s;
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

#[test]
fn test_return_in_for_loop_not_sufficient() {
    let diagnostics = typecheck_source(
        r#"
        fn test() -> number {
            for (let i: number = 0; i < 10; i = i + 1) {
                return i;
            }
        }
    "#,
    );
    assert_has_error(&diagnostics, "AT3004"); // loop might not execute
}

#[test]
fn test_return_after_for_loop() {
    let diagnostics = typecheck_source(
        r#"
        fn sum() -> number {
            let s: number = 0;
            for (let i: number = 0; i < 10; i = i + 1) {
                s = s + i;
            }
            return s;
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

// ========== Complex Control Flow ==========

#[test]
fn test_if_else_with_early_return() {
    let diagnostics = typecheck_source(
        r#"
        fn complex(x: number, y: number) -> number {
            if (x < 0) {
                return -1;
            }
            if (y < 0) {
                return -2;
            }
            if (x > y) {
                return 1;
            } else {
                return 2;
            }
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

#[test]
fn test_multiple_if_without_final_return() {
    let diagnostics = typecheck_source(
        r#"
        fn test(x: number, y: number) -> number {
            if (x < 0) {
                return -1;
            }
            if (y < 0) {
                return -2;
            }
        }
    "#,
    );
    assert_has_error(&diagnostics, "AT3004"); // no final return
}

#[test]
fn test_nested_loops_with_return() {
    let diagnostics = typecheck_source(
        r#"
        fn test() -> number {
            let i: number = 0;
            while (i < 10) {
                let j: number = 0;
                while (j < 10) {
                    j = j + 1;
                }
                i = i + 1;
            }
            return i;
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

// ========== Return Type Matching ==========

#[test]
fn test_return_number_to_number() {
    let diagnostics = typecheck_source(
        r#"
        fn getNumber() -> number {
            return 42;
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

#[test]
fn test_return_string_to_string() {
    let diagnostics = typecheck_source(
        r#"
        fn getString() -> string {
            return "hello";
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

#[test]
fn test_return_bool_to_bool() {
    let diagnostics = typecheck_source(
        r#"
        fn getBool() -> bool {
            return true;
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

#[test]
fn test_return_array() {
    let diagnostics = typecheck_source(
        r#"
        fn getArray() -> number {
            let arr = [1, 2, 3];
            return arr[0];
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

// ========== Edge Cases ==========

#[test]
fn test_function_returning_number_no_body_error() {
    // Even an empty function needs to return if return type is non-void
    let diagnostics = typecheck_source(
        r#"
        fn getNumber() -> number {
        }
    "#,
    );
    assert_has_error(&diagnostics, "AT3004");
}

#[test]
fn test_function_with_only_declaration() {
    let diagnostics = typecheck_source(
        r#"
        fn test() -> number {
            let x: number = 42;
        }
    "#,
    );
    assert_has_error(&diagnostics, "AT3004");
}

#[test]
fn test_all_branches_return_same_value() {
    let diagnostics = typecheck_source(
        r#"
        fn alwaysOne() -> number {
            if (true) {
                return 1;
            } else {
                return 1;
            }
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

#[test]
fn test_if_else_if_else_all_return() {
    let diagnostics = typecheck_source(
        r#"
        fn classify(x: number) -> number {
            if (x < 0) {
                return -1;
            } else {
                if (x == 0) {
                    return 0;
                } else {
                    return 1;
                }
            }
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

#[test]
fn test_simple_return_without_nesting() {
    // Direct return statement works
    let diagnostics = typecheck_source(
        r#"
        fn test() -> number {
            return 42;
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

#[test]
fn test_return_after_if_without_else() {
    let diagnostics = typecheck_source(
        r#"
        fn max(a: number, b: number) -> number {
            if (a > b) {
                return a;
            }
            return b;
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

// ========== Multiple Functions ==========

#[test]
fn test_multiple_functions_all_valid() {
    let diagnostics = typecheck_source(
        r#"
        fn add(a: number, b: number) -> number {
            return a + b;
        }

        fn multiply(a: number, b: number) -> number {
            return a * b;
        }

        fn greet() -> string {
            return "Hello";
        }
    "#,
    );
    assert_no_errors(&diagnostics);
}

#[test]
fn test_multiple_functions_one_invalid() {
    let diagnostics = typecheck_source(
        r#"
        fn add(a: number, b: number) -> number {
            return a + b;
        }

        fn broken() -> number {
            let x: number = 42;
        }

        fn greet() -> string {
            return "Hello";
        }
    "#,
    );
    assert_has_error(&diagnostics, "AT3004"); // broken() doesn't return
}
