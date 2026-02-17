//! Debugger inspection tests — Phase 05.
//!
//! Tests variable inspection, expression evaluation, watch expressions,
//! hover, and the Inspector API.

use atlas_runtime::bytecode::Bytecode;
use atlas_runtime::compiler::Compiler;
use atlas_runtime::debugger::inspection::{
    format_value_with_depth, EvalResult, Inspector, ScopedVariable, VariableScope,
};
use atlas_runtime::debugger::protocol::{DebugRequest, DebugResponse, Variable};
use atlas_runtime::debugger::DebuggerSession;
use atlas_runtime::lexer::Lexer;
use atlas_runtime::parser::Parser;
use atlas_runtime::security::SecurityContext;
use atlas_runtime::value::Value;

fn compile(source: &str) -> Bytecode {
    let tokens = Lexer::new(source).tokenize().0;
    let (ast, _) = Parser::new(tokens).parse();
    let mut compiler = Compiler::new();
    compiler.compile(&ast).expect("compile failed")
}

fn _security() -> SecurityContext {
    SecurityContext::allow_all()
}

// ══════════════════════════════════════════════════════════════════════════════
// Inspector Unit Tests
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_inspector_default() {
    let inspector = Inspector::new();
    assert_eq!(inspector.max_format_depth(), 3);
    assert!(inspector.watch_expressions().is_empty());
}

#[test]
fn test_inspector_set_depth() {
    let mut inspector = Inspector::new();
    inspector.set_max_format_depth(5);
    assert_eq!(inspector.max_format_depth(), 5);
}

#[test]
fn test_inspector_add_watch() {
    let mut inspector = Inspector::new();
    inspector.add_watch("x + 1".into());
    assert_eq!(inspector.watch_expressions(), &["x + 1"]);
}

#[test]
fn test_inspector_add_duplicate_watch() {
    let mut inspector = Inspector::new();
    inspector.add_watch("x".into());
    inspector.add_watch("x".into());
    assert_eq!(inspector.watch_expressions().len(), 1);
}

#[test]
fn test_inspector_remove_watch() {
    let mut inspector = Inspector::new();
    inspector.add_watch("x".into());
    assert!(inspector.remove_watch("x"));
    assert!(inspector.watch_expressions().is_empty());
}

#[test]
fn test_inspector_remove_nonexistent_watch() {
    let mut inspector = Inspector::new();
    assert!(!inspector.remove_watch("y"));
}

#[test]
fn test_inspector_clear_watches() {
    let mut inspector = Inspector::new();
    inspector.add_watch("a".into());
    inspector.add_watch("b".into());
    inspector.clear_watches();
    assert!(inspector.watch_expressions().is_empty());
}

#[test]
fn test_inspector_multiple_watches() {
    let mut inspector = Inspector::new();
    inspector.add_watch("x".into());
    inspector.add_watch("y".into());
    inspector.add_watch("z".into());
    assert_eq!(inspector.watch_expressions().len(), 3);
}

// ══════════════════════════════════════════════════════════════════════════════
// Expression Evaluation
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_eval_simple_number() {
    let inspector = Inspector::new();
    match inspector.evaluate_expression("42", &[]) {
        EvalResult::Success { value, type_name } => {
            assert_eq!(type_name, "number");
            assert!(value.contains("42"));
        }
        EvalResult::Error(e) => panic!("error: {e}"),
    }
}

#[test]
fn test_eval_addition() {
    let inspector = Inspector::new();
    match inspector.evaluate_expression("1 + 2", &[]) {
        EvalResult::Success { value, type_name } => {
            assert_eq!(type_name, "number");
            assert!(value.contains('3'));
        }
        EvalResult::Error(e) => panic!("error: {e}"),
    }
}

#[test]
fn test_eval_string_concat() {
    let inspector = Inspector::new();
    match inspector.evaluate_expression(r#""hello" + " world""#, &[]) {
        EvalResult::Success { type_name, .. } => {
            assert_eq!(type_name, "string");
        }
        EvalResult::Error(e) => panic!("error: {e}"),
    }
}

#[test]
fn test_eval_boolean() {
    let inspector = Inspector::new();
    match inspector.evaluate_expression("true && false", &[]) {
        EvalResult::Success { value, type_name } => {
            assert_eq!(type_name, "bool");
            assert!(value.contains("false"));
        }
        EvalResult::Error(e) => panic!("error: {e}"),
    }
}

#[test]
fn test_eval_with_number_variable() {
    let inspector = Inspector::new();
    let vars = vec![Variable::new("x", "10", "number")];
    match inspector.evaluate_expression("x + 5", &vars) {
        EvalResult::Success { value, .. } => {
            assert!(value.contains("15"));
        }
        EvalResult::Error(e) => panic!("error: {e}"),
    }
}

#[test]
fn test_eval_with_bool_variable() {
    let inspector = Inspector::new();
    let vars = vec![Variable::new("flag", "true", "bool")];
    match inspector.evaluate_expression("flag", &vars) {
        EvalResult::Success { type_name, .. } => {
            assert_eq!(type_name, "bool");
        }
        EvalResult::Error(e) => panic!("error: {e}"),
    }
}

#[test]
fn test_eval_with_string_variable() {
    let inspector = Inspector::new();
    let vars = vec![Variable::new("name", "\"Atlas\"", "string")];
    match inspector.evaluate_expression("name", &vars) {
        EvalResult::Success { type_name, .. } => {
            assert_eq!(type_name, "string");
        }
        EvalResult::Error(e) => panic!("error: {e}"),
    }
}

#[test]
fn test_eval_with_null_variable() {
    let inspector = Inspector::new();
    let vars = vec![Variable::new("nothing", "null", "null")];
    match inspector.evaluate_expression("nothing", &vars) {
        EvalResult::Success { type_name, .. } => {
            assert_eq!(type_name, "null");
        }
        EvalResult::Error(e) => panic!("error: {e}"),
    }
}

#[test]
fn test_eval_invalid_syntax() {
    let inspector = Inspector::new();
    match inspector.evaluate_expression("!!!bad", &[]) {
        EvalResult::Error(_) => {}
        EvalResult::Success { .. } => panic!("expected error"),
    }
}

#[test]
fn test_eval_multiple_variables() {
    let inspector = Inspector::new();
    let vars = vec![
        Variable::new("a", "10", "number"),
        Variable::new("b", "20", "number"),
    ];
    match inspector.evaluate_expression("a + b", &vars) {
        EvalResult::Success { value, .. } => {
            assert!(value.contains("30"));
        }
        EvalResult::Error(e) => panic!("error: {e}"),
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// Watch Expressions
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_evaluate_watches_empty() {
    let inspector = Inspector::new();
    let results = inspector.evaluate_watches(&[]);
    assert!(results.is_empty());
}

#[test]
fn test_evaluate_watches_single() {
    let mut inspector = Inspector::new();
    inspector.add_watch("1 + 1".into());
    let results = inspector.evaluate_watches(&[]);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].expression, "1 + 1");
    match &results[0].result {
        EvalResult::Success { value, .. } => assert!(value.contains('2')),
        EvalResult::Error(e) => panic!("error: {e}"),
    }
}

#[test]
fn test_evaluate_watches_multiple() {
    let mut inspector = Inspector::new();
    inspector.add_watch("2 * 3".into());
    inspector.add_watch("true".into());
    let results = inspector.evaluate_watches(&[]);
    assert_eq!(results.len(), 2);
}

#[test]
fn test_evaluate_watches_with_error() {
    let mut inspector = Inspector::new();
    inspector.add_watch("!!!".into());
    let results = inspector.evaluate_watches(&[]);
    assert_eq!(results.len(), 1);
    match &results[0].result {
        EvalResult::Error(_) => {}
        _ => panic!("expected error"),
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// Hover
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_hover_found() {
    let inspector = Inspector::new();
    let vars = vec![Variable::new("x", "42", "number")];
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
fn test_hover_multiple_vars() {
    let inspector = Inspector::new();
    let vars = vec![
        Variable::new("x", "1", "number"),
        Variable::new("y", "2", "number"),
    ];
    let x = inspector.hover("x", &vars).unwrap();
    let y = inspector.hover("y", &vars).unwrap();
    assert_eq!(x.value, "1");
    assert_eq!(y.value, "2");
}

// ══════════════════════════════════════════════════════════════════════════════
// Value Formatting
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_format_integer() {
    assert_eq!(format_value_with_depth(&Value::Number(42.0), 3), "42");
}

#[test]
fn test_format_float() {
    assert_eq!(format_value_with_depth(&Value::Number(3.14), 3), "3.14");
}

#[test]
fn test_format_bool_true() {
    assert_eq!(format_value_with_depth(&Value::Bool(true), 3), "true");
}

#[test]
fn test_format_bool_false() {
    assert_eq!(format_value_with_depth(&Value::Bool(false), 3), "false");
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
fn test_format_empty_string() {
    let val = Value::String(std::sync::Arc::new(String::new()));
    assert_eq!(format_value_with_depth(&val, 3), "\"\"");
}

#[test]
fn test_format_negative_number() {
    assert_eq!(format_value_with_depth(&Value::Number(-5.0), 3), "-5");
}

#[test]
fn test_format_zero() {
    assert_eq!(format_value_with_depth(&Value::Number(0.0), 3), "0");
}

// ══════════════════════════════════════════════════════════════════════════════
// Scoped Variables
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_scoped_variable_local() {
    let var = Variable::new("x", "42", "number");
    let scoped = ScopedVariable::new(var.clone(), VariableScope::Local);
    assert_eq!(scoped.scope, VariableScope::Local);
    assert_eq!(scoped.variable, var);
}

#[test]
fn test_scoped_variable_global() {
    let var = Variable::new("PI", "3.14", "number");
    let scoped = ScopedVariable::new(var.clone(), VariableScope::Global);
    assert_eq!(scoped.scope, VariableScope::Global);
}

// ══════════════════════════════════════════════════════════════════════════════
// DebuggerSession Inspection Integration
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_session_get_variables_frame0() {
    let source = "let x = 42;";
    let bc = compile(source);
    let mut session = DebuggerSession::new(bc, source, "test.atlas");
    match session.process_request(DebugRequest::GetVariables { frame_index: 0 }) {
        DebugResponse::Variables { frame_index, .. } => assert_eq!(frame_index, 0),
        r => panic!("unexpected: {:?}", r),
    }
}

#[test]
fn test_session_get_variables_nonexistent_frame() {
    let source = "let x = 1;";
    let bc = compile(source);
    let mut session = DebuggerSession::new(bc, source, "test.atlas");
    match session.process_request(DebugRequest::GetVariables { frame_index: 99 }) {
        DebugResponse::Variables { .. } => {} // returns empty or globals
        r => panic!("unexpected: {:?}", r),
    }
}

#[test]
fn test_session_eval_in_context() {
    let source = "let x = 10;";
    let bc = compile(source);
    let mut session = DebuggerSession::new(bc, source, "test.atlas");
    match session.process_request(DebugRequest::Evaluate {
        expression: "1 + 2".into(),
        frame_index: 0,
    }) {
        DebugResponse::EvalResult { value, type_name } => {
            assert_eq!(type_name, "number");
            assert!(value.contains('3'));
        }
        r => panic!("unexpected: {:?}", r),
    }
}

#[test]
fn test_session_stack_trace() {
    let source = "let x = 1;\nlet y = 2;";
    let bc = compile(source);
    let mut session = DebuggerSession::new(bc, source, "test.atlas");
    match session.process_request(DebugRequest::GetStack) {
        DebugResponse::StackTrace { frames } => {
            assert!(!frames.is_empty());
            assert_eq!(frames[0].index, 0);
        }
        r => panic!("unexpected: {:?}", r),
    }
}

#[test]
fn test_session_stack_trace_has_main() {
    let source = "let a = 1;";
    let bc = compile(source);
    let mut session = DebuggerSession::new(bc, source, "test.atlas");
    match session.process_request(DebugRequest::GetStack) {
        DebugResponse::StackTrace { frames } => {
            assert_eq!(frames[0].function_name, "<main>");
        }
        r => panic!("unexpected: {:?}", r),
    }
}
