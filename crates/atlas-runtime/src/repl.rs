//! REPL core logic (UI-agnostic)

use crate::ast::{Item, Stmt};
use crate::binder::Binder;
use crate::diagnostic::Diagnostic;
use crate::interpreter::Interpreter;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::security::SecurityContext;
use crate::symbol::{SymbolKind, SymbolTable};
use crate::typechecker::TypeChecker;
use crate::types::Type;
use crate::value::Value;

/// A captured variable binding for REPL display
#[derive(Debug, Clone)]
pub struct ReplBinding {
    /// Variable name
    pub name: String,
    /// Inferred or declared type
    pub ty: Type,
    /// Current value in the interpreter environment
    pub value: Value,
    /// Whether the variable is mutable
    pub mutable: bool,
}

/// Result of a type-only REPL query (e.g., :type command)
pub struct TypeQueryResult {
    /// Inferred type of the expression (None if errors)
    pub ty: Option<Type>,
    /// Diagnostics produced during lex/parse/bind/typecheck
    pub diagnostics: Vec<Diagnostic>,
}

/// REPL result type
pub struct ReplResult {
    /// The value produced by evaluation (None if statement or error)
    pub value: Option<Value>,
    /// Diagnostics from all phases
    pub diagnostics: Vec<Diagnostic>,
    /// Standard output captured during execution
    pub stdout: String,
    /// Type of the last expression statement (if any)
    pub expr_type: Option<Type>,
    /// Any variable bindings created by this line (for richer feedback)
    pub bindings: Vec<ReplBinding>,
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

    /// Perform type checking only (no evaluation) for a single expression input.
    /// This is used by REPL commands like `:type` to display inferred types without
    /// mutating the current interpreter or symbol table state.
    pub fn type_of_expression(&self, input: &str) -> TypeQueryResult {
        let mut diagnostics = Vec::new();

        let mut lexer = Lexer::new(input.to_string());
        let (tokens, lex_diags) = lexer.tokenize();
        diagnostics.extend(lex_diags);

        if !diagnostics.is_empty() {
            return TypeQueryResult {
                ty: None,
                diagnostics,
            };
        }

        let mut parser = Parser::new(tokens);
        let (ast, parse_diags) = parser.parse();
        diagnostics.extend(parse_diags);

        if !diagnostics.is_empty() {
            return TypeQueryResult {
                ty: None,
                diagnostics,
            };
        }

        // Bind using a clone so we don't mutate live REPL state
        let mut binder = Binder::with_symbol_table(self.symbol_table.clone());
        let (mut bound_symbols, bind_diags) = binder.bind(&ast);
        diagnostics.extend(bind_diags);

        if !diagnostics.is_empty() {
            return TypeQueryResult {
                ty: None,
                diagnostics,
            };
        }

        let mut typechecker = TypeChecker::new(&mut bound_symbols);
        let type_diags = typechecker.check(&ast);
        diagnostics.extend(type_diags);

        if !diagnostics.is_empty() {
            return TypeQueryResult {
                ty: None,
                diagnostics,
            };
        }

        TypeQueryResult {
            ty: typechecker.last_expression_type(),
            diagnostics,
        }
    }

    /// Snapshot all current variables (name, type, value) sorted alphabetically.
    pub fn variables(&self) -> Vec<ReplBinding> {
        let mut vars = Vec::new();
        for symbol in self.symbol_table.all_symbols() {
            if symbol.kind != SymbolKind::Variable {
                continue;
            }
            if let Some(value) = self.interpreter.get_binding(&symbol.name) {
                vars.push(ReplBinding {
                    name: symbol.name.clone(),
                    ty: symbol.ty.clone(),
                    value,
                    mutable: symbol.mutable,
                });
            }
        }
        vars.sort_by(|a, b| a.name.cmp(&b.name));
        vars
    }

    /// Evaluate a line of input
    ///
    /// Runs the full pipeline: lex -> parse -> bind -> typecheck -> eval
    /// State persists across calls - variables and functions remain defined
    pub fn eval_line(&mut self, input: &str) -> ReplResult {
        let mut diagnostics = Vec::new();
        let mut expr_type: Option<Type> = None;
        let bindings: Vec<ReplBinding> = Vec::new();

        // Phase 1: Lex
        let mut lexer = Lexer::new(input.to_string());
        let (tokens, lex_diags) = lexer.tokenize();
        diagnostics.extend(lex_diags);

        if !diagnostics.is_empty() {
            return ReplResult {
                value: None,
                diagnostics,
                stdout: String::new(),
                expr_type,
                bindings,
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
                expr_type,
                bindings,
            };
        }

        // Track variables declared in this input for richer feedback
        let declared_vars = collect_declared_vars(&ast);

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
                expr_type,
                bindings,
            };
        }

        // Phase 4: Typecheck
        let mut typechecker = TypeChecker::new(&mut self.symbol_table);
        let typecheck_diags = typechecker.check(&ast);
        diagnostics.extend(typecheck_diags);
        expr_type = typechecker.last_expression_type();

        if !diagnostics.is_empty() {
            return ReplResult {
                value: None,
                diagnostics,
                stdout: String::new(),
                expr_type,
                bindings,
            };
        }

        // Phase 5: Evaluate
        match self.interpreter.eval(&ast, &self.security) {
            Ok(value) => ReplResult {
                value: Some(value),
                diagnostics,
                stdout: String::new(), // TODO: Capture stdout
                expr_type,
                bindings: self.collect_bindings(&declared_vars),
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
                    expr_type,
                    bindings,
                }
            }
        }
    }

    /// Build binding metadata for variables declared in the current input.
    fn collect_bindings(&self, declared_vars: &[String]) -> Vec<ReplBinding> {
        let mut results = Vec::new();
        for name in declared_vars {
            if let Some(symbol) = self.symbol_table.lookup(name) {
                if let Some(value) = self.interpreter.get_binding(name) {
                    results.push(ReplBinding {
                        name: name.clone(),
                        ty: symbol.ty.clone(),
                        value,
                        mutable: symbol.mutable,
                    });
                }
            }
        }
        results
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

// ============================================================================
// Multiline Input Detection
// ============================================================================

/// Result of checking if input is complete.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputCompleteness {
    /// Input is complete and can be evaluated.
    Complete,
    /// Input is incomplete (more lines needed).
    Incomplete {
        /// Reason the input is incomplete.
        reason: IncompleteReason,
    },
}

/// Reason for incomplete input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IncompleteReason {
    /// Unclosed brace `{`.
    UnclosedBrace,
    /// Unclosed bracket `[`.
    UnclosedBracket,
    /// Unclosed parenthesis `(`.
    UnclosedParen,
    /// Unclosed string literal.
    UnclosedString,
    /// Unclosed multi-line comment.
    UnclosedComment,
}

impl IncompleteReason {
    /// Get a human-readable description.
    pub fn description(&self) -> &'static str {
        match self {
            IncompleteReason::UnclosedBrace => "unclosed brace '{'",
            IncompleteReason::UnclosedBracket => "unclosed bracket '['",
            IncompleteReason::UnclosedParen => "unclosed parenthesis '('",
            IncompleteReason::UnclosedString => "unclosed string literal",
            IncompleteReason::UnclosedComment => "unclosed multi-line comment",
        }
    }
}

/// Multiline input state for REPL.
pub struct MultilineInput {
    /// Accumulated lines.
    lines: Vec<String>,
}

impl MultilineInput {
    /// Create a new multiline input handler.
    pub fn new() -> Self {
        Self { lines: Vec::new() }
    }

    /// Add a line to the input.
    pub fn add_line(&mut self, line: &str) {
        self.lines.push(line.to_string());
    }

    /// Check if the accumulated input is complete.
    pub fn check_completeness(&self) -> InputCompleteness {
        let combined = self.lines.join("\n");
        is_input_complete(&combined)
    }

    /// Get the combined input.
    pub fn combined(&self) -> String {
        self.lines.join("\n")
    }

    /// Clear the accumulated input.
    pub fn clear(&mut self) {
        self.lines.clear();
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// Number of lines accumulated.
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }
}

impl Default for MultilineInput {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if input is complete (all delimiters are balanced).
pub fn is_input_complete(input: &str) -> InputCompleteness {
    let mut brace_depth = 0i32;
    let mut bracket_depth = 0i32;
    let mut paren_depth = 0i32;
    let mut in_string = false;
    let mut in_char = false;
    let mut in_line_comment = false;
    let mut in_block_comment = false;
    let mut escape_next = false;

    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let ch = chars[i];
        let next = chars.get(i + 1).copied();

        // Handle escape sequences in strings
        if escape_next {
            escape_next = false;
            i += 1;
            continue;
        }

        // Handle different states
        if in_line_comment {
            if ch == '\n' {
                in_line_comment = false;
            }
            i += 1;
            continue;
        }

        if in_block_comment {
            if ch == '*' && next == Some('/') {
                in_block_comment = false;
                i += 2;
                continue;
            }
            i += 1;
            continue;
        }

        if in_string {
            if ch == '\\' {
                escape_next = true;
            } else if ch == '"' {
                in_string = false;
            }
            i += 1;
            continue;
        }

        if in_char {
            if ch == '\\' {
                escape_next = true;
            } else if ch == '\'' {
                in_char = false;
            }
            i += 1;
            continue;
        }

        // Not in any special state - check for delimiters
        match ch {
            '/' if next == Some('/') => {
                in_line_comment = true;
                i += 2;
                continue;
            }
            '/' if next == Some('*') => {
                in_block_comment = true;
                i += 2;
                continue;
            }
            '"' => in_string = true,
            '\'' => in_char = true,
            '{' => brace_depth += 1,
            '}' => brace_depth -= 1,
            '[' => bracket_depth += 1,
            ']' => bracket_depth -= 1,
            '(' => paren_depth += 1,
            ')' => paren_depth -= 1,
            _ => {}
        }

        i += 1;
    }

    // Check for incomplete states
    if in_string {
        return InputCompleteness::Incomplete {
            reason: IncompleteReason::UnclosedString,
        };
    }

    if in_block_comment {
        return InputCompleteness::Incomplete {
            reason: IncompleteReason::UnclosedComment,
        };
    }

    if brace_depth > 0 {
        return InputCompleteness::Incomplete {
            reason: IncompleteReason::UnclosedBrace,
        };
    }

    if bracket_depth > 0 {
        return InputCompleteness::Incomplete {
            reason: IncompleteReason::UnclosedBracket,
        };
    }

    if paren_depth > 0 {
        return InputCompleteness::Incomplete {
            reason: IncompleteReason::UnclosedParen,
        };
    }

    InputCompleteness::Complete
}

// ============================================================================
// File Loading
// ============================================================================

impl ReplCore {
    /// Load and execute an Atlas file in the REPL context.
    ///
    /// Variables and functions defined in the file persist in the REPL.
    ///
    /// # Arguments
    /// * `path` - Path to the Atlas source file
    ///
    /// # Returns
    /// * `Ok(ReplResult)` - Result of evaluating the file
    /// * `Err(String)` - Error message if file cannot be loaded
    pub fn load_file(&mut self, path: &std::path::Path) -> Result<ReplResult, String> {
        // Read the file
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file '{}': {}", path.display(), e))?;

        // Evaluate in REPL context
        Ok(self.eval_line(&content))
    }
}

/// Collect variable names declared in the parsed program (current REPL input).
fn collect_declared_vars(program: &crate::ast::Program) -> Vec<String> {
    let mut names = Vec::new();
    for item in &program.items {
        if let Item::Statement(Stmt::VarDecl(var)) = item {
            names.push(var.name.name.clone());
        }
    }
    names
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
