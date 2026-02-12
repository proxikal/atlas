//! Lexical analysis (tokenization)
//!
//! The lexer converts Atlas source code into a stream of tokens with accurate span information.

use crate::diagnostic::Diagnostic;
use crate::span::Span;
use crate::token::{Token, TokenKind};

/// Lexer state for tokenizing source code
pub struct Lexer {
    /// Original source code
    source: String,
    /// Characters of source code
    chars: Vec<char>,
    /// Current position in chars
    current: usize,
    /// Current line number (1-indexed)
    line: u32,
    /// Current column number (1-indexed)
    column: u32,
    /// Start position of current token
    start_pos: usize,
    /// Start line of current token
    start_line: u32,
    /// Start column of current token
    start_column: u32,
    /// Collected diagnostics
    diagnostics: Vec<Diagnostic>,
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
            '+' => self.make_token(TokenKind::Plus, "+"),
            '*' => self.make_token(TokenKind::Star, "*"),
            '/' => self.make_token(TokenKind::Slash, "/"),
            '%' => self.make_token(TokenKind::Percent, "%"),

            // Two-character tokens
            '-' => {
                if self.match_char('>') {
                    self.make_token(TokenKind::Arrow, "->")
                } else {
                    self.make_token(TokenKind::Minus, "-")
                }
            }
            '=' => {
                if self.match_char('=') {
                    self.make_token(TokenKind::EqualEqual, "==")
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
                    self.error_token("Unexpected character '&', did you mean '&&'?")
                }
            }
            '|' => {
                if self.match_char('|') {
                    self.make_token(TokenKind::PipePipe, "||")
                } else {
                    self.error_token("Unexpected character '|', did you mean '||'?")
                }
            }

            // String literals
            '"' => self.string(),

            // Numbers
            c if c.is_ascii_digit() => self.number(),

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
                        // Single-line comment
                        while !self.is_at_end() && self.peek() != '\n' {
                            self.advance();
                        }
                    } else if self.peek_next() == Some('*') {
                        // Multi-line comment
                        self.advance(); // /
                        self.advance(); // *

                        while !self.is_at_end() {
                            if self.peek() == '*' && self.peek_next() == Some('/') {
                                self.advance(); // *
                                self.advance(); // /
                                break;
                            }
                            if self.peek() == '\n' {
                                self.line += 1;
                                self.column = 1;
                            }
                            self.advance();
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
    fn string(&mut self) -> Token {
        let mut value = String::new();

        while !self.is_at_end() && self.peek() != '"' {
            if self.peek() == '\n' {
                self.line += 1;
                self.column = 1;
            }

            if self.peek() == '\\' {
                self.advance(); // consume backslash
                if self.is_at_end() {
                    return self.error_token("Unterminated string literal");
                }

                let escaped = match self.peek() {
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    '\\' => '\\',
                    '"' => '"',
                    c => {
                        return self.error_token(&format!("Invalid escape sequence '\\{}'", c));
                    }
                };

                self.advance(); // consume escaped character
                value.push(escaped);
            } else {
                value.push(self.advance());
            }
        }

        if self.is_at_end() {
            return self.error_token("Unterminated string literal");
        }

        self.advance(); // Closing "
        self.make_token(TokenKind::String, &value)
    }

    /// Scan a number literal (integer or float)
    fn number(&mut self) -> Token {
        let start = self.current - 1; // -1 because we already advanced past first digit

        // Consume all digits
        while !self.is_at_end() && self.peek().is_ascii_digit() {
            self.advance();
        }

        // Check for decimal point
        if !self.is_at_end() && self.peek() == '.' {
            // Look ahead to ensure there's a digit after the dot
            if let Some(c) = self.peek_next() {
                if c.is_ascii_digit() {
                    self.advance(); // consume .

                    // Consume fractional digits
                    while !self.is_at_end() && self.peek().is_ascii_digit() {
                        self.advance();
                    }
                }
            }
        }

        let lexeme: String = self.chars[start..self.current].iter().collect();
        self.make_token(TokenKind::Number, &lexeme)
    }

    /// Scan an identifier or keyword
    fn identifier(&mut self) -> Token {
        let start = self.current - 1; // -1 because we already advanced past first char

        while !self.is_at_end() {
            let c = self.peek();
            if c.is_alphanumeric() || c == '_' {
                self.advance();
            } else {
                break;
            }
        }

        let lexeme: String = self.chars[start..self.current].iter().collect();

        // Check if it's a keyword
        let kind = TokenKind::is_keyword(&lexeme).unwrap_or(TokenKind::Identifier);

        self.make_token(kind, &lexeme)
    }

    // === Character navigation ===

    /// Advance to next character and return it
    fn advance(&mut self) -> char {
        let c = self.chars[self.current];
        self.current += 1;
        self.column += 1;
        c
    }

    /// Peek at current character without advancing
    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.chars[self.current]
        }
    }

    /// Peek at next character (current + 1)
    fn peek_next(&self) -> Option<char> {
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
    fn is_at_end(&self) -> bool {
        self.current >= self.chars.len()
    }

    // === Token creation ===

    /// Create a token with the given kind and lexeme
    fn make_token(&self, kind: TokenKind, lexeme: &str) -> Token {
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

    /// Create an error token and record a diagnostic
    fn error_token(&mut self, message: &str) -> Token {
        let span = Span {
            start: self.start_pos,
            end: self.current.max(self.start_pos + 1),
        };

        // Extract snippet from source for this line
        let snippet = self.get_line_snippet(self.start_line);

        // Record diagnostic
        self.diagnostics.push(
            Diagnostic::error_with_code("AT1000", message, span)
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
        assert!(tokens.len() >= 1); // At minimum, should have EOF
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
        let mut lexer = Lexer::new("let var fn if else while for return break continue");
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
    fn test_invalid_single_ampersand() {
        let mut lexer = Lexer::new("&");
        let (tokens, diagnostics) = lexer.tokenize();

        assert_eq!(tokens[0].kind, TokenKind::Error);
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("&"));
    }

    #[test]
    fn test_invalid_single_pipe() {
        let mut lexer = Lexer::new("|");
        let (tokens, diagnostics) = lexer.tokenize();

        assert_eq!(tokens[0].kind, TokenKind::Error);
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("|"));
    }
}
