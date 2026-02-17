//! Variable inspection and expression evaluation for the Atlas debugger.
//!
//! Provides scoped variable collection, formatted output, watch expressions,
//! and hover evaluation for the debugger.

use crate::debugger::protocol::Variable;
use crate::interpreter::Interpreter;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::security::SecurityContext;
use crate::value::Value;
use crate::vm::VM;

// ── VariableScope ────────────────────────────────────────────────────────────

/// The scope a variable belongs to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VariableScope {
    /// Local variable in the current function frame.
    Local,
    /// Global variable.
    Global,
}

// ── ScopedVariable ───────────────────────────────────────────────────────────

/// A variable with scope information.
#[derive(Debug, Clone, PartialEq)]
pub struct ScopedVariable {
    /// Protocol-level variable (name, value, type_name).
    pub variable: Variable,
    /// Scope the variable belongs to.
    pub scope: VariableScope,
}

impl ScopedVariable {
    /// Create a new scoped variable.
    pub fn new(variable: Variable, scope: VariableScope) -> Self {
        Self { variable, scope }
    }
}

// ── Inspector ────────────────────────────────────────────────────────────────

/// Inspects VM state to collect variables and evaluate expressions.
pub struct Inspector {
    /// Watch expressions that are evaluated on each pause.
    watch_expressions: Vec<String>,
    /// Maximum depth for formatting nested values.
    max_format_depth: usize,
}

impl Inspector {
    /// Create a new inspector.
    pub fn new() -> Self {
        Self {
            watch_expressions: Vec::new(),
            max_format_depth: 3,
        }
    }

    /// Set the maximum format depth for nested values.
    pub fn set_max_format_depth(&mut self, depth: usize) {
        self.max_format_depth = depth;
    }

    /// Get the maximum format depth.
    pub fn max_format_depth(&self) -> usize {
        self.max_format_depth
    }

    /// Add a watch expression.
    pub fn add_watch(&mut self, expression: String) {
        if !self.watch_expressions.contains(&expression) {
            self.watch_expressions.push(expression);
        }
    }

    /// Remove a watch expression.
    pub fn remove_watch(&mut self, expression: &str) -> bool {
        let len_before = self.watch_expressions.len();
        self.watch_expressions.retain(|e| e != expression);
        self.watch_expressions.len() < len_before
    }

    /// Clear all watch expressions.
    pub fn clear_watches(&mut self) {
        self.watch_expressions.clear();
    }

    /// Get all watch expressions.
    pub fn watch_expressions(&self) -> &[String] {
        &self.watch_expressions
    }

    /// Collect variables with scope information from a VM frame.
    pub fn collect_scoped_variables(&self, vm: &VM, frame_index: usize) -> Vec<ScopedVariable> {
        let mut vars = Vec::new();

        // Locals from the requested frame
        for (slot, value) in vm.get_locals_for_frame(frame_index) {
            vars.push(ScopedVariable::new(
                Variable::new(
                    format!("local_{slot}"),
                    format_value_with_depth(value, self.max_format_depth),
                    value.type_name(),
                ),
                VariableScope::Local,
            ));
        }

        // Globals
        for (name, value) in vm.get_global_variables() {
            vars.push(ScopedVariable::new(
                Variable::new(
                    name.clone(),
                    format_value_with_depth(value, self.max_format_depth),
                    value.type_name(),
                ),
                VariableScope::Global,
            ));
        }

        vars.sort_by(|a, b| a.variable.name.cmp(&b.variable.name));
        vars
    }

    /// Collect only local variables for a frame.
    pub fn collect_locals(&self, vm: &VM, frame_index: usize) -> Vec<Variable> {
        vm.get_locals_for_frame(frame_index)
            .into_iter()
            .map(|(slot, value)| {
                Variable::new(
                    format!("local_{slot}"),
                    format_value_with_depth(value, self.max_format_depth),
                    value.type_name(),
                )
            })
            .collect()
    }

    /// Collect only global variables.
    pub fn collect_globals(&self, vm: &VM) -> Vec<Variable> {
        let mut vars: Vec<Variable> = vm
            .get_global_variables()
            .iter()
            .map(|(name, value)| {
                Variable::new(
                    name.clone(),
                    format_value_with_depth(value, self.max_format_depth),
                    value.type_name(),
                )
            })
            .collect();
        vars.sort_by(|a, b| a.name.cmp(&b.name));
        vars
    }

    /// Evaluate an expression in the context of visible variables.
    ///
    /// Injects variable bindings as `let` statements before the expression.
    pub fn evaluate_expression(&self, expression: &str, variables: &[Variable]) -> EvalResult {
        let mut snippet = String::new();
        for var in variables {
            if is_valid_identifier(&var.name) {
                if let Some(lit) = value_to_atlas_literal(&var.type_name, &var.value) {
                    snippet.push_str(&format!("let {} = {};\n", var.name, lit));
                }
            }
        }
        snippet.push_str(expression);
        let trimmed = expression.trim();
        if !trimmed.ends_with(';') && !trimmed.ends_with('}') {
            snippet.push(';');
        }

        let tokens = Lexer::new(&snippet).tokenize().0;
        let (ast, errors) = Parser::new(tokens).parse();
        if !errors.is_empty() {
            return EvalResult::Error(format!("parse error: {:?}", errors[0]));
        }

        let mut interp = Interpreter::new();
        let security = SecurityContext::allow_all();
        match interp.eval(&ast, &security) {
            Ok(value) => EvalResult::Success {
                value: format_value_with_depth(&value, self.max_format_depth),
                type_name: value.type_name().to_string(),
            },
            Err(e) => EvalResult::Error(format!("{:?}", e)),
        }
    }

    /// Evaluate all watch expressions and return results.
    pub fn evaluate_watches(&self, variables: &[Variable]) -> Vec<WatchResult> {
        self.watch_expressions
            .iter()
            .map(|expr| {
                let result = self.evaluate_expression(expr, variables);
                WatchResult {
                    expression: expr.clone(),
                    result,
                }
            })
            .collect()
    }

    /// Quick hover evaluation: just look up a variable by name.
    pub fn hover(&self, name: &str, variables: &[Variable]) -> Option<Variable> {
        variables.iter().find(|v| v.name == name).cloned()
    }
}

impl Default for Inspector {
    fn default() -> Self {
        Self::new()
    }
}

// ── EvalResult ───────────────────────────────────────────────────────────────

/// Result of evaluating an expression.
#[derive(Debug, Clone, PartialEq)]
pub enum EvalResult {
    /// Successful evaluation.
    Success { value: String, type_name: String },
    /// Evaluation error.
    Error(String),
}

// ── WatchResult ──────────────────────────────────────────────────────────────

/// Result of a watch expression evaluation.
#[derive(Debug, Clone, PartialEq)]
pub struct WatchResult {
    /// The watch expression.
    pub expression: String,
    /// The evaluation result.
    pub result: EvalResult,
}

// ── Formatting helpers ───────────────────────────────────────────────────────

/// Format a `Value` for display with depth control.
pub fn format_value_with_depth(value: &Value, max_depth: usize) -> String {
    format_value_recursive(value, 0, max_depth)
}

fn format_value_recursive(value: &Value, depth: usize, max_depth: usize) -> String {
    if depth > max_depth {
        return "...".to_string();
    }
    match value {
        Value::Number(n) => {
            if n.fract() == 0.0 && n.abs() < 1e15 {
                format!("{}", *n as i64)
            } else {
                format!("{n}")
            }
        }
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        Value::String(s) => format!("\"{}\"", s.as_ref()),
        Value::Array(arr) => {
            let guard = arr.lock().unwrap();
            if depth >= max_depth {
                return format!("[{} items]", guard.len());
            }
            let items: Vec<String> = guard
                .iter()
                .take(10)
                .map(|v| format_value_recursive(v, depth + 1, max_depth))
                .collect();
            if guard.len() > 10 {
                format!("[{}, ... +{} more]", items.join(", "), guard.len() - 10)
            } else {
                format!("[{}]", items.join(", "))
            }
        }
        Value::HashMap(m) => {
            let guard = m.lock().unwrap();
            format!("{{HashMap, {} entries}}", guard.len())
        }
        Value::HashSet(s) => {
            let guard = s.lock().unwrap();
            format!("{{HashSet, {} items}}", guard.len())
        }
        Value::Queue(q) => {
            let guard = q.lock().unwrap();
            format!("[Queue, {} items]", guard.len())
        }
        Value::Stack(s) => {
            let guard = s.lock().unwrap();
            format!("[Stack, {} items]", guard.len())
        }
        Value::Function(f) => format!("<fn {}>", f.name),
        _ => format!("{:?}", value),
    }
}

/// Check if a string is a valid Atlas identifier.
fn is_valid_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_alphanumeric() || c == '_')
}

/// Try to produce an Atlas literal from type_name + display value.
fn value_to_atlas_literal(type_name: &str, display: &str) -> Option<String> {
    match type_name {
        "number" => {
            display.parse::<f64>().ok()?;
            Some(display.to_string())
        }
        "bool" => Some(display.to_string()),
        "null" => Some("null".to_string()),
        "string" => Some(display.to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inspector_new() {
        let inspector = Inspector::new();
        assert_eq!(inspector.max_format_depth(), 3);
        assert!(inspector.watch_expressions().is_empty());
    }

    #[test]
    fn test_set_max_format_depth() {
        let mut inspector = Inspector::new();
        inspector.set_max_format_depth(5);
        assert_eq!(inspector.max_format_depth(), 5);
    }

    #[test]
    fn test_add_watch() {
        let mut inspector = Inspector::new();
        inspector.add_watch("x + 1".into());
        assert_eq!(inspector.watch_expressions(), &["x + 1"]);
    }

    #[test]
    fn test_add_duplicate_watch() {
        let mut inspector = Inspector::new();
        inspector.add_watch("x".into());
        inspector.add_watch("x".into());
        assert_eq!(inspector.watch_expressions().len(), 1);
    }

    #[test]
    fn test_remove_watch() {
        let mut inspector = Inspector::new();
        inspector.add_watch("x".into());
        assert!(inspector.remove_watch("x"));
        assert!(inspector.watch_expressions().is_empty());
    }

    #[test]
    fn test_remove_nonexistent_watch() {
        let mut inspector = Inspector::new();
        assert!(!inspector.remove_watch("y"));
    }

    #[test]
    fn test_clear_watches() {
        let mut inspector = Inspector::new();
        inspector.add_watch("x".into());
        inspector.add_watch("y".into());
        inspector.clear_watches();
        assert!(inspector.watch_expressions().is_empty());
    }

    #[test]
    fn test_eval_simple_expression() {
        let inspector = Inspector::new();
        let result = inspector.evaluate_expression("1 + 2", &[]);
        match result {
            EvalResult::Success { value, type_name } => {
                assert_eq!(type_name, "number");
                assert!(value.contains('3'));
            }
            EvalResult::Error(e) => panic!("unexpected error: {}", e),
        }
    }

    #[test]
    fn test_eval_with_variables() {
        let inspector = Inspector::new();
        let vars = vec![Variable::new("x", "10", "number")];
        let result = inspector.evaluate_expression("x + 5", &vars);
        match result {
            EvalResult::Success { value, type_name } => {
                assert_eq!(type_name, "number");
                assert!(value.contains("15"));
            }
            EvalResult::Error(e) => panic!("unexpected error: {}", e),
        }
    }

    #[test]
    fn test_eval_string_expression() {
        let inspector = Inspector::new();
        let result = inspector.evaluate_expression(r#""hello" + " world""#, &[]);
        match result {
            EvalResult::Success { type_name, .. } => {
                assert_eq!(type_name, "string");
            }
            EvalResult::Error(e) => panic!("unexpected error: {}", e),
        }
    }

    #[test]
    fn test_eval_invalid_expression() {
        let inspector = Inspector::new();
        let result = inspector.evaluate_expression("!!!invalid", &[]);
        match result {
            EvalResult::Error(_) => {}
            EvalResult::Success { .. } => panic!("expected error"),
        }
    }

    #[test]
    fn test_eval_with_bool_variable() {
        let inspector = Inspector::new();
        let vars = vec![Variable::new("flag", "true", "bool")];
        let result = inspector.evaluate_expression("flag", &vars);
        match result {
            EvalResult::Success { value, type_name } => {
                assert_eq!(type_name, "bool");
                assert!(value.contains("true"));
            }
            EvalResult::Error(e) => panic!("unexpected error: {}", e),
        }
    }

    #[test]
    fn test_evaluate_watches() {
        let mut inspector = Inspector::new();
        inspector.add_watch("1 + 1".into());
        inspector.add_watch("2 * 3".into());
        let results = inspector.evaluate_watches(&[]);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].expression, "1 + 1");
    }

    #[test]
    fn test_hover_found() {
        let inspector = Inspector::new();
        let vars = vec![
            Variable::new("x", "42", "number"),
            Variable::new("y", "\"hello\"", "string"),
        ];
        let result = inspector.hover("x", &vars);
        assert!(result.is_some());
        assert_eq!(result.unwrap().value, "42");
    }

    #[test]
    fn test_hover_not_found() {
        let inspector = Inspector::new();
        let result = inspector.hover("z", &[]);
        assert!(result.is_none());
    }

    #[test]
    fn test_format_number_integer() {
        let val = Value::Number(42.0);
        assert_eq!(format_value_with_depth(&val, 3), "42");
    }

    #[test]
    fn test_format_number_float() {
        let val = Value::Number(3.14);
        assert_eq!(format_value_with_depth(&val, 3), "3.14");
    }

    #[test]
    fn test_format_bool() {
        assert_eq!(format_value_with_depth(&Value::Bool(true), 3), "true");
    }

    #[test]
    fn test_format_null() {
        assert_eq!(format_value_with_depth(&Value::Null, 3), "null");
    }

    #[test]
    fn test_format_string() {
        let val = Value::String(std::sync::Arc::new("hello".to_string()));
        assert_eq!(format_value_with_depth(&val, 3), "\"hello\"");
    }

    #[test]
    fn test_format_depth_exceeded() {
        let val = Value::Number(1.0);
        assert_eq!(format_value_recursive(&val, 5, 3), "...");
    }

    #[test]
    fn test_is_valid_identifier() {
        assert!(is_valid_identifier("x"));
        assert!(is_valid_identifier("_foo"));
        assert!(is_valid_identifier("hello_world"));
        assert!(!is_valid_identifier(""));
        assert!(!is_valid_identifier("123abc"));
        assert!(!is_valid_identifier("a-b"));
    }

    #[test]
    fn test_value_to_literal_number() {
        assert_eq!(value_to_atlas_literal("number", "42"), Some("42".into()));
    }

    #[test]
    fn test_value_to_literal_bool() {
        assert_eq!(value_to_atlas_literal("bool", "true"), Some("true".into()));
    }

    #[test]
    fn test_value_to_literal_null() {
        assert_eq!(value_to_atlas_literal("null", "null"), Some("null".into()));
    }

    #[test]
    fn test_value_to_literal_complex_returns_none() {
        assert_eq!(value_to_atlas_literal("array", "[1,2]"), None);
    }

    #[test]
    fn test_scoped_variable() {
        let var = Variable::new("x", "42", "number");
        let scoped = ScopedVariable::new(var.clone(), VariableScope::Local);
        assert_eq!(scoped.scope, VariableScope::Local);
        assert_eq!(scoped.variable, var);
    }
}
