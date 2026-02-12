//! Keyword Policy Tests (Phase 09)
//!
//! These tests validate that reserved keywords cannot be used as identifiers
//! and that reserved keywords for future features (import, match) are properly rejected.

use atlas_runtime::lexer::Lexer;
use atlas_runtime::parser::Parser;

fn parse_source(source: &str) -> (atlas_runtime::ast::Program, Vec<atlas_runtime::diagnostic::Diagnostic>) {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    parser.parse()
}

fn assert_has_error_with_code(diagnostics: &[atlas_runtime::diagnostic::Diagnostic], code: &str) {
    assert!(!diagnostics.is_empty(), "Expected at least one diagnostic");
    let found = diagnostics.iter().any(|d| d.code == code);
    assert!(found, "Expected diagnostic with code '{}', got: {:?}",
        code, diagnostics.iter().map(|d| (&d.code, &d.message)).collect::<Vec<_>>());
}

// ========== Reserved Keywords as Variable Names ==========

#[test]
fn test_keyword_import_as_let_variable() {
    let (_program, diagnostics) = parse_source("let import = 1;");

    // Should get AT1000 parser error because 'import' is a keyword, not an identifier
    assert_has_error_with_code(&diagnostics, "AT1000");

    // Verify error message mentions identifier or variable name
    assert!(diagnostics.iter().any(|d|
        d.message.to_lowercase().contains("variable") ||
        d.message.to_lowercase().contains("identifier")
    ), "Expected error about variable name or identifier");
}

#[test]
fn test_keyword_match_as_let_variable() {
    let (_program, diagnostics) = parse_source("let match = 1;");

    assert_has_error_with_code(&diagnostics, "AT1000");
    assert!(diagnostics.iter().any(|d|
        d.message.to_lowercase().contains("variable") ||
        d.message.to_lowercase().contains("identifier")
    ));
}

#[test]
fn test_keyword_import_as_var_variable() {
    let (_program, diagnostics) = parse_source("var import = 1;");

    assert_has_error_with_code(&diagnostics, "AT1000");
    assert!(diagnostics.iter().any(|d|
        d.message.to_lowercase().contains("variable") ||
        d.message.to_lowercase().contains("identifier")
    ));
}

#[test]
fn test_keyword_match_as_var_variable() {
    let (_program, diagnostics) = parse_source("var match = 1;");

    assert_has_error_with_code(&diagnostics, "AT1000");
    assert!(diagnostics.iter().any(|d|
        d.message.to_lowercase().contains("variable") ||
        d.message.to_lowercase().contains("identifier")
    ));
}

// ========== Reserved Keywords as Function Names ==========

#[test]
fn test_keyword_import_as_function_name() {
    let (_program, diagnostics) = parse_source("fn import() { }");

    assert_has_error_with_code(&diagnostics, "AT1000");
    assert!(diagnostics.iter().any(|d|
        d.message.to_lowercase().contains("function") ||
        d.message.to_lowercase().contains("identifier")
    ));
}

#[test]
fn test_keyword_match_as_function_name() {
    let (_program, diagnostics) = parse_source("fn match() { }");

    assert_has_error_with_code(&diagnostics, "AT1000");
    assert!(diagnostics.iter().any(|d|
        d.message.to_lowercase().contains("function") ||
        d.message.to_lowercase().contains("identifier")
    ));
}

// ========== Reserved Keywords as Function Parameters ==========

#[test]
fn test_keyword_import_as_parameter() {
    let (_program, diagnostics) = parse_source("fn foo(import: number) { }");

    assert_has_error_with_code(&diagnostics, "AT1000");
    assert!(diagnostics.iter().any(|d|
        d.message.to_lowercase().contains("parameter") ||
        d.message.to_lowercase().contains("identifier")
    ));
}

#[test]
fn test_keyword_match_as_parameter() {
    let (_program, diagnostics) = parse_source("fn foo(match: number) { }");

    assert_has_error_with_code(&diagnostics, "AT1000");
    assert!(diagnostics.iter().any(|d|
        d.message.to_lowercase().contains("parameter") ||
        d.message.to_lowercase().contains("identifier")
    ));
}

// ========== Other Active Keywords Cannot Be Identifiers ==========

#[test]
fn test_keyword_let_as_variable_name() {
    let (_program, diagnostics) = parse_source("var let = 1;");

    assert!(!diagnostics.is_empty(), "Expected error for using 'let' as identifier");
    assert_has_error_with_code(&diagnostics, "AT1000");
}

#[test]
fn test_keyword_fn_as_variable_name() {
    let (_program, diagnostics) = parse_source("let fn = 1;");

    assert!(!diagnostics.is_empty(), "Expected error for using 'fn' as identifier");
    assert_has_error_with_code(&diagnostics, "AT1000");
}

#[test]
fn test_keyword_if_as_variable_name() {
    let (_program, diagnostics) = parse_source("let if = 1;");

    assert!(!diagnostics.is_empty(), "Expected error for using 'if' as identifier");
    assert_has_error_with_code(&diagnostics, "AT1000");
}

#[test]
fn test_keyword_while_as_variable_name() {
    let (_program, diagnostics) = parse_source("let while = 1;");

    assert!(!diagnostics.is_empty(), "Expected error for using 'while' as identifier");
    assert_has_error_with_code(&diagnostics, "AT1000");
}

#[test]
fn test_keyword_return_as_variable_name() {
    let (_program, diagnostics) = parse_source("let return = 1;");

    assert!(!diagnostics.is_empty(), "Expected error for using 'return' as identifier");
    assert_has_error_with_code(&diagnostics, "AT1000");
}

// ========== Boolean and Null Literals Cannot Be Identifiers ==========

#[test]
fn test_keyword_true_as_variable_name() {
    let (_program, diagnostics) = parse_source("let true = 1;");

    assert!(!diagnostics.is_empty(), "Expected error for using 'true' as identifier");
    assert_has_error_with_code(&diagnostics, "AT1000");
}

#[test]
fn test_keyword_false_as_variable_name() {
    let (_program, diagnostics) = parse_source("let false = 1;");

    assert!(!diagnostics.is_empty(), "Expected error for using 'false' as identifier");
    assert_has_error_with_code(&diagnostics, "AT1000");
}

#[test]
fn test_keyword_null_as_variable_name() {
    let (_program, diagnostics) = parse_source("let null = 1;");

    assert!(!diagnostics.is_empty(), "Expected error for using 'null' as identifier");
    assert_has_error_with_code(&diagnostics, "AT1000");
}

// ========== Import Statement Not Supported ==========

#[test]
fn test_import_statement_not_supported() {
    let (_program, diagnostics) = parse_source("import math;");

    // Import keyword at statement level should produce error
    // The parser doesn't have import statement support, so it will fail
    assert!(!diagnostics.is_empty(), "Expected error for unsupported import statement");
    // Should produce AT1000 as a syntax error
    assert_has_error_with_code(&diagnostics, "AT1000");
}

#[test]
fn test_import_with_module_path() {
    let (_program, diagnostics) = parse_source("import std.math;");

    assert!(!diagnostics.is_empty(), "Expected error for unsupported import statement");
    assert_has_error_with_code(&diagnostics, "AT1000");
}

// ========== Match Expression Not Supported ==========

#[test]
fn test_match_expression_not_supported() {
    // Since match syntax is not defined in v0.1, trying to use it should fail
    let (_program, diagnostics) = parse_source("let x = match;");

    assert!(!diagnostics.is_empty(), "Expected error for using 'match' keyword");
    assert_has_error_with_code(&diagnostics, "AT1000");
}

// ========== Valid Uses of Keywords (Sanity Checks) ==========

#[test]
fn test_valid_use_of_let_keyword() {
    let (_program, diagnostics) = parse_source("let x = 1;");

    // Should have no errors - this is valid use of 'let'
    assert_eq!(diagnostics.len(), 0, "Expected no errors for valid let statement");
}

#[test]
fn test_valid_use_of_if_keyword() {
    let (_program, diagnostics) = parse_source("if (true) { let x = 1; }");

    assert_eq!(diagnostics.len(), 0, "Expected no errors for valid if statement");
}

#[test]
fn test_valid_use_of_true_false_null() {
    let (_program, diagnostics) = parse_source("let x = true; let y = false; let z = null;");

    assert_eq!(diagnostics.len(), 0, "Expected no errors for valid use of boolean/null literals");
}

// ========== Enhanced Error Message Tests ==========

#[test]
fn test_error_message_mentions_keyword() {
    // Verify error messages specifically mention the keyword that was misused
    let (_program, diagnostics) = parse_source("let import = 1;");

    assert!(!diagnostics.is_empty(), "Expected error");
    assert_has_error_with_code(&diagnostics, "AT1000");

    // Error message should mention 'import' keyword
    assert!(diagnostics.iter().any(|d|
        d.message.contains("import") && d.message.contains("reserved")
    ), "Expected error message to mention 'import' as reserved keyword, got: {:?}",
        diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>());
}

#[test]
fn test_error_message_for_future_keyword() {
    // import/match should mention they're reserved for future use
    let (_program, diagnostics) = parse_source("fn match() {}");

    assert!(!diagnostics.is_empty(), "Expected error");
    assert_has_error_with_code(&diagnostics, "AT1000");

    // Error message should mention it's reserved for future use
    assert!(diagnostics.iter().any(|d|
        d.message.contains("match") && d.message.contains("future")
    ), "Expected error message to mention 'match' is reserved for future use, got: {:?}",
        diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>());
}

#[test]
fn test_import_statement_error_message() {
    // import statement should mention it's not supported in v0.1
    let (_program, diagnostics) = parse_source("import foo;");

    assert!(!diagnostics.is_empty(), "Expected error");
    assert_has_error_with_code(&diagnostics, "AT1000");

    // Error message should mention import is not supported
    assert!(diagnostics.iter().any(|d|
        d.message.to_lowercase().contains("import") &&
        (d.message.contains("not supported") || d.message.contains("v0.1"))
    ), "Expected error message to mention import is not supported, got: {:?}",
        diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>());
}

// ========== Contextual Tests ==========

#[test]
fn test_keyword_as_identifier_in_expression() {
    // Trying to reference 'import' as if it were a variable
    let (_program, diagnostics) = parse_source("let x = import;");

    assert!(!diagnostics.is_empty(), "Expected error for using keyword as expression");
    assert_has_error_with_code(&diagnostics, "AT1000");
}

#[test]
fn test_multiple_keyword_errors() {
    let (_program, diagnostics) = parse_source(r#"
        let import = 1;
        let match = 2;
    "#);

    // Should have at least 2 errors (one for each invalid use)
    assert!(diagnostics.len() >= 2, "Expected at least 2 errors");

    // All should be AT1000 syntax errors
    let at1000_count = diagnostics.iter().filter(|d| d.code == "AT1000").count();
    assert!(at1000_count >= 2, "Expected at least 2 AT1000 errors");
}
