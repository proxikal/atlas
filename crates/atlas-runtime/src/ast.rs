//! Abstract Syntax Tree (AST) definitions
//!
//! Complete AST implementation matching the Atlas specification.

use crate::span::Span;
use serde::{Deserialize, Serialize};

/// AST schema version
///
/// This version number is included in JSON dumps to ensure compatibility.
/// Increment when making breaking changes to the AST structure.
pub const AST_VERSION: u32 = 2;

/// Top-level program containing all items
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Program {
    pub items: Vec<Item>,
}

/// Versioned AST wrapper for JSON serialization
///
/// This struct wraps a Program with version metadata for stable JSON output.
/// Used when dumping AST to JSON for tooling and AI agents.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VersionedProgram {
    /// AST schema version
    pub ast_version: u32,
    /// The actual program AST
    #[serde(flatten)]
    pub program: Program,
}

impl VersionedProgram {
    /// Create a new versioned program wrapper
    pub fn new(program: Program) -> Self {
        Self {
            ast_version: AST_VERSION,
            program,
        }
    }

    /// Serialize to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize from JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

impl From<Program> for VersionedProgram {
    fn from(program: Program) -> Self {
        Self::new(program)
    }
}

/// Top-level item (function, statement, import, export, or extern)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Item {
    Function(FunctionDecl),
    Statement(Stmt),
    Import(ImportDecl),
    Export(ExportDecl),
    Extern(ExternDecl),
    TypeAlias(TypeAliasDecl),
}

/// Import declaration
///
/// Syntax: `import { x, y } from "./path"` or `import * as ns from "./path"`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImportDecl {
    /// What to import (named imports or namespace)
    pub specifiers: Vec<ImportSpecifier>,
    /// Module path (e.g., "./math", "/src/utils")
    pub source: String,
    pub span: Span,
}

/// Import specifier (what to import)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ImportSpecifier {
    /// Named import: `{ x }`
    Named { name: Identifier, span: Span },
    /// Namespace import: `* as ns`
    Namespace { alias: Identifier, span: Span },
}

/// Export declaration
///
/// Syntax: `export fn foo()` or `export let x = 5`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportDecl {
    /// What is being exported
    pub item: ExportItem,
    pub span: Span,
}

/// Exportable items
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExportItem {
    /// Export function: `export fn foo() {}`
    Function(FunctionDecl),
    /// Export variable: `export let x = 5`
    Variable(VarDecl),
    /// Export type alias: `export type Foo = bar`
    TypeAlias(TypeAliasDecl),
}

/// Extern function declaration (FFI)
///
/// Syntax: `extern fn name(param: c_type, ...) -> c_type from "library"`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExternDecl {
    pub name: String,
    pub library: String,
    pub symbol: Option<String>, // Optional symbol name (if different from name)
    pub params: Vec<(String, ExternTypeAnnotation)>,
    pub return_type: ExternTypeAnnotation,
    pub span: Span,
}

/// Type alias declaration
///
/// Syntax: `type Name = type_expr;`
/// Supports optional type parameters: `type Result<T, E> = ...;`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeAliasDecl {
    pub name: Identifier,
    /// Type parameters (e.g., <T, E> in type Foo<T, E> = ...)
    pub type_params: Vec<TypeParam>,
    /// Aliased type expression
    pub type_ref: TypeRef,
    /// Optional doc comment text (without leading ///)
    pub doc_comment: Option<String>,
    pub span: Span,
}

/// Extern type annotation for FFI signatures
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExternTypeAnnotation {
    CInt,
    CLong,
    CDouble,
    CCharPtr,
    CVoid,
    CBool,
}

/// Function declaration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionDecl {
    pub name: Identifier,
    /// Type parameters (e.g., <T, E> in fn foo<T, E>(...))
    pub type_params: Vec<TypeParam>,
    pub params: Vec<Param>,
    pub return_type: TypeRef,
    pub body: Block,
    pub span: Span,
}

/// Type parameter declaration (e.g., T in fn foo<T>(...))
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeParam {
    pub name: String,
    pub span: Span,
}

/// Function parameter
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Param {
    pub name: Identifier,
    pub type_ref: TypeRef,
    pub span: Span,
}

/// Block of statements
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Block {
    pub statements: Vec<Stmt>,
    pub span: Span,
}

/// Statement
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Stmt {
    VarDecl(VarDecl),
    FunctionDecl(FunctionDecl),
    Assign(Assign),
    CompoundAssign(CompoundAssign),
    Increment(IncrementStmt),
    Decrement(DecrementStmt),
    If(IfStmt),
    While(WhileStmt),
    For(ForStmt),
    ForIn(ForInStmt),
    Return(ReturnStmt),
    Break(Span),
    Continue(Span),
    Expr(ExprStmt),
}

/// Variable declaration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VarDecl {
    pub mutable: bool,
    pub name: Identifier,
    pub type_ref: Option<TypeRef>,
    pub init: Expr,
    pub span: Span,
}

/// Assignment statement
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Assign {
    pub target: AssignTarget,
    pub value: Expr,
    pub span: Span,
}

/// Assignment target (name or indexed expression)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AssignTarget {
    Name(Identifier),
    Index {
        target: Box<Expr>,
        index: Box<Expr>,
        span: Span,
    },
}

/// Compound assignment operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompoundOp {
    AddAssign, // +=
    SubAssign, // -=
    MulAssign, // *=
    DivAssign, // /=
    ModAssign, // %=
}

/// Compound assignment statement (+=, -=, *=, /=, %=)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompoundAssign {
    pub target: AssignTarget,
    pub op: CompoundOp,
    pub value: Expr,
    pub span: Span,
}

/// Increment statement (++)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IncrementStmt {
    pub target: AssignTarget,
    pub span: Span,
}

/// Decrement statement (--)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecrementStmt {
    pub target: AssignTarget,
    pub span: Span,
}

/// If statement
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IfStmt {
    pub cond: Expr,
    pub then_block: Block,
    pub else_block: Option<Block>,
    pub span: Span,
}

/// While loop
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WhileStmt {
    pub cond: Expr,
    pub body: Block,
    pub span: Span,
}

/// For loop
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ForStmt {
    pub init: Box<Stmt>,
    pub cond: Expr,
    pub step: Box<Stmt>,
    pub body: Block,
    pub span: Span,
}

/// For-in loop statement
///
/// Syntax: `for item in array { body }`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ForInStmt {
    /// Loop variable name
    pub variable: Identifier,
    /// Expression to iterate over
    pub iterable: Box<Expr>,
    /// Loop body
    pub body: Block,
    pub span: Span,
}

/// Return statement
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReturnStmt {
    pub value: Option<Expr>,
    pub span: Span,
}

/// Expression statement
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExprStmt {
    pub expr: Expr,
    pub span: Span,
}

/// Expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    Literal(Literal, Span),
    Identifier(Identifier),
    Unary(UnaryExpr),
    Binary(BinaryExpr),
    Call(CallExpr),
    Index(IndexExpr),
    Member(MemberExpr),
    ArrayLiteral(ArrayLiteral),
    Group(GroupExpr),
    Match(MatchExpr),
    Try(TryExpr),
}

/// Unary expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnaryExpr {
    pub op: UnaryOp,
    pub expr: Box<Expr>,
    pub span: Span,
}

/// Binary expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BinaryExpr {
    pub op: BinaryOp,
    pub left: Box<Expr>,
    pub right: Box<Expr>,
    pub span: Span,
}

/// Function call expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CallExpr {
    pub callee: Box<Expr>,
    pub args: Vec<Expr>,
    pub span: Span,
}

/// Array index expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IndexExpr {
    pub target: Box<Expr>,
    pub index: Box<Expr>,
    pub span: Span,
}

/// Member access expression (method call or property access)
///
/// Syntax: `expr.member` or `expr.method(args)`
/// This is sugar for function calls: `Type::method(expr, args)`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemberExpr {
    /// The target expression (left side of dot)
    pub target: Box<Expr>,
    /// The member name (right side of dot)
    pub member: Identifier,
    /// Arguments if this is a method call, None if property access
    pub args: Option<Vec<Expr>>,
    pub span: Span,
}

/// Array literal expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArrayLiteral {
    pub elements: Vec<Expr>,
    pub span: Span,
}

/// Grouped expression (parenthesized)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroupExpr {
    pub expr: Box<Expr>,
    pub span: Span,
}

/// Try expression (error propagation operator ?)
///
/// Unwraps Ok value or returns Err early from current function
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TryExpr {
    pub expr: Box<Expr>,
    pub span: Span,
}

/// Match expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchExpr {
    pub scrutinee: Box<Expr>,
    pub arms: Vec<MatchArm>,
    pub span: Span,
}

/// Match arm (pattern => expression)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Expr,
    pub span: Span,
}

/// Pattern for match expressions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Pattern {
    /// Literal pattern: 42, "hello", true, false, null
    Literal(Literal, Span),
    /// Wildcard pattern: _
    Wildcard(Span),
    /// Variable binding pattern: x, value, etc.
    Variable(Identifier),
    /// Constructor pattern: Ok(x), Err(e), Some(value), None
    Constructor {
        name: Identifier,
        args: Vec<Pattern>,
        span: Span,
    },
    /// Array pattern: [], [x], [x, y]
    Array { elements: Vec<Pattern>, span: Span },
}

/// Literal value
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Literal {
    Number(f64),
    String(String),
    Bool(bool),
    Null,
}

/// Identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Identifier {
    pub name: String,
    pub span: Span,
}

/// Type reference
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypeRef {
    Named(String, Span),
    Array(Box<TypeRef>, Span),
    Function {
        params: Vec<TypeRef>,
        return_type: Box<TypeRef>,
        span: Span,
    },
    /// Generic type application: Type<T1, T2, ...>
    Generic {
        name: String,
        type_args: Vec<TypeRef>,
        span: Span,
    },
}

/// Unary operator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnaryOp {
    Negate, // -
    Not,    // !
}

/// Binary operator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinaryOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    // Comparison
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    // Logical
    And,
    Or,
}

// Helper methods for getting spans from AST nodes

impl Expr {
    /// Get the span of this expression
    pub fn span(&self) -> Span {
        match self {
            Expr::Literal(_, span) => *span,
            Expr::Identifier(id) => id.span,
            Expr::Unary(u) => u.span,
            Expr::Binary(b) => b.span,
            Expr::Call(c) => c.span,
            Expr::Index(i) => i.span,
            Expr::Member(m) => m.span,
            Expr::ArrayLiteral(a) => a.span,
            Expr::Group(g) => g.span,
            Expr::Match(m) => m.span,
            Expr::Try(t) => t.span,
        }
    }
}

impl Stmt {
    /// Get the span of this statement
    pub fn span(&self) -> Span {
        match self {
            Stmt::VarDecl(v) => v.span,
            Stmt::FunctionDecl(f) => f.span,
            Stmt::Assign(a) => a.span,
            Stmt::CompoundAssign(c) => c.span,
            Stmt::Increment(i) => i.span,
            Stmt::Decrement(d) => d.span,
            Stmt::If(i) => i.span,
            Stmt::While(w) => w.span,
            Stmt::For(f) => f.span,
            Stmt::ForIn(f) => f.span,
            Stmt::Return(r) => r.span,
            Stmt::Break(s) | Stmt::Continue(s) => *s,
            Stmt::Expr(e) => e.span,
        }
    }
}

impl TypeRef {
    /// Get the span of this type reference
    pub fn span(&self) -> Span {
        match self {
            TypeRef::Named(_, span) => *span,
            TypeRef::Array(_, span) => *span,
            TypeRef::Function { span, .. } => *span,
            TypeRef::Generic { span, .. } => *span,
        }
    }
}

impl Pattern {
    /// Get the span of this pattern
    pub fn span(&self) -> Span {
        match self {
            Pattern::Literal(_, span) => *span,
            Pattern::Wildcard(span) => *span,
            Pattern::Variable(id) => id.span,
            Pattern::Constructor { span, .. } => *span,
            Pattern::Array { span, .. } => *span,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_program_creation() {
        let program = Program { items: vec![] };
        assert_eq!(program.items.len(), 0);
    }

    #[test]
    fn test_literal_expr() {
        let expr = Expr::Literal(Literal::Number(42.0), Span::new(0, 2));
        assert_eq!(expr.span(), Span::new(0, 2));

        if let Expr::Literal(Literal::Number(n), _) = expr {
            assert_eq!(n, 42.0);
        } else {
            panic!("Expected number literal");
        }
    }

    #[test]
    fn test_identifier() {
        let ident = Identifier {
            name: "x".to_string(),
            span: Span::new(0, 1),
        };
        assert_eq!(ident.name, "x");
        assert_eq!(ident.span, Span::new(0, 1));
    }

    #[test]
    fn test_binary_expr() {
        let left = Expr::Literal(Literal::Number(1.0), Span::new(0, 1));
        let right = Expr::Literal(Literal::Number(2.0), Span::new(4, 5));

        let binary = BinaryExpr {
            op: BinaryOp::Add,
            left: Box::new(left),
            right: Box::new(right),
            span: Span::new(0, 5),
        };

        assert_eq!(binary.op, BinaryOp::Add);
        assert_eq!(binary.span, Span::new(0, 5));
    }

    #[test]
    fn test_var_decl() {
        let var_decl = VarDecl {
            mutable: false,
            name: Identifier {
                name: "x".to_string(),
                span: Span::new(4, 5),
            },
            type_ref: Some(TypeRef::Named("number".to_string(), Span::new(7, 13))),
            init: Expr::Literal(Literal::Number(42.0), Span::new(16, 18)),
            span: Span::new(0, 19),
        };

        assert!(!var_decl.mutable);
        assert_eq!(var_decl.name.name, "x");
        assert!(var_decl.type_ref.is_some());
    }

    #[test]
    fn test_function_decl() {
        let func = FunctionDecl {
            name: Identifier {
                name: "test".to_string(),
                span: Span::new(5, 9),
            },
            type_params: vec![],
            params: vec![],
            return_type: TypeRef::Named("void".to_string(), Span::new(14, 18)),
            body: Block {
                statements: vec![],
                span: Span::new(19, 21),
            },
            span: Span::new(0, 21),
        };

        assert_eq!(func.name.name, "test");
        assert_eq!(func.params.len(), 0);
    }

    #[test]
    fn test_if_stmt() {
        let if_stmt = IfStmt {
            cond: Expr::Literal(Literal::Bool(true), Span::new(4, 8)),
            then_block: Block {
                statements: vec![],
                span: Span::new(9, 11),
            },
            else_block: None,
            span: Span::new(0, 11),
        };

        assert!(if_stmt.else_block.is_none());
        assert_eq!(if_stmt.span, Span::new(0, 11));
    }

    #[test]
    fn test_array_literal() {
        let arr = ArrayLiteral {
            elements: vec![
                Expr::Literal(Literal::Number(1.0), Span::new(1, 2)),
                Expr::Literal(Literal::Number(2.0), Span::new(4, 5)),
                Expr::Literal(Literal::Number(3.0), Span::new(7, 8)),
            ],
            span: Span::new(0, 9),
        };

        assert_eq!(arr.elements.len(), 3);
    }

    #[test]
    fn test_call_expr() {
        let call = CallExpr {
            callee: Box::new(Expr::Identifier(Identifier {
                name: "print".to_string(),
                span: Span::new(0, 5),
            })),
            args: vec![Expr::Literal(
                Literal::String("hello".to_string()),
                Span::new(6, 13),
            )],
            span: Span::new(0, 14),
        };

        assert_eq!(call.args.len(), 1);
    }

    #[test]
    fn test_stmt_span() {
        let stmt = Stmt::Break(Span::new(0, 5));
        assert_eq!(stmt.span(), Span::new(0, 5));

        let stmt2 = Stmt::Continue(Span::new(10, 18));
        assert_eq!(stmt2.span(), Span::new(10, 18));
    }

    #[test]
    fn test_type_ref_array() {
        let arr_type = TypeRef::Array(
            Box::new(TypeRef::Named("number".to_string(), Span::new(0, 6))),
            Span::new(0, 8),
        );

        assert_eq!(arr_type.span(), Span::new(0, 8));

        if let TypeRef::Array(inner, _) = arr_type {
            if let TypeRef::Named(name, _) = *inner {
                assert_eq!(name, "number");
            } else {
                panic!("Expected named type");
            }
        } else {
            panic!("Expected array type");
        }
    }

    // === AST Versioning Tests (Phase 08) ===

    #[test]
    fn test_ast_version_constant() {
        // Verify AST_VERSION is set to 2
        assert_eq!(AST_VERSION, 2);
    }

    #[test]
    fn test_versioned_program_creation() {
        let program = Program { items: vec![] };
        let versioned = VersionedProgram::new(program);

        assert_eq!(versioned.ast_version, AST_VERSION);
        assert_eq!(versioned.ast_version, 2);
        assert_eq!(versioned.program.items.len(), 0);
    }

    #[test]
    fn test_versioned_program_from_program() {
        let program = Program { items: vec![] };
        let versioned: VersionedProgram = program.into();

        assert_eq!(versioned.ast_version, 2);
    }

    #[test]
    fn test_versioned_program_to_json() {
        let program = Program { items: vec![] };
        let versioned = VersionedProgram::new(program);

        let json = versioned.to_json().expect("Failed to serialize to JSON");

        // Verify JSON contains ast_version field
        assert!(json.contains("\"ast_version\""));
        assert!(json.contains("\"ast_version\": 2"));
        assert!(json.contains("\"items\""));
    }

    #[test]
    fn test_versioned_program_from_json() {
        let json = r#"{
            "ast_version": 2,
            "items": []
        }"#;

        let versioned = VersionedProgram::from_json(json).expect("Failed to parse JSON");

        assert_eq!(versioned.ast_version, 2);
        assert_eq!(versioned.program.items.len(), 0);
    }

    #[test]
    fn test_versioned_program_with_content() {
        // Create a program with an actual statement
        let program = Program {
            items: vec![Item::Statement(Stmt::Expr(ExprStmt {
                expr: Expr::Literal(Literal::Number(42.0), Span::new(0, 2)),
                span: Span::new(0, 2),
            }))],
        };

        let versioned = VersionedProgram::new(program);
        let json = versioned.to_json().expect("Failed to serialize");

        // Verify version is included in JSON with content
        assert!(json.contains("\"ast_version\": 2"));
        assert!(json.contains("\"items\""));
    }

    #[test]
    fn test_versioned_program_round_trip() {
        // Create a simple program
        let original_program = Program {
            items: vec![Item::Statement(Stmt::Expr(ExprStmt {
                expr: Expr::Literal(Literal::Bool(true), Span::new(0, 4)),
                span: Span::new(0, 4),
            }))],
        };

        let versioned = VersionedProgram::new(original_program.clone());

        // Serialize to JSON
        let json = versioned.to_json().expect("Failed to serialize");

        // Deserialize back
        let deserialized = VersionedProgram::from_json(&json).expect("Failed to deserialize");

        // Verify version is preserved
        assert_eq!(deserialized.ast_version, 2);
        assert_eq!(deserialized.program.items.len(), 1);
    }

    #[test]
    fn test_version_mismatch_detection() {
        // Test with a future version number (forward compatibility test)
        let json_future = r#"{
            "ast_version": 3,
            "items": []
        }"#;

        // Should still parse (for forward compatibility)
        let result = VersionedProgram::from_json(json_future);
        assert!(result.is_ok(), "Should parse future versions");

        if let Ok(versioned) = result {
            // Can detect version mismatch
            assert_ne!(versioned.ast_version, AST_VERSION);
            assert_eq!(versioned.ast_version, 3);
        }
    }

    #[test]
    fn test_missing_version_field() {
        // Test JSON without version field (backward compatibility)
        let json_no_version = r#"{
            "items": []
        }"#;

        // This should fail because ast_version is required
        let result = VersionedProgram::from_json(json_no_version);
        assert!(result.is_err(), "Should fail without ast_version field");
    }

    #[test]
    fn test_ast_dump_field_order() {
        // Verify ast_version comes first in JSON output
        let program = Program { items: vec![] };
        let versioned = VersionedProgram::new(program);
        let json = versioned.to_json().expect("Failed to serialize");

        // ast_version should appear before items in the JSON
        let version_pos = json.find("\"ast_version\"").expect("ast_version not found");
        let items_pos = json.find("\"items\"").expect("items not found");

        assert!(
            version_pos < items_pos,
            "ast_version should appear before items in JSON output"
        );
    }
}
