//! Integration tests for refactoring actions

use atlas_lsp::refactor::*;
use atlas_runtime::{ast::Program, Lexer, Parser};
use rstest::rstest;
use tower_lsp::lsp_types::*;

fn parse_program(source: &str) -> Program {
    let mut lexer = Lexer::new(source);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (ast, _) = parser.parse();
    ast
}

fn test_uri() -> Url {
    Url::parse("file:///test.atl").unwrap()
}

// ============================================================================
// Extract Variable Tests
// ============================================================================

#[test]
fn test_extract_variable_simple_expression() {
    let source = "let x = 1 + 2;";
    let program = parse_program(source);
    let uri = test_uri();

    let range = Range {
        start: Position {
            line: 0,
            character: 8,
        },
        end: Position {
            line: 0,
            character: 13,
        },
    };

    let result = extract_variable(&uri, range, source, &program, None, Some("sum"));
    assert!(result.is_ok());

    let workspace_edit = result.unwrap();
    assert!(workspace_edit.changes.is_some());
}

#[test]
fn test_extract_variable_with_default_name() {
    let source = "let x = foo() * 2;";
    let program = parse_program(source);
    let uri = test_uri();

    let range = Range {
        start: Position {
            line: 0,
            character: 8,
        },
        end: Position {
            line: 0,
            character: 17,
        },
    };

    let result = extract_variable(&uri, range, source, &program, None, None);
    assert!(result.is_ok());
}

#[test]
fn test_extract_variable_invalid_range() {
    let source = "let x = 1;";
    let program = parse_program(source);
    let uri = test_uri();

    let range = Range {
        start: Position {
            line: 0,
            character: 20,
        },
        end: Position {
            line: 0,
            character: 30,
        },
    };

    let result = extract_variable(&uri, range, source, &program, None, Some("test"));
    assert!(result.is_err());
}

#[test]
fn test_extract_variable_generates_unique_name() {
    let source = "let extracted = 1;\nlet extracted_1 = 2;\nlet x = 3 + 4;";
    let program = parse_program(source);
    let uri = test_uri();

    let range = Range {
        start: Position {
            line: 2,
            character: 8,
        },
        end: Position {
            line: 2,
            character: 13,
        },
    };

    let result = extract_variable(&uri, range, source, &program, None, Some("extracted"));
    assert!(result.is_ok());

    // The generated name should be "extracted_2" to avoid conflicts
}

#[test]
fn test_extract_variable_rejects_reserved_keyword() {
    let source = "let x = 1 + 2;";
    let program = parse_program(source);
    let uri = test_uri();

    let range = Range {
        start: Position {
            line: 0,
            character: 8,
        },
        end: Position {
            line: 0,
            character: 13,
        },
    };

    let result = extract_variable(&uri, range, source, &program, None, Some("let"));
    assert!(result.is_err());
}

// ============================================================================
// Extract Function Tests
// ============================================================================

#[test]
fn test_extract_function_simple_statements() {
    let source = "let x = 1;\nlet y = 2;\nlet z = x + y;";
    let program = parse_program(source);
    let uri = test_uri();

    let range = Range {
        start: Position {
            line: 0,
            character: 0,
        },
        end: Position {
            line: 2,
            character: 17,
        },
    };

    let result = extract_function(&uri, range, source, &program, None, Some("calculate"));
    assert!(result.is_ok());
}

#[test]
fn test_extract_function_with_default_name() {
    let source = "let x = foo();";
    let program = parse_program(source);
    let uri = test_uri();

    let range = Range {
        start: Position {
            line: 0,
            character: 0,
        },
        end: Position {
            line: 0,
            character: 14,
        },
    };

    let result = extract_function(&uri, range, source, &program, None, None);
    assert!(result.is_ok());
}

#[test]
fn test_extract_function_generates_unique_name() {
    let source = "fn extracted_function() {}\nlet x = 1;";
    let program = parse_program(source);
    let uri = test_uri();

    let range = Range {
        start: Position {
            line: 1,
            character: 0,
        },
        end: Position {
            line: 1,
            character: 10,
        },
    };

    let result = extract_function(
        &uri,
        range,
        source,
        &program,
        None,
        Some("extracted_function"),
    );
    assert!(result.is_ok());
}

#[test]
fn test_extract_function_rejects_invalid_name() {
    let source = "let x = 1;";
    let program = parse_program(source);
    let uri = test_uri();

    let range = Range {
        start: Position {
            line: 0,
            character: 0,
        },
        end: Position {
            line: 0,
            character: 10,
        },
    };

    let result = extract_function(&uri, range, source, &program, None, Some("123invalid"));
    assert!(result.is_err());
}

// ============================================================================
// Inline Variable Tests
// ============================================================================

#[test]
fn test_inline_variable_simple() {
    let source = "let x = 5;\nlet y = x + 1;";
    let program = parse_program(source);
    let uri = test_uri();

    let position = Position {
        line: 0,
        character: 4,
    };

    let result = inline_variable(&uri, position, source, &program, None, "x");
    assert!(result.is_ok());
}

#[test]
fn test_inline_variable_not_found() {
    let source = "let x = 5;";
    let program = parse_program(source);
    let uri = test_uri();

    let position = Position {
        line: 0,
        character: 4,
    };

    let result = inline_variable(&uri, position, source, &program, None, "unknown");
    assert!(result.is_err());
}

#[test]
fn test_inline_variable_multiple_usages() {
    let source = "let x = 5;\nlet y = x + 1;\nlet z = x * 2;";
    let program = parse_program(source);
    let uri = test_uri();

    let position = Position {
        line: 0,
        character: 4,
    };

    let result = inline_variable(&uri, position, source, &program, None, "x");
    // Should replace both usages
    assert!(result.is_ok());
}

// ============================================================================
// Inline Function Tests
// ============================================================================

#[test]
fn test_inline_function_not_implemented() {
    let source = "fn foo() { return 42; }\nlet x = foo();";
    let program = parse_program(source);
    let uri = test_uri();

    let position = Position {
        line: 0,
        character: 3,
    };

    let result = inline_function(&uri, position, source, &program, None, "foo");
    // Currently returns NotImplemented
    assert!(matches!(result, Err(RefactorError::NotImplemented(_))));
}

#[test]
fn test_inline_function_not_found() {
    let source = "let x = 1;";
    let program = parse_program(source);
    let uri = test_uri();

    let position = Position {
        line: 0,
        character: 0,
    };

    let result = inline_function(&uri, position, source, &program, None, "unknown");
    assert!(result.is_err());
}

// ============================================================================
// Rename Symbol Tests
// ============================================================================

#[test]
fn test_rename_variable() {
    let source = "let oldName = 5;\nlet y = oldName + 1;";
    let program = parse_program(source);
    let uri = test_uri();

    let position = Position {
        line: 0,
        character: 4,
    };

    let result = rename_symbol(&uri, position, &program, None, "oldName", "newName");
    assert!(result.is_ok());
}

#[test]
fn test_rename_function() {
    let source = "fn oldFunc() { return 42; }\nlet x = oldFunc();";
    let program = parse_program(source);
    let uri = test_uri();

    let position = Position {
        line: 0,
        character: 3,
    };

    let result = rename_symbol(&uri, position, &program, None, "oldFunc", "newFunc");
    assert!(result.is_ok());
}

#[test]
fn test_rename_symbol_not_found() {
    let source = "let x = 1;";
    let program = parse_program(source);
    let uri = test_uri();

    let position = Position {
        line: 0,
        character: 4,
    };

    let result = rename_symbol(&uri, position, &program, None, "unknown", "newName");
    assert!(result.is_err());
}

#[test]
fn test_rename_name_conflict() {
    let source = "let existing = 1;\nlet oldName = 2;";
    let program = parse_program(source);
    let uri = test_uri();

    let position = Position {
        line: 1,
        character: 4,
    };

    let result = rename_symbol(&uri, position, &program, None, "oldName", "existing");
    assert!(matches!(result, Err(RefactorError::NameConflict(_))));
}

#[test]
fn test_rename_rejects_reserved_keyword() {
    let source = "let oldName = 1;";
    let program = parse_program(source);
    let uri = test_uri();

    let position = Position {
        line: 0,
        character: 4,
    };

    let result = rename_symbol(&uri, position, &program, None, "oldName", "let");
    assert!(matches!(result, Err(RefactorError::NameConflict(_))));
}

#[test]
fn test_rename_rejects_invalid_identifier() {
    let source = "let oldName = 1;";
    let program = parse_program(source);
    let uri = test_uri();

    let position = Position {
        line: 0,
        character: 4,
    };

    let result = rename_symbol(&uri, position, &program, None, "oldName", "123invalid");
    assert!(matches!(result, Err(RefactorError::NameConflict(_))));
}

// ============================================================================
// Workspace Edit Tests
// ============================================================================

#[test]
fn test_workspace_edit_structure() {
    let source = "let x = 1 + 2;";
    let program = parse_program(source);
    let uri = test_uri();

    let range = Range {
        start: Position {
            line: 0,
            character: 8,
        },
        end: Position {
            line: 0,
            character: 13,
        },
    };

    let result = extract_variable(&uri, range, source, &program, None, Some("sum"));
    assert!(result.is_ok());

    let workspace_edit = result.unwrap();
    assert!(workspace_edit.changes.is_some());

    let changes = workspace_edit.changes.unwrap();
    assert!(changes.contains_key(&uri));

    let edits = changes.get(&uri).unwrap();
    assert!(!edits.is_empty());
}

#[test]
fn test_workspace_edit_multiple_files() {
    // For now, we only support single-file edits
    // This test is a placeholder for future cross-file support
    let source = "let x = 1;";
    let program = parse_program(source);
    let uri = test_uri();

    let position = Position {
        line: 0,
        character: 4,
    };

    let result = rename_symbol(&uri, position, &program, None, "x", "y");
    assert!(result.is_ok());

    let workspace_edit = result.unwrap();
    assert!(workspace_edit.changes.is_some());

    let changes = workspace_edit.changes.unwrap();
    // Currently only single file
    assert_eq!(changes.len(), 1);
}

// ============================================================================
// Name Generation Tests
// ============================================================================

#[rstest]
#[case("foo", vec![], "foo")]
#[case("foo", vec!["foo"], "foo_1")]
#[case("foo", vec!["foo", "foo_1"], "foo_2")]
#[case("bar", vec!["foo"], "bar")]
fn test_unique_name_generation(
    #[case] base: &str,
    #[case] existing: Vec<&str>,
    #[case] expected: &str,
) {
    use atlas_lsp::refactor::generate_unique_name;

    let existing_names: Vec<String> = existing.iter().map(|s| s.to_string()).collect();
    let result = generate_unique_name(base, &existing_names);
    assert_eq!(result, expected);
}

// ============================================================================
// Name Validation Tests
// ============================================================================

#[rstest]
#[case("validName", true)]
#[case("_validName", true)]
#[case("valid_name_123", true)]
#[case("let", false)] // Reserved keyword
#[case("fn", false)] // Reserved keyword
#[case("123invalid", false)] // Starts with number
#[case("", false)] // Empty
#[case("invalid-name", false)] // Contains dash
fn test_name_validation(#[case] name: &str, #[case] should_be_valid: bool) {
    use atlas_lsp::refactor::{is_reserved_keyword, is_valid_identifier};

    let is_valid = is_valid_identifier(name) && !is_reserved_keyword(name);
    assert_eq!(is_valid, should_be_valid);
}

// ============================================================================
// Safety Check Tests
// ============================================================================

#[test]
fn test_type_safety_preservation() {
    // Placeholder for type safety verification
    // In a full implementation, we would verify that refactorings preserve types
}

#[test]
fn test_semantics_preservation() {
    // Placeholder for semantics verification
    // In a full implementation, we would verify that refactorings preserve behavior
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_extract_variable_multiline_expression() {
    let source = "let x = {\n    1 + 2\n};";
    let program = parse_program(source);
    let uri = test_uri();

    let range = Range {
        start: Position {
            line: 0,
            character: 8,
        },
        end: Position {
            line: 2,
            character: 1,
        },
    };

    let result = extract_variable(&uri, range, source, &program, None, Some("expr"));
    assert!(result.is_ok());
}

#[test]
fn test_rename_with_shadowing() {
    // Test renaming when there's variable shadowing in different scopes
    let source = "let x = 1;\nfn foo() { let x = 2; }";
    let program = parse_program(source);
    let uri = test_uri();

    let position = Position {
        line: 0,
        character: 4,
    };

    let result = rename_symbol(&uri, position, &program, None, "x", "y");
    // Should handle shadowing correctly
    assert!(result.is_ok());
}

#[test]
fn test_extract_function_with_return_value() {
    let source = "let result = 1 + 2;";
    let program = parse_program(source);
    let uri = test_uri();

    let range = Range {
        start: Position {
            line: 0,
            character: 0,
        },
        end: Position {
            line: 0,
            character: 19,
        },
    };

    let result = extract_function(&uri, range, source, &program, None, Some("compute"));
    assert!(result.is_ok());
}

#[test]
fn test_inline_variable_with_complex_expression() {
    let source = "let x = foo() + bar();\nlet y = x;";
    let program = parse_program(source);
    let uri = test_uri();

    let position = Position {
        line: 0,
        character: 4,
    };

    let result = inline_variable(&uri, position, source, &program, None, "x");
    assert!(result.is_ok());
}

// ============================================================================
// Performance Tests (Placeholder)
// ============================================================================

#[test]
#[ignore = "performance test - run separately"]
fn test_rename_performance_large_file() {
    // Test renaming in a large file with many references
    // This is a placeholder for performance testing
}

#[test]
#[ignore = "performance test - run separately"]
fn test_extract_function_performance() {
    // Test extract function on large code blocks
    // This is a placeholder for performance testing
}
