//! AST Instantiation Tests
//!
//! These tests verify that all AST node types can be properly instantiated,
//! serialized, and used in practice.

use atlas_runtime::ast::*;
use atlas_runtime::span::Span;

#[test]
fn test_complete_program_construction() {
    // Build a complete program with various node types
    let program = Program {
        items: vec![
            // Function declaration
            Item::Function(FunctionDecl {
                name: Identifier {
                    name: "add".to_string(),
                    span: Span::new(5, 8),
                },
                type_params: vec![],
                params: vec![
                    Param {
                        name: Identifier {
                            name: "a".to_string(),
                            span: Span::new(9, 10),
                        },
                        type_ref: TypeRef::Named("number".to_string(), Span::new(12, 18)),
                        span: Span::new(9, 18),
                    },
                    Param {
                        name: Identifier {
                            name: "b".to_string(),
                            span: Span::new(20, 21),
                        },
                        type_ref: TypeRef::Named("number".to_string(), Span::new(23, 29)),
                        span: Span::new(20, 29),
                    },
                ],
                return_type: TypeRef::Named("number".to_string(), Span::new(34, 40)),
                body: Block {
                    statements: vec![Stmt::Return(ReturnStmt {
                        value: Some(Expr::Binary(BinaryExpr {
                            op: BinaryOp::Add,
                            left: Box::new(Expr::Identifier(Identifier {
                                name: "a".to_string(),
                                span: Span::new(50, 51),
                            })),
                            right: Box::new(Expr::Identifier(Identifier {
                                name: "b".to_string(),
                                span: Span::new(54, 55),
                            })),
                            span: Span::new(50, 55),
                        })),
                        span: Span::new(43, 56),
                    })],
                    span: Span::new(41, 58),
                },
                span: Span::new(0, 58),
            }),
            // Variable declaration statement
            Item::Statement(Stmt::VarDecl(VarDecl {
                mutable: false,
                name: Identifier {
                    name: "result".to_string(),
                    span: Span::new(64, 70),
                },
                type_ref: Some(TypeRef::Named("number".to_string(), Span::new(72, 78))),
                init: Expr::Call(CallExpr {
                    callee: Box::new(Expr::Identifier(Identifier {
                        name: "add".to_string(),
                        span: Span::new(81, 84),
                    })),
                    args: vec![
                        Expr::Literal(Literal::Number(5.0), Span::new(85, 86)),
                        Expr::Literal(Literal::Number(3.0), Span::new(88, 89)),
                    ],
                    span: Span::new(81, 90),
                }),
                span: Span::new(60, 91),
            })),
        ],
    };

    // Verify structure
    assert_eq!(program.items.len(), 2);

    // Verify function
    if let Item::Function(func) = &program.items[0] {
        assert_eq!(func.name.name, "add");
        assert_eq!(func.params.len(), 2);
        assert_eq!(func.body.statements.len(), 1);
    } else {
        panic!("Expected function declaration");
    }

    // Verify variable declaration
    if let Item::Statement(Stmt::VarDecl(var_decl)) = &program.items[1] {
        assert_eq!(var_decl.name.name, "result");
        assert!(!var_decl.mutable);
    } else {
        panic!("Expected variable declaration");
    }
}

#[test]
fn test_all_statement_types() {
    let statements = vec![
        // Variable declaration
        Stmt::VarDecl(VarDecl {
            mutable: true,
            name: Identifier {
                name: "x".to_string(),
                span: Span::new(0, 1),
            },
            type_ref: None,
            init: Expr::Literal(Literal::Number(42.0), Span::new(4, 6)),
            span: Span::new(0, 7),
        }),
        // Assignment
        Stmt::Assign(Assign {
            target: AssignTarget::Name(Identifier {
                name: "x".to_string(),
                span: Span::new(0, 1),
            }),
            value: Expr::Literal(Literal::Number(100.0), Span::new(4, 7)),
            span: Span::new(0, 8),
        }),
        // If statement
        Stmt::If(IfStmt {
            cond: Expr::Literal(Literal::Bool(true), Span::new(4, 8)),
            then_block: Block {
                statements: vec![],
                span: Span::new(9, 11),
            },
            else_block: Some(Block {
                statements: vec![],
                span: Span::new(17, 19),
            }),
            span: Span::new(0, 19),
        }),
        // While loop
        Stmt::While(WhileStmt {
            cond: Expr::Literal(Literal::Bool(true), Span::new(6, 10)),
            body: Block {
                statements: vec![],
                span: Span::new(11, 13),
            },
            span: Span::new(0, 13),
        }),
        // For loop
        Stmt::For(ForStmt {
            init: Box::new(Stmt::VarDecl(VarDecl {
                mutable: true,
                name: Identifier {
                    name: "i".to_string(),
                    span: Span::new(8, 9),
                },
                type_ref: None,
                init: Expr::Literal(Literal::Number(0.0), Span::new(12, 13)),
                span: Span::new(4, 14),
            })),
            cond: Expr::Binary(BinaryExpr {
                op: BinaryOp::Lt,
                left: Box::new(Expr::Identifier(Identifier {
                    name: "i".to_string(),
                    span: Span::new(16, 17),
                })),
                right: Box::new(Expr::Literal(Literal::Number(10.0), Span::new(20, 22))),
                span: Span::new(16, 22),
            }),
            step: Box::new(Stmt::Assign(Assign {
                target: AssignTarget::Name(Identifier {
                    name: "i".to_string(),
                    span: Span::new(24, 25),
                }),
                value: Expr::Binary(BinaryExpr {
                    op: BinaryOp::Add,
                    left: Box::new(Expr::Identifier(Identifier {
                        name: "i".to_string(),
                        span: Span::new(28, 29),
                    })),
                    right: Box::new(Expr::Literal(Literal::Number(1.0), Span::new(32, 33))),
                    span: Span::new(28, 33),
                }),
                span: Span::new(24, 33),
            })),
            body: Block {
                statements: vec![],
                span: Span::new(35, 37),
            },
            span: Span::new(0, 37),
        }),
        // Return statement
        Stmt::Return(ReturnStmt {
            value: Some(Expr::Literal(Literal::Number(42.0), Span::new(7, 9))),
            span: Span::new(0, 10),
        }),
        // Break statement
        Stmt::Break(Span::new(0, 5)),
        // Continue statement
        Stmt::Continue(Span::new(0, 8)),
        // Expression statement
        Stmt::Expr(ExprStmt {
            expr: Expr::Call(CallExpr {
                callee: Box::new(Expr::Identifier(Identifier {
                    name: "print".to_string(),
                    span: Span::new(0, 5),
                })),
                args: vec![Expr::Literal(
                    Literal::String("hello".to_string()),
                    Span::new(6, 13),
                )],
                span: Span::new(0, 14),
            }),
            span: Span::new(0, 15),
        }),
    ];

    assert_eq!(statements.len(), 9);

    // Verify each statement can be pattern matched
    assert!(matches!(statements[0], Stmt::VarDecl(_)));
    assert!(matches!(statements[1], Stmt::Assign(_)));
    assert!(matches!(statements[2], Stmt::If(_)));
    assert!(matches!(statements[3], Stmt::While(_)));
    assert!(matches!(statements[4], Stmt::For(_)));
    assert!(matches!(statements[5], Stmt::Return(_)));
    assert!(matches!(statements[6], Stmt::Break(_)));
    assert!(matches!(statements[7], Stmt::Continue(_)));
    assert!(matches!(statements[8], Stmt::Expr(_)));
}

#[test]
fn test_all_expression_types() {
    let expressions = vec![
        // Literal expressions
        Expr::Literal(Literal::Number(42.0), Span::new(0, 2)),
        Expr::Literal(Literal::String("hello".to_string()), Span::new(0, 7)),
        Expr::Literal(Literal::Bool(true), Span::new(0, 4)),
        Expr::Literal(Literal::Null, Span::new(0, 4)),
        // Identifier
        Expr::Identifier(Identifier {
            name: "x".to_string(),
            span: Span::new(0, 1),
        }),
        // Unary expressions
        Expr::Unary(UnaryExpr {
            op: UnaryOp::Negate,
            expr: Box::new(Expr::Literal(Literal::Number(5.0), Span::new(1, 2))),
            span: Span::new(0, 2),
        }),
        Expr::Unary(UnaryExpr {
            op: UnaryOp::Not,
            expr: Box::new(Expr::Literal(Literal::Bool(true), Span::new(1, 5))),
            span: Span::new(0, 5),
        }),
        // Binary expression
        Expr::Binary(BinaryExpr {
            op: BinaryOp::Add,
            left: Box::new(Expr::Literal(Literal::Number(1.0), Span::new(0, 1))),
            right: Box::new(Expr::Literal(Literal::Number(2.0), Span::new(4, 5))),
            span: Span::new(0, 5),
        }),
        // Call expression
        Expr::Call(CallExpr {
            callee: Box::new(Expr::Identifier(Identifier {
                name: "func".to_string(),
                span: Span::new(0, 4),
            })),
            args: vec![],
            span: Span::new(0, 6),
        }),
        // Index expression
        Expr::Index(IndexExpr {
            target: Box::new(Expr::Identifier(Identifier {
                name: "arr".to_string(),
                span: Span::new(0, 3),
            })),
            index: Box::new(Expr::Literal(Literal::Number(0.0), Span::new(4, 5))),
            span: Span::new(0, 6),
        }),
        // Array literal
        Expr::ArrayLiteral(ArrayLiteral {
            elements: vec![
                Expr::Literal(Literal::Number(1.0), Span::new(1, 2)),
                Expr::Literal(Literal::Number(2.0), Span::new(4, 5)),
                Expr::Literal(Literal::Number(3.0), Span::new(7, 8)),
            ],
            span: Span::new(0, 9),
        }),
        // Grouped expression
        Expr::Group(GroupExpr {
            expr: Box::new(Expr::Literal(Literal::Number(42.0), Span::new(1, 3))),
            span: Span::new(0, 4),
        }),
    ];

    assert_eq!(expressions.len(), 12);

    // Verify all expressions have valid spans
    for expr in &expressions {
        let span = expr.span();
        assert!(span.len() > 0 || span == Span::new(0, 4)); // Allow null literal span
    }
}

#[test]
fn test_all_binary_operators() {
    let operators = vec![
        BinaryOp::Add,
        BinaryOp::Sub,
        BinaryOp::Mul,
        BinaryOp::Div,
        BinaryOp::Mod,
        BinaryOp::Eq,
        BinaryOp::Ne,
        BinaryOp::Lt,
        BinaryOp::Le,
        BinaryOp::Gt,
        BinaryOp::Ge,
        BinaryOp::And,
        BinaryOp::Or,
    ];

    assert_eq!(operators.len(), 13);

    // Verify all operators can be used in expressions
    for op in operators {
        let expr = BinaryExpr {
            op,
            left: Box::new(Expr::Literal(Literal::Number(1.0), Span::new(0, 1))),
            right: Box::new(Expr::Literal(Literal::Number(2.0), Span::new(4, 5))),
            span: Span::new(0, 5),
        };

        assert_eq!(expr.op, op);
    }
}

#[test]
fn test_nested_expressions() {
    // Test deeply nested expression: (1 + 2) * (3 - 4)
    let expr = Expr::Binary(BinaryExpr {
        op: BinaryOp::Mul,
        left: Box::new(Expr::Group(GroupExpr {
            expr: Box::new(Expr::Binary(BinaryExpr {
                op: BinaryOp::Add,
                left: Box::new(Expr::Literal(Literal::Number(1.0), Span::new(1, 2))),
                right: Box::new(Expr::Literal(Literal::Number(2.0), Span::new(5, 6))),
                span: Span::new(1, 6),
            })),
            span: Span::new(0, 7),
        })),
        right: Box::new(Expr::Group(GroupExpr {
            expr: Box::new(Expr::Binary(BinaryExpr {
                op: BinaryOp::Sub,
                left: Box::new(Expr::Literal(Literal::Number(3.0), Span::new(11, 12))),
                right: Box::new(Expr::Literal(Literal::Number(4.0), Span::new(15, 16))),
                span: Span::new(11, 16),
            })),
            span: Span::new(10, 17),
        })),
        span: Span::new(0, 17),
    });

    assert_eq!(expr.span(), Span::new(0, 17));

    if let Expr::Binary(binary) = expr {
        assert_eq!(binary.op, BinaryOp::Mul);
        assert!(matches!(*binary.left, Expr::Group(_)));
        assert!(matches!(*binary.right, Expr::Group(_)));
    }
}

#[test]
fn test_array_type_ref() {
    // Test array type: number[][]
    let arr_type = TypeRef::Array(
        Box::new(TypeRef::Array(
            Box::new(TypeRef::Named("number".to_string(), Span::new(0, 6))),
            Span::new(0, 8),
        )),
        Span::new(0, 10),
    );

    assert_eq!(arr_type.span(), Span::new(0, 10));

    // Verify nested structure
    if let TypeRef::Array(inner, _) = arr_type {
        if let TypeRef::Array(inner_inner, _) = *inner {
            if let TypeRef::Named(name, _) = *inner_inner {
                assert_eq!(name, "number");
            } else {
                panic!("Expected named type");
            }
        } else {
            panic!("Expected array type");
        }
    } else {
        panic!("Expected array type");
    }
}

#[test]
fn test_assignment_target_variants() {
    // Test name assignment target
    let name_target = AssignTarget::Name(Identifier {
        name: "x".to_string(),
        span: Span::new(0, 1),
    });

    assert!(matches!(name_target, AssignTarget::Name(_)));

    // Test index assignment target
    let index_target = AssignTarget::Index {
        target: Box::new(Expr::Identifier(Identifier {
            name: "arr".to_string(),
            span: Span::new(0, 3),
        })),
        index: Box::new(Expr::Literal(Literal::Number(0.0), Span::new(4, 5))),
        span: Span::new(0, 6),
    };

    assert!(matches!(index_target, AssignTarget::Index { .. }));
}

#[test]
fn test_ast_serialization() {
    // Test that AST nodes can be serialized to JSON
    let program = Program {
        items: vec![Item::Statement(Stmt::VarDecl(VarDecl {
            mutable: false,
            name: Identifier {
                name: "x".to_string(),
                span: Span::new(4, 5),
            },
            type_ref: Some(TypeRef::Named("number".to_string(), Span::new(7, 13))),
            init: Expr::Literal(Literal::Number(42.0), Span::new(16, 18)),
            span: Span::new(0, 19),
        }))],
    };

    // Serialize to JSON
    let json = serde_json::to_string(&program).expect("Failed to serialize AST");

    // Deserialize back
    let deserialized: Program = serde_json::from_str(&json).expect("Failed to deserialize AST");

    assert_eq!(program, deserialized);
}
