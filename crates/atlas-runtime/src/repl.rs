//! REPL core logic (UI-agnostic)

use crate::binder::Binder;
use crate::diagnostic::Diagnostic;
use crate::interpreter::Interpreter;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::security::SecurityContext;
use crate::symbol::SymbolTable;
use crate::typechecker::TypeChecker;
use crate::value::Value;

/// REPL result type
pub struct ReplResult {
    /// The value produced by evaluation (None if statement or error)
    pub value: Option<Value>,
    /// Diagnostics from all phases
    pub diagnostics: Vec<Diagnostic>,
    /// Standard output captured during execution
    pub stdout: String,
}

/// REPL core state
///
/// Maintains persistent state across multiple eval calls:
/// - Variable and function declarations persist
/// - Errors do not reset state
pub struct ReplCore {
    /// Interpreter state (variables, functions)
    interpreter: Interpreter,
    /// Symbol table (type information)
    symbol_table: SymbolTable,
    /// Security context for permission checks
    security: SecurityContext,
}

impl ReplCore {
    /// Create a new REPL core
    pub fn new() -> Self {
        Self::new_with_security(SecurityContext::allow_all())
    }

    /// Create a new REPL core with specific security context
    pub fn new_with_security(security: SecurityContext) -> Self {
        Self {
            interpreter: Interpreter::new(),
            symbol_table: SymbolTable::new(),
            security,
        }
    }

    /// Evaluate a line of input
    ///
    /// Runs the full pipeline: lex -> parse -> bind -> typecheck -> eval
    /// State persists across calls - variables and functions remain defined
    pub fn eval_line(&mut self, input: &str) -> ReplResult {
        let mut diagnostics = Vec::new();

        // Phase 1: Lex
        let mut lexer = Lexer::new(input.to_string());
        let (tokens, lex_diags) = lexer.tokenize();
        diagnostics.extend(lex_diags);

        if !diagnostics.is_empty() {
            return ReplResult {
                value: None,
                diagnostics,
                stdout: String::new(),
            };
        }

        // Phase 2: Parse
        let mut parser = Parser::new(tokens);
        let (ast, parse_diags) = parser.parse();
        diagnostics.extend(parse_diags);

        if !diagnostics.is_empty() {
            return ReplResult {
                value: None,
                diagnostics,
                stdout: String::new(),
            };
        }

        // Phase 3: Bind (using existing symbol table for state persistence)
        let mut binder = Binder::with_symbol_table(self.symbol_table.clone());
        let (updated_symbols, bind_diags) = binder.bind(&ast);
        diagnostics.extend(bind_diags);

        // Replace symbol table with updated one
        self.symbol_table = updated_symbols;

        if !diagnostics.is_empty() {
            return ReplResult {
                value: None,
                diagnostics,
                stdout: String::new(),
            };
        }

        // Phase 4: Typecheck
        let mut typechecker = TypeChecker::new(&mut self.symbol_table);
        let typecheck_diags = typechecker.check(&ast);
        diagnostics.extend(typecheck_diags);

        if !diagnostics.is_empty() {
            return ReplResult {
                value: None,
                diagnostics,
                stdout: String::new(),
            };
        }

        // Phase 5: Evaluate
        match self.interpreter.eval(&ast, &self.security) {
            Ok(value) => ReplResult {
                value: Some(value),
                diagnostics,
                stdout: String::new(), // TODO: Capture stdout
            },
            Err(e) => {
                use crate::span::Span;
                diagnostics.push(Diagnostic::error(
                    format!("Runtime error: {:?}", e),
                    Span::dummy(),
                ));
                ReplResult {
                    value: None,
                    diagnostics,
                    stdout: String::new(),
                }
            }
        }
    }

    /// Reset REPL state
    ///
    /// Clears all variables, functions, and type information
    pub fn reset(&mut self) {
        self.interpreter = Interpreter::new();
        self.symbol_table = SymbolTable::new();
    }
}

impl Default for ReplCore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repl_creation() {
        let mut repl = ReplCore::new();
        let result = repl.eval_line("1 + 1;");
        // Should evaluate successfully
        assert!(
            result.diagnostics.is_empty(),
            "Diagnostics: {:?}",
            result.diagnostics
        );
        assert!(result.value.is_some());
        assert_eq!(result.value.unwrap(), Value::Number(2.0));
    }

    #[test]
    fn test_variable_persistence() {
        let mut repl = ReplCore::new();

        // Declare a variable
        let result = repl.eval_line("let x = 42;");
        assert!(
            result.diagnostics.is_empty(),
            "Diagnostics: {:?}",
            result.diagnostics
        );

        // Use the variable in a subsequent eval
        let result = repl.eval_line("x + 8;");
        assert!(
            result.diagnostics.is_empty(),
            "Diagnostics: {:?}",
            result.diagnostics
        );
        assert_eq!(result.value.unwrap(), Value::Number(50.0));
    }

    #[test]
    fn test_mutable_variable_persistence() {
        let mut repl = ReplCore::new();

        // Declare a mutable variable
        let result = repl.eval_line("var y = 10;");
        assert!(
            result.diagnostics.is_empty(),
            "Diagnostics: {:?}",
            result.diagnostics
        );

        // Mutate it
        let result = repl.eval_line("y = y + 5;");
        assert!(
            result.diagnostics.is_empty(),
            "Diagnostics: {:?}",
            result.diagnostics
        );

        // Check new value
        let result = repl.eval_line("y;");
        assert!(
            result.diagnostics.is_empty(),
            "Diagnostics: {:?}",
            result.diagnostics
        );
        assert_eq!(result.value.unwrap(), Value::Number(15.0));
    }

    #[test]
    fn test_function_persistence() {
        let mut repl = ReplCore::new();

        // Declare a function
        let result = repl.eval_line("fn double(x: number) -> number { return x * 2; }");
        assert!(
            result.diagnostics.is_empty(),
            "Diagnostics: {:?}",
            result.diagnostics
        );

        // Call the function
        let result = repl.eval_line("double(21);");
        assert!(
            result.diagnostics.is_empty(),
            "Diagnostics: {:?}",
            result.diagnostics
        );
        assert_eq!(result.value.unwrap(), Value::Number(42.0));
    }

    #[test]
    fn test_error_does_not_reset_state() {
        let mut repl = ReplCore::new();

        // Declare a variable
        let result = repl.eval_line("let x = 100;");
        assert!(
            result.diagnostics.is_empty(),
            "Diagnostics: {:?}",
            result.diagnostics
        );

        // Cause an error (type error)
        let result = repl.eval_line("x + \"hello\";");
        assert!(!result.diagnostics.is_empty(), "Should have type error");

        // Variable should still be accessible
        let result = repl.eval_line("x;");
        assert!(
            result.diagnostics.is_empty(),
            "Diagnostics: {:?}",
            result.diagnostics
        );
        assert_eq!(result.value.unwrap(), Value::Number(100.0));
    }

    #[test]
    fn test_runtime_error_does_not_reset_state() {
        let mut repl = ReplCore::new();

        // Declare a variable
        let result = repl.eval_line("let arr = [1, 2, 3];");
        assert!(
            result.diagnostics.is_empty(),
            "Diagnostics: {:?}",
            result.diagnostics
        );

        // Cause a runtime error (out of bounds)
        let result = repl.eval_line("arr[10];");
        assert!(!result.diagnostics.is_empty(), "Should have runtime error");

        // Variable should still be accessible
        let result = repl.eval_line("arr[0];");
        assert!(
            result.diagnostics.is_empty(),
            "Diagnostics: {:?}",
            result.diagnostics
        );
        assert_eq!(result.value.unwrap(), Value::Number(1.0));
    }

    #[test]
    fn test_multi_statement_input() {
        let mut repl = ReplCore::new();

        // Multiple statements in one input
        let result = repl.eval_line("let a = 5; let b = 10; a + b;");
        assert!(
            result.diagnostics.is_empty(),
            "Diagnostics: {:?}",
            result.diagnostics
        );
        assert_eq!(result.value.unwrap(), Value::Number(15.0));
    }

    #[test]
    fn test_expression_vs_statement() {
        let mut repl = ReplCore::new();

        // Expression - should return value
        let result = repl.eval_line("2 + 3;");
        assert!(
            result.diagnostics.is_empty(),
            "Diagnostics: {:?}",
            result.diagnostics
        );
        assert!(result.value.is_some());
        assert_eq!(result.value.unwrap(), Value::Number(5.0));

        // Statement - may not return value
        let result = repl.eval_line("let x = 10;");
        assert!(
            result.diagnostics.is_empty(),
            "Diagnostics: {:?}",
            result.diagnostics
        );
        // Variable declarations typically don't return values in the REPL
    }

    #[test]
    fn test_reset_clears_state() {
        let mut repl = ReplCore::new();

        // Declare a variable
        let result = repl.eval_line("let x = 42;");
        assert!(
            result.diagnostics.is_empty(),
            "Diagnostics: {:?}",
            result.diagnostics
        );

        // Reset
        repl.reset();

        // Variable should no longer exist
        let result = repl.eval_line("x;");
        assert!(
            !result.diagnostics.is_empty(),
            "Should have error - variable undefined"
        );
    }

    #[test]
    fn test_complex_interactions() {
        let mut repl = ReplCore::new();

        // Build up a more complex program step by step
        let result = repl.eval_line("fn add(a: number, b: number) -> number { return a + b; }");
        assert!(
            result.diagnostics.is_empty(),
            "Diagnostics: {:?}",
            result.diagnostics
        );

        let result = repl.eval_line("let x = 10;");
        assert!(
            result.diagnostics.is_empty(),
            "Diagnostics: {:?}",
            result.diagnostics
        );

        let result = repl.eval_line("var y = 20;");
        assert!(
            result.diagnostics.is_empty(),
            "Diagnostics: {:?}",
            result.diagnostics
        );

        let result = repl.eval_line("let result = add(x, y);");
        assert!(
            result.diagnostics.is_empty(),
            "Diagnostics: {:?}",
            result.diagnostics
        );

        let result = repl.eval_line("result;");
        assert!(
            result.diagnostics.is_empty(),
            "Diagnostics: {:?}",
            result.diagnostics
        );
        assert_eq!(result.value.unwrap(), Value::Number(30.0));

        // Modify y
        let result = repl.eval_line("y = 50;");
        assert!(
            result.diagnostics.is_empty(),
            "Diagnostics: {:?}",
            result.diagnostics
        );

        // Call function again with new value
        let result = repl.eval_line("add(x, y);");
        assert!(
            result.diagnostics.is_empty(),
            "Diagnostics: {:?}",
            result.diagnostics
        );
        assert_eq!(result.value.unwrap(), Value::Number(60.0));
    }

    #[test]
    fn test_string_values() {
        use std::sync::Arc;
        let mut repl = ReplCore::new();

        let result = repl.eval_line("\"hello\";");
        assert!(
            result.diagnostics.is_empty(),
            "Diagnostics: {:?}",
            result.diagnostics
        );
        assert_eq!(
            result.value.unwrap(),
            Value::String(Arc::new("hello".to_string()))
        );

        let result = repl.eval_line("\"hello\" + \" world\";");
        assert!(
            result.diagnostics.is_empty(),
            "Diagnostics: {:?}",
            result.diagnostics
        );
        assert_eq!(
            result.value.unwrap(),
            Value::String(Arc::new("hello world".to_string()))
        );
    }

    #[test]
    fn test_boolean_values() {
        let mut repl = ReplCore::new();

        let result = repl.eval_line("true;");
        assert!(
            result.diagnostics.is_empty(),
            "Diagnostics: {:?}",
            result.diagnostics
        );
        assert_eq!(result.value.unwrap(), Value::Bool(true));

        let result = repl.eval_line("1 < 2;");
        assert!(
            result.diagnostics.is_empty(),
            "Diagnostics: {:?}",
            result.diagnostics
        );
        assert_eq!(result.value.unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_array_persistence() {
        let mut repl = ReplCore::new();

        let result = repl.eval_line("var arr = [1, 2, 3];");
        assert!(
            result.diagnostics.is_empty(),
            "Diagnostics: {:?}",
            result.diagnostics
        );

        let result = repl.eval_line("arr[1];");
        assert!(
            result.diagnostics.is_empty(),
            "Diagnostics: {:?}",
            result.diagnostics
        );
        assert_eq!(result.value.unwrap(), Value::Number(2.0));

        // Mutate array
        let result = repl.eval_line("arr[1] = 42;");
        assert!(
            result.diagnostics.is_empty(),
            "Diagnostics: {:?}",
            result.diagnostics
        );

        // Check mutation persisted
        let result = repl.eval_line("arr[1];");
        assert!(
            result.diagnostics.is_empty(),
            "Diagnostics: {:?}",
            result.diagnostics
        );
        assert_eq!(result.value.unwrap(), Value::Number(42.0));
    }
}
