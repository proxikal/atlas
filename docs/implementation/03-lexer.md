# Lexer Implementation

## Token Definition

```rust
// token.rs
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    // Literals
    Number, String, True, False, Null, Identifier,

    // Keywords
    Let, Var, Fn, If, Else, While, For, Return, Break, Continue,
    Import, Match,  // Reserved for future

    // Operators
    Plus, Minus, Star, Slash, Percent, Bang,
    EqualEqual, BangEqual, Less, LessEqual, Greater, GreaterEqual,
    AmpAmp, PipePipe,

    // Compound assignment & increment/decrement
    PlusEqual, MinusEqual, StarEqual, SlashEqual, PercentEqual,
    PlusPlus, MinusMinus,

    // Punctuation
    Equal, LeftParen, RightParen, LeftBrace, RightBrace,
    LeftBracket, RightBracket, Semicolon, Comma, Colon, Arrow,

    // Special
    Eof, Error,
}

impl TokenKind {
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
}
```

## Lexer State Machine

```rust
// lexer/ module (lexer/mod.rs + lexer/literals.rs)
pub struct Lexer {
    source: String,
    chars: Vec<char>,
    current: usize,
    line: u32,
    column: u32,
    start_pos: usize,
    diagnostics: Vec<Diagnostic>,
}

impl Lexer {
    pub fn new(source: String) -> Self {
        let chars: Vec<char> = source.chars().collect();
        Self {
            source,
            chars,
            current: 0,
            line: 1,
            column: 1,
            start_pos: 0,
            diagnostics: Vec::new(),
        }
    }

    pub fn tokenize(&mut self) -> (Vec<Token>, Vec<Diagnostic>) {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token();
            let is_eof = token.kind == TokenKind::Eof;
            tokens.push(token);
            if is_eof { break; }
        }
        (tokens, std::mem::take(&mut self.diagnostics))
    }

    fn next_token(&mut self) -> Token {
        self.skip_whitespace_and_comments();
        self.start_pos = self.current;

        if self.is_at_end() {
            return self.make_token(TokenKind::Eof, "");
        }

        let c = self.advance();
        match c {
            '(' => self.make_token(TokenKind::LeftParen, "("),
            ')' => self.make_token(TokenKind::RightParen, ")"),
            '{' => self.make_token(TokenKind::LeftBrace, "{"),
            '}' => self.make_token(TokenKind::RightBrace, "}"),
            '[' => self.make_token(TokenKind::LeftBracket, "["),
            ']' => self.make_token(TokenKind::RightBracket, "]"),
            ';' => self.make_token(TokenKind::Semicolon, ";"),
            ',' => self.make_token(TokenKind::Comma, ","),
            ':' => self.make_token(TokenKind::Colon, ":"),
            '+' => {
                if self.match_char('+') {
                    self.make_token(TokenKind::PlusPlus, "++")
                } else if self.match_char('=') {
                    self.make_token(TokenKind::PlusEqual, "+=")
                } else {
                    self.make_token(TokenKind::Plus, "+")
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
            '-' => {
                if self.match_char('-') {
                    self.make_token(TokenKind::MinusMinus, "--")
                } else if self.match_char('>') {
                    self.make_token(TokenKind::Arrow, "->")
                } else if self.match_char('=') {
                    self.make_token(TokenKind::MinusEqual, "-=")
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
                    self.error_token("Unexpected character '&'")
                }
            }
            '|' => {
                if self.match_char('|') {
                    self.make_token(TokenKind::PipePipe, "||")
                } else {
                    self.error_token("Unexpected character '|'")
                }
            }
            '"' => self.string(),
            c if c.is_ascii_digit() => self.number(),
            c if c.is_alphabetic() || c == '_' => self.identifier(),
            _ => self.error_token(&format!("Unexpected character '{}'", c)),
        }
    }
}
```

## String Handling

```rust
fn string(&mut self) -> Token {
    let mut value = String::new();
    while !self.is_at_end() && self.peek() != '"' {
        if self.peek() == '\n' {
            self.line += 1;
            self.column = 1;
        }
        if self.peek() == '\\' {
            self.advance();
            if self.is_at_end() {
                return self.error_token("Unterminated string");
            }
            let escaped = match self.advance() {
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                '\\' => '\\',
                '"' => '"',
                c => return self.error_token(&format!("Invalid escape sequence '\\{}'", c)),
            };
            value.push(escaped);
        } else {
            value.push(self.advance());
        }
    }

    if self.is_at_end() {
        return self.error_token("Unterminated string");
    }

    self.advance(); // Closing "
    self.make_token(TokenKind::String, &value)
}
```

## Comment Handling

```rust
fn skip_whitespace_and_comments(&mut self) {
    loop {
        if self.is_at_end() { return; }
        match self.peek() {
            ' ' | '\r' | '\t' => { self.advance(); }
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
```

## Key Design Decisions

- **Single-pass:** Lexer completes in one forward scan
- **Error recovery:** Invalid tokens become `TokenKind::Error`, lexing continues
- **Span tracking:** Every token has accurate line/column for diagnostics
- **String escapes:** `\n \r \t \\ \"` only (v0.1)
- **Comments:** Both `//` and `/* */` supported
- **Operator lookahead:** Uses `match_char()` to distinguish `+`, `++`, and `+=`
- **Scientific notation:** Number format supports `1.5e10`, `2e-3`, etc.
