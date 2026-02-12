//! REPL core logic (UI-agnostic)

use crate::binder::Binder;
use crate::diagnostic::Diagnostic;
use crate::interpreter::Interpreter;
use crate::lexer::Lexer;
use crate::parser::Parser;
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
}

impl ReplCore {
    /// Create a new REPL core
    pub fn new() -> Self {
        Self {
            interpreter: Interpreter::new(),
            symbol_table: SymbolTable::new(),
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
        let mut typechecker = TypeChecker::new(&self.symbol_table);
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
        match self.interpreter.eval(&ast) {
            Ok(value) => ReplResult {
                value: Some(value),
                diagnostics,
                stdout: String::new(), // TODO: Capture stdout
            },
            Err(e) => {
                use crate::span::Span;
                diagnostics.push(Diagnostic::error(format!("Runtime error: {:?}", e), Span::dummy()));
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
        assert!(result.diagnostics.is_empty(), "Diagnostics: {:?}", result.diagnostics);
        assert!(result.value.is_some());
        assert_eq!(result.value.unwrap(), Value::Number(2.0));
    }
}
