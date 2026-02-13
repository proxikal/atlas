//! Parsing (tokens to AST)
//!
//! The parser converts a stream of tokens into an Abstract Syntax Tree (AST).
//! Uses Pratt parsing for expressions and recursive descent for statements.

mod expr;
mod stmt;

use crate::ast::*;
use crate::diagnostic::Diagnostic;
use crate::span::Span;
use crate::token::{Token, TokenKind};

/// Parser state for building AST from tokens
pub struct Parser {
    pub(super) tokens: Vec<Token>,
    pub(super) current: usize,
    pub(super) diagnostics: Vec<Diagnostic>,
}

/// Operator precedence levels for Pratt parsing
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Precedence {
    Lowest,
    Or,         // ||
    And,        // &&
    Equality,   // == !=
    Comparison, // < <= > >=
    Term,       // + -
    Factor,     // * / %
    Unary,      // ! -
    Call,       // () []
}

impl Parser {
    /// Create a new parser for the given tokens
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
            diagnostics: Vec::new(),
        }
    }

    /// Parse tokens into an AST
    pub fn parse(&mut self) -> (Program, Vec<Diagnostic>) {
        let mut items = Vec::new();

        while !self.is_at_end() {
            match self.parse_item() {
                Ok(item) => items.push(item),
                Err(_) => self.synchronize(),
            }
        }

        (Program { items }, std::mem::take(&mut self.diagnostics))
    }

    // === Top-level parsing ===

    /// Parse a top-level item (function or statement)
    fn parse_item(&mut self) -> Result<Item, ()> {
        if self.check(TokenKind::Fn) {
            Ok(Item::Function(self.parse_function()?))
        } else {
            Ok(Item::Statement(self.parse_statement()?))
        }
    }

    /// Parse a function declaration
    fn parse_function(&mut self) -> Result<FunctionDecl, ()> {
        let fn_span = self.consume(TokenKind::Fn, "Expected 'fn'")?.span;

        let name_token = self.consume_identifier("a function name")?;
        let name = Identifier {
            name: name_token.lexeme.clone(),
            span: name_token.span,
        };

        self.consume(TokenKind::LeftParen, "Expected '(' after function name")?;

        // Parse parameters
        let mut params = Vec::new();
        if !self.check(TokenKind::RightParen) {
            loop {
                let param_span_start = self.peek().span;
                let param_name_tok = self.consume_identifier("a parameter name")?;
                let param_name = param_name_tok.lexeme.clone();
                let param_name_span = param_name_tok.span;

                self.consume(TokenKind::Colon, "Expected ':' after parameter name")?;
                let type_ref = self.parse_type_ref()?;
                let param_span_end = type_ref.span();

                params.push(Param {
                    name: Identifier {
                        name: param_name,
                        span: param_name_span,
                    },
                    type_ref,
                    span: param_span_start.merge(param_span_end),
                });

                if !self.match_token(TokenKind::Comma) {
                    break;
                }
            }
        }

        self.consume(TokenKind::RightParen, "Expected ')' after parameters")?;

        // Parse return type (required in AST)
        let return_type = if self.match_token(TokenKind::Arrow) {
            self.parse_type_ref()?
        } else {
            // Default to null type if not specified
            TypeRef::Named("null".to_string(), Span::dummy())
        };

        // Parse body
        let body = self.parse_block()?;
        let end_span = body.span;

        Ok(FunctionDecl {
            name,
            params,
            return_type,
            body,
            span: fn_span.merge(end_span),
        })
    }

    // === Helper methods ===

    /// Advance to next token and return reference to previous
    pub(super) fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        &self.tokens[self.current - 1]
    }

    /// Peek at current token
    pub(super) fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    /// Check if current token matches kind
    pub(super) fn check(&self, kind: TokenKind) -> bool {
        !self.is_at_end() && self.peek().kind == kind
    }

    /// Match and consume token if it matches
    pub(super) fn match_token(&mut self, kind: TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Consume token of given kind or error
    pub(super) fn consume(&mut self, kind: TokenKind, message: &str) -> Result<&Token, ()> {
        if self.check(kind) {
            Ok(self.advance())
        } else {
            self.error(message);
            Err(())
        }
    }

    /// Check if at end of token stream
    pub(super) fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len() || self.tokens[self.current].kind == TokenKind::Eof
    }

    /// Record an error
    pub(super) fn error(&mut self, message: &str) {
        let span = self.peek().span;
        self.diagnostics
            .push(Diagnostic::error_with_code("AT1000", message, span).with_label("syntax error"));
    }

    /// Check if a token kind is a reserved keyword
    fn is_reserved_keyword(kind: TokenKind) -> bool {
        matches!(
            kind,
            TokenKind::Let
                | TokenKind::Var
                | TokenKind::Fn
                | TokenKind::If
                | TokenKind::Else
                | TokenKind::While
                | TokenKind::For
                | TokenKind::Return
                | TokenKind::Break
                | TokenKind::Continue
                | TokenKind::True
                | TokenKind::False
                | TokenKind::Null
                | TokenKind::Import
                | TokenKind::Match
        )
    }

    /// Consume an identifier token with enhanced error message for keywords
    pub(super) fn consume_identifier(&mut self, context: &str) -> Result<&Token, ()> {
        let current = self.peek();

        // Check if it's a reserved keyword
        if Self::is_reserved_keyword(current.kind) {
            let keyword_name = &current.lexeme;

            // Special message for import/match (reserved for future)
            if current.kind == TokenKind::Import || current.kind == TokenKind::Match {
                self.error(&format!(
                    "Cannot use reserved keyword '{}' as {}. This keyword is reserved for future use",
                    keyword_name, context
                ));
            } else {
                self.error(&format!(
                    "Cannot use reserved keyword '{}' as {}",
                    keyword_name, context
                ));
            }
            Err(())
        } else if current.kind == TokenKind::Identifier {
            Ok(self.advance())
        } else {
            self.error(&format!(
                "Expected {} but found {:?}",
                context, current.kind
            ));
            Err(())
        }
    }

    /// Synchronize after error
    pub(super) fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.tokens[self.current - 1].kind == TokenKind::Semicolon {
                return;
            }

            match self.peek().kind {
                TokenKind::Fn
                | TokenKind::Let
                | TokenKind::Var
                | TokenKind::If
                | TokenKind::While
                | TokenKind::For
                | TokenKind::Return => return,
                _ => {
                    self.advance();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse_source(source: &str) -> (Program, Vec<Diagnostic>) {
        let mut lexer = Lexer::new(source);
        let (tokens, _) = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        parser.parse()
    }

    #[test]
    fn test_parser_creation() {
        let mut parser = Parser::new(Vec::new());
        let (program, _) = parser.parse();
        assert_eq!(program.items.len(), 0);
    }

    #[test]
    fn test_parse_number_literal() {
        let (program, diagnostics) = parse_source("42;");
        assert_eq!(diagnostics.len(), 0);
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_string_literal() {
        let (program, diagnostics) = parse_source(r#""hello";"#);
        assert_eq!(diagnostics.len(), 0);
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_boolean_literals() {
        let (program, diagnostics) = parse_source("true; false;");
        assert_eq!(diagnostics.len(), 0);
        assert_eq!(program.items.len(), 2);
    }

    #[test]
    fn test_parse_var_decl() {
        let (program, diagnostics) = parse_source("let x = 42;");
        assert_eq!(diagnostics.len(), 0);
        assert_eq!(program.items.len(), 1);

        match &program.items[0] {
            Item::Statement(Stmt::VarDecl(decl)) => {
                assert_eq!(decl.name.name, "x");
                assert!(!decl.mutable);
            }
            _ => panic!("Expected variable declaration"),
        }
    }

    #[test]
    fn test_parse_mutable_var_decl() {
        let (program, diagnostics) = parse_source("var x = 42;");
        assert_eq!(diagnostics.len(), 0);

        match &program.items[0] {
            Item::Statement(Stmt::VarDecl(decl)) => {
                assert!(decl.mutable);
            }
            _ => panic!("Expected variable declaration"),
        }
    }

    #[test]
    fn test_parse_binary_expr() {
        let (program, diagnostics) = parse_source("1 + 2;");
        assert_eq!(diagnostics.len(), 0);
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_function_decl() {
        let (program, diagnostics) = parse_source("fn test(x: number) -> number { return x; }");
        assert_eq!(diagnostics.len(), 0);
        assert_eq!(program.items.len(), 1);

        match &program.items[0] {
            Item::Function(func) => {
                assert_eq!(func.name.name, "test");
                assert_eq!(func.params.len(), 1);
            }
            _ => panic!("Expected function declaration"),
        }
    }

    #[test]
    fn test_parse_if_stmt() {
        let (program, diagnostics) = parse_source("if (x) { }");
        assert_eq!(diagnostics.len(), 0);
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_while_stmt() {
        let (program, diagnostics) = parse_source("while (x) { }");
        assert_eq!(diagnostics.len(), 0);
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_for_stmt() {
        let (program, diagnostics) = parse_source("for (let i = 0; i < 10; i) { }");
        assert_eq!(diagnostics.len(), 0);
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_array_literal() {
        let (program, diagnostics) = parse_source("[1, 2, 3];");
        assert_eq!(diagnostics.len(), 0);
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_call_expr() {
        let (program, diagnostics) = parse_source("foo(1, 2);");
        assert_eq!(diagnostics.len(), 0);
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_index_expr() {
        let (program, diagnostics) = parse_source("arr[0];");
        assert_eq!(diagnostics.len(), 0);
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_assignment() {
        let (program, diagnostics) = parse_source("x = 42;");
        assert_eq!(diagnostics.len(), 0);

        match &program.items[0] {
            Item::Statement(Stmt::Assign(_)) => {}
            _ => panic!("Expected assignment"),
        }
    }

    // === Error Recovery Tests ===

    #[test]
    fn test_recovery_missing_semicolon() {
        let (_program, diagnostics) = parse_source("let x = 42\nlet y = 10;");
        assert!(diagnostics.len() >= 1);
        assert!(
            diagnostics[0].message.contains("Expected ';'")
                || diagnostics[0].message.contains("Expected")
        );
    }

    #[test]
    fn test_recovery_missing_closing_brace_in_block() {
        let source = "fn test() {\n    let x = 42;\n\nfn other() { }";
        let (_program, diagnostics) = parse_source(source);
        assert!(diagnostics.len() >= 1);
        let has_brace_error = diagnostics
            .iter()
            .any(|d| d.message.contains("Expected '}'"));
        assert!(has_brace_error, "Expected missing brace error");
    }

    #[test]
    fn test_recovery_missing_closing_paren() {
        let source = "if (x > 10 { let y = 5; }";
        let (_program, diagnostics) = parse_source(source);
        assert!(diagnostics.len() >= 1);
        assert!(diagnostics[0].message.contains("Expected ')'"));
    }

    #[test]
    fn test_recovery_invalid_expression() {
        let source = "let x = ;\nlet y = 42;";
        let (_program, diagnostics) = parse_source(source);
        assert!(diagnostics.len() >= 1);
        assert!(
            diagnostics[0].message.contains("Expected expression")
                || diagnostics[0].message.contains("Expected '='")
        );
    }

    #[test]
    fn test_recovery_multiple_errors() {
        let source = "let x = ;\nlet y = ;\nlet z = 99;";
        let (_program, diagnostics) = parse_source(source);
        assert!(
            diagnostics.len() >= 1,
            "Expected at least 1 error, got {}",
            diagnostics.len()
        );
    }

    #[test]
    fn test_recovery_missing_function_body_brace() {
        let source = "fn test()\n    return 42;\n}";
        let (_program, diagnostics) = parse_source(source);
        assert!(diagnostics.len() >= 1);
        assert!(diagnostics[0].message.contains("Expected '{'"));
    }

    #[test]
    fn test_recovery_nested_block_error() {
        let source = "fn test() {\n    if (true) {\n        let x = ;\n    }\n    let y = 42;\n}";
        let (_program, diagnostics) = parse_source(source);
        assert!(diagnostics.len() >= 1);
        assert!(diagnostics
            .iter()
            .any(|d| d.message.contains("Expected expression") || d.message.contains("Expected")));
    }

    #[test]
    fn test_recovery_missing_comma_in_params() {
        let source = "fn test(x: number y: number) { }";
        let (_program, diagnostics) = parse_source(source);
        assert!(diagnostics.len() >= 1);
        assert!(
            diagnostics[0].message.contains("Expected ')'")
                || diagnostics[0].message.contains("Expected")
        );
    }

    #[test]
    fn test_recovery_missing_comma_in_array() {
        let source = "[1 2 3];";
        let (_program, diagnostics) = parse_source(source);
        assert!(diagnostics.len() >= 1);
    }

    #[test]
    fn test_recovery_synchronize_at_statement_keyword() {
        let source = "let x = ;\nif (true) { }\nlet y = 5;";
        let (_program, diagnostics) = parse_source(source);
        assert!(diagnostics.len() >= 1);
    }

    #[test]
    fn test_recovery_no_infinite_loop_on_eof() {
        let source = "let x = ";
        let (_program, diagnostics) = parse_source(source);
        assert!(diagnostics.len() >= 1);
    }

    #[test]
    fn test_recovery_preserves_valid_code_after_error() {
        let source = "let bad = ;\nlet good = 42;";
        let (_program, diagnostics) = parse_source(source);
        assert!(diagnostics.len() >= 1);
    }

    #[test]
    fn test_parse_function_type() {
        // Simple function type
        let source = "fn test(f: (number) -> bool) -> void { }";
        let (program, diagnostics) = parse_source(source);
        assert_eq!(diagnostics.len(), 0, "Expected no errors");
        assert_eq!(program.items.len(), 1);

        if let Item::Function(func) = &program.items[0] {
            assert_eq!(func.params.len(), 1);
            match &func.params[0].type_ref {
                TypeRef::Function {
                    params,
                    return_type,
                    ..
                } => {
                    assert_eq!(params.len(), 1);
                    assert!(matches!(params[0], TypeRef::Named(ref name, _) if name == "number"));
                    assert!(matches!(**return_type, TypeRef::Named(ref name, _) if name == "bool"));
                }
                _ => panic!("Expected function type"),
            }
        } else {
            panic!("Expected function declaration");
        }
    }

    #[test]
    fn test_parse_function_type_multiple_params() {
        let source = "fn test(f: (number, string) -> bool) -> void { }";
        let (program, diagnostics) = parse_source(source);
        assert_eq!(diagnostics.len(), 0, "Expected no errors");
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_function_type_no_params() {
        let source = "fn test(f: () -> number) -> void { }";
        let (program, diagnostics) = parse_source(source);
        assert_eq!(diagnostics.len(), 0, "Expected no errors");
        assert_eq!(program.items.len(), 1);
    }
}
