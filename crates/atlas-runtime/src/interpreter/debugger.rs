//! Interpreter debugger infrastructure.
//!
//! Provides debugging capabilities for the tree-walking interpreter,
//! achieving feature parity with the VM debugger.

use crate::ast::{Block, Program, Stmt};
use crate::debugger::protocol::{
    Breakpoint, DebugRequest, DebugResponse, DebugStackFrame, PauseReason, SourceLocation, Variable,
};
use crate::debugger::state::{DebuggerState, StepMode};
use crate::interpreter::Interpreter;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::security::SecurityContext;
use crate::span::Span;
use crate::value::Value;

/// Stack frame for interpreter debugging.
#[derive(Debug, Clone)]
pub struct InterpreterStackFrame {
    /// Frame index (0 = innermost).
    pub index: usize,
    /// Function name ("<main>" for top-level).
    pub function_name: String,
    /// Source location where execution is paused.
    pub location: Option<SourceLocation>,
    /// Local variable count.
    pub local_count: usize,
}

/// Interpreter debugger session.
///
/// Wraps an `Interpreter` with debugging capabilities, providing the same
/// protocol as `DebuggerSession` for feature parity.
pub struct InterpreterDebuggerSession {
    /// The interpreter instance.
    interpreter: Interpreter,
    /// Debugger state (shared with VM debugger).
    state: DebuggerState,
    /// Source code for location mapping.
    source: String,
    /// Source file name.
    file: String,
    /// Line offsets for source mapping.
    line_offsets: Vec<usize>,
    /// Call stack for debugging.
    call_stack: Vec<InterpreterStackFrame>,
    /// Current statement span (for location reporting).
    current_span: Option<Span>,
}

impl InterpreterDebuggerSession {
    /// Create a new interpreter debugger session.
    pub fn new(source: &str, file: &str) -> Self {
        let line_offsets = compute_line_offsets(source);
        Self {
            interpreter: Interpreter::new(),
            state: DebuggerState::new(),
            source: source.to_string(),
            file: file.to_string(),
            line_offsets,
            call_stack: vec![InterpreterStackFrame {
                index: 0,
                function_name: "<main>".to_string(),
                location: None,
                local_count: 0,
            }],
            current_span: None,
        }
    }

    /// Process a debugger request.
    pub fn process_request(&mut self, request: DebugRequest) -> DebugResponse {
        match request {
            DebugRequest::SetBreakpoint { location } => {
                let id = self.state.add_breakpoint(location.clone());
                // For interpreter, we verify breakpoints immediately if the line exists
                if location.file == self.file && self.line_exists(location.line) {
                    self.state.verify_breakpoint(id, location.line as usize);
                }
                let bp = self
                    .state
                    .get_breakpoint(id)
                    .cloned()
                    .unwrap_or_else(|| Breakpoint::new(id, location));
                DebugResponse::BreakpointSet { breakpoint: bp }
            }

            DebugRequest::RemoveBreakpoint { id } => {
                if self.state.remove_breakpoint(id).is_some() {
                    DebugResponse::BreakpointRemoved { id }
                } else {
                    DebugResponse::error(format!("no breakpoint with id {id}"))
                }
            }

            DebugRequest::ListBreakpoints => DebugResponse::Breakpoints {
                breakpoints: self.state.breakpoints_owned(),
            },

            DebugRequest::ClearBreakpoints => {
                self.state.clear_breakpoints();
                DebugResponse::BreakpointsCleared
            }

            DebugRequest::Continue => {
                self.state.resume();
                DebugResponse::Resumed
            }

            DebugRequest::StepInto => {
                let depth = self.call_stack.len();
                self.state.set_step_mode(StepMode::Into, depth);
                self.state.resume();
                DebugResponse::Resumed
            }

            DebugRequest::StepOver => {
                let depth = self.call_stack.len();
                self.state.set_step_mode(StepMode::Over, depth);
                self.state.resume();
                DebugResponse::Resumed
            }

            DebugRequest::StepOut => {
                let depth = self.call_stack.len();
                self.state.set_step_mode(StepMode::Out, depth);
                self.state.resume();
                DebugResponse::Resumed
            }

            DebugRequest::Pause => {
                let depth = self.call_stack.len();
                self.state.set_step_mode(StepMode::Into, depth);
                DebugResponse::Resumed
            }

            DebugRequest::GetVariables { frame_index } => {
                let variables = self.collect_variables(frame_index);
                DebugResponse::Variables {
                    frame_index,
                    variables,
                }
            }

            DebugRequest::GetStack => {
                let frames = self.build_stack_trace();
                DebugResponse::StackTrace { frames }
            }

            DebugRequest::Evaluate {
                expression,
                frame_index,
            } => self.evaluate_expression(&expression, frame_index),

            DebugRequest::GetLocation => {
                let location = self.current_location();
                DebugResponse::Location { location, ip: 0 }
            }
        }
    }

    /// Run interpreter until pause (breakpoint/step) or completion.
    pub fn run_until_pause(&mut self, security: &SecurityContext) -> DebugResponse {
        // Parse the source
        let tokens = Lexer::new(&self.source).tokenize().0;
        let (ast, errors) = Parser::new(tokens).parse();

        if !errors.is_empty() {
            return DebugResponse::error(format!("parse error: {:?}", errors[0]));
        }

        // Run with debug hooks
        match self.eval_program_debugged(&ast, security) {
            Ok(DebugRunResult::Completed(_)) => {
                self.state.stop();
                DebugResponse::Paused {
                    reason: PauseReason::Step,
                    location: None,
                    ip: 0,
                }
            }
            Ok(DebugRunResult::Paused { reason, location }) => DebugResponse::Paused {
                reason,
                location,
                ip: 0,
            },
            Err(e) => {
                self.state.stop();
                DebugResponse::error(format!("{:?}", e))
            }
        }
    }

    /// Returns true if currently paused.
    pub fn is_paused(&self) -> bool {
        self.state.is_paused()
    }

    /// Returns true if execution has stopped.
    pub fn is_stopped(&self) -> bool {
        self.state.is_stopped()
    }

    /// Get current call depth.
    pub fn frame_depth(&self) -> usize {
        self.call_stack.len()
    }

    /// Get debugger state reference.
    pub fn debug_state(&self) -> &DebuggerState {
        &self.state
    }

    // === Private implementation ===

    /// Check if a line exists in the source.
    fn line_exists(&self, line: u32) -> bool {
        line > 0 && (line as usize) <= self.line_offsets.len()
    }

    /// Get current source location.
    fn current_location(&self) -> Option<SourceLocation> {
        self.current_span.map(|span| {
            let (line, column) = byte_offset_to_line_column(span.start, &self.line_offsets);
            SourceLocation::new(&self.file, line, column)
        })
    }

    /// Collect variables for a frame.
    fn collect_variables(&self, _frame_index: usize) -> Vec<Variable> {
        let mut vars = Vec::new();

        // Collect globals
        for (name, (value, _)) in &self.interpreter.globals {
            vars.push(Variable::new(
                name.clone(),
                format_value(value),
                value.type_name(),
            ));
        }

        // Collect locals from current scope
        if let Some(scope) = self.interpreter.locals.last() {
            for (name, (value, _)) in scope {
                vars.push(Variable::new(
                    name.clone(),
                    format_value(value),
                    value.type_name(),
                ));
            }
        }

        vars.sort_by(|a, b| a.name.cmp(&b.name));
        vars
    }

    /// Build stack trace.
    fn build_stack_trace(&self) -> Vec<DebugStackFrame> {
        self.call_stack
            .iter()
            .map(|frame| DebugStackFrame {
                index: frame.index,
                function_name: frame.function_name.clone(),
                location: frame.location.clone(),
                stack_base: 0,
                local_count: frame.local_count,
            })
            .collect()
    }

    /// Evaluate expression in context.
    fn evaluate_expression(&self, expression: &str, _frame_index: usize) -> DebugResponse {
        // Build snippet with visible variables
        let vars = self.collect_variables(0);
        let mut snippet = String::new();

        for var in &vars {
            if is_valid_identifier(&var.name) {
                if let Some(lit) = value_to_literal(&var.type_name, &var.value) {
                    snippet.push_str(&format!("let {} = {};\n", var.name, lit));
                }
            }
        }
        snippet.push_str(expression);
        let trimmed = expression.trim();
        if !trimmed.ends_with(';') && !trimmed.ends_with('}') {
            snippet.push(';');
        }

        // Parse and evaluate
        let tokens = Lexer::new(&snippet).tokenize().0;
        let (ast, errors) = Parser::new(tokens).parse();

        if !errors.is_empty() {
            return DebugResponse::error(format!("parse error: {:?}", errors[0]));
        }

        let mut interp = Interpreter::new();
        let security = SecurityContext::allow_all();

        match interp.eval(&ast, &security) {
            Ok(value) => DebugResponse::EvalResult {
                value: format_value(&value),
                type_name: value.type_name().to_string(),
            },
            Err(e) => DebugResponse::error(format!("{:?}", e)),
        }
    }

    /// Evaluate program with debug hooks.
    fn eval_program_debugged(
        &mut self,
        program: &Program,
        security: &SecurityContext,
    ) -> Result<DebugRunResult, crate::value::RuntimeError> {
        use crate::ast::Item;

        // Store security context
        self.interpreter.current_security = Some(std::sync::Arc::new(security.clone()));

        let mut last_value = Value::Null;

        for item in &program.items {
            match item {
                Item::Function(func) => {
                    // Store function definition (no debug pause needed)
                    self.interpreter.function_bodies.insert(
                        func.name.name.clone(),
                        crate::interpreter::UserFunction {
                            name: func.name.name.clone(),
                            params: func.params.clone(),
                            body: func.body.clone(),
                        },
                    );

                    let func_value = Value::Function(crate::value::FunctionRef {
                        name: func.name.name.clone(),
                        arity: func.params.len(),
                        bytecode_offset: 0,
                        local_count: 0,
                        param_ownership: vec![],
                        return_ownership: None,
                    });
                    self.interpreter
                        .globals
                        .insert(func.name.name.clone(), (func_value, false));
                }
                Item::Statement(stmt) => {
                    // Check for debug pause BEFORE executing
                    if let Some(pause_result) = self.check_debug_pause(stmt) {
                        return Ok(pause_result);
                    }

                    last_value = self.interpreter.eval_statement(stmt)?;

                    // Handle control flow
                    if let crate::interpreter::ControlFlow::Return(val) =
                        &self.interpreter.control_flow
                    {
                        last_value = val.clone();
                        self.interpreter.control_flow = crate::interpreter::ControlFlow::None;
                        break;
                    }
                }
                Item::Import(_) | Item::Export(_) | Item::Extern(_) | Item::TypeAlias(_) => {
                    // These don't need debug pauses
                }
            }
        }

        Ok(DebugRunResult::Completed(last_value))
    }

    /// Check if we should pause at this statement.
    fn check_debug_pause(&mut self, stmt: &Stmt) -> Option<DebugRunResult> {
        let span = stmt.span();
        self.current_span = Some(span);

        let (line, column) = byte_offset_to_line_column(span.start, &self.line_offsets);
        let location = SourceLocation::new(&self.file, line, column);

        // Update call stack location
        if let Some(frame) = self.call_stack.last_mut() {
            frame.location = Some(location.clone());
        }

        // Check breakpoints - collect ID first to avoid borrow conflict
        let hit_breakpoint: Option<u32> = {
            let bps = self.state.breakpoints();
            bps.iter()
                .find(|bp| bp.verified && bp.location.file == self.file && bp.location.line == line)
                .map(|bp| bp.id)
        };

        if let Some(bp_id) = hit_breakpoint {
            self.state.pause(
                PauseReason::Breakpoint { id: bp_id },
                Some(location.clone()),
                0,
            );
            return Some(DebugRunResult::Paused {
                reason: PauseReason::Breakpoint { id: bp_id },
                location: Some(location),
            });
        }

        // Check step mode
        let depth = self.call_stack.len();
        if self.state.should_pause_for_step(depth) {
            self.state
                .pause(PauseReason::Step, Some(location.clone()), 0);
            return Some(DebugRunResult::Paused {
                reason: PauseReason::Step,
                location: Some(location),
            });
        }

        None
    }

    /// Execute a block with debug hooks.
    pub fn eval_block_debugged(
        &mut self,
        block: &Block,
    ) -> Result<Option<DebugRunResult>, crate::value::RuntimeError> {
        self.interpreter.push_scope();

        for stmt in &block.statements {
            // Check for debug pause
            if let Some(pause_result) = self.check_debug_pause(stmt) {
                self.interpreter.pop_scope();
                return Ok(Some(pause_result));
            }

            self.interpreter.eval_statement(stmt)?;

            // Handle control flow
            if self.interpreter.control_flow != crate::interpreter::ControlFlow::None {
                break;
            }
        }

        self.interpreter.pop_scope();
        Ok(None)
    }

    /// Push a new call frame.
    pub fn push_frame(&mut self, function_name: &str) {
        let index = self.call_stack.len();
        self.call_stack.push(InterpreterStackFrame {
            index,
            function_name: function_name.to_string(),
            location: self.current_location(),
            local_count: 0,
        });
    }

    /// Pop current call frame.
    pub fn pop_frame(&mut self) {
        self.call_stack.pop();
    }
}

/// Result of debug-enabled execution.
pub enum DebugRunResult {
    /// Execution completed with a value.
    Completed(Value),
    /// Execution paused.
    Paused {
        reason: PauseReason,
        location: Option<SourceLocation>,
    },
}

// === Helper functions ===

/// Compute line offsets for source mapping.
fn compute_line_offsets(source: &str) -> Vec<usize> {
    let mut offsets = vec![0];
    for (i, ch) in source.char_indices() {
        if ch == '\n' {
            offsets.push(i + 1);
        }
    }
    offsets
}

/// Convert byte offset to line/column.
fn byte_offset_to_line_column(offset: usize, line_offsets: &[usize]) -> (u32, u32) {
    let mut line = 1;
    for (i, &line_start) in line_offsets.iter().enumerate() {
        if offset >= line_start {
            line = (i + 1) as u32;
        } else {
            break;
        }
    }
    let line_start = line_offsets.get(line as usize - 1).copied().unwrap_or(0);
    let column = (offset - line_start + 1) as u32;
    (line, column)
}

/// Format a value for display.
fn format_value(value: &Value) -> String {
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
        Value::Array(arr) => format!("[{} items]", arr.len()),
        Value::HashMap(m) => format!("{{HashMap, {} entries}}", m.inner().len()),
        Value::HashSet(s) => format!("{{HashSet, {} items}}", s.inner().len()),
        Value::Queue(q) => format!("[Queue, {} items]", q.inner().len()),
        Value::Stack(s) => format!("[Stack, {} items]", s.inner().len()),
        Value::Function(f) => format!("<fn {}>", f.name),
        _ => format!("{:?}", value),
    }
}

/// Check if a string is a valid identifier.
fn is_valid_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_alphanumeric() || c == '_')
}

/// Convert value representation to literal.
fn value_to_literal(type_name: &str, display: &str) -> Option<String> {
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

    fn loc(line: u32) -> SourceLocation {
        SourceLocation::new("test.atlas", line, 1)
    }

    fn security() -> SecurityContext {
        SecurityContext::allow_all()
    }

    #[test]
    fn test_session_creation() {
        let session = InterpreterDebuggerSession::new("let x = 1;", "test.atlas");
        assert!(!session.is_paused());
        assert!(!session.is_stopped());
    }

    #[test]
    fn test_set_breakpoint() {
        let mut session = InterpreterDebuggerSession::new("let x = 1;\nlet y = 2;", "test.atlas");
        let resp = session.process_request(DebugRequest::SetBreakpoint { location: loc(1) });
        match resp {
            DebugResponse::BreakpointSet { breakpoint } => {
                assert_eq!(breakpoint.id, 1);
            }
            r => panic!("expected BreakpointSet, got {:?}", r),
        }
    }

    #[test]
    fn test_list_breakpoints_empty() {
        let mut session = InterpreterDebuggerSession::new("let x = 1;", "test.atlas");
        match session.process_request(DebugRequest::ListBreakpoints) {
            DebugResponse::Breakpoints { breakpoints } => {
                assert!(breakpoints.is_empty());
            }
            r => panic!("unexpected: {:?}", r),
        }
    }

    #[test]
    fn test_step_into_pauses() {
        let source = "let x = 1;\nlet y = 2;";
        let mut session = InterpreterDebuggerSession::new(source, "test.atlas");
        session.process_request(DebugRequest::StepInto);
        let resp = session.run_until_pause(&security());
        match resp {
            DebugResponse::Paused { reason, .. } => {
                assert_eq!(reason, PauseReason::Step);
            }
            r => panic!("expected Paused, got {:?}", r),
        }
    }

    #[test]
    fn test_run_to_completion() {
        let source = "let x = 1 + 2;";
        let mut session = InterpreterDebuggerSession::new(source, "test.atlas");
        session.process_request(DebugRequest::Continue);
        let _resp = session.run_until_pause(&security());
        assert!(session.is_stopped());
    }

    #[test]
    fn test_get_stack() {
        let source = "let x = 1;";
        let mut session = InterpreterDebuggerSession::new(source, "test.atlas");
        match session.process_request(DebugRequest::GetStack) {
            DebugResponse::StackTrace { frames } => {
                assert!(!frames.is_empty());
                assert_eq!(frames[0].function_name, "<main>");
            }
            r => panic!("unexpected: {:?}", r),
        }
    }

    #[test]
    fn test_evaluate_expression() {
        let source = "let x = 1;";
        let mut session = InterpreterDebuggerSession::new(source, "test.atlas");
        match session.process_request(DebugRequest::Evaluate {
            expression: "2 + 3".to_string(),
            frame_index: 0,
        }) {
            DebugResponse::EvalResult { value, type_name } => {
                assert_eq!(type_name, "number");
                assert!(value.contains('5'));
            }
            r => panic!("unexpected: {:?}", r),
        }
    }

    #[test]
    fn test_get_variables() {
        let source = "let x = 1;";
        let mut session = InterpreterDebuggerSession::new(source, "test.atlas");
        match session.process_request(DebugRequest::GetVariables { frame_index: 0 }) {
            DebugResponse::Variables { frame_index, .. } => {
                assert_eq!(frame_index, 0);
            }
            r => panic!("unexpected: {:?}", r),
        }
    }

    #[test]
    fn test_step_modes_set() {
        let source = "let x = 1;";
        let mut session = InterpreterDebuggerSession::new(source, "test.atlas");

        session.process_request(DebugRequest::StepInto);
        assert_eq!(session.debug_state().step_mode, StepMode::Into);

        session.process_request(DebugRequest::StepOver);
        assert_eq!(session.debug_state().step_mode, StepMode::Over);

        session.process_request(DebugRequest::StepOut);
        assert_eq!(session.debug_state().step_mode, StepMode::Out);
    }

    #[test]
    fn test_compute_line_offsets() {
        let src = "abc\ndef\nghi";
        let offsets = compute_line_offsets(src);
        assert_eq!(offsets[0], 0);
        assert_eq!(offsets[1], 4);
        assert_eq!(offsets[2], 8);
    }

    #[test]
    fn test_byte_offset_to_line_column() {
        let src = "let x = 1;\nlet y = 2;";
        let offsets = compute_line_offsets(src);
        assert_eq!(byte_offset_to_line_column(0, &offsets), (1, 1));
        assert_eq!(byte_offset_to_line_column(4, &offsets), (1, 5));
        assert_eq!(byte_offset_to_line_column(11, &offsets), (2, 1));
    }
}
