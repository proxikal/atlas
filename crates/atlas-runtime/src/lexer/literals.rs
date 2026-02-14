//! Literal parsing for the lexer

use crate::lexer::Lexer;
use crate::token::{Token, TokenKind};

impl Lexer {
    /// Scan a string literal
    pub(super) fn string(&mut self) -> Token {
        let mut value = String::new();
        let mut has_error = false;
        let mut error_token = None;

        while !self.is_at_end() && self.peek() != '"' {
            if self.peek() == '\n' {
                self.line += 1;
                self.column = 1;
            }

            if self.peek() == '\\' {
                self.advance(); // consume backslash
                if self.is_at_end() {
                    return self.error_unterminated_string();
                }

                let escape_char = self.peek();
                let escaped = match escape_char {
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    '\\' => '\\',
                    '"' => '"',
                    _ => {
                        // Record error but continue parsing to find end of string
                        if !has_error {
                            error_token = Some(self.error_invalid_escape(escape_char));
                            has_error = true;
                        }
                        self.advance(); // consume the invalid character
                        continue; // Skip adding to value
                    }
                };

                self.advance(); // consume escaped character
                value.push(escaped);
            } else {
                value.push(self.advance());
            }
        }

        if self.is_at_end() {
            return self.error_unterminated_string();
        }

        self.advance(); // Closing "

        // If we had an error, return that instead of a valid token
        if let Some(err) = error_token {
            err
        } else {
            self.make_token(TokenKind::String, &value)
        }
    }

    /// Scan a number literal (integer, float, or scientific notation)
    pub(super) fn number(&mut self) -> Token {
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

        // Check for scientific notation (e or E)
        if !self.is_at_end() && (self.peek() == 'e' || self.peek() == 'E') {
            self.advance(); // consume e/E

            // Optional + or - sign
            if !self.is_at_end() && (self.peek() == '+' || self.peek() == '-') {
                self.advance();
            }

            // Must have at least one digit in exponent
            if self.is_at_end() || !self.peek().is_ascii_digit() {
                return self.error_token("Invalid number: exponent requires digits");
            }

            // Consume exponent digits
            while !self.is_at_end() && self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        let lexeme: String = self.chars[start..self.current].iter().collect();
        self.make_token(TokenKind::Number, &lexeme)
    }

    /// Scan an identifier or keyword
    pub(super) fn identifier(&mut self) -> Token {
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

        // Check for standalone underscore (wildcard pattern)
        if lexeme == "_" {
            return self.make_token(TokenKind::Underscore, "_");
        }

        // Check if it's a keyword
        let kind = TokenKind::is_keyword(&lexeme).unwrap_or(TokenKind::Identifier);

        self.make_token(kind, &lexeme)
    }
}
