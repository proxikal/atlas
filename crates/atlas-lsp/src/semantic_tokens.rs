//! Semantic tokens provider for LSP
//!
//! Provides rich syntax highlighting through semantic token classification.
//! Tokens are classified by type (variable, function, keyword, etc.) and
//! modified by attributes (declaration, readonly, deprecated, etc.).

use atlas_runtime::ast::*;
use atlas_runtime::symbol::SymbolTable;
use atlas_runtime::token::{Token, TokenKind};
use atlas_runtime::Lexer;
use tower_lsp::lsp_types::*;

/// Semantic token types we support
pub const TOKEN_TYPES: &[SemanticTokenType] = &[
    SemanticTokenType::NAMESPACE,     // 0
    SemanticTokenType::TYPE,          // 1
    SemanticTokenType::CLASS,         // 2
    SemanticTokenType::ENUM,          // 3
    SemanticTokenType::INTERFACE,     // 4
    SemanticTokenType::STRUCT,        // 5
    SemanticTokenType::TYPE_PARAMETER, // 6
    SemanticTokenType::PARAMETER,     // 7
    SemanticTokenType::VARIABLE,      // 8
    SemanticTokenType::PROPERTY,      // 9
    SemanticTokenType::ENUM_MEMBER,   // 10
    SemanticTokenType::EVENT,         // 11
    SemanticTokenType::FUNCTION,      // 12
    SemanticTokenType::METHOD,        // 13
    SemanticTokenType::MACRO,         // 14
    SemanticTokenType::KEYWORD,       // 15
    SemanticTokenType::MODIFIER,      // 16
    SemanticTokenType::COMMENT,       // 17
    SemanticTokenType::STRING,        // 18
    SemanticTokenType::NUMBER,        // 19
    SemanticTokenType::REGEXP,        // 20
    SemanticTokenType::OPERATOR,      // 21
];

/// Semantic token modifiers we support
pub const TOKEN_MODIFIERS: &[SemanticTokenModifier] = &[
    SemanticTokenModifier::DECLARATION,    // 0
    SemanticTokenModifier::DEFINITION,     // 1
    SemanticTokenModifier::READONLY,       // 2
    SemanticTokenModifier::STATIC,         // 3
    SemanticTokenModifier::DEPRECATED,     // 4
    SemanticTokenModifier::ABSTRACT,       // 5
    SemanticTokenModifier::ASYNC,          // 6
    SemanticTokenModifier::MODIFICATION,   // 7
    SemanticTokenModifier::DOCUMENTATION,  // 8
    SemanticTokenModifier::DEFAULT_LIBRARY, // 9
];

/// Token type indices
#[allow(dead_code)]
mod token_type_idx {
    pub const NAMESPACE: u32 = 0;
    pub const TYPE: u32 = 1;
    pub const TYPE_PARAMETER: u32 = 6;
    pub const PARAMETER: u32 = 7;
    pub const VARIABLE: u32 = 8;
    pub const PROPERTY: u32 = 9;
    pub const FUNCTION: u32 = 12;
    pub const KEYWORD: u32 = 15;
    pub const COMMENT: u32 = 17;
    pub const STRING: u32 = 18;
    pub const NUMBER: u32 = 19;
    pub const OPERATOR: u32 = 21;
}

/// Token modifier bit flags
mod token_modifier_bits {
    pub const DECLARATION: u32 = 1 << 0;
    #[allow(dead_code)]
    pub const DEFINITION: u32 = 1 << 1;
    pub const READONLY: u32 = 1 << 2;
    pub const DOCUMENTATION: u32 = 1 << 8;
    pub const DEFAULT_LIBRARY: u32 = 1 << 9;
}

/// Get the semantic token legend for capabilities
pub fn get_legend() -> SemanticTokensLegend {
    SemanticTokensLegend {
        token_types: TOKEN_TYPES.to_vec(),
        token_modifiers: TOKEN_MODIFIERS.to_vec(),
    }
}

/// Generate full semantic tokens for a document
pub fn generate_semantic_tokens(
    text: &str,
    ast: Option<&Program>,
    symbols: Option<&SymbolTable>,
) -> SemanticTokensResult {
    let tokens = tokenize_document(text, ast, symbols);
    let encoded = encode_tokens(&tokens, text);

    SemanticTokensResult::Tokens(SemanticTokens {
        result_id: None,
        data: encoded,
    })
}

/// Generate semantic tokens for a range
pub fn generate_semantic_tokens_range(
    text: &str,
    range: Range,
    ast: Option<&Program>,
    symbols: Option<&SymbolTable>,
) -> SemanticTokensRangeResult {
    let all_tokens = tokenize_document(text, ast, symbols);

    // Filter tokens to those in range
    let filtered: Vec<SemanticTokenInfo> = all_tokens
        .into_iter()
        .filter(|t| is_token_in_range(t, range))
        .collect();

    let encoded = encode_tokens(&filtered, text);

    SemanticTokensRangeResult::Tokens(SemanticTokens {
        result_id: None,
        data: encoded,
    })
}

/// Information about a semantic token
#[derive(Debug, Clone)]
struct SemanticTokenInfo {
    /// Byte offset in source
    start: usize,
    /// Token length
    length: usize,
    /// Token type index
    token_type: u32,
    /// Token modifier bitmask
    modifiers: u32,
}

/// Tokenize a document and classify tokens semantically
fn tokenize_document(
    text: &str,
    ast: Option<&Program>,
    symbols: Option<&SymbolTable>,
) -> Vec<SemanticTokenInfo> {
    let mut semantic_tokens = Vec::new();

    // First, get lexical tokens
    let mut lexer = Lexer::new(text);
    let (tokens, _) = lexer.tokenize_with_comments();

    // Build a set of known declarations for classification
    let declarations = collect_declarations(ast);
    let parameters = collect_parameters(ast);
    let builtins = get_builtin_names();

    // Classify each token
    for token in &tokens {
        if let Some(semantic) = classify_token(token, &declarations, &parameters, &builtins, symbols) {
            semantic_tokens.push(semantic);
        }
    }

    semantic_tokens
}

/// Classify a lexical token into a semantic token
fn classify_token(
    token: &Token,
    declarations: &std::collections::HashSet<String>,
    parameters: &std::collections::HashSet<String>,
    builtins: &std::collections::HashSet<&str>,
    symbols: Option<&SymbolTable>,
) -> Option<SemanticTokenInfo> {
    let (token_type, modifiers) = match token.kind {
        // Keywords
        TokenKind::Let | TokenKind::Var | TokenKind::Fn | TokenKind::Type |
        TokenKind::If | TokenKind::Else | TokenKind::While | TokenKind::For |
        TokenKind::In | TokenKind::Return | TokenKind::Break | TokenKind::Continue |
        TokenKind::Import | TokenKind::Export | TokenKind::From | TokenKind::Extern |
        TokenKind::Match | TokenKind::As | TokenKind::Extends | TokenKind::Is => {
            (token_type_idx::KEYWORD, 0)
        }

        // Boolean literals (also keywords semantically)
        TokenKind::True | TokenKind::False | TokenKind::Null => {
            (token_type_idx::KEYWORD, 0)
        }

        // Literals
        TokenKind::Number => (token_type_idx::NUMBER, 0),
        TokenKind::String => (token_type_idx::STRING, 0),

        // Comments
        TokenKind::LineComment | TokenKind::BlockComment => {
            (token_type_idx::COMMENT, 0)
        }
        TokenKind::DocComment => {
            (token_type_idx::COMMENT, token_modifier_bits::DOCUMENTATION)
        }

        // Operators
        TokenKind::Plus | TokenKind::Minus | TokenKind::Star | TokenKind::Slash |
        TokenKind::Percent | TokenKind::Bang | TokenKind::EqualEqual |
        TokenKind::BangEqual | TokenKind::Less | TokenKind::LessEqual |
        TokenKind::Greater | TokenKind::GreaterEqual | TokenKind::AmpAmp |
        TokenKind::PipePipe | TokenKind::Ampersand | TokenKind::Pipe |
        TokenKind::PlusEqual | TokenKind::MinusEqual | TokenKind::StarEqual |
        TokenKind::SlashEqual | TokenKind::PercentEqual | TokenKind::PlusPlus |
        TokenKind::MinusMinus | TokenKind::Equal | TokenKind::Arrow |
        TokenKind::FatArrow | TokenKind::Question => {
            (token_type_idx::OPERATOR, 0)
        }

        // Identifiers - need context to classify
        TokenKind::Identifier => {
            let name = &token.lexeme;

            // Check if it's a builtin
            if builtins.contains(name.as_str()) {
                return Some(SemanticTokenInfo {
                    start: token.span.start,
                    length: token.span.len(),
                    token_type: token_type_idx::FUNCTION,
                    modifiers: token_modifier_bits::DEFAULT_LIBRARY,
                });
            }

            // Check if it's a parameter
            if parameters.contains(name) {
                return Some(SemanticTokenInfo {
                    start: token.span.start,
                    length: token.span.len(),
                    token_type: token_type_idx::PARAMETER,
                    modifiers: 0,
                });
            }

            // Check symbol table for more info
            if let Some(table) = symbols {
                if let Some(symbol) = table.lookup(name) {
                    let (ttype, mods) = match symbol.kind {
                        atlas_runtime::symbol::SymbolKind::Function => {
                            (token_type_idx::FUNCTION, token_modifier_bits::DECLARATION)
                        }
                        atlas_runtime::symbol::SymbolKind::Variable => {
                            let mods = if symbol.mutable {
                                0
                            } else {
                                token_modifier_bits::READONLY
                            };
                            (token_type_idx::VARIABLE, mods)
                        }
                        atlas_runtime::symbol::SymbolKind::Parameter => {
                            (token_type_idx::PARAMETER, 0)
                        }
                        atlas_runtime::symbol::SymbolKind::Builtin => {
                            (token_type_idx::FUNCTION, token_modifier_bits::DEFAULT_LIBRARY)
                        }
                    };
                    return Some(SemanticTokenInfo {
                        start: token.span.start,
                        length: token.span.len(),
                        token_type: ttype,
                        modifiers: mods,
                    });
                }
            }

            // Check if it's a declaration (function name)
            if declarations.contains(name) {
                return Some(SemanticTokenInfo {
                    start: token.span.start,
                    length: token.span.len(),
                    token_type: token_type_idx::FUNCTION,
                    modifiers: token_modifier_bits::DECLARATION,
                });
            }

            // Check if name looks like a type (PascalCase)
            if is_type_name(name) {
                return Some(SemanticTokenInfo {
                    start: token.span.start,
                    length: token.span.len(),
                    token_type: token_type_idx::TYPE,
                    modifiers: 0,
                });
            }

            // Default to variable
            (token_type_idx::VARIABLE, 0)
        }

        // Punctuation - skip these (they're handled by syntax highlighting)
        TokenKind::LeftParen | TokenKind::RightParen | TokenKind::LeftBrace |
        TokenKind::RightBrace | TokenKind::LeftBracket | TokenKind::RightBracket |
        TokenKind::Semicolon | TokenKind::Comma | TokenKind::Dot |
        TokenKind::Colon | TokenKind::Underscore => {
            return None;
        }

        // Special tokens - skip
        TokenKind::Eof | TokenKind::Error => return None,
    };

    Some(SemanticTokenInfo {
        start: token.span.start,
        length: token.span.len(),
        token_type,
        modifiers,
    })
}

/// Collect function and type declarations from AST
fn collect_declarations(ast: Option<&Program>) -> std::collections::HashSet<String> {
    let mut names = std::collections::HashSet::new();

    if let Some(program) = ast {
        for item in &program.items {
            match item {
                Item::Function(func) => {
                    names.insert(func.name.name.clone());
                }
                Item::TypeAlias(alias) => {
                    names.insert(alias.name.name.clone());
                }
                _ => {}
            }
        }
    }

    names
}

/// Collect parameter names from AST
fn collect_parameters(ast: Option<&Program>) -> std::collections::HashSet<String> {
    let mut params = std::collections::HashSet::new();

    if let Some(program) = ast {
        for item in &program.items {
            if let Item::Function(func) = item {
                for param in &func.params {
                    params.insert(param.name.name.clone());
                }
            }
        }
    }

    params
}

/// Get builtin function names
fn get_builtin_names() -> std::collections::HashSet<&'static str> {
    [
        // I/O
        "print", "println", "input",
        // Type conversion
        "string", "number", "bool", "int",
        // Type checking
        "typeof", "is_number", "is_string", "is_bool", "is_null", "is_array", "is_function",
        // Collections
        "len", "push", "pop", "shift", "unshift", "slice", "concat", "reverse", "sort",
        "map", "filter", "reduce", "find", "every", "some", "includes", "index_of", "join", "flat",
        // String
        "split", "trim", "to_upper", "to_lower", "starts_with", "ends_with", "contains",
        "replace", "char_at", "substring", "pad_start", "pad_end", "repeat",
        // Math
        "abs", "floor", "ceil", "round", "sqrt", "pow", "min", "max",
        "sin", "cos", "tan", "log", "log10", "exp", "random", "random_range",
        // HashMap
        "keys", "values", "entries", "has_key", "remove",
        // Assertions
        "assert", "assert_eq", "assert_ne",
        // Time
        "time", "sleep",
        // Error handling
        "error", "try_catch",
    ]
    .into_iter()
    .collect()
}

/// Check if a name looks like a type (PascalCase)
fn is_type_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    let first = name.chars().next().unwrap();
    first.is_uppercase() && name.chars().skip(1).any(|c| c.is_lowercase())
}

/// Check if a token is within a range
fn is_token_in_range(_token: &SemanticTokenInfo, _range: Range) -> bool {
    // This is a simplified check - would need line/column info for full accuracy
    // For now, return true (include all tokens when doing range requests)
    true
}

/// Encode tokens into LSP delta format
fn encode_tokens(tokens: &[SemanticTokenInfo], text: &str) -> Vec<SemanticToken> {
    let line_offsets = compute_line_offsets(text);
    let mut result = Vec::new();

    let mut prev_line = 0u32;
    let mut prev_start = 0u32;

    for token in tokens {
        // Convert byte offset to line/column
        let (line, column) = offset_to_position(&line_offsets, token.start);

        // Compute deltas
        let delta_line = line - prev_line;
        let delta_start = if delta_line == 0 {
            column - prev_start
        } else {
            column
        };

        result.push(SemanticToken {
            delta_line,
            delta_start,
            length: token.length as u32,
            token_type: token.token_type,
            token_modifiers_bitset: token.modifiers,
        });

        prev_line = line;
        prev_start = column;
    }

    result
}

/// Compute line start offsets
fn compute_line_offsets(text: &str) -> Vec<usize> {
    let mut offsets = vec![0];

    for (i, c) in text.char_indices() {
        if c == '\n' {
            offsets.push(i + 1);
        }
    }

    offsets
}

/// Convert byte offset to line/column position
fn offset_to_position(line_offsets: &[usize], offset: usize) -> (u32, u32) {
    // Binary search for line
    let line = match line_offsets.binary_search(&offset) {
        Ok(l) => l,
        Err(l) => l.saturating_sub(1),
    };

    let line_start = line_offsets.get(line).copied().unwrap_or(0);
    let column = offset - line_start;

    (line as u32, column as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_legend() {
        let legend = get_legend();
        assert!(!legend.token_types.is_empty());
        assert!(!legend.token_modifiers.is_empty());
    }

    #[test]
    fn test_compute_line_offsets() {
        let text = "line1\nline2\nline3";
        let offsets = compute_line_offsets(text);
        assert_eq!(offsets, vec![0, 6, 12]);
    }

    #[test]
    fn test_offset_to_position_first_line() {
        let offsets = vec![0, 10, 20];
        assert_eq!(offset_to_position(&offsets, 5), (0, 5));
    }

    #[test]
    fn test_offset_to_position_second_line() {
        let offsets = vec![0, 10, 20];
        assert_eq!(offset_to_position(&offsets, 15), (1, 5));
    }

    #[test]
    fn test_offset_to_position_line_start() {
        let offsets = vec![0, 10, 20];
        assert_eq!(offset_to_position(&offsets, 10), (1, 0));
    }

    #[test]
    fn test_is_type_name() {
        assert!(is_type_name("MyType"));
        assert!(is_type_name("HashMap"));
        assert!(!is_type_name("myVariable"));
        assert!(!is_type_name("CONSTANT"));
        assert!(!is_type_name(""));
    }

    #[test]
    fn test_get_builtin_names() {
        let builtins = get_builtin_names();
        assert!(builtins.contains("print"));
        assert!(builtins.contains("len"));
        assert!(builtins.contains("map"));
        assert!(!builtins.contains("foo"));
    }

    #[test]
    fn test_tokenize_simple() {
        let text = "let x = 42;";
        let tokens = tokenize_document(text, None, None);

        // Should have tokens for: let, x, =, 42
        assert!(!tokens.is_empty());

        // First token should be 'let' keyword
        assert_eq!(tokens[0].token_type, token_type_idx::KEYWORD);
    }

    #[test]
    fn test_tokenize_function() {
        let text = "fn foo() { return 1; }";
        let tokens = tokenize_document(text, None, None);

        // Should have tokens for: fn, foo, return, 1
        let keyword_count = tokens.iter().filter(|t| t.token_type == token_type_idx::KEYWORD).count();
        assert!(keyword_count >= 2); // fn, return
    }

    #[test]
    fn test_tokenize_builtin() {
        let text = "print(42);";
        let tokens = tokenize_document(text, None, None);

        // 'print' should be classified as function with DEFAULT_LIBRARY
        let print_token = tokens.iter().find(|t| t.token_type == token_type_idx::FUNCTION);
        assert!(print_token.is_some());
        assert!(print_token.unwrap().modifiers & token_modifier_bits::DEFAULT_LIBRARY != 0);
    }

    #[test]
    fn test_tokenize_string() {
        let text = "let s = \"hello\";";
        let tokens = tokenize_document(text, None, None);

        let string_token = tokens.iter().find(|t| t.token_type == token_type_idx::STRING);
        assert!(string_token.is_some());
    }

    #[test]
    fn test_tokenize_number() {
        let text = "let n = 3.14;";
        let tokens = tokenize_document(text, None, None);

        let num_token = tokens.iter().find(|t| t.token_type == token_type_idx::NUMBER);
        assert!(num_token.is_some());
    }

    #[test]
    fn test_tokenize_comment() {
        let text = "// This is a comment\nlet x = 1;";
        let tokens = tokenize_document(text, None, None);

        let comment_token = tokens.iter().find(|t| t.token_type == token_type_idx::COMMENT);
        assert!(comment_token.is_some());
    }

    #[test]
    fn test_tokenize_operator() {
        let text = "let x = a + b;";
        let tokens = tokenize_document(text, None, None);

        let op_tokens: Vec<_> = tokens.iter().filter(|t| t.token_type == token_type_idx::OPERATOR).collect();
        assert!(op_tokens.len() >= 2); // = and +
    }

    #[test]
    fn test_encode_tokens_delta() {
        let text = "let x = 1;";
        let tokens = vec![
            SemanticTokenInfo { start: 0, length: 3, token_type: token_type_idx::KEYWORD, modifiers: 0 },
            SemanticTokenInfo { start: 4, length: 1, token_type: token_type_idx::VARIABLE, modifiers: 0 },
        ];

        let encoded = encode_tokens(&tokens, text);
        assert_eq!(encoded.len(), 2);

        // First token: line 0, start 0
        assert_eq!(encoded[0].delta_line, 0);
        assert_eq!(encoded[0].delta_start, 0);
        assert_eq!(encoded[0].length, 3);

        // Second token: same line, delta_start = 4
        assert_eq!(encoded[1].delta_line, 0);
        assert_eq!(encoded[1].delta_start, 4);
        assert_eq!(encoded[1].length, 1);
    }

    #[test]
    fn test_encode_tokens_multiline() {
        let text = "let x = 1;\nlet y = 2;";
        let tokens = vec![
            SemanticTokenInfo { start: 0, length: 3, token_type: token_type_idx::KEYWORD, modifiers: 0 },
            SemanticTokenInfo { start: 11, length: 3, token_type: token_type_idx::KEYWORD, modifiers: 0 },
        ];

        let encoded = encode_tokens(&tokens, text);
        assert_eq!(encoded.len(), 2);

        // First token: line 0, start 0
        assert_eq!(encoded[0].delta_line, 0);

        // Second token: delta_line = 1, start = 0
        assert_eq!(encoded[1].delta_line, 1);
        assert_eq!(encoded[1].delta_start, 0);
    }

    #[test]
    fn test_collect_declarations() {
        // Without AST, should return empty set
        let decls = collect_declarations(None);
        assert!(decls.is_empty());
    }

    #[test]
    fn test_collect_parameters() {
        // Without AST, should return empty set
        let params = collect_parameters(None);
        assert!(params.is_empty());
    }
}
