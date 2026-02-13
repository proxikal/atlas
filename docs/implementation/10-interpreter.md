# Interpreter Architecture

Direct AST evaluation (tree-walk interpreter).

## Interpreter Structure

```rust
// interpreter/ module (interpreter/mod.rs + interpreter/stmt.rs + interpreter/expr.rs)
pub struct Interpreter {
    globals: HashMap<String, Value>,
    locals: Vec<HashMap<String, Value>>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            globals: HashMap::new(),
            locals: vec![HashMap::new()],
        }
    }

    pub fn eval(&mut self, program: &Program) -> Result<Value, RuntimeError> {
        let mut last_value = Value::Null;

        for item in &program.items {
            match item {
                Item::Function(func) => {
                    // Store function in globals (simplified)
                    let func_value = Value::Function(FunctionRef {
                        name: func.name.name.clone(),
                        arity: func.params.len(),
                        bytecode_offset: 0,  // Not used in interpreter
                    });
                    self.globals.insert(func.name.name.clone(), func_value);
                }
                Item::Statement(stmt) => {
                    last_value = self.eval_statement(stmt)?;
                }
            }
        }

        Ok(last_value)
    }

    fn eval_statement(&mut self, stmt: &Stmt) -> Result<Value, RuntimeError> {
        match stmt {
            Stmt::VarDecl(var) => {
                let value = self.eval_expr(&var.init)?;
                let scope = self.locals.last_mut().unwrap();
                scope.insert(var.name.name.clone(), value);
                Ok(Value::Null)
            }
            Stmt::Assign(assign) => {
                let value = self.eval_expr(&assign.value)?;
                match &assign.target {
                    AssignTarget::Name(id) => {
                        self.set_variable(&id.name, value)?;
                    }
                    AssignTarget::Index { target, index, .. } => {
                        let arr = self.eval_expr(target)?;
                        let idx = self.eval_expr(index)?;
                        self.set_array_element(arr, idx, value)?;
                    }
                }
                Ok(Value::Null)
            }
            Stmt::CompoundAssign(compound) => {
                self.eval_compound_assign(compound)
            }
            Stmt::Increment(inc) => {
                self.eval_increment(inc)
            }
            Stmt::Decrement(dec) => {
                self.eval_decrement(dec)
            }
            Stmt::Expr(expr_stmt) => self.eval_expr(&expr_stmt.expr),
            _ => Ok(Value::Null),
        }
    }

    fn eval_compound_assign(&mut self, compound: &CompoundAssign) -> Result<Value, RuntimeError> {
        // Get current value
        let current = match &compound.target {
            AssignTarget::Name(id) => self.get_variable(&id.name)?,
            AssignTarget::Index { target, index, .. } => {
                let arr = self.eval_expr(target)?;
                let idx = self.eval_expr(index)?;
                self.get_array_element(arr, idx)?
            }
        };

        // Evaluate new value
        let rhs = self.eval_expr(&compound.value)?;

        // Apply operation
        let result = match (&current, &rhs) {
            (Value::Number(a), Value::Number(b)) => {
                let value = match compound.op {
                    CompoundOp::AddAssign => a + b,
                    CompoundOp::SubAssign => a - b,
                    CompoundOp::MulAssign => a * b,
                    CompoundOp::DivAssign => {
                        if b == &0.0 {
                            return Err(RuntimeError::DivideByZero);
                        }
                        a / b
                    }
                    CompoundOp::ModAssign => {
                        if b == &0.0 {
                            return Err(RuntimeError::DivideByZero);
                        }
                        a % b
                    }
                };

                if value.is_nan() || value.is_infinite() {
                    return Err(RuntimeError::InvalidNumericResult);
                }
                Value::Number(value)
            }
            _ => return Err(RuntimeError::TypeError),
        };

        // Store result
        match &compound.target {
            AssignTarget::Name(id) => {
                self.set_variable(&id.name, result)?;
            }
            AssignTarget::Index { target, index, .. } => {
                let arr = self.eval_expr(target)?;
                let idx = self.eval_expr(index)?;
                self.set_array_element(arr, idx, result)?;
            }
        }

        Ok(Value::Null)
    }

    fn eval_increment(&mut self, inc: &IncrementStmt) -> Result<Value, RuntimeError> {
        let current = match &inc.target {
            AssignTarget::Name(id) => self.get_variable(&id.name)?,
            AssignTarget::Index { target, index, .. } => {
                let arr = self.eval_expr(target)?;
                let idx = self.eval_expr(index)?;
                self.get_array_element(arr, idx)?
            }
        };

        if let Value::Number(n) = current {
            let result = n + 1.0;
            if result.is_nan() || result.is_infinite() {
                return Err(RuntimeError::InvalidNumericResult);
            }

            match &inc.target {
                AssignTarget::Name(id) => {
                    self.set_variable(&id.name, Value::Number(result))?;
                }
                AssignTarget::Index { target, index, .. } => {
                    let arr = self.eval_expr(target)?;
                    let idx = self.eval_expr(index)?;
                    self.set_array_element(arr, idx, Value::Number(result))?;
                }
            }
            Ok(Value::Null)
        } else {
            Err(RuntimeError::TypeError)
        }
    }

    fn eval_decrement(&mut self, dec: &DecrementStmt) -> Result<Value, RuntimeError> {
        let current = match &dec.target {
            AssignTarget::Name(id) => self.get_variable(&id.name)?,
            AssignTarget::Index { target, index, .. } => {
                let arr = self.eval_expr(target)?;
                let idx = self.eval_expr(index)?;
                self.get_array_element(arr, idx)?
            }
        };

        if let Value::Number(n) = current {
            let result = n - 1.0;
            if result.is_nan() || result.is_infinite() {
                return Err(RuntimeError::InvalidNumericResult);
            }

            match &dec.target {
                AssignTarget::Name(id) => {
                    self.set_variable(&id.name, Value::Number(result))?;
                }
                AssignTarget::Index { target, index, .. } => {
                    let arr = self.eval_expr(target)?;
                    let idx = self.eval_expr(index)?;
                    self.set_array_element(arr, idx, Value::Number(result))?;
                }
            }
            Ok(Value::Null)
        } else {
            Err(RuntimeError::TypeError)
        }
    }

    fn eval_expr(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
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

    fn eval_literal(&self, lit: &Literal) -> Value {
        match lit {
            Literal::Number(n) => Value::Number(*n),
            Literal::String(s) => Value::String(Rc::new(s.clone())),
            Literal::Bool(b) => Value::Bool(*b),
            Literal::Null => Value::Null,
        }
    }

    fn eval_binary(&mut self, binary: &BinaryExpr) -> Result<Value, RuntimeError> {
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
                    Ok(Value::String(Rc::new(format!("{}{}", a, b))))
                }
                _ => Err(RuntimeError::TypeError),
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
                    Err(RuntimeError::TypeError)
                }
            }
            BinaryOp::Eq => Ok(Value::Bool(left == right)),
            BinaryOp::Ne => Ok(Value::Bool(left != right)),
            BinaryOp::Lt => self.numeric_comparison(left, right, |a, b| a < b),
            BinaryOp::Le => self.numeric_comparison(left, right, |a, b| a <= b),
            BinaryOp::Gt => self.numeric_comparison(left, right, |a, b| a > b),
            BinaryOp::Ge => self.numeric_comparison(left, right, |a, b| a >= b),
            BinaryOp::And => {
                if let Value::Bool(a) = left {
                    if !a {
                        return Ok(Value::Bool(false));
                    }
                    if let Value::Bool(b) = right {
                        return Ok(Value::Bool(b));
                    }
                }
                Err(RuntimeError::TypeError)
            }
            BinaryOp::Or => {
                if let Value::Bool(a) = left {
                    if a {
                        return Ok(Value::Bool(true));
                    }
                    if let Value::Bool(b) = right {
                        return Ok(Value::Bool(b));
                    }
                }
                Err(RuntimeError::TypeError)
            }
            _ => Err(RuntimeError::TypeError),
        }
    }

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
            Err(RuntimeError::TypeError)
        }
    }

    fn get_variable(&self, name: &str) -> Result<Value, RuntimeError> {
        // Check locals (innermost to outermost)
        for scope in self.locals.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Ok(value.clone());
            }
        }

        // Check globals
        self.globals.get(name)
            .cloned()
            .ok_or_else(|| RuntimeError::UnknownFunction(name.to_string()))
    }

    fn set_variable(&mut self, name: &str, value: Value) -> Result<(), RuntimeError> {
        // Find in locals
        for scope in self.locals.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), value);
                return Ok(());
            }
        }

        // Set in global
        self.globals.insert(name.to_string(), value);
        Ok(())
    }

    fn eval_call(&mut self, call: &CallExpr) -> Result<Value, RuntimeError> {
        // Simplified - actual implementation handles both stdlib and user functions
        if let Expr::Identifier(id) = call.callee.as_ref() {
            let args: Result<Vec<Value>, _> = call.args.iter()
                .map(|arg| self.eval_expr(arg))
                .collect();
            let args = args?;

            // Check for stdlib functions
            match id.name.as_str() {
                "print" | "len" | "str" => {
                    return crate::stdlib::call_builtin(&id.name, &args)
                        .map_err(|_| RuntimeError::InvalidStdlibArgument);
                }
                _ => {}
            }
        }

        Err(RuntimeError::TypeError)
    }
}
```

## Key Design Decisions

- **Environment model:** Global + local scopes (Vec of HashMaps)
- **Value cloning:** Values are cloned on assignment (cheap for Rc)
- **Evaluation order:** Left-to-right, strictly
- **Short-circuit:** `&&` and `||` short-circuit properly
- **Error checking:** NaN/Infinity checked on every numeric operation
