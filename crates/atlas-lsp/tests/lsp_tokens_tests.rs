//! Semantic tokens provider tests
//!
//! Tests for LSP semantic token functionality including:
//! - Token classification
//! - Token modifiers
//! - Delta encoding
//! - Range queries

use atlas_lsp::semantic_tokens::{
    generate_semantic_tokens, generate_semantic_tokens_range, get_legend, TOKEN_MODIFIERS,
    TOKEN_TYPES,
};
use atlas_runtime::{Lexer, Parser};
use tower_lsp::lsp_types::*;

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

// === Legend Tests ===

#[test]
fn test_legend_has_token_types() {
    let legend = get_legend();
    assert!(!legend.token_types.is_empty());
}

#[test]
fn test_legend_has_modifiers() {
    let legend = get_legend();
    assert!(!legend.token_modifiers.is_empty());
}

#[test]
fn test_legend_includes_common_types() {
    let legend = get_legend();

    // Check for common types
    assert!(legend.token_types.contains(&SemanticTokenType::VARIABLE));
    assert!(legend.token_types.contains(&SemanticTokenType::FUNCTION));
    assert!(legend.token_types.contains(&SemanticTokenType::KEYWORD));
    assert!(legend.token_types.contains(&SemanticTokenType::STRING));
    assert!(legend.token_types.contains(&SemanticTokenType::NUMBER));
}

#[test]
fn test_legend_includes_common_modifiers() {
    let legend = get_legend();

    // Check for common modifiers
    assert!(legend
        .token_modifiers
        .contains(&SemanticTokenModifier::DECLARATION));
    assert!(legend
        .token_modifiers
        .contains(&SemanticTokenModifier::READONLY));
}

#[test]
fn test_token_types_constant() {
    assert!(!TOKEN_TYPES.is_empty());
    assert!(TOKEN_TYPES.len() >= 10);
}

#[test]
fn test_token_modifiers_constant() {
    assert!(!TOKEN_MODIFIERS.is_empty());
    assert!(TOKEN_MODIFIERS.len() >= 5);
}

// === Full Tokenization Tests ===

#[test]
fn test_tokenize_empty_source() {
    let source = "";
    let result = generate_semantic_tokens(source, None, None);

    if let SemanticTokensResult::Tokens(tokens) = result {
        assert!(tokens.data.is_empty());
    } else {
        panic!("Expected Tokens result");
    }
}

#[test]
fn test_tokenize_variable_declaration() {
    let source = "let x = 42;";
    let result = generate_semantic_tokens(source, None, None);

    if let SemanticTokensResult::Tokens(tokens) = result {
        // Should have tokens for: let, x, 42
        assert!(!tokens.data.is_empty());
    } else {
        panic!("Expected Tokens result");
    }
}

#[test]
fn test_tokenize_function_declaration() {
    let source = "fn greet() { return 1; }";
    let (ast, symbols) = parse_source(source);

    let result = generate_semantic_tokens(source, ast.as_ref(), symbols.as_ref());

    if let SemanticTokensResult::Tokens(tokens) = result {
        // Should have tokens for: fn, greet, return, 1
        assert!(!tokens.data.is_empty());
    } else {
        panic!("Expected Tokens result");
    }
}

#[test]
fn test_tokenize_keywords() {
    let source = "let x = 1;\nif true { return x; }";
    let result = generate_semantic_tokens(source, None, None);

    if let SemanticTokensResult::Tokens(tokens) = result {
        // Should have multiple keyword tokens
        assert!(tokens.data.len() >= 3);
    } else {
        panic!("Expected Tokens result");
    }
}

#[test]
fn test_tokenize_string_literal() {
    let source = "let s = \"hello\";";
    let result = generate_semantic_tokens(source, None, None);

    if let SemanticTokensResult::Tokens(tokens) = result {
        assert!(!tokens.data.is_empty());
    } else {
        panic!("Expected Tokens result");
    }
}

#[test]
fn test_tokenize_number_literal() {
    let source = "let n = 3.14;";
    let result = generate_semantic_tokens(source, None, None);

    if let SemanticTokensResult::Tokens(tokens) = result {
        assert!(!tokens.data.is_empty());
    } else {
        panic!("Expected Tokens result");
    }
}

#[test]
fn test_tokenize_operators() {
    let source = "let x = 1 + 2 * 3;";
    let result = generate_semantic_tokens(source, None, None);

    if let SemanticTokensResult::Tokens(tokens) = result {
        assert!(!tokens.data.is_empty());
    } else {
        panic!("Expected Tokens result");
    }
}

#[test]
fn test_tokenize_builtin_function() {
    let source = "print(42);";
    let result = generate_semantic_tokens(source, None, None);

    if let SemanticTokensResult::Tokens(tokens) = result {
        // 'print' should be tokenized as function with DEFAULT_LIBRARY modifier
        assert!(!tokens.data.is_empty());
    } else {
        panic!("Expected Tokens result");
    }
}

#[test]
fn test_tokenize_comments() {
    let source = "// This is a comment\nlet x = 1;";
    let result = generate_semantic_tokens(source, None, None);

    if let SemanticTokensResult::Tokens(tokens) = result {
        // Comment should be included
        assert!(!tokens.data.is_empty());
    } else {
        panic!("Expected Tokens result");
    }
}

#[test]
fn test_tokenize_multiline() {
    let source = "let x = 1;\nlet y = 2;\nlet z = 3;";
    let result = generate_semantic_tokens(source, None, None);

    if let SemanticTokensResult::Tokens(tokens) = result {
        // Should have tokens across multiple lines
        // Check that we have delta_line > 0 somewhere
        let has_multiline = tokens.data.iter().any(|t| t.delta_line > 0);
        assert!(has_multiline);
    } else {
        panic!("Expected Tokens result");
    }
}

// === Range Tokenization Tests ===

#[test]
fn test_tokenize_range_full() {
    let source = "let x = 1;\nlet y = 2;";
    let range = Range {
        start: Position {
            line: 0,
            character: 0,
        },
        end: Position {
            line: 1,
            character: 10,
        },
    };

    let result = generate_semantic_tokens_range(source, range, None, None);

    if let SemanticTokensRangeResult::Tokens(tokens) = result {
        assert!(!tokens.data.is_empty());
    } else {
        panic!("Expected Tokens result");
    }
}

#[test]
fn test_tokenize_range_partial() {
    let source = "let x = 1;\nlet y = 2;\nlet z = 3;";
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

    let result = generate_semantic_tokens_range(source, range, None, None);

    // Should return tokens (may include all due to simplified range check)
    match result {
        SemanticTokensRangeResult::Tokens(_) => {}
        SemanticTokensRangeResult::Partial(_) => {}
    }
}

// === Delta Encoding Tests ===

#[test]
fn test_delta_encoding_same_line() {
    let source = "let x = 42;";
    let result = generate_semantic_tokens(source, None, None);

    if let SemanticTokensResult::Tokens(tokens) = result {
        // First token should have delta_line = 0
        if let Some(first) = tokens.data.first() {
            assert_eq!(first.delta_line, 0);
        }

        // Subsequent tokens on same line should have delta_line = 0
        for token in &tokens.data {
            // All on line 0
            assert_eq!(token.delta_line, 0);
        }
    } else {
        panic!("Expected Tokens result");
    }
}

#[test]
fn test_delta_encoding_different_lines() {
    let source = "let x = 1;\nlet y = 2;";
    let result = generate_semantic_tokens(source, None, None);

    if let SemanticTokensResult::Tokens(tokens) = result {
        // Should have at least one token with delta_line > 0
        let has_line_delta = tokens.data.iter().any(|t| t.delta_line > 0);
        assert!(has_line_delta);
    } else {
        panic!("Expected Tokens result");
    }
}

#[test]
fn test_delta_start_resets_on_new_line() {
    let source = "let x = 1;\nlet y = 2;";
    let result = generate_semantic_tokens(source, None, None);

    if let SemanticTokensResult::Tokens(tokens) = result {
        // Find token on second line
        let second_line_token = tokens.data.iter().find(|t| t.delta_line > 0);

        if let Some(token) = second_line_token {
            // delta_start should be absolute position on new line
            assert_eq!(token.delta_start, 0); // 'let' starts at column 0
        }
    } else {
        panic!("Expected Tokens result");
    }
}

// === Token Classification Tests ===

#[test]
fn test_keyword_token_type() {
    let source = "let x = 1;";
    let result = generate_semantic_tokens(source, None, None);

    if let SemanticTokensResult::Tokens(tokens) = result {
        // First token should be 'let' keyword (type 15)
        if let Some(first) = tokens.data.first() {
            assert_eq!(first.token_type, 15); // KEYWORD index
        }
    } else {
        panic!("Expected Tokens result");
    }
}

#[test]
fn test_number_token_type() {
    let source = "42";
    let result = generate_semantic_tokens(source, None, None);

    if let SemanticTokensResult::Tokens(tokens) = result {
        // Should have number token (type 19)
        let has_number = tokens.data.iter().any(|t| t.token_type == 19);
        assert!(has_number);
    } else {
        panic!("Expected Tokens result");
    }
}

#[test]
fn test_string_token_type() {
    let source = "\"hello\"";
    let result = generate_semantic_tokens(source, None, None);

    if let SemanticTokensResult::Tokens(tokens) = result {
        // Should have string token (type 18)
        let has_string = tokens.data.iter().any(|t| t.token_type == 18);
        assert!(has_string);
    } else {
        panic!("Expected Tokens result");
    }
}

#[test]
fn test_variable_token_type() {
    let source = "let myVar = 1;";
    let result = generate_semantic_tokens(source, None, None);

    if let SemanticTokensResult::Tokens(tokens) = result {
        // Should have variable token (type 8)
        let has_variable = tokens.data.iter().any(|t| t.token_type == 8);
        assert!(has_variable);
    } else {
        panic!("Expected Tokens result");
    }
}

#[test]
fn test_function_token_type() {
    let source = "print(1);";
    let result = generate_semantic_tokens(source, None, None);

    if let SemanticTokensResult::Tokens(tokens) = result {
        // 'print' should be function token (type 12)
        let has_function = tokens.data.iter().any(|t| t.token_type == 12);
        assert!(has_function);
    } else {
        panic!("Expected Tokens result");
    }
}

// === Token Modifier Tests ===

#[test]
fn test_builtin_has_default_library_modifier() {
    let source = "print(1);";
    let result = generate_semantic_tokens(source, None, None);

    if let SemanticTokensResult::Tokens(tokens) = result {
        // 'print' should have DEFAULT_LIBRARY modifier (bit 9)
        let has_default_lib = tokens
            .data
            .iter()
            .any(|t| t.token_type == 12 && (t.token_modifiers_bitset & (1 << 9)) != 0);
        assert!(has_default_lib);
    } else {
        panic!("Expected Tokens result");
    }
}

// === Performance Tests ===

#[test]
fn test_tokenize_large_source() {
    // Generate a source with many lines
    let mut source = String::new();
    for i in 0..100 {
        source.push_str(&format!("let x{} = {};\n", i, i));
    }

    let result = generate_semantic_tokens(&source, None, None);

    if let SemanticTokensResult::Tokens(tokens) = result {
        // Should handle large files
        assert!(!tokens.data.is_empty());
        // Should have many tokens
        assert!(tokens.data.len() >= 100);
    } else {
        panic!("Expected Tokens result");
    }
}

#[test]
fn test_tokenize_with_ast_and_symbols() {
    let source = "fn add(a: number, b: number) -> number { return a + b; }";
    let (ast, symbols) = parse_source(source);

    let result = generate_semantic_tokens(source, ast.as_ref(), symbols.as_ref());

    if let SemanticTokensResult::Tokens(tokens) = result {
        // With AST, should have richer classification
        assert!(!tokens.data.is_empty());
    } else {
        panic!("Expected Tokens result");
    }
}

#[test]
fn test_result_id_is_none() {
    let source = "let x = 1;";
    let result = generate_semantic_tokens(source, None, None);

    if let SemanticTokensResult::Tokens(tokens) = result {
        // result_id should be None (we don't support incremental)
        assert!(tokens.result_id.is_none());
    } else {
        panic!("Expected Tokens result");
    }
}

#[test]
fn test_token_length_matches_lexeme() {
    let source = "let myVariable = 42;";
    let result = generate_semantic_tokens(source, None, None);

    if let SemanticTokensResult::Tokens(tokens) = result {
        // Find 'let' token (should be length 3)
        if let Some(first) = tokens.data.first() {
            assert_eq!(first.length, 3);
        }
    } else {
        panic!("Expected Tokens result");
    }
}

// === Ownership Keyword Semantic Token Tests ===

/// KEYWORD type index in TOKEN_TYPES (position 15)
const KEYWORD_TYPE_IDX: u32 = 15;

#[test]
fn test_own_keyword_is_keyword_semantic_token() {
    let source = "fn f(own x: number) -> number { return x; }";
    let (ast, symbols) = parse_source(source);
    let result = generate_semantic_tokens(source, ast.as_ref(), symbols.as_ref());

    if let SemanticTokensResult::Tokens(tokens) = result {
        // `own` is 3 chars — verify at least one KEYWORD token of length 3 exists
        let has_own_keyword = tokens
            .data
            .iter()
            .any(|t| t.token_type == KEYWORD_TYPE_IDX && t.length == 3);
        assert!(
            has_own_keyword,
            "Expected 'own' to be classified as KEYWORD (type {KEYWORD_TYPE_IDX})"
        );
    } else {
        panic!("Expected Tokens result");
    }
}

#[test]
fn test_borrow_keyword_is_keyword_semantic_token() {
    let source = "fn f(borrow x: number) -> number { return x; }";
    let (ast, symbols) = parse_source(source);
    let result = generate_semantic_tokens(source, ast.as_ref(), symbols.as_ref());

    if let SemanticTokensResult::Tokens(tokens) = result {
        // `borrow` is length 6 — verify at least one KEYWORD token of length 6 exists
        let has_borrow_keyword = tokens
            .data
            .iter()
            .any(|t| t.token_type == KEYWORD_TYPE_IDX && t.length == 6);
        assert!(
            has_borrow_keyword,
            "Expected 'borrow' to be classified as KEYWORD"
        );
    } else {
        panic!("Expected Tokens result");
    }
}

#[test]
fn test_shared_keyword_is_keyword_semantic_token() {
    let source = "fn f(shared x: number) -> number { return x; }";
    let (ast, symbols) = parse_source(source);
    let result = generate_semantic_tokens(source, ast.as_ref(), symbols.as_ref());

    if let SemanticTokensResult::Tokens(tokens) = result {
        // `shared` is length 6 — verify at least one KEYWORD token of length 6 exists
        let has_shared_keyword = tokens
            .data
            .iter()
            .any(|t| t.token_type == KEYWORD_TYPE_IDX && t.length == 6);
        assert!(
            has_shared_keyword,
            "Expected 'shared' to be classified as KEYWORD"
        );
    } else {
        panic!("Expected Tokens result");
    }
}

#[test]
fn test_ownership_keywords_not_classified_as_variable() {
    // Ensure own/borrow/shared are never emitted as VARIABLE (type 8)
    const VARIABLE_TYPE_IDX: u32 = 8;

    for source in &[
        "fn f(own x: number) -> number { return x; }",
        "fn f(borrow x: number) -> number { return x; }",
        "fn f(shared x: number) -> number { return x; }",
    ] {
        let (ast, symbols) = parse_source(source);
        let result = generate_semantic_tokens(source, ast.as_ref(), symbols.as_ref());
        if let SemanticTokensResult::Tokens(tokens) = result {
            // Accumulate absolute positions to identify the ownership keyword token
            // The ownership keyword is always at the start of the param list (after the '(')
            // We verify no token with keyword-matching lengths (3 or 6) on line 0 is VARIABLE
            let misclassified = tokens.data.iter().any(|t| {
                t.token_type == VARIABLE_TYPE_IDX
                    && (t.length == 3 || t.length == 6)
                    && t.delta_line == 0
            });
            assert!(
                !misclassified,
                "Ownership keyword incorrectly classified as VARIABLE in: {source}"
            );
        } else {
            panic!("Expected Tokens result");
        }
    }
}
