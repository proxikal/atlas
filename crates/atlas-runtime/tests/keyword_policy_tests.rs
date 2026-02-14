//! Modern Keyword Policy Tests
//!
//! Validates that reserved keywords cannot be used as identifiers.
//! Converted from keyword_policy_tests.rs (316 lines → ~150 lines = 53% reduction)
//!
//! Tests:
//! - Reserved keywords (import, match) cannot be used as identifiers
//! - Active keywords cannot be used as identifiers
//! - Valid keyword usage is allowed

mod common;

use atlas_runtime::lexer::Lexer;
use atlas_runtime::parser::Parser;
use rstest::rstest;

// ============================================================================
// Test Helpers
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

fn assert_has_parser_error(diagnostics: &[atlas_runtime::diagnostic::Diagnostic]) {
    assert!(!diagnostics.is_empty(), "Expected at least one diagnostic");
    let found = diagnostics.iter().any(|d| d.code == "AT1000");
    assert!(
        found,
        "Expected diagnostic with code 'AT1000', got: {:?}",
        diagnostics
            .iter()
            .map(|d| (&d.code, &d.message))
            .collect::<Vec<_>>()
    );
}

fn assert_error_mentions(diagnostics: &[atlas_runtime::diagnostic::Diagnostic], keywords: &[&str]) {
    assert!(
        diagnostics.iter().any(|d| {
            let msg_lower = d.message.to_lowercase();
            keywords.iter().any(|kw| msg_lower.contains(kw))
        }),
        "Expected error message to mention one of {:?}, got: {:?}",
        keywords,
        diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

// ============================================================================
// Reserved Future Keywords - Cannot be used as identifiers
// ============================================================================

#[rstest]
#[case("let import = 1;", &["variable", "identifier"])]
#[case("let match = 1;", &["variable", "identifier"])]
#[case("var import = 1;", &["variable", "identifier"])]
#[case("var match = 1;", &["variable", "identifier"])]
fn test_future_keywords_as_variables(#[case] source: &str, #[case] expected_mentions: &[&str]) {
    let (_program, diagnostics) = parse_source(source);
    assert_has_parser_error(&diagnostics);
    assert_error_mentions(&diagnostics, expected_mentions);
}

#[rstest]
#[case("fn import() { }", &["function", "identifier"])]
#[case("fn match() { }", &["function", "identifier"])]
fn test_future_keywords_as_function_names(
    #[case] source: &str,
    #[case] expected_mentions: &[&str],
) {
    let (_program, diagnostics) = parse_source(source);
    assert_has_parser_error(&diagnostics);
    assert_error_mentions(&diagnostics, expected_mentions);
}

#[rstest]
#[case("fn foo(import: number) { }", &["parameter", "identifier"])]
#[case("fn foo(match: number) { }", &["parameter", "identifier"])]
fn test_future_keywords_as_parameters(#[case] source: &str, #[case] expected_mentions: &[&str]) {
    let (_program, diagnostics) = parse_source(source);
    assert_has_parser_error(&diagnostics);
    assert_error_mentions(&diagnostics, expected_mentions);
}

// ============================================================================
// Active Keywords - Cannot be used as identifiers
// ============================================================================

#[rstest]
#[case("var let = 1;")]
#[case("let fn = 1;")]
#[case("let if = 1;")]
#[case("let while = 1;")]
#[case("let return = 1;")]
#[case("let true = 1;")]
#[case("let false = 1;")]
#[case("let null = 1;")]
fn test_active_keywords_as_identifiers(#[case] source: &str) {
    let (_program, diagnostics) = parse_source(source);
    assert!(
        !diagnostics.is_empty(),
        "Expected error for using active keyword as identifier"
    );
    assert_has_parser_error(&diagnostics);
}

// ============================================================================
// Future Feature Keywords - Statements not supported (v0.1)
// Note: Imports ARE supported as of v0.2 (BLOCKER 04-A)
// ============================================================================

// Import statements now supported - removed outdated tests
// See module_syntax_tests.rs for valid import syntax tests

#[rstest]
#[case("match x { 1 => 2 }", "match")]
fn test_match_expressions_not_supported(#[case] source: &str, #[case] keyword: &str) {
    let (_program, diagnostics) = parse_source(source);
    assert!(
        !diagnostics.is_empty(),
        "Expected error for '{}' expression",
        keyword
    );
    // Should have some error since match is not supported
}

// ============================================================================
// Valid Keyword Usage
// ============================================================================

#[rstest]
#[case("let x = 1;")]
#[case("fn foo() { }")]
#[case("if (true) { }")]
#[case("while (false) { }")]
#[case("return 42;")]
fn test_valid_keyword_usage(#[case] source: &str) {
    let (_program, diagnostics) = parse_source(source);
    // These should parse without errors (though return outside function might have semantic errors)
    // At parser level, these are valid
    let has_parser_error = diagnostics.iter().any(|d| d.code == "AT1000");
    assert!(
        !has_parser_error,
        "Should not have parser errors for valid keyword usage: {:?}",
        diagnostics
    );
}

// ============================================================================
// Edge Cases - Keywords in valid contexts
// ============================================================================

#[test]
fn test_keywords_in_strings_allowed() {
    let source = r#"let x = "import match let fn";"#;
    let (_program, diagnostics) = parse_source(source);

    // Keywords in strings are fine
    let has_parser_error = diagnostics.iter().any(|d| d.code == "AT1000");
    assert!(!has_parser_error, "Keywords in strings should be allowed");
}

#[test]
fn test_keywords_in_comments_allowed() {
    let source = "// import match let\nlet x = 1;";
    let (_program, diagnostics) = parse_source(source);

    // Keywords in comments are fine
    let has_parser_error = diagnostics.iter().any(|d| d.code == "AT1000");
    assert!(!has_parser_error, "Keywords in comments should be allowed");
}

// ============================================================================
// Error Message Quality Tests
// ============================================================================

#[test]
fn test_error_message_mentions_keyword_and_reserved() {
    let (_program, diagnostics) = parse_source("let import = 1;");

    assert!(!diagnostics.is_empty(), "Expected error");
    assert_has_parser_error(&diagnostics);

    // Error message should mention 'import' keyword and that it's reserved
    assert!(
        diagnostics
            .iter()
            .any(|d| d.message.contains("import") && d.message.contains("reserved")),
        "Expected error message to mention 'import' as reserved keyword, got: {:?}",
        diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

#[test]
fn test_error_message_for_future_keyword_mentions_future() {
    let (_program, diagnostics) = parse_source("fn match() {}");

    assert!(!diagnostics.is_empty(), "Expected error");
    assert_has_parser_error(&diagnostics);

    // Error message should mention it's reserved for future use
    assert!(
        diagnostics
            .iter()
            .any(|d| d.message.contains("match") && d.message.contains("future")),
        "Expected error message to mention 'match' is reserved for future use, got: {:?}",
        diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

// Import syntax tests moved to module_syntax_tests.rs (BLOCKER 04-A)
// Imports are now fully supported as of v0.2

// ============================================================================
// Contextual Tests
// ============================================================================

#[test]
fn test_keyword_as_identifier_in_expression() {
    // Trying to reference 'import' as if it were a variable
    let (_program, diagnostics) = parse_source("let x = import;");

    assert!(
        !diagnostics.is_empty(),
        "Expected error for using keyword as expression"
    );
    assert_has_parser_error(&diagnostics);
}

#[test]
fn test_multiple_keyword_errors() {
    let (_program, diagnostics) = parse_source(
        r#"
        let import = 1;
        let match = 2;
    "#,
    );

    // Should have at least 2 errors (one for each invalid use)
    assert!(diagnostics.len() >= 2, "Expected at least 2 errors");

    // All should be AT1000 syntax errors
    let at1000_count = diagnostics.iter().filter(|d| d.code == "AT1000").count();
    assert!(
        at1000_count >= 2,
        "Expected at least 2 AT1000 errors, got {}",
        at1000_count
    );
}

// ============================================================================
// Additional Valid Uses
// ============================================================================

#[test]
fn test_valid_use_of_boolean_and_null_literals() {
    let source = "let x = true; let y = false; let z = null;";
    let (_program, diagnostics) = parse_source(source);

    assert_eq!(
        diagnostics.len(),
        0,
        "Expected no errors for valid use of boolean/null literals"
    );
}
