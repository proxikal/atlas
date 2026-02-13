# Parser Strategy

## Approach

- **Expressions:** Pratt parsing (precedence climbing)
- **Statements:** Recursive descent
- **Error recovery:** Synchronize on statement boundaries

## Parser Structure

```rust
// parser/ module (parser/mod.rs + parser/stmt.rs + parser/expr.rs)
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    diagnostics: Vec<Diagnostic>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0, diagnostics: Vec::new() }
    }

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

    fn synchronize(&mut self) {
        while !self.is_at_end() {
            if self.previous().kind == TokenKind::Semicolon {
                return;
            }
            match self.peek().kind {
                TokenKind::Fn | TokenKind::Let | TokenKind::Var |
                TokenKind::If | TokenKind::While | TokenKind::For |
                TokenKind::Return => return,
                _ => { self.advance(); }
            }
        }
    }
}
```

## Pratt Parsing for Expressions

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Precedence {
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
    fn parse_expression(&mut self) -> Result<Expr, ()> {
        self.parse_precedence(Precedence::Lowest)
    }

    fn parse_precedence(&mut self, precedence: Precedence) -> Result<Expr, ()> {
        let mut left = self.parse_prefix()?;

        while precedence < self.current_precedence() {
            left = self.parse_infix(left)?;
        }

        Ok(left)
    }

    fn parse_prefix(&mut self) -> Result<Expr, ()> {
        match self.peek().kind {
            TokenKind::Number => self.parse_number(),
            TokenKind::String => self.parse_string(),
            TokenKind::True | TokenKind::False => self.parse_bool(),
            TokenKind::Null => self.parse_null(),
            TokenKind::Identifier => self.parse_identifier(),
            TokenKind::LeftParen => self.parse_group(),
            TokenKind::LeftBracket => self.parse_array_literal(),
            TokenKind::Minus | TokenKind::Bang => self.parse_unary(),
            _ => {
                self.error("Expected expression");
                Err(())
            }
        }
    }

    fn parse_infix(&mut self, left: Expr) -> Result<Expr, ()> {
        match self.peek().kind {
            TokenKind::Plus | TokenKind::Minus | TokenKind::Star |
            TokenKind::Slash | TokenKind::Percent | TokenKind::EqualEqual |
            TokenKind::BangEqual | TokenKind::Less | TokenKind::LessEqual |
            TokenKind::Greater | TokenKind::GreaterEqual |
            TokenKind::AmpAmp | TokenKind::PipePipe => {
                self.parse_binary(left)
            }
            TokenKind::LeftParen => self.parse_call(left),
            TokenKind::LeftBracket => self.parse_index(left),
            _ => Ok(left),
        }
    }

    fn current_precedence(&self) -> Precedence {
        match self.peek().kind {
            TokenKind::PipePipe => Precedence::Or,
            TokenKind::AmpAmp => Precedence::And,
            TokenKind::EqualEqual | TokenKind::BangEqual => Precedence::Equality,
            TokenKind::Less | TokenKind::LessEqual |
            TokenKind::Greater | TokenKind::GreaterEqual => Precedence::Comparison,
            TokenKind::Plus | TokenKind::Minus => Precedence::Term,
            TokenKind::Star | TokenKind::Slash | TokenKind::Percent => Precedence::Factor,
            TokenKind::LeftParen | TokenKind::LeftBracket => Precedence::Call,
            _ => Precedence::Lowest,
        }
    }

    fn parse_binary(&mut self, left: Expr) -> Result<Expr, ()> {
        let op_token = self.advance().clone();
        let op = match op_token.kind {
            TokenKind::Plus => BinaryOp::Add,
            TokenKind::Minus => BinaryOp::Sub,
            TokenKind::Star => BinaryOp::Mul,
            TokenKind::Slash => BinaryOp::Div,
            TokenKind::Percent => BinaryOp::Mod,
            TokenKind::EqualEqual => BinaryOp::Eq,
            TokenKind::BangEqual => BinaryOp::Ne,
            TokenKind::Less => BinaryOp::Lt,
            TokenKind::LessEqual => BinaryOp::Le,
            TokenKind::Greater => BinaryOp::Gt,
            TokenKind::GreaterEqual => BinaryOp::Ge,
            TokenKind::AmpAmp => BinaryOp::And,
            TokenKind::PipePipe => BinaryOp::Or,
            _ => unreachable!(),
        };

        let precedence = self.token_precedence(&op_token);
        let right = self.parse_precedence(precedence)?;
        let span = Span::combine(self.expr_span(&left), self.expr_span(&right));

        Ok(Expr::Binary(BinaryExpr {
            op,
            left: Box::new(left),
            right: Box::new(right),
            span,
        }))
    }
}
```

## Recursive Descent for Statements

```rust
impl Parser {
    fn parse_statement(&mut self) -> Result<Stmt, ()> {
        match self.peek().kind {
            TokenKind::Let | TokenKind::Var => self.parse_var_decl(),
            TokenKind::If => self.parse_if_stmt(),
            TokenKind::While => self.parse_while_stmt(),
            TokenKind::For => self.parse_for_stmt(),
            TokenKind::Return => self.parse_return_stmt(),
            TokenKind::Break => self.parse_break_stmt(),
            TokenKind::Continue => self.parse_continue_stmt(),
            _ => {
                // Parse expression then check for assignment operators
                self.parse_assign_or_expr_stmt()
            }
        }
    }

    fn parse_assign_or_expr_stmt(&mut self) -> Result<Stmt, ()> {
        let expr = self.parse_expression()?;

        match self.peek().kind {
            TokenKind::Equal => {
                self.advance(); // consume =
                let value = self.parse_expression()?;
                let target = self.expr_to_assign_target(expr)?;
                self.consume(TokenKind::Semicolon, "Expected ';'")?;
                Ok(Stmt::Assign(Assign { target, value, span }))
            }
            TokenKind::PlusEqual | TokenKind::MinusEqual |
            TokenKind::StarEqual | TokenKind::SlashEqual |
            TokenKind::PercentEqual => {
                let op_token = self.advance();
                let op = match op_token.kind {
                    TokenKind::PlusEqual => CompoundOp::AddAssign,
                    TokenKind::MinusEqual => CompoundOp::SubAssign,
                    TokenKind::StarEqual => CompoundOp::MulAssign,
                    TokenKind::SlashEqual => CompoundOp::DivAssign,
                    TokenKind::PercentEqual => CompoundOp::ModAssign,
                    _ => unreachable!(),
                };
                let value = self.parse_expression()?;
                let target = self.expr_to_assign_target(expr)?;
                self.consume(TokenKind::Semicolon, "Expected ';'")?;
                Ok(Stmt::CompoundAssign(CompoundAssign { target, op, value, span }))
            }
            TokenKind::PlusPlus => {
                self.advance(); // consume ++
                let target = self.expr_to_assign_target(expr)?;
                self.consume(TokenKind::Semicolon, "Expected ';'")?;
                Ok(Stmt::Increment(IncrementStmt { target, span }))
            }
            TokenKind::MinusMinus => {
                self.advance(); // consume --
                let target = self.expr_to_assign_target(expr)?;
                self.consume(TokenKind::Semicolon, "Expected ';'")?;
                Ok(Stmt::Decrement(DecrementStmt { target, span }))
            }
            _ => {
                // It's an expression statement
                self.consume(TokenKind::Semicolon, "Expected ';'")?;
                Ok(Stmt::Expr(ExprStmt { expr, span }))
            }
        }
    }

    fn parse_var_decl(&mut self) -> Result<Stmt, ()> {
        let keyword = self.advance();
        let mutable = keyword.kind == TokenKind::Var;

        let name_token = self.consume(TokenKind::Identifier, "Expected variable name")?;
        let name = Identifier {
            name: name_token.lexeme.clone(),
            span: name_token.span,
        };

        let type_ref = if self.match_token(TokenKind::Colon) {
            Some(self.parse_type_ref()?)
        } else {
            None
        };

        self.consume(TokenKind::Equal, "Expected '=' after variable name")?;
        let init = self.parse_expression()?;
        self.consume(TokenKind::Semicolon, "Expected ';' after variable declaration")?;

        Ok(Stmt::VarDecl(VarDecl {
            mutable,
            name,
            type_ref,
            init,
            span: keyword.span,
        }))
    }
}
```

## Helper Methods

```rust
impl Parser {
    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn check(&self, kind: TokenKind) -> bool {
        !self.is_at_end() && self.peek().kind == kind
    }

    fn match_token(&mut self, kind: TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn consume(&mut self, kind: TokenKind, message: &str) -> Result<&Token, ()> {
        if self.check(kind) {
            Ok(self.advance())
        } else {
            self.error(message);
            Err(())
        }
    }

    fn is_at_end(&self) -> bool {
        self.peek().kind == TokenKind::Eof
    }

    fn error(&mut self, message: &str) {
        let span = self.peek().span;
        self.diagnostics.push(Diagnostic::error("AT1000", message, span));
    }
}
```

## Key Design Decisions

- **Pratt parsing:** Handles operator precedence elegantly
- **Left-to-right associativity:** All binary operators
- **Error recovery:** Continue parsing after errors via synchronization
- **Span tracking:** Every AST node gets accurate source location
