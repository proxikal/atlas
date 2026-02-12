//! AST interpreter (tree-walking)

use crate::ast::Program;
use crate::value::{RuntimeError, Value};
use std::collections::HashMap;

/// Interpreter state
pub struct Interpreter {
    /// Global environment
    globals: HashMap<String, Value>,
}

impl Interpreter {
    /// Create a new interpreter
    pub fn new() -> Self {
        Self {
            globals: HashMap::new(),
        }
    }

    /// Evaluate a program
    pub fn eval(&mut self, _program: &Program) -> Result<Value, RuntimeError> {
        // Placeholder implementation
        Ok(Value::Null)
    }

    /// Define a global variable
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

    #[test]
    fn test_interpreter_creation() {
        let mut interp = Interpreter::new();
        interp.define_global("x".to_string(), Value::Number(42.0));
        assert!(interp.globals.contains_key("x"));
    }
}
