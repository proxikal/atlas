//! Type checking and inference
//!
//! The type checker enforces Atlas's strict type rules:
//! - No implicit any - all types must be explicit or inferrable
//! - No nullable - null only assigns to null type
//! - No truthy/falsey - conditionals require bool
//! - Strict equality - == requires same-type operands

use crate::ast::*;
use crate::diagnostic::Diagnostic;
use crate::symbol::SymbolTable;
use crate::types::Type;

/// Type checker state
pub struct TypeChecker<'a> {
    /// Symbol table from binder
    symbol_table: &'a SymbolTable,
    /// Collected diagnostics
    diagnostics: Vec<Diagnostic>,
    /// Current function's return type (for return statement checking)
    current_function_return_type: Option<Type>,
    /// Whether we're inside a loop (for break/continue checking)
    in_loop: bool,
}

impl<'a> TypeChecker<'a> {
    /// Create a new type checker
    pub fn new(symbol_table: &'a SymbolTable) -> Self {
        Self {
            symbol_table,
            diagnostics: Vec::new(),
            current_function_return_type: None,
            in_loop: false,
        }
    }

    /// Type check a program
    pub fn check(&mut self, program: &Program) -> Vec<Diagnostic> {
        for item in &program.items {
            self.check_item(item);
        }
        std::mem::take(&mut self.diagnostics)
    }

    /// Check a top-level item
    fn check_item(&mut self, item: &Item) {
        match item {
            Item::Function(func) => self.check_function(func),
            Item::Statement(stmt) => self.check_statement(stmt),
        }
    }

    /// Check a function declaration
    fn check_function(&mut self, func: &FunctionDecl) {
        let return_type = self.resolve_type_ref(&func.return_type);
        self.current_function_return_type = Some(return_type.clone());

        self.check_block(&func.body);

        // Check if all paths return (if return type != void/null)
        if return_type != Type::Void && return_type != Type::Null {
            if !self.block_always_returns(&func.body) {
                self.diagnostics.push(
                    Diagnostic::error_with_code(
                        "AT3004",
                        "Not all code paths return a value",
                        func.span,
                    )
                    .with_label("function body"),
                );
            }
        }

        self.current_function_return_type = None;
    }

    /// Check a block
    fn check_block(&mut self, block: &Block) {
        for stmt in &block.statements {
            self.check_statement(stmt);
        }
    }

    /// Check if a block always returns
    fn block_always_returns(&self, block: &Block) -> bool {
        for stmt in &block.statements {
            if self.statement_always_returns(stmt) {
                return true;
            }
        }
        false
    }

    /// Check if a statement always returns
    fn statement_always_returns(&self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::Return(_) => true,
            Stmt::If(if_stmt) => {
                if let Some(else_block) = &if_stmt.else_block {
                    self.block_always_returns(&if_stmt.then_block)
                        && self.block_always_returns(else_block)
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Check a statement
    fn check_statement(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::VarDecl(var) => {
                let init_type = self.check_expr(&var.init);

                if let Some(type_ref) = &var.type_ref {
                    let declared_type = self.resolve_type_ref(type_ref);
                    if !init_type.is_assignable_to(&declared_type) {
                        self.diagnostics.push(
                            Diagnostic::error_with_code(
                                "AT3001",
                                &format!(
                                    "Type mismatch: cannot assign {} to variable of type {}",
                                    init_type.display_name(),
                                    declared_type.display_name()
                                ),
                                var.span,
                            )
                            .with_label("type mismatch"),
                        );
                    }
                }
            }
            Stmt::Assign(assign) => {
                let value_type = self.check_expr(&assign.value);
                let target_type = self.check_assign_target(&assign.target);

                if !value_type.is_assignable_to(&target_type) {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3001",
                            &format!(
                                "Type mismatch in assignment: cannot assign {} to {}",
                                value_type.display_name(),
                                target_type.display_name()
                            ),
                            assign.span,
                        )
                        .with_label("type mismatch"),
                    );
                }

                // Check mutability
                if let AssignTarget::Name(id) = &assign.target {
                    if let Some(symbol) = self.symbol_table.lookup(&id.name) {
                        if !symbol.mutable {
                            self.diagnostics.push(
                                Diagnostic::error_with_code(
                                    "AT3003",
                                    &format!("Cannot assign to immutable variable '{}'", id.name),
                                    id.span,
                                )
                                .with_label("immutable variable"),
                            );
                        }
                    }
                }
            }
            Stmt::If(if_stmt) => {
                let cond_type = self.check_expr(&if_stmt.cond);
                if cond_type != Type::Bool && cond_type != Type::Unknown {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3001",
                            &format!(
                                "Condition must be bool, found {}",
                                cond_type.display_name()
                            ),
                            if_stmt.cond.span(),
                        )
                        .with_label("type mismatch"),
                    );
                }
                self.check_block(&if_stmt.then_block);
                if let Some(else_block) = &if_stmt.else_block {
                    self.check_block(else_block);
                }
            }
            Stmt::While(while_stmt) => {
                let cond_type = self.check_expr(&while_stmt.cond);
                if cond_type != Type::Bool && cond_type != Type::Unknown {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3001",
                            &format!(
                                "Condition must be bool, found {}",
                                cond_type.display_name()
                            ),
                            while_stmt.cond.span(),
                        )
                        .with_label("type mismatch"),
                    );
                }
                let old_in_loop = self.in_loop;
                self.in_loop = true;
                self.check_block(&while_stmt.body);
                self.in_loop = old_in_loop;
            }
            Stmt::For(for_stmt) => {
                self.check_statement(&for_stmt.init);
                let cond_type = self.check_expr(&for_stmt.cond);
                if cond_type != Type::Bool && cond_type != Type::Unknown {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3001",
                            &format!(
                                "Condition must be bool, found {}",
                                cond_type.display_name()
                            ),
                            for_stmt.cond.span(),
                        )
                        .with_label("type mismatch"),
                    );
                }
                self.check_statement(&for_stmt.step);

                let old_in_loop = self.in_loop;
                self.in_loop = true;
                self.check_block(&for_stmt.body);
                self.in_loop = old_in_loop;
            }
            Stmt::Return(ret) => {
                if self.current_function_return_type.is_none() {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3011",
                            "Return statement outside function",
                            ret.span,
                        )
                        .with_label("invalid return"),
                    );
                    return;
                }

                let return_type = if let Some(value) = &ret.value {
                    self.check_expr(value)
                } else {
                    Type::Void
                };

                let expected = self.current_function_return_type.as_ref().unwrap();
                if !return_type.is_assignable_to(expected) {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3001",
                            &format!(
                                "Return type mismatch: expected {}, found {}",
                                expected.display_name(),
                                return_type.display_name()
                            ),
                            ret.span,
                        )
                        .with_label("type mismatch"),
                    );
                }
            }
            Stmt::Break(span) => {
                if !self.in_loop {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3010",
                            "Break statement outside loop",
                            *span,
                        )
                        .with_label("invalid break"),
                    );
                }
            }
            Stmt::Continue(span) => {
                if !self.in_loop {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3010",
                            "Continue statement outside loop",
                            *span,
                        )
                        .with_label("invalid continue"),
                    );
                }
            }
            Stmt::Expr(expr_stmt) => {
                self.check_expr(&expr_stmt.expr);
            }
        }
    }

    /// Check an assignment target and return its type
    fn check_assign_target(&mut self, target: &AssignTarget) -> Type {
        match target {
            AssignTarget::Name(id) => {
                if let Some(symbol) = self.symbol_table.lookup(&id.name) {
                    symbol.ty.clone()
                } else {
                    Type::Unknown
                }
            }
            AssignTarget::Index { target, index, .. } => {
                let target_type = self.check_expr(target);
                let index_type = self.check_expr(index);

                // Check that index is a number
                if index_type != Type::Number && index_type != Type::Unknown {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3001",
                            &format!(
                                "Array index must be number, found {}",
                                index_type.display_name()
                            ),
                            index.span(),
                        )
                        .with_label("type mismatch"),
                    );
                }

                // Extract element type from array
                match target_type {
                    Type::Array(elem_type) => *elem_type,
                    Type::Unknown => Type::Unknown,
                    _ => {
                        self.diagnostics.push(
                            Diagnostic::error_with_code(
                                "AT3001",
                                &format!(
                                    "Cannot index into non-array type {}",
                                    target_type.display_name()
                                ),
                                target.span(),
                            )
                            .with_label("not an array"),
                        );
                        Type::Unknown
                    }
                }
            }
        }
    }

    /// Check an expression and return its type
    fn check_expr(&mut self, expr: &Expr) -> Type {
        match expr {
            Expr::Literal(lit, _) => match lit {
                Literal::Number(_) => Type::Number,
                Literal::String(_) => Type::String,
                Literal::Bool(_) => Type::Bool,
                Literal::Null => Type::Null,
            },
            Expr::Identifier(id) => {
                if let Some(symbol) = self.symbol_table.lookup(&id.name) {
                    symbol.ty.clone()
                } else {
                    // Binder should have caught this
                    Type::Unknown
                }
            }
            Expr::Binary(binary) => self.check_binary(binary),
            Expr::Unary(unary) => self.check_unary(unary),
            Expr::Call(call) => self.check_call(call),
            Expr::Index(index) => self.check_index(index),
            Expr::ArrayLiteral(arr) => self.check_array_literal(arr),
            Expr::Group(group) => self.check_expr(&group.expr),
        }
    }

    /// Check a binary expression
    fn check_binary(&mut self, binary: &BinaryExpr) -> Type {
        let left_type = self.check_expr(&binary.left);
        let right_type = self.check_expr(&binary.right);

        // Skip type checking if either side is Unknown (error recovery)
        if left_type == Type::Unknown || right_type == Type::Unknown {
            return Type::Unknown;
        }

        match binary.op {
            BinaryOp::Add => {
                if (left_type == Type::Number && right_type == Type::Number)
                    || (left_type == Type::String && right_type == Type::String)
                {
                    left_type
                } else {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3002",
                            &format!(
                                "'+' requires both operands to be number or both to be string, found {} and {}",
                                left_type.display_name(),
                                right_type.display_name()
                            ),
                            binary.span,
                        )
                        .with_label("type mismatch"),
                    );
                    Type::Unknown
                }
            }
            BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                if left_type == Type::Number && right_type == Type::Number {
                    Type::Number
                } else {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3002",
                            &format!(
                                "Arithmetic operator requires number operands, found {} and {}",
                                left_type.display_name(),
                                right_type.display_name()
                            ),
                            binary.span,
                        )
                        .with_label("type mismatch"),
                    );
                    Type::Unknown
                }
            }
            BinaryOp::Eq | BinaryOp::Ne => {
                // Equality requires same types
                if left_type != right_type {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3002",
                            &format!(
                                "Equality comparison requires same types, found {} and {}",
                                left_type.display_name(),
                                right_type.display_name()
                            ),
                            binary.span,
                        )
                        .with_label("type mismatch"),
                    );
                }
                Type::Bool
            }
            BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
                if left_type == Type::Number && right_type == Type::Number {
                    Type::Bool
                } else {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3002",
                            &format!(
                                "Comparison requires number operands, found {} and {}",
                                left_type.display_name(),
                                right_type.display_name()
                            ),
                            binary.span,
                        )
                        .with_label("type mismatch"),
                    );
                    Type::Bool // Still return bool for error recovery
                }
            }
            BinaryOp::And | BinaryOp::Or => {
                if left_type != Type::Bool || right_type != Type::Bool {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3002",
                            &format!(
                                "Logical operators require bool operands, found {} and {}",
                                left_type.display_name(),
                                right_type.display_name()
                            ),
                            binary.span,
                        )
                        .with_label("type mismatch"),
                    );
                }
                Type::Bool
            }
        }
    }

    /// Check a unary expression
    fn check_unary(&mut self, unary: &UnaryExpr) -> Type {
        let expr_type = self.check_expr(&unary.expr);

        match unary.op {
            UnaryOp::Negate => {
                if expr_type != Type::Number && expr_type != Type::Unknown {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3002",
                            &format!(
                                "Unary '-' requires number operand, found {}",
                                expr_type.display_name()
                            ),
                            unary.span,
                        )
                        .with_label("type mismatch"),
                    );
                    Type::Unknown
                } else {
                    Type::Number
                }
            }
            UnaryOp::Not => {
                if expr_type != Type::Bool && expr_type != Type::Unknown {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3002",
                            &format!(
                                "Unary '!' requires bool operand, found {}",
                                expr_type.display_name()
                            ),
                            unary.span,
                        )
                        .with_label("type mismatch"),
                    );
                    Type::Unknown
                } else {
                    Type::Bool
                }
            }
        }
    }

    /// Check a function call
    fn check_call(&mut self, call: &CallExpr) -> Type {
        let callee_type = self.check_expr(&call.callee);

        match callee_type {
            Type::Function {
                params,
                return_type,
            } => {
                // Check argument count
                if call.args.len() != params.len() {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3005",
                            &format!(
                                "Function expects {} arguments, found {}",
                                params.len(),
                                call.args.len()
                            ),
                            call.span,
                        )
                        .with_label("argument count mismatch"),
                    );
                }

                // Check argument types
                for (i, arg) in call.args.iter().enumerate() {
                    let arg_type = self.check_expr(arg);
                    if let Some(expected_type) = params.get(i) {
                        if !arg_type.is_assignable_to(expected_type) && arg_type != Type::Unknown {
                            self.diagnostics.push(
                                Diagnostic::error_with_code(
                                    "AT3001",
                                    &format!(
                                        "Argument {} has wrong type: expected {}, found {}",
                                        i + 1,
                                        expected_type.display_name(),
                                        arg_type.display_name()
                                    ),
                                    arg.span(),
                                )
                                .with_label("type mismatch"),
                            );
                        }
                    }
                }

                *return_type
            }
            Type::Unknown => Type::Unknown, // Error recovery
            _ => {
                self.diagnostics.push(
                    Diagnostic::error_with_code(
                        "AT3006",
                        &format!(
                            "Cannot call non-function type {}",
                            callee_type.display_name()
                        ),
                        call.span,
                    )
                    .with_label("not callable"),
                );
                Type::Unknown
            }
        }
    }

    /// Check an index expression
    fn check_index(&mut self, index: &IndexExpr) -> Type {
        let target_type = self.check_expr(&index.target);
        let index_type = self.check_expr(&index.index);

        // Check that index is a number
        if index_type != Type::Number && index_type != Type::Unknown {
            self.diagnostics.push(
                Diagnostic::error_with_code(
                    "AT3001",
                    &format!(
                        "Array index must be number, found {}",
                        index_type.display_name()
                    ),
                    index.index.span(),
                )
                .with_label("type mismatch"),
            );
        }

        // Extract element type from array
        match target_type {
            Type::Array(elem_type) => *elem_type,
            Type::Unknown => Type::Unknown,
            _ => {
                self.diagnostics.push(
                    Diagnostic::error_with_code(
                        "AT3001",
                        &format!(
                            "Cannot index into non-array type {}",
                            target_type.display_name()
                        ),
                        index.target.span(),
                    )
                    .with_label("not an array"),
                );
                Type::Unknown
            }
        }
    }

    /// Check an array literal
    fn check_array_literal(&mut self, arr: &ArrayLiteral) -> Type {
        if arr.elements.is_empty() {
            // Empty array - infer as array of unknown
            return Type::Array(Box::new(Type::Unknown));
        }

        // Check first element to determine array type
        let first_type = self.check_expr(&arr.elements[0]);

        // Check that all elements have the same type
        for (i, elem) in arr.elements.iter().enumerate().skip(1) {
            let elem_type = self.check_expr(elem);
            if !elem_type.is_assignable_to(&first_type) && elem_type != Type::Unknown {
                self.diagnostics.push(
                    Diagnostic::error_with_code(
                        "AT3001",
                        &format!(
                            "Array element {} has wrong type: expected {}, found {}",
                            i,
                            first_type.display_name(),
                            elem_type.display_name()
                        ),
                        elem.span(),
                    )
                    .with_label("type mismatch"),
                );
            }
        }

        Type::Array(Box::new(first_type))
    }

    /// Resolve a type reference to a Type
    fn resolve_type_ref(&self, type_ref: &TypeRef) -> Type {
        match type_ref {
            TypeRef::Named(name, _) => match name.as_str() {
                "number" => Type::Number,
                "string" => Type::String,
                "bool" => Type::Bool,
                "void" => Type::Void,
                "null" => Type::Null,
                _ => Type::Unknown,
            },
            TypeRef::Array(elem, _) => Type::Array(Box::new(self.resolve_type_ref(elem))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binder::Binder;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn typecheck_source(source: &str) -> Vec<Diagnostic> {
        let mut lexer = Lexer::new(source);
        let (tokens, _) = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (program, _) = parser.parse();

        let mut binder = Binder::new();
        let (table, _) = binder.bind(&program);

        let mut checker = TypeChecker::new(&table);
        checker.check(&program)
    }

    #[test]
    fn test_valid_variable() {
        let diagnostics = typecheck_source("let x: number = 42;");
        assert_eq!(diagnostics.len(), 0);
    }

    #[test]
    fn test_type_mismatch() {
        let diagnostics = typecheck_source("let x: number = \"hello\";");
        assert!(diagnostics.len() > 0);
        assert_eq!(diagnostics[0].code, "AT3001");
    }

    #[test]
    fn test_arithmetic_type_error() {
        let diagnostics = typecheck_source(r#"let x = 5 + "hello";"#);
        assert!(diagnostics.len() > 0);
        assert_eq!(diagnostics[0].code, "AT3002");
    }

    #[test]
    fn test_condition_must_be_bool() {
        let diagnostics = typecheck_source("if (5) { }");
        assert!(diagnostics.len() > 0);
        assert_eq!(diagnostics[0].code, "AT3001");
    }

    #[test]
    fn test_immutable_assignment() {
        let diagnostics = typecheck_source(r#"
            let x = 5;
            x = 10;
        "#);
        assert!(diagnostics.len() > 0);
        assert_eq!(diagnostics[0].code, "AT3003");
    }

    #[test]
    fn test_break_outside_loop() {
        let diagnostics = typecheck_source("break;");
        assert!(diagnostics.len() > 0);
        assert_eq!(diagnostics[0].code, "AT3010");
    }

    #[test]
    fn test_return_outside_function() {
        let diagnostics = typecheck_source("return 5;");
        assert!(diagnostics.len() > 0);
        assert_eq!(diagnostics[0].code, "AT3011");
    }
}
