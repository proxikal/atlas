//! Expression parsing (Pratt parsing)

use super::E_BAD_NUMBER;
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
            TokenKind::Match => self.parse_match_expr(),
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
            TokenKind::Dot => self.parse_member(left),
            TokenKind::Question => self.parse_try(left),
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
            TokenKind::LeftParen
            | TokenKind::LeftBracket
            | TokenKind::Dot
            | TokenKind::Question => Precedence::Call,
            _ => Precedence::Lowest,
        }
    }

    /// Parse number literal
    fn parse_number(&mut self) -> Result<Expr, ()> {
        let token = self.advance();
        let span = token.span;
        let lexeme = token.lexeme.clone();
        let value: f64 = match lexeme.parse::<f64>() {
            Ok(value) if value.is_finite() => value,
            _ => {
                self.error_at_with_code(
                    E_BAD_NUMBER,
                    &format!("Invalid number literal: '{}'", lexeme),
                    span,
                );
                0.0
            }
        };
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
        let start_span = self.consume(TokenKind::LeftParen, "Expected '('")?.span;
        let expr = self.parse_expression()?;
        let end_span = self.consume(TokenKind::RightParen, "Expected ')'")?.span;

        Ok(Expr::Group(GroupExpr {
            expr: Box::new(expr),
            span: start_span.merge(end_span),
        }))
    }

    /// Parse array literal
    fn parse_array_literal(&mut self) -> Result<Expr, ()> {
        let start_span = self.consume(TokenKind::LeftBracket, "Expected '['")?.span;
        let mut elements = Vec::new();

        if !self.check(TokenKind::RightBracket) {
            loop {
                elements.push(self.parse_expression()?);
                if !self.match_token(TokenKind::Comma) {
                    break;
                }
            }
        }

        let end_span = self.consume(TokenKind::RightBracket, "Expected ']'")?.span;

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
            TokenKind::Less
            | TokenKind::LessEqual
            | TokenKind::Greater
            | TokenKind::GreaterEqual => Precedence::Comparison,
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

        let end_span = self.consume(TokenKind::RightParen, "Expected ')'")?.span;

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
        let end_span = self.consume(TokenKind::RightBracket, "Expected ']'")?.span;

        Ok(Expr::Index(IndexExpr {
            target: Box::new(target),
            index: Box::new(index),
            span: target_span.merge(end_span),
        }))
    }

    /// Parse member expression (method call or property access)
    fn parse_member(&mut self, target: Expr) -> Result<Expr, ()> {
        let target_span = target.span();
        self.consume(TokenKind::Dot, "Expected '.'")?;

        // Member name must be an identifier
        let member_token = self.consume_identifier("a method or property name")?;
        let member = Identifier {
            name: member_token.lexeme.clone(),
            span: member_token.span,
        };

        // Check for method call (with parentheses)
        let (args, end_span) = if self.check(TokenKind::LeftParen) {
            self.consume(TokenKind::LeftParen, "Expected '('")?;
            let mut args_vec = Vec::new();

            if !self.check(TokenKind::RightParen) {
                loop {
                    args_vec.push(self.parse_expression()?);
                    if !self.match_token(TokenKind::Comma) {
                        break;
                    }
                }
            }

            let end = self.consume(TokenKind::RightParen, "Expected ')'")?.span;
            (Some(args_vec), end)
        } else {
            // No parentheses - property access (for now, treat as method call with no args)
            // Phase 16 only supports method calls, but parser can handle both
            (None, member.span)
        };

        Ok(Expr::Member(MemberExpr {
            target: Box::new(target),
            member,
            args,
            type_tag: std::cell::Cell::new(None),
            trait_dispatch: std::cell::RefCell::new(None),
            span: target_span.merge(end_span),
        }))
    }

    /// Parse try expression (error propagation operator ?)
    fn parse_try(&mut self, expr: Expr) -> Result<Expr, ()> {
        let expr_span = expr.span();
        let question_span = self.consume(TokenKind::Question, "Expected '?'")?.span;

        Ok(Expr::Try(TryExpr {
            expr: Box::new(expr),
            span: expr_span.merge(question_span),
        }))
    }

    /// Parse type reference
    pub(super) fn parse_type_ref(&mut self) -> Result<TypeRef, ()> {
        self.parse_union_type()
    }

    /// Parse union type: A | B
    fn parse_union_type(&mut self) -> Result<TypeRef, ()> {
        let mut members = vec![self.parse_intersection_type()?];

        while self.match_token(TokenKind::Pipe) {
            members.push(self.parse_intersection_type()?);
        }

        if members.len() == 1 {
            Ok(members.remove(0))
        } else {
            let start = members.first().unwrap().span();
            let end = members.last().unwrap().span();
            Ok(TypeRef::Union {
                members,
                span: start.merge(end),
            })
        }
    }

    /// Parse intersection type: A & B
    fn parse_intersection_type(&mut self) -> Result<TypeRef, ()> {
        let mut members = vec![self.parse_type_primary()?];

        while self.match_token(TokenKind::Ampersand) {
            members.push(self.parse_type_primary()?);
        }

        if members.len() == 1 {
            Ok(members.remove(0))
        } else {
            let start = members.first().unwrap().span();
            let end = members.last().unwrap().span();
            Ok(TypeRef::Intersection {
                members,
                span: start.merge(end),
            })
        }
    }

    /// Parse primary type (named, generic, grouped, or function), plus array suffixes.
    fn parse_type_primary(&mut self) -> Result<TypeRef, ()> {
        let mut type_ref = if self.check(TokenKind::LeftParen) {
            self.parse_paren_type()?
        } else if self.check(TokenKind::LeftBrace) {
            self.parse_structural_type()?
        } else {
            let token = if self.check(TokenKind::Null) {
                self.advance()
            } else {
                self.consume_identifier("a type name")?
            };
            let span = token.span;
            let name = token.lexeme.clone();

            // Check for generic type: Type<T1, T2>
            if self.check(TokenKind::Less) {
                self.parse_generic_type(name, span)?
            } else {
                TypeRef::Named(name, span)
            }
        };

        // Handle array type syntax: type[]
        loop {
            if !self.match_token(TokenKind::LeftBracket) {
                break;
            }
            let rbracket_token = self.consume(
                TokenKind::RightBracket,
                "Expected ']' after '[' in array type",
            )?;
            let end_span = rbracket_token.span;
            let full_span = type_ref.span().merge(end_span);
            type_ref = TypeRef::Array(Box::new(type_ref), full_span);
        }

        Ok(type_ref)
    }

    /// Parse structural type: { field: type, method: (params) -> return }
    fn parse_structural_type(&mut self) -> Result<TypeRef, ()> {
        use crate::ast::StructuralMember;

        let start_span = self
            .consume(
                TokenKind::LeftBrace,
                "Expected '{' at start of structural type",
            )?
            .span;

        let mut members = Vec::new();
        if !self.check(TokenKind::RightBrace) {
            loop {
                let member_start = self.peek().span;
                let name_tok = self.consume_identifier("a structural member name")?;
                let member_name = name_tok.lexeme.clone();
                let member_name_span = name_tok.span;
                self.consume(TokenKind::Colon, "Expected ':' after member name")?;
                let type_ref = self.parse_type_ref()?;
                let member_span = member_start.merge(type_ref.span());
                members.push(StructuralMember {
                    name: member_name,
                    type_ref,
                    span: member_span.merge(member_name_span),
                });

                if !self.match_token(TokenKind::Comma) {
                    break;
                }
                if self.check(TokenKind::RightBrace) {
                    break;
                }
            }
        }

        let end_span = self
            .consume(TokenKind::RightBrace, "Expected '}' after structural type")?
            .span;

        if members.is_empty() {
            self.error("Structural type must include at least one member");
            return Err(());
        }

        Ok(TypeRef::Structural {
            members,
            span: start_span.merge(end_span),
        })
    }

    /// Parse parenthesized type or function type.
    fn parse_paren_type(&mut self) -> Result<TypeRef, ()> {
        let start_token = self.consume(TokenKind::LeftParen, "Expected '(' at start of type")?;
        let start_span = start_token.span;

        if self.check(TokenKind::RightParen) {
            self.consume(TokenKind::RightParen, "Expected ')'")?;
            if self.match_token(TokenKind::Arrow) {
                let return_type = self.parse_type_ref()?;
                let full_span = Span::new(start_span.start, return_type.span().end);
                return Ok(TypeRef::Function {
                    params: Vec::new(),
                    return_type: Box::new(return_type),
                    span: full_span,
                });
            }
            self.error("Unexpected empty type group");
            return Err(());
        }

        let mut params = Vec::new();
        params.push(self.parse_type_ref()?);

        while self.match_token(TokenKind::Comma) {
            params.push(self.parse_type_ref()?);
        }

        self.consume(TokenKind::RightParen, "Expected ')' after type list")?;

        if self.match_token(TokenKind::Arrow) {
            let return_type = self.parse_type_ref()?;
            let full_span = Span::new(start_span.start, return_type.span().end);
            return Ok(TypeRef::Function {
                params,
                return_type: Box::new(return_type),
                span: full_span,
            });
        }

        if params.len() == 1 {
            return Ok(params.remove(0));
        }

        self.error("Expected '->' after function type parameters");
        Err(())
    }

    /// Parse generic type: Type<T1, T2, ...>
    fn parse_generic_type(&mut self, name: String, start: Span) -> Result<TypeRef, ()> {
        // Consume '<'
        self.consume(TokenKind::Less, "Expected '<'")?;

        // Parse type arguments
        let mut type_args = vec![];
        loop {
            type_args.push(self.parse_type_ref()?);

            if !self.match_token(TokenKind::Comma) {
                break;
            }
        }

        // Ensure at least one type argument
        if type_args.is_empty() {
            self.error("Generic type requires at least one type argument");
            return Err(());
        }

        // Consume '>'
        let end_token = self.consume(TokenKind::Greater, "Expected '>' after type arguments")?;
        let span = Span::new(start.start, end_token.span.end);

        Ok(TypeRef::Generic {
            name,
            type_args,
            span,
        })
    }

    // parse_function_type removed in favor of parse_paren_type

    /// Parse match expression
    fn parse_match_expr(&mut self) -> Result<Expr, ()> {
        use crate::ast::MatchExpr;

        let start_span = self.consume(TokenKind::Match, "Expected 'match'")?.span;

        // Parse scrutinee (the expression being matched)
        let scrutinee = self.parse_expression()?;

        // Parse match block
        self.consume(TokenKind::LeftBrace, "Expected '{' after match expression")?;

        // Parse match arms
        let mut arms = Vec::new();
        while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
            arms.push(self.parse_match_arm()?);

            // Arms are separated by commas, trailing comma is optional
            if !self.check(TokenKind::RightBrace) {
                self.consume(TokenKind::Comma, "Expected ',' after match arm")?;
                // Allow trailing comma
                if self.check(TokenKind::RightBrace) {
                    break;
                }
            }
        }

        let end_span = self
            .consume(TokenKind::RightBrace, "Expected '}' after match arms")?
            .span;

        if arms.is_empty() {
            self.error("Match expression must have at least one arm");
            return Err(());
        }

        Ok(Expr::Match(MatchExpr {
            scrutinee: Box::new(scrutinee),
            arms,
            span: start_span.merge(end_span),
        }))
    }

    /// Parse match arm (pattern => expression)
    fn parse_match_arm(&mut self) -> Result<MatchArm, ()> {
        use crate::ast::MatchArm;

        let pattern = self.parse_or_pattern()?;
        let pattern_span = pattern.span();

        // Parse optional guard clause: `pattern if <expr> => body`
        let guard = if self.match_token(TokenKind::If) {
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };

        self.consume(TokenKind::FatArrow, "Expected '=>' after pattern")?;

        let body = self.parse_expression()?;
        let body_span = body.span();

        Ok(MatchArm {
            pattern,
            guard,
            body,
            span: pattern_span.merge(body_span),
        })
    }

    /// Parse OR pattern: primary | primary | ...
    fn parse_or_pattern(&mut self) -> Result<crate::ast::Pattern, ()> {
        use crate::ast::Pattern;

        let first = self.parse_pattern()?;
        let start_span = first.span();

        // If no `|` follows, return single pattern (no wrapping)
        if !self.check(TokenKind::Pipe) {
            return Ok(first);
        }

        let mut alternatives = vec![first];
        while self.match_token(TokenKind::Pipe) {
            alternatives.push(self.parse_pattern()?);
        }

        let end_span = alternatives.last().unwrap().span();
        Ok(Pattern::Or(alternatives, start_span.merge(end_span)))
    }

    /// Parse pattern
    fn parse_pattern(&mut self) -> Result<crate::ast::Pattern, ()> {
        use crate::ast::Pattern;

        match self.peek().kind {
            // Literal patterns: numbers, strings, bools, null
            TokenKind::Number => {
                let token = self.advance();
                let span = token.span;
                let lexeme = token.lexeme.clone();
                let value: f64 = match lexeme.parse::<f64>() {
                    Ok(value) if value.is_finite() => value,
                    _ => {
                        self.error_at_with_code(
                            E_BAD_NUMBER,
                            &format!("Invalid number literal: '{}'", lexeme),
                            span,
                        );
                        0.0
                    }
                };
                Ok(Pattern::Literal(Literal::Number(value), span))
            }
            TokenKind::String => {
                let token = self.advance();
                Ok(Pattern::Literal(
                    Literal::String(token.lexeme.clone()),
                    token.span,
                ))
            }
            TokenKind::True => {
                let token = self.advance();
                Ok(Pattern::Literal(Literal::Bool(true), token.span))
            }
            TokenKind::False => {
                let token = self.advance();
                Ok(Pattern::Literal(Literal::Bool(false), token.span))
            }
            TokenKind::Null => {
                let token = self.advance();
                Ok(Pattern::Literal(Literal::Null, token.span))
            }

            // Wildcard pattern: _
            TokenKind::Underscore => {
                let token = self.advance();
                Ok(Pattern::Wildcard(token.span))
            }

            // Array pattern: [...]
            TokenKind::LeftBracket => self.parse_array_pattern(),

            // Constructor pattern or variable binding: Identifier or Identifier(...)
            TokenKind::Identifier => {
                let id_token = self.advance();
                let id = Identifier {
                    name: id_token.lexeme.clone(),
                    span: id_token.span,
                };

                // Check if this is a constructor pattern (has arguments)
                if self.check(TokenKind::LeftParen) {
                    self.parse_constructor_pattern(id)
                } else {
                    // Check if this is a zero-argument constructor (None, unit-like variants)
                    // For now, recognize built-in constructors: None
                    if id.name == "None" {
                        // Zero-argument constructor
                        Ok(Pattern::Constructor {
                            name: id.clone(),
                            args: Vec::new(),
                            span: id.span,
                        })
                    } else {
                        // Variable binding pattern
                        Ok(Pattern::Variable(id))
                    }
                }
            }

            _ => {
                self.error("Expected pattern");
                Err(())
            }
        }
    }

    /// Parse array pattern: [pattern, pattern, ...]
    fn parse_array_pattern(&mut self) -> Result<crate::ast::Pattern, ()> {
        use crate::ast::Pattern;

        let start_span = self.consume(TokenKind::LeftBracket, "Expected '['")?.span;
        let mut elements = Vec::new();

        if !self.check(TokenKind::RightBracket) {
            loop {
                elements.push(self.parse_pattern()?);
                if !self.match_token(TokenKind::Comma) {
                    break;
                }
            }
        }

        let end_span = self.consume(TokenKind::RightBracket, "Expected ']'")?.span;

        Ok(Pattern::Array {
            elements,
            span: start_span.merge(end_span),
        })
    }

    /// Parse constructor pattern: Name(pattern, pattern, ...)
    fn parse_constructor_pattern(&mut self, name: Identifier) -> Result<crate::ast::Pattern, ()> {
        use crate::ast::Pattern;

        let name_span = name.span;
        self.consume(TokenKind::LeftParen, "Expected '('")?;

        let mut args = Vec::new();
        if !self.check(TokenKind::RightParen) {
            loop {
                args.push(self.parse_or_pattern()?);
                if !self.match_token(TokenKind::Comma) {
                    break;
                }
            }
        }

        let end_span = self.consume(TokenKind::RightParen, "Expected ')'")?.span;

        Ok(Pattern::Constructor {
            name,
            args,
            span: name_span.merge(end_span),
        })
    }
}
