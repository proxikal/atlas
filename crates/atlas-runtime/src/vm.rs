//! Stack-based virtual machine

use crate::bytecode::Bytecode;
use crate::value::{RuntimeError, Value};

/// Virtual machine state
pub struct VM {
    /// Value stack
    stack: Vec<Value>,
}

impl VM {
    /// Create a new VM
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    /// Execute bytecode
    pub fn execute(&mut self, _bytecode: &Bytecode) -> Result<Value, RuntimeError> {
        // Placeholder implementation
        Ok(Value::Null)
    }

    /// Push a value onto the stack
    pub fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    /// Pop a value from the stack
    pub fn pop(&mut self) -> Result<Value, RuntimeError> {
        self.stack
            .pop()
            .ok_or_else(|| RuntimeError::TypeError("Stack underflow".to_string()))
    }
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vm_stack() {
        let mut vm = VM::new();
        vm.push(Value::Number(42.0));
        let val = vm.pop().unwrap();
        assert_eq!(val, Value::Number(42.0));
    }
}
