//! Expression type checking

use crate::ast::*;
use crate::diagnostic::Diagnostic;
use crate::span::Span;
use crate::typechecker::suggestions;
use crate::typechecker::TypeChecker;
use crate::types::Type;

impl<'a> TypeChecker<'a> {
    /// Check an expression and return its type
    pub(super) fn check_expr(&mut self, expr: &Expr) -> Type {
        match expr {
            Expr::Literal(lit, _) => match lit {
                Literal::Number(_) => Type::Number,
                Literal::String(_) => Type::String,
                Literal::Bool(_) => Type::Bool,
                Literal::Null => Type::Null,
            },
            Expr::Identifier(id) => {
                // Track that this symbol was used
                self.used_symbols.insert(id.name.clone());

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
            Expr::Match(match_expr) => self.check_match(match_expr),
            Expr::Member(member) => self.check_member(member),
            Expr::Try(try_expr) => self.check_try(try_expr),
        }
    }

    /// Check a binary expression
    fn check_binary(&mut self, binary: &BinaryExpr) -> Type {
        let left_type = self.check_expr(&binary.left);
        let right_type = self.check_expr(&binary.right);
        let left_norm = left_type.normalized();
        let right_norm = right_type.normalized();

        // Skip type checking if either side is Unknown (error recovery)
        if left_norm == Type::Unknown || right_norm == Type::Unknown {
            return Type::Unknown;
        }

        match binary.op {
            BinaryOp::Add => {
                if (left_norm == Type::Number && right_norm == Type::Number)
                    || (left_norm == Type::String && right_norm == Type::String)
                {
                    left_norm
                } else {
                    let help = suggestions::suggest_binary_operator_fix("+", &left_type, &right_type)
                        .unwrap_or_else(|| "ensure both operands are numbers (for addition) or both are strings (for concatenation)".to_string());
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3002",
                            format!(
                                "'+' requires matching types, found {} and {}",
                                left_type.display_name(),
                                right_type.display_name()
                            ),
                            binary.span,
                        )
                        .with_label(format!(
                            "found {} and {}",
                            left_type.display_name(),
                            right_type.display_name()
                        ))
                        .with_help(help),
                    );
                    Type::Unknown
                }
            }
            BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                if left_norm == Type::Number && right_norm == Type::Number {
                    Type::Number
                } else {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3002",
                            format!(
                                "Arithmetic operator requires number operands, found {} and {}",
                                left_type.display_name(),
                                right_type.display_name()
                            ),
                            binary.span,
                        )
                        .with_label("type mismatch")
                        .with_help("arithmetic operators (-, *, /, %) only work with numbers"),
                    );
                    Type::Unknown
                }
            }
            BinaryOp::Eq | BinaryOp::Ne => {
                // Equality requires same types
                if left_norm != right_norm {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3002",
                            format!(
                                "Equality comparison requires same types, found {} and {}",
                                left_type.display_name(),
                                right_type.display_name()
                            ),
                            binary.span,
                        )
                        .with_label("type mismatch")
                        .with_help("both operands must have the same type for equality comparison"),
                    );
                }
                Type::Bool
            }
            BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
                if left_norm == Type::Number && right_norm == Type::Number {
                    Type::Bool
                } else {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3002",
                            format!(
                                "Comparison requires number operands, found {} and {}",
                                left_type.display_name(),
                                right_type.display_name()
                            ),
                            binary.span,
                        )
                        .with_label("type mismatch")
                        .with_help("comparison operators (<, <=, >, >=) only work with numbers"),
                    );
                    Type::Bool // Still return bool for error recovery
                }
            }
            BinaryOp::And | BinaryOp::Or => {
                if left_norm != Type::Bool || right_norm != Type::Bool {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3002",
                            format!(
                                "Logical operators require bool operands, found {} and {}",
                                left_type.display_name(),
                                right_type.display_name()
                            ),
                            binary.span,
                        )
                        .with_label("type mismatch")
                        .with_help("logical operators (and, or) only work with bool values"),
                    );
                }
                Type::Bool
            }
        }
    }

    /// Check a unary expression
    fn check_unary(&mut self, unary: &UnaryExpr) -> Type {
        let expr_type = self.check_expr(&unary.expr);
        let expr_norm = expr_type.normalized();

        match unary.op {
            UnaryOp::Negate => {
                if expr_norm != Type::Number && expr_norm != Type::Unknown {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3002",
                            format!(
                                "Unary '-' requires number operand, found {}",
                                expr_type.display_name()
                            ),
                            unary.span,
                        )
                        .with_label("type mismatch")
                        .with_help("negation (-) only works with numbers"),
                    );
                    Type::Unknown
                } else {
                    Type::Number
                }
            }
            UnaryOp::Not => {
                if expr_norm != Type::Bool && expr_norm != Type::Unknown {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3002",
                            format!(
                                "Unary '!' requires bool operand, found {}",
                                expr_type.display_name()
                            ),
                            unary.span,
                        )
                        .with_label("type mismatch")
                        .with_help("logical not (!) only works with bool values"),
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
        let callee_norm = callee_type.normalized();

        match &callee_norm {
            Type::Function {
                type_params,
                params,
                return_type,
            } => {
                // Check argument count
                if call.args.len() != params.len() {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3005",
                            format!(
                                "Function expects {} argument{}, found {}",
                                params.len(),
                                if params.len() == 1 { "" } else { "s" },
                                call.args.len()
                            ),
                            call.span,
                        )
                        .with_label("argument count mismatch")
                        .with_help(suggestions::suggest_arity_fix(
                            params.len(),
                            call.args.len(),
                            &callee_type,
                        )),
                    );
                }

                // If function has type parameters, use type inference
                if !type_params.is_empty() {
                    return self.check_call_with_inference(type_params, params, return_type, call);
                }

                // Non-generic function - check argument types normally
                for (i, arg) in call.args.iter().enumerate() {
                    let arg_type = self.check_expr(arg);
                    if let Some(expected_type) = params.get(i) {
                        if !arg_type.is_assignable_to(expected_type)
                            && arg_type.normalized() != Type::Unknown
                        {
                            let help = suggestions::suggest_type_mismatch(expected_type, &arg_type)
                                .unwrap_or_else(|| {
                                    format!(
                                        "argument {} must be of type {}",
                                        i + 1,
                                        expected_type.display_name()
                                    )
                                });
                            self.diagnostics.push(
                                Diagnostic::error_with_code(
                                    "AT3001",
                                    format!(
                                        "Argument {} type mismatch: expected {}, found {}",
                                        i + 1,
                                        expected_type.display_name(),
                                        arg_type.display_name()
                                    ),
                                    arg.span(),
                                )
                                .with_label(format!(
                                    "expected {}, found {}",
                                    expected_type.display_name(),
                                    arg_type.display_name()
                                ))
                                .with_help(help),
                            );
                        }
                    }
                }

                (**return_type).clone()
            }
            Type::Unknown => {
                // Error recovery: still check arguments for side effects (usage tracking)
                // This ensures parameters referenced in arguments are marked as used
                for arg in &call.args {
                    self.check_expr(arg);
                }
                Type::Unknown
            }
            _ => {
                self.diagnostics.push(
                    Diagnostic::error_with_code(
                        "AT3006",
                        format!(
                            "Cannot call non-function type {}",
                            callee_type.display_name()
                        ),
                        call.span,
                    )
                    .with_label("not callable")
                    .with_help(suggestions::suggest_not_callable(&callee_type)),
                );
                Type::Unknown
            }
        }
    }

    /// Check a generic function call with type inference
    fn check_call_with_inference(
        &mut self,
        type_params: &[String],
        params: &[Type],
        return_type: &Type,
        call: &CallExpr,
    ) -> Type {
        use crate::typechecker::generics::TypeInferer;

        let mut inferer = TypeInferer::new();

        // Check each argument and try to infer type parameters
        for (i, arg) in call.args.iter().enumerate() {
            let arg_type = self.check_expr(arg);

            if let Some(param_type) = params.get(i) {
                // Try to unify parameter type with argument type
                if let Err(e) = inferer.unify(param_type, &arg_type) {
                    // Inference failed - report error
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3001",
                            format!(
                                "Type inference failed: cannot match argument {} of type {} with parameter of type {}",
                                i + 1,
                                arg_type.display_name(),
                                param_type.display_name()
                            ),
                            arg.span(),
                        )
                        .with_label("type mismatch")
                        .with_help(format!("Inference error: {:?}", e)),
                    );
                    return Type::Unknown;
                }
            }
        }

        // Check if all type parameters were inferred
        if !inferer.all_inferred(type_params) {
            // Some type parameters couldn't be inferred
            let uninferred: Vec<String> = type_params
                .iter()
                .filter(|param| inferer.get_substitution(param).is_none())
                .cloned()
                .collect();

            self.diagnostics.push(
                Diagnostic::error(
                    format!("Cannot infer type parameter(s): {}", uninferred.join(", ")),
                    call.span,
                )
                .with_label("type inference failed")
                .with_help("Try providing explicit type arguments".to_string()),
            );
            return Type::Unknown;
        }

        // Apply substitutions to return type
        inferer.apply_substitutions(return_type)
    }

    /// Check a member expression (method call)
    fn check_member(&mut self, member: &MemberExpr) -> Type {
        // Type-check the target expression
        let target_type = self.check_expr(&member.target);

        // Skip error recovery cases
        if target_type.normalized() == Type::Unknown {
            return Type::Unknown;
        }

        // Look up the method in the method table and clone the signature to avoid borrow issues
        let method_name = &member.member.name;
        let method_sig = self.method_table.lookup(&target_type, method_name).cloned();

        if let Some(method_sig) = method_sig {
            // Check argument count
            let provided_args = member.args.as_ref().map(|args| args.len()).unwrap_or(0);
            let expected_args = method_sig.arg_types.len();

            if provided_args != expected_args {
                self.diagnostics.push(
                    Diagnostic::error_with_code(
                        "AT3005",
                        format!(
                            "Method '{}' expects {} arguments, found {}",
                            method_name, expected_args, provided_args
                        ),
                        member.span,
                    )
                    .with_label("argument count mismatch")
                    .with_help(format!(
                        "method '{}' requires exactly {} argument{}",
                        method_name,
                        expected_args,
                        if expected_args == 1 { "" } else { "s" }
                    )),
                );
            }

            // Check argument types if present
            if let Some(args) = &member.args {
                for (i, arg) in args.iter().enumerate() {
                    let arg_type = self.check_expr(arg);
                    if let Some(expected_type) = method_sig.arg_types.get(i) {
                        if !arg_type.is_assignable_to(expected_type)
                            && arg_type.normalized() != Type::Unknown
                        {
                            self.diagnostics.push(
                                Diagnostic::error_with_code(
                                    "AT3001",
                                    format!(
                                        "Argument {} has wrong type: expected {}, found {}",
                                        i + 1,
                                        expected_type.display_name(),
                                        arg_type.display_name()
                                    ),
                                    arg.span(),
                                )
                                .with_label("type mismatch")
                                .with_help(format!(
                                    "argument {} must be of type {}",
                                    i + 1,
                                    expected_type.display_name()
                                )),
                            );
                        }
                    }
                }
            }

            // Return the method's return type
            method_sig.return_type
        } else {
            // Method not found for this type
            self.diagnostics.push(
                Diagnostic::error_with_code(
                    "AT3010",
                    format!(
                        "Type '{}' has no method named '{}'",
                        target_type.display_name(),
                        method_name
                    ),
                    member.member.span,
                )
                .with_label("method not found")
                .with_help(format!(
                    "type '{}' does not support method '{}'",
                    target_type.display_name(),
                    method_name
                )),
            );
            Type::Unknown
        }
    }

    /// Check an index expression
    fn check_index(&mut self, index: &IndexExpr) -> Type {
        let target_type = self.check_expr(&index.target);
        let index_type = self.check_expr(&index.index);
        let target_norm = target_type.normalized();
        let index_norm = index_type.normalized();

        match target_norm {
            // Array indexing: requires number index, returns element type
            Type::Array(elem_type) => {
                if index_norm != Type::Number && index_norm != Type::Unknown {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3001",
                            format!(
                                "Array index must be number, found {}",
                                index_type.display_name()
                            ),
                            index.index.span(),
                        )
                        .with_label("type mismatch")
                        .with_help("array indices must be numbers"),
                    );
                }
                *elem_type
            }
            // JSON indexing: accepts string or number, always returns json
            Type::JsonValue => {
                if index_norm != Type::String
                    && index_norm != Type::Number
                    && index_norm != Type::Unknown
                {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3001",
                            format!(
                                "JSON index must be string or number, found {}",
                                index_type.display_name()
                            ),
                            index.index.span(),
                        )
                        .with_label("type mismatch")
                        .with_help("use a string key or numeric index to access JSON values"),
                    );
                }
                Type::JsonValue
            }
            Type::Unknown => Type::Unknown,
            _ => {
                self.diagnostics.push(
                    Diagnostic::error_with_code(
                        "AT3001",
                        format!("Cannot index into type {}", target_type.display_name()),
                        index.target.span(),
                    )
                    .with_label("not indexable")
                    .with_help("only arrays and json values can be indexed"),
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
            if !elem_type.is_assignable_to(&first_type) && elem_type.normalized() != Type::Unknown {
                self.diagnostics.push(
                    Diagnostic::error_with_code(
                        "AT3001",
                        format!(
                            "Array element {} has wrong type: expected {}, found {}",
                            i,
                            first_type.display_name(),
                            elem_type.display_name()
                        ),
                        elem.span(),
                    )
                    .with_label("type mismatch")
                    .with_help(format!(
                        "all array elements must be type {} (inferred from first element)",
                        first_type.display_name()
                    )),
                );
            }
        }

        Type::Array(Box::new(first_type))
    }

    /// Check a match expression
    fn check_match(&mut self, match_expr: &crate::ast::MatchExpr) -> Type {
        // 1. Check scrutinee type
        let scrutinee_type = self.check_expr(&match_expr.scrutinee);

        if scrutinee_type.normalized() == Type::Unknown {
            // Error in scrutinee, skip match checking
            return Type::Unknown;
        }

        // 2. Check each arm and collect result types
        let mut arm_types = Vec::new();

        for (arm_idx, arm) in match_expr.arms.iter().enumerate() {
            // Check pattern against scrutinee type
            let pattern_bindings = self.check_pattern(&arm.pattern, &scrutinee_type);

            // Enter a new scope for pattern bindings
            self.symbol_table.enter_scope();

            // Add pattern bindings to symbol table for this arm's scope
            for (var_name, var_type, var_span) in &pattern_bindings {
                let symbol = crate::symbol::Symbol {
                    name: var_name.clone(),
                    ty: var_type.clone(),
                    mutable: false, // Pattern bindings are immutable
                    kind: crate::symbol::SymbolKind::Variable,
                    span: *var_span,
                    exported: false,
                };
                // Ignore if binding fails (duplicate names in pattern - will be caught separately)
                let _ = self.symbol_table.define(symbol);
            }

            // Check arm body with bindings in scope
            let arm_type = self.check_expr(&arm.body);
            arm_types.push((arm_type.clone(), arm.body.span(), arm_idx));

            // Exit scope (removes pattern bindings)
            self.symbol_table.exit_scope();
        }

        // 3. Ensure all arms return compatible types
        if arm_types.is_empty() {
            // Empty match (parser should prevent this, but handle gracefully)
            self.diagnostics.push(
                Diagnostic::error_with_code(
                    "AT3020",
                    "Match expression must have at least one arm",
                    match_expr.span,
                )
                .with_label("empty match")
                .with_help("add at least one match arm with a pattern and expression"),
            );
            return Type::Unknown;
        }

        // Get the first arm's type as the expected type
        let (first_type, _, _) = &arm_types[0];

        // Check that all other arms have compatible types
        for (arm_type, arm_span, arm_idx) in &arm_types[1..] {
            if !arm_type.is_assignable_to(first_type) && arm_type.normalized() != Type::Unknown {
                self.diagnostics.push(
                    Diagnostic::error_with_code(
                        "AT3021",
                        format!(
                            "Match arm {} returns incompatible type: expected {}, found {}",
                            arm_idx + 1,
                            first_type.display_name(),
                            arm_type.display_name()
                        ),
                        *arm_span,
                    )
                    .with_label("type mismatch")
                    .with_help(format!(
                        "all match arms must return the same type ({})",
                        first_type.display_name()
                    )),
                );
            }
        }

        // 4. Check exhaustiveness
        self.check_exhaustiveness(&match_expr.arms, &scrutinee_type, match_expr.span);

        // 5. Return the unified type (first arm's type)
        first_type.clone()
    }

    /// Check exhaustiveness of match arms
    fn check_exhaustiveness(
        &mut self,
        arms: &[crate::ast::MatchArm],
        scrutinee_type: &Type,
        match_span: Span,
    ) {
        use crate::ast::Pattern;

        // Check if there's a catch-all pattern (wildcard or variable binding)
        let has_catch_all = arms
            .iter()
            .any(|arm| matches!(arm.pattern, Pattern::Wildcard(_) | Pattern::Variable(_)));

        if has_catch_all {
            // Wildcard or variable catches everything - exhaustive
            return;
        }

        // Check exhaustiveness based on scrutinee type
        let scrutinee_norm = scrutinee_type.normalized();
        match scrutinee_norm {
            Type::Generic { name, .. } if name == "Option" => {
                // Option<T> requires Some and None to be covered
                let has_some = arms.iter().any(|arm| {
                    if let Pattern::Constructor {
                        name: ctor_name, ..
                    } = &arm.pattern
                    {
                        ctor_name.name == "Some"
                    } else {
                        false
                    }
                });

                let has_none = arms.iter().any(|arm| {
                    if let Pattern::Constructor {
                        name: ctor_name, ..
                    } = &arm.pattern
                    {
                        ctor_name.name == "None"
                    } else {
                        false
                    }
                });

                if !has_some || !has_none {
                    let missing = if !has_some && !has_none {
                        "Some(_), None".to_string()
                    } else if !has_some {
                        "Some(_)".to_string()
                    } else {
                        "None".to_string()
                    };

                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3027",
                            format!("Non-exhaustive match on Option: missing {}", missing),
                            match_span,
                        )
                        .with_label("non-exhaustive")
                        .with_help(format!("Add arm: {} => ...", missing)),
                    );
                }
            }

            Type::Generic { name, .. } if name == "Result" => {
                // Result<T,E> requires Ok and Err to be covered
                let has_ok = arms.iter().any(|arm| {
                    if let Pattern::Constructor {
                        name: ctor_name, ..
                    } = &arm.pattern
                    {
                        ctor_name.name == "Ok"
                    } else {
                        false
                    }
                });

                let has_err = arms.iter().any(|arm| {
                    if let Pattern::Constructor {
                        name: ctor_name, ..
                    } = &arm.pattern
                    {
                        ctor_name.name == "Err"
                    } else {
                        false
                    }
                });

                if !has_ok || !has_err {
                    let missing = if !has_ok && !has_err {
                        "Ok(_), Err(_)".to_string()
                    } else if !has_ok {
                        "Ok(_)".to_string()
                    } else {
                        "Err(_)".to_string()
                    };

                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3027",
                            format!("Non-exhaustive match on Result: missing {}", missing),
                            match_span,
                        )
                        .with_label("non-exhaustive")
                        .with_help(format!("Add arm: {} => ...", missing)),
                    );
                }
            }

            Type::Bool => {
                // Bool requires true and false to be covered (or wildcard)
                let has_true = arms
                    .iter()
                    .any(|arm| matches!(arm.pattern, Pattern::Literal(Literal::Bool(true), _)));

                let has_false = arms
                    .iter()
                    .any(|arm| matches!(arm.pattern, Pattern::Literal(Literal::Bool(false), _)));

                if !has_true || !has_false {
                    let missing = if !has_true && !has_false {
                        "true, false".to_string()
                    } else if !has_true {
                        "true".to_string()
                    } else {
                        "false".to_string()
                    };

                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3027",
                            format!("Non-exhaustive match on bool: missing {}", missing),
                            match_span,
                        )
                        .with_label("non-exhaustive")
                        .with_help(format!("Add arm: {} => ... or use wildcard _", missing)),
                    );
                }
            }

            Type::Number | Type::String | Type::Array(_) | Type::Null => {
                // These types have infinite values - require wildcard
                self.diagnostics.push(
                    Diagnostic::error_with_code(
                        "AT3027",
                        format!(
                            "Non-exhaustive match on {}: patterns must cover all possible values",
                            scrutinee_type.display_name()
                        ),
                        match_span,
                    )
                    .with_label("non-exhaustive")
                    .with_help("Add wildcard pattern: _ => ..."),
                );
            }

            _ => {
                // For other types, warn but don't error (conservative approach)
            }
        }
    }

    /// Check a pattern and return variable bindings (name, type, span)
    fn check_pattern(
        &mut self,
        pattern: &Pattern,
        expected_type: &Type,
    ) -> Vec<(String, Type, Span)> {
        let mut bindings = Vec::new();

        match pattern {
            Pattern::Literal(lit, span) => {
                // Check literal type matches expected type
                let lit_type = match lit {
                    Literal::Number(_) => Type::Number,
                    Literal::String(_) => Type::String,
                    Literal::Bool(_) => Type::Bool,
                    Literal::Null => Type::Null,
                };

                if !lit_type.is_assignable_to(expected_type) {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3022",
                            format!(
                                "Pattern type mismatch: expected {}, found {}",
                                expected_type.display_name(),
                                lit_type.display_name()
                            ),
                            *span,
                        )
                        .with_label("type mismatch")
                        .with_help(format!(
                            "use a {} literal or wildcard pattern",
                            expected_type.display_name()
                        )),
                    );
                }
            }

            Pattern::Wildcard(_) => {
                // Wildcard matches anything, no bindings
            }

            Pattern::Variable(id) => {
                // Variable binding - binds the entire scrutinee value
                bindings.push((id.name.clone(), expected_type.clone(), id.span));
            }

            Pattern::Constructor { name, args, span } => {
                // Check constructor pattern (Ok, Err, Some, None)
                bindings.extend(self.check_constructor_pattern(name, args, expected_type, *span));
            }

            Pattern::Array { elements, span } => {
                // Check array pattern
                bindings.extend(self.check_array_pattern(elements, expected_type, *span));
            }
        }

        bindings
    }

    /// Check constructor pattern (Ok, Err, Some, None)
    fn check_constructor_pattern(
        &mut self,
        name: &Identifier,
        args: &[Pattern],
        expected_type: &Type,
        span: Span,
    ) -> Vec<(String, Type, Span)> {
        let mut bindings = Vec::new();
        let expected_norm = expected_type.normalized();

        match expected_norm {
            Type::Generic {
                name: type_name,
                type_args,
            } => {
                match type_name.as_str() {
                    "Option" if type_args.len() == 1 => {
                        // Option<T> has constructors: Some(T), None
                        match name.name.as_str() {
                            "Some" => {
                                if args.len() != 1 {
                                    self.diagnostics.push(
                                        Diagnostic::error_with_code(
                                            "AT3023",
                                            format!(
                                                "Some expects 1 argument, found {}",
                                                args.len()
                                            ),
                                            span,
                                        )
                                        .with_label("wrong arity")
                                        .with_help("Some requires exactly 1 argument: Some(value)"),
                                    );
                                } else {
                                    // Check inner pattern against T
                                    bindings.extend(self.check_pattern(&args[0], &type_args[0]));
                                }
                            }
                            "None" => {
                                if !args.is_empty() {
                                    self.diagnostics.push(
                                        Diagnostic::error_with_code(
                                            "AT3023",
                                            format!(
                                                "None expects 0 arguments, found {}",
                                                args.len()
                                            ),
                                            span,
                                        )
                                        .with_label("wrong arity")
                                        .with_help("None requires no arguments: None"),
                                    );
                                }
                            }
                            _ => {
                                self.diagnostics.push(
                                    Diagnostic::error_with_code(
                                        "AT3024",
                                        format!("Unknown Option constructor: {}", name.name),
                                        name.span,
                                    )
                                    .with_label("unknown constructor")
                                    .with_help(
                                        "Option only has constructors: Some(value) and None",
                                    ),
                                );
                            }
                        }
                    }
                    "Result" if type_args.len() == 2 => {
                        // Result<T, E> has constructors: Ok(T), Err(E)
                        match name.name.as_str() {
                            "Ok" => {
                                if args.len() != 1 {
                                    self.diagnostics.push(
                                        Diagnostic::error_with_code(
                                            "AT3023",
                                            format!("Ok expects 1 argument, found {}", args.len()),
                                            span,
                                        )
                                        .with_label("wrong arity")
                                        .with_help("Ok requires exactly 1 argument: Ok(value)"),
                                    );
                                } else {
                                    // Check inner pattern against T
                                    bindings.extend(self.check_pattern(&args[0], &type_args[0]));
                                }
                            }
                            "Err" => {
                                if args.len() != 1 {
                                    self.diagnostics.push(
                                        Diagnostic::error_with_code(
                                            "AT3023",
                                            format!("Err expects 1 argument, found {}", args.len()),
                                            span,
                                        )
                                        .with_label("wrong arity")
                                        .with_help("Err requires exactly 1 argument: Err(error)"),
                                    );
                                } else {
                                    // Check inner pattern against E
                                    bindings.extend(self.check_pattern(&args[0], &type_args[1]));
                                }
                            }
                            _ => {
                                self.diagnostics.push(
                                    Diagnostic::error_with_code(
                                        "AT3024",
                                        format!("Unknown Result constructor: {}", name.name),
                                        name.span,
                                    )
                                    .with_label("unknown constructor")
                                    .with_help(
                                        "Result only has constructors: Ok(value) and Err(error)",
                                    ),
                                );
                            }
                        }
                    }
                    _ => {
                        self.diagnostics.push(
                            Diagnostic::error_with_code(
                                "AT3025",
                                format!(
                                    "Constructor patterns not supported for type {}",
                                    expected_type.display_name()
                                ),
                                span,
                            )
                            .with_label("unsupported type")
                            .with_help(
                                "constructor patterns only work with Option and Result types",
                            ),
                        );
                    }
                }
            }
            _ => {
                self.diagnostics.push(
                    Diagnostic::error_with_code(
                        "AT3025",
                        format!(
                            "Constructor patterns not supported for type {}",
                            expected_type.display_name()
                        ),
                        span,
                    )
                    .with_label("unsupported type"),
                );
            }
        }

        bindings
    }

    /// Check array pattern
    fn check_array_pattern(
        &mut self,
        elements: &[Pattern],
        expected_type: &Type,
        span: Span,
    ) -> Vec<(String, Type, Span)> {
        let mut bindings = Vec::new();
        let expected_norm = expected_type.normalized();

        match expected_norm {
            Type::Array(elem_type) => {
                // Check each pattern element against the array element type
                for pattern in elements {
                    bindings.extend(self.check_pattern(pattern, &elem_type));
                }
            }
            _ => {
                self.diagnostics.push(
                    Diagnostic::error_with_code(
                        "AT3026",
                        format!(
                            "Array pattern used on non-array type: {}",
                            expected_type.display_name()
                        ),
                        span,
                    )
                    .with_label("type mismatch")
                    .with_help("array patterns can only match array types"),
                );
            }
        }

        bindings
    }

    /// Check try expression (error propagation operator ?)
    fn check_try(&mut self, try_expr: &TryExpr) -> Type {
        // Type check the expression being tried
        let expr_type = self.check_expr(&try_expr.expr);
        let expr_norm = expr_type.normalized();

        // Skip if expression type is unknown (error already reported)
        if expr_norm == Type::Unknown {
            return Type::Unknown;
        }

        // Expression must be a Result<T, E>
        let (ok_type, err_type) = match &expr_norm {
            Type::Generic { name, type_args } if name == "Result" && type_args.len() == 2 => {
                (type_args[0].clone(), type_args[1].clone())
            }
            _ => {
                self.diagnostics.push(
                    Diagnostic::error_with_code(
                        "AT3027",
                        format!(
                            "? operator requires Result<T, E> type, found {}",
                            expr_type.display_name()
                        ),
                        try_expr.span,
                    )
                    .with_label("not a Result type")
                    .with_help("the ? operator can only be applied to Result<T, E> values"),
                );
                return Type::Unknown;
            }
        };

        // Must be inside a function that returns Result<T', E'>
        let function_return_type = match &self.current_function_return_type {
            Some(ty) => ty.clone(),
            None => {
                self.diagnostics.push(
                    Diagnostic::error_with_code(
                        "AT3028",
                        "? operator can only be used inside functions",
                        try_expr.span,
                    )
                    .with_label("not in a function")
                    .with_help("? operator propagates errors by early return"),
                );
                return Type::Unknown;
            }
        };

        // Function must return Result<T', E'>
        let function_return_norm = function_return_type.normalized();
        match &function_return_norm {
            Type::Generic { name, type_args } if name == "Result" && type_args.len() == 2 => {
                let function_err_type = &type_args[1];

                // Error types must be compatible (for now, they must be the same)
                if err_type.normalized() != function_err_type.normalized() {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3029",
                            format!(
                                "? operator error type mismatch: expression has error type {}, but function returns {}",
                                err_type.display_name(),
                                function_err_type.display_name()
                            ),
                            try_expr.span,
                        )
                        .with_label("error type mismatch")
                        .with_help(format!(
                            "convert the error type to {} or change the function's error type",
                            function_err_type.display_name()
                        )),
                    );
                }

                // Return the Ok type (T)
                ok_type
            }
            _ => {
                self.diagnostics.push(
                    Diagnostic::error_with_code(
                        "AT3030",
                        format!(
                            "? operator requires function to return Result<T, E>, found {}",
                            function_return_type.display_name()
                        ),
                        try_expr.span,
                    )
                    .with_label("function does not return Result")
                    .with_help(
                        "change the function's return type to Result<T, E> to use ? operator",
                    ),
                );
                Type::Unknown
            }
        }
    }
}
