//! AST interpreter (tree-walking)
//!
//! Direct AST evaluation with environment-based variable storage.
//! Supports:
//! - Expression evaluation (literals, binary/unary ops, calls, indexing)
//! - Statement execution (declarations, assignments, control flow)
//! - Function calls and stack frames
//! - Block scoping with shadowing

mod expr;
mod stmt;

use crate::ast::{Block, Item, Param, Program};
use crate::value::{FunctionRef, RuntimeError, Value};
use std::collections::HashMap;

/// Control flow signal for handling break, continue, and return
#[derive(Debug, Clone, PartialEq)]
pub(super) enum ControlFlow {
    None,
    Break,
    Continue,
    Return(Value),
}

/// User-defined function
#[derive(Debug, Clone)]
pub(super) struct UserFunction {
    pub(super) name: String,
    pub(super) params: Vec<Param>,
    pub(super) body: Block,
}

/// Interpreter state
pub struct Interpreter {
    /// Global variables
    pub(super) globals: HashMap<String, Value>,
    /// Local scopes (stack of environments)
    pub(super) locals: Vec<HashMap<String, Value>>,
    /// User-defined functions
    pub(super) functions: HashMap<String, UserFunction>,
    /// Current control flow state
    pub(super) control_flow: ControlFlow,
}

impl Interpreter {
    /// Create a new interpreter
    pub fn new() -> Self {
        Self {
            globals: HashMap::new(),
            locals: vec![HashMap::new()],
            functions: HashMap::new(),
            control_flow: ControlFlow::None,
        }
    }

    /// Evaluate a program
    pub fn eval(&mut self, program: &Program) -> Result<Value, RuntimeError> {
        let mut last_value = Value::Null;

        for item in &program.items {
            match item {
                Item::Function(func) => {
                    // Store user-defined function
                    self.functions.insert(
                        func.name.name.clone(),
                        UserFunction {
                            name: func.name.name.clone(),
                            params: func.params.clone(),
                            body: func.body.clone(),
                        },
                    );

                    // Also store as a value for reference
                    let func_value = Value::Function(FunctionRef {
                        name: func.name.name.clone(),
                        arity: func.params.len(),
                        bytecode_offset: 0, // Not used in interpreter
                    });
                    self.globals.insert(func.name.name.clone(), func_value);
                }
                Item::Statement(stmt) => {
                    last_value = self.eval_statement(stmt)?;

                    // Check for early return at top level
                    if let ControlFlow::Return(val) = &self.control_flow {
                        last_value = val.clone();
                        self.control_flow = ControlFlow::None;
                        break;
                    }
                }
            }
        }

        Ok(last_value)
    }

    /// Get a variable value
    pub(super) fn get_variable(&self, name: &str) -> Result<Value, RuntimeError> {
        // Check locals (innermost to outermost)
        for scope in self.locals.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Ok(value.clone());
            }
        }

        // Check globals
        self.globals
            .get(name)
            .cloned()
            .ok_or_else(|| RuntimeError::UndefinedVariable(name.to_string()))
    }

    /// Set a variable value
    pub(super) fn set_variable(&mut self, name: &str, value: Value) -> Result<(), RuntimeError> {
        // Find in locals (innermost to outermost)
        for scope in self.locals.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), value);
                return Ok(());
            }
        }

        // Check globals
        if self.globals.contains_key(name) {
            self.globals.insert(name.to_string(), value);
            return Ok(());
        }

        Err(RuntimeError::UndefinedVariable(name.to_string()))
    }

    /// Get an array element by index
    pub(super) fn get_array_element(&self, arr: Value, idx: Value) -> Result<Value, RuntimeError> {
        if let Value::Array(arr) = arr {
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
        } else {
            Err(RuntimeError::TypeError("Cannot index non-array".to_string()))
        }
    }

    /// Set an array element by index
    pub(super) fn set_array_element(
        &self,
        arr: Value,
        idx: Value,
        value: Value,
    ) -> Result<(), RuntimeError> {
        if let Value::Array(arr) = arr {
            if let Value::Number(n) = idx {
                let index_val = n as i64;
                if n.fract() != 0.0 || n < 0.0 {
                    return Err(RuntimeError::InvalidIndex);
                }

                let mut borrowed = arr.borrow_mut();
                if index_val >= 0 && (index_val as usize) < borrowed.len() {
                    borrowed[index_val as usize] = value;
                    Ok(())
                } else {
                    Err(RuntimeError::OutOfBounds)
                }
            } else {
                Err(RuntimeError::InvalidIndex)
            }
        } else {
            Err(RuntimeError::TypeError("Cannot index non-array".to_string()))
        }
    }

    /// Push a new scope
    pub(super) fn push_scope(&mut self) {
        self.locals.push(HashMap::new());
    }

    /// Pop the current scope
    pub(super) fn pop_scope(&mut self) {
        self.locals.pop();
    }

    /// Define a global variable (for testing/REPL)
    pub fn define_global(&mut self, name: String, value: Value) {
        self.globals.insert(name, value);
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Literal;

    #[test]
    fn test_interpreter_creation() {
        let mut interp = Interpreter::new();
        interp.define_global("x".to_string(), Value::Number(42.0));
        assert!(interp.globals.contains_key("x"));
    }

    #[test]
    fn test_eval_literal() {
        let interp = Interpreter::new();
        assert_eq!(interp.eval_literal(&Literal::Number(42.0)), Value::Number(42.0));
        assert_eq!(interp.eval_literal(&Literal::Bool(true)), Value::Bool(true));
        assert_eq!(interp.eval_literal(&Literal::Null), Value::Null);
    }

    #[test]
    fn test_scope_management() {
        let mut interp = Interpreter::new();
        assert_eq!(interp.locals.len(), 1);

        interp.push_scope();
        assert_eq!(interp.locals.len(), 2);

        interp.pop_scope();
        assert_eq!(interp.locals.len(), 1);
    }
}
