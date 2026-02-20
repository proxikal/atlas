//! Document and workspace symbol tests
//!
//! Tests for LSP symbol functionality including:
//! - Document symbol extraction
//! - Symbol hierarchy
//! - Workspace symbol search
//! - Fuzzy matching

use atlas_lsp::symbols::{
    extract_document_symbols, offset_to_position, position_to_offset, span_to_range, WorkspaceIndex,
};
use atlas_runtime::{Lexer, Parser};
use tower_lsp::lsp_types::{Position, SymbolKind, Url};

/// Parse source and get AST for testing
fn parse_source(source: &str) -> atlas_runtime::ast::Program {
    let mut lexer = Lexer::new(source);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (ast, _) = parser.parse();
    ast
}

// === Document Symbol Tests ===

#[test]
fn test_extract_function_symbol() {
    let source = "fn greet() {}";
    let ast = parse_source(source);
    let symbols = extract_document_symbols(source, &ast);

    assert_eq!(symbols.len(), 1);
    assert_eq!(symbols[0].name, "greet");
    assert_eq!(symbols[0].kind, SymbolKind::FUNCTION);
}

#[test]
fn test_extract_multiple_functions() {
    let source = "fn foo() {}\nfn bar() {}\nfn baz() {}";
    let ast = parse_source(source);
    let symbols = extract_document_symbols(source, &ast);

    assert_eq!(symbols.len(), 3);
    let names: Vec<_> = symbols.iter().map(|s| s.name.as_str()).collect();
    assert!(names.contains(&"foo"));
    assert!(names.contains(&"bar"));
    assert!(names.contains(&"baz"));
}

#[test]
fn test_extract_variable_symbol() {
    let source = "let x = 42;";
    let ast = parse_source(source);
    let symbols = extract_document_symbols(source, &ast);

    assert_eq!(symbols.len(), 1);
    assert_eq!(symbols[0].name, "x");
    assert_eq!(symbols[0].kind, SymbolKind::CONSTANT);
}

#[test]
fn test_extract_mutable_variable_symbol() {
    let source = "var y = 42;";
    let ast = parse_source(source);
    let symbols = extract_document_symbols(source, &ast);

    assert_eq!(symbols.len(), 1);
    assert_eq!(symbols[0].name, "y");
    assert_eq!(symbols[0].kind, SymbolKind::VARIABLE);
}

#[test]
fn test_extract_type_alias_symbol() {
    let source = "type MyInt = number;";
    let ast = parse_source(source);
    let symbols = extract_document_symbols(source, &ast);

    assert_eq!(symbols.len(), 1);
    assert_eq!(symbols[0].name, "MyInt");
    assert_eq!(symbols[0].kind, SymbolKind::TYPE_PARAMETER);
}

#[test]
fn test_function_with_parameters() {
    let source = "fn add(a: number, b: number) -> number { return a + b; }";
    let ast = parse_source(source);
    let symbols = extract_document_symbols(source, &ast);

    assert_eq!(symbols.len(), 1);
    let func = &symbols[0];
    assert_eq!(func.name, "add");

    // Check that parameters are children
    assert!(func.children.is_some());
    let children = func.children.as_ref().unwrap();
    assert!(children.len() >= 2);

    let param_names: Vec<_> = children.iter().map(|c| c.name.as_str()).collect();
    assert!(param_names.contains(&"a"));
    assert!(param_names.contains(&"b"));
}

#[test]
fn test_nested_function() {
    let source = "fn outer() { fn inner() {} }";
    let ast = parse_source(source);
    let symbols = extract_document_symbols(source, &ast);

    assert_eq!(symbols.len(), 1);
    assert_eq!(symbols[0].name, "outer");

    // Inner function should be a child
    let children = symbols[0].children.as_ref();
    assert!(children.is_some());
    let inner_names: Vec<_> = children.unwrap().iter().map(|c| c.name.as_str()).collect();
    assert!(inner_names.contains(&"inner"));
}

#[test]
fn test_symbol_ranges_accurate() {
    let source = "fn test() {}";
    let ast = parse_source(source);
    let symbols = extract_document_symbols(source, &ast);

    assert_eq!(symbols.len(), 1);
    let range = symbols[0].range;

    // Range should span the whole function
    assert_eq!(range.start.line, 0);
    assert_eq!(range.start.character, 0);
}

#[test]
fn test_symbol_selection_range() {
    let source = "fn myFunction() {}";
    let ast = parse_source(source);
    let symbols = extract_document_symbols(source, &ast);

    assert_eq!(symbols.len(), 1);
    let selection = symbols[0].selection_range;

    // Selection range should be the function name
    assert_eq!(selection.start.line, 0);
    assert!(selection.start.character >= 3); // After "fn "
}

#[test]
fn test_empty_document() {
    let source = "";
    let ast = parse_source(source);
    let symbols = extract_document_symbols(source, &ast);

    assert!(symbols.is_empty());
}

#[test]
fn test_variables_inside_function() {
    let source = "fn test() { let x = 1; let y = 2; }";
    let ast = parse_source(source);
    let symbols = extract_document_symbols(source, &ast);

    assert_eq!(symbols.len(), 1);
    let children = symbols[0].children.as_ref().unwrap();

    let var_names: Vec<_> = children.iter().map(|c| c.name.as_str()).collect();
    assert!(var_names.contains(&"x"));
    assert!(var_names.contains(&"y"));
}

#[test]
fn test_function_signature_in_detail() {
    let source = "fn add(a: number, b: number) -> number { return a + b; }";
    let ast = parse_source(source);
    let symbols = extract_document_symbols(source, &ast);

    assert_eq!(symbols.len(), 1);
    let detail = symbols[0].detail.as_ref().unwrap();

    assert!(detail.contains("add"));
    assert!(detail.contains("number"));
}

// === Position/Range Conversion Tests ===

#[test]
fn test_offset_to_position_start() {
    let text = "hello";
    let pos = offset_to_position(text, 0);
    assert_eq!(pos.line, 0);
    assert_eq!(pos.character, 0);
}

#[test]
fn test_offset_to_position_middle() {
    let text = "hello";
    let pos = offset_to_position(text, 3);
    assert_eq!(pos.line, 0);
    assert_eq!(pos.character, 3);
}

#[test]
fn test_offset_to_position_multiline() {
    let text = "line1\nline2";
    let pos = offset_to_position(text, 7); // "l" in line2
    assert_eq!(pos.line, 1);
    assert_eq!(pos.character, 1);
}

#[test]
fn test_position_to_offset_start() {
    let text = "hello";
    let offset = position_to_offset(
        text,
        Position {
            line: 0,
            character: 0,
        },
    );
    assert_eq!(offset, 0);
}

#[test]
fn test_position_to_offset_middle() {
    let text = "hello";
    let offset = position_to_offset(
        text,
        Position {
            line: 0,
            character: 3,
        },
    );
    assert_eq!(offset, 3);
}

#[test]
fn test_position_to_offset_multiline() {
    let text = "line1\nline2";
    let offset = position_to_offset(
        text,
        Position {
            line: 1,
            character: 1,
        },
    );
    assert_eq!(offset, 7);
}

#[test]
fn test_span_to_range_single_line() {
    let text = "let x = 42;";
    let span = atlas_runtime::span::Span::new(0, 11);
    let range = span_to_range(text, span);

    assert_eq!(range.start.line, 0);
    assert_eq!(range.start.character, 0);
    assert_eq!(range.end.line, 0);
    assert_eq!(range.end.character, 11);
}

// === Workspace Symbol Tests ===

#[test]
fn test_workspace_index_empty() {
    let index = WorkspaceIndex::new();
    assert_eq!(index.total_symbols(), 0);
}

#[test]
fn test_workspace_index_add_document() {
    let mut index = WorkspaceIndex::new();
    let uri = Url::parse("file:///test.atlas").unwrap();
    let source = "fn foo() {}";
    let ast = parse_source(source);

    index.index_document(uri.clone(), source, &ast);
    assert!(index.symbol_count(&uri) > 0);
}

#[test]
fn test_workspace_search_exact() {
    let mut index = WorkspaceIndex::new();
    let uri = Url::parse("file:///test.atlas").unwrap();
    let source = "fn myFunction() {}";
    let ast = parse_source(source);

    index.index_document(uri, source, &ast);
    let results = index.search("myFunction", 100, None);

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "myFunction");
}

#[test]
fn test_workspace_search_prefix() {
    let mut index = WorkspaceIndex::new();
    let uri = Url::parse("file:///test.atlas").unwrap();
    let source = "fn myFunction() {}";
    let ast = parse_source(source);

    index.index_document(uri, source, &ast);
    let results = index.search("my", 100, None);

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "myFunction");
}

#[test]
fn test_workspace_search_substring() {
    let mut index = WorkspaceIndex::new();
    let uri = Url::parse("file:///test.atlas").unwrap();
    let source = "fn myFunction() {}";
    let ast = parse_source(source);

    index.index_document(uri, source, &ast);
    let results = index.search("func", 100, None);

    assert_eq!(results.len(), 1);
}

#[test]
fn test_workspace_search_camel_case() {
    let mut index = WorkspaceIndex::new();
    let uri = Url::parse("file:///test.atlas").unwrap();
    let source = "fn myFunctionName() {}";
    let ast = parse_source(source);

    index.index_document(uri, source, &ast);
    let results = index.search("mfn", 100, None);

    assert_eq!(results.len(), 1);
}

#[test]
fn test_workspace_search_case_insensitive() {
    let mut index = WorkspaceIndex::new();
    let uri = Url::parse("file:///test.atlas").unwrap();
    let source = "fn MyFunction() {}";
    let ast = parse_source(source);

    index.index_document(uri, source, &ast);
    let results = index.search("myfunction", 100, None);

    assert_eq!(results.len(), 1);
}

#[test]
fn test_workspace_search_empty_query() {
    let mut index = WorkspaceIndex::new();
    let uri = Url::parse("file:///test.atlas").unwrap();
    let source = "fn foo() {}\nfn bar() {}";
    let ast = parse_source(source);

    index.index_document(uri, source, &ast);
    let results = index.search("", 100, None);

    assert_eq!(results.len(), 2);
}

#[test]
fn test_workspace_search_no_match() {
    let mut index = WorkspaceIndex::new();
    let uri = Url::parse("file:///test.atlas").unwrap();
    let source = "fn foo() {}";
    let ast = parse_source(source);

    index.index_document(uri, source, &ast);
    let results = index.search("xyz", 100, None);

    assert!(results.is_empty());
}

#[test]
fn test_workspace_search_limit() {
    let mut index = WorkspaceIndex::new();
    let uri = Url::parse("file:///test.atlas").unwrap();
    let source = "fn a() {}\nfn ab() {}\nfn abc() {}";
    let ast = parse_source(source);

    index.index_document(uri, source, &ast);
    let results = index.search("a", 2, None);

    assert_eq!(results.len(), 2);
}

#[test]
fn test_workspace_remove_document() {
    let mut index = WorkspaceIndex::new();
    let uri = Url::parse("file:///test.atlas").unwrap();
    let source = "fn foo() {}";
    let ast = parse_source(source);

    index.index_document(uri.clone(), source, &ast);
    assert!(index.symbol_count(&uri) > 0);

    index.remove_document(&uri);
    assert_eq!(index.symbol_count(&uri), 0);
}

#[test]
fn test_workspace_multiple_files() {
    let mut index = WorkspaceIndex::new();

    let uri1 = Url::parse("file:///file1.atlas").unwrap();
    let source1 = "fn foo() {}";
    let ast1 = parse_source(source1);
    index.index_document(uri1, source1, &ast1);

    let uri2 = Url::parse("file:///file2.atlas").unwrap();
    let source2 = "fn bar() {}";
    let ast2 = parse_source(source2);
    index.index_document(uri2, source2, &ast2);

    let results = index.search("", 100, None);
    assert_eq!(results.len(), 2);
}

#[test]
fn test_workspace_search_ranking() {
    let mut index = WorkspaceIndex::new();
    let uri = Url::parse("file:///test.atlas").unwrap();
    let source = "fn fooBar() {}\nfn barFoo() {}";
    let ast = parse_source(source);

    index.index_document(uri, source, &ast);
    let results = index.search("foo", 100, None);

    // fooBar should come first (prefix match)
    assert_eq!(results[0].name, "fooBar");
}
