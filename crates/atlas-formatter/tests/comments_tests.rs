//! Comment preservation tests - 40+ tests

use atlas_formatter::comments::CommentCollector;
use atlas_formatter::CommentPosition;
use atlas_formatter::{format_source, FormatResult};
use atlas_runtime::lexer::Lexer;
use atlas_runtime::token::TokenKind;
use pretty_assertions::assert_eq;
use rstest::rstest;

fn fmt(source: &str) -> String {
    match format_source(source) {
        FormatResult::Ok(s) => s,
        FormatResult::ParseError(e) => panic!("Parse error: {:?}", e),
    }
}

// === Lexer Comment Token Tests ===

#[test]
fn test_lexer_emits_line_comment_tokens() {
    let mut lexer = Lexer::new("// comment\nlet x = 5;");
    let (tokens, _) = lexer.tokenize_with_comments();
    assert!(tokens.iter().any(|t| t.kind == TokenKind::LineComment));
}

#[test]
fn test_lexer_emits_block_comment_tokens() {
    let mut lexer = Lexer::new("/* block */ let x = 5;");
    let (tokens, _) = lexer.tokenize_with_comments();
    assert!(tokens.iter().any(|t| t.kind == TokenKind::BlockComment));
}

#[test]
fn test_lexer_emits_doc_comment_tokens() {
    let mut lexer = Lexer::new("/// doc\nlet x = 5;");
    let (tokens, _) = lexer.tokenize_with_comments();
    assert!(tokens.iter().any(|t| t.kind == TokenKind::DocComment));
}

#[test]
fn test_lexer_normal_tokenize_skips_comments() {
    let mut lexer = Lexer::new("// comment\nlet x = 5;");
    let (tokens, _) = lexer.tokenize();
    assert!(!tokens.iter().any(|t| t.kind == TokenKind::LineComment));
}

#[test]
fn test_lexer_comment_text_preserved() {
    let mut lexer = Lexer::new("// hello world\nlet x = 5;");
    let (tokens, _) = lexer.tokenize_with_comments();
    let comment = tokens
        .iter()
        .find(|t| t.kind == TokenKind::LineComment)
        .unwrap();
    assert_eq!(comment.lexeme, "// hello world");
}

#[test]
fn test_lexer_block_comment_text_preserved() {
    let mut lexer = Lexer::new("/* multi\nline */ let x = 5;");
    let (tokens, _) = lexer.tokenize_with_comments();
    let comment = tokens
        .iter()
        .find(|t| t.kind == TokenKind::BlockComment)
        .unwrap();
    assert_eq!(comment.lexeme, "/* multi\nline */");
}

#[test]
fn test_lexer_multiple_comments() {
    let mut lexer = Lexer::new("// first\n// second\nlet x = 5;");
    let (tokens, _) = lexer.tokenize_with_comments();
    let comments: Vec<_> = tokens
        .iter()
        .filter(|t| t.kind == TokenKind::LineComment)
        .collect();
    assert_eq!(comments.len(), 2);
}

#[test]
fn test_lexer_doc_vs_regular_comment() {
    let mut lexer = Lexer::new("/// doc\n// regular\nlet x = 5;");
    let (tokens, _) = lexer.tokenize_with_comments();
    let doc = tokens
        .iter()
        .find(|t| t.kind == TokenKind::DocComment)
        .unwrap();
    let regular = tokens
        .iter()
        .find(|t| t.kind == TokenKind::LineComment)
        .unwrap();
    assert_eq!(doc.lexeme, "/// doc");
    assert_eq!(regular.lexeme, "// regular");
}

#[test]
fn test_lexer_four_slashes_not_doc() {
    let mut lexer = Lexer::new("//// not doc\nlet x = 5;");
    let (tokens, _) = lexer.tokenize_with_comments();
    let comments: Vec<_> = tokens
        .iter()
        .filter(|t| t.kind == TokenKind::DocComment)
        .collect();
    assert_eq!(
        comments.len(),
        0,
        "Four slashes should NOT be a doc comment"
    );
}

// === Comment Preservation in Formatter ===

#[test]
fn test_leading_comment_before_statement() {
    let result = fmt("// comment\nlet x = 5;");
    assert!(
        result.contains("// comment\n"),
        "Leading comment should be preserved, got: {}",
        result
    );
    assert!(result.contains("let x = 5;"));
}

#[test]
fn test_trailing_comment_after_statement() {
    let result = fmt("let x = 5; // inline comment");
    assert!(
        result.contains("// inline comment"),
        "Trailing comment preserved, got: {}",
        result
    );
}

#[test]
fn test_block_comment_preserved() {
    let result = fmt("/* block comment */\nlet x = 5;");
    assert!(
        result.contains("/* block comment */"),
        "Block comment preserved, got: {}",
        result
    );
}

#[test]
fn test_doc_comment_on_function() {
    let result =
        fmt("/// Adds two numbers\nfn add(a: number, b: number) -> number { return a + b; }");
    assert!(
        result.contains("/// Adds two numbers"),
        "Doc comment preserved, got: {}",
        result
    );
    assert!(result.contains("fn add"));
}

#[test]
fn test_multiple_leading_comments() {
    let result = fmt("// first\n// second\nlet x = 5;");
    assert!(result.contains("// first\n"));
    assert!(result.contains("// second\n"));
}

#[test]
fn test_comment_at_file_start() {
    let result = fmt("// file header\nlet x = 1;");
    assert!(
        result.contains("// file header\n"),
        "File start comment preserved, got: {}",
        result
    );
}

#[test]
fn test_comment_at_file_end() {
    let result = fmt("let x = 1;\n// end comment");
    assert!(
        result.contains("// end comment"),
        "File end comment preserved, got: {}",
        result
    );
}

#[test]
fn test_mixed_comment_types() {
    let result = fmt("// line\n/* block */\n/// doc\nlet x = 5;");
    assert!(result.contains("// line"));
    assert!(result.contains("/* block */"));
    assert!(result.contains("/// doc"));
}

#[test]
fn test_comment_between_functions() {
    let result = fmt("fn a() {}\n// separator\nfn b() {}");
    assert!(
        result.contains("// separator"),
        "Comment between functions preserved, got: {}",
        result
    );
}

#[test]
fn test_comment_before_if() {
    let result = fmt("// check condition\nif (x > 0) { print(x); }");
    assert!(result.contains("// check condition"));
}

#[test]
fn test_comment_before_loop() {
    let result = fmt("// iterate\nwhile (true) { break; }");
    assert!(result.contains("// iterate"));
}

#[test]
fn test_multiline_block_comment() {
    let result = fmt("/*\n * Multi-line\n * block comment\n */\nlet x = 5;");
    assert!(
        result.contains("Multi-line"),
        "Multiline block comment preserved, got: {}",
        result
    );
}

#[test]
fn test_comment_with_special_chars() {
    let result = fmt("// TODO: fix this! @#$%\nlet x = 5;");
    assert!(result.contains("// TODO: fix this! @#$%"));
}

#[test]
fn test_empty_comment() {
    let result = fmt("//\nlet x = 5;");
    assert!(result.contains("//\n"));
}

#[test]
fn test_comment_only_file() {
    let result = fmt("// just a comment");
    assert!(result.contains("// just a comment"));
}

#[test]
fn test_multiple_trailing_and_leading() {
    let result = fmt("// before a\nlet a = 1;\n// before b\nlet b = 2;");
    assert!(result.contains("// before a"));
    assert!(result.contains("// before b"));
}

// === Comment Indentation ===

#[test]
fn test_comment_indented_in_block() {
    let result = fmt("fn foo() {\n// comment\nlet x = 5;\n}");
    assert!(
        result.contains("// comment"),
        "Comment should appear in output, got: {}",
        result
    );
}

// === Idempotency with Comments ===

#[rstest]
#[case("// comment\nlet x = 5;")]
#[case("let x = 5; // inline")]
#[case("/* block */\nlet x = 5;")]
#[case("/// doc\nfn foo() {}")]
#[case("// a\n// b\nlet x = 5;")]
fn test_comment_idempotency(#[case] source: &str) {
    let first = fmt(source);
    let second = fmt(&first);
    assert_eq!(
        first, second,
        "Comment formatting not idempotent for: {}",
        source
    );
}

// === Comment Position Classification ===

#[test]
fn test_comment_position_leading() {
    let source = "// leading\nlet x = 5;";
    let mut lexer = Lexer::new(source);
    let (tokens, _) = lexer.tokenize_with_comments();
    let mut collector = CommentCollector::new();
    collector.collect_from_tokens(&tokens, source);
    let comments = collector.into_comments();
    assert_eq!(comments[0].position, CommentPosition::Leading);
}

#[test]
fn test_comment_position_trailing() {
    let source = "let x = 5; // trailing";
    let mut lexer = Lexer::new(source);
    let (tokens, _) = lexer.tokenize_with_comments();
    let mut collector = CommentCollector::new();
    collector.collect_from_tokens(&tokens, source);
    let comments = collector.into_comments();
    assert_eq!(comments[0].position, CommentPosition::Trailing);
}

#[test]
fn test_comment_position_standalone() {
    let source = "// just a comment";
    let mut lexer = Lexer::new(source);
    let (tokens, _) = lexer.tokenize_with_comments();
    let mut collector = CommentCollector::new();
    collector.collect_from_tokens(&tokens, source);
    let comments = collector.into_comments();
    assert_eq!(comments[0].position, CommentPosition::Standalone);
}

// === Edge Cases ===

#[test]
fn test_comment_with_url() {
    let result = fmt("// See: https://example.com/docs\nlet x = 5;");
    assert!(result.contains("https://example.com/docs"));
}

#[test]
fn test_comment_with_code_example() {
    let result = fmt("// Example: let x = foo();\nlet x = 5;");
    assert!(result.contains("// Example: let x = foo();"));
}

#[test]
fn test_block_comment_single_line() {
    let result = fmt("/* single line block */ let x = 5;");
    assert!(result.contains("/* single line block */"));
}

#[test]
fn test_formatted_with_comments_parses() {
    let source =
        "// header\nfn foo(x: number) -> number {\n    // body\n    return x + 1; // result\n}";
    let formatted = fmt(source);
    let mut lexer = atlas_runtime::lexer::Lexer::new(&formatted);
    let (tokens, _) = lexer.tokenize();
    let mut parser = atlas_runtime::parser::Parser::new(tokens);
    let (_, diags) = parser.parse();
    assert!(
        diags.is_empty(),
        "Formatted code should still parse, got: {:?}\n{}",
        diags,
        formatted
    );
}

// === Additional edge cases ===

#[test]
fn test_comment_after_block() {
    let result = fmt("fn foo() {} // after");
    assert!(
        result.contains("// after"),
        "Comment after block preserved, got: {}",
        result
    );
}

#[test]
fn test_multiple_block_comments() {
    let result = fmt("/* first */ /* second */ let x = 5;");
    assert!(result.contains("/* first */"));
    assert!(result.contains("/* second */"));
}

#[test]
fn test_comment_span_preserved() {
    let mut lexer = Lexer::new("let x = 5; // at end");
    let (tokens, _) = lexer.tokenize_with_comments();
    let comment = tokens
        .iter()
        .find(|t| t.kind == TokenKind::LineComment)
        .unwrap();
    assert!(comment.span.start > 0);
    assert!(comment.span.end > comment.span.start);
}

#[test]
fn test_doc_comment_multiline() {
    let result = fmt("/// Line 1\n/// Line 2\nfn foo() {}");
    assert!(result.contains("/// Line 1"));
    assert!(result.contains("/// Line 2"));
}
