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
fn parse_source(
    source: &str,
) -> (
    Option<atlas_runtime::ast::Program>,
    Option<atlas_runtime::symbol::SymbolTable>,
) {
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
    let pos = Position {
        line: 0,
        character: 5,
    };
    assert_eq!(
        find_identifier_at_position(text, pos),
        Some("foo".to_string())
    );
}

#[test]
fn test_find_identifier_at_start() {
    let text = "foo bar";
    let pos = Position {
        line: 0,
        character: 0,
    };
    assert_eq!(
        find_identifier_at_position(text, pos),
        Some("foo".to_string())
    );
}

#[test]
fn test_find_identifier_at_end_of_word() {
    let text = "foo bar";
    let pos = Position {
        line: 0,
        character: 2,
    };
    assert_eq!(
        find_identifier_at_position(text, pos),
        Some("foo".to_string())
    );
}

#[test]
fn test_find_identifier_second_word() {
    let text = "foo bar";
    let pos = Position {
        line: 0,
        character: 5,
    };
    assert_eq!(
        find_identifier_at_position(text, pos),
        Some("bar".to_string())
    );
}

#[test]
fn test_find_identifier_multiline() {
    let text = "let x = 1;\nlet y = 2;";
    let pos = Position {
        line: 1,
        character: 4,
    };
    assert_eq!(
        find_identifier_at_position(text, pos),
        Some("y".to_string())
    );
}

#[test]
fn test_find_identifier_with_underscore() {
    let text = "let my_var = 1;";
    let pos = Position {
        line: 0,
        character: 6,
    };
    assert_eq!(
        find_identifier_at_position(text, pos),
        Some("my_var".to_string())
    );
}

#[test]
fn test_find_identifier_with_numbers() {
    let text = "let var123 = 1;";
    let pos = Position {
        line: 0,
        character: 6,
    };
    assert_eq!(
        find_identifier_at_position(text, pos),
        Some("var123".to_string())
    );
}

#[test]
fn test_find_identifier_on_operator_returns_none() {
    let text = "x + y";
    let pos = Position {
        line: 0,
        character: 2,
    };
    assert_eq!(find_identifier_at_position(text, pos), None);
}

#[test]
fn test_find_identifier_on_semicolon() {
    // Position on ';' which is adjacent to 'x' - finds 'x' since semicolon isn't alphanumeric
    let text = "let x;";
    let pos = Position {
        line: 0,
        character: 5,
    };
    // Since ';' is not alphanumeric, it acts as boundary and we get 'x'
    // This is expected behavior - cursor at end of identifier
    assert_eq!(
        find_identifier_at_position(text, pos),
        Some("x".to_string())
    );
}

#[test]
fn test_find_identifier_past_line_end_returns_none() {
    let text = "foo";
    let pos = Position {
        line: 0,
        character: 10,
    };
    assert_eq!(find_identifier_at_position(text, pos), None);
}

#[test]
fn test_find_identifier_invalid_line_returns_none() {
    let text = "foo";
    let pos = Position {
        line: 5,
        character: 0,
    };
    assert_eq!(find_identifier_at_position(text, pos), None);
}

// === Hover Generation Tests ===

#[test]
fn test_hover_on_keyword_let() {
    let text = "let x = 42;";
    let pos = Position {
        line: 0,
        character: 1,
    };
    let hover = generate_hover(text, pos, None, None);

    assert!(hover.is_some());
    let contents = format!("{:?}", hover.unwrap().contents);
    assert!(contents.contains("immutable"));
}

#[test]
fn test_hover_on_keyword_fn() {
    let text = "fn foo() {}";
    let pos = Position {
        line: 0,
        character: 1,
    };
    let hover = generate_hover(text, pos, None, None);

    assert!(hover.is_some());
    let contents = format!("{:?}", hover.unwrap().contents);
    assert!(contents.contains("function"));
}

#[test]
fn test_hover_on_keyword_if() {
    let text = "if true {}";
    let pos = Position {
        line: 0,
        character: 1,
    };
    let hover = generate_hover(text, pos, None, None);

    assert!(hover.is_some());
    let contents = format!("{:?}", hover.unwrap().contents);
    assert!(contents.contains("Conditional"));
}

#[test]
fn test_hover_on_keyword_while() {
    let text = "while true {}";
    let pos = Position {
        line: 0,
        character: 2,
    };
    let hover = generate_hover(text, pos, None, None);

    assert!(hover.is_some());
    let contents = format!("{:?}", hover.unwrap().contents);
    assert!(contents.contains("Loop"));
}

#[test]
fn test_hover_on_keyword_return() {
    let text = "fn f() { return 1; }";
    let pos = Position {
        line: 0,
        character: 10,
    };
    let hover = generate_hover(text, pos, None, None);

    assert!(hover.is_some());
    let contents = format!("{:?}", hover.unwrap().contents);
    assert!(contents.contains("Returns"));
}

#[test]
fn test_hover_on_builtin_print() {
    let text = "print(42);";
    let pos = Position {
        line: 0,
        character: 2,
    };
    let hover = generate_hover(text, pos, None, None);

    assert!(hover.is_some());
    let contents = format!("{:?}", hover.unwrap().contents);
    assert!(contents.contains("print"));
    assert!(contents.contains("builtin"));
}

#[test]
fn test_hover_on_builtin_len() {
    let text = "len(arr);";
    let pos = Position {
        line: 0,
        character: 1,
    };
    let hover = generate_hover(text, pos, None, None);

    assert!(hover.is_some());
    let contents = format!("{:?}", hover.unwrap().contents);
    assert!(contents.contains("len"));
    assert!(contents.contains("length"));
}

#[test]
fn test_hover_on_builtin_map() {
    let text = "map(arr, fn(x) { x });";
    let pos = Position {
        line: 0,
        character: 1,
    };
    let hover = generate_hover(text, pos, None, None);

    assert!(hover.is_some());
    let contents = format!("{:?}", hover.unwrap().contents);
    assert!(contents.contains("map"));
}

#[test]
fn test_hover_on_builtin_filter() {
    let text = "filter(arr, fn(x) { x > 0 });";
    let pos = Position {
        line: 0,
        character: 2,
    };
    let hover = generate_hover(text, pos, None, None);

    assert!(hover.is_some());
    let contents = format!("{:?}", hover.unwrap().contents);
    assert!(contents.contains("filter"));
}

#[test]
fn test_hover_on_builtin_sqrt() {
    let text = "sqrt(4);";
    let pos = Position {
        line: 0,
        character: 2,
    };
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

    let pos = Position {
        line: 0,
        character: 4,
    };
    let hover = generate_hover(source, pos, ast.as_ref(), symbols.as_ref());

    assert!(hover.is_some());
    let contents = format!("{:?}", hover.unwrap().contents);
    assert!(contents.contains("greet"));
}

#[test]
fn test_hover_with_ast_variable() {
    let source = "let counter = 0;";
    let (ast, symbols) = parse_source(source);

    let pos = Position {
        line: 0,
        character: 6,
    };
    let hover = generate_hover(source, pos, ast.as_ref(), symbols.as_ref());

    assert!(hover.is_some());
    let contents = format!("{:?}", hover.unwrap().contents);
    assert!(contents.contains("counter") || contents.contains("let"));
}

#[test]
fn test_hover_includes_range() {
    let text = "print(42);";
    let pos = Position {
        line: 0,
        character: 2,
    };
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
    let pos = Position {
        line: 0,
        character: 9,
    };
    let hover = generate_hover(text, pos, None, None);

    assert!(hover.is_some());
    let contents = format!("{:?}", hover.unwrap().contents);
    assert!(contents.contains("true"));
}

#[test]
fn test_hover_on_false_keyword() {
    let text = "let b = false;";
    let pos = Position {
        line: 0,
        character: 10,
    };
    let hover = generate_hover(text, pos, None, None);

    assert!(hover.is_some());
    let contents = format!("{:?}", hover.unwrap().contents);
    assert!(contents.contains("false"));
}

#[test]
fn test_hover_on_null_keyword() {
    let text = "let n = null;";
    let pos = Position {
        line: 0,
        character: 9,
    };
    let hover = generate_hover(text, pos, None, None);

    assert!(hover.is_some());
    let contents = format!("{:?}", hover.unwrap().contents);
    assert!(contents.contains("null"));
}

#[test]
fn test_hover_uses_markdown() {
    let text = "print(42);";
    let pos = Position {
        line: 0,
        character: 2,
    };
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
    let pos = Position {
        line: 0,
        character: 5,
    };
    let hover = generate_hover(text, pos, None, None);

    // Without AST/symbols, unknown identifiers have no hover
    assert!(hover.is_none());
}

// === Ownership Annotation Hover Tests ===

#[test]
fn test_own_param_shows_in_hover() {
    let source = "fn process(own data: number) -> number { return data; }";
    let (ast, symbols) = parse_source(source);
    let pos = Position {
        line: 0,
        character: 15, // 'data' in the param list
    };
    let hover = generate_hover(source, pos, ast.as_ref(), symbols.as_ref());
    assert!(hover.is_some());
    let contents = format!("{:?}", hover.unwrap().contents);
    assert!(
        contents.contains("(own parameter) data"),
        "Expected '(own parameter) data' in: {contents}"
    );
}

#[test]
fn test_borrow_param_shows_in_hover() {
    let source = "fn process(borrow data: number) -> number { return data; }";
    let (ast, symbols) = parse_source(source);
    let pos = Position {
        line: 0,
        character: 18, // 'data' in the param list
    };
    let hover = generate_hover(source, pos, ast.as_ref(), symbols.as_ref());
    assert!(hover.is_some());
    let contents = format!("{:?}", hover.unwrap().contents);
    assert!(
        contents.contains("(borrow parameter) data"),
        "Expected '(borrow parameter) data' in: {contents}"
    );
}

#[test]
fn test_shared_param_shows_in_hover() {
    let source = "fn process(shared data: number) -> number { return data; }";
    let (ast, symbols) = parse_source(source);
    let pos = Position {
        line: 0,
        character: 18, // 'data' in the param list
    };
    let hover = generate_hover(source, pos, ast.as_ref(), symbols.as_ref());
    assert!(hover.is_some());
    let contents = format!("{:?}", hover.unwrap().contents);
    assert!(
        contents.contains("(shared parameter) data"),
        "Expected '(shared parameter) data' in: {contents}"
    );
}

#[test]
fn test_unannotated_param_hover_unchanged() {
    let source = "fn f(x: number) -> number { return x; }";
    let (ast, symbols) = parse_source(source);
    let pos = Position {
        line: 0,
        character: 5, // 'x'
    };
    let hover = generate_hover(source, pos, ast.as_ref(), symbols.as_ref());
    assert!(hover.is_some());
    let contents = format!("{:?}", hover.unwrap().contents);
    assert!(
        contents.contains("(parameter) x"),
        "Expected '(parameter) x' in: {contents}"
    );
    assert!(
        !contents.contains("(own parameter)")
            && !contents.contains("(borrow parameter)")
            && !contents.contains("(shared parameter)"),
        "Unannotated param should not show ownership prefix, got: {contents}"
    );
}

#[test]
fn test_function_signature_hover_shows_ownership() {
    let source = "fn process(own data: number) -> number { return data; }";
    let (ast, symbols) = parse_source(source);
    let pos = Position {
        line: 0,
        character: 3, // 'process' function name
    };
    let hover = generate_hover(source, pos, ast.as_ref(), symbols.as_ref());
    assert!(hover.is_some());
    let contents = format!("{:?}", hover.unwrap().contents);
    assert!(
        contents.contains("own data"),
        "Function signature should include ownership annotation, got: {contents}"
    );
}

#[test]
fn test_keyword_hover_own() {
    let source = "fn f(own x: number) -> number { return x; }";
    let pos = Position {
        line: 0,
        character: 5, // 'own' keyword
    };
    let hover = generate_hover(source, pos, None, None);
    assert!(hover.is_some());
    let contents = format!("{:?}", hover.unwrap().contents);
    assert!(
        contents.contains("own"),
        "Expected keyword hover for 'own', got: {contents}"
    );
    assert!(
        contents.contains("ownership") || contents.contains("exclusive"),
        "Expected ownership description for 'own', got: {contents}"
    );
}

#[test]
fn test_keyword_hover_borrow() {
    let hover = generate_hover(
        "borrow",
        Position {
            line: 0,
            character: 3,
        },
        None,
        None,
    );
    assert!(hover.is_some());
    let contents = format!("{:?}", hover.unwrap().contents);
    assert!(contents.contains("borrow"));
}

#[test]
fn test_keyword_hover_shared() {
    let hover = generate_hover(
        "shared",
        Position {
            line: 0,
            character: 3,
        },
        None,
        None,
    );
    assert!(hover.is_some());
    let contents = format!("{:?}", hover.unwrap().contents);
    assert!(contents.contains("shared"));
}

// === Trait/Impl Keyword Hover Tests ===

#[test]
fn test_keyword_hover_trait() {
    let hover = generate_hover(
        "trait",
        Position {
            line: 0,
            character: 2,
        },
        None,
        None,
    );
    assert!(hover.is_some(), "Expected hover for 'trait' keyword");
    let contents = format!("{:?}", hover.unwrap().contents);
    assert!(
        contents.contains("trait"),
        "Expected 'trait' in hover, got: {contents}"
    );
    assert!(
        contents.contains("Declares a trait") || contents.contains("named set"),
        "Expected trait description, got: {contents}"
    );
}

#[test]
fn test_keyword_hover_impl() {
    let hover = generate_hover(
        "impl",
        Position {
            line: 0,
            character: 2,
        },
        None,
        None,
    );
    assert!(hover.is_some(), "Expected hover for 'impl' keyword");
    let contents = format!("{:?}", hover.unwrap().contents);
    assert!(
        contents.contains("impl"),
        "Expected 'impl' in hover, got: {contents}"
    );
    assert!(
        contents.contains("Implements a trait") || contents.contains("matching signatures"),
        "Expected impl description, got: {contents}"
    );
}

#[test]
fn test_hover_trait_name_in_declaration() {
    let source = "trait Display { fn display(self: Display) -> string; }";
    let (ast, symbols) = parse_source(source);
    // Hover over 'Display' (the trait name) at character 6
    let pos = Position {
        line: 0,
        character: 8,
    };
    let hover = generate_hover(source, pos, ast.as_ref(), symbols.as_ref());
    assert!(hover.is_some(), "Expected hover for trait name 'Display'");
    let contents = format!("{:?}", hover.unwrap().contents);
    assert!(
        contents.contains("trait") && contents.contains("Display"),
        "Expected '(trait) Display' in hover, got: {contents}"
    );
    assert!(
        contents.contains("display"),
        "Expected method name 'display' in trait hover, got: {contents}"
    );
}

#[test]
fn test_hover_trait_name_shows_method_signatures() {
    let source = "trait Math { fn double(self: Math) -> number; fn triple(self: Math) -> number; }";
    let (ast, symbols) = parse_source(source);
    let pos = Position {
        line: 0,
        character: 8,
    };
    let hover = generate_hover(source, pos, ast.as_ref(), symbols.as_ref());
    assert!(hover.is_some(), "Expected hover for 'Math' trait name");
    let contents = format!("{:?}", hover.unwrap().contents);
    assert!(
        contents.contains("double") && contents.contains("triple"),
        "Expected both method signatures in trait hover, got: {contents}"
    );
}

#[test]
fn test_hover_impl_block_type_name() {
    let source = "trait Display { fn display(self: Display) -> string; } \
                  impl Display for number { fn display(self: number) -> string { return \"n\"; } }";
    let (ast, symbols) = parse_source(source);
    // Hover over 'number' in the impl block (after 'for ')
    // 'impl Display for number' — 'number' starts at col 72
    let pos = Position {
        line: 0,
        character: 74,
    };
    let hover = generate_hover(source, pos, ast.as_ref(), symbols.as_ref());
    assert!(
        hover.is_some(),
        "Expected hover for type name in impl block"
    );
    let contents = format!("{:?}", hover.unwrap().contents);
    assert!(
        contents.contains("impl") && contents.contains("number") && contents.contains("Display"),
        "Expected impl hover showing type implements trait, got: {contents}"
    );
}

#[test]
fn test_hover_impl_block_trait_name() {
    let source = "trait Display { fn display(self: Display) -> string; } \
                  impl Display for number { fn display(self: number) -> string { return \"n\"; } }";
    let (ast, symbols) = parse_source(source);
    // Hover over 'Display' in the impl block header (col 60)
    // Since Display is also a declared trait, find_trait_hover fires first → shows trait signature
    let pos = Position {
        line: 0,
        character: 60,
    };
    let hover = generate_hover(source, pos, ast.as_ref(), symbols.as_ref());
    assert!(
        hover.is_some(),
        "Expected hover for trait name in impl header"
    );
    let contents = format!("{:?}", hover.unwrap().contents);
    assert!(
        contents.contains("Display"),
        "Expected 'Display' in hover for trait name in impl header, got: {contents}"
    );
    assert!(
        contents.contains("trait") || contents.contains("display"),
        "Expected trait info for 'Display' hover, got: {contents}"
    );
}

#[test]
fn test_hover_trait_with_no_methods() {
    let source = "trait Marker { }";
    let (ast, symbols) = parse_source(source);
    let pos = Position {
        line: 0,
        character: 8,
    };
    let hover = generate_hover(source, pos, ast.as_ref(), symbols.as_ref());
    assert!(hover.is_some(), "Expected hover for trait with no methods");
    let contents = format!("{:?}", hover.unwrap().contents);
    assert!(
        contents.contains("Marker"),
        "Expected trait name in hover, got: {contents}"
    );
}

#[test]
fn test_hover_impl_method_list_shown() {
    let source =
        "trait Shape { fn area(self: Shape) -> number; fn perimeter(self: Shape) -> number; } \
                  impl Shape for number { \
                    fn area(self: number) -> number { return self; } \
                    fn perimeter(self: number) -> number { return self * 4; } \
                  }";
    let (ast, symbols) = parse_source(source);
    // Hover over 'Shape' in 'impl Shape for number'
    let pos = Position {
        line: 0,
        character: 90,
    };
    let hover = generate_hover(source, pos, ast.as_ref(), symbols.as_ref());
    assert!(
        hover.is_some(),
        "Expected hover for trait name in impl block"
    );
    let contents = format!("{:?}", hover.unwrap().contents);
    assert!(
        contents.contains("area") || contents.contains("perimeter"),
        "Expected method names in impl hover, got: {contents}"
    );
}
