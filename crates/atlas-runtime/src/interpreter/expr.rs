//! Expression evaluation

use crate::ast::*;
use crate::interpreter::{ControlFlow, Interpreter, UserFunction};
use crate::value::{RuntimeError, Value};
use std::rc::Rc;

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

        // Evaluate arguments
        let args: Result<Vec<Value>, _> = call.args.iter().map(|arg| self.eval_expr(arg)).collect();
        let args = args?;

        // Callee must be a function value
        match callee_value {
            Value::Function(func_ref) => {
                // Check for stdlib functions first
                if crate::stdlib::is_builtin(&func_ref.name) {
                    let security =
                        unsafe { &*self.current_security.expect("Security context not set") };
                    return crate::stdlib::call_builtin(&func_ref.name, &args, call.span, security);
                }

                // Check for array intrinsics (callback-based functions)
                match func_ref.name.as_str() {
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
                    _ => {}
                }

                // User-defined function - look up body
                if let Some(func) = self.function_bodies.get(&func_ref.name).cloned() {
                    return self.call_user_function(&func, args, call.span);
                }

                Err(RuntimeError::UnknownFunction {
                    name: func_ref.name.clone(),
                    span: call.span,
                })
            }
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

        // 2. Build desugared function name from target type and method name
        let func_name = method_to_function_name(&target_value, &member.member.name);

        // 3. Build argument list (target + method args)
        let mut args = vec![target_value];
        if let Some(method_args) = &member.args {
            for arg in method_args {
                args.push(self.eval_expr(arg)?);
            }
        }

        // 4. Call stdlib function
        let security = unsafe { &*self.current_security.expect("Security context not set") };
        crate::stdlib::call_builtin(&func_name, &args, member.span, security)
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

        // Bind parameters
        for (param, arg) in func.params.iter().zip(args.iter()) {
            let scope = self.locals.last_mut().unwrap();
            scope.insert(param.name.name.clone(), arg.clone());
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

                    let borrowed = arr.borrow();
                    if index_val >= 0 && (index_val as usize) < borrowed.len() {
                        Ok(borrowed[index_val as usize].clone())
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
                Ok(Value::JsonValue(Rc::new(result)))
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

                // Bind pattern variables
                for (name, value) in bindings {
                    let scope = self.locals.last_mut().unwrap();
                    scope.insert(name, value);
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
            let arr_borrow = arr.borrow();

            // Array patterns must have exact length match
            if arr_borrow.len() != pattern_elements.len() {
                return None;
            }

            let mut all_bindings = Vec::new();

            // Match each element
            for (pattern, element) in pattern_elements.iter().zip(arr_borrow.iter()) {
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
            Value::Array(a) => a.borrow().clone(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "map() first argument must be array".to_string(),
                    span,
                })
            }
        };

        let callback = match &args[1] {
            Value::Function(_) => &args[1],
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
            Value::Array(a) => a.borrow().clone(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "filter() first argument must be array".to_string(),
                    span,
                })
            }
        };

        let predicate = match &args[1] {
            Value::Function(_) => &args[1],
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
            Value::Array(a) => a.borrow().clone(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "reduce() first argument must be array".to_string(),
                    span,
                })
            }
        };

        let reducer = match &args[1] {
            Value::Function(_) => &args[1],
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
            Value::Array(a) => a.borrow().clone(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "forEach() first argument must be array".to_string(),
                    span,
                })
            }
        };

        let callback = match &args[1] {
            Value::Function(_) => &args[1],
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
            Value::Array(a) => a.borrow().clone(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "find() first argument must be array".to_string(),
                    span,
                })
            }
        };

        let predicate = match &args[1] {
            Value::Function(_) => &args[1],
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
            Value::Array(a) => a.borrow().clone(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "findIndex() first argument must be array".to_string(),
                    span,
                })
            }
        };

        let predicate = match &args[1] {
            Value::Function(_) => &args[1],
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
            Value::Array(a) => a.borrow().clone(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "flatMap() first argument must be array".to_string(),
                    span,
                })
            }
        };

        let callback = match &args[1] {
            Value::Function(_) => &args[1],
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
                    result.extend(nested.borrow().clone());
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
            Value::Array(a) => a.borrow().clone(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "some() first argument must be array".to_string(),
                    span,
                })
            }
        };

        let predicate = match &args[1] {
            Value::Function(_) => &args[1],
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
            Value::Array(a) => a.borrow().clone(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "every() first argument must be array".to_string(),
                    span,
                })
            }
        };

        let predicate = match &args[1] {
            Value::Function(_) => &args[1],
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
            Value::Array(a) => a.borrow().clone(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "sort() first argument must be array".to_string(),
                    span,
                })
            }
        };

        let comparator = match &args[1] {
            Value::Function(_) => &args[1],
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
            Value::Array(a) => a.borrow().clone(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "sortBy() first argument must be array".to_string(),
                    span,
                })
            }
        };

        let key_extractor = match &args[1] {
            Value::Function(_) => &args[1],
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

    /// Helper: Call a function value with arguments
    fn call_value(
        &mut self,
        func: &Value,
        args: Vec<Value>,
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        match func {
            Value::Function(func_ref) => {
                // Check for builtins
                if crate::stdlib::is_builtin(&func_ref.name) {
                    let security =
                        unsafe { &*self.current_security.expect("Security context not set") };
                    return crate::stdlib::call_builtin(&func_ref.name, &args, span, security);
                }

                // User-defined function
                if let Some(user_func) = self.function_bodies.get(&func_ref.name).cloned() {
                    return self.call_user_function(&user_func, args, span);
                }

                Err(RuntimeError::UnknownFunction {
                    name: func_ref.name.clone(),
                    span,
                })
            }
            _ => Err(RuntimeError::TypeError {
                msg: "Expected function value".to_string(),
                span,
            }),
        }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Convert method call to stdlib function name
///
/// Maps (type, method) pairs to function names:
///   (JsonValue, "as_string") → "jsonAsString"
///   (JsonValue, "as_number") → "jsonAsNumber"
///   (JsonValue, "as_bool") → "jsonAsBool"
///   (JsonValue, "is_null") → "jsonIsNull"
fn method_to_function_name(target: &Value, method: &str) -> String {
    match target {
        Value::JsonValue(_) => {
            // JSON methods: json.as_string() → jsonAsString()
            format!("json{}", capitalize_first(method))
        }
        // Future: Add more types here (String, Array, etc.)
        _ => {
            // Fallback (should not happen due to type checking)
            format!("unknown_{}", method)
        }
    }
}

/// Capitalize first letter and convert to camelCase
///
/// "as_string" → "AsString"
/// "is_null" → "IsNull"
fn capitalize_first(s: &str) -> String {
    // Handle snake_case methods
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect(),
                None => String::new(),
            }
        })
        .collect()
}
