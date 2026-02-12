//! Comprehensive lexer tests

use atlas_runtime::lexer::Lexer;
use atlas_runtime::token::TokenKind;

#[test]
fn test_string_literal_basic() {
    let mut lexer = Lexer::new(r#""hello world""#);
    let (tokens, diagnostics) = lexer.tokenize();

    assert_eq!(diagnostics.len(), 0);
    assert_eq!(tokens.len(), 2); // String + EOF
    assert_eq!(tokens[0].kind, TokenKind::String);
    assert_eq!(tokens[0].lexeme, "hello world");
}

#[test]
fn test_string_literal_escapes() {
    let mut lexer = Lexer::new(r#""line1\nline2\ttab\r\n""#);
    let (tokens, diagnostics) = lexer.tokenize();

    assert_eq!(diagnostics.len(), 0);
    assert_eq!(tokens[0].kind, TokenKind::String);
    assert_eq!(tokens[0].lexeme, "line1\nline2\ttab\r\n");
}

#[test]
fn test_string_literal_escaped_quote() {
    let mut lexer = Lexer::new(r#""He said \"hello\"""#);
    let (tokens, diagnostics) = lexer.tokenize();

    assert_eq!(diagnostics.len(), 0);
    assert_eq!(tokens[0].kind, TokenKind::String);
    assert_eq!(tokens[0].lexeme, r#"He said "hello""#);
}

#[test]
fn test_string_literal_escaped_backslash() {
    let mut lexer = Lexer::new(r#""path\\to\\file""#);
    let (tokens, diagnostics) = lexer.tokenize();

    assert_eq!(diagnostics.len(), 0);
    assert_eq!(tokens[0].kind, TokenKind::String);
    assert_eq!(tokens[0].lexeme, r"path\to\file");
}

#[test]
fn test_string_literal_multiline() {
    let mut lexer = Lexer::new("\"line1\nline2\nline3\"");
    let (tokens, diagnostics) = lexer.tokenize();

    assert_eq!(diagnostics.len(), 0);
    assert_eq!(tokens[0].kind, TokenKind::String);
    assert_eq!(tokens[0].lexeme, "line1\nline2\nline3");
}

#[test]
fn test_string_literal_unterminated() {
    let mut lexer = Lexer::new(r#""unterminated string"#);
    let (tokens, diagnostics) = lexer.tokenize();

    assert_eq!(tokens[0].kind, TokenKind::Error);
    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].message.contains("Unterminated"));
}

#[test]
fn test_string_literal_invalid_escape() {
    let mut lexer = Lexer::new(r#""invalid\xescape""#);
    let (tokens, diagnostics) = lexer.tokenize();

    assert_eq!(tokens[0].kind, TokenKind::Error);
    // May have multiple errors: invalid escape
    assert!(!diagnostics.is_empty());
    assert!(diagnostics[0].message.contains("Invalid escape"));
}

#[test]
fn test_number_integer() {
    let mut lexer = Lexer::new("0 1 42 999 1234567890");
    let (tokens, diagnostics) = lexer.tokenize();

    assert_eq!(diagnostics.len(), 0);
    assert_eq!(tokens[0].lexeme, "0");
    assert_eq!(tokens[1].lexeme, "1");
    assert_eq!(tokens[2].lexeme, "42");
    assert_eq!(tokens[3].lexeme, "999");
    assert_eq!(tokens[4].lexeme, "1234567890");
}

#[test]
fn test_number_float() {
    let mut lexer = Lexer::new("0.0 3.14 99.999 0.5");
    let (tokens, diagnostics) = lexer.tokenize();

    assert_eq!(diagnostics.len(), 0);
    assert_eq!(tokens[0].lexeme, "0.0");
    assert_eq!(tokens[1].lexeme, "3.14");
    assert_eq!(tokens[2].lexeme, "99.999");
    assert_eq!(tokens[3].lexeme, "0.5");
}

#[test]
fn test_number_dot_without_fractional() {
    // "42." should be parsed as "42" followed by unexpected "."
    let mut lexer = Lexer::new("42.");
    let (tokens, _) = lexer.tokenize();

    // Should parse as: Number("42"), Error("."), EOF
    assert_eq!(tokens[0].kind, TokenKind::Number);
    assert_eq!(tokens[0].lexeme, "42");
}

#[test]
fn test_comment_single_line() {
    let source = r#"
// This is a comment
let x = 5; // Another comment
// Final comment
"#;
    let mut lexer = Lexer::new(source);
    let (tokens, diagnostics) = lexer.tokenize();

    assert_eq!(diagnostics.len(), 0);

    // Should have: Let, Identifier, Equal, Number, Semicolon, EOF
    let kinds: Vec<TokenKind> = tokens.iter().map(|t| t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Let,
            TokenKind::Identifier,
            TokenKind::Equal,
            TokenKind::Number,
            TokenKind::Semicolon,
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_comment_multi_line() {
    let source = r#"
let x = /* inline comment */ 5;
/* Multi-line
   comment
   here */
let y = 10;
"#;
    let mut lexer = Lexer::new(source);
    let (tokens, diagnostics) = lexer.tokenize();

    assert_eq!(diagnostics.len(), 0);

    // Should have: Let, Identifier, Equal, Number, Semicolon, Let, Identifier, Equal, Number, Semicolon, EOF
    let kinds: Vec<TokenKind> = tokens.iter().map(|t| t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Let,
            TokenKind::Identifier,
            TokenKind::Equal,
            TokenKind::Number,
            TokenKind::Semicolon,
            TokenKind::Let,
            TokenKind::Identifier,
            TokenKind::Equal,
            TokenKind::Number,
            TokenKind::Semicolon,
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_comment_nested_block() {
    let source = "/* outer /* inner */ still in comment */ let x = 5;";
    let mut lexer = Lexer::new(source);
    let (tokens, _) = lexer.tokenize();

    // Note: We don't support nested block comments in v0.1
    // The comment ends at first */
    // This should parse the text after first */ as code
    assert!(tokens.len() > 1);
}

#[test]
fn test_all_keywords() {
    let source = "let var fn if else while for return break continue true false null import match";
    let mut lexer = Lexer::new(source);
    let (tokens, diagnostics) = lexer.tokenize();

    assert_eq!(diagnostics.len(), 0);

    let expected = vec![
        TokenKind::Let,
        TokenKind::Var,
        TokenKind::Fn,
        TokenKind::If,
        TokenKind::Else,
        TokenKind::While,
        TokenKind::For,
        TokenKind::Return,
        TokenKind::Break,
        TokenKind::Continue,
        TokenKind::True,
        TokenKind::False,
        TokenKind::Null,
        TokenKind::Import,
        TokenKind::Match,
        TokenKind::Eof,
    ];

    let kinds: Vec<TokenKind> = tokens.iter().map(|t| t.kind).collect();
    assert_eq!(kinds, expected);
}

#[test]
fn test_all_operators() {
    let source = "+ - * / % ! = == != < <= > >= && || ->";
    let mut lexer = Lexer::new(source);
    let (tokens, diagnostics) = lexer.tokenize();

    assert_eq!(diagnostics.len(), 0);

    let expected = vec![
        TokenKind::Plus,
        TokenKind::Minus,
        TokenKind::Star,
        TokenKind::Slash,
        TokenKind::Percent,
        TokenKind::Bang,
        TokenKind::Equal,
        TokenKind::EqualEqual,
        TokenKind::BangEqual,
        TokenKind::Less,
        TokenKind::LessEqual,
        TokenKind::Greater,
        TokenKind::GreaterEqual,
        TokenKind::AmpAmp,
        TokenKind::PipePipe,
        TokenKind::Arrow,
        TokenKind::Eof,
    ];

    let kinds: Vec<TokenKind> = tokens.iter().map(|t| t.kind).collect();
    assert_eq!(kinds, expected);
}

#[test]
fn test_all_punctuation() {
    let source = "( ) { } [ ] ; , :";
    let mut lexer = Lexer::new(source);
    let (tokens, diagnostics) = lexer.tokenize();

    assert_eq!(diagnostics.len(), 0);

    let expected = vec![
        TokenKind::LeftParen,
        TokenKind::RightParen,
        TokenKind::LeftBrace,
        TokenKind::RightBrace,
        TokenKind::LeftBracket,
        TokenKind::RightBracket,
        TokenKind::Semicolon,
        TokenKind::Comma,
        TokenKind::Colon,
        TokenKind::Eof,
    ];

    let kinds: Vec<TokenKind> = tokens.iter().map(|t| t.kind).collect();
    assert_eq!(kinds, expected);
}

#[test]
fn test_identifier_variations() {
    let source = "x _x x123 _123 snake_case camelCase SCREAMING_SNAKE";
    let mut lexer = Lexer::new(source);
    let (tokens, diagnostics) = lexer.tokenize();

    assert_eq!(diagnostics.len(), 0);

    for i in 0..7 {
        assert_eq!(tokens[i].kind, TokenKind::Identifier);
    }

    assert_eq!(tokens[0].lexeme, "x");
    assert_eq!(tokens[1].lexeme, "_x");
    assert_eq!(tokens[2].lexeme, "x123");
    assert_eq!(tokens[3].lexeme, "_123");
    assert_eq!(tokens[4].lexeme, "snake_case");
    assert_eq!(tokens[5].lexeme, "camelCase");
    assert_eq!(tokens[6].lexeme, "SCREAMING_SNAKE");
}

#[test]
fn test_complete_program() {
    let source = r#"
fn fibonacci(n: number) -> number {
    if (n <= 1) {
        return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

let result = fibonacci(10);
"#;

    let mut lexer = Lexer::new(source);
    let (tokens, diagnostics) = lexer.tokenize();

    assert_eq!(diagnostics.len(), 0);

    // Just verify we got a reasonable number of tokens and they make sense
    assert!(tokens.len() > 30);

    // First tokens should be: fn, identifier("fibonacci"), (
    assert_eq!(tokens[0].kind, TokenKind::Fn);
    assert_eq!(tokens[1].kind, TokenKind::Identifier);
    assert_eq!(tokens[1].lexeme, "fibonacci");
    assert_eq!(tokens[2].kind, TokenKind::LeftParen);
}

#[test]
fn test_span_accuracy() {
    let source = "let x = 42;";
    let mut lexer = Lexer::new(source);
    let (tokens, _) = lexer.tokenize();

    // "let" at 0-3
    assert_eq!(tokens[0].span.start, 0);
    assert_eq!(tokens[0].span.end, 3);

    // "x" at 4-5
    assert_eq!(tokens[1].span.start, 4);
    assert_eq!(tokens[1].span.end, 5);

    // "=" at 6-7
    assert_eq!(tokens[2].span.start, 6);
    assert_eq!(tokens[2].span.end, 7);

    // "42" at 8-10
    assert_eq!(tokens[3].span.start, 8);
    assert_eq!(tokens[3].span.end, 10);

    // ";" at 10-11
    assert_eq!(tokens[4].span.start, 10);
    assert_eq!(tokens[4].span.end, 11);
}

#[test]
fn test_error_unexpected_character() {
    let mut lexer = Lexer::new("let x = @;");
    let (tokens, diagnostics) = lexer.tokenize();

    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].message.contains("Unexpected character"));

    // Should have: Let, Identifier, Equal, Error, Semicolon, EOF
    assert_eq!(tokens[0].kind, TokenKind::Let);
    assert_eq!(tokens[1].kind, TokenKind::Identifier);
    assert_eq!(tokens[2].kind, TokenKind::Equal);
    assert_eq!(tokens[3].kind, TokenKind::Error);
    assert_eq!(tokens[4].kind, TokenKind::Semicolon);
}

#[test]
fn test_error_single_ampersand() {
    let mut lexer = Lexer::new("x & y");
    let (tokens, diagnostics) = lexer.tokenize();

    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].message.contains("&"));

    assert_eq!(tokens[0].kind, TokenKind::Identifier);
    assert_eq!(tokens[1].kind, TokenKind::Error);
    assert_eq!(tokens[2].kind, TokenKind::Identifier);
}

#[test]
fn test_error_single_pipe() {
    let mut lexer = Lexer::new("x | y");
    let (tokens, diagnostics) = lexer.tokenize();

    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].message.contains("|"));

    assert_eq!(tokens[0].kind, TokenKind::Identifier);
    assert_eq!(tokens[1].kind, TokenKind::Error);
    assert_eq!(tokens[2].kind, TokenKind::Identifier);
}

#[test]
fn test_error_recovery() {
    // Test that lexer continues after errors
    let source = "let x = @ y = 42;";
    let mut lexer = Lexer::new(source);
    let (tokens, diagnostics) = lexer.tokenize();

    assert_eq!(diagnostics.len(), 1);

    // Should still lex: Let, Identifier, Equal, Error, Identifier, Equal, Number, Semicolon, EOF
    assert_eq!(tokens[0].kind, TokenKind::Let);
    assert_eq!(tokens[1].kind, TokenKind::Identifier);
    assert_eq!(tokens[2].kind, TokenKind::Equal);
    assert_eq!(tokens[3].kind, TokenKind::Error);
    assert_eq!(tokens[4].kind, TokenKind::Identifier);
    assert_eq!(tokens[5].kind, TokenKind::Equal);
    assert_eq!(tokens[6].kind, TokenKind::Number);
    assert_eq!(tokens[7].kind, TokenKind::Semicolon);
}

#[test]
fn test_operators_without_spaces() {
    let source = "a+b*c-d/e%f";
    let mut lexer = Lexer::new(source);
    let (tokens, diagnostics) = lexer.tokenize();

    assert_eq!(diagnostics.len(), 0);

    let expected = vec![
        TokenKind::Identifier, // a
        TokenKind::Plus,
        TokenKind::Identifier, // b
        TokenKind::Star,
        TokenKind::Identifier, // c
        TokenKind::Minus,
        TokenKind::Identifier, // d
        TokenKind::Slash,
        TokenKind::Identifier, // e
        TokenKind::Percent,
        TokenKind::Identifier, // f
        TokenKind::Eof,
    ];

    let kinds: Vec<TokenKind> = tokens.iter().map(|t| t.kind).collect();
    assert_eq!(kinds, expected);
}

#[test]
fn test_comparison_chain() {
    let source = "a==b!=c<d<=e>f>=g";
    let mut lexer = Lexer::new(source);
    let (tokens, diagnostics) = lexer.tokenize();

    assert_eq!(diagnostics.len(), 0);

    let expected = vec![
        TokenKind::Identifier, // a
        TokenKind::EqualEqual,
        TokenKind::Identifier, // b
        TokenKind::BangEqual,
        TokenKind::Identifier, // c
        TokenKind::Less,
        TokenKind::Identifier, // d
        TokenKind::LessEqual,
        TokenKind::Identifier, // e
        TokenKind::Greater,
        TokenKind::Identifier, // f
        TokenKind::GreaterEqual,
        TokenKind::Identifier, // g
        TokenKind::Eof,
    ];

    let kinds: Vec<TokenKind> = tokens.iter().map(|t| t.kind).collect();
    assert_eq!(kinds, expected);
}

#[test]
fn test_empty_string() {
    let mut lexer = Lexer::new(r#""""#);
    let (tokens, diagnostics) = lexer.tokenize();

    assert_eq!(diagnostics.len(), 0);
    assert_eq!(tokens[0].kind, TokenKind::String);
    assert_eq!(tokens[0].lexeme, "");
}

#[test]
fn test_whitespace_only() {
    let mut lexer = Lexer::new("   \t\n\r\n   ");
    let (tokens, diagnostics) = lexer.tokenize();

    assert_eq!(diagnostics.len(), 0);
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::Eof);
}

#[test]
fn test_comments_only() {
    let source = r#"
// Comment 1
/* Comment 2 */
// Comment 3
"#;
    let mut lexer = Lexer::new(source);
    let (tokens, diagnostics) = lexer.tokenize();

    assert_eq!(diagnostics.len(), 0);
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::Eof);
}
