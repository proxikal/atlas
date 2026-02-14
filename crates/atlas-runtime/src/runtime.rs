//! Atlas runtime API for embedding

use crate::binder::Binder;
use crate::diagnostic::Diagnostic;
use crate::interpreter::Interpreter;
use crate::lexer::Lexer;
use crate::module_executor::ModuleExecutor;
use crate::parser::Parser;
use crate::security::SecurityContext;
use crate::span::Span;
use crate::typechecker::TypeChecker;
use crate::value::{RuntimeError, Value};
use std::cell::RefCell;

/// Result type for runtime operations
pub type RuntimeResult<T> = Result<T, Vec<Diagnostic>>;

/// Atlas runtime instance
///
/// Provides a high-level API for embedding Atlas in host applications.
///
/// # Examples
///
/// ```
/// use atlas_runtime::Atlas;
///
/// let runtime = Atlas::new();
/// let result = runtime.eval("1 + 2");
/// ```
pub struct Atlas {
    /// Interpreter for executing code (using interior mutability)
    interpreter: RefCell<Interpreter>,
    /// Security context for permission checks
    security: SecurityContext,
}

impl Atlas {
    /// Create a new Atlas runtime instance with default (deny-all) security
    ///
    /// # Examples
    ///
    /// ```
    /// use atlas_runtime::Atlas;
    ///
    /// let runtime = Atlas::new();
    /// ```
    pub fn new() -> Self {
        Self {
            interpreter: RefCell::new(Interpreter::new()),
            security: SecurityContext::new(),
        }
    }

    /// Create a new Atlas runtime instance with custom security context
    ///
    /// # Examples
    ///
    /// ```
    /// use atlas_runtime::{Atlas, SecurityContext};
    ///
    /// let security = SecurityContext::allow_all();
    /// let runtime = Atlas::new_with_security(security);
    /// ```
    pub fn new_with_security(security: SecurityContext) -> Self {
        Self {
            interpreter: RefCell::new(Interpreter::new()),
            security,
        }
    }

    /// Evaluate Atlas source code
    ///
    /// Returns the result of evaluating the source code, or diagnostics if there are errors.
    ///
    /// # Arguments
    ///
    /// * `source` - Atlas source code to evaluate
    ///
    /// # Examples
    ///
    /// ```
    /// use atlas_runtime::{Atlas, Value};
    ///
    /// let runtime = Atlas::new();
    /// let result = runtime.eval("1 + 2");
    /// match result {
    ///     Ok(Value::Number(n)) => assert_eq!(n, 3.0),
    ///     Err(diagnostics) => panic!("Error: {:?}", diagnostics),
    ///     Ok(_) => panic!("Unexpected value"),
    /// }
    /// ```
    pub fn eval(&self, source: &str) -> RuntimeResult<Value> {
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
            return Err(lex_diagnostics);
        }

        // Parse tokens into AST
        let mut parser = Parser::new(tokens);
        let (ast, parse_diagnostics) = parser.parse();

        if !parse_diagnostics.is_empty() {
            return Err(parse_diagnostics);
        }

        // Bind symbols
        let mut binder = Binder::new();
        let (mut symbol_table, bind_diagnostics) = binder.bind(&ast);

        if !bind_diagnostics.is_empty() {
            return Err(bind_diagnostics);
        }

        // Type check
        let mut type_checker = TypeChecker::new(&mut symbol_table);
        let type_diagnostics = type_checker.check(&ast);

        if !type_diagnostics.is_empty() {
            return Err(type_diagnostics);
        }

        // Interpret the AST
        let mut interpreter = self.interpreter.borrow_mut();

        match interpreter.eval(&ast) {
            Ok(value) => Ok(value),
            Err(runtime_error) => Err(vec![runtime_error_to_diagnostic(runtime_error)]),
        }
    }

    /// Evaluate an Atlas source file
    ///
    /// Reads and evaluates the Atlas source code from the specified file path.
    /// If the file contains imports, uses the module system to load dependencies.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the Atlas source file
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use atlas_runtime::Atlas;
    ///
    /// let runtime = Atlas::new();
    /// let result = runtime.eval_file("program.atlas");
    /// ```
    pub fn eval_file(&self, path: &str) -> RuntimeResult<Value> {
        use std::path::Path;

        let file_path = Path::new(path);

        // Get absolute path
        let abs_path = file_path.canonicalize().map_err(|e| {
            vec![Diagnostic::error(
                format!("Failed to resolve path: {}", e),
                Span::dummy(),
            )]
        })?;

        // Check filesystem read permission
        self.security
            .check_filesystem_read(&abs_path)
            .map_err(|_| {
                vec![runtime_error_to_diagnostic(
                    RuntimeError::FilesystemPermissionDenied {
                        operation: "file read".to_string(),
                        path: abs_path.display().to_string(),
                        span: Span::dummy(),
                    },
                )]
            })?;

        // Quick check: does the file contain imports?
        // If so, use module executor. If not, use simple eval.
        let source = std::fs::read_to_string(&abs_path).map_err(|e| {
            vec![Diagnostic::error(
                format!("Failed to read file: {}", e),
                Span::dummy(),
            )]
        })?;

        // Check if source contains "import {" or "import *"
        if source.contains("import {") || source.contains("import *") {
            // Use module executor for multi-file programs
            let root = abs_path
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .to_path_buf();

            let mut executor = ModuleExecutor::new(root);
            executor.execute_module(&abs_path)
        } else {
            // Simple single-file program - use regular eval
            self.eval(&source)
        }
    }
}

impl Default for Atlas {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert a RuntimeError to a Diagnostic
fn runtime_error_to_diagnostic(error: RuntimeError) -> Diagnostic {
    // Map runtime errors to their corresponding diagnostic codes from Atlas-SPEC.md
    // Extract span from error (all RuntimeError variants now include span)
    let span = error.span();

    let (code, message) = match &error {
        RuntimeError::DivideByZero { .. } => ("AT0005", "Divide by zero".to_string()),
        RuntimeError::OutOfBounds { .. } => ("AT0006", "Array index out of bounds".to_string()),
        RuntimeError::InvalidNumericResult { .. } => (
            "AT0007",
            "Invalid numeric result (NaN or Infinity)".to_string(),
        ),
        RuntimeError::InvalidIndex { .. } => (
            "AT0103",
            "Invalid index: array indices must be whole numbers".to_string(),
        ),
        RuntimeError::InvalidStdlibArgument { .. } => (
            "AT0102",
            "Invalid argument to standard library function".to_string(),
        ),
        RuntimeError::TypeError { msg, .. } => ("AT0001", format!("Type error: {}", msg)),
        RuntimeError::UndefinedVariable { name, .. } => {
            ("AT0002", format!("Unknown symbol: {}", name))
        }
        RuntimeError::UnknownFunction { name, .. } => {
            ("AT0002", format!("Unknown function: {}", name))
        }
        // VM-specific errors
        RuntimeError::UnknownOpcode { .. } => ("AT9998", "Unknown bytecode opcode".to_string()),
        RuntimeError::StackUnderflow { .. } => ("AT9997", "Stack underflow".to_string()),
        // Permission errors
        RuntimeError::FilesystemPermissionDenied {
            operation, path, ..
        } => (
            "AT0300",
            format!("Permission denied: {} access to {}", operation, path),
        ),
        RuntimeError::NetworkPermissionDenied { host, .. } => (
            "AT0301",
            format!("Permission denied: network access to {}", host),
        ),
        RuntimeError::ProcessPermissionDenied { command, .. } => (
            "AT0302",
            format!("Permission denied: process execution of {}", command),
        ),
        RuntimeError::EnvironmentPermissionDenied { var, .. } => (
            "AT0303",
            format!("Permission denied: environment variable {}", var),
        ),
    };

    Diagnostic::error_with_code(code, message, span)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::DiagnosticLevel;

    // Basic API Tests

    #[test]
    fn test_runtime_creation() {
        let _runtime = Atlas::new();
        // Runtime can be created successfully
    }

    #[test]
    fn test_runtime_default() {
        let _runtime = Atlas::default();
        // Runtime can be created via Default trait
    }

    // eval() Tests

    #[test]
    fn test_eval_number_literal() {
        let runtime = Atlas::new();
        let result = runtime.eval("42");
        match result {
            Ok(Value::Number(n)) => assert_eq!(n, 42.0),
            _ => panic!("Expected Number(42.0)"),
        }
    }

    #[test]
    fn test_eval_simple_arithmetic() {
        let runtime = Atlas::new();
        let result = runtime.eval("1 + 2");
        match result {
            Ok(Value::Number(n)) => assert_eq!(n, 3.0),
            _ => panic!("Expected Number(3.0)"),
        }
    }

    #[test]
    fn test_eval_variable_declaration() {
        let runtime = Atlas::new();
        let result = runtime.eval("let x: number = 42;");
        match result {
            Ok(Value::Null) => (),
            _ => panic!("Expected Null for variable declaration"),
        }
    }

    #[test]
    fn test_eval_variable_use() {
        let runtime = Atlas::new();
        let result = runtime.eval("let x: number = 42; x");
        match result {
            Ok(Value::Number(n)) => assert_eq!(n, 42.0),
            _ => panic!("Expected Number(42.0)"),
        }
    }

    #[test]
    fn test_eval_syntax_error() {
        let runtime = Atlas::new();
        let result = runtime.eval("let x: number =");
        // Should return parse error diagnostic
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_invalid_syntax() {
        let runtime = Atlas::new();
        let result = runtime.eval("@#$%^&*");
        // Should return lexer/parser error
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_returns_diagnostics() {
        let runtime = Atlas::new();
        let result = runtime.eval("let x: number =");
        match result {
            Err(diagnostics) => {
                assert!(!diagnostics.is_empty());
                assert_eq!(diagnostics[0].level, DiagnosticLevel::Error);
            }
            Ok(_) => panic!("Expected error diagnostics"),
        }
    }

    #[test]
    fn test_eval_multiple_statements() {
        let runtime = Atlas::new();
        let result = runtime.eval("let x: number = 1; let y: number = 2; y");
        match result {
            Ok(Value::Number(n)) => assert_eq!(n, 2.0),
            _ => panic!("Expected Number(2.0)"),
        }
    }

    // eval_file() Tests

    #[test]
    fn test_eval_file_missing_file() {
        let runtime = Atlas::new();
        let result = runtime.eval_file("nonexistent.atlas");
        // Should return error (file not found)
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_file_returns_diagnostics() {
        let runtime = Atlas::new();
        let result = runtime.eval_file("nonexistent.atlas");
        match result {
            Err(diagnostics) => {
                assert!(!diagnostics.is_empty());
                assert_eq!(diagnostics[0].level, DiagnosticLevel::Error);
            }
            Ok(_) => panic!("Expected error diagnostics"),
        }
    }

    // Error Handling Tests

    #[test]
    fn test_diagnostic_contains_message() {
        let runtime = Atlas::new();
        let result = runtime.eval("let x: number =");
        match result {
            Err(diagnostics) => {
                assert!(!diagnostics[0].message.is_empty());
            }
            Ok(_) => panic!("Expected error"),
        }
    }

    // Value Tests

    #[test]
    fn test_eval_string_literal() {
        let runtime = Atlas::new();
        let result = runtime.eval(r#""hello""#);
        match result {
            Ok(Value::String(s)) => assert_eq!(*s, "hello"),
            _ => panic!("Expected String(hello)"),
        }
    }

    #[test]
    fn test_eval_boolean() {
        let runtime = Atlas::new();
        let result = runtime.eval("true");
        match result {
            Ok(Value::Bool(b)) => assert!(b),
            _ => panic!("Expected Bool(true)"),
        }
    }

    #[test]
    fn test_eval_null() {
        let runtime = Atlas::new();
        let result = runtime.eval("null");
        match result {
            Ok(Value::Null) => (),
            _ => panic!("Expected Null"),
        }
    }
}
