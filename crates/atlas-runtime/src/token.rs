//! Token types for lexical analysis
//!
//! Defines all token types recognized by the Atlas lexer.

use crate::span::Span;
use serde::{Deserialize, Serialize};

/// Token type produced by the lexer
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Token {
    /// The kind of token
    pub kind: TokenKind,
    /// The source text of this token
    pub lexeme: String,
    /// Source location
    pub span: Span,
}

impl Token {
    /// Create a new token
    pub fn new(kind: TokenKind, lexeme: impl Into<String>, span: Span) -> Self {
        Self {
            kind,
            lexeme: lexeme.into(),
            span,
        }
    }
}

/// Classification of token types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TokenKind {
    // Literals
    /// Number literal (42, 3.14)
    Number,
    /// String literal ("hello")
    String,
    /// `true` keyword
    True,
    /// `false` keyword
    False,
    /// `null` keyword
    Null,
    /// Identifier
    Identifier,

    // Keywords
    /// `let` keyword (immutable variable)
    Let,
    /// `var` keyword (mutable variable)
    Var,
    /// `fn` keyword (function declaration)
    Fn,
    /// `if` keyword
    If,
    /// `else` keyword
    Else,
    /// `while` keyword
    While,
    /// `for` keyword
    For,
    /// `return` keyword
    Return,
    /// `break` keyword
    Break,
    /// `continue` keyword
    Continue,

    // Reserved for future
    /// `import` keyword (reserved, not in v0.1)
    Import,
    /// `match` keyword (reserved, not in v0.1)
    Match,

    // Operators
    /// `+` (addition)
    Plus,
    /// `-` (subtraction or negation)
    Minus,
    /// `*` (multiplication)
    Star,
    /// `/` (division)
    Slash,
    /// `%` (modulo)
    Percent,
    /// `!` (logical not)
    Bang,
    /// `==` (equality)
    EqualEqual,
    /// `!=` (inequality)
    BangEqual,
    /// `<` (less than)
    Less,
    /// `<=` (less than or equal)
    LessEqual,
    /// `>` (greater than)
    Greater,
    /// `>=` (greater than or equal)
    GreaterEqual,
    /// `&&` (logical and)
    AmpAmp,
    /// `||` (logical or)
    PipePipe,

    // Compound assignment operators
    /// `+=` (add and assign)
    PlusEqual,
    /// `-=` (subtract and assign)
    MinusEqual,
    /// `*=` (multiply and assign)
    StarEqual,
    /// `/=` (divide and assign)
    SlashEqual,
    /// `%=` (modulo and assign)
    PercentEqual,

    // Increment/decrement operators
    /// `++` (increment)
    PlusPlus,
    /// `--` (decrement)
    MinusMinus,

    // Punctuation
    /// `=` (assignment)
    Equal,
    /// `(` (left parenthesis)
    LeftParen,
    /// `)` (right parenthesis)
    RightParen,
    /// `{` (left brace)
    LeftBrace,
    /// `}` (right brace)
    RightBrace,
    /// `[` (left bracket)
    LeftBracket,
    /// `]` (right bracket)
    RightBracket,
    /// `;` (semicolon)
    Semicolon,
    /// `,` (comma)
    Comma,
    /// `:` (colon)
    Colon,
    /// `->` (arrow for function return type)
    Arrow,
    /// `=>` (fat arrow for match arms)
    FatArrow,
    /// `_` (underscore for wildcard patterns)
    Underscore,

    // Special
    /// End of file
    Eof,
    /// Lexer error
    Error,
}

impl TokenKind {
    /// Check if a string is a keyword and return its token kind
    pub fn is_keyword(s: &str) -> Option<TokenKind> {
        match s {
            "let" => Some(TokenKind::Let),
            "var" => Some(TokenKind::Var),
            "fn" => Some(TokenKind::Fn),
            "if" => Some(TokenKind::If),
            "else" => Some(TokenKind::Else),
            "while" => Some(TokenKind::While),
            "for" => Some(TokenKind::For),
            "return" => Some(TokenKind::Return),
            "break" => Some(TokenKind::Break),
            "continue" => Some(TokenKind::Continue),
            "true" => Some(TokenKind::True),
            "false" => Some(TokenKind::False),
            "null" => Some(TokenKind::Null),
            "import" => Some(TokenKind::Import),
            "match" => Some(TokenKind::Match),
            _ => None,
        }
    }

    /// Get the string representation of this token kind
    pub fn as_str(&self) -> &'static str {
        match self {
            TokenKind::Number => "number",
            TokenKind::String => "string",
            TokenKind::True => "true",
            TokenKind::False => "false",
            TokenKind::Null => "null",
            TokenKind::Identifier => "identifier",
            TokenKind::Let => "let",
            TokenKind::Var => "var",
            TokenKind::Fn => "fn",
            TokenKind::If => "if",
            TokenKind::Else => "else",
            TokenKind::While => "while",
            TokenKind::For => "for",
            TokenKind::Return => "return",
            TokenKind::Break => "break",
            TokenKind::Continue => "continue",
            TokenKind::Import => "import",
            TokenKind::Match => "match",
            TokenKind::Plus => "+",
            TokenKind::Minus => "-",
            TokenKind::Star => "*",
            TokenKind::Slash => "/",
            TokenKind::Percent => "%",
            TokenKind::Bang => "!",
            TokenKind::EqualEqual => "==",
            TokenKind::BangEqual => "!=",
            TokenKind::Less => "<",
            TokenKind::LessEqual => "<=",
            TokenKind::Greater => ">",
            TokenKind::GreaterEqual => ">=",
            TokenKind::AmpAmp => "&&",
            TokenKind::PipePipe => "||",
            TokenKind::PlusEqual => "+=",
            TokenKind::MinusEqual => "-=",
            TokenKind::StarEqual => "*=",
            TokenKind::SlashEqual => "/=",
            TokenKind::PercentEqual => "%=",
            TokenKind::PlusPlus => "++",
            TokenKind::MinusMinus => "--",
            TokenKind::Equal => "=",
            TokenKind::LeftParen => "(",
            TokenKind::RightParen => ")",
            TokenKind::LeftBrace => "{",
            TokenKind::RightBrace => "}",
            TokenKind::LeftBracket => "[",
            TokenKind::RightBracket => "]",
            TokenKind::Semicolon => ";",
            TokenKind::Comma => ",",
            TokenKind::Colon => ":",
            TokenKind::Arrow => "->",
            TokenKind::FatArrow => "=>",
            TokenKind::Underscore => "_",
            TokenKind::Eof => "EOF",
            TokenKind::Error => "error",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_creation() {
        let token = Token::new(TokenKind::Number, "42", Span::new(0, 2));
        assert_eq!(token.kind, TokenKind::Number);
        assert_eq!(token.lexeme, "42");
        assert_eq!(token.span, Span::new(0, 2));
    }

    #[test]
    fn test_keyword_detection() {
        assert_eq!(TokenKind::is_keyword("let"), Some(TokenKind::Let));
        assert_eq!(TokenKind::is_keyword("var"), Some(TokenKind::Var));
        assert_eq!(TokenKind::is_keyword("fn"), Some(TokenKind::Fn));
        assert_eq!(TokenKind::is_keyword("if"), Some(TokenKind::If));
        assert_eq!(TokenKind::is_keyword("else"), Some(TokenKind::Else));
        assert_eq!(TokenKind::is_keyword("while"), Some(TokenKind::While));
        assert_eq!(TokenKind::is_keyword("for"), Some(TokenKind::For));
        assert_eq!(TokenKind::is_keyword("return"), Some(TokenKind::Return));
        assert_eq!(TokenKind::is_keyword("break"), Some(TokenKind::Break));
        assert_eq!(TokenKind::is_keyword("continue"), Some(TokenKind::Continue));
        assert_eq!(TokenKind::is_keyword("true"), Some(TokenKind::True));
        assert_eq!(TokenKind::is_keyword("false"), Some(TokenKind::False));
        assert_eq!(TokenKind::is_keyword("null"), Some(TokenKind::Null));
    }

    #[test]
    fn test_reserved_keywords() {
        assert_eq!(TokenKind::is_keyword("import"), Some(TokenKind::Import));
        assert_eq!(TokenKind::is_keyword("match"), Some(TokenKind::Match));
    }

    #[test]
    fn test_non_keyword() {
        assert_eq!(TokenKind::is_keyword("foo"), None);
        assert_eq!(TokenKind::is_keyword("x"), None);
        assert_eq!(TokenKind::is_keyword("Let"), None); // Case-sensitive
    }

    #[test]
    fn test_token_kind_as_str() {
        assert_eq!(TokenKind::Let.as_str(), "let");
        assert_eq!(TokenKind::Plus.as_str(), "+");
        assert_eq!(TokenKind::EqualEqual.as_str(), "==");
        assert_eq!(TokenKind::Arrow.as_str(), "->");
    }

    #[test]
    fn test_all_operators() {
        let operators = vec![
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

        for op in operators {
            assert!(!op.as_str().is_empty());
        }
    }

    #[test]
    fn test_all_punctuation() {
        let punctuation = vec![
            TokenKind::Equal,
            TokenKind::LeftParen,
            TokenKind::RightParen,
            TokenKind::LeftBrace,
            TokenKind::RightBrace,
            TokenKind::LeftBracket,
            TokenKind::RightBracket,
            TokenKind::Semicolon,
            TokenKind::Comma,
            TokenKind::Colon,
            TokenKind::Arrow,
        ];

        for punct in punctuation {
            assert!(!punct.as_str().is_empty());
        }
    }
}
