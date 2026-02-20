//! Hover provider tests
//!
//! Tests for LSP hover functionality including:
//! - Type information display
//! - Documentation extraction
//! - Builtin function help
//! - Keyword descriptions

use atlas_lsp::hover::{find_identifier_at_position, generate_hover};
use atlas_runtime::{Lexer, Parser};
use tower_lsp::lsp_types::Position;

/// Parse source and get AST/symbols for testing
fn parse_source(source: &str) -> (Option<atlas_runtime::ast::Program>, Option<atlas_runtime::symbol::SymbolTable>) {
    let mut lexer = Lexer::new(source);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (ast, _) = parser.parse();

    let mut binder = atlas_runtime::Binder::new();
    let (mut symbols, _) = binder.bind(&ast);

    let mut typechecker = atlas_runtime::TypeChecker::new(&mut symbols);
    let _ = typechecker.check(&ast);

    (Some(ast), Some(symbols))
}

// === Identifier Finding Tests ===

#[test]
fn test_find_identifier_simple() {
    let text = "let foo = 42;";
    let pos = Position { line: 0, character: 5 };
    assert_eq!(find_identifier_at_position(text, pos), Some("foo".to_string()));
}

#[test]
fn test_find_identifier_at_start() {
    let text = "foo bar";
    let pos = Position { line: 0, character: 0 };
    assert_eq!(find_identifier_at_position(text, pos), Some("foo".to_string()));
}

#[test]
fn test_find_identifier_at_end_of_word() {
    let text = "foo bar";
    let pos = Position { line: 0, character: 2 };
    assert_eq!(find_identifier_at_position(text, pos), Some("foo".to_string()));
}

#[test]
fn test_find_identifier_second_word() {
    let text = "foo bar";
    let pos = Position { line: 0, character: 5 };
    assert_eq!(find_identifier_at_position(text, pos), Some("bar".to_string()));
}

#[test]
fn test_find_identifier_multiline() {
    let text = "let x = 1;\nlet y = 2;";
    let pos = Position { line: 1, character: 4 };
    assert_eq!(find_identifier_at_position(text, pos), Some("y".to_string()));
}

#[test]
fn test_find_identifier_with_underscore() {
    let text = "let my_var = 1;";
    let pos = Position { line: 0, character: 6 };
    assert_eq!(find_identifier_at_position(text, pos), Some("my_var".to_string()));
}

#[test]
fn test_find_identifier_with_numbers() {
    let text = "let var123 = 1;";
    let pos = Position { line: 0, character: 6 };
    assert_eq!(find_identifier_at_position(text, pos), Some("var123".to_string()));
}

#[test]
fn test_find_identifier_on_operator_returns_none() {
    let text = "x + y";
    let pos = Position { line: 0, character: 2 };
    assert_eq!(find_identifier_at_position(text, pos), None);
}

#[test]
fn test_find_identifier_on_semicolon() {
    // Position on ';' which is adjacent to 'x' - finds 'x' since semicolon isn't alphanumeric
    let text = "let x;";
    let pos = Position { line: 0, character: 5 };
    // Since ';' is not alphanumeric, it acts as boundary and we get 'x'
    // This is expected behavior - cursor at end of identifier
    assert_eq!(find_identifier_at_position(text, pos), Some("x".to_string()));
}

#[test]
fn test_find_identifier_past_line_end_returns_none() {
    let text = "foo";
    let pos = Position { line: 0, character: 10 };
    assert_eq!(find_identifier_at_position(text, pos), None);
}

#[test]
fn test_find_identifier_invalid_line_returns_none() {
    let text = "foo";
    let pos = Position { line: 5, character: 0 };
    assert_eq!(find_identifier_at_position(text, pos), None);
}

// === Hover Generation Tests ===

#[test]
fn test_hover_on_keyword_let() {
    let text = "let x = 42;";
    let pos = Position { line: 0, character: 1 };
    let hover = generate_hover(text, pos, None, None);

    assert!(hover.is_some());
    let contents = format!("{:?}", hover.unwrap().contents);
    assert!(contents.contains("immutable"));
}

#[test]
fn test_hover_on_keyword_fn() {
    let text = "fn foo() {}";
    let pos = Position { line: 0, character: 1 };
    let hover = generate_hover(text, pos, None, None);

    assert!(hover.is_some());
    let contents = format!("{:?}", hover.unwrap().contents);
    assert!(contents.contains("function"));
}

#[test]
fn test_hover_on_keyword_if() {
    let text = "if true {}";
    let pos = Position { line: 0, character: 1 };
    let hover = generate_hover(text, pos, None, None);

    assert!(hover.is_some());
    let contents = format!("{:?}", hover.unwrap().contents);
    assert!(contents.contains("Conditional"));
}

#[test]
fn test_hover_on_keyword_while() {
    let text = "while true {}";
    let pos = Position { line: 0, character: 2 };
    let hover = generate_hover(text, pos, None, None);

    assert!(hover.is_some());
    let contents = format!("{:?}", hover.unwrap().contents);
    assert!(contents.contains("Loop"));
}

#[test]
fn test_hover_on_keyword_return() {
    let text = "fn f() { return 1; }";
    let pos = Position { line: 0, character: 10 };
    let hover = generate_hover(text, pos, None, None);

    assert!(hover.is_some());
    let contents = format!("{:?}", hover.unwrap().contents);
    assert!(contents.contains("Returns"));
}

#[test]
fn test_hover_on_builtin_print() {
    let text = "print(42);";
    let pos = Position { line: 0, character: 2 };
    let hover = generate_hover(text, pos, None, None);

    assert!(hover.is_some());
    let contents = format!("{:?}", hover.unwrap().contents);
    assert!(contents.contains("print"));
    assert!(contents.contains("builtin"));
}

#[test]
fn test_hover_on_builtin_len() {
    let text = "len(arr);";
    let pos = Position { line: 0, character: 1 };
    let hover = generate_hover(text, pos, None, None);

    assert!(hover.is_some());
    let contents = format!("{:?}", hover.unwrap().contents);
    assert!(contents.contains("len"));
    assert!(contents.contains("length"));
}

#[test]
fn test_hover_on_builtin_map() {
    let text = "map(arr, fn(x) { x });";
    let pos = Position { line: 0, character: 1 };
    let hover = generate_hover(text, pos, None, None);

    assert!(hover.is_some());
    let contents = format!("{:?}", hover.unwrap().contents);
    assert!(contents.contains("map"));
}

#[test]
fn test_hover_on_builtin_filter() {
    let text = "filter(arr, fn(x) { x > 0 });";
    let pos = Position { line: 0, character: 2 };
    let hover = generate_hover(text, pos, None, None);

    assert!(hover.is_some());
    let contents = format!("{:?}", hover.unwrap().contents);
    assert!(contents.contains("filter"));
}

#[test]
fn test_hover_on_builtin_sqrt() {
    let text = "sqrt(4);";
    let pos = Position { line: 0, character: 2 };
    let hover = generate_hover(text, pos, None, None);

    assert!(hover.is_some());
    let contents = format!("{:?}", hover.unwrap().contents);
    assert!(contents.contains("sqrt"));
    assert!(contents.contains("square root"));
}

#[test]
fn test_hover_with_ast_function() {
    let source = "fn greet(name: string) -> string { return name; }";
    let (ast, symbols) = parse_source(source);

    let pos = Position { line: 0, character: 4 };
    let hover = generate_hover(source, pos, ast.as_ref(), symbols.as_ref());

    assert!(hover.is_some());
    let contents = format!("{:?}", hover.unwrap().contents);
    assert!(contents.contains("greet"));
}

#[test]
fn test_hover_with_ast_variable() {
    let source = "let counter = 0;";
    let (ast, symbols) = parse_source(source);

    let pos = Position { line: 0, character: 6 };
    let hover = generate_hover(source, pos, ast.as_ref(), symbols.as_ref());

    assert!(hover.is_some());
    let contents = format!("{:?}", hover.unwrap().contents);
    assert!(contents.contains("counter") || contents.contains("let"));
}

#[test]
fn test_hover_includes_range() {
    let text = "print(42);";
    let pos = Position { line: 0, character: 2 };
    let hover = generate_hover(text, pos, None, None);

    assert!(hover.is_some());
    let h = hover.unwrap();
    assert!(h.range.is_some());

    let range = h.range.unwrap();
    assert_eq!(range.start.line, 0);
    assert_eq!(range.start.character, 0);
    assert_eq!(range.end.character, 5); // "print" is 5 chars
}

#[test]
fn test_hover_on_true_keyword() {
    let text = "let b = true;";
    let pos = Position { line: 0, character: 9 };
    let hover = generate_hover(text, pos, None, None);

    assert!(hover.is_some());
    let contents = format!("{:?}", hover.unwrap().contents);
    assert!(contents.contains("true"));
}

#[test]
fn test_hover_on_false_keyword() {
    let text = "let b = false;";
    let pos = Position { line: 0, character: 10 };
    let hover = generate_hover(text, pos, None, None);

    assert!(hover.is_some());
    let contents = format!("{:?}", hover.unwrap().contents);
    assert!(contents.contains("false"));
}

#[test]
fn test_hover_on_null_keyword() {
    let text = "let n = null;";
    let pos = Position { line: 0, character: 9 };
    let hover = generate_hover(text, pos, None, None);

    assert!(hover.is_some());
    let contents = format!("{:?}", hover.unwrap().contents);
    assert!(contents.contains("null"));
}

#[test]
fn test_hover_uses_markdown() {
    let text = "print(42);";
    let pos = Position { line: 0, character: 2 };
    let hover = generate_hover(text, pos, None, None);

    assert!(hover.is_some());
    if let tower_lsp::lsp_types::HoverContents::Markup(markup) = hover.unwrap().contents {
        assert_eq!(markup.kind, tower_lsp::lsp_types::MarkupKind::Markdown);
    } else {
        panic!("Expected Markup contents");
    }
}

#[test]
fn test_hover_on_unknown_identifier_returns_none() {
    let text = "foo_unknown_var;";
    let pos = Position { line: 0, character: 5 };
    let hover = generate_hover(text, pos, None, None);

    // Without AST/symbols, unknown identifiers have no hover
    assert!(hover.is_none());
}
