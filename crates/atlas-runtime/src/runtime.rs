//! Atlas runtime API for embedding

use crate::diagnostic::Diagnostic;
use crate::span::Span;
use crate::value::Value;

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
/// // Runtime API will be fully implemented in later phases
/// ```
pub struct Atlas {
    /// Placeholder for future state
    _state: (),
}

impl Atlas {
    /// Create a new Atlas runtime instance
    ///
    /// # Examples
    ///
    /// ```
    /// use atlas_runtime::Atlas;
    ///
    /// let runtime = Atlas::new();
    /// ```
    pub fn new() -> Self {
        Self { _state: () }
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
    /// use atlas_runtime::Atlas;
    ///
    /// let runtime = Atlas::new();
    /// let result = runtime.eval("let x: int = 42;");
    /// // Returns error because implementation is not complete yet
    /// assert!(result.is_err());
    /// ```
    ///
    /// # Status
    ///
    /// This method is a stub in v0.1. Full implementation will be added in later phases.
    pub fn eval(&self, _source: &str) -> RuntimeResult<Value> {
        Err(vec![Diagnostic::error(
            "Runtime API not yet implemented",
            Span::dummy(),
        )])
    }

    /// Evaluate an Atlas source file
    ///
    /// Reads and evaluates the Atlas source code from the specified file path.
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
    /// // Returns error because implementation is not complete yet
    /// assert!(result.is_err());
    /// ```
    ///
    /// # Status
    ///
    /// This method is a stub in v0.1. Full implementation will be added in later phases.
    pub fn eval_file(&self, _path: &str) -> RuntimeResult<Value> {
        Err(vec![Diagnostic::error(
            "Runtime API not yet implemented",
            Span::dummy(),
        )])
    }
}

impl Default for Atlas {
    fn default() -> Self {
        Self::new()
    }
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
    fn test_eval_empty_string() {
        let runtime = Atlas::new();
        let result = runtime.eval("");
        // Currently returns stub error, but test structure is ready
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_simple_expression() {
        let runtime = Atlas::new();
        let result = runtime.eval("1 + 2");
        // TODO: When implemented, should return Value::Number(3.0)
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_variable_declaration() {
        let runtime = Atlas::new();
        let result = runtime.eval("let x: int = 42;");
        // TODO: When implemented, should return Value::Null
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_type_error() {
        let runtime = Atlas::new();
        let result = runtime.eval("let x: int = \"string\";");
        // TODO: When implemented, should return diagnostic with error code
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_syntax_error() {
        let runtime = Atlas::new();
        let result = runtime.eval("let x: int =");
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
        let result = runtime.eval("invalid");
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
        let result = runtime.eval("let x: int = 1; let y: int = 2;");
        // TODO: When implemented, should execute both statements
        assert!(result.is_err());
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
    fn test_eval_file_with_path() {
        let runtime = Atlas::new();
        let result = runtime.eval_file("test/program.atlas");
        // Currently returns stub error
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_file_returns_diagnostics() {
        let runtime = Atlas::new();
        let result = runtime.eval_file("test.atlas");
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
        let result = runtime.eval("invalid");
        match result {
            Err(diagnostics) => {
                assert!(!diagnostics[0].message.is_empty());
            }
            Ok(_) => panic!("Expected error"),
        }
    }

    #[test]
    fn test_diagnostic_has_location() {
        let runtime = Atlas::new();
        let result = runtime.eval("test");
        match result {
            Err(diagnostics) => {
                // Diagnostic should have location information
                assert!(diagnostics[0].line >= 1);
                assert!(diagnostics[0].column >= 1);
            }
            Ok(_) => panic!("Expected error"),
        }
    }

    // Future Implementation Tests (currently stubbed)

    #[test]
    #[ignore] // TODO: Remove when eval is implemented
    fn test_eval_returns_correct_value() {
        let runtime = Atlas::new();
        let result = runtime.eval("42");
        match result {
            Ok(Value::Number(n)) => assert_eq!(n, 42.0),
            _ => panic!("Expected Number(42.0)"),
        }
    }

    #[test]
    #[ignore] // TODO: Remove when eval is implemented
    fn test_eval_arithmetic() {
        let runtime = Atlas::new();
        let result = runtime.eval("1 + 2 * 3");
        match result {
            Ok(Value::Number(n)) => assert_eq!(n, 7.0),
            _ => panic!("Expected Number(7.0)"),
        }
    }

    #[test]
    #[ignore] // TODO: Remove when eval is implemented
    fn test_eval_string_literal() {
        let runtime = Atlas::new();
        let result = runtime.eval("\"hello\"");
        match result {
            Ok(Value::String(s)) => assert_eq!(*s, "hello"),
            _ => panic!("Expected String(hello)"),
        }
    }

    #[test]
    #[ignore] // TODO: Remove when eval is implemented
    fn test_eval_boolean() {
        let runtime = Atlas::new();
        let result = runtime.eval("true");
        match result {
            Ok(Value::Bool(b)) => assert!(b),
            _ => panic!("Expected Bool(true)"),
        }
    }

    #[test]
    #[ignore] // TODO: Remove when eval is implemented
    fn test_eval_null() {
        let runtime = Atlas::new();
        let result = runtime.eval("null");
        match result {
            Ok(Value::Null) => (),
            _ => panic!("Expected Null"),
        }
    }
}
