# Type Checker

Type checking strategy with inference and strict rules.

## Type Checker Structure

```rust
// typechecker/ module (typechecker/mod.rs + typechecker/expr.rs)
use std::collections::{HashMap, HashSet};

pub struct TypeChecker<'a> {
    symbol_table: &'a SymbolTable,
    diagnostics: Vec<Diagnostic>,
    current_function_return_type: Option<Type>,
    in_loop: bool,
    // Warning tracking (added in phase-14-warnings)
    declared_symbols: HashMap<String, (Span, SymbolKind)>,  // Per-function tracking
    used_symbols: HashSet<String>,                            // Per-function tracking
}

impl<'a> TypeChecker<'a> {
    pub fn new(symbol_table: &'a SymbolTable) -> Self {
        Self {
            symbol_table,
            diagnostics: Vec::new(),
            current_function_return_type: None,
            in_loop: false,
        }
    }

    pub fn check(&mut self, program: &Program) -> Vec<Diagnostic> {
        for item in &program.items {
            self.check_item(item);
        }
        std::mem::take(&mut self.diagnostics)
    }

    fn check_item(&mut self, item: &Item) {
        match item {
            Item::Function(func) => self.check_function(func),
            Item::Statement(stmt) => self.check_statement(stmt),
        }
    }

    fn check_function(&mut self, func: &FunctionDecl) {
        let return_type = self.resolve_type_ref(&func.return_type);
        self.current_function_return_type = Some(return_type.clone());

        self.check_block(&func.body);

        // Check if all paths return (if return type != void)
        if return_type != Type::Void && !self.block_always_returns(&func.body) {
            self.diagnostics.push(
                Diagnostic::error(
                    "AT0004",
                    "Not all code paths return a value",
                    func.span
                )
            );
        }

        self.current_function_return_type = None;
    }

    fn check_statement(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::VarDecl(var) => {
                let init_type = self.check_expr(&var.init);

                if let Some(type_ref) = &var.type_ref {
                    let declared_type = self.resolve_type_ref(type_ref);
                    if !init_type.is_assignable_to(&declared_type) {
                        self.diagnostics.push(
                            Diagnostic::error(
                                "AT0001",
                                &format!(
                                    "Type mismatch: expected {}, found {}",
                                    declared_type.to_string(),
                                    init_type.to_string()
                                ),
                                var.span
                            )
                        );
                    }
                }
            }
            Stmt::Assign(assign) => {
                let value_type = self.check_expr(&assign.value);
                let target_type = self.check_assign_target(&assign.target);

                if !value_type.is_assignable_to(&target_type) {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "AT0001",
                            &format!(
                                "Type mismatch in assignment: cannot assign {} to {}",
                                value_type.to_string(),
                                target_type.to_string()
                            ),
                            assign.span
                        )
                    );
                }

                // Check mutability
                if let AssignTarget::Name(id) = &assign.target {
                    if let Some(symbol) = self.symbol_table.resolve(&id.name) {
                        if !symbol.mutable {
                            self.diagnostics.push(
                                Diagnostic::error(
                                    "AT0003",
                                    &format!("Cannot assign to immutable variable '{}'", id.name),
                                    id.span
                                )
                            );
                        }
                    }
                }
            }
            Stmt::If(if_stmt) => {
                let cond_type = self.check_expr(&if_stmt.cond);
                if cond_type != Type::Bool {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "AT0001",
                            &format!("Condition must be bool, found {}", cond_type.to_string()),
                            if_stmt.cond.span()
                        )
                    );
                }
                self.check_block(&if_stmt.then_block);
                if let Some(else_block) = &if_stmt.else_block {
                    self.check_block(else_block);
                }
            }
            Stmt::While(while_stmt) => {
                let cond_type = self.check_expr(&while_stmt.cond);
                if cond_type != Type::Bool {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "AT0001",
                            &format!("Condition must be bool, found {}", cond_type.to_string()),
                            while_stmt.cond.span()
                        )
                    );
                }
                let old_in_loop = self.in_loop;
                self.in_loop = true;
                self.check_block(&while_stmt.body);
                self.in_loop = old_in_loop;
            }
            Stmt::Return(ret) => {
                if self.current_function_return_type.is_none() {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "AT1011",
                            "Return statement outside function",
                            ret.span
                        )
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
                        Diagnostic::error(
                            "AT0001",
                            &format!(
                                "Return type mismatch: expected {}, found {}",
                                expected.to_string(),
                                return_type.to_string()
                            ),
                            ret.span
                        )
                    );
                }
            }
            Stmt::Break(span) | Stmt::Continue(span) => {
                if !self.in_loop {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "AT1010",
                            "break/continue outside loop",
                            *span
                        )
                    );
                }
            }
            _ => {}
        }
    }

    fn check_expr(&mut self, expr: &Expr) -> Type {
        match expr {
            Expr::Literal(lit, _) => match lit {
                Literal::Number(_) => Type::Number,
                Literal::String(_) => Type::String,
                Literal::Bool(_) => Type::Bool,
                Literal::Null => Type::Null,
            },
            Expr::Identifier(id) => {
                if let Some(symbol) = self.symbol_table.resolve(&id.name) {
                    symbol.ty.clone()
                } else {
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

    fn check_binary(&mut self, binary: &BinaryExpr) -> Type {
        let left_type = self.check_expr(&binary.left);
        let right_type = self.check_expr(&binary.right);

        match binary.op {
            BinaryOp::Add => {
                if (left_type == Type::Number && right_type == Type::Number) ||
                   (left_type == Type::String && right_type == Type::String) {
                    left_type
                } else {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "AT0001",
                            &format!(
                                "'+' requires both operands to be number or both to be string, found {} and {}",
                                left_type.to_string(),
                                right_type.to_string()
                            ),
                            binary.span
                        )
                    );
                    Type::Unknown
                }
            }
            BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                if left_type == Type::Number && right_type == Type::Number {
                    Type::Number
                } else {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "AT0001",
                            &format!(
                                "Arithmetic operator requires number operands, found {} and {}",
                                left_type.to_string(),
                                right_type.to_string()
                            ),
                            binary.span
                        )
                    );
                    Type::Unknown
                }
            }
            BinaryOp::Eq | BinaryOp::Ne => {
                if !left_type.is_assignable_to(&right_type) &&
                   !right_type.is_assignable_to(&left_type) {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "AT0001",
                            &format!(
                                "Equality comparison requires same types, found {} and {}",
                                left_type.to_string(),
                                right_type.to_string()
                            ),
                            binary.span
                        )
                    );
                }
                Type::Bool
            }
            BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
                if left_type == Type::Number && right_type == Type::Number {
                    Type::Bool
                } else {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "AT0001",
                            &format!(
                                "Comparison requires number operands, found {} and {}",
                                left_type.to_string(),
                                right_type.to_string()
                            ),
                            binary.span
                        )
                    );
                    Type::Bool
                }
            }
            BinaryOp::And | BinaryOp::Or => {
                if left_type != Type::Bool || right_type != Type::Bool {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "AT0001",
                            "Logical operators require bool operands",
                            binary.span
                        )
                    );
                }
                Type::Bool
            }
        }
    }
}
```

## Key Type Rules

- **No implicit any:** All types must be explicit or inferrable
- **No nullable:** `null` only assigns to `null` type
- **No truthy/falsey:** Conditionals require `bool`
- **Strict equality:** `==` requires same-type operands
- **Number arithmetic:** `+ - * / %` only for numbers (except `+` for strings)
- **Comparisons:** `< <= > >=` only for numbers
