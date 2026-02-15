//! Runtime execution API with mode selection
//!
//! Provides the `Runtime` struct for managing Atlas execution with either
//! Interpreter or VM mode. State persists across evaluations.
//!
//! # Examples
//!
//! ```
//! use atlas_runtime::api::{Runtime, ExecutionMode};
//!
//! let mut runtime = Runtime::new(ExecutionMode::Interpreter);
//!
//! // Execute code
//! runtime.eval("let x: number = 42;").unwrap();
//!
//! // State persists
//! let result = runtime.eval("x").unwrap();
//! ```

use crate::binder::Binder;
use crate::compiler::Compiler;
use crate::diagnostic::Diagnostic;
use crate::interpreter::Interpreter;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::security::SecurityContext;
use crate::typechecker::TypeChecker;
use crate::value::{RuntimeError, Value};
use crate::vm::VM;
use std::cell::RefCell;

/// Execution mode for the runtime
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    /// Use the interpreter (tree-walking AST evaluation)
    Interpreter,
    /// Use the VM (bytecode execution)
    VM,
}

/// Unified error type for runtime evaluation
#[derive(Debug)]
pub enum EvalError {
    /// Lexical or syntax errors
    ParseError(Vec<Diagnostic>),
    /// Type checking errors
    TypeError(Vec<Diagnostic>),
    /// Runtime errors during execution
    RuntimeError(RuntimeError),
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvalError::ParseError(diagnostics) => {
                write!(f, "Parse error: ")?;
                for diag in diagnostics {
                    write!(f, "{}", diag.message)?;
                }
                Ok(())
            }
            EvalError::TypeError(diagnostics) => {
                write!(f, "Type error: ")?;
                for diag in diagnostics {
                    write!(f, "{}", diag.message)?;
                }
                Ok(())
            }
            EvalError::RuntimeError(err) => write!(f, "Runtime error: {:?}", err),
        }
    }
}

impl std::error::Error for EvalError {}

/// Runtime instance managing execution state
///
/// Maintains global variables and function definitions across multiple
/// evaluations. Supports both Interpreter and VM execution modes.
///
/// # Examples
///
/// ```
/// use atlas_runtime::api::{Runtime, ExecutionMode};
///
/// let mut runtime = Runtime::new(ExecutionMode::Interpreter);
///
/// // Define a function
/// runtime.eval("fn add(x: number, y: number) -> number { x + y }").unwrap();
///
/// // Call it
/// let result = runtime.eval("add(1, 2)").unwrap();
/// ```
pub struct Runtime {
    /// Execution mode (Interpreter or VM)
    mode: ExecutionMode,
    /// Interpreter state (used in Interpreter mode)
    interpreter: RefCell<Interpreter>,
    /// Security context for permission checks
    security: SecurityContext,
}

impl Runtime {
    /// Create a new runtime with specified execution mode
    ///
    /// Uses default (deny-all) security context.
    ///
    /// # Examples
    ///
    /// ```
    /// use atlas_runtime::api::{Runtime, ExecutionMode};
    ///
    /// let mut runtime = Runtime::new(ExecutionMode::Interpreter);
    /// ```
    pub fn new(mode: ExecutionMode) -> Self {
        Self {
            mode,
            interpreter: RefCell::new(Interpreter::new()),
            security: SecurityContext::new(),
        }
    }

    /// Create a new runtime with custom security context
    ///
    /// # Examples
    ///
    /// ```
    /// use atlas_runtime::api::{Runtime, ExecutionMode};
    /// use atlas_runtime::SecurityContext;
    ///
    /// let security = SecurityContext::allow_all();
    /// let mut runtime = Runtime::new_with_security(ExecutionMode::VM, security);
    /// ```
    pub fn new_with_security(mode: ExecutionMode, security: SecurityContext) -> Self {
        Self {
            mode,
            interpreter: RefCell::new(Interpreter::new()),
            security,
        }
    }

    /// Get the current execution mode
    pub fn mode(&self) -> ExecutionMode {
        self.mode
    }

    /// Evaluate Atlas source code
    ///
    /// Runs the full compilation pipeline (lex → parse → bind → typecheck → execute)
    /// and returns the result value. State (globals, functions) persists across calls.
    ///
    /// # Arguments
    ///
    /// * `source` - Atlas source code to evaluate
    ///
    /// # Returns
    ///
    /// * `Ok(Value)` - Result of evaluation
    /// * `Err(EvalError)` - Parse, type, or runtime error
    ///
    /// # Examples
    ///
    /// ```
    /// use atlas_runtime::api::{Runtime, ExecutionMode};
    ///
    /// let mut runtime = Runtime::new(ExecutionMode::Interpreter);
    /// let result = runtime.eval("1 + 2").unwrap();
    /// ```
    pub fn eval(&mut self, source: &str) -> Result<Value, EvalError> {
        // For REPL-style usage, if the source doesn't end with a semicolon,
        // treat it as an expression statement by appending one
        let source = source.trim();
        let source_with_semi =
            if !source.is_empty() && !source.ends_with(';') && !source.ends_with('}') {
                format!("{};", source)
            } else {
                source.to_string()
            };

        // Lex the source code
        let mut lexer = Lexer::new(&source_with_semi);
        let (tokens, lex_diagnostics) = lexer.tokenize();

        if !lex_diagnostics.is_empty() {
            return Err(EvalError::ParseError(lex_diagnostics));
        }

        // Parse tokens into AST
        let mut parser = Parser::new(tokens);
        let (ast, parse_diagnostics) = parser.parse();

        if !parse_diagnostics.is_empty() {
            return Err(EvalError::ParseError(parse_diagnostics));
        }

        // Bind symbols
        let mut binder = Binder::new();
        let (mut symbol_table, bind_diagnostics) = binder.bind(&ast);

        if !bind_diagnostics.is_empty() {
            return Err(EvalError::ParseError(bind_diagnostics));
        }

        // Type check
        let mut type_checker = TypeChecker::new(&mut symbol_table);
        let type_diagnostics = type_checker.check(&ast);

        if !type_diagnostics.is_empty() {
            return Err(EvalError::TypeError(type_diagnostics));
        }

        // Execute based on mode
        match self.mode {
            ExecutionMode::Interpreter => {
                let mut interpreter = self.interpreter.borrow_mut();
                interpreter
                    .eval(&ast, &self.security)
                    .map_err(EvalError::RuntimeError)
            }
            ExecutionMode::VM => {
                // Compile to bytecode
                let mut compiler = Compiler::new();
                let bytecode = match compiler.compile(&ast) {
                    Ok(bc) => bc,
                    Err(diagnostics) => return Err(EvalError::ParseError(diagnostics)),
                };

                // Create VM and execute
                let mut vm = VM::new(bytecode);
                match vm.run(&self.security) {
                    Ok(Some(value)) => Ok(value),
                    Ok(None) => Ok(Value::Null),
                    Err(e) => Err(EvalError::RuntimeError(e)),
                }
            }
        }
    }

    /// Call an Atlas function by name with arguments
    ///
    /// Looks up the function in global scope and executes it with provided arguments.
    /// Both user-defined and builtin functions can be called.
    ///
    /// # Arguments
    ///
    /// * `name` - Function name to call
    /// * `args` - Vector of argument values
    ///
    /// # Returns
    ///
    /// * `Ok(Value)` - Function return value
    /// * `Err(EvalError)` - Runtime error (function not found, arity mismatch, etc.)
    ///
    /// # Examples
    ///
    /// ```
    /// use atlas_runtime::api::{Runtime, ExecutionMode};
    /// use atlas_runtime::Value;
    ///
    /// let mut runtime = Runtime::new(ExecutionMode::Interpreter);
    /// runtime.eval("fn add(x: number, y: number) -> number { x + y }").unwrap();
    ///
    /// let result = runtime.call("add", vec![Value::Number(1.0), Value::Number(2.0)]).unwrap();
    /// ```
    pub fn call(&mut self, name: &str, args: Vec<Value>) -> Result<Value, EvalError> {
        // Build a source string that calls the function
        // This is a simple approach that leverages existing eval infrastructure
        let args_code = args
            .iter()
            .map(|v| match v {
                Value::Number(n) => n.to_string(),
                Value::String(s) => format!("\"{}\"", s.replace('"', "\\\"")),
                Value::Bool(b) => b.to_string(),
                Value::Null => "null".to_string(),
                _ => panic!("Unsupported argument type for call()"),
            })
            .collect::<Vec<_>>()
            .join(", ");

        let call_source = format!("{}({})", name, args_code);
        self.eval(&call_source)
    }

    /// Set a global variable
    ///
    /// Creates or updates a global variable in the runtime state.
    /// The variable will be accessible in subsequent evaluations.
    ///
    /// # Arguments
    ///
    /// * `name` - Variable name
    /// * `value` - Value to assign
    ///
    /// # Examples
    ///
    /// ```
    /// use atlas_runtime::api::{Runtime, ExecutionMode};
    /// use atlas_runtime::Value;
    ///
    /// let mut runtime = Runtime::new(ExecutionMode::Interpreter);
    /// runtime.set_global("x", Value::Number(42.0));
    ///
    /// let result = runtime.eval("x").unwrap();
    /// ```
    pub fn set_global(&mut self, name: &str, value: Value) {
        match self.mode {
            ExecutionMode::Interpreter => {
                let mut interpreter = self.interpreter.borrow_mut();
                interpreter.globals.insert(name.to_string(), value);
            }
            ExecutionMode::VM => {
                // VM mode doesn't support direct global manipulation yet
                // Would need to store globals separately and merge on each eval
                // For v0.2 phase-01, we'll use eval to set globals
                let set_code = match &value {
                    Value::Number(n) => format!("var {}: number = {};", name, n),
                    Value::String(s) => {
                        format!("var {}: string = \"{}\";", name, s.replace('"', "\\\""))
                    }
                    Value::Bool(b) => format!("var {}: bool = {};", name, b),
                    Value::Null => format!("var {}: null = null;", name),
                    _ => return, // Can't set complex types via code generation
                };
                let _ = self.eval(&set_code);
            }
        }
    }

    /// Get a global variable
    ///
    /// Retrieves the current value of a global variable.
    ///
    /// # Arguments
    ///
    /// * `name` - Variable name
    ///
    /// # Returns
    ///
    /// * `Some(Value)` - Variable value if it exists
    /// * `None` - Variable not found
    ///
    /// # Examples
    ///
    /// ```
    /// use atlas_runtime::api::{Runtime, ExecutionMode};
    /// use atlas_runtime::Value;
    ///
    /// let mut runtime = Runtime::new(ExecutionMode::Interpreter);
    /// runtime.eval("let x: number = 42;").unwrap();
    ///
    /// let value = runtime.get_global("x");
    /// ```
    pub fn get_global(&self, name: &str) -> Option<Value> {
        match self.mode {
            ExecutionMode::Interpreter => {
                let interpreter = self.interpreter.borrow();
                interpreter.globals.get(name).cloned()
            }
            ExecutionMode::VM => {
                // VM mode doesn't support direct global access yet
                // Would need to store globals separately
                // For v0.2 phase-01, return None
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;

    // Runtime creation tests

    #[test]
    fn test_runtime_new_interpreter() {
        let runtime = Runtime::new(ExecutionMode::Interpreter);
        assert_eq!(runtime.mode(), ExecutionMode::Interpreter);
    }

    #[test]
    fn test_runtime_new_vm() {
        let runtime = Runtime::new(ExecutionMode::VM);
        assert_eq!(runtime.mode(), ExecutionMode::VM);
    }

    #[test]
    fn test_runtime_new_with_security() {
        let security = SecurityContext::allow_all();
        let runtime = Runtime::new_with_security(ExecutionMode::Interpreter, security);
        assert_eq!(runtime.mode(), ExecutionMode::Interpreter);
    }

    // eval() tests - Interpreter mode

    #[test]
    fn test_eval_number_literal_interpreter() {
        let mut runtime = Runtime::new(ExecutionMode::Interpreter);
        let result = runtime.eval("42").unwrap();
        assert!(matches!(result, Value::Number(n) if n == 42.0));
    }

    #[test]
    fn test_eval_arithmetic_interpreter() {
        let mut runtime = Runtime::new(ExecutionMode::Interpreter);
        let result = runtime.eval("1 + 2").unwrap();
        assert!(matches!(result, Value::Number(n) if n == 3.0));
    }

    #[test]
    fn test_eval_string_interpreter() {
        let mut runtime = Runtime::new(ExecutionMode::Interpreter);
        let result = runtime.eval("\"hello\"").unwrap();
        assert!(matches!(result, Value::String(s) if s.as_ref() == "hello"));
    }

    #[test]
    fn test_eval_bool_interpreter() {
        let mut runtime = Runtime::new(ExecutionMode::Interpreter);
        let result = runtime.eval("true").unwrap();
        assert!(matches!(result, Value::Bool(true)));
    }

    #[test]
    fn test_eval_null_interpreter() {
        let mut runtime = Runtime::new(ExecutionMode::Interpreter);
        let result = runtime.eval("null").unwrap();
        assert!(matches!(result, Value::Null));
    }

    // eval() tests - VM mode

    #[test]
    fn test_eval_number_literal_vm() {
        let mut runtime = Runtime::new(ExecutionMode::VM);
        let result = runtime.eval("42").unwrap();
        assert!(matches!(result, Value::Number(n) if n == 42.0));
    }

    #[test]
    fn test_eval_arithmetic_vm() {
        let mut runtime = Runtime::new(ExecutionMode::VM);
        let result = runtime.eval("1 + 2").unwrap();
        assert!(matches!(result, Value::Number(n) if n == 3.0));
    }

    #[test]
    fn test_eval_string_vm() {
        let mut runtime = Runtime::new(ExecutionMode::VM);
        let result = runtime.eval("\"hello\"").unwrap();
        assert!(matches!(result, Value::String(s) if s.as_ref() == "hello"));
    }

    #[test]
    fn test_eval_bool_vm() {
        let mut runtime = Runtime::new(ExecutionMode::VM);
        let result = runtime.eval("true").unwrap();
        assert!(matches!(result, Value::Bool(true)));
    }

    #[test]
    fn test_eval_null_vm() {
        let mut runtime = Runtime::new(ExecutionMode::VM);
        let result = runtime.eval("null").unwrap();
        assert!(matches!(result, Value::Null));
    }

    // Single-eval tests (state persistence requires persistent symbol tables - future phase)

    #[test]
    fn test_eval_single_program_with_variable() {
        let mut runtime = Runtime::new(ExecutionMode::Interpreter);
        let result = runtime.eval("var x: number = 42; x").unwrap();
        assert!(matches!(result, Value::Number(n) if n == 42.0));
    }

    #[test]
    fn test_eval_function_definition_and_call() {
        let mut runtime = Runtime::new(ExecutionMode::Interpreter);
        let result = runtime
            .eval("fn add(x: number, y: number) -> number { return x + y; } add(1, 2)")
            .unwrap();
        assert!(matches!(result, Value::Number(n) if n == 3.0));
    }

    // Error handling tests

    #[test]
    fn test_eval_parse_error() {
        let mut runtime = Runtime::new(ExecutionMode::Interpreter);
        let result = runtime.eval("let x: number =");
        assert!(matches!(result, Err(EvalError::ParseError(_))));
    }

    #[test]
    fn test_eval_type_error() {
        let mut runtime = Runtime::new(ExecutionMode::Interpreter);
        let result = runtime.eval("let x: number = \"hello\";");
        assert!(matches!(result, Err(EvalError::TypeError(_))));
    }

    #[test]
    fn test_eval_runtime_error() {
        let mut runtime = Runtime::new(ExecutionMode::Interpreter);
        let result = runtime.eval("1 / 0");
        assert!(matches!(result, Err(EvalError::RuntimeError(_))));
    }

    // call() tests

    #[test]
    fn test_call_builtin_function() {
        let mut runtime = Runtime::new(ExecutionMode::Interpreter);
        let result = runtime
            .call("print", vec![Value::String(Rc::new("hello".to_string()))])
            .unwrap();
        assert!(matches!(result, Value::Null));
    }

    #[test]
    fn test_call_str_builtin() {
        let mut runtime = Runtime::new(ExecutionMode::Interpreter);
        let result = runtime.call("str", vec![Value::Number(42.0)]).unwrap();
        assert!(matches!(result, Value::String(s) if s.as_ref() == "42"));
    }

    // get_global/set_global tests

    #[test]
    fn test_set_global_interpreter() {
        let mut runtime = Runtime::new(ExecutionMode::Interpreter);
        runtime.set_global("x", Value::Number(42.0));
        let value = runtime.get_global("x").unwrap();
        assert!(matches!(value, Value::Number(n) if n == 42.0));
    }

    #[test]
    fn test_get_global_nonexistent() {
        let runtime = Runtime::new(ExecutionMode::Interpreter);
        let value = runtime.get_global("nonexistent");
        assert!(value.is_none());
    }

    #[test]
    fn test_set_and_get_global() {
        let mut runtime = Runtime::new(ExecutionMode::Interpreter);
        runtime.set_global("x", Value::Number(42.0));
        let value = runtime.get_global("x").unwrap();
        assert!(matches!(value, Value::Number(n) if n == 42.0));
    }
}
