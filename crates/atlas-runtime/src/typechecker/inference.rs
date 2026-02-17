//! Enhanced type inference
//!
//! Provides:
//! - Return type inference from function body analysis
//! - Bidirectional type checking (expected type guides inference)
//! - Variable type inference from initializers
//! - Least upper bound computation for multiple types

use crate::ast::*;
use crate::types::Type;

/// Infer the return type of a function from its body.
///
/// Collects all `return` statement expression types and computes their
/// least upper bound. Returns `None` if no return statements found
/// (implies void), or `Some(type)` with the inferred return type.
pub fn infer_return_type(body: &Block) -> InferredReturn {
    let mut return_types = Vec::new();
    let mut has_implicit_void = false;

    collect_return_types(&body.statements, &mut return_types, &mut has_implicit_void);

    if return_types.is_empty() {
        return InferredReturn::Void;
    }

    // Check if all return types are the same
    let first = &return_types[0];
    let first_norm = first.normalized();
    let all_same = return_types.iter().all(|t| t.normalized() == first_norm);

    if all_same {
        if has_implicit_void && first_norm != Type::Void {
            // Some paths return a value, some fall through (void)
            InferredReturn::Inconsistent {
                types: return_types,
                has_void_path: true,
            }
        } else {
            InferredReturn::Uniform(first.clone())
        }
    } else {
        InferredReturn::Inconsistent {
            types: return_types,
            has_void_path: has_implicit_void,
        }
    }
}

/// Result of return type inference
#[derive(Debug, Clone, PartialEq)]
pub enum InferredReturn {
    /// All paths return void (no explicit returns, or only `return;`)
    Void,
    /// All explicit returns have the same type
    Uniform(Type),
    /// Returns have different types (error)
    Inconsistent {
        types: Vec<Type>,
        has_void_path: bool,
    },
}

/// Collect return types from statements recursively.
fn collect_return_types(
    stmts: &[Stmt],
    return_types: &mut Vec<Type>,
    has_implicit_void: &mut bool,
) {
    for stmt in stmts {
        match stmt {
            Stmt::Return(ret) => {
                let ty = if let Some(value) = &ret.value {
                    infer_expr_type(value)
                } else {
                    Type::Void
                };
                return_types.push(ty);
            }
            Stmt::If(if_stmt) => {
                collect_return_types(
                    &if_stmt.then_block.statements,
                    return_types,
                    has_implicit_void,
                );
                if let Some(else_block) = &if_stmt.else_block {
                    collect_return_types(&else_block.statements, return_types, has_implicit_void);
                } else {
                    // If without else: the "no else" path might fall through
                    *has_implicit_void = true;
                }
            }
            Stmt::While(while_stmt) => {
                collect_return_types(&while_stmt.body.statements, return_types, has_implicit_void);
            }
            Stmt::For(for_stmt) => {
                collect_return_types(&for_stmt.body.statements, return_types, has_implicit_void);
            }
            Stmt::ForIn(for_in_stmt) => {
                collect_return_types(
                    &for_in_stmt.body.statements,
                    return_types,
                    has_implicit_void,
                );
            }
            _ => {}
        }
    }
}

/// Quick type inference for an expression (without full type checking).
///
/// This is a lightweight version used during return type inference.
/// It handles the common cases; the full type checker handles the rest.
pub fn infer_expr_type(expr: &Expr) -> Type {
    match expr {
        Expr::Literal(lit, _) => match lit {
            Literal::Number(_) => Type::Number,
            Literal::String(_) => Type::String,
            Literal::Bool(_) => Type::Bool,
            Literal::Null => Type::Null,
        },
        Expr::Binary(binary) => infer_binary_type(&binary.op),
        Expr::Unary(unary) => match unary.op {
            UnaryOp::Negate => Type::Number,
            UnaryOp::Not => Type::Bool,
        },
        Expr::ArrayLiteral(_) => Type::Array(Box::new(Type::Unknown)),
        Expr::Group(group) => infer_expr_type(&group.expr),
        _ => Type::Unknown,
    }
}

/// Infer the result type of a binary operation.
fn infer_binary_type(op: &BinaryOp) -> Type {
    match op {
        BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
            Type::Number // Could be string for Add, but default to number
        }
        BinaryOp::Eq
        | BinaryOp::Ne
        | BinaryOp::Lt
        | BinaryOp::Le
        | BinaryOp::Gt
        | BinaryOp::Ge
        | BinaryOp::And
        | BinaryOp::Or => Type::Bool,
    }
}

/// Check if an expected type is compatible with an inferred type,
/// allowing the expected type to guide inference.
///
/// In bidirectional checking, when we know the expected type from context
/// (e.g., variable annotation, return type), we can use it to validate
/// and refine inference results.
pub fn check_bidirectional(expected: &Type, inferred: &Type) -> BidirectionalResult {
    // Unknown can flow to any expected type
    if inferred.normalized() == Type::Unknown {
        return BidirectionalResult::Compatible;
    }
    if expected.normalized() == Type::Unknown {
        return BidirectionalResult::Compatible;
    }

    if inferred.is_assignable_to(expected) {
        BidirectionalResult::Compatible
    } else {
        BidirectionalResult::Mismatch {
            expected: expected.clone(),
            found: inferred.clone(),
        }
    }
}

/// Result of bidirectional type checking
#[derive(Debug, Clone, PartialEq)]
pub enum BidirectionalResult {
    /// Types are compatible
    Compatible,
    /// Types don't match
    Mismatch { expected: Type, found: Type },
}

/// Compute the least upper bound (common type) of two types.
///
/// Returns `None` if the types are incompatible.
pub fn least_upper_bound(a: &Type, b: &Type) -> Option<Type> {
    let a_norm = a.normalized();
    let b_norm = b.normalized();

    if a_norm == b_norm {
        return Some(a_norm);
    }

    // Unknown is subsumed by any concrete type
    if a_norm == Type::Unknown {
        return Some(b_norm);
    }
    if b_norm == Type::Unknown {
        return Some(a_norm);
    }

    // Arrays: LUB of element types
    if let (Type::Array(ea), Type::Array(eb)) = (a_norm, b_norm) {
        return least_upper_bound(&ea, &eb).map(|lub| Type::Array(Box::new(lub)));
    }

    // No common type
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_literal_types() {
        assert_eq!(
            infer_expr_type(&Expr::Literal(
                Literal::Number(42.0),
                crate::span::Span::dummy()
            )),
            Type::Number
        );
        assert_eq!(
            infer_expr_type(&Expr::Literal(
                Literal::String("hi".into()),
                crate::span::Span::dummy()
            )),
            Type::String
        );
        assert_eq!(
            infer_expr_type(&Expr::Literal(
                Literal::Bool(true),
                crate::span::Span::dummy()
            )),
            Type::Bool
        );
    }

    #[test]
    fn test_bidirectional_compatible() {
        assert_eq!(
            check_bidirectional(&Type::Number, &Type::Number),
            BidirectionalResult::Compatible
        );
    }

    #[test]
    fn test_bidirectional_mismatch() {
        let result = check_bidirectional(&Type::Number, &Type::String);
        assert!(matches!(result, BidirectionalResult::Mismatch { .. }));
    }

    #[test]
    fn test_bidirectional_unknown_compatible() {
        assert_eq!(
            check_bidirectional(&Type::Number, &Type::Unknown),
            BidirectionalResult::Compatible
        );
    }

    #[test]
    fn test_least_upper_bound_same() {
        assert_eq!(
            least_upper_bound(&Type::Number, &Type::Number),
            Some(Type::Number)
        );
    }

    #[test]
    fn test_least_upper_bound_incompatible() {
        assert_eq!(least_upper_bound(&Type::Number, &Type::String), None);
    }

    #[test]
    fn test_least_upper_bound_unknown() {
        assert_eq!(
            least_upper_bound(&Type::Unknown, &Type::Number),
            Some(Type::Number)
        );
    }

    #[test]
    fn test_least_upper_bound_arrays() {
        assert_eq!(
            least_upper_bound(
                &Type::Array(Box::new(Type::Number)),
                &Type::Array(Box::new(Type::Number))
            ),
            Some(Type::Array(Box::new(Type::Number)))
        );
    }

    #[test]
    fn test_infer_return_void() {
        let block = Block {
            statements: vec![],
            span: crate::span::Span::dummy(),
        };
        assert_eq!(infer_return_type(&block), InferredReturn::Void);
    }

    #[test]
    fn test_infer_return_uniform() {
        let block = Block {
            statements: vec![Stmt::Return(ReturnStmt {
                value: Some(Expr::Literal(
                    Literal::Number(42.0),
                    crate::span::Span::dummy(),
                )),
                span: crate::span::Span::dummy(),
            })],
            span: crate::span::Span::dummy(),
        };
        assert_eq!(
            infer_return_type(&block),
            InferredReturn::Uniform(Type::Number)
        );
    }
}
