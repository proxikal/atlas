//! Lexical analysis (tokenization)
//!
//! The lexer converts Atlas source code into a stream of tokens with accurate span information.

use crate::diagnostic::Diagnostic;
use crate::span::Span;
use crate::token::{Token, TokenKind};

mod literals;

/// Lexer state for tokenizing source code
pub struct Lexer {
    /// Original source code
    pub(super) source: String,
    /// Characters of source code
    pub(super) chars: Vec<char>,
    /// Current position in chars
    pub(super) current: usize,
    /// Current line number (1-indexed)
    pub(super) line: u32,
    /// Current column number (1-indexed)
    pub(super) column: u32,
    /// Start position of current token
    pub(super) start_pos: usize,
    /// Start line of current token
    pub(super) start_line: u32,
    /// Start column of current token
    pub(super) start_column: u32,
    /// Collected diagnostics
    pub(super) diagnostics: Vec<Diagnostic>,
    /// Whether to emit comment tokens
    emit_comments: bool,
    /// Pending comment tokens to emit
    pending_comments: Vec<Token>,
}

impl Lexer {
    /// Create a new lexer for the given source code
    pub fn new(source: impl Into<String>) -> Self {
        let source = source.into();
        let chars: Vec<char> = source.chars().collect();
        Self {
            source,
            chars,
            current: 0,
            line: 1,
            column: 1,
            start_pos: 0,
            start_line: 1,
            start_column: 1,
            diagnostics: Vec::new(),
            emit_comments: false,
            pending_comments: Vec::new(),
        }
    }

    /// Tokenize the source code, returning tokens and any diagnostics
    pub fn tokenize(&mut self) -> (Vec<Token>, Vec<Diagnostic>) {
        let mut tokens = Vec::new();

        loop {
            let token = self.next_token();
            let is_eof = token.kind == TokenKind::Eof;
            tokens.push(token);
            if is_eof {
                break;
            }
        }

        (tokens, std::mem::take(&mut self.diagnostics))
    }

    /// Tokenize preserving comment tokens in the stream
    pub fn tokenize_with_comments(&mut self) -> (Vec<Token>, Vec<Diagnostic>) {
        self.emit_comments = true;
        let mut tokens = Vec::new();

        loop {
            let token = self.next_token();
            let is_eof = token.kind == TokenKind::Eof;

            // Drain any pending comment tokens collected before this token
            tokens.append(&mut self.pending_comments);
            tokens.push(token);

            if is_eof {
                break;
            }
        }

        self.emit_comments = false;
        (tokens, std::mem::take(&mut self.diagnostics))
    }

    /// Scan the next token
    fn next_token(&mut self) -> Token {
        self.skip_whitespace_and_comments();

        // Mark start of token
        self.start_pos = self.current;
        self.start_line = self.line;
        self.start_column = self.column;

        if self.is_at_end() {
            return self.make_token(TokenKind::Eof, "");
        }

        let c = self.advance();

        match c {
            // Single-character tokens
            '(' => self.make_token(TokenKind::LeftParen, "("),
            ')' => self.make_token(TokenKind::RightParen, ")"),
            '{' => self.make_token(TokenKind::LeftBrace, "{"),
            '}' => self.make_token(TokenKind::RightBrace, "}"),
            '[' => self.make_token(TokenKind::LeftBracket, "["),
            ']' => self.make_token(TokenKind::RightBracket, "]"),
            ';' => self.make_token(TokenKind::Semicolon, ";"),
            ',' => self.make_token(TokenKind::Comma, ","),
            ':' => self.make_token(TokenKind::Colon, ":"),
            '?' => self.make_token(TokenKind::Question, "?"),

            // Operators with potential compound forms
            '+' => {
                if self.match_char('+') {
                    self.make_token(TokenKind::PlusPlus, "++")
                } else if self.match_char('=') {
                    self.make_token(TokenKind::PlusEqual, "+=")
                } else {
                    self.make_token(TokenKind::Plus, "+")
                }
            }
            '-' => {
                if self.match_char('-') {
                    self.make_token(TokenKind::MinusMinus, "--")
                } else if self.match_char('=') {
                    self.make_token(TokenKind::MinusEqual, "-=")
                } else if self.match_char('>') {
                    self.make_token(TokenKind::Arrow, "->")
                } else {
                    self.make_token(TokenKind::Minus, "-")
                }
            }
            '*' => {
                if self.match_char('=') {
                    self.make_token(TokenKind::StarEqual, "*=")
                } else {
                    self.make_token(TokenKind::Star, "*")
                }
            }
            '/' => {
                if self.match_char('=') {
                    self.make_token(TokenKind::SlashEqual, "/=")
                } else {
                    self.make_token(TokenKind::Slash, "/")
                }
            }
            '%' => {
                if self.match_char('=') {
                    self.make_token(TokenKind::PercentEqual, "%=")
                } else {
                    self.make_token(TokenKind::Percent, "%")
                }
            }

            // Two-character tokens
            '=' => {
                if self.match_char('=') {
                    self.make_token(TokenKind::EqualEqual, "==")
                } else if self.match_char('>') {
                    self.make_token(TokenKind::FatArrow, "=>")
                } else {
                    self.make_token(TokenKind::Equal, "=")
                }
            }
            '!' => {
                if self.match_char('=') {
                    self.make_token(TokenKind::BangEqual, "!=")
                } else {
                    self.make_token(TokenKind::Bang, "!")
                }
            }
            '<' => {
                if self.match_char('=') {
                    self.make_token(TokenKind::LessEqual, "<=")
                } else {
                    self.make_token(TokenKind::Less, "<")
                }
            }
            '>' => {
                if self.match_char('=') {
                    self.make_token(TokenKind::GreaterEqual, ">=")
                } else {
                    self.make_token(TokenKind::Greater, ">")
                }
            }
            '&' => {
                if self.match_char('&') {
                    self.make_token(TokenKind::AmpAmp, "&&")
                } else {
                    self.make_token(TokenKind::Ampersand, "&")
                }
            }
            '|' => {
                if self.match_char('|') {
                    self.make_token(TokenKind::PipePipe, "||")
                } else {
                    self.make_token(TokenKind::Pipe, "|")
                }
            }

            // String literals
            '"' => self.string(),

            // Numbers
            c if c.is_ascii_digit() => self.number(),

            // Dot (member access) or start of decimal number
            '.' => {
                // Check if this is the start of a decimal number (e.g., .5)
                // NOTE: Atlas doesn't support .5 syntax, only 0.5
                // So . is always a member access operator
                self.make_token(TokenKind::Dot, ".")
            }

            // Identifiers and keywords
            c if c.is_alphabetic() || c == '_' => self.identifier(),

            // Unexpected character
            _ => self.error_token(&format!("Unexpected character '{}'", c)),
        }
    }

    /// Skip whitespace and comments
    fn skip_whitespace_and_comments(&mut self) {
        loop {
            if self.is_at_end() {
                return;
            }

            match self.peek() {
                ' ' | '\r' | '\t' => {
                    self.advance();
                }
                '\n' => {
                    self.advance();
                    self.line += 1;
                    self.column = 1;
                }
                '/' => {
                    if self.peek_next() == Some('/') {
                        let comment_start = self.current;
                        // Check for doc comment (///)
                        let is_doc = self.current + 2 < self.chars.len()
                            && self.chars[self.current + 2] == '/'
                            && (self.current + 3 >= self.chars.len()
                                || self.chars[self.current + 3] != '/');

                        // Single-line comment
                        while !self.is_at_end() && self.peek() != '\n' {
                            self.advance();
                        }

                        if self.emit_comments {
                            let text: String =
                                self.chars[comment_start..self.current].iter().collect();
                            let span = Span::new(comment_start, self.current);
                            let kind = if is_doc {
                                TokenKind::DocComment
                            } else {
                                TokenKind::LineComment
                            };
                            self.pending_comments.push(Token::new(kind, text, span));
                        }
                    } else if self.peek_next() == Some('*') {
                        // Multi-line comment
                        let comment_start = self.current;
                        let comment_start_line = self.line;
                        self.advance(); // /
                        self.advance(); // *

                        let mut terminated = false;
                        while !self.is_at_end() {
                            if self.peek() == '*' && self.peek_next() == Some('/') {
                                self.advance(); // *
                                self.advance(); // /
                                terminated = true;
                                break;
                            }
                            if self.peek() == '\n' {
                                self.line += 1;
                                self.column = 1;
                            }
                            self.advance();
                        }

                        if self.emit_comments && terminated {
                            let text: String =
                                self.chars[comment_start..self.current].iter().collect();
                            let span = Span::new(comment_start, self.current);
                            self.pending_comments.push(Token::new(
                                TokenKind::BlockComment,
                                text,
                                span,
                            ));
                        }

                        // Report error if comment was not terminated
                        if !terminated {
                            let span = Span {
                                start: self.start_pos,
                                end: self.current,
                            };
                            let snippet = self.get_line_snippet(comment_start_line);
                            self.diagnostics.push(
                                Diagnostic::error_with_code(
                                    "AT1004",
                                    "Unterminated multi-line comment",
                                    span,
                                )
                                .with_line(comment_start_line as usize)
                                .with_snippet(snippet)
                                .with_label("comment starts here")
                                .with_help("add '*/' to close the multi-line comment"),
                            );
                        }
                    } else {
                        return;
                    }
                }
                _ => return,
            }
        }
    }

    /// Scan a string literal
    // === Character navigation ===
    /// Advance to next character and return it
    pub(super) fn advance(&mut self) -> char {
        let c = self.chars[self.current];
        self.current += 1;
        self.column += 1;
        c
    }

    /// Peek at current character without advancing
    pub(super) fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.chars[self.current]
        }
    }

    /// Peek at next character (current + 1)
    pub(super) fn peek_next(&self) -> Option<char> {
        if self.current + 1 >= self.chars.len() {
            None
        } else {
            Some(self.chars[self.current + 1])
        }
    }

    /// Check if current character matches expected, and advance if so
    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.chars[self.current] != expected {
            false
        } else {
            self.advance();
            true
        }
    }

    /// Check if we've reached the end of source
    pub(super) fn is_at_end(&self) -> bool {
        self.current >= self.chars.len()
    }

    // === Token creation ===

    /// Create a token with the given kind and lexeme
    pub(super) fn make_token(&self, kind: TokenKind, lexeme: &str) -> Token {
        let span = Span {
            start: self.start_pos,
            end: self.current,
        };

        Token {
            kind,
            lexeme: lexeme.to_string(),
            span,
        }
    }

    /// Create an error token and record a diagnostic with a specific code
    pub(super) fn error_token_with_code(&mut self, code: &str, message: &str) -> Token {
        let span = Span {
            start: self.start_pos,
            end: self.current.max(self.start_pos + 1),
        };

        // Extract snippet from source for this line
        let snippet = self.get_line_snippet(self.start_line);

        // Record diagnostic
        self.diagnostics.push(
            Diagnostic::error_with_code(code, message, span)
                .with_line(self.start_line as usize)
                .with_snippet(snippet)
                .with_label("lexer error"),
        );

        Token {
            kind: TokenKind::Error,
            lexeme: message.to_string(),
            span,
        }
    }

    /// Create an error token for invalid/unexpected characters (AT1001)
    pub(super) fn error_token(&mut self, message: &str) -> Token {
        self.error_token_with_code("AT1001", message)
    }

    /// Create an error token for unterminated strings (AT1002)
    pub(super) fn error_unterminated_string(&mut self) -> Token {
        self.error_token_with_code("AT1002", "Unterminated string literal")
    }

    /// Create an error token for invalid escape sequences (AT1003)
    pub(super) fn error_invalid_escape(&mut self, escape_char: char) -> Token {
        self.error_token_with_code(
            "AT1003",
            &format!("Invalid escape sequence '\\{}'", escape_char),
        )
    }

    /// Get the source line for a given line number
    fn get_line_snippet(&self, line: u32) -> String {
        self.source
            .lines()
            .nth((line - 1) as usize)
            .unwrap_or("")
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_creation() {
        let mut lexer = Lexer::new("test");
        let (tokens, _) = lexer.tokenize();
        assert!(!tokens.is_empty()); // At minimum, should have EOF
    }

    #[test]
    fn test_empty_input() {
        let mut lexer = Lexer::new("");
        let (tokens, diagnostics) = lexer.tokenize();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::Eof);
        assert_eq!(diagnostics.len(), 0);
    }

    #[test]
    fn test_single_char_tokens() {
        let mut lexer = Lexer::new("(){}[];,:");
        let (tokens, _) = lexer.tokenize();

        assert_eq!(tokens[0].kind, TokenKind::LeftParen);
        assert_eq!(tokens[1].kind, TokenKind::RightParen);
        assert_eq!(tokens[2].kind, TokenKind::LeftBrace);
        assert_eq!(tokens[3].kind, TokenKind::RightBrace);
        assert_eq!(tokens[4].kind, TokenKind::LeftBracket);
        assert_eq!(tokens[5].kind, TokenKind::RightBracket);
        assert_eq!(tokens[6].kind, TokenKind::Semicolon);
        assert_eq!(tokens[7].kind, TokenKind::Comma);
        assert_eq!(tokens[8].kind, TokenKind::Colon);
    }

    #[test]
    fn test_operators() {
        let mut lexer = Lexer::new("+ - * / % ! == != < <= > >= && ||");
        let (tokens, _) = lexer.tokenize();

        let expected = vec![
            TokenKind::Plus,
            TokenKind::Minus,
            TokenKind::Star,
            TokenKind::Slash,
            TokenKind::Percent,
            TokenKind::Bang,
            TokenKind::EqualEqual,
            TokenKind::BangEqual,
            TokenKind::Less,
            TokenKind::LessEqual,
            TokenKind::Greater,
            TokenKind::GreaterEqual,
            TokenKind::AmpAmp,
            TokenKind::PipePipe,
        ];

        for (i, expected_kind) in expected.iter().enumerate() {
            assert_eq!(tokens[i].kind, *expected_kind);
        }
    }

    #[test]
    fn test_arrow_operator() {
        let mut lexer = Lexer::new("->");
        let (tokens, _) = lexer.tokenize();
        assert_eq!(tokens[0].kind, TokenKind::Arrow);
        assert_eq!(tokens[0].lexeme, "->");
    }

    #[test]
    fn test_keywords() {
        let mut lexer = Lexer::new("let var fn if else while for return break continue is");
        let (tokens, _) = lexer.tokenize();

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
            TokenKind::Is,
        ];

        for (i, expected_kind) in expected.iter().enumerate() {
            assert_eq!(tokens[i].kind, *expected_kind);
        }
    }

    #[test]
    fn test_reserved_keywords() {
        let mut lexer = Lexer::new("import match");
        let (tokens, _) = lexer.tokenize();

        assert_eq!(tokens[0].kind, TokenKind::Import);
        assert_eq!(tokens[1].kind, TokenKind::Match);
    }

    #[test]
    fn test_boolean_and_null() {
        let mut lexer = Lexer::new("true false null");
        let (tokens, _) = lexer.tokenize();

        assert_eq!(tokens[0].kind, TokenKind::True);
        assert_eq!(tokens[1].kind, TokenKind::False);
        assert_eq!(tokens[2].kind, TokenKind::Null);
    }

    #[test]
    fn test_identifiers() {
        let mut lexer = Lexer::new("foo bar_baz _test x123");
        let (tokens, _) = lexer.tokenize();

        assert_eq!(tokens[0].kind, TokenKind::Identifier);
        assert_eq!(tokens[0].lexeme, "foo");
        assert_eq!(tokens[1].kind, TokenKind::Identifier);
        assert_eq!(tokens[1].lexeme, "bar_baz");
        assert_eq!(tokens[2].kind, TokenKind::Identifier);
        assert_eq!(tokens[2].lexeme, "_test");
        assert_eq!(tokens[3].kind, TokenKind::Identifier);
        assert_eq!(tokens[3].lexeme, "x123");
    }

    #[test]
    fn test_numbers() {
        let mut lexer = Lexer::new("42 3.14 0 123.456");
        let (tokens, _) = lexer.tokenize();

        assert_eq!(tokens[0].kind, TokenKind::Number);
        assert_eq!(tokens[0].lexeme, "42");
        assert_eq!(tokens[1].kind, TokenKind::Number);
        assert_eq!(tokens[1].lexeme, "3.14");
        assert_eq!(tokens[2].kind, TokenKind::Number);
        assert_eq!(tokens[2].lexeme, "0");
        assert_eq!(tokens[3].kind, TokenKind::Number);
        assert_eq!(tokens[3].lexeme, "123.456");
    }

    #[test]
    fn test_scientific_notation() {
        let mut lexer = Lexer::new("1e10 1.5e-3 2.5E+10 1e308 1E-308");
        let (tokens, _) = lexer.tokenize();

        assert_eq!(tokens[0].kind, TokenKind::Number);
        assert_eq!(tokens[0].lexeme, "1e10");
        assert_eq!(tokens[1].kind, TokenKind::Number);
        assert_eq!(tokens[1].lexeme, "1.5e-3");
        assert_eq!(tokens[2].kind, TokenKind::Number);
        assert_eq!(tokens[2].lexeme, "2.5E+10");
        assert_eq!(tokens[3].kind, TokenKind::Number);
        assert_eq!(tokens[3].lexeme, "1e308");
        assert_eq!(tokens[4].kind, TokenKind::Number);
        assert_eq!(tokens[4].lexeme, "1E-308");
    }

    #[test]
    fn test_scientific_notation_invalid() {
        let mut lexer = Lexer::new("1e 1e+ 1e-");
        let (tokens, diags) = lexer.tokenize();

        // All three should be errors (exponent with no digits)
        assert_eq!(tokens[0].kind, TokenKind::Error);
        assert_eq!(tokens[1].kind, TokenKind::Error);
        assert_eq!(tokens[2].kind, TokenKind::Error);
        assert_eq!(diags.len(), 3);
    }

    #[test]
    fn test_compound_assignment_operators() {
        let mut lexer = Lexer::new("+= -= *= /= %=");
        let (tokens, _) = lexer.tokenize();

        assert_eq!(tokens[0].kind, TokenKind::PlusEqual);
        assert_eq!(tokens[0].lexeme, "+=");
        assert_eq!(tokens[1].kind, TokenKind::MinusEqual);
        assert_eq!(tokens[1].lexeme, "-=");
        assert_eq!(tokens[2].kind, TokenKind::StarEqual);
        assert_eq!(tokens[2].lexeme, "*=");
        assert_eq!(tokens[3].kind, TokenKind::SlashEqual);
        assert_eq!(tokens[3].lexeme, "/=");
        assert_eq!(tokens[4].kind, TokenKind::PercentEqual);
        assert_eq!(tokens[4].lexeme, "%=");
    }

    #[test]
    fn test_increment_decrement_operators() {
        let mut lexer = Lexer::new("++ --");
        let (tokens, _) = lexer.tokenize();

        assert_eq!(tokens[0].kind, TokenKind::PlusPlus);
        assert_eq!(tokens[0].lexeme, "++");
        assert_eq!(tokens[1].kind, TokenKind::MinusMinus);
        assert_eq!(tokens[1].lexeme, "--");
    }

    #[test]
    fn test_operator_disambiguation() {
        // Make sure += doesn't get lexed as + =
        let mut lexer = Lexer::new("x+=1");
        let (tokens, _) = lexer.tokenize();

        assert_eq!(tokens[0].kind, TokenKind::Identifier);
        assert_eq!(tokens[1].kind, TokenKind::PlusEqual);
        assert_eq!(tokens[2].kind, TokenKind::Number);

        // Make sure ++ doesn't get lexed as + +
        let mut lexer = Lexer::new("x++");
        let (tokens, _) = lexer.tokenize();

        assert_eq!(tokens[0].kind, TokenKind::Identifier);
        assert_eq!(tokens[1].kind, TokenKind::PlusPlus);
    }

    #[test]
    fn test_single_line_comment() {
        let mut lexer = Lexer::new("let x = 5; // This is a comment\nlet y = 10;");
        let (tokens, _) = lexer.tokenize();

        // Should skip the comment entirely
        assert_eq!(tokens[0].kind, TokenKind::Let);
        assert_eq!(tokens[1].kind, TokenKind::Identifier);
        assert_eq!(tokens[2].kind, TokenKind::Equal);
        assert_eq!(tokens[3].kind, TokenKind::Number);
        assert_eq!(tokens[4].kind, TokenKind::Semicolon);
        assert_eq!(tokens[5].kind, TokenKind::Let);
        assert_eq!(tokens[6].kind, TokenKind::Identifier);
    }

    #[test]
    fn test_multi_line_comment() {
        let mut lexer = Lexer::new("let x = /* comment */ 5;");
        let (tokens, _) = lexer.tokenize();

        assert_eq!(tokens[0].kind, TokenKind::Let);
        assert_eq!(tokens[1].kind, TokenKind::Identifier);
        assert_eq!(tokens[2].kind, TokenKind::Equal);
        assert_eq!(tokens[3].kind, TokenKind::Number);
        assert_eq!(tokens[4].kind, TokenKind::Semicolon);
    }

    #[test]
    fn test_single_ampersand_is_valid() {
        // & is valid: used for intersection types (A & B)
        let mut lexer = Lexer::new("&");
        let (tokens, diagnostics) = lexer.tokenize();

        assert_eq!(tokens[0].kind, TokenKind::Ampersand);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_single_pipe_is_valid() {
        // | is valid: used for union types (A | B)
        let mut lexer = Lexer::new("|");
        let (tokens, diagnostics) = lexer.tokenize();

        assert_eq!(tokens[0].kind, TokenKind::Pipe);
        assert!(diagnostics.is_empty());
    }

    // === Edge Case Tests (Phase 07) ===
    // These tests validate lexer behavior for edge cases and invalid inputs

    #[test]
    fn test_unterminated_string_basic() {
        let mut lexer = Lexer::new(r#""hello"#);
        let (tokens, diagnostics) = lexer.tokenize();

        assert_eq!(tokens[0].kind, TokenKind::Error);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, "AT1002");
        assert!(diagnostics[0].message.contains("Unterminated string"));
    }

    #[test]
    fn test_unterminated_string_with_newline() {
        let mut lexer = Lexer::new("\"hello\nworld");
        let (tokens, diagnostics) = lexer.tokenize();

        assert_eq!(tokens[0].kind, TokenKind::Error);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, "AT1002");
    }

    #[test]
    fn test_unterminated_string_with_escape() {
        let mut lexer = Lexer::new(r#""hello\n"#);
        let (tokens, diagnostics) = lexer.tokenize();

        assert_eq!(tokens[0].kind, TokenKind::Error);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, "AT1002");
    }

    #[test]
    fn test_unterminated_string_ends_with_backslash() {
        let mut lexer = Lexer::new(r#""hello\"#);
        let (tokens, diagnostics) = lexer.tokenize();

        assert_eq!(tokens[0].kind, TokenKind::Error);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, "AT1002");
    }

    #[test]
    fn test_invalid_escape_sequence_x() {
        let mut lexer = Lexer::new(r#""hello\x""#);
        let (tokens, diagnostics) = lexer.tokenize();

        assert_eq!(tokens[0].kind, TokenKind::Error);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, "AT1003");
        assert!(diagnostics[0].message.contains("\\x"));
    }

    #[test]
    fn test_invalid_escape_sequence_0() {
        let mut lexer = Lexer::new(r#""test\0""#);
        let (tokens, diagnostics) = lexer.tokenize();

        assert_eq!(tokens[0].kind, TokenKind::Error);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, "AT1003");
        assert!(diagnostics[0].message.contains("\\0"));
    }

    #[test]
    fn test_invalid_escape_sequence_u() {
        let mut lexer = Lexer::new(r#""unicode\u1234""#);
        let (tokens, diagnostics) = lexer.tokenize();

        assert_eq!(tokens[0].kind, TokenKind::Error);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, "AT1003");
        assert!(diagnostics[0].message.contains("\\u"));
    }

    #[test]
    fn test_invalid_escape_sequence_a() {
        let mut lexer = Lexer::new(r#""alert\a""#);
        let (tokens, diagnostics) = lexer.tokenize();

        assert_eq!(tokens[0].kind, TokenKind::Error);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, "AT1003");
        assert!(diagnostics[0].message.contains("\\a"));
    }

    #[test]
    fn test_valid_escape_sequences() {
        let mut lexer = Lexer::new(r#""a\nb\tc\rd\\e\"f""#);
        let (tokens, diagnostics) = lexer.tokenize();

        assert_eq!(tokens[0].kind, TokenKind::String);
        assert_eq!(diagnostics.len(), 0);
        assert_eq!(tokens[0].lexeme, "a\nb\tc\rd\\e\"f");
    }

    #[test]
    fn test_unexpected_character_at() {
        let mut lexer = Lexer::new("@");
        let (tokens, diagnostics) = lexer.tokenize();

        assert_eq!(tokens[0].kind, TokenKind::Error);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, "AT1001");
        assert!(diagnostics[0].message.contains("@"));
    }

    #[test]
    fn test_unexpected_character_hash() {
        let mut lexer = Lexer::new("#");
        let (tokens, diagnostics) = lexer.tokenize();

        assert_eq!(tokens[0].kind, TokenKind::Error);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, "AT1001");
        assert!(diagnostics[0].message.contains("#"));
    }

    #[test]
    fn test_unexpected_character_dollar() {
        let mut lexer = Lexer::new("$");
        let (tokens, diagnostics) = lexer.tokenize();

        assert_eq!(tokens[0].kind, TokenKind::Error);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, "AT1001");
    }

    #[test]
    fn test_unexpected_character_tilde() {
        let mut lexer = Lexer::new("~");
        let (tokens, diagnostics) = lexer.tokenize();

        assert_eq!(tokens[0].kind, TokenKind::Error);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, "AT1001");
    }

    #[test]
    fn test_unexpected_character_backtick() {
        let mut lexer = Lexer::new("`");
        let (tokens, diagnostics) = lexer.tokenize();

        assert_eq!(tokens[0].kind, TokenKind::Error);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, "AT1001");
    }

    #[test]
    fn test_ampersand_token_kind() {
        // & produces Ampersand token for intersection types (A & B)
        let mut lexer = Lexer::new("&");
        let (tokens, diagnostics) = lexer.tokenize();

        assert_eq!(tokens[0].kind, TokenKind::Ampersand);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_pipe_token_kind() {
        // | produces Pipe token for union types (A | B)
        let mut lexer = Lexer::new("|");
        let (tokens, diagnostics) = lexer.tokenize();

        assert_eq!(tokens[0].kind, TokenKind::Pipe);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_unterminated_multiline_comment() {
        let mut lexer = Lexer::new("/* This comment never ends");
        let (tokens, diagnostics) = lexer.tokenize();

        // Should have EOF token
        assert_eq!(tokens[0].kind, TokenKind::Eof);
        // Should have diagnostic for unterminated comment
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, "AT1004");
        assert!(diagnostics[0]
            .message
            .contains("Unterminated multi-line comment"));
    }

    #[test]
    fn test_unterminated_multiline_comment_with_content() {
        let mut lexer = Lexer::new("let x = 5;\n/* Comment\nmore comment");
        let (tokens, diagnostics) = lexer.tokenize();

        // Should tokenize "let x = 5;" then hit unterminated comment
        assert_eq!(tokens[0].kind, TokenKind::Let);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, "AT1004");
    }

    #[test]
    fn test_multiple_errors_in_source() {
        let mut lexer = Lexer::new(r#""unterminated @ $ "#);
        let (_tokens, diagnostics) = lexer.tokenize();

        // Should report multiple errors
        assert!(!diagnostics.is_empty());
        // First error should be unterminated string
        assert_eq!(diagnostics[0].code, "AT1002");
    }

    #[test]
    fn test_error_recovery_continues_lexing() {
        let mut lexer = Lexer::new("@ let x = 5;");
        let (tokens, diagnostics) = lexer.tokenize();

        // Should report error for @ but continue lexing
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, "AT1001");

        // Should still lex "let x = 5;"
        assert!(tokens.iter().any(|t| t.kind == TokenKind::Let));
        assert!(tokens.iter().any(|t| t.kind == TokenKind::Identifier));
    }

    #[test]
    fn test_precise_span_for_invalid_character() {
        let mut lexer = Lexer::new("let @ x");
        let (_tokens, diagnostics) = lexer.tokenize();

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, "AT1001");

        // The error should be reported with proper location
        // Column should be around 5 (1-indexed, after "let ")
        assert!(diagnostics[0].column >= 5);
        assert!(diagnostics[0].length >= 1);
    }

    #[test]
    fn test_precise_span_for_unterminated_string() {
        let mut lexer = Lexer::new(r#"let x = "test"#);
        let (_tokens, diagnostics) = lexer.tokenize();

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, "AT1002");

        // The error should be reported with proper location
        // Column should be around 9 (1-indexed, at opening quote)
        assert!(diagnostics[0].column >= 9);
        assert!(diagnostics[0].length >= 4); // "test
    }

    #[test]
    fn test_lexer_reports_first_error_cleanly() {
        // Ensure lexer reports the first error without crashing
        let test_cases = vec![
            (r#""unterminated"#, "AT1002"),
            (r#""invalid\x""#, "AT1003"),
            ("@", "AT1001"),
            ("/* unterminated", "AT1004"),
        ];

        for (source, expected_code) in test_cases {
            let mut lexer = Lexer::new(source);
            let (_, diagnostics) = lexer.tokenize();

            assert!(!diagnostics.is_empty(), "Expected error for: {}", source);
            assert_eq!(
                diagnostics[0].code, expected_code,
                "Wrong code for: {}",
                source
            );
        }
    }
}
