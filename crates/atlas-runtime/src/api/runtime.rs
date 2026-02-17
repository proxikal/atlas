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
    /// Interpreter state (used in Interpreter mode, also stores globals for VM mode)
    interpreter: RefCell<Interpreter>,
    /// Security context for permission checks
    security: SecurityContext,
    /// Accumulated bytecode for VM mode (persists across eval() calls)
    accumulated_bytecode: RefCell<crate::bytecode::Bytecode>,
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
            accumulated_bytecode: RefCell::new(crate::bytecode::Bytecode::new()),
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
            accumulated_bytecode: RefCell::new(crate::bytecode::Bytecode::new()),
        }
    }

    /// Create a new runtime with configuration
    ///
    /// Converts RuntimeConfig into appropriate SecurityContext settings.
    /// Note: Timeout and memory limits in config are documented but not yet enforced.
    ///
    /// # Examples
    ///
    /// ```
    /// use atlas_runtime::api::{Runtime, ExecutionMode, RuntimeConfig};
    ///
    /// let config = RuntimeConfig::sandboxed();
    /// let mut runtime = Runtime::with_config(ExecutionMode::VM, config);
    /// ```
    pub fn with_config(mode: ExecutionMode, config: super::config::RuntimeConfig) -> Self {
        // Create security context based on config flags
        let security = if config.allow_io {
            SecurityContext::allow_all() // For now, simplified - allows all if IO is allowed
        } else {
            SecurityContext::new() // Deny-all by default
        };
        // Note: max_execution_time and max_memory_bytes are stored in config but not yet enforced
        // TODO: Implement timeout and memory limit enforcement

        Self {
            mode,
            interpreter: RefCell::new(Interpreter::new()),
            security,
            accumulated_bytecode: RefCell::new(crate::bytecode::Bytecode::new()),
        }
    }

    /// Create a sandboxed runtime with restrictive defaults
    ///
    /// Equivalent to `Runtime::with_config(mode, RuntimeConfig::sandboxed())`.
    /// Disables IO and network operations.
    ///
    /// # Examples
    ///
    /// ```
    /// use atlas_runtime::api::{Runtime, ExecutionMode};
    ///
    /// let mut runtime = Runtime::sandboxed(ExecutionMode::VM);
    /// // Attempts to use IO operations will fail
    /// ```
    pub fn sandboxed(mode: ExecutionMode) -> Self {
        Self::with_config(mode, super::config::RuntimeConfig::sandboxed())
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

        // Create initial symbol table with registered globals
        let mut initial_symbol_table = crate::symbol::SymbolTable::new();
        {
            let interpreter = self.interpreter.borrow();
            for (name, value) in &interpreter.globals {
                // Determine symbol kind based on value type
                let kind = match value {
                    Value::NativeFunction(_) | Value::Function(_) => {
                        crate::symbol::SymbolKind::Function
                    }
                    _ => crate::symbol::SymbolKind::Variable,
                };

                // Create symbol with placeholder type (runtime values don't have compile-time types)
                let symbol = crate::symbol::Symbol {
                    name: name.clone(),
                    ty: crate::types::Type::Unknown, // Dynamic values have unknown type at compile time
                    mutable: false,
                    kind: kind.clone(),
                    span: crate::span::Span::dummy(),
                    exported: false,
                };

                // Add to initial symbol table
                if kind == crate::symbol::SymbolKind::Function {
                    let _ = initial_symbol_table.define_function(symbol);
                } else {
                    let _ = initial_symbol_table.define(symbol);
                }
            }
        }

        // Bind symbols with pre-populated symbol table
        let mut binder = Binder::with_symbol_table(initial_symbol_table);
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
                // Compile new AST to bytecode
                let mut compiler = Compiler::new();
                let new_bytecode = match compiler.compile(&ast) {
                    Ok(bc) => bc,
                    Err(diagnostics) => return Err(EvalError::ParseError(diagnostics)),
                };

                // Get the start offset of new code (before appending)
                let new_code_start = self.accumulated_bytecode.borrow().instructions.len();

                // Append to accumulated bytecode
                self.accumulated_bytecode.borrow_mut().append(new_bytecode);

                // Create VM with the accumulated bytecode
                let accumulated = self.accumulated_bytecode.borrow().clone();
                let mut vm = VM::new(accumulated);

                // Set IP to start of new code (so we don't re-execute old code)
                vm.set_ip(new_code_start);

                // Copy interpreter globals to VM (for natives and other complex types)
                {
                    let interpreter = self.interpreter.borrow();
                    for (name, value) in &interpreter.globals {
                        vm.set_global(name.clone(), value.clone());
                    }
                }

                let result = match vm.run(&self.security) {
                    Ok(Some(value)) => Ok(value),
                    Ok(None) => Ok(Value::Null),
                    Err(e) => Err(EvalError::RuntimeError(e)),
                };

                // Copy VM globals back to interpreter for persistence across eval() calls
                {
                    let mut interpreter = self.interpreter.borrow_mut();
                    for (name, value) in vm.get_globals() {
                        interpreter.globals.insert(name.clone(), value.clone());
                    }
                }

                result
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
                // For native functions and other complex types, store in interpreter globals
                // The VM will look them up from there during execution
                if matches!(
                    value,
                    Value::NativeFunction(_) | Value::Array(_) | Value::Function(_)
                ) {
                    let mut interpreter = self.interpreter.borrow_mut();
                    interpreter.globals.insert(name.to_string(), value);
                    return;
                }

                // VM mode doesn't support direct global manipulation for simple types
                // Would need to store globals separately and merge on each eval
                // For v0.2 phase-01, we'll use eval to set globals
                let set_code = match &value {
                    Value::Number(n) => format!("var {}: number = {};", name, n),
                    Value::String(s) => {
                        format!("var {}: string = \"{}\";", name, s.replace('"', "\\\""))
                    }
                    Value::Bool(b) => format!("var {}: bool = {};", name, b),
                    Value::Null => format!("var {}: null = null;", name),
                    _ => return, // Can't set other complex types via code generation
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

    /// Register a native function with fixed arity
    ///
    /// Registers a Rust closure as a callable function in Atlas code. The function
    /// will be available globally and can be called like any Atlas function.
    ///
    /// The function's arity (argument count) is validated automatically - calls with
    /// the wrong number of arguments will result in a runtime error.
    ///
    /// # Arguments
    ///
    /// * `name` - Function name (how it will be called from Atlas)
    /// * `arity` - Required number of arguments
    /// * `implementation` - Rust closure implementing the function
    ///
    /// # Examples
    ///
    /// ```
    /// use atlas_runtime::api::{Runtime, ExecutionMode};
    /// use atlas_runtime::value::{Value, RuntimeError};
    /// use atlas_runtime::span::Span;
    ///
    /// let mut runtime = Runtime::new(ExecutionMode::Interpreter);
    ///
    /// // Register a native "add" function
    /// runtime.register_function("add", 2, |args| {
    ///     let a = match &args[0] {
    ///         Value::Number(n) => *n,
    ///         _ => return Err(RuntimeError::TypeError {
    ///             msg: "Expected number".to_string(),
    ///             span: Span::dummy()
    ///         }),
    ///     };
    ///     let b = match &args[1] {
    ///         Value::Number(n) => *n,
    ///         _ => return Err(RuntimeError::TypeError {
    ///             msg: "Expected number".to_string(),
    ///             span: Span::dummy()
    ///         }),
    ///     };
    ///     Ok(Value::Number(a + b))
    /// });
    ///
    /// // Call from Atlas code
    /// let result = runtime.eval("add(10, 20)").unwrap();
    /// ```
    pub fn register_function<F>(&mut self, name: &str, arity: usize, implementation: F)
    where
        F: Fn(&[Value]) -> Result<Value, RuntimeError> + Send + Sync + 'static,
    {
        let native_fn = crate::api::native::NativeFunctionBuilder::new(name)
            .with_arity(arity)
            .with_implementation(implementation)
            .build()
            .expect("Failed to build native function");

        self.set_global(name, native_fn);
    }

    /// Register a variadic native function
    ///
    /// Registers a Rust closure as a callable function that accepts any number of arguments.
    /// The implementation is responsible for validating the argument count and types.
    ///
    /// # Arguments
    ///
    /// * `name` - Function name (how it will be called from Atlas)
    /// * `implementation` - Rust closure implementing the function
    ///
    /// # Examples
    ///
    /// ```
    /// use atlas_runtime::api::{Runtime, ExecutionMode};
    /// use atlas_runtime::value::{Value, RuntimeError};
    /// use atlas_runtime::span::Span;
    ///
    /// let mut runtime = Runtime::new(ExecutionMode::Interpreter);
    ///
    /// // Register a variadic "sum" function
    /// runtime.register_variadic("sum", |args| {
    ///     let mut total = 0.0;
    ///     for arg in args {
    ///         match arg {
    ///             Value::Number(n) => total += n,
    ///             _ => return Err(RuntimeError::TypeError {
    ///                 msg: "All arguments must be numbers".to_string(),
    ///                 span: Span::dummy()
    ///             }),
    ///         }
    ///     }
    ///     Ok(Value::Number(total))
    /// });
    ///
    /// // Call with any number of arguments
    /// let result = runtime.eval("sum(1, 2, 3, 4, 5)").unwrap();
    /// ```
    pub fn register_variadic<F>(&mut self, name: &str, implementation: F)
    where
        F: Fn(&[Value]) -> Result<Value, RuntimeError> + Send + Sync + 'static,
    {
        let native_fn = crate::api::native::NativeFunctionBuilder::new(name)
            .variadic()
            .with_implementation(implementation)
            .build()
            .expect("Failed to build native function");

        self.set_global(name, native_fn);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

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
            .call("print", vec![Value::String(Arc::new("hello".to_string()))])
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
