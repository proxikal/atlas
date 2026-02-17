//! Semantic analysis tests for for-in loops (Phase-20b)
//!
//! Tests that for-in loops bind correctly and type-check properly.

use atlas_runtime::{Binder, Lexer, Parser, TypeChecker};

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
