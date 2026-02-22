//! Statement execution

use crate::ast::*;
use crate::interpreter::{ControlFlow, Interpreter, UserFunction};
use crate::value::{FunctionRef, RuntimeError, Value};

impl Interpreter {
    /// Execute a statement
    pub(super) fn eval_statement(&mut self, stmt: &Stmt) -> Result<Value, RuntimeError> {
        match stmt {
            Stmt::VarDecl(var) => self.eval_var_decl(var),
            Stmt::FunctionDecl(func) => {
                // Nested function declaration
                // Create scoped name to avoid collisions between nested functions
                let scoped_name = format!("{}_{}", func.name.name, self.next_func_id);
                self.next_func_id += 1;

                // Store function body with scoped internal name
                self.function_bodies.insert(
                    scoped_name.clone(),
                    UserFunction {
                        name: func.name.name.clone(),
                        params: func.params.clone(),
                        body: func.body.clone(),
                    },
                );

                // Create FunctionRef value
                let func_value = Value::Function(FunctionRef {
                    name: scoped_name, // Internal scoped name for lookup
                    arity: func.params.len(),
                    bytecode_offset: 0, // Not used in interpreter
                    local_count: 0,     // Not used in interpreter
                    param_ownership: vec![],
                    return_ownership: None,
                });

                // Store in current scope (functions are immutable bindings)
                if self.locals.is_empty() {
                    // Global scope (shouldn't happen for nested functions, but handle it)
                    self.globals
                        .insert(func.name.name.clone(), (func_value, false));
                } else {
                    // Local scope - this is the normal case for nested functions
                    self.locals
                        .last_mut()
                        .unwrap()
                        .insert(func.name.name.clone(), (func_value, false));
                }

                Ok(Value::Null)
            }
            Stmt::Assign(assign) => self.eval_assign(assign),
            Stmt::CompoundAssign(compound) => self.eval_compound_assign(compound),
            Stmt::Increment(inc) => self.eval_increment(inc),
            Stmt::Decrement(dec) => self.eval_decrement(dec),
            Stmt::If(if_stmt) => self.eval_if(if_stmt),
            Stmt::While(while_stmt) => self.eval_while(while_stmt),
            Stmt::For(for_stmt) => self.eval_for(for_stmt),
            Stmt::ForIn(for_in_stmt) => self.eval_for_in(for_in_stmt),
            Stmt::Return(return_stmt) => self.eval_return(return_stmt),
            Stmt::Break(_) => {
                self.control_flow = ControlFlow::Break;
                Ok(Value::Null)
            }
            Stmt::Continue(_) => {
                self.control_flow = ControlFlow::Continue;
                Ok(Value::Null)
            }
            Stmt::Expr(expr_stmt) => self.eval_expr(&expr_stmt.expr),
        }
    }

    /// Evaluate a variable declaration
    fn eval_var_decl(&mut self, var: &VarDecl) -> Result<Value, RuntimeError> {
        let value = self.eval_expr(&var.init)?;
        let scope = self.locals.last_mut().unwrap();
        // Store with mutability flag from the declaration
        scope.insert(var.name.name.clone(), (value, var.mutable));
        Ok(Value::Null)
    }

    /// Evaluate an assignment
    fn eval_assign(&mut self, assign: &Assign) -> Result<Value, RuntimeError> {
        let value = self.eval_expr(&assign.value)?;

        match &assign.target {
            AssignTarget::Name(id) => {
                self.set_variable(&id.name, value, assign.span)?;
            }
            AssignTarget::Index {
                target,
                index,
                span,
            } => {
                let idx_val = self.eval_expr(index)?;
                self.assign_at_index(target, idx_val, value, *span)?;
            }
        }

        Ok(Value::Null)
    }

    /// Evaluate a compound assignment (+=, -=, *=, /=, %=)
    fn eval_compound_assign(&mut self, compound: &CompoundAssign) -> Result<Value, RuntimeError> {
        // Get current value
        let current = match &compound.target {
            AssignTarget::Name(id) => self.get_variable(&id.name, compound.span)?,
            AssignTarget::Index {
                target,
                index,
                span,
            } => {
                let arr_val = self.eval_expr(target.as_ref())?;
                let idx_val = self.eval_expr(index.as_ref())?;
                self.get_array_element(arr_val, idx_val, *span)?
            }
        };

        // Get the value to apply
        let value = self.eval_expr(&compound.value)?;

        // Perform the operation
        let result = match (&current, &value) {
            (Value::Number(a), Value::Number(b)) => {
                let res = match compound.op {
                    CompoundOp::AddAssign => a + b,
                    CompoundOp::SubAssign => a - b,
                    CompoundOp::MulAssign => a * b,
                    CompoundOp::DivAssign => {
                        if b == &0.0 {
                            return Err(RuntimeError::DivideByZero {
                                span: compound.span,
                            });
                        }
                        a / b
                    }
                    CompoundOp::ModAssign => {
                        if b == &0.0 {
                            return Err(RuntimeError::DivideByZero {
                                span: compound.span,
                            });
                        }
                        a % b
                    }
                };

                if res.is_nan() || res.is_infinite() {
                    return Err(RuntimeError::InvalidNumericResult {
                        span: compound.span,
                    });
                }

                Value::Number(res)
            }
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "Compound assignment requires numbers".to_string(),
                    span: compound.span,
                })
            }
        };

        // Store the result
        match &compound.target {
            AssignTarget::Name(id) => {
                self.set_variable(&id.name, result, compound.span)?;
            }
            AssignTarget::Index {
                target,
                index,
                span,
            } => {
                let idx_val = self.eval_expr(index.as_ref())?;
                self.assign_at_index(target, idx_val, result, *span)?;
            }
        }

        Ok(Value::Null)
    }

    /// Evaluate an increment (++)
    fn eval_increment(&mut self, inc: &IncrementStmt) -> Result<Value, RuntimeError> {
        // Get current value
        let current = match &inc.target {
            AssignTarget::Name(id) => self.get_variable(&id.name, inc.span)?,
            AssignTarget::Index {
                target,
                index,
                span,
            } => {
                let arr_val = self.eval_expr(target.as_ref())?;
                let idx_val = self.eval_expr(index.as_ref())?;
                self.get_array_element(arr_val, idx_val, *span)?
            }
        };

        // Increment by 1
        let result = match current {
            Value::Number(n) => {
                let res = n + 1.0;
                if res.is_nan() || res.is_infinite() {
                    return Err(RuntimeError::InvalidNumericResult { span: inc.span });
                }
                Value::Number(res)
            }
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "Increment requires number".to_string(),
                    span: inc.span,
                })
            }
        };

        // Store the result
        match &inc.target {
            AssignTarget::Name(id) => {
                self.set_variable(&id.name, result, inc.span)?;
            }
            AssignTarget::Index {
                target,
                index,
                span,
            } => {
                let idx_val = self.eval_expr(index.as_ref())?;
                self.assign_at_index(target, idx_val, result, *span)?;
            }
        }

        Ok(Value::Null)
    }

    /// Evaluate a decrement (--)
    fn eval_decrement(&mut self, dec: &DecrementStmt) -> Result<Value, RuntimeError> {
        // Get current value
        let current = match &dec.target {
            AssignTarget::Name(id) => self.get_variable(&id.name, dec.span)?,
            AssignTarget::Index {
                target,
                index,
                span,
            } => {
                let arr_val = self.eval_expr(target.as_ref())?;
                let idx_val = self.eval_expr(index.as_ref())?;
                self.get_array_element(arr_val, idx_val, *span)?
            }
        };

        // Decrement by 1
        let result = match current {
            Value::Number(n) => {
                let res = n - 1.0;
                if res.is_nan() || res.is_infinite() {
                    return Err(RuntimeError::InvalidNumericResult { span: dec.span });
                }
                Value::Number(res)
            }
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "Decrement requires number".to_string(),
                    span: dec.span,
                })
            }
        };

        // Store the result
        match &dec.target {
            AssignTarget::Name(id) => {
                self.set_variable(&id.name, result, dec.span)?;
            }
            AssignTarget::Index {
                target,
                index,
                span,
            } => {
                let idx_val = self.eval_expr(index.as_ref())?;
                self.assign_at_index(target, idx_val, result, *span)?;
            }
        }

        Ok(Value::Null)
    }

    /// Evaluate an if statement
    fn eval_if(&mut self, if_stmt: &IfStmt) -> Result<Value, RuntimeError> {
        let condition = self.eval_expr(&if_stmt.cond)?;

        if condition.is_truthy() {
            self.eval_block(&if_stmt.then_block)
        } else if let Some(else_block) = &if_stmt.else_block {
            self.eval_block(else_block)
        } else {
            Ok(Value::Null)
        }
    }

    /// Evaluate a while loop
    fn eval_while(&mut self, while_stmt: &WhileStmt) -> Result<Value, RuntimeError> {
        let mut last_value = Value::Null;

        loop {
            let condition = self.eval_expr(&while_stmt.cond)?;

            if !condition.is_truthy() {
                break;
            }

            last_value = self.eval_block(&while_stmt.body)?;

            match self.control_flow {
                ControlFlow::Break => {
                    self.control_flow = ControlFlow::None;
                    break;
                }
                ControlFlow::Continue => {
                    self.control_flow = ControlFlow::None;
                    continue;
                }
                ControlFlow::Return(_) => {
                    // Propagate return up
                    break;
                }
                ControlFlow::None => {}
            }
        }

        Ok(last_value)
    }

    /// Evaluate a for loop
    fn eval_for(&mut self, for_stmt: &ForStmt) -> Result<Value, RuntimeError> {
        // Push new scope for loop variable
        self.push_scope();

        // Initialize loop variable
        self.eval_statement(&for_stmt.init)?;

        let mut last_value = Value::Null;

        loop {
            // Check condition
            let cond_val = self.eval_expr(&for_stmt.cond)?;
            if !cond_val.is_truthy() {
                break;
            }

            // Execute body
            last_value = self.eval_block(&for_stmt.body)?;

            match self.control_flow {
                ControlFlow::Break => {
                    self.control_flow = ControlFlow::None;
                    break;
                }
                ControlFlow::Continue => {
                    self.control_flow = ControlFlow::None;
                    // Continue to step
                }
                ControlFlow::Return(_) => {
                    // Propagate return up
                    break;
                }
                ControlFlow::None => {}
            }

            // Execute step
            self.eval_statement(&for_stmt.step)?;
        }

        self.pop_scope();
        Ok(last_value)
    }

    /// Evaluate a for-in loop
    fn eval_for_in(&mut self, for_in_stmt: &ForInStmt) -> Result<Value, RuntimeError> {
        // Evaluate the iterable expression to get the array
        let iterable = self.eval_expr(&for_in_stmt.iterable)?;

        // Extract array elements
        let elements = match &iterable {
            Value::Array(arr) => arr.iter().cloned().collect::<Vec<_>>(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: format!("for-in requires an array, found {}", iterable.type_name()),
                    span: for_in_stmt.iterable.span(),
                });
            }
        };

        // Push new scope for loop variable
        self.push_scope();

        let mut last_value = Value::Null;

        // Iterate over each element
        for element in elements {
            // Bind loop variable to current element (loop variables are mutable)
            let scope = self.locals.last_mut().unwrap();
            scope.insert(for_in_stmt.variable.name.clone(), (element, true));

            // Execute body
            last_value = self.eval_block(&for_in_stmt.body)?;

            // Handle control flow
            match self.control_flow {
                ControlFlow::Break => {
                    self.control_flow = ControlFlow::None;
                    break;
                }
                ControlFlow::Continue => {
                    self.control_flow = ControlFlow::None;
                    // Continue to next iteration
                }
                ControlFlow::Return(_) => {
                    // Propagate return up
                    break;
                }
                ControlFlow::None => {}
            }
        }

        self.pop_scope();
        Ok(last_value)
    }

    /// Evaluate a return statement
    fn eval_return(&mut self, return_stmt: &ReturnStmt) -> Result<Value, RuntimeError> {
        let value = if let Some(expr) = &return_stmt.value {
            self.eval_expr(expr)?
        } else {
            Value::Null
        };

        self.control_flow = ControlFlow::Return(value.clone());
        Ok(value)
    }

    /// Evaluate a block
    pub(super) fn eval_block(&mut self, block: &Block) -> Result<Value, RuntimeError> {
        self.push_scope();

        let mut last_value = Value::Null;

        for stmt in &block.statements {
            last_value = self.eval_statement(stmt)?;

            // Check for control flow
            if self.control_flow != ControlFlow::None {
                break;
            }
        }

        self.pop_scope();
        Ok(last_value)
    }
}
