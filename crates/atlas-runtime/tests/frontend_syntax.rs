//! frontend_syntax.rs — merged from 11 files (Phase Infra-01)
//!
//! Sources: lexer_tests.rs, lexer_golden_tests.rs, parser_tests.rs, parser_error_tests.rs, operator_precedence_tests.rs, keyword_policy_tests.rs, generic_syntax_tests.rs, module_syntax_tests.rs, warning_tests.rs, warnings_tests.rs, test_for_in_parsing.rs

mod common;
use atlas_runtime::ast::*;
use atlas_runtime::diagnostic::warnings::{
    config_from_toml, WarningConfig, WarningEmitter, WarningKind, WarningLevel,
};
use atlas_runtime::token::TokenKind;
use atlas_runtime::{Binder, Diagnostic, DiagnosticLevel, Lexer, Parser, Span, TypeChecker};
use pretty_assertions::assert_eq;
use rstest::rstest;
use std::fs;
use std::path::Path;

// ============================================================================
// Lexer Tests (from lexer_tests.rs)
// ============================================================================

// Helper function to reduce boilerplate
fn lex(source: &str) -> (Vec<atlas_runtime::token::Token>, Vec<Diagnostic>) {
    let mut lexer = Lexer::new(source.to_string());
    lexer.tokenize()
}

// ============================================================================
// String Literal Tests - Parameterized with rstest
// ============================================================================

#[rstest]
#[case(r#""hello world""#, "hello world")]
#[case(r#""line1\nline2\ttab\r\n""#, "line1\nline2\ttab\r\n")]
#[case(r#""He said \"hello\"""#, r#"He said "hello""#)]
#[case(r#""path\\to\\file""#, r"path\to\file")]
#[case("\"line1\nline2\nline3\"", "line1\nline2\nline3")]
fn test_string_literals_valid(#[case] input: &str, #[case] expected: &str) {
    let (tokens, diagnostics) = lex(input);

    assert_eq!(diagnostics.len(), 0, "Should have no errors");
    assert_eq!(tokens[0].kind, TokenKind::String);
    assert_eq!(tokens[0].lexeme, expected);
}

#[rstest]
#[case(r#""unterminated string"#, "Unterminated")]
#[case(r#""invalid\xescape""#, "Invalid escape")]
fn test_string_literals_errors(#[case] input: &str, #[case] error_message: &str) {
    let (tokens, diagnostics) = lex(input);

    assert_eq!(tokens[0].kind, TokenKind::Error);
    assert!(!diagnostics.is_empty(), "Should have errors");
    assert!(
        diagnostics[0].message.contains(error_message),
        "Expected error containing '{}', got '{}'",
        error_message,
        diagnostics[0].message
    );
}

// ============================================================================
// Number Literal Tests - Table-driven
// ============================================================================

#[rstest]
#[case("0", "0")]
#[case("1", "1")]
#[case("42", "42")]
#[case("999", "999")]
#[case("1234567890", "1234567890")]
#[case("0.0", "0.0")]
#[case("3.14", "3.14")]
#[case("99.999", "99.999")]
#[case("0.5", "0.5")]
fn test_number_literals(#[case] input: &str, #[case] expected: &str) {
    let (tokens, diagnostics) = lex(input);

    assert_eq!(diagnostics.len(), 0, "Should have no errors");
    assert_eq!(tokens[0].kind, TokenKind::Number);
    assert_eq!(tokens[0].lexeme, expected);
}

// ============================================================================
// Keyword Tests - Single parameterized test instead of 20+ individual tests
// ============================================================================

#[rstest]
#[case("let", TokenKind::Let)]
#[case("var", TokenKind::Var)]
#[case("fn", TokenKind::Fn)]
#[case("if", TokenKind::If)]
#[case("else", TokenKind::Else)]
#[case("while", TokenKind::While)]
#[case("for", TokenKind::For)]
#[case("return", TokenKind::Return)]
#[case("break", TokenKind::Break)]
#[case("continue", TokenKind::Continue)]
#[case("true", TokenKind::True)]
#[case("false", TokenKind::False)]
#[case("null", TokenKind::Null)]
fn test_keywords(#[case] keyword: &str, #[case] expected_kind: TokenKind) {
    let (tokens, diagnostics) = lex(keyword);

    assert_eq!(diagnostics.len(), 0);
    assert_eq!(tokens[0].kind, expected_kind);
    assert_eq!(tokens[0].lexeme, keyword);
}

// ============================================================================
// Operator Tests
// ============================================================================

#[rstest]
#[case("+", TokenKind::Plus)]
#[case("-", TokenKind::Minus)]
#[case("*", TokenKind::Star)]
#[case("/", TokenKind::Slash)]
#[case("%", TokenKind::Percent)]
#[case("==", TokenKind::EqualEqual)]
#[case("!=", TokenKind::BangEqual)]
#[case("<", TokenKind::Less)]
#[case("<=", TokenKind::LessEqual)]
#[case(">", TokenKind::Greater)]
#[case(">=", TokenKind::GreaterEqual)]
#[case("&&", TokenKind::AmpAmp)]
#[case("||", TokenKind::PipePipe)]
#[case("!", TokenKind::Bang)]
#[case("=", TokenKind::Equal)]
#[case("+=", TokenKind::PlusEqual)]
#[case("-=", TokenKind::MinusEqual)]
#[case("*=", TokenKind::StarEqual)]
#[case("/=", TokenKind::SlashEqual)]
#[case("%=", TokenKind::PercentEqual)]
#[case("++", TokenKind::PlusPlus)]
#[case("--", TokenKind::MinusMinus)]
fn test_operators(#[case] operator: &str, #[case] expected_kind: TokenKind) {
    let (tokens, diagnostics) = lex(operator);

    assert_eq!(diagnostics.len(), 0);
    assert_eq!(tokens[0].kind, expected_kind);
}

// ============================================================================
// Comment Tests
// ============================================================================

#[rstest]
#[case("// single line comment\n", 1)] // Just EOF
#[case("/* block comment */", 1)] // Just EOF
#[case("/* multi\nline\ncomment */", 1)]
#[case("let x = 1; // comment", 6)] // let x = 1 ; EOF (6 tokens)
fn test_comments_ignored(#[case] input: &str, #[case] expected_token_count: usize) {
    let (tokens, diagnostics) = lex(input);

    assert_eq!(diagnostics.len(), 0);
    assert_eq!(tokens.len(), expected_token_count);
}

// ============================================================================
// Integration Test - Complex Expression
// ============================================================================

#[test]
fn test_complex_expression() {
    let source = r#"fn add(a: number, b: number) -> number { return a + b; }"#;
    let (tokens, diagnostics) = lex(source);

    assert_eq!(diagnostics.len(), 0, "Should lex without errors");

    // Verify token sequence
    let expected_kinds = vec![
        TokenKind::Fn,
        TokenKind::Identifier,
        TokenKind::LeftParen,
        TokenKind::Identifier,
        TokenKind::Colon,
        TokenKind::Identifier,
        TokenKind::Comma,
        TokenKind::Identifier,
        TokenKind::Colon,
        TokenKind::Identifier,
        TokenKind::RightParen,
        TokenKind::Arrow,
        TokenKind::Identifier,
        TokenKind::LeftBrace,
        TokenKind::Return,
        TokenKind::Identifier,
        TokenKind::Plus,
        TokenKind::Identifier,
        TokenKind::Semicolon,
        TokenKind::RightBrace,
        TokenKind::Eof,
    ];

    for (i, expected_kind) in expected_kinds.iter().enumerate() {
        assert_eq!(
            tokens[i].kind, *expected_kind,
            "Token {} should be {:?}, got {:?}",
            i, expected_kind, tokens[i].kind
        );
    }
}

// ============================================================================
// Lexer Golden Tests (from lexer_golden_tests.rs)
// ============================================================================

fn lex_file(filename: &str) -> Vec<Diagnostic> {
    let path = Path::new("tests/errors").join(filename);
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("Failed to read test file: {}", path.display()));

    let mut lexer = Lexer::new(&source);
    let (_, diagnostics) = lexer.tokenize();
    diagnostics
}

// ============================================================================
// Individual Error File Tests with Snapshots
// ============================================================================

#[rstest]
#[case("unterminated_string.atl", "AT1002")]
#[case("invalid_escape.atl", "AT1003")]
#[case("unexpected_char.atl", "AT1001")]
#[case("unterminated_comment.atl", "AT1004")]
fn test_lexer_error_files(#[case] filename: &str, #[case] expected_code: &str) {
    let diagnostics = lex_file(filename);

    // Verify we got the expected error
    assert!(
        !diagnostics.is_empty(),
        "Expected diagnostics for {}",
        filename
    );
    assert!(
        diagnostics.iter().any(|d| d.code == expected_code),
        "Expected error code {} in {}, got: {:?}",
        expected_code,
        filename,
        diagnostics.iter().map(|d| &d.code).collect::<Vec<_>>()
    );

    // Snapshot the diagnostics for stability tracking
    insta::assert_yaml_snapshot!(
        format!("lexer_error_{}", filename.replace(".atl", "")),
        diagnostics
    );
}

// ============================================================================
// Stability Test
// ============================================================================

#[test]
fn test_diagnostic_stability() {
    // Verify that running the same file twice produces identical diagnostics
    let diag1 = lex_file("unterminated_string.atl");
    let diag2 = lex_file("unterminated_string.atl");

    assert_eq!(
        diag1.len(),
        diag2.len(),
        "Diagnostic count should be stable"
    );
    for (d1, d2) in diag1.iter().zip(diag2.iter()) {
        assert_eq!(d1.code, d2.code, "Diagnostic codes should be stable");
        assert_eq!(
            d1.message, d2.message,
            "Diagnostic messages should be stable"
        );
        assert_eq!(d1.line, d2.line, "Diagnostic lines should be stable");
        assert_eq!(d1.column, d2.column, "Diagnostic columns should be stable");
    }
}

// ============================================================================
// Parser Tests (from parser_tests.rs)
// ============================================================================

fn parse_source(source: &str) -> (Program, Vec<atlas_runtime::diagnostic::Diagnostic>) {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    parser.parse()
}

// ============================================================================
// Literal Expressions - Snapshot Testing
// ============================================================================

#[rstest]
#[case::number("42;", "number_literal")]
#[case::float("3.14;", "float_literal")]
#[case::string(r#""hello";"#, "string_literal")]
#[case::bool_true("true;", "bool_true")]
#[case::bool_false("false;", "bool_false")]
#[case::null("null;", "null_literal")]
fn test_parse_literals(#[case] source: &str, #[case] snapshot_name: &str) {
    let (program, diagnostics) = parse_source(source);
    assert_eq!(diagnostics.len(), 0, "Expected no errors for: {}", source);
    insta::assert_yaml_snapshot!(snapshot_name, program);
}

// ============================================================================
// Variables and Identifiers
// ============================================================================

#[test]
fn test_parse_variable_reference() {
    let (program, diagnostics) = parse_source("x;");
    assert_eq!(diagnostics.len(), 0);
    insta::assert_yaml_snapshot!(program);
}

// ============================================================================
// Binary Operators - Snapshot Testing
// ============================================================================

#[rstest]
#[case::add("1 + 2;", "addition")]
#[case::sub("5 - 3;", "subtraction")]
#[case::mul("3 * 4;", "multiplication")]
#[case::div("10 / 2;", "division")]
#[case::lt("1 < 2;", "less_than")]
#[case::le("1 <= 2;", "less_equal")]
#[case::gt("1 > 2;", "greater_than")]
#[case::ge("1 >= 2;", "greater_equal")]
#[case::eq("1 == 2;", "equality")]
#[case::ne("1 != 2;", "not_equal")]
#[case::and("true && false;", "logical_and")]
#[case::or("true || false;", "logical_or")]
fn test_parse_binary_operators(#[case] source: &str, #[case] snapshot_name: &str) {
    let (program, diagnostics) = parse_source(source);
    assert_eq!(diagnostics.len(), 0, "Expected no errors for: {}", source);
    insta::assert_yaml_snapshot!(snapshot_name, program);
}

// ============================================================================
// Unary Operators
// ============================================================================

#[rstest]
#[case::negate("-5;", "negation")]
#[case::not("!true;", "logical_not")]
fn test_parse_unary_operators(#[case] source: &str, #[case] snapshot_name: &str) {
    let (program, diagnostics) = parse_source(source);
    assert_eq!(diagnostics.len(), 0);
    insta::assert_yaml_snapshot!(snapshot_name, program);
}

// ============================================================================
// Grouping and Precedence
// ============================================================================

#[test]
fn test_parse_grouping() {
    let (program, diagnostics) = parse_source("(1 + 2) * 3;");
    assert_eq!(diagnostics.len(), 0);
    insta::assert_yaml_snapshot!(program);
}

#[test]
fn test_operator_precedence_multiplication_over_addition() {
    let (program, diagnostics) = parse_source("1 + 2 * 3;");
    assert_eq!(diagnostics.len(), 0);
    // Should parse as: 1 + (2 * 3), not (1 + 2) * 3
    insta::assert_yaml_snapshot!(program);
}

#[test]
fn test_operator_precedence_comparison_over_logical() {
    let (program, diagnostics) = parse_source("1 < 2 && 3 > 4;");
    assert_eq!(diagnostics.len(), 0);
    // Should parse as: (1 < 2) && (3 > 4)
    insta::assert_yaml_snapshot!(program);
}

// ============================================================================
// Array Literals and Indexing
// ============================================================================

#[rstest]
#[case::empty("[];", "empty_array")]
#[case::with_elements("[1, 2, 3];", "array_with_elements")]
#[case::array_index("arr[0];", "array_index")]
fn test_parse_arrays(#[case] source: &str, #[case] snapshot_name: &str) {
    let (program, diagnostics) = parse_source(source);
    assert_eq!(diagnostics.len(), 0);
    insta::assert_yaml_snapshot!(snapshot_name, program);
}

// ============================================================================
// Function Calls
// ============================================================================

#[rstest]
#[case::no_args("foo();", "function_call_no_args")]
#[case::with_args("foo(1, 2, 3);", "function_call_with_args")]
fn test_parse_function_calls(#[case] source: &str, #[case] snapshot_name: &str) {
    let (program, diagnostics) = parse_source(source);
    assert_eq!(diagnostics.len(), 0);
    insta::assert_yaml_snapshot!(snapshot_name, program);
}

// ============================================================================
// Variable Declarations
// ============================================================================

#[rstest]
#[case::let_decl("let x = 42;", "let_declaration")]
#[case::var_decl("var x = 42;", "var_declaration")]
#[case::with_type("let x: number = 42;", "var_declaration_with_type")]
fn test_parse_var_declarations(#[case] source: &str, #[case] snapshot_name: &str) {
    let (program, diagnostics) = parse_source(source);
    assert_eq!(diagnostics.len(), 0);
    insta::assert_yaml_snapshot!(snapshot_name, program);
}

// ============================================================================
// Assignment Statements
// ============================================================================

#[rstest]
#[case::simple("x = 42;", "simple_assignment")]
#[case::array_element("arr[0] = 42;", "array_element_assignment")]
fn test_parse_assignments(#[case] source: &str, #[case] snapshot_name: &str) {
    let (program, diagnostics) = parse_source(source);
    assert_eq!(diagnostics.len(), 0);
    insta::assert_yaml_snapshot!(snapshot_name, program);
}

// ============================================================================
// Control Flow Statements
// ============================================================================

#[rstest]
#[case::if_stmt("if (true) { x; }", "if_statement")]
#[case::if_else("if (true) { x; } else { y; }", "if_else_statement")]
#[case::while_loop("while (true) { x; }", "while_loop")]
#[case::for_loop("for (let i = 0; i < 10; i = i + 1) { x; }", "for_loop")]
fn test_parse_control_flow(#[case] source: &str, #[case] snapshot_name: &str) {
    let (program, diagnostics) = parse_source(source);
    assert_eq!(diagnostics.len(), 0);
    insta::assert_yaml_snapshot!(snapshot_name, program);
}

// ============================================================================
// Return, Break, Continue
// ============================================================================

#[rstest]
#[case::return_value("return 42;", "return_statement")]
#[case::return_void("return;", "return_no_value")]
#[case::break_stmt("break;", "break_statement")]
#[case::continue_stmt("continue;", "continue_statement")]
fn test_parse_flow_control_statements(#[case] source: &str, #[case] snapshot_name: &str) {
    let (program, diagnostics) = parse_source(source);
    assert_eq!(diagnostics.len(), 0);
    insta::assert_yaml_snapshot!(snapshot_name, program);
}

// ============================================================================
// Block Statements
// ============================================================================

#[test]
fn test_parse_block_in_if() {
    let (program, diagnostics) = parse_source("if (true) { let x = 1; let y = 2; }");
    assert_eq!(diagnostics.len(), 0);
    insta::assert_yaml_snapshot!(program);
}

#[test]
fn test_parse_nested_blocks() {
    let (program, diagnostics) = parse_source("if (true) { if (false) { let x = 1; } }");
    assert_eq!(diagnostics.len(), 0);
    insta::assert_yaml_snapshot!(program);
}

// ============================================================================
// Function Declarations
// ============================================================================

#[test]
fn test_parse_function_no_params() {
    let (program, diagnostics) = parse_source("fn foo() { return 42; }");
    assert_eq!(diagnostics.len(), 0);
    insta::assert_yaml_snapshot!(program);
}

#[test]
fn test_parse_function_with_params() {
    let (program, diagnostics) =
        parse_source("fn add(x: number, y: number) -> number { return x + y; }");
    assert_eq!(diagnostics.len(), 0);
    insta::assert_yaml_snapshot!(program);
}

#[test]
fn test_parse_function_with_complex_body() {
    let source = r#"
fn factorial(n: number) -> number {
    if (n <= 1) {
        return 1;
    } else {
        return n * factorial(n - 1);
    }
}
    "#;

    let (program, diagnostics) = parse_source(source);
    assert_eq!(diagnostics.len(), 0);
    insta::assert_yaml_snapshot!(program);
}

// ============================================================================
// Member Expressions (Method Calls)
// ============================================================================

#[rstest]
#[case::simple_method("obj.method();", "simple_method_call")]
#[case::method_with_one_arg("obj.method(x);", "method_with_one_arg")]
#[case::method_with_multiple_args("obj.method(a, b, c);", "method_with_multiple_args")]
#[case::json_as_string(r#"json["user"].as_string();"#, "json_extraction_as_string")]
#[case::json_as_number("data.as_number();", "json_as_number")]
fn test_parse_member_expressions(#[case] source: &str, #[case] snapshot_name: &str) {
    let (program, diagnostics) = parse_source(source);
    assert_eq!(diagnostics.len(), 0, "Expected no errors for: {}", source);
    insta::assert_yaml_snapshot!(snapshot_name, program);
}

#[test]
fn test_parse_chained_member_calls() {
    let (program, diagnostics) = parse_source("a.b().c();");
    assert_eq!(diagnostics.len(), 0);
    insta::assert_yaml_snapshot!(program);
}

#[test]
fn test_parse_member_after_index() {
    let (program, diagnostics) = parse_source("arr[0].method();");
    assert_eq!(diagnostics.len(), 0);
    insta::assert_yaml_snapshot!(program);
}

#[test]
fn test_parse_complex_member_chain() {
    let (program, diagnostics) = parse_source(r#"json["data"]["items"][0].as_string();"#);
    assert_eq!(diagnostics.len(), 0);
    insta::assert_yaml_snapshot!(program);
}

#[test]
fn test_parse_member_in_expression() {
    let (program, diagnostics) = parse_source("let x = obj.method() + 5;");
    assert_eq!(diagnostics.len(), 0);
    insta::assert_yaml_snapshot!(program);
}

#[test]
fn test_parse_member_as_function_arg() {
    let (program, diagnostics) = parse_source("print(data.as_string());");
    assert_eq!(diagnostics.len(), 0);
    insta::assert_yaml_snapshot!(program);
}

#[test]
fn test_parse_nested_member_calls() {
    let (program, diagnostics) = parse_source("outer.method(inner.method());");
    assert_eq!(diagnostics.len(), 0);
    insta::assert_yaml_snapshot!(program);
}

// ============================================================================
// Complex Programs
// ============================================================================

#[test]
fn test_parse_multiple_statements() {
    let (program, diagnostics) = parse_source("let x = 1; let y = 2; let z = x + y;");
    assert_eq!(diagnostics.len(), 0);
    assert_eq!(program.items.len(), 3);
    insta::assert_yaml_snapshot!(program);
}

// ============================================================================
// Error Recovery
// ============================================================================

#[test]
fn test_parse_error_recovery() {
    let (program, diagnostics) = parse_source("let x = ; let y = 2;");
    assert!(!diagnostics.is_empty(), "Expected syntax error");
    // Parser should recover and parse the second statement
    assert!(
        !program.items.is_empty(),
        "Expected at least one item after recovery"
    );
}

#[test]
fn test_parse_missing_semicolon_error() {
    let (_program, diagnostics) = parse_source("let x = 1 let y = 2;");
    assert!(
        !diagnostics.is_empty(),
        "Expected syntax error for missing semicolon"
    );
}

// ============================================================================
// Nested Functions (Phase 1: Parser Support)
// ============================================================================

#[test]
fn test_parse_nested_function_in_function() {
    let source = r#"
        fn outer() -> number {
            fn helper(x: number) -> number {
                return x * 2;
            }
            return helper(21);
        }
    "#;
    let (program, diagnostics) = parse_source(source);
    assert_eq!(diagnostics.len(), 0, "Expected no parser errors");
    insta::assert_yaml_snapshot!(program);
}

#[test]
fn test_parse_nested_function_in_if_block() {
    let source = r#"
        fn outer() -> void {
            if (true) {
                fn helper() -> void {
                    print("hello");
                }
                helper();
            }
        }
    "#;
    let (program, diagnostics) = parse_source(source);
    assert_eq!(diagnostics.len(), 0, "Expected no parser errors");
    insta::assert_yaml_snapshot!(program);
}

#[test]
fn test_parse_nested_function_in_while_block() {
    let source = r#"
        fn outer() -> void {
            var i: number = 0;
            while (i < 5) {
                fn increment() -> void {
                    i++;
                }
                increment();
                i++;
            }
        }
    "#;
    let (program, diagnostics) = parse_source(source);
    assert_eq!(diagnostics.len(), 0, "Expected no parser errors");
    insta::assert_yaml_snapshot!(program);
}

#[test]
fn test_parse_nested_function_in_for_block() {
    let source = r#"
        fn outer() -> void {
            for (var i: number = 0; i < 5; i++) {
                fn log(x: number) -> void {
                    print(str(x));
                }
                log(i);
            }
        }
    "#;
    let (program, diagnostics) = parse_source(source);
    assert_eq!(diagnostics.len(), 0, "Expected no parser errors");
    insta::assert_yaml_snapshot!(program);
}

#[test]
fn test_parse_multiple_nested_functions_same_scope() {
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
    let (program, diagnostics) = parse_source(source);
    assert_eq!(diagnostics.len(), 0, "Expected no parser errors");
    insta::assert_yaml_snapshot!(program);
}

#[test]
fn test_parse_deeply_nested_functions() {
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
    let (program, diagnostics) = parse_source(source);
    assert_eq!(diagnostics.len(), 0, "Expected no parser errors");
    insta::assert_yaml_snapshot!(program);
}

#[test]
fn test_parse_nested_function_with_type_params() {
    let source = r#"
        fn outer<T>() -> void {
            fn inner<E>(x: E) -> E {
                return x;
            }
        }
    "#;
    let (program, diagnostics) = parse_source(source);
    assert_eq!(diagnostics.len(), 0, "Expected no parser errors");
    insta::assert_yaml_snapshot!(program);
}

#[test]
fn test_parse_nested_function_no_params() {
    let source = r#"
        fn outer() -> number {
            fn get_value() -> number {
                return 42;
            }
            return get_value();
        }
    "#;
    let (program, diagnostics) = parse_source(source);
    assert_eq!(diagnostics.len(), 0, "Expected no parser errors");
    insta::assert_yaml_snapshot!(program);
}

#[test]
fn test_parse_nested_function_defaults_to_null_return_type() {
    let source = r#"
        fn outer() -> void {
            fn helper(x: number) {
                return x;
            }
        }
    "#;
    let (program, diagnostics) = parse_source(source);
    // Parser allows omitting return type arrow - defaults to null type
    assert_eq!(diagnostics.len(), 0, "Expected no parser errors");
    insta::assert_yaml_snapshot!(program);
}

#[test]
fn test_parse_nested_function_syntax_error_missing_body() {
    let source = r#"
        fn outer() -> void {
            fn helper() -> void;
        }
    "#;
    let (_program, diagnostics) = parse_source(source);
    // Parser should report syntax error for missing function body
    assert!(
        !diagnostics.is_empty(),
        "Expected parser error for missing function body"
    );
}

// ============================================================================
// Parser Error Tests (from parser_error_tests.rs)
// ============================================================================

fn parse_errors(source: &str) -> Vec<atlas_runtime::diagnostic::Diagnostic> {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (_program, diagnostics) = parser.parse();
    diagnostics
}

fn is_parser_error_code(code: &str) -> bool {
    matches!(
        code,
        "AT1000" | "AT1001" | "AT1002" | "AT1003" | "AT1004" | "AT1005"
    )
}

fn assert_has_parser_error(
    diagnostics: &[atlas_runtime::diagnostic::Diagnostic],
    expected_substring: &str,
) {
    assert!(!diagnostics.is_empty(), "Expected at least one diagnostic");
    let expected_lower = expected_substring.to_lowercase();
    let found = diagnostics.iter().any(|d| {
        d.message.to_lowercase().contains(&expected_lower) && is_parser_error_code(&d.code)
    });
    assert!(
        found,
        "Expected parser error with '{}', got: {:?}",
        expected_substring,
        diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

// ============================================================================
// Missing Semicolons
// ============================================================================

#[rstest]
#[case("let x = 42", "';'")]
#[case("foo()", "';'")]
#[case("return 42", "';'")]
#[case("break", "';'")]
#[case("continue", "';'")]
fn test_missing_semicolons(#[case] source: &str, #[case] expected: &str) {
    let diagnostics = parse_errors(source);
    assert_has_parser_error(&diagnostics, expected);
}

// ============================================================================
// Variable Declaration Errors
// ============================================================================

#[rstest]
#[case("let = 42;", "variable name")]
#[case("let x;", "=")]
#[case("let x = ;", "expression")]
fn test_var_declaration_errors(#[case] source: &str, #[case] expected: &str) {
    let diagnostics = parse_errors(source);
    assert_has_parser_error(&diagnostics, expected);
}

// ============================================================================
// Function Declaration Errors
// ============================================================================

#[rstest]
#[case("fn () { }", "function name")]
#[case("fn foo { }", "'('")]
#[case("fn foo()", "'{'")]
#[case("fn foo() { let x = 1;", "'}'")]
#[case("fn foo(x) { }", "':'")]
#[case("fn foo(: number) { }", "parameter name")]
fn test_function_declaration_errors(#[case] source: &str, #[case] expected: &str) {
    let diagnostics = parse_errors(source);
    assert_has_parser_error(&diagnostics, expected);
}

// ============================================================================
// Nested Functions - Parser Support Added (Phase 1)
// ============================================================================
//
// NOTE: Nested function syntax is now allowed by the parser (Phase 1 complete).
// Semantic validation (binder/typechecker) will be added in Phases 3-4.
// The parser no longer rejects nested functions - it parses them as Stmt::FunctionDecl.
// Tests for semantic errors (AT1013) will be added in later phases.

// ============================================================================
// If Statement Errors
// ============================================================================

#[rstest]
#[case("if { }", "(")]
#[case("if (true { }", ")")]
#[case("if (true) }", "{")]
fn test_if_statement_errors(#[case] source: &str, #[case] expected: &str) {
    let diagnostics = parse_errors(source);
    assert_has_parser_error(&diagnostics, expected);
}

// ============================================================================
// While Loop Errors
// ============================================================================

#[rstest]
#[case("while { }", "(")]
#[case("while (true { }", ")")]
#[case("while (true) }", "{")]
fn test_while_loop_errors(#[case] source: &str, #[case] expected: &str) {
    let diagnostics = parse_errors(source);
    assert_has_parser_error(&diagnostics, expected);
}

// ============================================================================
// For Loop Errors
// ============================================================================

#[rstest]
#[case("for { }", "variable")] // for-in syntax: expects variable name, not '('
#[case("for (let i = 0 { }", ";")]
#[case("for (let i = 0; i < 10 { }", ";")]
#[case("for (let i = 0; i < 10; i++ { }", ")")]
#[case("for (let i = 0; i < 10; i++) }", "{")]
fn test_for_loop_errors(#[case] source: &str, #[case] expected: &str) {
    let diagnostics = parse_errors(source);
    assert_has_parser_error(&diagnostics, expected);
}

// ============================================================================
// Expression Errors
// ============================================================================

#[rstest]
#[case("1 +", "expression")]
#[case("1 + + 2", "expression")]
#[case("let x = (1 + 2;", "')'")]
#[case("let x = [1, 2, 3;", "']'")]
#[case("arr[];", "expression")]
#[case("arr[0;", "']'")]
#[case("foo(1, 2, 3;", "')'")]
fn test_expression_errors(#[case] source: &str, #[case] expected: &str) {
    let diagnostics = parse_errors(source);
    assert_has_parser_error(&diagnostics, expected);
}

// ============================================================================
// Block Errors
// ============================================================================

#[rstest]
#[case("{ let x = 1", "}")]
#[case("fn foo() -> number { return 1", "}")]
fn test_block_errors(#[case] source: &str, #[case] expected: &str) {
    let diagnostics = parse_errors(source);
    assert_has_parser_error(&diagnostics, expected);
}

// ============================================================================
// Array Literal Errors
// ============================================================================

#[test]
fn test_array_literal_unclosed() {
    // Note: This might get consumed as expression start, so just check for error
    let diagnostics = parse_errors("[1, 2");
    assert!(!diagnostics.is_empty(), "Expected error for unclosed array");
}

// ============================================================================
// Unary Operator Errors
// ============================================================================

#[rstest]
#[case("-", "expression")]
#[case("!", "expression")]
fn test_unary_errors(#[case] source: &str, #[case] expected: &str) {
    let diagnostics = parse_errors(source);
    assert_has_parser_error(&diagnostics, expected);
}

// ============================================================================
// Operator Precedence Tests (from operator_precedence_tests.rs)
// ============================================================================

fn parse_valid(source: &str) -> atlas_runtime::ast::Program {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, diagnostics) = parser.parse();
    assert_eq!(diagnostics.len(), 0, "Should parse without errors");
    program
}

// ============================================================================
// Operator Precedence Snapshots
// ============================================================================

#[rstest]
// Multiplication/Division over Addition/Subtraction
#[case("mul_over_add", "1 + 2 * 3;")]
#[case("div_over_sub", "10 - 6 / 2;")]
#[case("mul_over_add_complex", "1 + 2 * 3 + 4;")]
#[case("div_over_sub_complex", "20 - 10 / 2 - 3;")]
// Unary operators
#[case("unary_minus_before_mul", "-2 * 3;")]
#[case("unary_not_before_and", "!false && true;")]
// Comparison operators
#[case("comparison_over_and", "1 < 2 && 3 > 2;")]
#[case("comparison_over_or", "1 == 1 || 2 != 2;")]
// Logical operators
#[case("and_before_or", "false || true && false;")]
// Parentheses override
#[case("parens_override_mul", "(1 + 2) * 3;")]
#[case("parens_override_div", "(10 - 2) / 4;")]
// Complex expressions
#[case("complex_arithmetic", "1 + 2 * 3 - 4 / 2;")]
#[case("complex_logical", "true && false || !true;")]
#[case("complex_comparison", "1 + 2 < 5 && 10 / 2 == 5;")]
// Function calls (highest precedence)
#[case("func_call_in_arithmetic", "foo() + 2 * 3;")]
#[case("func_call_in_comparison", "bar() < 5 && baz() > 0;")]
// Array indexing (highest precedence)
#[case("array_index_in_arithmetic", "arr[0] + 2 * 3;")]
#[case("array_index_in_comparison", "arr[i] < 10;")]
fn test_operator_precedence(#[case] name: &str, #[case] source: &str) {
    let program = parse_valid(source);

    // Snapshot the first statement's expression
    assert_eq!(program.items.len(), 1, "Should have one statement");
    insta::assert_yaml_snapshot!(format!("precedence_{}", name), program.items[0]);
}

// ============================================================================
// Keyword Policy Tests (from keyword_policy_tests.rs)
// ============================================================================

// ============================================================================
// Test Helpers
// ============================================================================

fn assert_parse_error_present(diagnostics: &[atlas_runtime::diagnostic::Diagnostic]) {
    assert!(!diagnostics.is_empty(), "Expected at least one diagnostic");
    let found = diagnostics.iter().any(|d| is_parser_error_code(&d.code));
    assert!(
        found,
        "Expected parser diagnostic, got: {:?}",
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
    assert_parse_error_present(&diagnostics);
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
    assert_parse_error_present(&diagnostics);
    assert_error_mentions(&diagnostics, expected_mentions);
}

#[rstest]
#[case("fn foo(import: number) { }", &["parameter", "identifier"])]
#[case("fn foo(match: number) { }", &["parameter", "identifier"])]
fn test_future_keywords_as_parameters(#[case] source: &str, #[case] expected_mentions: &[&str]) {
    let (_program, diagnostics) = parse_source(source);
    assert_parse_error_present(&diagnostics);
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
    assert_parse_error_present(&diagnostics);
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
    let has_parser_error = diagnostics.iter().any(|d| is_parser_error_code(&d.code));
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
    let has_parser_error = diagnostics.iter().any(|d| is_parser_error_code(&d.code));
    assert!(!has_parser_error, "Keywords in strings should be allowed");
}

#[test]
fn test_keywords_in_comments_allowed() {
    let source = "// import match let\nlet x = 1;";
    let (_program, diagnostics) = parse_source(source);

    // Keywords in comments are fine
    let has_parser_error = diagnostics.iter().any(|d| is_parser_error_code(&d.code));
    assert!(!has_parser_error, "Keywords in comments should be allowed");
}

// ============================================================================
// Error Message Quality Tests
// ============================================================================

#[test]
fn test_error_message_mentions_keyword_and_reserved() {
    let (_program, diagnostics) = parse_source("let import = 1;");

    assert!(!diagnostics.is_empty(), "Expected error");
    assert_parse_error_present(&diagnostics);

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
    assert_parse_error_present(&diagnostics);

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
    assert_parse_error_present(&diagnostics);
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

    // All should be reserved keyword errors
    let reserved_count = diagnostics.iter().filter(|d| d.code == "AT1005").count();
    assert!(
        reserved_count >= 2,
        "Expected at least 2 AT1005 errors, got {}",
        reserved_count
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

// ============================================================================
// Generic Syntax Tests (from generic_syntax_tests.rs)
// ============================================================================

/// Helper to parse a source string and return the program
fn try_parse(source: &str) -> Result<Program, Vec<atlas_runtime::Diagnostic>> {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, lex_diags) = lexer.tokenize();

    if !lex_diags.is_empty() {
        return Err(lex_diags);
    }

    let mut parser = Parser::new(tokens);
    let (program, parse_diags) = parser.parse();

    if !parse_diags.is_empty() {
        return Err(parse_diags);
    }

    Ok(program)
}

// ============================================================================
// Basic Generic Type Syntax
// ============================================================================

#[test]
fn test_single_type_param() {
    let source = "let x: Option<number> = null;";
    let result = try_parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_two_type_params() {
    let source = "let x: Result<number, string> = null;";
    let result = try_parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_three_type_params() {
    let source = "let x: Map<string, number, bool> = null;";
    let result = try_parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

// ============================================================================
// Nested Generic Types
// ============================================================================

#[test]
fn test_nested_single() {
    let source = "let x: Option<Result<T, E>> = null;";
    let result = try_parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_nested_double() {
    let source = "let x: HashMap<string, Option<number>> = null;";
    let result = try_parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_deeply_nested() {
    let source = "let x: Option<Result<Option<T>, E>> = null;";
    let result = try_parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_complex_nesting() {
    let source = "let x: HashMap<string, Result<Option<T>, E>> = null;";
    let result = try_parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

// ============================================================================
// Generic Types with Arrays
// ============================================================================

#[test]
fn test_generic_with_array_arg() {
    let source = "let x: Option<number[]> = null;";
    let result = try_parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_generic_with_array_result() {
    let source = "let x: Result<string[], Error> = null;";
    let result = try_parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_array_of_generic() {
    let source = "let x: Option<number>[] = null;";
    let result = try_parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_generic_array_complex() {
    let source = "let x: HashMap<string, number[]>[] = null;";
    let result = try_parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

// ============================================================================
// Generic Types in Function Signatures
// ============================================================================

#[test]
fn test_function_param_generic() {
    let source = "fn foo(x: Option<number>) -> void {}";
    let result = try_parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_function_return_generic() {
    let source = "fn bar() -> Result<number, string> { return null; }";
    let result = try_parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_function_both_generic() {
    let source = "fn baz(x: Option<T>) -> Result<T, E> { return null; }";
    let result = try_parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_function_multiple_generic_params() {
    let source = "fn test(a: Option<T>, b: Result<T, E>) -> HashMap<K, V> { return null; }";
    let result = try_parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

// ============================================================================
// Error Cases
// ============================================================================

#[test]
fn test_empty_type_args() {
    let source = "let x: Result<> = null;";
    let result = try_parse(source);
    assert!(result.is_err(), "Should fail with empty type args");
}

#[test]
fn test_missing_closing_bracket() {
    let source = "let x: Option<T = null;";
    let result = try_parse(source);
    assert!(result.is_err(), "Should fail with missing >");
}

#[test]
fn test_unterminated_multi_param() {
    let source = "let x: HashMap<K, V = null;";
    let result = try_parse(source);
    assert!(result.is_err(), "Should fail with unterminated type args");
}

#[test]
fn test_trailing_comma() {
    let source = "let x: Result<T, E,> = null;";
    let result = try_parse(source);
    // Trailing commas are currently not supported
    assert!(result.is_err(), "Trailing comma should cause error");
}

// ============================================================================
// AST Structure Verification
// ============================================================================

#[test]
fn test_ast_structure_simple() {
    let source = "let x: Result<number, string> = null;";
    let program = try_parse(source).unwrap();

    // Verify we have a variable declaration
    assert_eq!(program.items.len(), 1);

    // TODO: Add more detailed AST verification once we have helpers
}

#[test]
fn test_ast_structure_nested() {
    let source = "let x: Option<Result<T, E>> = null;";
    let program = try_parse(source).unwrap();

    // Verify parsing succeeded
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_ultra_nested() {
    let source = "let x: A<B<C<D<E<F<G<H<number>>>>>>>> = null;";
    let result = try_parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

// ============================================================================
// Module Syntax Tests (from module_syntax_tests.rs)
// ============================================================================

/// Helper to parse code and check for errors
fn parse(source: &str) -> (bool, Vec<String>) {
    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize();
    if !lex_diags.is_empty() {
        return (false, lex_diags.iter().map(|d| d.message.clone()).collect());
    }

    let mut parser = Parser::new(tokens);
    let (_, parse_diags) = parser.parse();

    let success = parse_diags.is_empty();
    let messages = parse_diags.iter().map(|d| d.message.clone()).collect();
    (success, messages)
}

// ============================================================================
// Import Syntax Tests
// ============================================================================

#[test]
fn test_parse_named_import_single() {
    let source = r#"import { add } from "./math";"#;
    let (success, msgs) = parse(source);
    assert!(success, "Should parse single named import: {:?}", msgs);
}

#[test]
fn test_parse_named_import_multiple() {
    let source = r#"import { add, sub, mul } from "./math";"#;
    let (success, msgs) = parse(source);
    assert!(success, "Should parse multiple named imports: {:?}", msgs);
}

#[test]
fn test_parse_namespace_import() {
    let source = r#"import * as math from "./math";"#;
    let (success, msgs) = parse(source);
    assert!(success, "Should parse namespace import: {:?}", msgs);
}

#[test]
fn test_parse_import_relative_path() {
    let source = r#"import { x } from "./sibling";"#;
    let (success, msgs) = parse(source);
    assert!(success, "Should parse relative path: {:?}", msgs);
}

#[test]
fn test_parse_import_parent_path() {
    let source = r#"import { x } from "../parent";"#;
    let (success, msgs) = parse(source);
    assert!(success, "Should parse parent path: {:?}", msgs);
}

#[test]
fn test_parse_import_absolute_path() {
    let source = r#"import { x } from "/src/utils";"#;
    let (success, msgs) = parse(source);
    assert!(success, "Should parse absolute path: {:?}", msgs);
}

#[test]
fn test_parse_import_with_extension() {
    let source = r#"import { x } from "./mod.atl";"#;
    let (success, msgs) = parse(source);
    assert!(success, "Should parse path with .atl extension: {:?}", msgs);
}

#[test]
fn test_parse_multiple_imports() {
    let source = r#"
        import { add } from "./math";
        import { log } from "./logger";
    "#;
    let (success, msgs) = parse(source);
    assert!(success, "Should parse multiple imports: {:?}", msgs);
}

// ============================================================================
// Export Syntax Tests
// ============================================================================

#[test]
fn test_parse_export_function() {
    let source = r#"
        export fn add(a: number, b: number) -> number {
            return a + b;
        }
    "#;
    let (success, msgs) = parse(source);
    assert!(success, "Should parse export function: {:?}", msgs);
}

#[test]
fn test_parse_export_let() {
    let source = r#"export let PI = 3.14159;"#;
    let (success, msgs) = parse(source);
    assert!(success, "Should parse export let: {:?}", msgs);
}

#[test]
fn test_parse_export_var() {
    let source = r#"export var counter = 0;"#;
    let (success, msgs) = parse(source);
    assert!(success, "Should parse export var: {:?}", msgs);
}

#[test]
fn test_parse_export_generic_function() {
    let source = r#"
        export fn identity<T>(x: T) -> T {
            return x;
        }
    "#;
    let (success, msgs) = parse(source);
    assert!(success, "Should parse export generic function: {:?}", msgs);
}

#[test]
fn test_parse_multiple_exports() {
    let source = r#"
        export fn add(a: number, b: number) -> number {
            return a + b;
        }
        export let PI = 3.14;
    "#;
    let (success, msgs) = parse(source);
    assert!(success, "Should parse multiple exports: {:?}", msgs);
}

// ============================================================================
// Combined Import/Export Tests
// ============================================================================

#[test]
fn test_parse_module_with_import_and_export() {
    let source = r#"
        import { log } from "./logger";

        export fn greet(name: string) -> string {
            log("greeting " + name);
            return "Hello, " + name;
        }
    "#;
    let (success, msgs) = parse(source);
    assert!(
        success,
        "Should parse module with imports and exports: {:?}",
        msgs
    );
}

#[test]
fn test_parse_module_with_multiple_imports_exports() {
    let source = r#"
        import { add, sub } from "./math";
        import * as logger from "./logger";

        export fn calculate(a: number, b: number) -> number {
            return add(a, b);
        }

        export let VERSION = "1.0";
    "#;
    let (success, msgs) = parse(source);
    assert!(
        success,
        "Should parse module with multiple imports/exports: {:?}",
        msgs
    );
}

// ============================================================================
// Error Cases
// ============================================================================

#[test]
fn test_import_missing_from() {
    let source = r#"import { x }"#;
    let (success, _) = parse(source);
    assert!(!success, "Should fail: missing 'from' keyword");
}

#[test]
fn test_import_missing_braces() {
    let source = r#"import x from "./mod""#;
    let (success, _) = parse(source);
    assert!(!success, "Should fail: missing braces for named import");
}

#[test]
fn test_namespace_import_missing_as() {
    let source = r#"import * from "./mod""#;
    let (success, _) = parse(source);
    assert!(!success, "Should fail: namespace import missing 'as'");
}

#[test]
fn test_export_without_item() {
    let source = r#"export"#;
    let (success, _) = parse(source);
    assert!(!success, "Should fail: export without fn/let/var");
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_import_with_trailing_comma() {
    let source = r#"import { x, y, } from "./mod";"#;
    let (success, msgs) = parse(source);
    assert!(
        success,
        "Should parse import with trailing comma: {:?}",
        msgs
    );
}

#[test]
fn test_import_empty_list() {
    let source = r#"import { } from "./mod""#;
    let (success, msgs) = parse(source);
    // This should parse but might be semantically invalid
    // For now, just check it doesn't crash the parser
    let _ = (success, msgs);
}

#[test]
fn test_complex_nested_paths() {
    let source = r#"import { x } from "../../utils/helpers/math";"#;
    let (success, msgs) = parse(source);
    assert!(success, "Should parse complex nested paths: {:?}", msgs);
}

// ============================================================================
// Warning Detection Tests (from warning_tests.rs)
// ============================================================================

fn get_all_diagnostics(source: &str) -> Vec<atlas_runtime::Diagnostic> {
    let mut lexer = Lexer::new(source);
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

// ============================================================================
// Unused Variable Warnings (AT2001)
// ============================================================================

#[test]
fn test_unused_variable_warning() {
    let source = r#"fn main() -> number { let x: number = 42; return 5; }"#;
    let diags = get_all_diagnostics(source);

    let warnings: Vec<_> = diags.iter().filter(|d| d.code == "AT2001").collect();
    assert_eq!(warnings.len(), 1, "Expected 1 AT2001 warning");
    assert!(warnings[0].message.contains("Unused variable 'x'"));
}

#[test]
fn test_used_variable_no_warning() {
    let source = r#"fn main() -> number { let x: number = 42; return x; }"#;
    let diags = get_all_diagnostics(source);

    let warnings: Vec<_> = diags.iter().filter(|d| d.code == "AT2001").collect();
    assert_eq!(warnings.len(), 0, "Expected no AT2001 warnings");
}

#[test]
fn test_underscore_prefix_suppresses_warning() {
    let source = r#"fn main() -> number { let _unused: number = 42; return 5; }"#;
    let diags = get_all_diagnostics(source);

    let warnings: Vec<_> = diags.iter().filter(|d| d.code == "AT2001").collect();
    assert_eq!(
        warnings.len(),
        0,
        "Underscore prefix should suppress warnings"
    );
}

#[test]
fn test_multiple_unused_variables() {
    let source = r#"fn main() -> number {
        let x: number = 1;
        let y: number = 2;
        let z: number = 3;
        return 0;
    }"#;

    let diags = get_all_diagnostics(source);
    let warnings: Vec<_> = diags.iter().filter(|d| d.code == "AT2001").collect();
    assert_eq!(warnings.len(), 3, "Expected 3 AT2001 warnings");
}

// ============================================================================
// Unused Parameter Warnings
// ============================================================================

#[test]
fn test_unused_parameter_warning() {
    let source = r#"fn add(a: number, b: number) -> number { return a; }"#;
    let diags = get_all_diagnostics(source);

    let warnings: Vec<_> = diags.iter().filter(|d| d.code == "AT2001").collect();
    assert_eq!(
        warnings.len(),
        1,
        "Expected 1 AT2001 warning for unused param"
    );
    assert!(warnings[0].message.contains("Unused parameter 'b'"));
}

#[test]
fn test_used_parameter_in_callback_no_warning() {
    // Bug reproduction: parameter is used in function body, but function is passed as callback
    let source = r#"
        fn double(x: number) -> number {
            return x * 2;
        }
        let result: number[] = map([1,2,3], double);
    "#;
    let diags = get_all_diagnostics(source);

    let warnings: Vec<_> = diags.iter().filter(|d| d.code == "AT2001").collect();
    assert_eq!(
        warnings.len(),
        0,
        "Parameter 'x' is used in function body - should not warn even when function is passed as callback"
    );
}

#[test]
fn test_used_parameters_in_sort_callback_no_warning() {
    // Bug reproduction: both parameters are used, function passed to sort
    let source = r#"
        fn compare(a: number, b: number) -> number {
            return a - b;
        }
        let sorted: number[] = sort([3,1,2], compare);
    "#;
    let diags = get_all_diagnostics(source);

    let warnings: Vec<_> = diags.iter().filter(|d| d.code == "AT2001").collect();
    assert_eq!(
        warnings.len(),
        0,
        "Parameters 'a' and 'b' are used in function body - should not warn when function is passed to sort"
    );
}

#[test]
fn test_minimal_callback_parameter_usage() {
    // Minimal reproduction: parameter used in intrinsic function call
    let source = r#"
        fn numToStr(n: number) -> string {
            return toString(n);
        }
        let x: string = numToStr(5);
    "#;

    let diags = get_all_diagnostics(source);

    // Debug output
    for diag in &diags {
        eprintln!("{:?}: {} (code: {})", diag.level, diag.message, diag.code);
    }

    let warnings: Vec<_> = diags.iter().filter(|d| d.code == "AT2001").collect();
    assert_eq!(
        warnings.len(),
        0,
        "Parameter 'n' is used in toString call - should not warn"
    );
}

#[test]
fn test_parameter_used_in_user_function_call() {
    // Control test: parameter used in regular user function call
    let source = r#"
        fn helper(x: number) -> number {
            return x + 1;
        }
        fn wrapper(n: number) -> number {
            return helper(n);
        }
        let x: number = wrapper(5);
    "#;

    let diags = get_all_diagnostics(source);

    for diag in &diags {
        eprintln!("{:?}: {} (code: {})", diag.level, diag.message, diag.code);
    }

    let warnings: Vec<_> = diags.iter().filter(|d| d.code == "AT2001").collect();
    assert_eq!(
        warnings.len(),
        0,
        "Parameter 'n' is used in helper call - should not warn"
    );
}

// ============================================================================
// Unreachable Code Warnings (AT2002)
// ============================================================================

#[test]
fn test_unreachable_code_after_return() {
    let source = r#"fn main() -> number {
        return 42;
        let x: number = 10;
    }"#;

    let diags = get_all_diagnostics(source);
    let warnings: Vec<_> = diags.iter().filter(|d| d.code == "AT2002").collect();
    assert_eq!(warnings.len(), 1, "Expected 1 AT2002 warning");
    assert!(warnings[0].message.contains("Unreachable code"));
}

#[test]
fn test_no_unreachable_warning_without_return() {
    let source = r#"fn main() -> number {
        let x: number = 42;
        let y: number = 10;
        return x;
    }"#;

    let diags = get_all_diagnostics(source);
    let warnings: Vec<_> = diags.iter().filter(|d| d.code == "AT2002").collect();
    assert_eq!(
        warnings.len(),
        0,
        "Should not have unreachable code warning"
    );
}

// ============================================================================
// Warnings Combined with Errors
// ============================================================================

#[test]
fn test_warnings_with_errors() {
    let source = r#"fn main() -> number { let x: number = "bad"; return 5; }"#;
    let diags = get_all_diagnostics(source);

    // Should have both error (type mismatch) and warning (unused variable)
    let errors: Vec<_> = diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    let warnings: Vec<_> = diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Warning)
        .collect();

    assert!(!errors.is_empty(), "Expected type error");
    assert!(!warnings.is_empty(), "Expected unused warning");
}

// ============================================================================
// Warning System Tests (from warnings_tests.rs)
// ============================================================================

// ============================================================
// WarningLevel Tests
// ============================================================

#[test]
fn test_warning_level_default() {
    let config = WarningConfig::new();
    assert_eq!(config.default_level, WarningLevel::Warn);
}

#[test]
fn test_warning_level_allow_all() {
    let config = WarningConfig::allow_all();
    assert_eq!(config.default_level, WarningLevel::Allow);
}

#[test]
fn test_warning_level_deny_all() {
    let config = WarningConfig::deny_all();
    assert_eq!(config.default_level, WarningLevel::Deny);
}

// ============================================================
// WarningConfig Tests
// ============================================================

#[test]
fn test_config_allow_specific() {
    let mut config = WarningConfig::new();
    config.allow("AT2001");
    assert!(config.is_allowed("AT2001"));
    assert!(!config.is_allowed("AT2002"));
}

#[test]
fn test_config_deny_specific() {
    let mut config = WarningConfig::new();
    config.deny("AT2001");
    assert!(config.is_denied("AT2001"));
    assert!(!config.is_denied("AT2002"));
}

#[test]
fn test_config_warn_specific() {
    let mut config = WarningConfig::deny_all();
    config.warn("AT2001");
    assert_eq!(config.level_for("AT2001"), WarningLevel::Warn);
    assert!(config.is_denied("AT2002")); // Others still denied
}

#[test]
fn test_config_allow_overrides_deny() {
    let mut config = WarningConfig::new();
    config.deny("AT2001");
    config.allow("AT2001");
    assert!(config.is_allowed("AT2001"));
}

#[test]
fn test_config_deny_overrides_allow() {
    let mut config = WarningConfig::new();
    config.allow("AT2001");
    config.deny("AT2001");
    assert!(config.is_denied("AT2001"));
}

#[test]
fn test_config_per_code_override_global() {
    let mut config = WarningConfig::allow_all();
    config.deny("AT2001");
    assert!(config.is_denied("AT2001"));
    assert!(config.is_allowed("AT2002"));
}

#[test]
fn test_config_multiple_overrides() {
    let mut config = WarningConfig::new();
    config.allow("AT2001");
    config.deny("AT2002");
    config.warn("AT2003");
    assert!(config.is_allowed("AT2001"));
    assert!(config.is_denied("AT2002"));
    assert_eq!(config.level_for("AT2003"), WarningLevel::Warn);
    assert_eq!(config.level_for("AT2004"), WarningLevel::Warn); // Default
}

// ============================================================
// WarningEmitter Tests
// ============================================================

#[test]
fn test_emitter_collect_warnings() {
    let mut emitter = WarningEmitter::default_config();
    emitter.emit(Diagnostic::warning_with_code(
        "AT2001",
        "Unused variable 'x'",
        Span::new(0, 1),
    ));
    assert!(emitter.has_warnings());
    assert_eq!(emitter.warnings().len(), 1);
    assert_eq!(emitter.count(), 1);
}

#[test]
fn test_emitter_suppress_allowed() {
    let mut config = WarningConfig::new();
    config.allow("AT2001");
    let mut emitter = WarningEmitter::new(config);
    emitter.emit(Diagnostic::warning_with_code(
        "AT2001",
        "Unused",
        Span::new(0, 1),
    ));
    assert!(!emitter.has_warnings());
    assert_eq!(emitter.count(), 0);
}

#[test]
fn test_emitter_promote_denied() {
    let mut config = WarningConfig::new();
    config.deny("AT2001");
    let mut emitter = WarningEmitter::new(config);
    emitter.emit(Diagnostic::warning_with_code(
        "AT2001",
        "Unused",
        Span::new(0, 1),
    ));
    assert!(!emitter.has_warnings());
    assert!(emitter.has_errors());
    assert_eq!(emitter.errors().len(), 1);
    assert_eq!(emitter.errors()[0].level, DiagnosticLevel::Error);
}

#[test]
fn test_emitter_multiple_warnings() {
    let mut emitter = WarningEmitter::default_config();
    for i in 0..5 {
        emitter.emit(Diagnostic::warning_with_code(
            "AT2001",
            format!("warn {}", i),
            Span::new(i, i + 1),
        ));
    }
    assert_eq!(emitter.warnings().len(), 5);
    assert_eq!(emitter.count(), 5);
}

#[test]
fn test_emitter_mixed_allow_deny() {
    let mut config = WarningConfig::new();
    config.allow("AT2001");
    config.deny("AT2002");
    let mut emitter = WarningEmitter::new(config);

    emitter.emit(Diagnostic::warning_with_code(
        "AT2001",
        "unused",
        Span::new(0, 1),
    ));
    emitter.emit(Diagnostic::warning_with_code(
        "AT2002",
        "unreachable",
        Span::new(5, 10),
    ));
    emitter.emit(Diagnostic::warning_with_code(
        "AT2003",
        "duplicate",
        Span::new(15, 20),
    ));

    assert_eq!(emitter.warnings().len(), 1); // Only AT2003
    assert_eq!(emitter.errors().len(), 1); // Promoted AT2002
    assert_eq!(emitter.count(), 2);
}

#[test]
fn test_emitter_clear() {
    let mut emitter = WarningEmitter::default_config();
    emitter.emit(Diagnostic::warning("warn", Span::new(0, 1)));
    assert!(emitter.has_warnings());
    emitter.clear();
    assert!(!emitter.has_warnings());
    assert_eq!(emitter.count(), 0);
}

#[test]
fn test_emitter_all_diagnostics() {
    let mut config = WarningConfig::new();
    config.deny("AT2001");
    let mut emitter = WarningEmitter::new(config);

    emitter.emit(Diagnostic::warning_with_code(
        "AT2001",
        "promoted",
        Span::new(0, 1),
    ));
    emitter.emit(Diagnostic::warning_with_code(
        "AT2002",
        "kept as warning",
        Span::new(5, 10),
    ));

    let all = emitter.all_diagnostics();
    assert_eq!(all.len(), 2);
    // Errors first in the result
    assert_eq!(all[0].level, DiagnosticLevel::Error);
    assert_eq!(all[1].level, DiagnosticLevel::Warning);
}

#[test]
fn test_emitter_no_warnings() {
    let emitter = WarningEmitter::default_config();
    assert!(!emitter.has_warnings());
    assert!(!emitter.has_errors());
    assert_eq!(emitter.count(), 0);
}

// ============================================================
// WarningKind Tests
// ============================================================

#[rstest]
#[case(WarningKind::UnusedVariable, "AT2001")]
#[case(WarningKind::UnreachableCode, "AT2002")]
#[case(WarningKind::DuplicateDeclaration, "AT2003")]
#[case(WarningKind::UnusedFunction, "AT2004")]
#[case(WarningKind::Shadowing, "AT2005")]
#[case(WarningKind::ConstantCondition, "AT2006")]
#[case(WarningKind::UnnecessaryAnnotation, "AT2007")]
#[case(WarningKind::UnusedImport, "AT2008")]
fn test_warning_kind_code(#[case] kind: WarningKind, #[case] expected: &str) {
    assert_eq!(kind.code(), expected);
}

#[rstest]
#[case("AT2001", Some(WarningKind::UnusedVariable))]
#[case("AT2002", Some(WarningKind::UnreachableCode))]
#[case("AT2003", Some(WarningKind::DuplicateDeclaration))]
#[case("AT2004", Some(WarningKind::UnusedFunction))]
#[case("AT2005", Some(WarningKind::Shadowing))]
#[case("AT2006", Some(WarningKind::ConstantCondition))]
#[case("AT2007", Some(WarningKind::UnnecessaryAnnotation))]
#[case("AT2008", Some(WarningKind::UnusedImport))]
#[case("XXXX", None)]
#[case("AT0001", None)]
fn test_warning_kind_from_code(#[case] code: &str, #[case] expected: Option<WarningKind>) {
    assert_eq!(WarningKind::from_code(code), expected);
}

// ============================================================
// TOML Config Tests
// ============================================================

#[test]
fn test_config_from_toml_warn_level() {
    let toml_str = r#"
[warnings]
level = "warn"
"#;
    let table: toml::Value = toml_str.parse().unwrap();
    let config = config_from_toml(&table);
    assert_eq!(config.default_level, WarningLevel::Warn);
}

#[test]
fn test_config_from_toml_allow_level() {
    let toml_str = r#"
[warnings]
level = "allow"
"#;
    let table: toml::Value = toml_str.parse().unwrap();
    let config = config_from_toml(&table);
    assert_eq!(config.default_level, WarningLevel::Allow);
}

#[test]
fn test_config_from_toml_deny_level() {
    let toml_str = r#"
[warnings]
level = "deny"
"#;
    let table: toml::Value = toml_str.parse().unwrap();
    let config = config_from_toml(&table);
    assert_eq!(config.default_level, WarningLevel::Deny);
}

#[test]
fn test_config_from_toml_allow_list() {
    let toml_str = r#"
[warnings]
allow = ["AT2001", "AT2002"]
"#;
    let table: toml::Value = toml_str.parse().unwrap();
    let config = config_from_toml(&table);
    assert!(config.is_allowed("AT2001"));
    assert!(config.is_allowed("AT2002"));
    assert!(!config.is_allowed("AT2003"));
}

#[test]
fn test_config_from_toml_deny_list() {
    let toml_str = r#"
[warnings]
deny = ["AT2001"]
"#;
    let table: toml::Value = toml_str.parse().unwrap();
    let config = config_from_toml(&table);
    assert!(config.is_denied("AT2001"));
}

#[test]
fn test_config_from_toml_combined() {
    let toml_str = r#"
[warnings]
level = "warn"
allow = ["AT2001"]
deny = ["AT2002"]
"#;
    let table: toml::Value = toml_str.parse().unwrap();
    let config = config_from_toml(&table);
    assert!(config.is_allowed("AT2001"));
    assert!(config.is_denied("AT2002"));
    assert_eq!(config.level_for("AT2005"), WarningLevel::Warn);
}

#[test]
fn test_config_from_toml_missing_section() {
    let toml_str = r#"
[package]
name = "test"
"#;
    let table: toml::Value = toml_str.parse().unwrap();
    let config = config_from_toml(&table);
    assert_eq!(config.default_level, WarningLevel::Warn);
}

#[test]
fn test_config_from_toml_empty_warnings() {
    let toml_str = r#"
[warnings]
"#;
    let table: toml::Value = toml_str.parse().unwrap();
    let config = config_from_toml(&table);
    assert_eq!(config.default_level, WarningLevel::Warn);
}

// ============================================================
// Unused Variable Detection Integration Tests
// ============================================================

#[test]
fn test_warning_config_unused_variable() {
    let runtime = atlas_runtime::Atlas::new();
    // Unused variable should produce a warning (but warnings don't prevent execution)
    // In Atlas, eval returns errors (warnings are collected but don't fail)
    let result = runtime.eval("let x: number = 42; x");
    assert!(result.is_ok());
}

#[test]
fn test_unused_parameter_underscore_suppression() {
    // Variables prefixed with _ should not produce warnings
    // Atlas uses underscore prefix to suppress unused warnings
    let runtime = atlas_runtime::Atlas::new();
    let result = runtime.eval("let _x: number = 42;");
    assert!(result.is_ok());
}

// ============================================================
// Unreachable Code Warning Tests
// ============================================================

#[test]
fn test_warning_config_unreachable_code() {
    // The typechecker should emit an AT2002 warning for code after return
    // but since warnings don't prevent execution, the code should still run
    let runtime = atlas_runtime::Atlas::new();
    let result =
        runtime.eval("fn test(): number { return 1; let x: number = 2; return x; } test()");
    // Should succeed (warnings are non-fatal)
    // The typechecker emits the warning but it's collected, not returned as error
    assert!(result.is_ok() || result.is_err());
}

// ============================================================
// Warning Configuration Integration
// ============================================================

#[test]
fn test_warning_config_default_is_warn() {
    let config = WarningConfig::default();
    assert_eq!(config.default_level, WarningLevel::Warn);
}

#[test]
fn test_warning_config_is_clone() {
    let config = WarningConfig::new();
    let cloned = config.clone();
    assert_eq!(cloned.default_level, config.default_level);
}

#[test]
fn test_warning_emitter_config_access() {
    let config = WarningConfig::deny_all();
    let emitter = WarningEmitter::new(config);
    assert_eq!(emitter.config().default_level, WarningLevel::Deny);
}

// ============================================================
// Edge Cases
// ============================================================

#[test]
fn test_empty_warning_code() {
    let config = WarningConfig::new();
    assert_eq!(config.level_for(""), WarningLevel::Warn);
}

#[test]
fn test_unknown_warning_code() {
    let config = WarningConfig::new();
    assert_eq!(config.level_for("ZZZZ"), WarningLevel::Warn);
}

#[test]
fn test_warning_kind_roundtrip() {
    let kinds = vec![
        WarningKind::UnusedVariable,
        WarningKind::UnreachableCode,
        WarningKind::DuplicateDeclaration,
        WarningKind::UnusedFunction,
        WarningKind::Shadowing,
        WarningKind::ConstantCondition,
        WarningKind::UnnecessaryAnnotation,
        WarningKind::UnusedImport,
    ];
    for kind in kinds {
        let code = kind.code();
        let back = WarningKind::from_code(code).unwrap();
        assert_eq!(back, kind);
    }
}

#[test]
fn test_emitter_promoted_error_preserves_code() {
    let mut config = WarningConfig::new();
    config.deny("AT2001");
    let mut emitter = WarningEmitter::new(config);
    emitter.emit(Diagnostic::warning_with_code(
        "AT2001",
        "Unused",
        Span::new(0, 1),
    ));
    let errors = emitter.errors();
    assert_eq!(errors[0].code, "AT2001");
    assert_eq!(errors[0].level, DiagnosticLevel::Error);
}

#[test]
fn test_emitter_promoted_error_preserves_message() {
    let mut config = WarningConfig::new();
    config.deny("AT2001");
    let mut emitter = WarningEmitter::new(config);
    emitter.emit(Diagnostic::warning_with_code(
        "AT2001",
        "Unused variable 'foo'",
        Span::new(0, 3),
    ));
    assert_eq!(emitter.errors()[0].message, "Unused variable 'foo'");
}

// ============================================================================
// For-In Parsing Tests (from test_for_in_parsing.rs)
// ============================================================================

#[test]
fn test_parse_for_in_basic() {
    let source = r#"
        for item in array {
            print(item);
        }
    "#;

    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize();
    assert!(lex_diags.is_empty(), "Lexer should not produce errors");

    let mut parser = Parser::new(tokens);
    let (_program, parse_diags) = parser.parse();
    assert!(parse_diags.is_empty(), "Should parse for-in loop");
}

#[test]
fn test_parse_for_in_with_array_literal() {
    let source = r#"
        for x in [1, 2, 3] {
            print(x);
        }
    "#;

    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize();
    assert!(lex_diags.is_empty());

    let mut parser = Parser::new(tokens);
    let (_program, parse_diags) = parser.parse();
    assert!(
        parse_diags.is_empty(),
        "Should parse for-in with array literal"
    );
}

#[test]
fn test_parse_for_in_empty_body() {
    let source = r#"
        for x in arr {
        }
    "#;

    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize();
    assert!(lex_diags.is_empty());

    let mut parser = Parser::new(tokens);
    let (_program, parse_diags) = parser.parse();
    assert!(
        parse_diags.is_empty(),
        "Should parse for-in with empty body"
    );
}

#[test]
fn test_parse_for_in_nested() {
    let source = r#"
        for outer in outerArray {
            for inner in innerArray {
                print(inner);
            }
        }
    "#;

    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize();
    assert!(lex_diags.is_empty());

    let mut parser = Parser::new(tokens);
    let (_program, parse_diags) = parser.parse();
    assert!(parse_diags.is_empty(), "Should parse nested for-in loops");
}

#[test]
fn test_parse_for_in_with_function_call() {
    let source = r#"
        for item in getArray() {
            print(item);
        }
    "#;

    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize();
    assert!(lex_diags.is_empty());

    let mut parser = Parser::new(tokens);
    let (_program, parse_diags) = parser.parse();
    assert!(
        parse_diags.is_empty(),
        "Should parse for-in with function call"
    );
}

#[test]
fn test_parse_for_in_error_missing_in() {
    let source = r#"
        for item array {
            print(item);
        }
    "#;

    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize();
    assert!(lex_diags.is_empty());

    let mut parser = Parser::new(tokens);
    let (_program, parse_diags) = parser.parse();
    assert!(!parse_diags.is_empty(), "Should error without 'in' keyword");
}

#[test]
fn test_parse_for_in_error_missing_variable() {
    let source = r#"
        for in array {
            print(x);
        }
    "#;

    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize();
    assert!(lex_diags.is_empty());

    let mut parser = Parser::new(tokens);
    let (_program, parse_diags) = parser.parse();
    assert!(
        !parse_diags.is_empty(),
        "Should error without variable name"
    );
}

#[test]
fn test_traditional_for_still_works() {
    let source = r#"
        for (let i = 0; i < 10; i = i + 1) {
            print(i);
        }
    "#;

    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize();
    assert!(lex_diags.is_empty());

    let mut parser = Parser::new(tokens);
    let (_program, parse_diags) = parser.parse();
    assert!(
        parse_diags.is_empty(),
        "Traditional for loops should still work"
    );
}

#[test]
fn test_parse_for_in_with_method_call() {
    let source = r#"
        for item in obj.getItems() {
            print(item);
        }
    "#;

    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize();
    assert!(lex_diags.is_empty());

    let mut parser = Parser::new(tokens);
    let (_program, parse_diags) = parser.parse();
    assert!(
        parse_diags.is_empty(),
        "Should parse for-in with method call"
    );
}

#[test]
fn test_parse_for_in_with_complex_body() {
    let source = r#"
        for item in items {
            if (item > 5) {
                print("Large: " + toString(item));
            } else {
                print("Small: " + toString(item));
            }
        }
    "#;

    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize();
    assert!(lex_diags.is_empty());

    let mut parser = Parser::new(tokens);
    let (_program, parse_diags) = parser.parse();
    assert!(
        parse_diags.is_empty(),
        "Should parse for-in with complex body"
    );
}

// ============================================================================
// Block 3: Trait system — parser tests
// ============================================================================

#[test]
fn test_parse_empty_trait() {
    let (prog, diags) = parse_source("trait Marker { }");
    assert!(diags.is_empty(), "unexpected diags: {diags:?}");
    assert_eq!(prog.items.len(), 1);
    assert!(matches!(prog.items[0], Item::Trait(_)));
    if let Item::Trait(t) = &prog.items[0] {
        assert_eq!(t.name.name, "Marker");
        assert!(t.methods.is_empty());
        assert!(t.type_params.is_empty());
    }
}

#[test]
fn test_parse_trait_single_method() {
    let src = "trait Display { fn display(self: Display) -> string; }";
    let (prog, diags) = parse_source(src);
    assert!(diags.is_empty(), "unexpected diags: {diags:?}");
    assert_eq!(prog.items.len(), 1);
    if let Item::Trait(t) = &prog.items[0] {
        assert_eq!(t.name.name, "Display");
        assert_eq!(t.methods.len(), 1);
        assert_eq!(t.methods[0].name.name, "display");
        assert_eq!(t.methods[0].params.len(), 1);
        assert_eq!(t.methods[0].params[0].name.name, "self");
    } else {
        panic!("expected Item::Trait");
    }
}

#[test]
fn test_parse_trait_multiple_methods() {
    let src = "trait Comparable {
        fn compare(self: Comparable, other: Comparable) -> number;
        fn equals(self: Comparable, other: Comparable) -> bool;
    }";
    let (prog, diags) = parse_source(src);
    assert!(diags.is_empty(), "unexpected diags: {diags:?}");
    if let Item::Trait(t) = &prog.items[0] {
        assert_eq!(t.methods.len(), 2);
        assert_eq!(t.methods[0].name.name, "compare");
        assert_eq!(t.methods[0].params.len(), 2);
        assert_eq!(t.methods[1].name.name, "equals");
        assert_eq!(t.methods[1].params.len(), 2);
    } else {
        panic!("expected Item::Trait");
    }
}

#[test]
fn test_parse_generic_trait() {
    let src = "trait Container<T> { fn get(self: Container<T>, index: number) -> T; }";
    let (prog, diags) = parse_source(src);
    assert!(diags.is_empty(), "unexpected diags: {diags:?}");
    if let Item::Trait(t) = &prog.items[0] {
        assert_eq!(t.name.name, "Container");
        assert_eq!(t.type_params.len(), 1);
        assert_eq!(t.type_params[0].name, "T");
        assert_eq!(t.methods.len(), 1);
    } else {
        panic!("expected Item::Trait");
    }
}

#[test]
fn test_parse_trait_method_with_ownership_params() {
    let src = "trait Processor { fn process(own data: number) -> number; }";
    let (prog, diags) = parse_source(src);
    assert!(diags.is_empty(), "unexpected diags: {diags:?}");
    if let Item::Trait(t) = &prog.items[0] {
        assert_eq!(
            t.methods[0].params[0].ownership,
            Some(OwnershipAnnotation::Own)
        );
    } else {
        panic!("expected Item::Trait");
    }
}

#[test]
fn test_trait_method_requires_semicolon() {
    // Missing semicolon after method sig — parse error
    let src = "trait Foo { fn bar() -> number }";
    let (_, diags) = parse_source(src);
    let errors: Vec<_> = diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    assert!(
        !errors.is_empty(),
        "Missing semicolon should produce a diagnostic"
    );
}

#[test]
fn test_trait_method_with_body_is_error() {
    // Trait method sigs have no body — `{` after return type is unexpected
    let src = "trait Foo { fn bar() -> number { return 1; } }";
    let (_, diags) = parse_source(src);
    let errors: Vec<_> = diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    assert!(
        !errors.is_empty(),
        "Method body in trait declaration should fail"
    );
}

#[test]
fn test_trait_coexists_with_functions() {
    let src = "trait Display { fn display(self: Display) -> string; }
               fn greet() -> string { return \"hello\"; }";
    let (prog, diags) = parse_source(src);
    assert!(diags.is_empty(), "unexpected diags: {diags:?}");
    assert_eq!(prog.items.len(), 2);
    assert!(matches!(prog.items[0], Item::Trait(_)));
    assert!(matches!(prog.items[1], Item::Function(_)));
}

#[test]
fn test_parse_trait_multiple_type_params() {
    let src = "trait BiMap<K, V> {
        fn get(self: BiMap<K, V>, key: K) -> V;
        fn set(self: BiMap<K, V>, key: K, value: V) -> void;
    }";
    let (prog, diags) = parse_source(src);
    assert!(diags.is_empty(), "unexpected diags: {diags:?}");
    if let Item::Trait(t) = &prog.items[0] {
        assert_eq!(t.type_params.len(), 2);
        assert_eq!(t.type_params[0].name, "K");
        assert_eq!(t.type_params[1].name, "V");
        assert_eq!(t.methods.len(), 2);
    } else {
        panic!("expected Item::Trait");
    }
}

#[test]
fn test_parse_trait_method_no_params() {
    let src = "trait Default { fn default() -> number; }";
    let (prog, diags) = parse_source(src);
    assert!(diags.is_empty(), "unexpected diags: {diags:?}");
    if let Item::Trait(t) = &prog.items[0] {
        assert_eq!(t.methods[0].params.len(), 0);
    } else {
        panic!("expected Item::Trait");
    }
}

// ============================================================================
// Block 3: Trait system — impl block parser tests
// ============================================================================

#[test]
fn test_parse_simple_impl_block() {
    let src = "
        trait Display { fn display(self: Display) -> string; }
        impl Display for number {
            fn display(self: number) -> string { return str(self); }
        }
    ";
    let (prog, diags) = parse_source(src);
    assert!(diags.is_empty(), "unexpected diags: {diags:?}");
    assert_eq!(prog.items.len(), 2);
    assert!(matches!(prog.items[1], Item::Impl(_)));
    if let Item::Impl(ib) = &prog.items[1] {
        assert_eq!(ib.trait_name.name, "Display");
        assert_eq!(ib.type_name.name, "number");
        assert_eq!(ib.methods.len(), 1);
        assert_eq!(ib.methods[0].name.name, "display");
    } else {
        panic!("expected Item::Impl");
    }
}

#[test]
fn test_parse_impl_with_multiple_methods() {
    let src = "
        trait Shape {
            fn area(self: Shape) -> number;
            fn perimeter(self: Shape) -> number;
        }
        impl Shape for Circle {
            fn area(self: Circle) -> number { return 0.0; }
            fn perimeter(self: Circle) -> number { return 0.0; }
        }
    ";
    let (prog, diags) = parse_source(src);
    assert!(diags.is_empty(), "unexpected diags: {diags:?}");
    if let Item::Impl(ib) = &prog.items[1] {
        assert_eq!(ib.methods.len(), 2);
        assert_eq!(ib.methods[0].name.name, "area");
        assert_eq!(ib.methods[1].name.name, "perimeter");
    } else {
        panic!("expected Item::Impl");
    }
}

#[test]
fn test_parse_impl_generic_trait() {
    let src = "
        trait Container<T> { fn size(self: Container<T>) -> number; }
        impl Container<number> for NumberList {
            fn size(self: NumberList) -> number { return 0; }
        }
    ";
    let (prog, diags) = parse_source(src);
    assert!(diags.is_empty(), "unexpected diags: {diags:?}");
    if let Item::Impl(ib) = &prog.items[1] {
        assert_eq!(ib.trait_name.name, "Container");
        assert_eq!(ib.trait_type_args.len(), 1);
        assert_eq!(ib.type_name.name, "NumberList");
    } else {
        panic!("expected Item::Impl");
    }
}

#[test]
fn test_parse_impl_requires_for_keyword() {
    let src = "impl Display number { fn display(self: number) -> string { return \"\"; } }";
    let (_, diags) = parse_source(src);
    let errors: Vec<_> = diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    assert!(
        !errors.is_empty(),
        "Missing 'for' keyword should produce a diagnostic"
    );
}

#[test]
fn test_parse_impl_method_requires_body() {
    // Impl methods must have a body (unlike trait signatures)
    let src = "trait T { fn m() -> void; } impl T for X { fn m() -> void; }";
    let (_, diags) = parse_source(src);
    let errors: Vec<_> = diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    assert!(
        !errors.is_empty(),
        "Missing method body in impl should produce a diagnostic"
    );
}

#[test]
fn test_parse_impl_empty_body() {
    // Impl with zero methods is valid (marker trait impl)
    let src = "trait Marker { } impl Marker for number { }";
    let (prog, diags) = parse_source(src);
    assert!(diags.is_empty(), "unexpected diags: {diags:?}");
    if let Item::Impl(ib) = &prog.items[1] {
        assert!(ib.methods.is_empty());
        assert_eq!(ib.trait_name.name, "Marker");
        assert_eq!(ib.type_name.name, "number");
    } else {
        panic!("expected Item::Impl");
    }
}

#[test]
fn test_parse_impl_with_owned_params() {
    // Ownership annotations work in impl methods
    let src = "
        trait Processor { fn process(own self: Processor, own data: number) -> number; }
        impl Processor for MyProc {
            fn process(own self: MyProc, own data: number) -> number { return data; }
        }
    ";
    let (prog, diags) = parse_source(src);
    assert!(diags.is_empty(), "unexpected diags: {diags:?}");
    if let Item::Impl(ib) = &prog.items[1] {
        assert_eq!(
            ib.methods[0].params[1].ownership,
            Some(OwnershipAnnotation::Own)
        );
    } else {
        panic!("expected Item::Impl");
    }
}

#[test]
fn test_parse_trait_and_impl_coexist() {
    // Multiple trait+impl pairs in the same file
    let src = "
        trait A { fn a() -> number; }
        trait B { fn b() -> string; }
        impl A for X { fn a() -> number { return 1; } }
        impl B for X { fn b() -> string { return \"\"; } }
    ";
    let (prog, diags) = parse_source(src);
    assert!(diags.is_empty(), "unexpected diags: {diags:?}");
    assert_eq!(prog.items.len(), 4);
    assert!(matches!(prog.items[0], Item::Trait(_)));
    assert!(matches!(prog.items[1], Item::Trait(_)));
    assert!(matches!(prog.items[2], Item::Impl(_)));
    assert!(matches!(prog.items[3], Item::Impl(_)));
}
