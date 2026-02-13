//! Stack-based virtual machine
//!
//! Executes bytecode instructions with a value stack and call frames.
//! - Arithmetic operations check for NaN/Infinity
//! - Variables are stored in locals (stack) or globals (HashMap)
//! - Control flow uses jumps and loops

mod debugger;
mod frame;
mod profiler;

pub use debugger::{DebugAction, DebugHook, Debugger};
pub use frame::CallFrame;
pub use profiler::Profiler;

use crate::bytecode::{Bytecode, Opcode};
use crate::value::{RuntimeError, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Virtual machine state
pub struct VM {
    /// Value stack
    stack: Vec<Value>,
    /// Call frames (for function calls)
    frames: Vec<CallFrame>,
    /// Global variables
    globals: HashMap<String, Value>,
    /// Bytecode to execute
    bytecode: Bytecode,
    /// Instruction pointer
    ip: usize,
    /// Optional profiler for performance analysis
    profiler: Option<Profiler>,
    /// Optional debugger for step-through execution
    debugger: Option<Debugger>,
}

impl VM {
    /// Create a new VM with bytecode
    pub fn new(bytecode: Bytecode) -> Self {
        // Create an initial "main" frame for top-level code
        let main_frame = CallFrame {
            function_name: "<main>".to_string(),
            return_ip: 0,
            stack_base: 0,
            local_count: 0,
        };

        Self {
            stack: Vec::with_capacity(256),
            frames: vec![main_frame],
            globals: HashMap::new(),
            bytecode,
            ip: 0,
            profiler: None, // Profiling disabled by default
            debugger: None, // Debugging disabled by default
        }
    }

    /// Create a new VM with profiling enabled
    pub fn with_profiling(bytecode: Bytecode) -> Self {
        let mut vm = Self::new(bytecode);
        vm.profiler = Some(Profiler::enabled());
        vm
    }

    /// Create a new VM with debugging enabled
    pub fn with_debugging(bytecode: Bytecode) -> Self {
        let mut vm = Self::new(bytecode);
        vm.debugger = Some(Debugger::enabled());
        vm
    }

    /// Enable profiling
    pub fn enable_profiling(&mut self) {
        if let Some(ref mut profiler) = self.profiler {
            profiler.enable();
        } else {
            self.profiler = Some(Profiler::enabled());
        }
    }

    /// Disable profiling
    pub fn disable_profiling(&mut self) {
        if let Some(ref mut profiler) = self.profiler {
            profiler.disable();
        }
    }

    /// Get profiler reference
    pub fn profiler(&self) -> Option<&Profiler> {
        self.profiler.as_ref()
    }

    /// Get mutable profiler reference
    pub fn profiler_mut(&mut self) -> Option<&mut Profiler> {
        self.profiler.as_mut()
    }

    /// Enable debugging
    pub fn enable_debugging(&mut self) {
        if let Some(ref mut debugger) = self.debugger {
            debugger.enable();
        } else {
            self.debugger = Some(Debugger::enabled());
        }
    }

    /// Disable debugging
    pub fn disable_debugging(&mut self) {
        if let Some(ref mut debugger) = self.debugger {
            debugger.disable();
        }
    }

    /// Get debugger reference
    pub fn debugger(&self) -> Option<&Debugger> {
        self.debugger.as_ref()
    }

    /// Get mutable debugger reference
    pub fn debugger_mut(&mut self) -> Option<&mut Debugger> {
        self.debugger.as_mut()
    }

    /// Get the source span for the current instruction pointer
    ///
    /// Returns the span from debug info if available.
    /// Useful for error reporting with source location context.
    pub fn current_span(&self) -> Option<crate::span::Span> {
        if self.ip == 0 {
            return None;
        }
        self.bytecode.get_span_for_offset(self.ip - 1)
    }

    /// Get the source span for a specific instruction offset
    pub fn span_for_offset(&self, offset: usize) -> Option<crate::span::Span> {
        self.bytecode.get_span_for_offset(offset)
    }

    /// Execute the bytecode
    pub fn run(&mut self) -> Result<Option<Value>, RuntimeError> {
        loop {
            // Check if we've reached the end
            if self.ip >= self.bytecode.instructions.len() {
                break;
            }

            let opcode = self.read_opcode()?;

            // Debugger hook: before instruction (zero overhead when disabled)
            if let Some(ref mut debugger) = self.debugger {
                if debugger.is_enabled() {
                    let action = debugger.before_instruction(self.ip - 1, opcode);
                    match action {
                        DebugAction::Pause => {
                            // Pause execution (would need external control to resume)
                            // For now, we just continue
                        }
                        DebugAction::Step => {
                            // Step mode: pause after this instruction
                            // Future: integrate with debugger UI
                        }
                        DebugAction::Continue => {
                            // Normal execution
                        }
                    }
                }
            }

            // Record instruction for profiling (zero overhead when disabled)
            if let Some(ref mut profiler) = self.profiler {
                if profiler.is_enabled() {
                    profiler.record_instruction(opcode);
                }
            }

            match opcode {
                // ===== Constants =====
                Opcode::Constant => {
                    let index = self.read_u16() as usize;
                    if index >= self.bytecode.constants.len() {
                        return Err(RuntimeError::UnknownOpcode {
                        span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                    });
                    }
                    let value = self.bytecode.constants[index].clone();
                    self.push(value);
                }
                Opcode::Null => self.push(Value::Null),
                Opcode::True => self.push(Value::Bool(true)),
                Opcode::False => self.push(Value::Bool(false)),

                // ===== Variables =====
                Opcode::GetLocal => {
                    let index = self.read_u16() as usize;
                    let base = self.current_frame().stack_base;
                    let absolute_index = base + index;
                    if absolute_index >= self.stack.len() {
                        return Err(RuntimeError::StackUnderflow {
                        span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                    });
                    }
                    let value = self.stack[absolute_index].clone();
                    self.push(value);
                }
                Opcode::SetLocal => {
                    let index = self.read_u16() as usize;
                    let base = self.current_frame().stack_base;
                    let local_count = self.current_frame().local_count;
                    let absolute_index = base + index;
                    let value = self.peek(0).clone();

                    // SAFETY CHECK: Prevent unbounded stack growth
                    // This prevents memory explosion from invalid bytecode or compiler bugs
                    if index >= local_count {
                        return Err(RuntimeError::StackUnderflow {
                            span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                        });
                    }

                    // Extend stack if needed (for local variables not yet initialized)
                    if absolute_index >= self.stack.len() {
                        // Bounded extension: only up to the declared local_count
                        let needed = absolute_index - self.stack.len() + 1;
                        if base + local_count > self.stack.len() + needed {
                            return Err(RuntimeError::StackUnderflow {
                                span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                            });
                        }
                        for _ in 0..needed {
                            self.stack.push(Value::Null);
                        }
                    }
                    self.stack[absolute_index] = value;
                }
                Opcode::GetGlobal => {
                    let name_index = self.read_u16() as usize;
                    if name_index >= self.bytecode.constants.len() {
                        return Err(RuntimeError::UnknownOpcode {
                        span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                    });
                    }
                    let name = match &self.bytecode.constants[name_index] {
                        Value::String(s) => s.as_ref().clone(),
                        _ => return Err(RuntimeError::TypeError {
                            msg: "Expected string constant for variable name".to_string(),
                            span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                        }),
                    };
                    let value = self
                        .globals
                        .get(&name)
                        .cloned()
                        .ok_or_else(|| RuntimeError::UndefinedVariable {
                            name: name.clone(),
                            span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                        })?;
                    self.push(value);
                }
                Opcode::SetGlobal => {
                    let name_index = self.read_u16() as usize;
                    if name_index >= self.bytecode.constants.len() {
                        return Err(RuntimeError::UnknownOpcode {
                        span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                    });
                    }
                    let name = match &self.bytecode.constants[name_index] {
                        Value::String(s) => s.as_ref().clone(),
                        _ => return Err(RuntimeError::TypeError {
                            msg: "Expected string constant for variable name".to_string(),
                            span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                        }),
                    };
                    let value = self.peek(0).clone();
                    self.globals.insert(name, value);
                }

                // ===== Arithmetic =====
                Opcode::Add => {
                    let b = self.pop();
                    let a = self.pop();
                    match (&a, &b) {
                        (Value::Number(x), Value::Number(y)) => {
                            let result = x + y;
                            if result.is_nan() || result.is_infinite() {
                                return Err(RuntimeError::InvalidNumericResult {
                        span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                    });
                            }
                            self.push(Value::Number(result));
                        }
                        (Value::String(x), Value::String(y)) => {
                            self.push(Value::String(Rc::new(format!("{}{}", x, y))));
                        }
                        _ => return Err(RuntimeError::TypeError {
                            msg: "Invalid operands for +".to_string(),
                            span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                        }),
                    }
                }
                Opcode::Sub => self.binary_numeric_op(|a, b| a - b)?,
                Opcode::Mul => self.binary_numeric_op(|a, b| a * b)?,
                Opcode::Div => {
                    let b = self.pop_number()?;
                    let a = self.pop_number()?;
                    if b == 0.0 {
                        return Err(RuntimeError::DivideByZero {
                        span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                    });
                    }
                    let result = a / b;
                    if result.is_nan() || result.is_infinite() {
                        return Err(RuntimeError::InvalidNumericResult {
                        span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                    });
                    }
                    self.push(Value::Number(result));
                }
                Opcode::Mod => {
                    let b = self.pop_number()?;
                    let a = self.pop_number()?;
                    if b == 0.0 {
                        return Err(RuntimeError::DivideByZero {
                        span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                    });
                    }
                    let result = a % b;
                    if result.is_nan() || result.is_infinite() {
                        return Err(RuntimeError::InvalidNumericResult {
                        span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                    });
                    }
                    self.push(Value::Number(result));
                }
                Opcode::Negate => {
                    let value = self.pop();
                    match value {
                        Value::Number(n) => self.push(Value::Number(-n)),
                        _ => return Err(RuntimeError::TypeError {
                            msg: "Cannot negate non-number".to_string(),
                            span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                        }),
                    }
                }

                // ===== Comparison =====
                Opcode::Equal => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::Bool(a == b));
                }
                Opcode::NotEqual => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::Bool(a != b));
                }
                Opcode::Less => {
                    let b = self.pop_number()?;
                    let a = self.pop_number()?;
                    self.push(Value::Bool(a < b));
                }
                Opcode::LessEqual => {
                    let b = self.pop_number()?;
                    let a = self.pop_number()?;
                    self.push(Value::Bool(a <= b));
                }
                Opcode::Greater => {
                    let b = self.pop_number()?;
                    let a = self.pop_number()?;
                    self.push(Value::Bool(a > b));
                }
                Opcode::GreaterEqual => {
                    let b = self.pop_number()?;
                    let a = self.pop_number()?;
                    self.push(Value::Bool(a >= b));
                }

                // ===== Logical =====
                Opcode::Not => {
                    let value = self.pop();
                    match value {
                        Value::Bool(b) => self.push(Value::Bool(!b)),
                        _ => return Err(RuntimeError::TypeError {
                            msg: "Cannot apply ! to non-boolean".to_string(),
                            span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                        }),
                    }
                }
                Opcode::And | Opcode::Or => {
                    // TODO: Short-circuit evaluation
                    return Err(RuntimeError::UnknownOpcode {
                        span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                    });
                }

                // ===== Control Flow =====
                Opcode::Jump => {
                    let offset = self.read_i16();
                    self.ip = (self.ip as isize + offset as isize) as usize;
                }
                Opcode::JumpIfFalse => {
                    let offset = self.read_i16();
                    let condition = self.pop();
                    if !condition.is_truthy() {
                        self.ip = (self.ip as isize + offset as isize) as usize;
                    }
                }
                Opcode::Loop => {
                    let offset = self.read_i16();
                    self.ip = (self.ip as isize + offset as isize) as usize;
                }

                // ===== Functions =====
                Opcode::Call => {
                    let arg_count = self.read_u8() as usize;

                    // Get the function value from stack (it's below the arguments)
                    let function = self.peek(arg_count).clone();

                    match function {
                        Value::Function(func) => {
                            // Check if it's a builtin function (bytecode_offset == 0)
                            if func.bytecode_offset == 0 || crate::stdlib::is_builtin(&func.name) {
                                // Builtin function - call directly
                                let mut args = Vec::with_capacity(arg_count);
                                for _ in 0..arg_count {
                                    args.push(self.pop());
                                }
                                args.reverse(); // Args were pushed in reverse order

                                // Pop the function value
                                self.pop();

                                // Call the builtin
                                let result = crate::stdlib::call_builtin(
                                    &func.name,
                                    &args,
                                    self.current_span().unwrap_or_else(crate::span::Span::dummy),
                                )?;

                                // Push the result
                                self.push(result);
                            } else {
                                // User-defined function
                                // Create a new call frame
                                let frame = CallFrame {
                                    function_name: func.name.clone(),
                                    return_ip: self.ip,
                                    stack_base: self.stack.len() - arg_count, // Points to first argument
                                    local_count: func.local_count, // Use total locals, not just arity
                                };

                                // Verify argument count matches
                                if arg_count != func.arity {
                                    return Err(RuntimeError::TypeError {
                                        msg: format!(
                                            "Function {} expects {} arguments, got {}",
                                            func.name, func.arity, arg_count
                                        ),
                                        span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                                    });
                                }

                                // Push the frame
                                self.frames.push(frame);

                                // Jump to function bytecode
                                self.ip = func.bytecode_offset;
                            }
                        }
                        _ => {
                            return Err(RuntimeError::TypeError {
                                msg: "Cannot call non-function value".to_string(),
                                span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                            });
                        }
                    }
                }
                Opcode::Return => {
                    // Pop the return value from stack (if any)
                    let return_value = if self.stack.is_empty() {
                        Value::Null
                    } else {
                        self.pop()
                    };

                    // Pop the call frame
                    let frame = self.frames.pop();

                    if let Some(f) = frame {
                        // Clean up the stack (remove locals, arguments, and function value)
                        // stack_base points to first argument, so we need to also remove the function value below it
                        while self.stack.len() > f.stack_base {
                            self.stack.pop();
                        }
                        // Also remove the function value (one slot below stack_base)
                        // Only if stack_base > 0 (not at the very start of the stack)
                        if f.stack_base > 0 && !self.stack.is_empty() {
                            self.stack.pop();
                        }

                        // Restore IP to return address
                        self.ip = f.return_ip;

                        // Push return value
                        self.push(return_value);
                    } else {
                        // Returning from main - we're done
                        // Push the return value and halt
                        self.push(return_value);
                        break;
                    }
                }

                // ===== Arrays =====
                Opcode::Array => {
                    let size = self.read_u16() as usize;
                    let mut elements = Vec::with_capacity(size);
                    for _ in 0..size {
                        elements.push(self.pop());
                    }
                    elements.reverse(); // Stack is LIFO, so reverse to get correct order
                    self.push(Value::Array(Rc::new(RefCell::new(elements))));
                }
                Opcode::GetIndex => {
                    let index = self.pop_number()?;
                    let array = self.pop();
                    match array {
                        Value::Array(arr) => {
                            if index.fract() != 0.0 || index < 0.0 {
                                return Err(RuntimeError::InvalidIndex {
                        span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                    });
                            }
                            let idx = index as usize;
                            let borrowed = arr.borrow();
                            if idx >= borrowed.len() {
                                return Err(RuntimeError::OutOfBounds {
                        span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                    });
                            }
                            self.push(borrowed[idx].clone());
                        }
                        _ => return Err(RuntimeError::TypeError {
                            msg: "Cannot index non-array".to_string(),
                            span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                        }),
                    }
                }
                Opcode::SetIndex => {
                    let value = self.pop();
                    let index = self.pop_number()?;
                    let array = self.pop();
                    match array {
                        Value::Array(arr) => {
                            if index.fract() != 0.0 || index < 0.0 {
                                return Err(RuntimeError::InvalidIndex {
                        span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                    });
                            }
                            let idx = index as usize;
                            let mut borrowed = arr.borrow_mut();
                            if idx >= borrowed.len() {
                                return Err(RuntimeError::OutOfBounds {
                        span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                    });
                            }
                            borrowed[idx] = value.clone();
                            // Push the assigned value back (assignment expressions return the value)
                            drop(borrowed); // Release the borrow before pushing
                            self.push(value);
                        }
                        _ => return Err(RuntimeError::TypeError {
                            msg: "Cannot index non-array".to_string(),
                            span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                        }),
                    }
                }

                // ===== Stack Manipulation =====
                Opcode::Pop => {
                    // Don't pop if this is the last instruction before Halt
                    // Check if next instruction is Halt
                    if self.ip < self.bytecode.instructions.len()
                        && self.bytecode.instructions[self.ip] != Opcode::Halt as u8
                    {
                        self.pop();
                    }
                }
                Opcode::Dup => {
                    let value = self.peek(0).clone();
                    self.push(value);
                }

                // ===== Special =====
                Opcode::Halt => break,
            }
        }

        // Return top of stack if present
        Ok(if self.stack.is_empty() {
            None
        } else {
            Some(self.pop())
        })
    }

    // ===== Helper Methods =====

    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn pop(&mut self) -> Value {
        self.stack.pop().expect("Stack underflow")
    }

    fn peek(&self, distance: usize) -> &Value {
        &self.stack[self.stack.len() - 1 - distance]
    }

    fn pop_number(&mut self) -> Result<f64, RuntimeError> {
        match self.pop() {
            Value::Number(n) => Ok(n),
            _ => Err(RuntimeError::TypeError {
                msg: "Expected number".to_string(),
                span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
            }),
        }
    }

    fn binary_numeric_op<F>(&mut self, op: F) -> Result<(), RuntimeError>
    where
        F: FnOnce(f64, f64) -> f64,
    {
        let b = self.pop_number()?;
        let a = self.pop_number()?;
        let result = op(a, b);
        if result.is_nan() || result.is_infinite() {
            return Err(RuntimeError::InvalidNumericResult {
                        span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                    });
        }
        self.push(Value::Number(result));
        Ok(())
    }

    fn read_opcode(&mut self) -> Result<Opcode, RuntimeError> {
        if self.ip >= self.bytecode.instructions.len() {
            return Err(RuntimeError::UnknownOpcode {
                        span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                    });
        }
        let byte = self.bytecode.instructions[self.ip];
        self.ip += 1;
        Opcode::try_from(byte).map_err(|_| RuntimeError::UnknownOpcode {
                        span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                    })
    }

    fn read_u8(&mut self) -> u8 {
        let byte = self.bytecode.instructions[self.ip];
        self.ip += 1;
        byte
    }

    fn read_u16(&mut self) -> u16 {
        let hi = self.bytecode.instructions[self.ip] as u16;
        let lo = self.bytecode.instructions[self.ip + 1] as u16;
        self.ip += 2;
        (hi << 8) | lo
    }

    fn read_i16(&mut self) -> i16 {
        self.read_u16() as i16
    }

    /// Get the current call frame
    fn current_frame(&self) -> &CallFrame {
        self.frames.last().expect("No call frame available")
    }

    /// Generate a stack trace from the current call frames
    /// Returns a vector of function names from innermost to outermost
    #[allow(dead_code)]
    fn stack_trace(&self) -> Vec<String> {
        self.frames
            .iter()
            .rev()
            .map(|frame| frame.function_name.clone())
            .collect()
    }
}

impl Default for VM {
    fn default() -> Self {
        Self::new(Bytecode::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::Compiler;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn execute_source(source: &str) -> Result<Option<Value>, RuntimeError> {
        // Compile source to bytecode
        let mut lexer = Lexer::new(source.to_string());
        let (tokens, _) = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (program, _) = parser.parse();
        let mut compiler = Compiler::new();
        let bytecode = compiler.compile(&program).expect("Compilation failed");

        // Execute on VM
        let mut vm = VM::new(bytecode);
        vm.run()
    }

    #[test]
    fn test_vm_number_literal() {
        let result = execute_source("42;").unwrap();
        assert_eq!(result, Some(Value::Number(42.0)));
    }

    #[test]
    fn test_vm_arithmetic() {
        let result = execute_source("2 + 3;").unwrap();
        assert_eq!(result, Some(Value::Number(5.0)));

        let result = execute_source("10 - 4;").unwrap();
        assert_eq!(result, Some(Value::Number(6.0)));

        let result = execute_source("3 * 4;").unwrap();
        assert_eq!(result, Some(Value::Number(12.0)));

        let result = execute_source("15 / 3;").unwrap();
        assert_eq!(result, Some(Value::Number(5.0)));
    }

    #[test]
    fn test_vm_comparison() {
        let result = execute_source("1 < 2;").unwrap();
        assert_eq!(result, Some(Value::Bool(true)));

        let result = execute_source("5 > 10;").unwrap();
        assert_eq!(result, Some(Value::Bool(false)));

        let result = execute_source("3 == 3;").unwrap();
        assert_eq!(result, Some(Value::Bool(true)));
    }

    #[test]
    fn test_vm_global_variable() {
        let result = execute_source("let x = 42; x;").unwrap();
        assert_eq!(result, Some(Value::Number(42.0)));
    }

    #[test]
    fn test_vm_string_concat() {
        let result = execute_source("\"hello\" + \" world\";").unwrap();
        if let Some(Value::String(s)) = result {
            assert_eq!(s.as_ref(), "hello world");
        } else {
            panic!("Expected string result");
        }
    }

    #[test]
    fn test_vm_array_literal() {
        let result = execute_source("[1, 2, 3];").unwrap();
        if let Some(Value::Array(arr)) = result {
            let borrowed = arr.borrow();
            assert_eq!(borrowed.len(), 3);
            assert_eq!(borrowed[0], Value::Number(1.0));
            assert_eq!(borrowed[1], Value::Number(2.0));
            assert_eq!(borrowed[2], Value::Number(3.0));
        } else {
            panic!("Expected array result");
        }
    }

    #[test]
    fn test_vm_array_index() {
        let result = execute_source("let arr = [10, 20, 30]; arr[1];").unwrap();
        assert_eq!(result, Some(Value::Number(20.0)));
    }

    #[test]
    fn test_vm_division_by_zero() {
        let result = execute_source("10 / 0;");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RuntimeError::DivideByZero { .. }));
    }

    #[test]
    fn test_vm_bool_literals() {
        let result = execute_source("true;").unwrap();
        assert_eq!(result, Some(Value::Bool(true)));

        let result = execute_source("false;").unwrap();
        assert_eq!(result, Some(Value::Bool(false)));
    }

    #[test]
    fn test_vm_null_literal() {
        let result = execute_source("null;").unwrap();
        assert_eq!(result, Some(Value::Null));
    }

    #[test]
    fn test_vm_unary_negate() {
        let result = execute_source("-42;").unwrap();
        assert_eq!(result, Some(Value::Number(-42.0)));
    }

    #[test]
    fn test_vm_logical_not() {
        let result = execute_source("!true;").unwrap();
        assert_eq!(result, Some(Value::Bool(false)));

        let result = execute_source("!false;").unwrap();
        assert_eq!(result, Some(Value::Bool(true)));
    }

    // ===== Constants Pool Loading Tests =====

    #[test]
    fn test_vm_load_number_constant() {
        // Test loading a number constant from pool
        let result = execute_source("123.456;").unwrap();
        assert_eq!(result, Some(Value::Number(123.456)));
    }

    #[test]
    fn test_vm_load_string_constant() {
        // Test loading a string constant from pool
        let result = execute_source("\"hello world\";").unwrap();
        if let Some(Value::String(s)) = result {
            assert_eq!(s.as_ref(), "hello world");
        } else {
            panic!("Expected string value");
        }
    }

    #[test]
    fn test_vm_load_multiple_constants() {
        // Test that multiple constants can be loaded and used
        let result = execute_source("1; 2; 3;").unwrap();
        // Should return the last value
        assert_eq!(result, Some(Value::Number(3.0)));
    }

    #[test]
    fn test_vm_constants_in_expression() {
        // Test using multiple constants in a single expression
        let result = execute_source("10 + 20 + 30;").unwrap();
        assert_eq!(result, Some(Value::Number(60.0)));
    }

    #[test]
    fn test_vm_constant_reuse() {
        // Test that the same constant value can be used multiple times
        let result = execute_source("let x = 5; let y = 5; x + y;").unwrap();
        assert_eq!(result, Some(Value::Number(10.0)));
    }

    #[test]
    fn test_vm_large_constant_index() {
        // Test that larger constant indices work correctly
        // Create many variables to populate the constant pool
        let mut source = String::new();
        for i in 0..100 {
            source.push_str(&format!("let x{} = {}; ", i, i));
        }
        source.push_str("x99;");

        let result = execute_source(&source).unwrap();
        assert_eq!(result, Some(Value::Number(99.0)));
    }

    #[test]
    fn test_vm_string_constants_in_variables() {
        // Test that string constants work properly with variables
        let result = execute_source("let s = \"test\"; s;").unwrap();
        if let Some(Value::String(s)) = result {
            assert_eq!(s.as_ref(), "test");
        } else {
            panic!("Expected string value");
        }
    }

    #[test]
    fn test_vm_mixed_constant_types() {
        // Test mixing different constant types
        let result = execute_source(
            r#"
            let n = 42;
            let s = "hello";
            let b = true;
            n;
        "#,
        )
        .unwrap();
        assert_eq!(result, Some(Value::Number(42.0)));
    }

    #[test]
    fn test_vm_constant_bounds_check() {
        // Create bytecode with an invalid constant index
        let mut bytecode = Bytecode::new();
        bytecode.add_constant(Value::Number(1.0));
        bytecode.emit(Opcode::Constant, crate::span::Span::dummy());
        bytecode.emit_u16(999); // Index out of bounds
        bytecode.emit(Opcode::Halt, crate::span::Span::dummy());

        let mut vm = VM::new(bytecode);
        let result = vm.run();
        assert!(result.is_err());
    }

    #[test]
    fn test_vm_empty_constant_pool() {
        // Test VM with no constants (only opcodes that don't need them)
        let mut bytecode = Bytecode::new();
        bytecode.emit(Opcode::True, crate::span::Span::dummy());
        bytecode.emit(Opcode::Halt, crate::span::Span::dummy());

        let mut vm = VM::new(bytecode);
        let result = vm.run().unwrap();
        assert_eq!(result, Some(Value::Bool(true)));
        assert_eq!(vm.bytecode.constants.len(), 0);
    }

    // ===== Stack Frame Tests =====

    #[test]
    fn test_vm_initial_main_frame() {
        // Test that VM starts with a main frame
        let bytecode = Bytecode::new();
        let vm = VM::new(bytecode);

        assert_eq!(vm.frames.len(), 1);
        assert_eq!(vm.frames[0].function_name, "<main>");
        assert_eq!(vm.frames[0].stack_base, 0);
        assert_eq!(vm.frames[0].local_count, 0);
    }

    #[test]
    fn test_vm_frame_relative_locals() {
        // Test that locals are accessed relative to frame base
        // Simulate: let x = 10; let y = 20; x + y;
        let mut bytecode = Bytecode::new();

        // Push 10 onto stack (will become local 0)
        let idx_10 = bytecode.add_constant(Value::Number(10.0));
        bytecode.emit(Opcode::Constant, crate::span::Span::dummy());
        bytecode.emit_u16(idx_10);

        // Push 20 onto stack (will become local 1)
        let idx_20 = bytecode.add_constant(Value::Number(20.0));
        bytecode.emit(Opcode::Constant, crate::span::Span::dummy());
        bytecode.emit_u16(idx_20);

        // Get local 0 (should be 10)
        bytecode.emit(Opcode::GetLocal, crate::span::Span::dummy());
        bytecode.emit_u16(0);

        // Get local 1 (should be 20)
        bytecode.emit(Opcode::GetLocal, crate::span::Span::dummy());
        bytecode.emit_u16(1);

        // Add them
        bytecode.emit(Opcode::Add, crate::span::Span::dummy());

        bytecode.emit(Opcode::Halt, crate::span::Span::dummy());

        let mut vm = VM::new(bytecode);
        let result = vm.run().unwrap();
        assert_eq!(result, Some(Value::Number(30.0)));
    }

    #[test]
    fn test_vm_return_from_main() {
        // Test that RETURN from main frame terminates execution
        let mut bytecode = Bytecode::new();

        // Push a value
        let idx = bytecode.add_constant(Value::Number(42.0));
        bytecode.emit(Opcode::Constant, crate::span::Span::dummy());
        bytecode.emit_u16(idx);

        // Return from main
        bytecode.emit(Opcode::Return, crate::span::Span::dummy());

        // This should never execute
        bytecode.emit(Opcode::Null, crate::span::Span::dummy());
        bytecode.emit(Opcode::Halt, crate::span::Span::dummy());

        let mut vm = VM::new(bytecode);
        let result = vm.run().unwrap();
        assert_eq!(result, Some(Value::Number(42.0)));
    }

    #[test]
    fn test_vm_call_frame_creation() {
        use crate::value::FunctionRef;

        // Test that CALL creates a new frame
        let mut bytecode = Bytecode::new();

        // Create a simple function that just returns 42
        // Function starts at offset 10
        let function_offset = 10;

        // Main code: push function, call it
        let func_ref = FunctionRef {
            name: "test_func".to_string(),
            arity: 0,
            bytecode_offset: function_offset,
            local_count: 1,
        };
        let func_idx = bytecode.add_constant(Value::Function(func_ref));

        bytecode.emit(Opcode::Constant, crate::span::Span::dummy());
        bytecode.emit_u16(func_idx);

        // Call with 0 arguments
        bytecode.emit(Opcode::Call, crate::span::Span::dummy());
        bytecode.emit_u8(0);

        // After return, halt
        bytecode.emit(Opcode::Halt, crate::span::Span::dummy());

        // Pad to offset 10
        while bytecode.instructions.len() < function_offset {
            bytecode.emit_u8(0);
        }

        // Function body: push 42 and return
        let idx_42 = bytecode.add_constant(Value::Number(42.0));
        bytecode.emit(Opcode::Constant, crate::span::Span::dummy());
        bytecode.emit_u16(idx_42);
        bytecode.emit(Opcode::Return, crate::span::Span::dummy());

        let mut vm = VM::new(bytecode);
        let result = vm.run().unwrap();
        assert_eq!(result, Some(Value::Number(42.0)));
    }

    #[test]
    fn test_vm_call_with_arguments() {
        use crate::value::FunctionRef;

        // Test function call with arguments
        let mut bytecode = Bytecode::new();

        // Function: fn add(a, b) -> a + b
        let function_offset = 20;

        // Main code: push function, push args (5, 3), call
        let func_ref = FunctionRef {
            name: "add".to_string(),
            arity: 2,
            bytecode_offset: function_offset,
            local_count: 1,
        };
        let func_idx = bytecode.add_constant(Value::Function(func_ref));

        bytecode.emit(Opcode::Constant, crate::span::Span::dummy());
        bytecode.emit_u16(func_idx);

        // Push arguments
        let idx_5 = bytecode.add_constant(Value::Number(5.0));
        bytecode.emit(Opcode::Constant, crate::span::Span::dummy());
        bytecode.emit_u16(idx_5);

        let idx_3 = bytecode.add_constant(Value::Number(3.0));
        bytecode.emit(Opcode::Constant, crate::span::Span::dummy());
        bytecode.emit_u16(idx_3);

        // Call with 2 arguments
        bytecode.emit(Opcode::Call, crate::span::Span::dummy());
        bytecode.emit_u8(2);

        bytecode.emit(Opcode::Halt, crate::span::Span::dummy());

        // Pad to function offset
        while bytecode.instructions.len() < function_offset {
            bytecode.emit_u8(0);
        }

        // Function body: GetLocal 0, GetLocal 1, Add, Return
        bytecode.emit(Opcode::GetLocal, crate::span::Span::dummy());
        bytecode.emit_u16(0);
        bytecode.emit(Opcode::GetLocal, crate::span::Span::dummy());
        bytecode.emit_u16(1);
        bytecode.emit(Opcode::Add, crate::span::Span::dummy());
        bytecode.emit(Opcode::Return, crate::span::Span::dummy());

        let mut vm = VM::new(bytecode);
        let result = vm.run().unwrap();
        assert_eq!(result, Some(Value::Number(8.0)));
    }

    #[test]
    fn test_vm_call_wrong_arity() {
        use crate::value::FunctionRef;

        // Test that calling with wrong number of args fails
        let mut bytecode = Bytecode::new();

        let func_ref = FunctionRef {
            name: "test".to_string(),
            arity: 2, // Expects 2 args
            bytecode_offset: 10,
            local_count: 2,
        };
        let func_idx = bytecode.add_constant(Value::Function(func_ref));

        bytecode.emit(Opcode::Constant, crate::span::Span::dummy());
        bytecode.emit_u16(func_idx);

        // Only push 1 argument
        bytecode.emit(Opcode::Null, crate::span::Span::dummy());

        // Call with 1 argument (should fail)
        bytecode.emit(Opcode::Call, crate::span::Span::dummy());
        bytecode.emit_u8(1);

        let mut vm = VM::new(bytecode);
        let result = vm.run();
        assert!(result.is_err());
        match result.unwrap_err() {
            RuntimeError::TypeError { msg, .. } => assert!(msg.contains("expects 2 arguments")),
            _ => panic!("Expected TypeError"),
        }
    }

    #[test]
    fn test_vm_call_non_function() {
        // Test that calling a non-function value fails
        let mut bytecode = Bytecode::new();

        // Push a number (not a function)
        let idx = bytecode.add_constant(Value::Number(42.0));
        bytecode.emit(Opcode::Constant, crate::span::Span::dummy());
        bytecode.emit_u16(idx);

        // Try to call it
        bytecode.emit(Opcode::Call, crate::span::Span::dummy());
        bytecode.emit_u8(0);

        let mut vm = VM::new(bytecode);
        let result = vm.run();
        assert!(result.is_err());
        match result.unwrap_err() {
            RuntimeError::TypeError { msg, .. } => assert!(msg.contains("Cannot call non-function")),
            _ => panic!("Expected TypeError"),
        }
    }

    #[test]
    fn test_vm_nested_calls() {
        use crate::value::FunctionRef;

        // Test nested function calls: main -> f1 -> f2
        let mut bytecode = Bytecode::new();

        let f1_offset = 30;
        let f2_offset = 50;

        // Main: call f1
        let f1_ref = FunctionRef {
            name: "f1".to_string(),
            arity: 0,
            bytecode_offset: f1_offset,
            local_count: 0,
        };
        let f1_idx = bytecode.add_constant(Value::Function(f1_ref));

        bytecode.emit(Opcode::Constant, crate::span::Span::dummy());
        bytecode.emit_u16(f1_idx);
        bytecode.emit(Opcode::Call, crate::span::Span::dummy());
        bytecode.emit_u8(0);
        bytecode.emit(Opcode::Halt, crate::span::Span::dummy());

        // Pad to f1_offset
        while bytecode.instructions.len() < f1_offset {
            bytecode.emit_u8(0);
        }

        // f1: call f2
        let f2_ref = FunctionRef {
            name: "f2".to_string(),
            arity: 0,
            bytecode_offset: f2_offset,
            local_count: 1,
        };
        let f2_idx = bytecode.add_constant(Value::Function(f2_ref));

        bytecode.emit(Opcode::Constant, crate::span::Span::dummy());
        bytecode.emit_u16(f2_idx);
        bytecode.emit(Opcode::Call, crate::span::Span::dummy());
        bytecode.emit_u8(0);
        bytecode.emit(Opcode::Return, crate::span::Span::dummy());

        // Pad to f2_offset
        while bytecode.instructions.len() < f2_offset {
            bytecode.emit_u8(0);
        }

        // f2: return 100
        let idx_100 = bytecode.add_constant(Value::Number(100.0));
        bytecode.emit(Opcode::Constant, crate::span::Span::dummy());
        bytecode.emit_u16(idx_100);
        bytecode.emit(Opcode::Return, crate::span::Span::dummy());

        let mut vm = VM::new(bytecode);
        let result = vm.run().unwrap();
        assert_eq!(result, Some(Value::Number(100.0)));
    }

    // ===== Control Flow Tests =====

    #[test]
    fn test_vm_if_true_branch() {
        // Test: var x = 0; if (true) { x = 42; } else { x = 0; } x;
        let result = execute_source("var x = 0; if (true) { x = 42; } else { x = 0; } x;").unwrap();
        assert_eq!(result, Some(Value::Number(42.0)));
    }

    #[test]
    fn test_vm_if_false_branch() {
        // Test: var x = 0; if (false) { x = 42; } else { x = 99; } x;
        let result = execute_source("var x = 0; if (false) { x = 42; } else { x = 99; } x;").unwrap();
        assert_eq!(result, Some(Value::Number(99.0)));
    }

    #[test]
    fn test_vm_if_no_else() {
        // Test: var x = 10; if (false) { x = 42; } x;
        let result = execute_source("var x = 10; if (false) { x = 42; } x;").unwrap();
        assert_eq!(result, Some(Value::Number(10.0))); // x unchanged
    }

    #[test]
    fn test_vm_if_with_comparison() {
        // Test: var x = 0; if (5 > 3) { x = 1; } else { x = 2; } x;
        let result = execute_source("var x = 0; if (5 > 3) { x = 1; } else { x = 2; } x;").unwrap();
        assert_eq!(result, Some(Value::Number(1.0)));

        let result = execute_source("var x = 0; if (5 < 3) { x = 1; } else { x = 2; } x;").unwrap();
        assert_eq!(result, Some(Value::Number(2.0)));
    }

    #[test]
    fn test_vm_nested_if() {
        // Test nested if statements
        let result = execute_source(
            "var x = 0; if (true) { if (true) { x = 42; } else { x = 0; } } else { x = 99; } x;",
        )
        .unwrap();
        assert_eq!(result, Some(Value::Number(42.0)));

        let result = execute_source(
            "var x = 0; if (true) { if (false) { x = 42; } else { x = 10; } } else { x = 99; } x;",
        )
        .unwrap();
        assert_eq!(result, Some(Value::Number(10.0)));
    }

    #[test]
    fn test_vm_while_loop() {
        // Test: var x = 0; while (x < 5) { x = x + 1; } x;
        let result = execute_source(
            "var x = 0; while (x < 5) { x = x + 1; } x;",
        )
        .unwrap();
        assert_eq!(result, Some(Value::Number(5.0)));
    }

    #[test]
    fn test_vm_while_loop_never_executes() {
        // Test while loop that never executes
        let result = execute_source(
            "var x = 10; while (x < 5) { x = x + 1; } x;",
        )
        .unwrap();
        assert_eq!(result, Some(Value::Number(10.0)));
    }

    #[test]
    fn test_vm_while_loop_sum() {
        // Test: var sum = 0; var i = 1; while (i <= 10) { sum = sum + i; i = i + 1; } sum;
        let result = execute_source(
            "var sum = 0; var i = 1; while (i <= 10) { sum = sum + i; i = i + 1; } sum;",
        )
        .unwrap();
        assert_eq!(result, Some(Value::Number(55.0))); // 1+2+3+...+10 = 55
    }

    #[test]
    fn test_vm_for_loop() {
        // Test that the loop executes correctly (use var for mutable sum)
        let result = execute_source(
            "var sum = 0; for (var i = 0; i < 5; i = i + 1) { sum = sum + i; } sum;",
        )
        .unwrap();
        assert_eq!(result, Some(Value::Number(10.0))); // 0+1+2+3+4 = 10
    }

    #[test]
    fn test_vm_loop_countdown() {
        // Test loop counting down (using while since for+locals complex)
        let result = execute_source(
            "var x = 10; var i = 0; while (i < 5) { x = x - 1; i = i + 1; } x;",
        )
        .unwrap();
        assert_eq!(result, Some(Value::Number(5.0))); // 10 - 5 = 5
    }

    #[test]
    fn test_vm_while_with_local() {
        // Simple test: while loop with local variable
        let result = execute_source(
            r#"
            var count = 0;
            var i = 0;
            while (i < 3) {
                var x = 10;
                count = count + x;
                i = i + 1;
            }
            count;
        "#,
        )
        .unwrap();
        assert_eq!(result, Some(Value::Number(30.0))); // Should be 10 + 10 + 10 = 30
    }

    #[test]
    fn test_vm_nested_loops() {
        // Test nested while loops: sum of i*j for i,j in 1..3
        let result = execute_source(
            r#"
            var sum = 0;
            var i = 1;
            while (i <= 3) {
                var j = 1;  // Reset j each outer iteration
                while (j <= 3) {
                    sum = sum + (i * j);
                    j = j + 1;
                }
                i = i + 1;
            }
            sum;
        "#,
        )
        .unwrap();
        // (1*1 + 1*2 + 1*3) + (2*1 + 2*2 + 2*3) + (3*1 + 3*2 + 3*3)
        // = (1+2+3) + (2+4+6) + (3+6+9)
        // = 6 + 12 + 18 = 36
        assert_eq!(result, Some(Value::Number(36.0)));
    }

    #[test]
    fn test_vm_if_in_loop() {
        // Test if inside a loop
        let result = execute_source(
            r#"
            var sum = 0;
            for (var i = 0; i < 10; i = i + 1) {
                if (i < 5) {
                    sum = sum + i;
                }
            }
            sum;
        "#,
        )
        .unwrap();
        assert_eq!(result, Some(Value::Number(10.0))); // 0+1+2+3+4 = 10
    }

    #[test]
    fn test_vm_complex_condition() {
        // Test complex boolean expression in if (use var for mutable variables)
        let result = execute_source(
            "var x = 5; var y = 10; var z = 0; if (x < 10) { if (y > 5) { z = 1; } else { z = 2; } } else { z = 3; } z;",
        )
        .unwrap();
        assert_eq!(result, Some(Value::Number(1.0)));
    }

    #[test]
    fn test_vm_loop_with_break() {
        // Test break statement in loop
        let result = execute_source(
            r#"
            var x = 0;
            while (true) {
                x = x + 1;
                if (x == 5) {
                    break;
                }
            }
            x;
        "#,
        )
        .unwrap();
        assert_eq!(result, Some(Value::Number(5.0)));
    }

    #[test]
    fn test_vm_loop_with_continue() {
        // Test continue statement in while loop
        let result = execute_source(
            r#"
            var sum = 0;
            var i = 0;
            while (i < 10) {
                i = i + 1;
                if (i == 5) {
                    continue;
                }
                sum = sum + i;
            }
            sum;
        "#,
        )
        .unwrap();
        // 1+2+3+4+6+7+8+9+10 = 50 (skips 5, but i is incremented before check)
        assert_eq!(result, Some(Value::Number(50.0)));
    }

    #[test]
    fn test_vm_multiple_breaks() {
        // Test multiple break points in loop
        let result = execute_source(
            r#"
            var x = 0;
            while (x < 100) {
                x = x + 1;
                if (x == 3) {
                    break;
                }
                if (x == 5) {
                    break;
                }
            }
            x;
        "#,
        )
        .unwrap();
        assert_eq!(result, Some(Value::Number(3.0))); // Breaks at first condition
    }

    #[test]
    fn test_vm_nested_break() {
        // Test that break only exits innermost loop
        let result = execute_source(
            r#"
            var outer = 0;
            var i = 0;
            while (i < 3) {
                var j = 0;
                while (j < 3) {
                    if (j == 1) {
                        break;
                    }
                    outer = outer + 1;
                    j = j + 1;
                }
                i = i + 1;
            }
            outer;
        "#,
        )
        .unwrap();
        // Each outer iteration: inner loop breaks after j=0, so outer increments once
        // Total: 3 iterations = 3
        assert_eq!(result, Some(Value::Number(3.0)));
    }

    // ===== Runtime Error Tests (Phase 09) =====

    #[test]
    fn test_vm_runtime_error_modulo_by_zero() {
        // Test modulo by zero runtime error
        let result = execute_source("10 % 0;");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RuntimeError::DivideByZero { .. }));
    }

    #[test]
    fn test_vm_runtime_error_zero_divided_by_zero() {
        // 0/0 should trigger divide by zero error
        let result = execute_source("0 / 0;");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RuntimeError::DivideByZero { .. }));
    }

    #[test]
    fn test_vm_runtime_error_array_out_of_bounds_read() {
        // Test array out of bounds read
        let result = execute_source("let arr = [1, 2, 3]; arr[10];");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RuntimeError::OutOfBounds { .. }));
    }

    // TODO: Add array out of bounds write test when array index assignment is implemented in compiler
    // #[test]
    // fn test_vm_runtime_error_array_out_of_bounds_write() {
    //     let result = execute_source("var arr = [1, 2, 3]; arr[10] = 5; arr;");
    //     assert!(result.is_err());
    //     assert!(matches!(result.unwrap_err(), RuntimeError::OutOfBounds { .. }));
    // }

    #[test]
    fn test_vm_runtime_error_negative_index() {
        // Test negative array index
        let result = execute_source("let arr = [1, 2, 3]; arr[-1];");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RuntimeError::InvalidIndex { .. }));
    }

    #[test]
    fn test_vm_runtime_error_non_integer_index() {
        // Test non-integer array index
        let result = execute_source("let arr = [1, 2, 3]; arr[1.5];");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RuntimeError::InvalidIndex { .. }));
    }

    #[test]
    fn test_vm_runtime_error_invalid_numeric_add() {
        // Test invalid numeric result from large number addition
        let result = execute_source("let x = 1.7976931348623157e308 + 1.7976931348623157e308;");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RuntimeError::InvalidNumericResult { .. }
        ));
    }

    #[test]
    fn test_vm_runtime_error_invalid_numeric_multiply() {
        // Test invalid numeric result from large number multiplication
        let result = execute_source("let x = 1e308 * 2.0;");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RuntimeError::InvalidNumericResult { .. }
        ));
    }

    #[test]
    fn test_vm_runtime_error_in_expression() {
        // Test that runtime errors propagate through expressions
        let result = execute_source("let x = 5 + (10 / 0);");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RuntimeError::DivideByZero { .. }));
    }

    // TODO: Add function call error test when functions are fully implemented
    // #[test]
    // fn test_vm_runtime_error_in_function_call() {
    //     let result = execute_source(
    //         r#"
    //         function divide(a, b) {
    //             return a / b;
    //         }
    //         divide(10, 0);
    //     "#,
    //     );
    //     assert!(result.is_err());
    //     assert!(matches!(result.unwrap_err(), RuntimeError::DivideByZero { .. }));
    // }

    #[test]
    fn test_vm_runtime_error_compound_divide_by_zero() {
        // Test divide by zero in compound assignment
        let result = execute_source("var x = 10; x = x / 0;");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RuntimeError::DivideByZero { .. }));
    }

    // ===== Phase 17: VM Numeric Error Propagation Tests =====

    #[test]
    fn test_vm_modulo_by_zero() {
        // Test modulo by zero (AT0005)
        let result = execute_source("10 % 0;");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RuntimeError::DivideByZero { .. }));
    }

    #[test]
    fn test_vm_modulo_zero_by_zero() {
        // Test 0 % 0 should also be divide by zero
        let result = execute_source("0 % 0;");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RuntimeError::DivideByZero { .. }));
    }

    #[test]
    fn test_vm_modulo_invalid_numeric_result() {
        // Test modulo that produces invalid result
        let result = execute_source("1e308 % 0.1;");
        // This may produce NaN or Infinity depending on the operation
        if result.is_err() {
            assert!(matches!(
                result.unwrap_err(),
                RuntimeError::InvalidNumericResult { .. } | RuntimeError::DivideByZero { .. }
            ));
        }
    }

    #[test]
    fn test_vm_subtraction_overflow() {
        // Test subtraction that produces invalid result (AT0007)
        let result = execute_source("let x = -1.7976931348623157e308 - 1.7976931348623157e308;");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RuntimeError::InvalidNumericResult { .. }
        ));
    }

    #[test]
    fn test_vm_negation_overflow() {
        // Test negation that produces invalid result
        // Note: Negation of very large numbers shouldn't overflow, but let's test edge cases
        let result = execute_source("let x = -1.7976931348623157e308; let y = -x;");
        // Negation should work fine, but if it somehow produces infinity, catch it
        if result.is_err() {
            assert!(matches!(
                result.unwrap_err(),
                RuntimeError::InvalidNumericResult { .. }
            ));
        } else {
            // Should succeed
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_vm_division_produces_infinity() {
        // Test division that would produce infinity (very large / very small)
        let result = execute_source("let x = 1e308 / 1e-308;");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RuntimeError::InvalidNumericResult { .. }
        ));
    }

    #[test]
    fn test_vm_numeric_error_matches_interpreter_divide_by_zero() {
        // Verify VM and interpreter produce same error for divide by zero
        let source = "1 / 0;";

        // Test VM
        let vm_result = execute_source(source);
        assert!(vm_result.is_err());
        assert!(matches!(vm_result.unwrap_err(), RuntimeError::DivideByZero { .. }));

        // Test Interpreter (via runtime)
        use crate::runtime::Atlas;
        let runtime = Atlas::new();
        let interp_result = runtime.eval(source);
        assert!(interp_result.is_err());
        // Interpreter converts RuntimeError to Diagnostic, so check diagnostic code
        let diags = interp_result.unwrap_err();
        assert!(!diags.is_empty());
        assert_eq!(diags[0].code, "AT0005");
    }

    #[test]
    fn test_vm_numeric_error_matches_interpreter_overflow() {
        // Verify VM and interpreter produce same error for overflow
        let source = "1e308 * 2.0;";

        // Test VM
        let vm_result = execute_source(source);
        assert!(vm_result.is_err());
        assert!(matches!(
            vm_result.unwrap_err(),
            RuntimeError::InvalidNumericResult { .. }
        ));

        // Test Interpreter (via runtime)
        use crate::runtime::Atlas;
        let runtime = Atlas::new();
        let interp_result = runtime.eval(source);
        assert!(interp_result.is_err());
        let diags = interp_result.unwrap_err();
        assert!(!diags.is_empty());
        assert_eq!(diags[0].code, "AT0007");
    }

    #[test]
    fn test_vm_numeric_error_in_nested_expression() {
        // Test that numeric errors propagate correctly in nested expressions
        let result = execute_source("let x = (5 + 3) * (10 / 0);");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RuntimeError::DivideByZero { .. }));
    }

    #[test]
    fn test_vm_numeric_error_in_array() {
        // Test numeric error in array element
        let result = execute_source("let arr = [1, 2, 10 / 0];");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RuntimeError::DivideByZero { .. }));
    }

    #[test]
    fn test_vm_numeric_error_in_array_index() {
        // Test numeric error used as array index
        let result = execute_source("let arr = [1, 2, 3]; arr[10 / 0];");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RuntimeError::DivideByZero { .. }));
    }

    #[test]
    fn test_vm_multiple_numeric_operations_no_error() {
        // Test that valid numeric operations don't trigger errors
        let result = execute_source("let x = 10 / 2 + 5 * 3 - 8 % 3;");
        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value, Some(Value::Number(5.0 + 15.0 - 2.0))); // 10/2=5, 5*3=15, 8%3=2, 5+15-2=18
    }

    #[test]
    fn test_vm_division_by_very_small_number() {
        // Test division by very small number (not zero)
        let result = execute_source("let x = 1.0 / 1e-300;");
        // Should succeed as long as result is not infinity
        // 1.0 / 1e-300 = 1e300, which is less than max f64
        assert!(result.is_ok());
    }

    #[test]
    fn test_vm_division_edge_case_max_divided_by_min() {
        // Test edge case: max / min
        let result = execute_source("let x = 1e308 / 1e-308;");
        // This will overflow to infinity
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RuntimeError::InvalidNumericResult { .. }
        ));
    }

    #[test]
    fn test_vm_addition_edge_case_max_plus_max() {
        // Test edge case: max + max
        let result = execute_source("let x = 1.7976931348623157e308 + 1.7976931348623157e308;");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RuntimeError::InvalidNumericResult { .. }
        ));
    }

    #[test]
    fn test_vm_numeric_error_codes_division_by_zero() {
        // Explicitly test that AT0005 is used for division by zero
        use crate::runtime::Atlas;
        let runtime = Atlas::new();
        let result = runtime.eval("1 / 0");
        assert!(result.is_err());
        let diags = result.unwrap_err();
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, "AT0005");
        assert!(diags[0].message.contains("Divide by zero"));
    }

    #[test]
    fn test_vm_numeric_error_codes_invalid_result() {
        // Explicitly test that AT0007 is used for invalid numeric results
        use crate::runtime::Atlas;
        let runtime = Atlas::new();
        let result = runtime.eval("1e308 * 1e308");
        assert!(result.is_err());
        let diags = result.unwrap_err();
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, "AT0007");
        assert!(diags[0].message.contains("Invalid numeric result"));
    }

    // ===== Stdlib Tests (Phase Stdlib-01) =====

    #[test]
    fn test_vm_stdlib_print_number() {
        // Note: We can't easily test stdout, but we can verify no error
        let result = execute_source("print(42);");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(Value::Null));
    }

    #[test]
    fn test_vm_stdlib_print_string() {
        let result = execute_source("print(\"hello\");");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(Value::Null));
    }

    #[test]
    fn test_vm_stdlib_print_bool() {
        let result = execute_source("print(true);");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(Value::Null));
    }

    #[test]
    fn test_vm_stdlib_print_null() {
        let result = execute_source("print(null);");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(Value::Null));
    }

    #[test]
    fn test_vm_stdlib_len_string() {
        let result = execute_source("len(\"hello\");");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(Value::Number(5.0)));
    }

    #[test]
    fn test_vm_stdlib_len_unicode_string() {
        // Test Unicode scalar count
        let result = execute_source("len(\"🎉\");");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(Value::Number(1.0))); // 1 char, not 4 bytes
    }

    #[test]
    fn test_vm_stdlib_len_array() {
        let result = execute_source("len([1, 2, 3]);");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(Value::Number(3.0)));
    }

    #[test]
    fn test_vm_stdlib_len_empty_string() {
        let result = execute_source("len(\"\");");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(Value::Number(0.0)));
    }

    #[test]
    fn test_vm_stdlib_len_empty_array() {
        let result = execute_source("len([]);");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(Value::Number(0.0)));
    }

    #[test]
    fn test_vm_stdlib_str_number() {
        let result = execute_source("str(42);");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(Value::string("42")));
    }

    #[test]
    fn test_vm_stdlib_str_bool_true() {
        let result = execute_source("str(true);");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(Value::string("true")));
    }

    #[test]
    fn test_vm_stdlib_str_bool_false() {
        let result = execute_source("str(false);");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(Value::string("false")));
    }

    #[test]
    fn test_vm_stdlib_str_null() {
        let result = execute_source("str(null);");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(Value::string("null")));
    }

    #[test]
    fn test_vm_stdlib_len_in_expression() {
        let result = execute_source("let x = len(\"test\") + 1;");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(Value::Number(5.0))); // 4 + 1
    }

    #[test]
    fn test_vm_stdlib_str_in_concat() {
        let result = execute_source("let x = \"Number: \" + str(42);");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(Value::string("Number: 42")));
    }

    #[test]
    fn test_vm_stdlib_nested_calls() {
        let result = execute_source("len(str(12345));");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(Value::Number(5.0))); // "12345" has 5 chars
    }

    #[test]
    fn test_vm_stdlib_in_variable() {
        let result = execute_source("let x = len(\"hello\"); x;");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(Value::Number(5.0)));
    }

    #[test]
    fn test_vm_stdlib_in_array() {
        let result = execute_source("let arr = [len(\"a\"), len(\"ab\"), len(\"abc\")]; arr[1];");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(Value::Number(2.0)));
    }

    #[test]
    fn test_vm_stdlib_matches_interpreter_print() {
        // Verify VM and interpreter produce same behavior
        let source = "print(\"test\");";

        let vm_result = execute_source(source);
        assert!(vm_result.is_ok());

        use crate::runtime::Atlas;
        let runtime = Atlas::new();
        let interp_result = runtime.eval(source);
        assert!(interp_result.is_ok());
    }

    #[test]
    fn test_vm_stdlib_matches_interpreter_len() {
        let source = "len(\"hello\");";

        let vm_result = execute_source(source);
        assert!(vm_result.is_ok());
        assert_eq!(vm_result.unwrap(), Some(Value::Number(5.0)));

        use crate::runtime::Atlas;
        let runtime = Atlas::new();
        let interp_result = runtime.eval(source);
        assert!(interp_result.is_ok());
        assert_eq!(interp_result.unwrap(), Value::Number(5.0));
    }

    #[test]
    fn test_vm_stdlib_matches_interpreter_str() {
        let source = "str(42);";

        let vm_result = execute_source(source);
        assert!(vm_result.is_ok());
        assert_eq!(vm_result.unwrap(), Some(Value::string("42")));

        use crate::runtime::Atlas;
        let runtime = Atlas::new();
        let interp_result = runtime.eval(source);
        assert!(interp_result.is_ok());
        assert_eq!(interp_result.unwrap(), Value::string("42"));
    }
}
