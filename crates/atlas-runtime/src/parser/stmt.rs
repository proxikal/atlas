//! Statement parsing

use crate::ast::*;
use crate::parser::Parser;
use crate::span::Span;
use crate::token::TokenKind;

impl Parser {
    /// Parse a statement
    pub(super) fn parse_statement(&mut self) -> Result<Stmt, ()> {
        match self.peek().kind {
            TokenKind::Let | TokenKind::Var => self.parse_var_decl(),
            TokenKind::If => self.parse_if_stmt(),
            TokenKind::While => self.parse_while_stmt(),
            TokenKind::For => self.parse_for_stmt(),
            TokenKind::Return => self.parse_return_stmt(),
            TokenKind::Break => self.parse_break_stmt(),
            TokenKind::Continue => self.parse_continue_stmt(),
            TokenKind::LeftBrace => {
                // Standalone block statement
                let block = self.parse_block()?;
                Ok(Stmt::Expr(ExprStmt {
                    expr: Expr::Literal(Literal::Null, block.span),
                    span: block.span,
                }))
            }
            TokenKind::Fn => Ok(Stmt::FunctionDecl(self.parse_function()?)),
            TokenKind::Import => {
                self.error("Import statements are not supported in Atlas v0.1");
                Err(())
            }
            _ => self.parse_assign_or_expr_stmt(),
        }
    }

    /// Parse a variable declaration
    pub(super) fn parse_var_decl(&mut self) -> Result<Stmt, ()> {
        let keyword_span = self.peek().span;
        let keyword = self.advance().kind;
        let mutable = keyword == TokenKind::Var;

        let name_token = self.consume_identifier("a variable name")?;
        let name = Identifier {
            name: name_token.lexeme.clone(),
            span: name_token.span,
        };

        let type_ref = if self.match_token(TokenKind::Colon) {
            Some(self.parse_type_ref()?)
        } else {
            None
        };

        self.consume(TokenKind::Equal, "Expected '=' in variable declaration")?;
        let init = self.parse_expression()?;
        let end_span = self
            .consume(
                TokenKind::Semicolon,
                "Expected ';' after variable declaration",
            )?
            .span;

        Ok(Stmt::VarDecl(VarDecl {
            mutable,
            name,
            type_ref,
            init,
            span: keyword_span.merge(end_span),
        }))
    }

    /// Parse assignment or expression statement
    pub(super) fn parse_assign_or_expr_stmt(&mut self) -> Result<Stmt, ()> {
        let expr = self.parse_expression()?;
        let expr_span = expr.span();

        // Check what follows the expression
        let next_kind = self.peek().kind;

        match next_kind {
            // Regular assignment: x = value
            TokenKind::Equal => {
                self.advance(); // consume =
                let target = self.expr_to_assign_target(expr)?;
                let value = self.parse_expression()?;
                let end_span = self
                    .consume(TokenKind::Semicolon, "Expected ';' after assignment")?
                    .span;

                Ok(Stmt::Assign(Assign {
                    target,
                    value,
                    span: expr_span.merge(end_span),
                }))
            }

            // Compound assignment: x += value, x -= value, etc.
            TokenKind::PlusEqual
            | TokenKind::MinusEqual
            | TokenKind::StarEqual
            | TokenKind::SlashEqual
            | TokenKind::PercentEqual => {
                let op_token = self.advance();
                let op = match op_token.kind {
                    TokenKind::PlusEqual => CompoundOp::AddAssign,
                    TokenKind::MinusEqual => CompoundOp::SubAssign,
                    TokenKind::StarEqual => CompoundOp::MulAssign,
                    TokenKind::SlashEqual => CompoundOp::DivAssign,
                    TokenKind::PercentEqual => CompoundOp::ModAssign,
                    _ => unreachable!(),
                };

                let target = self.expr_to_assign_target(expr)?;
                let value = self.parse_expression()?;
                let end_span = self
                    .consume(
                        TokenKind::Semicolon,
                        "Expected ';' after compound assignment",
                    )?
                    .span;

                Ok(Stmt::CompoundAssign(CompoundAssign {
                    target,
                    op,
                    value,
                    span: expr_span.merge(end_span),
                }))
            }

            // Increment: x++
            TokenKind::PlusPlus => {
                self.advance(); // consume ++
                let target = self.expr_to_assign_target(expr)?;
                let end_span = self
                    .consume(TokenKind::Semicolon, "Expected ';' after increment")?
                    .span;

                Ok(Stmt::Increment(IncrementStmt {
                    target,
                    span: expr_span.merge(end_span),
                }))
            }

            // Decrement: x--
            TokenKind::MinusMinus => {
                self.advance(); // consume --
                let target = self.expr_to_assign_target(expr)?;
                let end_span = self
                    .consume(TokenKind::Semicolon, "Expected ';' after decrement")?
                    .span;

                Ok(Stmt::Decrement(DecrementStmt {
                    target,
                    span: expr_span.merge(end_span),
                }))
            }

            // Expression statement
            _ => {
                let end_span = self
                    .consume(TokenKind::Semicolon, "Expected ';' after expression")?
                    .span;
                Ok(Stmt::Expr(ExprStmt {
                    expr,
                    span: expr_span.merge(end_span),
                }))
            }
        }
    }

    /// Convert an expression to an assignment target
    pub(super) fn expr_to_assign_target(&mut self, expr: Expr) -> Result<AssignTarget, ()> {
        match expr {
            Expr::Identifier(ident) => Ok(AssignTarget::Name(ident)),
            Expr::Index(idx) => Ok(AssignTarget::Index {
                target: idx.target,
                index: idx.index,
                span: idx.span,
            }),
            _ => {
                self.error("Invalid assignment target");
                Err(())
            }
        }
    }

    /// Parse if statement
    pub(super) fn parse_if_stmt(&mut self) -> Result<Stmt, ()> {
        let if_span = self.consume(TokenKind::If, "Expected 'if'")?.span;

        self.consume(TokenKind::LeftParen, "Expected '(' after 'if'")?;
        let cond = self.parse_expression()?;
        self.consume(TokenKind::RightParen, "Expected ')' after if condition")?;

        let then_block = self.parse_block()?;
        let then_span = then_block.span;

        let else_block = if self.match_token(TokenKind::Else) {
            Some(self.parse_block()?)
        } else {
            None
        };

        let end_span = else_block.as_ref().map_or(then_span, |b| b.span);

        Ok(Stmt::If(IfStmt {
            cond,
            then_block,
            else_block,
            span: if_span.merge(end_span),
        }))
    }

    /// Parse while statement
    pub(super) fn parse_while_stmt(&mut self) -> Result<Stmt, ()> {
        let while_span = self.consume(TokenKind::While, "Expected 'while'")?.span;

        self.consume(TokenKind::LeftParen, "Expected '(' after 'while'")?;
        let cond = self.parse_expression()?;
        self.consume(TokenKind::RightParen, "Expected ')' after while condition")?;

        let body = self.parse_block()?;
        let body_span = body.span;

        Ok(Stmt::While(WhileStmt {
            cond,
            body,
            span: while_span.merge(body_span),
        }))
    }

    /// Parse for statement
    pub(super) fn parse_for_stmt(&mut self) -> Result<Stmt, ()> {
        let for_span = self.consume(TokenKind::For, "Expected 'for'")?.span;

        self.consume(TokenKind::LeftParen, "Expected '(' after 'for'")?;

        // Parse initializer - create dummy statement if missing
        let init = if self.check(TokenKind::Let) || self.check(TokenKind::Var) {
            Box::new(self.parse_var_decl()?)
        } else if !self.check(TokenKind::Semicolon) {
            let expr = self.parse_expression()?;
            let expr_span = expr.span();
            self.consume(TokenKind::Semicolon, "Expected ';' after for initializer")?;
            Box::new(Stmt::Expr(ExprStmt {
                expr,
                span: expr_span,
            }))
        } else {
            self.advance(); // consume semicolon
                            // Create dummy expression statement
            Box::new(Stmt::Expr(ExprStmt {
                expr: Expr::Literal(Literal::Null, Span::dummy()),
                span: Span::dummy(),
            }))
        };

        // Parse condition - create dummy if missing
        let cond = if !self.check(TokenKind::Semicolon) {
            self.parse_expression()?
        } else {
            Expr::Literal(Literal::Bool(true), Span::dummy())
        };
        self.consume(TokenKind::Semicolon, "Expected ';' after for condition")?;

        // Parse step - create dummy if missing
        // Step can be assignment, compound assignment, increment, decrement, or expression
        let step = if !self.check(TokenKind::RightParen) {
            let expr = self.parse_expression()?;
            let start_span = expr.span();

            // Check what follows the expression
            match self.peek().kind {
                TokenKind::Equal => {
                    self.advance(); // consume =
                    let value = self.parse_expression()?;
                    let target = self.expr_to_assign_target(expr)?;
                    Box::new(Stmt::Assign(Assign {
                        target,
                        value,
                        span: start_span,
                    }))
                }
                TokenKind::PlusEqual
                | TokenKind::MinusEqual
                | TokenKind::StarEqual
                | TokenKind::SlashEqual
                | TokenKind::PercentEqual => {
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
                    Box::new(Stmt::CompoundAssign(CompoundAssign {
                        target,
                        op,
                        value,
                        span: start_span,
                    }))
                }
                TokenKind::PlusPlus => {
                    self.advance(); // consume ++
                    let target = self.expr_to_assign_target(expr)?;
                    Box::new(Stmt::Increment(IncrementStmt {
                        target,
                        span: start_span,
                    }))
                }
                TokenKind::MinusMinus => {
                    self.advance(); // consume --
                    let target = self.expr_to_assign_target(expr)?;
                    Box::new(Stmt::Decrement(DecrementStmt {
                        target,
                        span: start_span,
                    }))
                }
                _ => {
                    // Just an expression statement
                    Box::new(Stmt::Expr(ExprStmt {
                        expr,
                        span: start_span,
                    }))
                }
            }
        } else {
            Box::new(Stmt::Expr(ExprStmt {
                expr: Expr::Literal(Literal::Null, Span::dummy()),
                span: Span::dummy(),
            }))
        };
        self.consume(TokenKind::RightParen, "Expected ')' after for clauses")?;

        let body = self.parse_block()?;
        let body_span = body.span;

        Ok(Stmt::For(ForStmt {
            init,
            cond,
            step,
            body,
            span: for_span.merge(body_span),
        }))
    }

    /// Parse return statement
    pub(super) fn parse_return_stmt(&mut self) -> Result<Stmt, ()> {
        let return_span = self.consume(TokenKind::Return, "Expected 'return'")?.span;

        let value = if !self.check(TokenKind::Semicolon) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        let end_span = self
            .consume(TokenKind::Semicolon, "Expected ';' after return")?
            .span;

        Ok(Stmt::Return(ReturnStmt {
            value,
            span: return_span.merge(end_span),
        }))
    }

    /// Parse break statement
    pub(super) fn parse_break_stmt(&mut self) -> Result<Stmt, ()> {
        let break_span = self.consume(TokenKind::Break, "Expected 'break'")?.span;
        let end_span = self
            .consume(TokenKind::Semicolon, "Expected ';' after break")?
            .span;
        Ok(Stmt::Break(break_span.merge(end_span)))
    }

    /// Parse continue statement
    pub(super) fn parse_continue_stmt(&mut self) -> Result<Stmt, ()> {
        let continue_span = self
            .consume(TokenKind::Continue, "Expected 'continue'")?
            .span;
        let end_span = self
            .consume(TokenKind::Semicolon, "Expected ';' after continue")?
            .span;
        Ok(Stmt::Continue(continue_span.merge(end_span)))
    }

    /// Parse a block
    pub(super) fn parse_block(&mut self) -> Result<Block, ()> {
        let start_span = self.consume(TokenKind::LeftBrace, "Expected '{'")?.span;
        let mut statements = Vec::new();

        while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
            match self.parse_statement() {
                Ok(stmt) => statements.push(stmt),
                Err(_) => self.synchronize(),
            }
        }

        let end_span = self.consume(TokenKind::RightBrace, "Expected '}'")?.span;

        Ok(Block {
            statements,
            span: start_span.merge(end_span),
        })
    }
}
