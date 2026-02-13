//! Modern lexer tests using rstest for parameterization
//!
//! This demonstrates the new testing approach with ~90% less boilerplate.
//! Compare with lexer_tests.rs to see the improvement.

use atlas_runtime::lexer::Lexer;
use atlas_runtime::token::TokenKind;
use atlas_runtime::diagnostic::Diagnostic;
use rstest::rstest;
use pretty_assertions::assert_eq;

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
