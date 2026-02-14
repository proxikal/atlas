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
    /// User-defined function bodies (accessed via Value::Function references)
    pub(super) function_bodies: HashMap<String, UserFunction>,
    /// Current control flow state
    pub(super) control_flow: ControlFlow,
    /// Monomorphizer for generic functions (tracks type substitutions)
    #[allow(dead_code)] // Will be used when generic runtime support is fully integrated
    pub(super) monomorphizer: crate::typechecker::generics::Monomorphizer,
}

impl Interpreter {
    /// Create a new interpreter
    pub fn new() -> Self {
        let mut interpreter = Self {
            globals: HashMap::new(),
            locals: vec![HashMap::new()],
            function_bodies: HashMap::new(),
            control_flow: ControlFlow::None,
            monomorphizer: crate::typechecker::generics::Monomorphizer::new(),
        };

        // Register builtin functions in globals
        // Core builtins
        interpreter.register_builtin("print", 1);
        interpreter.register_builtin("len", 1);
        interpreter.register_builtin("str", 1);

        // String functions
        interpreter.register_builtin("split", 2);
        interpreter.register_builtin("join", 2);
        interpreter.register_builtin("trim", 1);
        interpreter.register_builtin("trimStart", 1);
        interpreter.register_builtin("trimEnd", 1);
        interpreter.register_builtin("indexOf", 2);
        interpreter.register_builtin("lastIndexOf", 2);
        interpreter.register_builtin("includes", 2);
        interpreter.register_builtin("startsWith", 2);
        interpreter.register_builtin("endsWith", 2);
        interpreter.register_builtin("substring", 3);
        interpreter.register_builtin("charAt", 2);
        interpreter.register_builtin("toUpperCase", 1);
        interpreter.register_builtin("toLowerCase", 1);
        interpreter.register_builtin("repeat", 2);
        interpreter.register_builtin("replace", 3);
        interpreter.register_builtin("padStart", 3);
        interpreter.register_builtin("padEnd", 3);

        interpreter
    }

    /// Register a builtin function in globals
    fn register_builtin(&mut self, name: &str, arity: usize) {
        let func_value = Value::Function(FunctionRef {
            name: name.to_string(),
            arity,
            bytecode_offset: 0, // Not used in interpreter
            local_count: 0,     // Not used in interpreter
        });
        self.globals.insert(name.to_string(), func_value);
    }

    /// Evaluate a program
    pub fn eval(&mut self, program: &Program) -> Result<Value, RuntimeError> {
        let mut last_value = Value::Null;

        for item in &program.items {
            match item {
                Item::Function(func) => {
                    // Store user-defined function body
                    self.function_bodies.insert(
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
                        local_count: 0,     // Not used in interpreter
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
    pub(super) fn get_variable(
        &self,
        name: &str,
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        // Check locals (innermost to outermost)
        for scope in self.locals.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Ok(value.clone());
            }
        }

        // Check globals
        if let Some(value) = self.globals.get(name) {
            return Ok(value.clone());
        }

        // Check builtins - return a function value for builtin functions
        if crate::stdlib::is_builtin(name) {
            return Ok(Value::Function(crate::value::FunctionRef {
                name: name.to_string(),
                arity: 0, // Arity not used for builtins
                bytecode_offset: 0,
                local_count: 0,
            }));
        }

        Err(RuntimeError::UndefinedVariable {
            name: name.to_string(),
            span,
        })
    }

    /// Set a variable value
    pub(super) fn set_variable(
        &mut self,
        name: &str,
        value: Value,
        span: crate::span::Span,
    ) -> Result<(), RuntimeError> {
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

        Err(RuntimeError::UndefinedVariable {
            name: name.to_string(),
            span,
        })
    }

    /// Get an array element by index
    pub(super) fn get_array_element(
        &self,
        arr: Value,
        idx: Value,
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if let Value::Array(arr) = arr {
            if let Value::Number(n) = idx {
                let index_val = n as i64;
                if n.fract() != 0.0 || n < 0.0 {
                    return Err(RuntimeError::InvalidIndex { span });
                }

                let borrowed = arr.borrow();
                if index_val >= 0 && (index_val as usize) < borrowed.len() {
                    Ok(borrowed[index_val as usize].clone())
                } else {
                    Err(RuntimeError::OutOfBounds { span })
                }
            } else {
                Err(RuntimeError::InvalidIndex { span })
            }
        } else {
            Err(RuntimeError::TypeError {
                msg: "Cannot index non-array".to_string(),
                span,
            })
        }
    }

    /// Set an array element by index
    pub(super) fn set_array_element(
        &self,
        arr: Value,
        idx: Value,
        value: Value,
        span: crate::span::Span,
    ) -> Result<(), RuntimeError> {
        if let Value::Array(arr) = arr {
            if let Value::Number(n) = idx {
                let index_val = n as i64;
                if n.fract() != 0.0 || n < 0.0 {
                    return Err(RuntimeError::InvalidIndex { span });
                }

                let mut borrowed = arr.borrow_mut();
                if index_val >= 0 && (index_val as usize) < borrowed.len() {
                    borrowed[index_val as usize] = value;
                    Ok(())
                } else {
                    Err(RuntimeError::OutOfBounds { span })
                }
            } else {
                Err(RuntimeError::InvalidIndex { span })
            }
        } else {
            Err(RuntimeError::TypeError {
                msg: "Cannot index non-array".to_string(),
                span,
            })
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
        assert_eq!(
            interp.eval_literal(&Literal::Number(42.0)),
            Value::Number(42.0)
        );
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
