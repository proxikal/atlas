//! Integration tests for the Atlas debugger infrastructure (phase-04).
//!
//! Covers:
//! 1. Protocol request/response serialization
//! 2. Source mapping (bidirectional accuracy)
//! 3. Breakpoint management (set, remove, hit)
//! 4. Step operations (into, over, out)
//! 5. Variable inspection at breakpoints
//! 6. Stack trace generation
//! 7. Expression evaluation in context
//! 8. Performance impact when debugging is disabled

use atlas_runtime::bytecode::Bytecode;
use atlas_runtime::bytecode::DebugSpan;
use atlas_runtime::compiler::Compiler;
use atlas_runtime::debugger::protocol::{
    DebugEvent, DebugRequest, DebugResponse, DebugStackFrame, PauseReason, SourceLocation, Variable,
};
use atlas_runtime::debugger::source_map::{
    byte_offset_to_line_column, compute_line_offsets, SourceMap,
};
use atlas_runtime::debugger::state::{DebuggerState, StepMode};
use atlas_runtime::debugger::DebuggerSession;
use atlas_runtime::lexer::Lexer;
use atlas_runtime::parser::Parser;
use atlas_runtime::security::SecurityContext;
use atlas_runtime::span::Span;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn compile(source: &str) -> Bytecode {
    let tokens = Lexer::new(source).tokenize().0;
    let (ast, _) = Parser::new(tokens).parse();
    let mut compiler = Compiler::new();
    compiler.compile(&ast).expect("compile failed")
}

fn security() -> SecurityContext {
    SecurityContext::allow_all()
}

fn new_session(source: &str) -> DebuggerSession {
    let bc = compile(source);
    DebuggerSession::new(bc, source, "test.atlas")
}

fn loc(line: u32) -> SourceLocation {
    SourceLocation::new("test.atlas", line, 1)
}

// ═════════════════════════════════════════════════════════════════════════════
// 1. Protocol serialization
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn proto_serialize_set_breakpoint() {
    use atlas_runtime::debugger::protocol::{deserialize_request, serialize_request};
    let req = DebugRequest::SetBreakpoint { location: loc(5) };
    let json = serialize_request(&req).unwrap();
    let back: DebugRequest = deserialize_request(&json).unwrap();
    assert_eq!(req, back);
}

#[test]
fn proto_serialize_remove_breakpoint() {
    use atlas_runtime::debugger::protocol::{deserialize_request, serialize_request};
    let req = DebugRequest::RemoveBreakpoint { id: 3 };
    let json = serialize_request(&req).unwrap();
    let back: DebugRequest = deserialize_request(&json).unwrap();
    assert_eq!(req, back);
}

#[test]
fn proto_serialize_continue() {
    use atlas_runtime::debugger::protocol::{deserialize_request, serialize_request};
    let req = DebugRequest::Continue;
    let json = serialize_request(&req).unwrap();
    let back: DebugRequest = deserialize_request(&json).unwrap();
    assert_eq!(req, back);
}

#[test]
fn proto_serialize_step_over() {
    use atlas_runtime::debugger::protocol::serialize_request;
    let req = DebugRequest::StepOver;
    let json = serialize_request(&req).unwrap();
    assert!(json.contains("StepOver"));
}

#[test]
fn proto_serialize_step_into() {
    use atlas_runtime::debugger::protocol::serialize_request;
    let req = DebugRequest::StepInto;
    let json = serialize_request(&req).unwrap();
    assert!(json.contains("StepInto"));
}

#[test]
fn proto_serialize_step_out() {
    use atlas_runtime::debugger::protocol::serialize_request;
    let req = DebugRequest::StepOut;
    let json = serialize_request(&req).unwrap();
    assert!(json.contains("StepOut"));
}

#[test]
fn proto_serialize_get_variables() {
    use atlas_runtime::debugger::protocol::{deserialize_request, serialize_request};
    let req = DebugRequest::GetVariables { frame_index: 2 };
    let json = serialize_request(&req).unwrap();
    let back: DebugRequest = deserialize_request(&json).unwrap();
    assert_eq!(req, back);
}

#[test]
fn proto_serialize_evaluate() {
    use atlas_runtime::debugger::protocol::{deserialize_request, serialize_request};
    let req = DebugRequest::Evaluate {
        expression: "x + 1".to_string(),
        frame_index: 0,
    };
    let json = serialize_request(&req).unwrap();
    let back: DebugRequest = deserialize_request(&json).unwrap();
    assert_eq!(req, back);
}

#[test]
fn proto_serialize_paused_response() {
    use atlas_runtime::debugger::protocol::{deserialize_response, serialize_response};
    let resp = DebugResponse::Paused {
        reason: PauseReason::Breakpoint { id: 1 },
        location: Some(loc(3)),
        ip: 42,
    };
    let json = serialize_response(&resp).unwrap();
    let back: DebugResponse = deserialize_response(&json).unwrap();
    assert_eq!(resp, back);
}

#[test]
fn proto_serialize_variables_response() {
    use atlas_runtime::debugger::protocol::{deserialize_response, serialize_response};
    let resp = DebugResponse::Variables {
        frame_index: 0,
        variables: vec![Variable::new("x", "42", "Number")],
    };
    let json = serialize_response(&resp).unwrap();
    let back: DebugResponse = deserialize_response(&json).unwrap();
    assert_eq!(resp, back);
}

#[test]
fn proto_serialize_stack_trace_response() {
    use atlas_runtime::debugger::protocol::{deserialize_response, serialize_response};
    let resp = DebugResponse::StackTrace {
        frames: vec![DebugStackFrame {
            index: 0,
            function_name: "<main>".to_string(),
            location: Some(loc(1)),
            stack_base: 0,
            local_count: 2,
        }],
    };
    let json = serialize_response(&resp).unwrap();
    let back: DebugResponse = deserialize_response(&json).unwrap();
    assert_eq!(resp, back);
}

#[test]
fn proto_serialize_error_response() {
    use atlas_runtime::debugger::protocol::{deserialize_response, serialize_response};
    let resp = DebugResponse::Error {
        message: "unknown error".to_string(),
    };
    let json = serialize_response(&resp).unwrap();
    let back: DebugResponse = deserialize_response(&json).unwrap();
    assert_eq!(resp, back);
}

#[test]
fn proto_serialize_debug_event_paused() {
    use atlas_runtime::debugger::protocol::{deserialize_event, serialize_event};
    let event = DebugEvent::Paused {
        reason: PauseReason::Step,
        location: Some(loc(2)),
        ip: 10,
    };
    let json = serialize_event(&event).unwrap();
    let back: DebugEvent = deserialize_event(&json).unwrap();
    assert_eq!(event, back);
}

#[test]
fn proto_serialize_debug_event_stopped() {
    use atlas_runtime::debugger::protocol::{deserialize_event, serialize_event};
    let event = DebugEvent::Stopped {
        result: Some("42".to_string()),
        error: None,
    };
    let json = serialize_event(&event).unwrap();
    let back: DebugEvent = deserialize_event(&json).unwrap();
    assert_eq!(event, back);
}

#[test]
fn proto_source_location_display() {
    let loc = SourceLocation::new("main.atlas", 10, 5);
    assert_eq!(loc.to_string(), "main.atlas:10:5");
    let anon = SourceLocation::anonymous(3, 1);
    assert_eq!(anon.to_string(), "<anonymous>:3:1");
}

// ═════════════════════════════════════════════════════════════════════════════
// 2. Source mapping
// ═════════════════════════════════════════════════════════════════════════════

fn make_debug_spans(pairs: &[(usize, usize, usize)]) -> Vec<DebugSpan> {
    pairs
        .iter()
        .map(|&(off, s, e)| DebugSpan {
            instruction_offset: off,
            span: Span::new(s, e),
        })
        .collect()
}

#[test]
fn srcmap_compute_line_offsets_basic() {
    let src = "abc\ndef\nghi";
    let offsets = compute_line_offsets(src);
    assert_eq!(offsets[0], 0);
    assert_eq!(offsets[1], 4);
    assert_eq!(offsets[2], 8);
}

#[test]
fn srcmap_byte_offset_to_line_column_line1() {
    let src = "let x = 1;\nlet y = 2;";
    let offsets = compute_line_offsets(src);
    assert_eq!(byte_offset_to_line_column(0, &offsets), (1, 1));
    assert_eq!(byte_offset_to_line_column(4, &offsets), (1, 5));
}

#[test]
fn srcmap_byte_offset_to_line_column_line2() {
    let src = "let x = 1;\nlet y = 2;";
    let offsets = compute_line_offsets(src);
    let line2_start = 11; // after "let x = 1;\n"
    assert_eq!(byte_offset_to_line_column(line2_start, &offsets), (2, 1));
}

#[test]
fn srcmap_from_debug_spans_no_source_defaults() {
    let spans = make_debug_spans(&[(0, 0, 5), (5, 5, 10)]);
    let map = SourceMap::from_debug_spans(&spans, "test.atlas", None);
    let loc = map.location_for_offset(0).unwrap();
    assert_eq!(loc.line, 1);
    assert_eq!(loc.column, 1);
}

#[test]
fn srcmap_from_debug_spans_with_source() {
    let src = "let x = 1;\nlet y = 2;\n";
    let spans = make_debug_spans(&[(0, 0, 10), (3, 11, 21)]);
    let map = SourceMap::from_debug_spans(&spans, "m.atlas", Some(src));
    let loc0 = map.location_for_offset(0).unwrap();
    let loc1 = map.location_for_offset(3).unwrap();
    assert_eq!(loc0.line, 1);
    assert_eq!(loc1.line, 2);
}

#[test]
fn srcmap_forward_lookup_exact() {
    let mut map = SourceMap::new();
    map.insert(10, SourceLocation::new("a.atlas", 3, 1));
    assert_eq!(map.location_for_offset(10).unwrap().line, 3);
}

#[test]
fn srcmap_forward_lookup_closest_preceding() {
    let mut map = SourceMap::new();
    map.insert(0, SourceLocation::new("a.atlas", 1, 1));
    map.insert(10, SourceLocation::new("a.atlas", 3, 1));
    assert_eq!(map.location_for_offset(5).unwrap().line, 1);
}

#[test]
fn srcmap_reverse_lookup_exact() {
    let mut map = SourceMap::new();
    map.insert(42, SourceLocation::new("a.atlas", 7, 3));
    assert_eq!(map.offset_for_location("a.atlas", 7, 3), Some(42));
}

#[test]
fn srcmap_offsets_for_line() {
    let mut map = SourceMap::new();
    map.insert(0, SourceLocation::new("a.atlas", 1, 1));
    map.insert(2, SourceLocation::new("a.atlas", 1, 5));
    map.insert(5, SourceLocation::new("a.atlas", 2, 1));
    let offsets = map.offsets_for_line("a.atlas", 1);
    assert_eq!(offsets, vec![0, 2]);
}

#[test]
fn srcmap_first_offset_for_line() {
    let mut map = SourceMap::new();
    map.insert(5, SourceLocation::new("a.atlas", 2, 1));
    map.insert(2, SourceLocation::new("a.atlas", 2, 5));
    assert_eq!(map.first_offset_for_line("a.atlas", 2), Some(2));
}

#[test]
fn srcmap_all_offsets_sorted() {
    let mut map = SourceMap::new();
    map.insert(10, SourceLocation::new("a.atlas", 1, 1));
    map.insert(3, SourceLocation::new("a.atlas", 1, 5));
    map.insert(7, SourceLocation::new("a.atlas", 2, 1));
    assert_eq!(map.all_offsets(), vec![3, 7, 10]);
}

#[test]
fn srcmap_empty_map_queries() {
    let map = SourceMap::new();
    assert!(map.is_empty());
    assert!(map.location_for_offset(0).is_none());
    assert_eq!(map.offset_for_location("a.atlas", 1, 1), None);
    assert!(map.offsets_for_line("a.atlas", 1).is_empty());
}

// ═════════════════════════════════════════════════════════════════════════════
// 3. Breakpoint management
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn bp_add_breakpoint_assigns_sequential_ids() {
    let mut state = DebuggerState::new();
    let id1 = state.add_breakpoint(loc(1));
    let id2 = state.add_breakpoint(loc(2));
    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
}

#[test]
fn bp_unverified_by_default() {
    let mut state = DebuggerState::new();
    let id = state.add_breakpoint(loc(5));
    assert!(!state.get_breakpoint(id).unwrap().verified);
}

#[test]
fn bp_verify_binds_offset() {
    let mut state = DebuggerState::new();
    let id = state.add_breakpoint(loc(5));
    state.verify_breakpoint(id, 100);
    let bp = state.get_breakpoint(id).unwrap();
    assert!(bp.verified);
    assert_eq!(bp.instruction_offset, Some(100));
}

#[test]
fn bp_has_breakpoint_at_verified_offset() {
    let mut state = DebuggerState::new();
    let id = state.add_breakpoint(loc(3));
    state.verify_breakpoint(id, 50);
    assert!(state.has_breakpoint_at_offset(50));
    assert!(!state.has_breakpoint_at_offset(51));
}

#[test]
fn bp_unverified_does_not_match_offset() {
    let mut state = DebuggerState::new();
    state.add_breakpoint(loc(3));
    assert!(!state.has_breakpoint_at_offset(0));
}

#[test]
fn bp_remove_breakpoint() {
    let mut state = DebuggerState::new();
    let id = state.add_breakpoint(loc(5));
    state.remove_breakpoint(id);
    assert_eq!(state.breakpoint_count(), 0);
}

#[test]
fn bp_clear_all_breakpoints() {
    let mut state = DebuggerState::new();
    for line in 1..=5 {
        state.add_breakpoint(loc(line));
    }
    state.clear_breakpoints();
    assert_eq!(state.breakpoint_count(), 0);
}

#[test]
fn bp_session_set_returns_breakpoint_set() {
    let mut session = new_session("let x = 1;\n");
    match session.process_request(DebugRequest::SetBreakpoint { location: loc(1) }) {
        DebugResponse::BreakpointSet { breakpoint } => {
            assert_eq!(breakpoint.id, 1);
        }
        r => panic!("expected BreakpointSet, got {:?}", r),
    }
}

#[test]
fn bp_session_list_empty() {
    let mut session = new_session("let x = 1;");
    match session.process_request(DebugRequest::ListBreakpoints) {
        DebugResponse::Breakpoints { breakpoints } => assert!(breakpoints.is_empty()),
        r => panic!("{:?}", r),
    }
}

#[test]
fn bp_session_remove_existing() {
    let mut session = new_session("let x = 1;");
    session.process_request(DebugRequest::SetBreakpoint { location: loc(1) });
    match session.process_request(DebugRequest::RemoveBreakpoint { id: 1 }) {
        DebugResponse::BreakpointRemoved { id } => assert_eq!(id, 1),
        r => panic!("{:?}", r),
    }
}

#[test]
fn bp_session_remove_nonexistent_is_error() {
    let mut session = new_session("let x = 1;");
    match session.process_request(DebugRequest::RemoveBreakpoint { id: 99 }) {
        DebugResponse::Error { .. } => {}
        r => panic!("expected Error, got {:?}", r),
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// 4. Step operations
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn step_into_mode_is_set() {
    let mut session = new_session("let x = 1;");
    session.process_request(DebugRequest::StepInto);
    assert_eq!(session.debug_state().step_mode, StepMode::Into);
}

#[test]
fn step_over_mode_is_set() {
    let mut session = new_session("let x = 1;");
    session.process_request(DebugRequest::StepOver);
    assert_eq!(session.debug_state().step_mode, StepMode::Over);
}

#[test]
fn step_out_mode_is_set() {
    let mut session = new_session("let x = 1;");
    session.process_request(DebugRequest::StepOut);
    assert_eq!(session.debug_state().step_mode, StepMode::Out);
}

#[test]
fn step_into_pauses_execution() {
    let source = "let x = 1;\nlet y = 2;\nlet z = 3;";
    let mut session = new_session(source);
    session.process_request(DebugRequest::StepInto);
    let resp = session.run_until_pause(&security());
    match resp {
        DebugResponse::Paused { .. } => {}
        r => panic!("expected Paused, got {:?}", r),
    }
}

#[test]
fn step_state_over_logic() {
    let mut state = DebuggerState::new();
    state.set_step_mode(StepMode::Over, 2);
    assert!(state.should_pause_for_step(2)); // same depth → pause
    assert!(state.should_pause_for_step(1)); // shallower → pause
    assert!(!state.should_pause_for_step(3)); // deeper → keep going
}

#[test]
fn step_state_out_logic() {
    let mut state = DebuggerState::new();
    state.set_step_mode(StepMode::Out, 3);
    assert!(state.should_pause_for_step(2)); // returned
    assert!(!state.should_pause_for_step(3)); // still in same frame
}

#[test]
fn step_state_into_always_pauses() {
    let mut state = DebuggerState::new();
    state.set_step_mode(StepMode::Into, 1);
    assert!(state.should_pause_for_step(1));
    assert!(state.should_pause_for_step(5));
}

#[test]
fn step_mode_cleared_after_pause() {
    let mut state = DebuggerState::new();
    state.set_step_mode(StepMode::Into, 1);
    state.pause(PauseReason::Step, None, 5);
    assert_eq!(state.step_mode, StepMode::None);
}

// ═════════════════════════════════════════════════════════════════════════════
// 5. Variable inspection
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn vars_get_variables_response_has_correct_frame_index() {
    let mut session = new_session("let x = 1;");
    match session.process_request(DebugRequest::GetVariables { frame_index: 0 }) {
        DebugResponse::Variables { frame_index, .. } => assert_eq!(frame_index, 0),
        r => panic!("{:?}", r),
    }
}

#[test]
fn vars_get_variables_nonexistent_frame() {
    let mut session = new_session("let x = 1;");
    // Should not panic, just return empty or global-only variables
    let resp = session.process_request(DebugRequest::GetVariables { frame_index: 99 });
    match resp {
        DebugResponse::Variables { .. } => {}
        r => panic!("expected Variables, got {:?}", r),
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// 6. Stack trace generation
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn stack_trace_has_main_frame() {
    let mut session = new_session("let x = 1;");
    match session.process_request(DebugRequest::GetStack) {
        DebugResponse::StackTrace { frames } => {
            assert!(!frames.is_empty());
            assert_eq!(frames[0].function_name, "<main>");
        }
        r => panic!("{:?}", r),
    }
}

#[test]
fn stack_trace_innermost_frame_index_0() {
    let mut session = new_session("let x = 1;");
    match session.process_request(DebugRequest::GetStack) {
        DebugResponse::StackTrace { frames } => assert_eq!(frames[0].index, 0),
        r => panic!("{:?}", r),
    }
}

#[test]
fn stack_trace_frame_has_stack_base() {
    let mut session = new_session("let x = 1;");
    match session.process_request(DebugRequest::GetStack) {
        DebugResponse::StackTrace { frames } => {
            // Main frame starts at stack base 0
            assert_eq!(frames[0].stack_base, 0);
        }
        r => panic!("{:?}", r),
    }
}

#[test]
fn stack_trace_serializable() {
    use atlas_runtime::debugger::protocol::{deserialize_response, serialize_response};
    let mut session = new_session("let x = 1;");
    let resp = session.process_request(DebugRequest::GetStack);
    let json = serialize_response(&resp).unwrap();
    let back = deserialize_response(&json).unwrap();
    assert_eq!(resp, back);
}

// ═════════════════════════════════════════════════════════════════════════════
// 7. Expression evaluation
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn eval_simple_arithmetic() {
    let mut session = new_session("let x = 1;");
    match session.process_request(DebugRequest::Evaluate {
        expression: "2 + 2".to_string(),
        frame_index: 0,
    }) {
        DebugResponse::EvalResult { value, type_name } => {
            assert_eq!(type_name, "number");
            assert!(value.contains('4'));
        }
        r => panic!("{:?}", r),
    }
}

#[test]
fn eval_boolean_expression() {
    let mut session = new_session("let x = 1;");
    match session.process_request(DebugRequest::Evaluate {
        expression: "1 == 1".to_string(),
        frame_index: 0,
    }) {
        DebugResponse::EvalResult { value, type_name } => {
            assert_eq!(type_name, "bool");
            assert!(value.contains("true"));
        }
        r => panic!("{:?}", r),
    }
}

#[test]
fn eval_string_concatenation() {
    let mut session = new_session("let x = 1;");
    match session.process_request(DebugRequest::Evaluate {
        expression: r#""foo" + "bar""#.to_string(),
        frame_index: 0,
    }) {
        DebugResponse::EvalResult { value, .. } => {
            assert!(value.contains("foo"));
            assert!(value.contains("bar"));
        }
        r => panic!("{:?}", r),
    }
}

#[test]
fn eval_null_literal() {
    let mut session = new_session("let x = 1;");
    match session.process_request(DebugRequest::Evaluate {
        expression: "null".to_string(),
        frame_index: 0,
    }) {
        DebugResponse::EvalResult { value, type_name } => {
            assert_eq!(type_name, "null");
            assert!(value.contains("null"));
        }
        r => panic!("{:?}", r),
    }
}

#[test]
fn eval_invalid_syntax_does_not_panic() {
    let mut session = new_session("let x = 1;");
    // Should return EvalResult or Error, not panic
    let resp = session.process_request(DebugRequest::Evaluate {
        expression: "@#$%".to_string(),
        frame_index: 0,
    });
    match resp {
        DebugResponse::EvalResult { .. } | DebugResponse::Error { .. } => {}
        r => panic!("unexpected {:?}", r),
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// 8. Performance: no overhead when debugging disabled
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn perf_vm_without_debugger_runs_normally() {
    // Create a VM without debugging enabled – should produce correct results.
    use atlas_runtime::vm::VM;
    let source = "let x = 10; let y = 20; let z = x + y;";
    let bc = compile(source);
    let mut vm = VM::new(bc);
    let sec = security();
    let result = vm.run(&sec);
    assert!(result.is_ok(), "VM should run without errors");
}

#[test]
fn perf_debugger_disabled_by_default() {
    use atlas_runtime::vm::VM;
    let bc = compile("let x = 1;");
    let vm = VM::new(bc);
    // Debugger should be None (disabled) on a plain VM
    assert!(vm.debugger().is_none());
}

#[test]
fn perf_debugger_disabled_after_run() {
    // Running without debugging doesn't accidentally enable debugging.
    use atlas_runtime::vm::VM;
    let bc = compile("let x = 1;");
    let mut vm = VM::new(bc);
    let sec = security();
    vm.run(&sec).unwrap();
    assert!(vm.debugger().is_none() || !vm.debugger().unwrap().is_enabled());
}

#[test]
fn perf_run_completes_correctly_multiple_operations() {
    use atlas_runtime::vm::VM;
    let source = "let a = 5;\nlet b = 10;\nlet c = a + b;\nlet d = c * 2;";
    let bc = compile(source);
    let mut vm = VM::new(bc);
    let sec = security();
    let result = vm.run(&sec);
    assert!(result.is_ok());
}

// ═════════════════════════════════════════════════════════════════════════════
// Additional integration: debuggable session end-to-end
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn e2e_run_to_completion_no_breakpoints() {
    let source = "let x = 1 + 2;\nlet y = x * 3;";
    let mut session = new_session(source);
    let resp = session.run_until_pause(&security());
    // Should complete (either Paused with Step sentinel or similar)
    match resp {
        DebugResponse::Paused { .. } | DebugResponse::Error { .. } => {}
        r => panic!("unexpected {:?}", r),
    }
    assert!(session.is_stopped());
}

#[test]
fn e2e_get_location_returns_valid_ip() {
    let mut session = new_session("let x = 42;");
    match session.process_request(DebugRequest::GetLocation) {
        DebugResponse::Location { ip, .. } => {
            assert_eq!(ip, 0); // Before execution starts, IP is 0
        }
        r => panic!("{:?}", r),
    }
}

#[test]
fn e2e_breakpoint_id_is_stable() {
    let source = "let x = 1;\nlet y = 2;";
    let mut session = new_session(source);
    let id1 = match session.process_request(DebugRequest::SetBreakpoint { location: loc(1) }) {
        DebugResponse::BreakpointSet { breakpoint } => breakpoint.id,
        r => panic!("{:?}", r),
    };
    let id2 = match session.process_request(DebugRequest::SetBreakpoint { location: loc(2) }) {
        DebugResponse::BreakpointSet { breakpoint } => breakpoint.id,
        r => panic!("{:?}", r),
    };
    assert_ne!(id1, id2);
    // IDs are sequential
    assert_eq!(id2, id1 + 1);
}

#[test]
fn e2e_clear_breakpoints_empties_list() {
    let source = "let x = 1;\nlet y = 2;\nlet z = 3;";
    let mut session = new_session(source);
    for line in 1..=3 {
        session.process_request(DebugRequest::SetBreakpoint {
            location: loc(line),
        });
    }
    session.process_request(DebugRequest::ClearBreakpoints);
    match session.process_request(DebugRequest::ListBreakpoints) {
        DebugResponse::Breakpoints { breakpoints } => assert!(breakpoints.is_empty()),
        r => panic!("{:?}", r),
    }
}

#[test]
fn e2e_debug_state_initial_is_running() {
    let session = new_session("let x = 1;");
    assert!(session.debug_state().is_running());
}

#[test]
fn e2e_step_into_causes_paused_state() {
    let source = "let x = 1;\nlet y = 2;\nlet z = 3;";
    let mut session = new_session(source);
    session.process_request(DebugRequest::StepInto);
    session.run_until_pause(&security());
    // After stepping, the state should be Paused or Stopped
    assert!(session.is_paused() || session.is_stopped());
}
