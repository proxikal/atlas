//! Expression parsing (Pratt parsing)

use crate::ast::*;
use crate::parser::{Parser, Precedence};
use crate::span::Span;
use crate::token::{Token, TokenKind};

impl Parser {
    /// Parse an expression
    pub(super) fn parse_expression(&mut self) -> Result<Expr, ()> {
        self.parse_precedence(Precedence::Lowest)
    }

    /// Parse expression with given precedence
    pub(super) fn parse_precedence(&mut self, precedence: Precedence) -> Result<Expr, ()> {
        let mut left = self.parse_prefix()?;

        while precedence < self.current_precedence() {
            left = self.parse_infix(left)?;
        }

        Ok(left)
    }

    /// Parse prefix expression
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

    /// Parse infix expression
    fn parse_infix(&mut self, left: Expr) -> Result<Expr, ()> {
        match self.peek().kind {
            TokenKind::Plus
            | TokenKind::Minus
            | TokenKind::Star
            | TokenKind::Slash
            | TokenKind::Percent
            | TokenKind::EqualEqual
            | TokenKind::BangEqual
            | TokenKind::Less
            | TokenKind::LessEqual
            | TokenKind::Greater
            | TokenKind::GreaterEqual
            | TokenKind::AmpAmp
            | TokenKind::PipePipe => self.parse_binary(left),
            TokenKind::LeftParen => self.parse_call(left),
            TokenKind::LeftBracket => self.parse_index(left),
            _ => Ok(left),
        }
    }

    /// Get current token precedence
    pub(super) fn current_precedence(&self) -> Precedence {
        self.token_precedence(self.peek())
    }

    /// Get precedence for a token
    pub(super) fn token_precedence(&self, token: &Token) -> Precedence {
        match token.kind {
            TokenKind::PipePipe => Precedence::Or,
            TokenKind::AmpAmp => Precedence::And,
            TokenKind::EqualEqual | TokenKind::BangEqual => Precedence::Equality,
            TokenKind::Less
            | TokenKind::LessEqual
            | TokenKind::Greater
            | TokenKind::GreaterEqual => Precedence::Comparison,
            TokenKind::Plus | TokenKind::Minus => Precedence::Term,
            TokenKind::Star | TokenKind::Slash | TokenKind::Percent => Precedence::Factor,
            TokenKind::LeftParen | TokenKind::LeftBracket => Precedence::Call,
            _ => Precedence::Lowest,
        }
    }

    /// Parse number literal
    fn parse_number(&mut self) -> Result<Expr, ()> {
        let token = self.advance();
        let span = token.span;
        let value: f64 = token.lexeme.parse().unwrap_or(0.0);
        Ok(Expr::Literal(Literal::Number(value), span))
    }

    /// Parse string literal
    fn parse_string(&mut self) -> Result<Expr, ()> {
        let token = self.advance();
        let span = token.span;
        Ok(Expr::Literal(Literal::String(token.lexeme.clone()), span))
    }

    /// Parse boolean literal
    fn parse_bool(&mut self) -> Result<Expr, ()> {
        let token = self.advance();
        let span = token.span;
        let value = token.kind == TokenKind::True;
        Ok(Expr::Literal(Literal::Bool(value), span))
    }

    /// Parse null literal
    fn parse_null(&mut self) -> Result<Expr, ()> {
        let token = self.advance();
        let span = token.span;
        Ok(Expr::Literal(Literal::Null, span))
    }

    /// Parse identifier
    fn parse_identifier(&mut self) -> Result<Expr, ()> {
        let token = self.advance();
        Ok(Expr::Identifier(Identifier {
            name: token.lexeme.clone(),
            span: token.span,
        }))
    }

    /// Parse grouped expression
    fn parse_group(&mut self) -> Result<Expr, ()> {
        let start_span = self.consume(TokenKind::LeftParen, "Expected '('")?. span;
        let expr = self.parse_expression()?;
        let end_span = self.consume(TokenKind::RightParen, "Expected ')'")?. span;

        Ok(Expr::Group(GroupExpr {
            expr: Box::new(expr),
            span: start_span.merge(end_span),
        }))
    }

    /// Parse array literal
    fn parse_array_literal(&mut self) -> Result<Expr, ()> {
        let start_span = self.consume(TokenKind::LeftBracket, "Expected '['")?. span;
        let mut elements = Vec::new();

        if !self.check(TokenKind::RightBracket) {
            loop {
                elements.push(self.parse_expression()?);
                if !self.match_token(TokenKind::Comma) {
                    break;
                }
            }
        }

        let end_span = self.consume(TokenKind::RightBracket, "Expected ']'")?. span;

        Ok(Expr::ArrayLiteral(ArrayLiteral {
            elements,
            span: start_span.merge(end_span),
        }))
    }

    /// Parse unary expression
    fn parse_unary(&mut self) -> Result<Expr, ()> {
        let op_token = self.advance();
        let op_span = op_token.span;
        let op = match op_token.kind {
            TokenKind::Minus => UnaryOp::Negate,
            TokenKind::Bang => UnaryOp::Not,
            _ => unreachable!(),
        };

        let operand = self.parse_precedence(Precedence::Unary)?;
        let operand_span = operand.span();

        Ok(Expr::Unary(UnaryExpr {
            op,
            expr: Box::new(operand),
            span: op_span.merge(operand_span),
        }))
    }

    /// Parse binary expression
    fn parse_binary(&mut self, left: Expr) -> Result<Expr, ()> {
        let left_span = left.span();
        let op_token = self.advance();
        let op_kind = op_token.kind;

        let op = match op_kind {
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

        // Get precedence from the operator kind
        let precedence = match op_kind {
            TokenKind::PipePipe => Precedence::Or,
            TokenKind::AmpAmp => Precedence::And,
            TokenKind::EqualEqual | TokenKind::BangEqual => Precedence::Equality,
            TokenKind::Less | TokenKind::LessEqual | TokenKind::Greater | TokenKind::GreaterEqual => Precedence::Comparison,
            TokenKind::Plus | TokenKind::Minus => Precedence::Term,
            TokenKind::Star | TokenKind::Slash | TokenKind::Percent => Precedence::Factor,
            _ => Precedence::Lowest,
        };

        let right = self.parse_precedence(precedence)?;
        let right_span = right.span();

        Ok(Expr::Binary(BinaryExpr {
            op,
            left: Box::new(left),
            right: Box::new(right),
            span: left_span.merge(right_span),
        }))
    }

    /// Parse call expression
    fn parse_call(&mut self, callee: Expr) -> Result<Expr, ()> {
        let callee_span = callee.span();
        self.consume(TokenKind::LeftParen, "Expected '('")?;
        let mut args = Vec::new();

        if !self.check(TokenKind::RightParen) {
            loop {
                args.push(self.parse_expression()?);
                if !self.match_token(TokenKind::Comma) {
                    break;
                }
            }
        }

        let end_span = self.consume(TokenKind::RightParen, "Expected ')'")?. span;

        Ok(Expr::Call(CallExpr {
            callee: Box::new(callee),
            args,
            span: callee_span.merge(end_span),
        }))
    }

    /// Parse index expression
    fn parse_index(&mut self, target: Expr) -> Result<Expr, ()> {
        let target_span = target.span();
        self.consume(TokenKind::LeftBracket, "Expected '['")?;
        let index = self.parse_expression()?;
        let end_span = self.consume(TokenKind::RightBracket, "Expected ']'")?. span;

        Ok(Expr::Index(IndexExpr {
            target: Box::new(target),
            index: Box::new(index),
            span: target_span.merge(end_span),
        }))
    }

    /// Parse type reference
    pub(super) fn parse_type_ref(&mut self) -> Result<TypeRef, ()> {
        let token = self.consume_identifier("a type name")?;
        let span = token.span;

        let name = match token.lexeme.as_str() {
            "number" | "string" | "bool" | "null" | "void" => token.lexeme.clone(),
            _ => {
                self.error("Unknown type");
                return Err(());
            }
        };

        let mut type_ref = TypeRef::Named(name, span);

        // Handle array type syntax: type[]
        while self.match_token(TokenKind::LeftBracket) {
            let rbracket_token = self.consume(TokenKind::RightBracket, "Expected ']' after '[' in array type")?;
            let end_span = rbracket_token.span;
            let full_span = Span::new(span.start, end_span.end);
            type_ref = TypeRef::Array(Box::new(type_ref), full_span);
        }

        Ok(type_ref)
    }
}
