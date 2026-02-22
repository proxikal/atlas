//! Expression evaluation

use crate::ast::*;
use crate::interpreter::{ControlFlow, Interpreter, UserFunction};
use crate::value::{RuntimeError, Value};
use std::sync::Arc;

impl Interpreter {
    /// Evaluate an expression
    pub(super) fn eval_expr(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Literal(lit, _) => Ok(self.eval_literal(lit)),
            Expr::Identifier(id) => self.get_variable(&id.name, id.span),
            Expr::Binary(binary) => self.eval_binary(binary),
            Expr::Unary(unary) => self.eval_unary(unary),
            Expr::Call(call) => self.eval_call(call),
            Expr::Index(index) => self.eval_index(index),
            Expr::ArrayLiteral(arr) => self.eval_array_literal(arr),
            Expr::Group(group) => self.eval_expr(&group.expr),
            Expr::Match(match_expr) => self.eval_match(match_expr),
            Expr::Member(member) => self.eval_member(member),
            Expr::Try(try_expr) => self.eval_try(try_expr),
        }
    }

    /// Evaluate a literal
    pub(super) fn eval_literal(&self, lit: &Literal) -> Value {
        match lit {
            Literal::Number(n) => Value::Number(*n),
            Literal::String(s) => Value::string(s.clone()),
            Literal::Bool(b) => Value::Bool(*b),
            Literal::Null => Value::Null,
        }
    }

    /// Evaluate a binary expression
    fn eval_binary(&mut self, binary: &BinaryExpr) -> Result<Value, RuntimeError> {
        // Short-circuit evaluation for && and ||
        if binary.op == BinaryOp::And {
            let left = self.eval_expr(&binary.left)?;
            if let Value::Bool(false) = left {
                return Ok(Value::Bool(false));
            }
            if let Value::Bool(true) = left {
                let right = self.eval_expr(&binary.right)?;
                if let Value::Bool(b) = right {
                    return Ok(Value::Bool(b));
                }
            }
            return Err(RuntimeError::TypeError {
                msg: "Expected bool for &&".to_string(),
                span: binary.span,
            });
        }

        if binary.op == BinaryOp::Or {
            let left = self.eval_expr(&binary.left)?;
            if let Value::Bool(true) = left {
                return Ok(Value::Bool(true));
            }
            if let Value::Bool(false) = left {
                let right = self.eval_expr(&binary.right)?;
                if let Value::Bool(b) = right {
                    return Ok(Value::Bool(b));
                }
            }
            return Err(RuntimeError::TypeError {
                msg: "Expected bool for ||".to_string(),
                span: binary.span,
            });
        }

        // Regular binary operations
        let left = self.eval_expr(&binary.left)?;
        let right = self.eval_expr(&binary.right)?;

        match binary.op {
            BinaryOp::Add => match (&left, &right) {
                (Value::Number(a), Value::Number(b)) => {
                    let result = a + b;
                    if result.is_nan() || result.is_infinite() {
                        return Err(RuntimeError::InvalidNumericResult { span: binary.span });
                    }
                    Ok(Value::Number(result))
                }
                (Value::String(a), Value::String(b)) => Ok(Value::string(format!("{}{}", a, b))),
                _ => Err(RuntimeError::TypeError {
                    msg: "Invalid operands for +".to_string(),
                    span: binary.span,
                }),
            },
            BinaryOp::Sub => self.numeric_binary_op(left, right, |a, b| a - b, binary.span),
            BinaryOp::Mul => self.numeric_binary_op(left, right, |a, b| a * b, binary.span),
            BinaryOp::Div => {
                if let (Value::Number(a), Value::Number(b)) = (&left, &right) {
                    if *b == 0.0 {
                        return Err(RuntimeError::DivideByZero { span: binary.span });
                    }
                    let result = a / b;
                    if result.is_nan() || result.is_infinite() {
                        return Err(RuntimeError::InvalidNumericResult { span: binary.span });
                    }
                    Ok(Value::Number(result))
                } else {
                    Err(RuntimeError::TypeError {
                        msg: "Expected numbers for /".to_string(),
                        span: binary.span,
                    })
                }
            }
            BinaryOp::Mod => {
                if let (Value::Number(a), Value::Number(b)) = (&left, &right) {
                    if *b == 0.0 {
                        return Err(RuntimeError::DivideByZero { span: binary.span });
                    }
                    let result = a % b;
                    if result.is_nan() || result.is_infinite() {
                        return Err(RuntimeError::InvalidNumericResult { span: binary.span });
                    }
                    Ok(Value::Number(result))
                } else {
                    Err(RuntimeError::TypeError {
                        msg: "Expected numbers for %".to_string(),
                        span: binary.span,
                    })
                }
            }
            BinaryOp::Eq => Ok(Value::Bool(left == right)),
            BinaryOp::Ne => Ok(Value::Bool(left != right)),
            BinaryOp::Lt => self.numeric_comparison(left, right, |a, b| a < b, binary.span),
            BinaryOp::Le => self.numeric_comparison(left, right, |a, b| a <= b, binary.span),
            BinaryOp::Gt => self.numeric_comparison(left, right, |a, b| a > b, binary.span),
            BinaryOp::Ge => self.numeric_comparison(left, right, |a, b| a >= b, binary.span),
            BinaryOp::And | BinaryOp::Or => {
                // Already handled above
                unreachable!()
            }
        }
    }

    /// Helper for numeric binary operations
    fn numeric_binary_op<F>(
        &self,
        left: Value,
        right: Value,
        op: F,
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError>
    where
        F: FnOnce(f64, f64) -> f64,
    {
        if let (Value::Number(a), Value::Number(b)) = (left, right) {
            let result = op(a, b);
            if result.is_nan() || result.is_infinite() {
                return Err(RuntimeError::InvalidNumericResult { span });
            }
            Ok(Value::Number(result))
        } else {
            Err(RuntimeError::TypeError {
                msg: "Expected numbers".to_string(),
                span,
            })
        }
    }

    /// Helper for numeric comparisons
    fn numeric_comparison<F>(
        &self,
        left: Value,
        right: Value,
        op: F,
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError>
    where
        F: FnOnce(f64, f64) -> bool,
    {
        if let (Value::Number(a), Value::Number(b)) = (left, right) {
            Ok(Value::Bool(op(a, b)))
        } else {
            Err(RuntimeError::TypeError {
                msg: "Expected numbers for comparison".to_string(),
                span,
            })
        }
    }

    /// Evaluate a unary expression
    fn eval_unary(&mut self, unary: &UnaryExpr) -> Result<Value, RuntimeError> {
        let operand = self.eval_expr(&unary.expr)?;

        match unary.op {
            UnaryOp::Negate => {
                if let Value::Number(n) = operand {
                    Ok(Value::Number(-n))
                } else {
                    Err(RuntimeError::TypeError {
                        msg: "Expected number for -".to_string(),
                        span: unary.span,
                    })
                }
            }
            UnaryOp::Not => {
                if let Value::Bool(b) = operand {
                    Ok(Value::Bool(!b))
                } else {
                    Err(RuntimeError::TypeError {
                        msg: "Expected bool for !".to_string(),
                        span: unary.span,
                    })
                }
            }
        }
    }

    /// Evaluate a function call
    pub(super) fn eval_call(&mut self, call: &CallExpr) -> Result<Value, RuntimeError> {
        // Evaluate callee as ANY expression (enables first-class functions)
        let callee_value = self.eval_expr(&call.callee)?;

        // Check for early return from callee evaluation
        if self.control_flow != ControlFlow::None {
            // Propagate control flow (e.g., from ? operator)
            return Ok(match &self.control_flow {
                ControlFlow::Return(v) => v.clone(),
                _ => Value::Null,
            });
        }

        // Evaluate arguments, checking for control flow after each
        let mut args = Vec::new();
        for arg in &call.args {
            let val = self.eval_expr(arg)?;

            // Check for early return from argument evaluation (e.g., ? operator)
            if self.control_flow != ControlFlow::None {
                return Ok(match &self.control_flow {
                    ControlFlow::Return(v) => v.clone(),
                    _ => Value::Null,
                });
            }

            args.push(val);
        }

        // Callee must be a function value
        match callee_value {
            Value::Builtin(ref name) => {
                // Check for array intrinsics (callback-based functions)
                match name.as_ref() {
                    "map" => return self.intrinsic_map(&args, call.span),
                    "filter" => return self.intrinsic_filter(&args, call.span),
                    "reduce" => return self.intrinsic_reduce(&args, call.span),
                    "forEach" => return self.intrinsic_for_each(&args, call.span),
                    "find" => return self.intrinsic_find(&args, call.span),
                    "findIndex" => return self.intrinsic_find_index(&args, call.span),
                    "flatMap" => return self.intrinsic_flat_map(&args, call.span),
                    "some" => return self.intrinsic_some(&args, call.span),
                    "every" => return self.intrinsic_every(&args, call.span),
                    "sort" => return self.intrinsic_sort(&args, call.span),
                    "sortBy" => return self.intrinsic_sort_by(&args, call.span),
                    "result_map" => return self.intrinsic_result_map(&args, call.span),
                    "result_map_err" => return self.intrinsic_result_map_err(&args, call.span),
                    "result_and_then" => return self.intrinsic_result_and_then(&args, call.span),
                    "result_or_else" => return self.intrinsic_result_or_else(&args, call.span),
                    "hashMapForEach" => return self.intrinsic_hashmap_for_each(&args, call.span),
                    "hashMapMap" => return self.intrinsic_hashmap_map(&args, call.span),
                    "hashMapFilter" => return self.intrinsic_hashmap_filter(&args, call.span),
                    "hashSetForEach" => return self.intrinsic_hashset_for_each(&args, call.span),
                    "hashSetMap" => return self.intrinsic_hashset_map(&args, call.span),
                    "hashSetFilter" => return self.intrinsic_hashset_filter(&args, call.span),
                    "regexReplaceWith" => {
                        return self.intrinsic_regex_replace_with(&args, call.span)
                    }
                    "regexReplaceAllWith" => {
                        return self.intrinsic_regex_replace_all_with(&args, call.span)
                    }
                    _ => {}
                }

                // Stdlib builtin dispatch
                let security = self
                    .current_security
                    .as_ref()
                    .expect("Security context not set");
                let result = crate::stdlib::call_builtin(
                    name,
                    &args,
                    call.span,
                    security,
                    &self.output_writer,
                )?;
                // CoW write-back: collection mutation builtins return the new collection
                // but the caller's variable still holds the old value. Write it back.
                self.apply_cow_writeback(name, result, &call.args, call.span)
            }
            Value::Function(func_ref) => {
                // Extern function - check if it's an FFI function
                if let Some(extern_fn) = self.extern_functions.get(&func_ref.name) {
                    // Call the extern function using FFI
                    return unsafe { extern_fn.call(&args) }.map_err(|e| RuntimeError::TypeError {
                        msg: format!("FFI call error: {}", e),
                        span: call.span,
                    });
                }

                // User-defined function - look up body
                if let Some(func) = self.function_bodies.get(&func_ref.name).cloned() {
                    // In debug mode, mark caller bindings consumed for `own` parameters.
                    // Only applies when the argument is a direct variable reference —
                    // literals and expression results have no binding to consume.
                    #[cfg(debug_assertions)]
                    for (param, arg_expr) in func.params.iter().zip(call.args.iter()) {
                        if param.ownership == Some(crate::ast::OwnershipAnnotation::Own) {
                            if let Expr::Identifier(id) = arg_expr {
                                self.mark_consumed(&id.name);
                            }
                        }
                    }
                    return self.call_user_function(&func, args, call.span);
                }

                Err(RuntimeError::UnknownFunction {
                    name: func_ref.name.clone(),
                    span: call.span,
                })
            }
            Value::NativeFunction(native_fn) => {
                // Call the native Rust closure
                native_fn(&args)
            }
            // None() is a valid call that returns Option::None (zero-arg constructor)
            Value::Option(None) if args.is_empty() => Ok(Value::Option(None)),
            _ => Err(RuntimeError::TypeError {
                msg: format!("Cannot call non-function type {}", callee_value.type_name()),
                span: call.span,
            }),
        }
    }

    /// Evaluate a member expression (method call)
    ///
    /// Desugars method calls to stdlib function calls:
    ///   value.method(args) → Type_method(value, args)
    pub(super) fn eval_member(&mut self, member: &MemberExpr) -> Result<Value, RuntimeError> {
        // 1. Evaluate target expression
        let target_value = self.eval_expr(&member.target)?;

        // 1b. Check for trait dispatch (user-defined impl methods).
        // The typechecker annotates `trait_dispatch` when a trait method is resolved.
        if let Some((type_name, trait_name)) = member.trait_dispatch.borrow().clone() {
            let mangled_name = format!(
                "__impl__{}__{}__{}",
                type_name, trait_name, member.member.name
            );
            // Build argument list: receiver first (self), then method args
            let mut args = vec![target_value];
            if let Some(method_args) = &member.args {
                for arg in method_args {
                    args.push(self.eval_expr(arg)?);
                }
            }
            let func = self
                .function_bodies
                .get(&mangled_name)
                .cloned()
                .ok_or_else(|| RuntimeError::TypeError {
                    msg: format!(
                        "Trait method '{}' not found (impl not registered for this type)",
                        member.member.name
                    ),
                    span: member.span,
                })?;
            return self.call_user_function(&func, args, member.span);
        }

        // 2. Build desugared function name via shared dispatch table.
        // Prefer the static TypeTag set by the typechecker; fall back to dynamic dispatch
        // from the runtime value when the typechecker couldn't infer the type (e.g. `array`
        // annotation resolves to Unknown in some paths, or the typechecker is not run).
        let dynamic_tag = match &target_value {
            Value::Array(_) => Some(crate::method_dispatch::TypeTag::Array),
            _ => None,
        };
        let type_tag = member.type_tag.get().or(dynamic_tag);
        let type_tag = type_tag.ok_or_else(|| RuntimeError::TypeError {
            msg: format!(
                "Cannot call method '{}' on value of this type",
                member.member.name
            ),
            span: member.span,
        })?;
        let func_name = crate::method_dispatch::resolve_method(type_tag, &member.member.name)
            .ok_or_else(|| RuntimeError::TypeError {
                msg: format!("No method '{}' on type {:?}", member.member.name, type_tag),
                span: member.span,
            })?;

        // 3. Build argument list (target + method args)
        let mut args = vec![target_value];
        if let Some(method_args) = &member.args {
            for arg in method_args {
                args.push(self.eval_expr(arg)?);
            }
        }

        // 4. Call stdlib function
        let security = self
            .current_security
            .as_ref()
            .expect("Security context not set");
        let result = crate::stdlib::call_builtin(
            &func_name,
            &args,
            member.span,
            security,
            &self.output_writer,
        )?;

        // 5. CoW write-back: if the method mutates the receiver, update the receiver variable.
        //    Only possible when the target is a simple identifier (not a complex expression).
        if let Expr::Identifier(id) = member.target.as_ref() {
            if crate::method_dispatch::is_array_mutating_collection(&func_name) {
                // Push/unshift/reverse: result IS the new array — write it back
                self.force_set_collection(&id.name, result.clone());
                return Ok(result);
            }
            if crate::method_dispatch::is_array_mutating_pair(&func_name) {
                // Pop/shift: result is [extracted_value, new_array] — write back new_array, return extracted
                if let Value::Array(ref arr) = result {
                    let s = arr.as_slice();
                    if s.len() == 2 {
                        let extracted = s[0].clone();
                        let new_arr = s[1].clone();
                        self.force_set_collection(&id.name, new_arr);
                        return Ok(extracted);
                    }
                }
                return Ok(result);
            }
        }

        Ok(result)
    }

    /// Evaluate try expression (error propagation operator ?)
    ///
    /// Unwraps Ok value or returns Err early from current function
    pub(super) fn eval_try(&mut self, try_expr: &TryExpr) -> Result<Value, RuntimeError> {
        let value = self.eval_expr(&try_expr.expr)?;

        match value {
            Value::Result(Ok(inner)) => {
                // Unwrap Ok value
                Ok(*inner)
            }
            Value::Result(Err(err)) => {
                // Propagate error by early return
                let err_result = Value::Result(Err(err));
                self.control_flow = ControlFlow::Return(err_result.clone());
                Ok(err_result)
            }
            _ => {
                // Type checker should prevent this, but handle gracefully
                Err(RuntimeError::TypeError {
                    msg: "? operator requires Result<T, E> type".to_string(),
                    span: try_expr.span,
                })
            }
        }
    }

    /// Call a user-defined function
    fn call_user_function(
        &mut self,
        func: &UserFunction,
        args: Vec<Value>,
        call_span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        // Check arity
        if args.len() != func.params.len() {
            return Err(RuntimeError::TypeError {
                msg: format!(
                    "Function {} expects {} arguments, got {}",
                    func.name,
                    func.params.len(),
                    args.len()
                ),
                span: call_span,
            });
        }

        // Push new scope for function
        self.push_scope();

        // Bind parameters (parameters are mutable)
        for (param, arg) in func.params.iter().zip(args.iter()) {
            // Debug-mode ownership enforcement for `shared` parameters.
            #[cfg(debug_assertions)]
            {
                use crate::ast::OwnershipAnnotation;
                match &param.ownership {
                    Some(OwnershipAnnotation::Shared) => {
                        if !matches!(arg, Value::SharedValue(_)) {
                            // Must pop scope before returning — we already pushed it.
                            self.pop_scope();
                            return Err(RuntimeError::TypeError {
                                msg: format!(
                                    "ownership violation: parameter '{}' expects shared<T> but received {}",
                                    param.name.name,
                                    arg.type_name()
                                ),
                                span: call_span,
                            });
                        }
                    }
                    Some(ann @ OwnershipAnnotation::Own)
                    | Some(ann @ OwnershipAnnotation::Borrow) => {
                        if matches!(arg, Value::SharedValue(_)) {
                            let ann_str = match ann {
                                OwnershipAnnotation::Own => "own",
                                OwnershipAnnotation::Borrow => "borrow",
                                OwnershipAnnotation::Shared => unreachable!(),
                            };
                            eprintln!(
                                "warning: passing shared<T> value to '{}' parameter '{}' — consider using the 'shared' annotation",
                                ann_str, param.name.name
                            );
                        }
                    }
                    None => {}
                }
            }
            let scope = self.locals.last_mut().unwrap();
            scope.insert(param.name.name.clone(), (arg.clone(), true));
        }

        // Execute function body
        let mut result = Value::Null;
        for stmt in &func.body.statements {
            result = self.eval_statement(stmt)?;

            // Check for return
            if let ControlFlow::Return(val) = &self.control_flow {
                result = val.clone();
                self.control_flow = ControlFlow::None;
                break;
            }
        }

        self.pop_scope();
        Ok(result)
    }

    /// Evaluate array indexing
    fn eval_index(&mut self, index: &IndexExpr) -> Result<Value, RuntimeError> {
        let target = self.eval_expr(&index.target)?;
        let idx = self.eval_expr(&index.index)?;

        match target {
            Value::Array(arr) => {
                if let Value::Number(n) = idx {
                    let index_val = n as i64;
                    if n.fract() != 0.0 || n < 0.0 {
                        return Err(RuntimeError::InvalidIndex { span: index.span });
                    }

                    if index_val >= 0 && (index_val as usize) < arr.len() {
                        Ok(arr[index_val as usize].clone())
                    } else {
                        Err(RuntimeError::OutOfBounds { span: index.span })
                    }
                } else {
                    Err(RuntimeError::InvalidIndex { span: index.span })
                }
            }
            Value::String(s) => {
                if let Value::Number(n) = idx {
                    let index_val = n as i64;
                    if n.fract() != 0.0 || n < 0.0 {
                        return Err(RuntimeError::InvalidIndex { span: index.span });
                    }

                    let chars: Vec<char> = s.chars().collect();
                    if index_val >= 0 && (index_val as usize) < chars.len() {
                        Ok(Value::string(chars[index_val as usize].to_string()))
                    } else {
                        Err(RuntimeError::OutOfBounds { span: index.span })
                    }
                } else {
                    Err(RuntimeError::InvalidIndex { span: index.span })
                }
            }
            Value::JsonValue(json) => {
                // JSON indexing with string or number, returns JsonValue
                let result = match idx {
                    Value::String(key) => json.index_str(key.as_ref()),
                    Value::Number(n) => json.index_num(n),
                    _ => {
                        return Err(RuntimeError::TypeError {
                            msg: "JSON index must be string or number".to_string(),
                            span: index.span,
                        })
                    }
                };
                Ok(Value::JsonValue(Arc::new(result)))
            }
            _ => Err(RuntimeError::TypeError {
                msg: "Cannot index non-array/string/json".to_string(),
                span: index.span,
            }),
        }
    }

    /// Evaluate array literal
    fn eval_array_literal(
        &mut self,
        arr: &crate::ast::ArrayLiteral,
    ) -> Result<Value, RuntimeError> {
        let elements: Result<Vec<Value>, _> =
            arr.elements.iter().map(|e| self.eval_expr(e)).collect();
        Ok(Value::array(elements?))
    }

    /// Evaluate match expression
    fn eval_match(&mut self, match_expr: &crate::ast::MatchExpr) -> Result<Value, RuntimeError> {
        // Evaluate scrutinee
        let scrutinee = self.eval_expr(&match_expr.scrutinee)?;

        // Try each arm in order
        for arm in &match_expr.arms {
            // Try to match pattern against scrutinee
            if let Some(bindings) = self.try_match_pattern(&arm.pattern, &scrutinee) {
                // Pattern matched! Create new scope and bind variables
                self.push_scope();

                // Bind pattern variables (pattern bindings are immutable - they're destructured values)
                for (name, value) in &bindings {
                    let scope = self.locals.last_mut().unwrap();
                    scope.insert(name.clone(), (value.clone(), false));
                }

                // Check guard if present — guard failure means try next arm
                if let Some(guard_expr) = &arm.guard {
                    let guard_result = self.eval_expr(guard_expr)?;
                    if guard_result != Value::Bool(true) {
                        self.pop_scope();
                        continue; // Guard failed — try next arm
                    }
                }

                // Evaluate arm body with bindings in scope
                let result = self.eval_expr(&arm.body)?;

                // Pop scope (remove bindings)
                self.pop_scope();

                // Return result
                return Ok(result);
            }
        }

        // No pattern matched - this should be prevented by exhaustiveness checking
        // but provide a fallback error just in case
        Err(RuntimeError::TypeError {
            msg: "Non-exhaustive pattern match - no arm matched".to_string(),
            span: match_expr.span,
        })
    }

    /// Try to match a pattern against a value
    /// Returns Some(bindings) if match succeeds, None if match fails
    fn try_match_pattern(&self, pattern: &Pattern, value: &Value) -> Option<Vec<(String, Value)>> {
        match pattern {
            // Literal patterns: must match exactly
            Pattern::Literal(lit, _) => {
                let pattern_value = self.eval_literal(lit);
                if self.values_equal(&pattern_value, value) {
                    Some(Vec::new()) // Match, no bindings
                } else {
                    None // No match
                }
            }

            // Wildcard: matches anything, no bindings
            Pattern::Wildcard(_) => Some(Vec::new()),

            // Variable: matches anything, binds to name
            Pattern::Variable(id) => Some(vec![(id.name.clone(), value.clone())]),

            // Constructor patterns: Some(x), None, Ok(x), Err(e)
            Pattern::Constructor { name, args, .. } => {
                self.try_match_constructor(name, args, value)
            }

            // Array patterns: [x, y, z]
            Pattern::Array { elements, .. } => self.try_match_array(elements, value),

            // OR patterns: try each sub-pattern, return first match
            Pattern::Or(alternatives, _) => {
                for alt in alternatives {
                    if let Some(bindings) = self.try_match_pattern(alt, value) {
                        return Some(bindings);
                    }
                }
                None
            }
        }
    }

    /// Try to match constructor pattern
    fn try_match_constructor(
        &self,
        name: &crate::ast::Identifier,
        args: &[Pattern],
        value: &Value,
    ) -> Option<Vec<(String, Value)>> {
        match name.name.as_str() {
            "Some" => {
                // Match Option::Some
                if let Value::Option(Some(inner)) = value {
                    if args.len() != 1 {
                        return None; // Type checker should prevent this
                    }
                    self.try_match_pattern(&args[0], inner)
                } else {
                    None
                }
            }
            "None" => {
                // Match Option::None
                if let Value::Option(None) = value {
                    if args.is_empty() {
                        Some(Vec::new())
                    } else {
                        None // Type checker should prevent this
                    }
                } else {
                    None
                }
            }
            "Ok" => {
                // Match Result::Ok
                if let Value::Result(Ok(inner)) = value {
                    if args.len() != 1 {
                        return None; // Type checker should prevent this
                    }
                    self.try_match_pattern(&args[0], inner)
                } else {
                    None
                }
            }
            "Err" => {
                // Match Result::Err
                if let Value::Result(Err(inner)) = value {
                    if args.len() != 1 {
                        return None; // Type checker should prevent this
                    }
                    self.try_match_pattern(&args[0], inner)
                } else {
                    None
                }
            }
            _ => None, // Unknown constructor
        }
    }

    /// Try to match array pattern
    fn try_match_array(
        &self,
        pattern_elements: &[Pattern],
        value: &Value,
    ) -> Option<Vec<(String, Value)>> {
        if let Value::Array(arr) = value {
            // Array patterns must have exact length match
            if arr.len() != pattern_elements.len() {
                return None;
            }

            let mut all_bindings = Vec::new();

            // Match each element
            for (pattern, element) in pattern_elements.iter().zip(arr.iter()) {
                if let Some(bindings) = self.try_match_pattern(pattern, element) {
                    all_bindings.extend(bindings);
                } else {
                    return None; // One element didn't match
                }
            }

            Some(all_bindings)
        } else {
            None // Not an array
        }
    }

    /// Check if two values are equal (for pattern matching)
    fn values_equal(&self, a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Number(x), Value::Number(y)) => x == y,
            (Value::String(x), Value::String(y)) => x == y,
            (Value::Bool(x), Value::Bool(y)) => x == y,
            (Value::Null, Value::Null) => true,
            _ => false,
        }
    }

    // ========================================================================
    // Array Intrinsics (Callback-based operations)
    // ========================================================================

    /// map(array, callback) - Transform each element
    fn intrinsic_map(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "map() expects 2 arguments (array, callback)".to_string(),
                span,
            });
        }

        let arr = match &args[0] {
            Value::Array(a) => a.iter().cloned().collect::<Vec<_>>(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "map() first argument must be array".to_string(),
                    span,
                })
            }
        };

        let callback = match &args[1] {
            Value::Function(_) | Value::Builtin(_) | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "map() second argument must be function".to_string(),
                    span,
                })
            }
        };

        let mut result = Vec::with_capacity(arr.len());
        for elem in arr {
            // Call callback with element
            let callback_result = self.call_value(callback, vec![elem], span)?;
            result.push(callback_result);
        }

        Ok(Value::array(result))
    }

    /// filter(array, predicate) - Keep elements matching predicate
    fn intrinsic_filter(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "filter() expects 2 arguments (array, predicate)".to_string(),
                span,
            });
        }

        let arr = match &args[0] {
            Value::Array(a) => a.iter().cloned().collect::<Vec<_>>(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "filter() first argument must be array".to_string(),
                    span,
                })
            }
        };

        let predicate = match &args[1] {
            Value::Function(_) | Value::Builtin(_) | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "filter() second argument must be function".to_string(),
                    span,
                })
            }
        };

        let mut result = Vec::new();
        for elem in arr {
            let pred_result = self.call_value(predicate, vec![elem.clone()], span)?;
            match pred_result {
                Value::Bool(true) => result.push(elem),
                Value::Bool(false) => {}
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "filter() predicate must return bool".to_string(),
                        span,
                    })
                }
            }
        }

        Ok(Value::array(result))
    }

    /// reduce(array, reducer, initial) - Accumulate to single value
    fn intrinsic_reduce(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 3 {
            return Err(RuntimeError::TypeError {
                msg: "reduce() expects 3 arguments (array, reducer, initial)".to_string(),
                span,
            });
        }

        let arr = match &args[0] {
            Value::Array(a) => a.iter().cloned().collect::<Vec<_>>(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "reduce() first argument must be array".to_string(),
                    span,
                })
            }
        };

        let reducer = match &args[1] {
            Value::Function(_) | Value::Builtin(_) | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "reduce() second argument must be function".to_string(),
                    span,
                })
            }
        };

        let mut accumulator = args[2].clone();
        for elem in arr {
            accumulator = self.call_value(reducer, vec![accumulator, elem], span)?;
        }

        Ok(accumulator)
    }

    /// forEach(array, callback) - Execute callback for each element
    fn intrinsic_for_each(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "forEach() expects 2 arguments (array, callback)".to_string(),
                span,
            });
        }

        let arr = match &args[0] {
            Value::Array(a) => a.iter().cloned().collect::<Vec<_>>(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "forEach() first argument must be array".to_string(),
                    span,
                })
            }
        };

        let callback = match &args[1] {
            Value::Function(_) | Value::Builtin(_) | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "forEach() second argument must be function".to_string(),
                    span,
                })
            }
        };

        for elem in arr {
            self.call_value(callback, vec![elem], span)?;
        }

        Ok(Value::Null)
    }

    /// find(array, predicate) - Find first matching element
    fn intrinsic_find(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "find() expects 2 arguments (array, predicate)".to_string(),
                span,
            });
        }

        let arr = match &args[0] {
            Value::Array(a) => a.iter().cloned().collect::<Vec<_>>(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "find() first argument must be array".to_string(),
                    span,
                })
            }
        };

        let predicate = match &args[1] {
            Value::Function(_) | Value::Builtin(_) | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "find() second argument must be function".to_string(),
                    span,
                })
            }
        };

        for elem in arr {
            let pred_result = self.call_value(predicate, vec![elem.clone()], span)?;
            match pred_result {
                Value::Bool(true) => return Ok(elem),
                Value::Bool(false) => {}
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "find() predicate must return bool".to_string(),
                        span,
                    })
                }
            }
        }

        Ok(Value::Null)
    }

    /// findIndex(array, predicate) - Find index of first matching element
    fn intrinsic_find_index(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "findIndex() expects 2 arguments (array, predicate)".to_string(),
                span,
            });
        }

        let arr = match &args[0] {
            Value::Array(a) => a.iter().cloned().collect::<Vec<_>>(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "findIndex() first argument must be array".to_string(),
                    span,
                })
            }
        };

        let predicate = match &args[1] {
            Value::Function(_) | Value::Builtin(_) | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "findIndex() second argument must be function".to_string(),
                    span,
                })
            }
        };

        for (i, elem) in arr.iter().enumerate() {
            let pred_result = self.call_value(predicate, vec![elem.clone()], span)?;
            match pred_result {
                Value::Bool(true) => return Ok(Value::Number(i as f64)),
                Value::Bool(false) => {}
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "findIndex() predicate must return bool".to_string(),
                        span,
                    })
                }
            }
        }

        Ok(Value::Number(-1.0))
    }

    /// flatMap(array, callback) - Map and flatten one level
    fn intrinsic_flat_map(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "flatMap() expects 2 arguments (array, callback)".to_string(),
                span,
            });
        }

        let arr = match &args[0] {
            Value::Array(a) => a.iter().cloned().collect::<Vec<_>>(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "flatMap() first argument must be array".to_string(),
                    span,
                })
            }
        };

        let callback = match &args[1] {
            Value::Function(_) | Value::Builtin(_) | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "flatMap() second argument must be function".to_string(),
                    span,
                })
            }
        };

        let mut result = Vec::new();
        for elem in arr {
            let callback_result = self.call_value(callback, vec![elem], span)?;
            match callback_result {
                Value::Array(nested) => {
                    result.extend(nested.iter().cloned());
                }
                other => result.push(other),
            }
        }

        Ok(Value::array(result))
    }

    /// some(array, predicate) - Check if any element matches
    fn intrinsic_some(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "some() expects 2 arguments (array, predicate)".to_string(),
                span,
            });
        }

        let arr = match &args[0] {
            Value::Array(a) => a.iter().cloned().collect::<Vec<_>>(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "some() first argument must be array".to_string(),
                    span,
                })
            }
        };

        let predicate = match &args[1] {
            Value::Function(_) | Value::Builtin(_) | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "some() second argument must be function".to_string(),
                    span,
                })
            }
        };

        for elem in arr {
            let pred_result = self.call_value(predicate, vec![elem], span)?;
            match pred_result {
                Value::Bool(true) => return Ok(Value::Bool(true)),
                Value::Bool(false) => {}
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "some() predicate must return bool".to_string(),
                        span,
                    })
                }
            }
        }

        Ok(Value::Bool(false))
    }

    /// every(array, predicate) - Check if all elements match
    fn intrinsic_every(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "every() expects 2 arguments (array, predicate)".to_string(),
                span,
            });
        }

        let arr = match &args[0] {
            Value::Array(a) => a.iter().cloned().collect::<Vec<_>>(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "every() first argument must be array".to_string(),
                    span,
                })
            }
        };

        let predicate = match &args[1] {
            Value::Function(_) | Value::Builtin(_) | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "every() second argument must be function".to_string(),
                    span,
                })
            }
        };

        for elem in arr {
            let pred_result = self.call_value(predicate, vec![elem], span)?;
            match pred_result {
                Value::Bool(false) => return Ok(Value::Bool(false)),
                Value::Bool(true) => {}
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "every() predicate must return bool".to_string(),
                        span,
                    })
                }
            }
        }

        Ok(Value::Bool(true))
    }

    /// sort(array, comparator) - Sort with custom comparator
    ///
    /// Uses insertion sort for stability and simplicity with callbacks
    fn intrinsic_sort(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "sort() expects 2 arguments (array, comparator)".to_string(),
                span,
            });
        }

        let arr = match &args[0] {
            Value::Array(a) => a.iter().cloned().collect::<Vec<_>>(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "sort() first argument must be array".to_string(),
                    span,
                })
            }
        };

        let comparator = match &args[1] {
            Value::Function(_) | Value::Builtin(_) | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "sort() second argument must be function".to_string(),
                    span,
                })
            }
        };

        // Simple insertion sort (stable) with callback comparisons
        let mut sorted = arr;
        for i in 1..sorted.len() {
            let mut j = i;
            while j > 0 {
                let cmp_result = self.call_value(
                    comparator,
                    vec![sorted[j].clone(), sorted[j - 1].clone()],
                    span,
                )?;
                match cmp_result {
                    Value::Number(n) if n < 0.0 => {
                        sorted.swap(j, j - 1);
                        j -= 1;
                    }
                    Value::Number(_) => break,
                    _ => {
                        return Err(RuntimeError::TypeError {
                            msg: "sort() comparator must return number".to_string(),
                            span,
                        })
                    }
                }
            }
        }

        Ok(Value::array(sorted))
    }

    /// sortBy(array, keyExtractor) - Sort by extracted key
    fn intrinsic_sort_by(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "sortBy() expects 2 arguments (array, keyExtractor)".to_string(),
                span,
            });
        }

        let arr = match &args[0] {
            Value::Array(a) => a.iter().cloned().collect::<Vec<_>>(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "sortBy() first argument must be array".to_string(),
                    span,
                })
            }
        };

        let key_extractor = match &args[1] {
            Value::Function(_) | Value::Builtin(_) | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "sortBy() second argument must be function".to_string(),
                    span,
                })
            }
        };

        // Extract keys first
        let mut keyed: Vec<(Value, Value)> = Vec::new();
        for elem in arr {
            let key = self.call_value(key_extractor, vec![elem.clone()], span)?;
            keyed.push((key, elem));
        }

        // Sort by keys (stable)
        keyed.sort_by(|(key_a, _), (key_b, _)| match (key_a, key_b) {
            (Value::Number(a), Value::Number(b)) => {
                if a < b {
                    std::cmp::Ordering::Less
                } else if a > b {
                    std::cmp::Ordering::Greater
                } else {
                    std::cmp::Ordering::Equal
                }
            }
            (Value::String(a), Value::String(b)) => a.cmp(b),
            _ => std::cmp::Ordering::Equal,
        });

        // Extract sorted elements
        let sorted: Vec<Value> = keyed.into_iter().map(|(_, elem)| elem).collect();
        Ok(Value::array(sorted))
    }

    // ========================================================================
    // Result Intrinsics (Callback-based operations)
    // ========================================================================

    /// result_map(result, transform_fn) - Transform Ok value
    fn intrinsic_result_map(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "result_map() expects 2 arguments (result, transform_fn)".to_string(),
                span,
            });
        }

        let result_val = &args[0];
        let transform_fn = match &args[1] {
            Value::Function(_) | Value::Builtin(_) | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "result_map() second argument must be function".to_string(),
                    span,
                })
            }
        };

        match result_val {
            Value::Result(Ok(val)) => {
                let transformed = self.call_value(transform_fn, vec![(**val).clone()], span)?;
                Ok(Value::Result(Ok(Box::new(transformed))))
            }
            Value::Result(Err(err)) => Ok(Value::Result(Err(err.clone()))),
            _ => Err(RuntimeError::TypeError {
                msg: "result_map() first argument must be Result".to_string(),
                span,
            }),
        }
    }

    /// result_map_err(result, transform_fn) - Transform Err value
    fn intrinsic_result_map_err(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "result_map_err() expects 2 arguments (result, transform_fn)".to_string(),
                span,
            });
        }

        let result_val = &args[0];
        let transform_fn = match &args[1] {
            Value::Function(_) | Value::Builtin(_) | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "result_map_err() second argument must be function".to_string(),
                    span,
                })
            }
        };

        match result_val {
            Value::Result(Ok(val)) => Ok(Value::Result(Ok(val.clone()))),
            Value::Result(Err(err)) => {
                let transformed = self.call_value(transform_fn, vec![(**err).clone()], span)?;
                Ok(Value::Result(Err(Box::new(transformed))))
            }
            _ => Err(RuntimeError::TypeError {
                msg: "result_map_err() first argument must be Result".to_string(),
                span,
            }),
        }
    }

    /// result_and_then(result, next_fn) - Chain Results (monadic bind)
    fn intrinsic_result_and_then(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "result_and_then() expects 2 arguments (result, next_fn)".to_string(),
                span,
            });
        }

        let result_val = &args[0];
        let next_fn = match &args[1] {
            Value::Function(_) | Value::Builtin(_) | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "result_and_then() second argument must be function".to_string(),
                    span,
                })
            }
        };

        match result_val {
            Value::Result(Ok(val)) => {
                // Call next_fn which should return a Result
                self.call_value(next_fn, vec![(**val).clone()], span)
            }
            Value::Result(Err(err)) => Ok(Value::Result(Err(err.clone()))),
            _ => Err(RuntimeError::TypeError {
                msg: "result_and_then() first argument must be Result".to_string(),
                span,
            }),
        }
    }

    /// result_or_else(result, recovery_fn) - Recover from Err
    fn intrinsic_result_or_else(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "result_or_else() expects 2 arguments (result, recovery_fn)".to_string(),
                span,
            });
        }

        let result_val = &args[0];
        let recovery_fn = match &args[1] {
            Value::Function(_) | Value::Builtin(_) | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "result_or_else() second argument must be function".to_string(),
                    span,
                })
            }
        };

        match result_val {
            Value::Result(Ok(val)) => Ok(Value::Result(Ok(val.clone()))),
            Value::Result(Err(err)) => {
                // Call recovery_fn which should return a Result
                self.call_value(recovery_fn, vec![(**err).clone()], span)
            }
            _ => Err(RuntimeError::TypeError {
                msg: "result_or_else() first argument must be Result".to_string(),
                span,
            }),
        }
    }

    /// hashMapForEach(map, callback) - Iterate over map entries with side effects
    fn intrinsic_hashmap_for_each(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "hashMapForEach() expects 2 arguments (map, callback)".to_string(),
                span,
            });
        }

        let map = match &args[0] {
            Value::HashMap(m) => m.inner().entries(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "hashMapForEach() first argument must be HashMap".to_string(),
                    span,
                })
            }
        };

        let callback = match &args[1] {
            Value::Function(_) | Value::Builtin(_) | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "hashMapForEach() second argument must be function".to_string(),
                    span,
                })
            }
        };

        for (key, value) in map {
            // Call callback with (value, key) arguments
            self.call_value(callback, vec![value, key.to_value()], span)?;
        }

        Ok(Value::Null)
    }

    /// hashMapMap(map, callback) - Transform values, return new map
    fn intrinsic_hashmap_map(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "hashMapMap() expects 2 arguments (map, callback)".to_string(),
                span,
            });
        }

        let map = match &args[0] {
            Value::HashMap(m) => m.inner().entries(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "hashMapMap() first argument must be HashMap".to_string(),
                    span,
                })
            }
        };

        let callback = match &args[1] {
            Value::Function(_) | Value::Builtin(_) | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "hashMapMap() second argument must be function".to_string(),
                    span,
                })
            }
        };

        let mut result_map = crate::value::ValueHashMap::new();
        for (key, value) in map {
            // Call callback with (value, key) arguments
            let new_value = self.call_value(callback, vec![value, key.clone().to_value()], span)?;
            result_map.inner_mut().insert(key, new_value);
        }

        Ok(Value::HashMap(result_map))
    }

    /// hashMapFilter(map, predicate) - Filter entries, return new map
    fn intrinsic_hashmap_filter(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "hashMapFilter() expects 2 arguments (map, predicate)".to_string(),
                span,
            });
        }

        let map = match &args[0] {
            Value::HashMap(m) => m.inner().entries(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "hashMapFilter() first argument must be HashMap".to_string(),
                    span,
                })
            }
        };

        let predicate = match &args[1] {
            Value::Function(_) | Value::Builtin(_) | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "hashMapFilter() second argument must be function".to_string(),
                    span,
                })
            }
        };

        let mut result_map = crate::value::ValueHashMap::new();
        for (key, value) in map {
            // Call predicate with (value, key) arguments
            let pred_result =
                self.call_value(predicate, vec![value.clone(), key.clone().to_value()], span)?;
            match pred_result {
                Value::Bool(true) => {
                    result_map.inner_mut().insert(key, value);
                }
                Value::Bool(false) => {}
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "hashMapFilter() predicate must return bool".to_string(),
                        span,
                    })
                }
            }
        }

        Ok(Value::HashMap(result_map))
    }

    /// hashSetForEach(set, callback) - Iterate over set elements with side effects
    fn intrinsic_hashset_for_each(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "hashSetForEach() expects 2 arguments (set, callback)".to_string(),
                span,
            });
        }

        let set = match &args[0] {
            Value::HashSet(s) => s.inner().to_vec(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "hashSetForEach() first argument must be HashSet".to_string(),
                    span,
                })
            }
        };

        let callback = match &args[1] {
            Value::Function(_) | Value::Builtin(_) | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "hashSetForEach() second argument must be function".to_string(),
                    span,
                })
            }
        };

        for element in set {
            // Call callback with element argument
            self.call_value(callback, vec![element.to_value()], span)?;
        }

        Ok(Value::Null)
    }

    /// hashSetMap(set, callback) - Transform elements to array
    fn intrinsic_hashset_map(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "hashSetMap() expects 2 arguments (set, callback)".to_string(),
                span,
            });
        }

        let set = match &args[0] {
            Value::HashSet(s) => s.inner().to_vec(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "hashSetMap() first argument must be HashSet".to_string(),
                    span,
                })
            }
        };

        let callback = match &args[1] {
            Value::Function(_) | Value::Builtin(_) | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "hashSetMap() second argument must be function".to_string(),
                    span,
                })
            }
        };

        let mut result = Vec::new();
        for element in set {
            // Call callback with element argument
            let mapped_value = self.call_value(callback, vec![element.to_value()], span)?;
            result.push(mapped_value);
        }

        Ok(Value::array(result))
    }

    /// hashSetFilter(set, predicate) - Filter elements, return new set
    fn intrinsic_hashset_filter(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "hashSetFilter() expects 2 arguments (set, predicate)".to_string(),
                span,
            });
        }

        let set = match &args[0] {
            Value::HashSet(s) => s.inner().to_vec(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "hashSetFilter() first argument must be HashSet".to_string(),
                    span,
                })
            }
        };

        let predicate = match &args[1] {
            Value::Function(_) | Value::Builtin(_) | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "hashSetFilter() second argument must be function".to_string(),
                    span,
                })
            }
        };

        let mut result_set = crate::value::ValueHashSet::new();
        for element in set {
            // Call predicate with element argument
            let pred_result = self.call_value(predicate, vec![element.clone().to_value()], span)?;
            match pred_result {
                Value::Bool(true) => {
                    result_set.inner_mut().insert(element);
                }
                Value::Bool(false) => {}
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "hashSetFilter() predicate must return bool".to_string(),
                        span,
                    })
                }
            }
        }

        Ok(Value::HashSet(result_set))
    }

    /// Regex intrinsic: Replace first match using callback
    fn intrinsic_regex_replace_with(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 3 {
            return Err(RuntimeError::TypeError {
                msg: "regexReplaceWith() expects 3 arguments (regex, text, callback)".to_string(),
                span,
            });
        }

        let regex = match &args[0] {
            Value::Regex(r) => r.as_ref(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "regexReplaceWith() first argument must be Regex".to_string(),
                    span,
                })
            }
        };

        let text = match &args[1] {
            Value::String(s) => s.as_ref(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "regexReplaceWith() second argument must be string".to_string(),
                    span,
                })
            }
        };

        let callback = match &args[2] {
            Value::Function(_) | Value::Builtin(_) | Value::NativeFunction(_) => &args[2],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "regexReplaceWith() third argument must be function".to_string(),
                    span,
                })
            }
        };

        // Find first match
        if let Some(mat) = regex.find(text) {
            let match_start = mat.start();
            let match_end = mat.end();
            let match_text = mat.as_str();

            // Build match data HashMap
            let mut match_map = crate::stdlib::collections::hashmap::AtlasHashMap::new();
            match_map.insert(
                crate::stdlib::collections::hash::HashKey::String(std::sync::Arc::new(
                    "text".to_string(),
                )),
                Value::string(match_text),
            );
            match_map.insert(
                crate::stdlib::collections::hash::HashKey::String(std::sync::Arc::new(
                    "start".to_string(),
                )),
                Value::Number(match_start as f64),
            );
            match_map.insert(
                crate::stdlib::collections::hash::HashKey::String(std::sync::Arc::new(
                    "end".to_string(),
                )),
                Value::Number(match_end as f64),
            );

            // Extract capture groups
            if let Some(caps) = regex.captures(text) {
                let mut groups = Vec::new();
                for i in 0..caps.len() {
                    if let Some(group) = caps.get(i) {
                        groups.push(Value::string(group.as_str()));
                    } else {
                        groups.push(Value::Null);
                    }
                }
                match_map.insert(
                    crate::stdlib::collections::hash::HashKey::String(std::sync::Arc::new(
                        "groups".to_string(),
                    )),
                    Value::array(groups),
                );
            } else {
                match_map.insert(
                    crate::stdlib::collections::hash::HashKey::String(std::sync::Arc::new(
                        "groups".to_string(),
                    )),
                    Value::array(vec![]),
                );
            }

            let match_value = Value::HashMap(crate::value::ValueHashMap::from_atlas(match_map));

            // Call callback with match data
            let replacement_value = self.call_value(callback, vec![match_value], span)?;

            // Expect string return value and clone to avoid lifetime issues
            let replacement_str = match &replacement_value {
                Value::String(s) => s.as_ref().to_string(),
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "regexReplaceWith() callback must return string".to_string(),
                        span,
                    })
                }
            };

            // Build result string
            let mut result = String::with_capacity(text.len());
            result.push_str(&text[..match_start]);
            result.push_str(&replacement_str);
            result.push_str(&text[match_end..]);

            Ok(Value::string(result))
        } else {
            // No match, return original text
            Ok(Value::string(text))
        }
    }

    /// Regex intrinsic: Replace all matches using callback
    fn intrinsic_regex_replace_all_with(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 3 {
            return Err(RuntimeError::TypeError {
                msg: "regexReplaceAllWith() expects 3 arguments (regex, text, callback)"
                    .to_string(),
                span,
            });
        }

        let regex = match &args[0] {
            Value::Regex(r) => r.as_ref(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "regexReplaceAllWith() first argument must be Regex".to_string(),
                    span,
                })
            }
        };

        let text = match &args[1] {
            Value::String(s) => s.as_ref(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "regexReplaceAllWith() second argument must be string".to_string(),
                    span,
                })
            }
        };

        let callback = match &args[2] {
            Value::Function(_) | Value::Builtin(_) | Value::NativeFunction(_) => &args[2],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "regexReplaceAllWith() third argument must be function".to_string(),
                    span,
                })
            }
        };

        // Find all matches and collect them
        let matches: Vec<_> = regex.find_iter(text).collect();

        if matches.is_empty() {
            return Ok(Value::string(text));
        }

        // Build result string by processing all matches
        let mut result = String::with_capacity(text.len());
        let mut last_end = 0;

        for mat in matches {
            let match_start = mat.start();
            let match_end = mat.end();
            let match_text = mat.as_str();

            // Build match data HashMap
            let mut match_map = crate::stdlib::collections::hashmap::AtlasHashMap::new();
            match_map.insert(
                crate::stdlib::collections::hash::HashKey::String(std::sync::Arc::new(
                    "text".to_string(),
                )),
                Value::string(match_text),
            );
            match_map.insert(
                crate::stdlib::collections::hash::HashKey::String(std::sync::Arc::new(
                    "start".to_string(),
                )),
                Value::Number(match_start as f64),
            );
            match_map.insert(
                crate::stdlib::collections::hash::HashKey::String(std::sync::Arc::new(
                    "end".to_string(),
                )),
                Value::Number(match_end as f64),
            );

            // Extract capture groups
            if let Some(caps) = regex.captures(mat.as_str()) {
                let mut groups = Vec::new();
                for i in 0..caps.len() {
                    if let Some(group) = caps.get(i) {
                        groups.push(Value::string(group.as_str()));
                    } else {
                        groups.push(Value::Null);
                    }
                }
                match_map.insert(
                    crate::stdlib::collections::hash::HashKey::String(std::sync::Arc::new(
                        "groups".to_string(),
                    )),
                    Value::array(groups),
                );
            } else {
                match_map.insert(
                    crate::stdlib::collections::hash::HashKey::String(std::sync::Arc::new(
                        "groups".to_string(),
                    )),
                    Value::array(vec![]),
                );
            }

            let match_value = Value::HashMap(crate::value::ValueHashMap::from_atlas(match_map));

            // Call callback with match data
            let replacement_value = self.call_value(callback, vec![match_value], span)?;

            // Expect string return value and clone to avoid lifetime issues
            let replacement_str = match &replacement_value {
                Value::String(s) => s.as_ref().to_string(),
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "regexReplaceAllWith() callback must return string".to_string(),
                        span,
                    })
                }
            };

            // Add text before this match
            result.push_str(&text[last_end..match_start]);
            // Add replacement
            result.push_str(&replacement_str);

            last_end = match_end;
        }

        // Add remaining text after last match
        result.push_str(&text[last_end..]);

        Ok(Value::string(result))
    }

    /// Helper: Call a function value with arguments
    fn call_value(
        &mut self,
        func: &Value,
        args: Vec<Value>,
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        match func {
            Value::Builtin(name) => {
                let security = self
                    .current_security
                    .as_ref()
                    .expect("Security context not set");
                crate::stdlib::call_builtin(name, &args, span, security, &self.output_writer)
            }
            Value::Function(func_ref) => {
                // User-defined function
                if let Some(user_func) = self.function_bodies.get(&func_ref.name).cloned() {
                    return self.call_user_function(&user_func, args, span);
                }

                Err(RuntimeError::UnknownFunction {
                    name: func_ref.name.clone(),
                    span,
                })
            }
            Value::NativeFunction(native_fn) => native_fn(&args),
            _ => Err(RuntimeError::TypeError {
                msg: "Expected function value".to_string(),
                span,
            }),
        }
    }

    /// Apply CoW write-back for collection mutation builtins.
    ///
    /// When a builtin mutates a collection by returning a new value, we write the
    /// new value back to the first argument variable (if it's an identifier).
    ///
    /// - "returns new collection" builtins: write `result` back to arg[0]
    /// - "returns [extracted, new collection]" builtins: write `result[1]` back to arg[0]
    fn apply_cow_writeback(
        &mut self,
        name: &str,
        result: Value,
        call_args: &[Expr],
        _span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        // Builtins that return the modified collection directly
        const RETURNS_COLLECTION: &[&str] = &[
            // HashMap
            "hashMapPut",
            "hashMapClear",
            // HashSet
            "hashSetAdd",
            "hashSetClear",
            // Queue
            "queueEnqueue",
            "queueClear",
            // Stack
            "stackPush",
            "stackClear",
            // Array (free-function variants)
            "unshift",
            "reverse",
            "flatten",
        ];

        // Builtins that return [extracted_value, new_collection]
        const RETURNS_PAIR: &[&str] = &[
            // HashMap / HashSet / Queue / Stack
            "hashMapRemove",
            "hashSetRemove",
            "queueDequeue",
            "stackPop",
            // Array (free-function variants)
            "pop",
            "shift",
        ];

        // Identify first arg as an identifier (only then can we write back)
        let first_ident = call_args.first().and_then(|e| {
            if let Expr::Identifier(id) = e {
                Some(id.name.clone())
            } else {
                None
            }
        });

        if let Some(var_name) = first_ident {
            if RETURNS_COLLECTION.contains(&name) {
                // Bypass mutability check: this is container content mutation,
                // not a variable rebinding.
                self.force_set_collection(&var_name, result.clone());
                return Ok(result);
            }

            if RETURNS_PAIR.contains(&name) {
                // result is [extracted_value, new_collection].
                // Write new_collection back to variable, return only extracted_value.
                // Atlas-level: `let item = queueDequeue(q)` → item is Option, q is updated.
                if let Value::Array(ref arr) = result {
                    let s = arr.as_slice();
                    if s.len() == 2 {
                        let extracted = s[0].clone();
                        let new_col = s[1].clone();
                        self.force_set_collection(&var_name, new_col);
                        return Ok(extracted);
                    }
                }
                return Ok(result);
            }
        }

        Ok(result)
    }
}
