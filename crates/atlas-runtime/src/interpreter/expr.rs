//! Expression evaluation

use crate::ast::*;
use crate::interpreter::{ControlFlow, Interpreter, UserFunction};
use crate::value::{RuntimeError, Value};

impl Interpreter {
    /// Evaluate an expression
    pub(super) fn eval_expr(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Literal(lit, _) => Ok(self.eval_literal(lit)),
            Expr::Identifier(id) => self.get_variable(&id.name),
            Expr::Binary(binary) => self.eval_binary(binary),
            Expr::Unary(unary) => self.eval_unary(unary),
            Expr::Call(call) => self.eval_call(call),
            Expr::Index(index) => self.eval_index(index),
            Expr::ArrayLiteral(arr) => self.eval_array_literal(arr),
            Expr::Group(group) => self.eval_expr(&group.expr),
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
            return Err(RuntimeError::TypeError("Expected bool for &&".to_string()));
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
            return Err(RuntimeError::TypeError("Expected bool for ||".to_string()));
        }

        // Regular binary operations
        let left = self.eval_expr(&binary.left)?;
        let right = self.eval_expr(&binary.right)?;

        match binary.op {
            BinaryOp::Add => match (&left, &right) {
                (Value::Number(a), Value::Number(b)) => {
                    let result = a + b;
                    if result.is_nan() || result.is_infinite() {
                        return Err(RuntimeError::InvalidNumericResult);
                    }
                    Ok(Value::Number(result))
                }
                (Value::String(a), Value::String(b)) => {
                    Ok(Value::string(format!("{}{}", a, b)))
                }
                _ => Err(RuntimeError::TypeError("Invalid operands for +".to_string())),
            },
            BinaryOp::Sub => self.numeric_binary_op(left, right, |a, b| a - b),
            BinaryOp::Mul => self.numeric_binary_op(left, right, |a, b| a * b),
            BinaryOp::Div => {
                if let (Value::Number(a), Value::Number(b)) = (&left, &right) {
                    if *b == 0.0 {
                        return Err(RuntimeError::DivideByZero);
                    }
                    let result = a / b;
                    if result.is_nan() || result.is_infinite() {
                        return Err(RuntimeError::InvalidNumericResult);
                    }
                    Ok(Value::Number(result))
                } else {
                    Err(RuntimeError::TypeError("Expected numbers for /".to_string()))
                }
            }
            BinaryOp::Mod => {
                if let (Value::Number(a), Value::Number(b)) = (&left, &right) {
                    if *b == 0.0 {
                        return Err(RuntimeError::DivideByZero);
                    }
                    let result = a % b;
                    if result.is_nan() || result.is_infinite() {
                        return Err(RuntimeError::InvalidNumericResult);
                    }
                    Ok(Value::Number(result))
                } else {
                    Err(RuntimeError::TypeError("Expected numbers for %".to_string()))
                }
            }
            BinaryOp::Eq => Ok(Value::Bool(left == right)),
            BinaryOp::Ne => Ok(Value::Bool(left != right)),
            BinaryOp::Lt => self.numeric_comparison(left, right, |a, b| a < b),
            BinaryOp::Le => self.numeric_comparison(left, right, |a, b| a <= b),
            BinaryOp::Gt => self.numeric_comparison(left, right, |a, b| a > b),
            BinaryOp::Ge => self.numeric_comparison(left, right, |a, b| a >= b),
            BinaryOp::And | BinaryOp::Or => {
                // Already handled above
                unreachable!()
            }
        }
    }

    /// Helper for numeric binary operations
    fn numeric_binary_op<F>(&self, left: Value, right: Value, op: F) -> Result<Value, RuntimeError>
    where
        F: FnOnce(f64, f64) -> f64,
    {
        if let (Value::Number(a), Value::Number(b)) = (left, right) {
            let result = op(a, b);
            if result.is_nan() || result.is_infinite() {
                return Err(RuntimeError::InvalidNumericResult);
            }
            Ok(Value::Number(result))
        } else {
            Err(RuntimeError::TypeError("Expected numbers".to_string()))
        }
    }

    /// Helper for numeric comparisons
    fn numeric_comparison<F>(
        &self,
        left: Value,
        right: Value,
        op: F,
    ) -> Result<Value, RuntimeError>
    where
        F: FnOnce(f64, f64) -> bool,
    {
        if let (Value::Number(a), Value::Number(b)) = (left, right) {
            Ok(Value::Bool(op(a, b)))
        } else {
            Err(RuntimeError::TypeError("Expected numbers for comparison".to_string()))
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
                    Err(RuntimeError::TypeError("Expected number for -".to_string()))
                }
            }
            UnaryOp::Not => {
                if let Value::Bool(b) = operand {
                    Ok(Value::Bool(!b))
                } else {
                    Err(RuntimeError::TypeError("Expected bool for !".to_string()))
                }
            }
        }
    }

    /// Evaluate a function call
    pub(super) fn eval_call(&mut self, call: &CallExpr) -> Result<Value, RuntimeError> {
        // Evaluate callee to get function name
        if let Expr::Identifier(id) = call.callee.as_ref() {
            let func_name = &id.name;

            // Evaluate arguments
            let args: Result<Vec<Value>, _> =
                call.args.iter().map(|arg| self.eval_expr(arg)).collect();
            let args = args?;

            // Check for stdlib functions first
            if crate::stdlib::is_builtin(func_name) {
                return crate::stdlib::call_builtin(func_name, &args)
                    .map_err(|_| RuntimeError::InvalidStdlibArgument);
            }

            // Check for user-defined functions
            if let Some(func) = self.functions.get(func_name).cloned() {
                return self.call_user_function(&func, args);
            }

            return Err(RuntimeError::UnknownFunction(func_name.clone()));
        }

        Err(RuntimeError::TypeError("Expected function name".to_string()))
    }

    /// Call a user-defined function
    fn call_user_function(
        &mut self,
        func: &UserFunction,
        args: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        // Check arity
        if args.len() != func.params.len() {
            return Err(RuntimeError::TypeError(format!(
                "Function {} expects {} arguments, got {}",
                func.name,
                func.params.len(),
                args.len()
            )));
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
                        return Err(RuntimeError::InvalidIndex);
                    }

                    let borrowed = arr.borrow();
                    if index_val >= 0 && (index_val as usize) < borrowed.len() {
                        Ok(borrowed[index_val as usize].clone())
                    } else {
                        Err(RuntimeError::OutOfBounds)
                    }
                } else {
                    Err(RuntimeError::InvalidIndex)
                }
            }
            Value::String(s) => {
                if let Value::Number(n) = idx {
                    let index_val = n as i64;
                    if n.fract() != 0.0 || n < 0.0 {
                        return Err(RuntimeError::InvalidIndex);
                    }

                    let chars: Vec<char> = s.chars().collect();
                    if index_val >= 0 && (index_val as usize) < chars.len() {
                        Ok(Value::string(chars[index_val as usize].to_string()))
                    } else {
                        Err(RuntimeError::OutOfBounds)
                    }
                } else {
                    Err(RuntimeError::InvalidIndex)
                }
            }
            _ => Err(RuntimeError::TypeError("Cannot index non-array/string".to_string())),
        }
    }

    /// Evaluate array literal
    fn eval_array_literal(&mut self, arr: &crate::ast::ArrayLiteral) -> Result<Value, RuntimeError> {
        let elements: Result<Vec<Value>, _> =
            arr.elements.iter().map(|e| self.eval_expr(e)).collect();
        Ok(Value::array(elements?))
    }
}
