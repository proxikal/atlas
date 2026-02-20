// debugger.rs — Debugger: execution control, inspection, and protocol tests

use atlas_runtime::bytecode::{Bytecode, DebugSpan};
use atlas_runtime::compiler::Compiler;
use atlas_runtime::debugger::breakpoints::{
    BreakpointCondition, BreakpointEntry, BreakpointManager, ShouldFire,
};
use atlas_runtime::debugger::inspection::{
    format_value_with_depth, EvalResult, Inspector, ScopedVariable, VariableScope,
};
use atlas_runtime::debugger::protocol::{
    deserialize_event, deserialize_request, deserialize_response, serialize_event,
    serialize_request, serialize_response, Breakpoint, DebugEvent, DebugRequest, DebugResponse,
    DebugStackFrame, PauseReason, SourceLocation, Variable,
};
use atlas_runtime::debugger::source_map::{
    byte_offset_to_line_column, compute_line_offsets, SourceMap,
};
use atlas_runtime::debugger::state::{DebuggerState, StepMode};
use atlas_runtime::debugger::stepping::{StepRequest, StepTracker};
use atlas_runtime::debugger::DebuggerSession;
use atlas_runtime::lexer::Lexer;
use atlas_runtime::parser::Parser;
use atlas_runtime::security::SecurityContext;
use atlas_runtime::span::Span;
use atlas_runtime::value::Value;
use atlas_runtime::vm::VM;

// --- Execution control (breakpoints, stepping) ---

// Debugger execution control tests — Phase 05.
//
// Tests breakpoint management (set, hit, remove, conditional, hit counts, log points),
// step operations (into, over, out, run-to-line), and execution flow.

fn compile(source: &str) -> Bytecode {
    let tokens = Lexer::new(source).tokenize().0;
    let (ast, _) = Parser::new(tokens).parse();
    let mut compiler = Compiler::new();
    compiler.compile(&ast).expect("compile failed")
}

fn security() -> SecurityContext {
    SecurityContext::allow_all()
}

fn loc(line: u32) -> SourceLocation {
    SourceLocation::new("test.atlas", line, 1)
}

// ══════════════════════════════════════════════════════════════════════════════
// Breakpoint Manager Tests
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_bp_manager_add_simple() {
    let mut mgr = BreakpointManager::new();
    let id = mgr.add(loc(1));
    assert_eq!(id, 1);
    assert_eq!(mgr.count(), 1);
}

#[test]
fn test_bp_manager_add_conditional_hit_count() {
    let mut mgr = BreakpointManager::new();
    let id = mgr.add_conditional(loc(1), BreakpointCondition::HitCount(5));
    mgr.verify(id, 10);
    // Should skip until hit 5
    for _ in 0..4 {
        assert_eq!(mgr.check_offset(10), ShouldFire::Skip);
    }
    assert_eq!(mgr.check_offset(10), ShouldFire::Pause);
}

#[test]
fn test_bp_manager_add_conditional_hit_count_multiple() {
    let mut mgr = BreakpointManager::new();
    let id = mgr.add_conditional(loc(1), BreakpointCondition::HitCountMultiple(3));
    mgr.verify(id, 10);
    assert_eq!(mgr.check_offset(10), ShouldFire::Skip); // 1
    assert_eq!(mgr.check_offset(10), ShouldFire::Skip); // 2
    assert_eq!(mgr.check_offset(10), ShouldFire::Pause); // 3
    assert_eq!(mgr.check_offset(10), ShouldFire::Skip); // 4
    assert_eq!(mgr.check_offset(10), ShouldFire::Skip); // 5
    assert_eq!(mgr.check_offset(10), ShouldFire::Pause); // 6
}

#[test]
fn test_bp_manager_log_point() {
    let mut mgr = BreakpointManager::new();
    let id = mgr.add_log_point(loc(1), "value of x".to_string());
    mgr.verify(id, 10);
    // Log points return Skip (they log but don't pause)
    assert_eq!(mgr.check_offset(10), ShouldFire::Skip);
    let logs = mgr.drain_log_output();
    assert_eq!(logs, vec!["value of x"]);
}

#[test]
fn test_bp_manager_log_point_accumulates() {
    let mut mgr = BreakpointManager::new();
    let id = mgr.add_log_point(loc(1), "msg".to_string());
    mgr.verify(id, 10);
    mgr.check_offset(10);
    mgr.check_offset(10);
    let logs = mgr.drain_log_output();
    assert_eq!(logs.len(), 2);
}

#[test]
fn test_bp_manager_enable_disable() {
    let mut mgr = BreakpointManager::new();
    let id = mgr.add(loc(1));
    mgr.verify(id, 10);
    assert_eq!(mgr.check_offset(10), ShouldFire::Pause);

    mgr.disable(id);
    // Reset hit state by removing and re-adding
    assert_eq!(mgr.check_offset(10), ShouldFire::Skip);

    mgr.enable(id);
    assert_eq!(mgr.check_offset(10), ShouldFire::Pause);
}

#[test]
fn test_bp_manager_remove_cleans_offset_index() {
    let mut mgr = BreakpointManager::new();
    let id = mgr.add(loc(1));
    mgr.verify(id, 10);
    assert!(mgr.has_breakpoint_at(10));
    mgr.remove(id);
    assert!(!mgr.has_breakpoint_at(10));
}

#[test]
fn test_bp_manager_clear_all() {
    let mut mgr = BreakpointManager::new();
    let id1 = mgr.add(loc(1));
    let id2 = mgr.add(loc(2));
    mgr.verify(id1, 10);
    mgr.verify(id2, 20);
    mgr.clear();
    assert_eq!(mgr.count(), 0);
    assert!(!mgr.has_breakpoint_at(10));
    assert!(!mgr.has_breakpoint_at(20));
}

#[test]
fn test_bp_manager_multiple_at_same_offset() {
    let mut mgr = BreakpointManager::new();
    let id1 = mgr.add(loc(1));
    let id2 = mgr.add_conditional(loc(1), BreakpointCondition::HitCount(3));
    mgr.verify(id1, 10);
    mgr.verify(id2, 10);
    // First bp (unconditional) should fire immediately
    assert_eq!(mgr.check_offset(10), ShouldFire::Pause);
}

#[test]
fn test_bp_manager_set_condition_after_creation() {
    let mut mgr = BreakpointManager::new();
    let id = mgr.add(loc(1));
    mgr.verify(id, 10);
    mgr.set_condition(id, BreakpointCondition::HitCount(2));
    assert_eq!(mgr.check_offset(10), ShouldFire::Skip); // hit 1
    assert_eq!(mgr.check_offset(10), ShouldFire::Pause); // hit 2
}

#[test]
fn test_bp_manager_reset_hit_counts() {
    let mut mgr = BreakpointManager::new();
    let id = mgr.add_conditional(loc(1), BreakpointCondition::HitCount(2));
    mgr.verify(id, 10);
    mgr.check_offset(10); // hit 1
    mgr.reset_all_hit_counts();
    assert_eq!(mgr.check_offset(10), ShouldFire::Skip); // hit 1 again (reset)
}

#[test]
fn test_bp_manager_expression_condition_always_passes() {
    let mut mgr = BreakpointManager::new();
    let id = mgr.add_conditional(loc(1), BreakpointCondition::Expression("x > 0".into()));
    mgr.verify(id, 10);
    // Expression conditions pass through (caller must evaluate)
    assert_eq!(mgr.check_offset(10), ShouldFire::Pause);
}

#[test]
fn test_bp_manager_all_entries_sorted() {
    let mut mgr = BreakpointManager::new();
    mgr.add(loc(3));
    mgr.add(loc(1));
    mgr.add(loc(2));
    let ids: Vec<u32> = mgr.all_entries().iter().map(|e| e.breakpoint.id).collect();
    assert_eq!(ids, vec![1, 2, 3]);
}

#[test]
fn test_bp_manager_enabled_count() {
    let mut mgr = BreakpointManager::new();
    let id1 = mgr.add(loc(1));
    let _id2 = mgr.add(loc(2));
    assert_eq!(mgr.enabled_count(), 2);
    mgr.disable(id1);
    assert_eq!(mgr.enabled_count(), 1);
}

#[test]
fn test_bp_entry_unverified_skips() {
    let bp = Breakpoint::new(1, loc(1)); // unverified
    let mut entry = BreakpointEntry::new(bp);
    assert_eq!(entry.check_and_increment(), ShouldFire::Skip);
}

#[test]
fn test_bp_entry_disabled_skips() {
    let bp = Breakpoint::verified_at(1, loc(1), 10);
    let mut entry = BreakpointEntry::new(bp);
    entry.enabled = false;
    assert_eq!(entry.check_and_increment(), ShouldFire::Skip);
}

#[test]
fn test_bp_entry_is_log_point() {
    let bp = Breakpoint::new(1, loc(1));
    let entry = BreakpointEntry::log_point(bp, "msg".into());
    assert!(entry.is_log_point());
}

#[test]
fn test_bp_entry_not_log_point() {
    let bp = Breakpoint::new(1, loc(1));
    let entry = BreakpointEntry::new(bp);
    assert!(!entry.is_log_point());
}

// ══════════════════════════════════════════════════════════════════════════════
// Step Tracker Tests
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_step_tracker_initial() {
    let tracker = StepTracker::new();
    assert!(!tracker.is_stepping());
    assert!(tracker.active_request().is_none());
}

#[test]
fn test_step_tracker_begin_into() {
    let mut tracker = StepTracker::new();
    tracker.begin_step(StepRequest::Into, 1, Some(&loc(1)));
    assert!(tracker.is_stepping());
    assert_eq!(tracker.active_request(), Some(&StepRequest::Into));
}

#[test]
fn test_step_tracker_begin_over() {
    let mut tracker = StepTracker::new();
    tracker.begin_step(StepRequest::Over, 2, Some(&loc(3)));
    assert_eq!(tracker.start_depth(), 2);
}

#[test]
fn test_step_tracker_cancel() {
    let mut tracker = StepTracker::new();
    tracker.begin_step(StepRequest::Into, 1, None);
    tracker.cancel();
    assert!(!tracker.is_stepping());
}

#[test]
fn test_step_tracker_run_to_offset() {
    let map = SourceMap::new();
    let mut tracker = StepTracker::new();
    tracker.begin_step(StepRequest::RunToOffset(5), 1, None);
    assert!(tracker.should_pause(3, 1, &map).is_none());
    assert!(tracker.should_pause(5, 1, &map).is_some());
}

#[test]
fn test_step_tracker_instructions_counter() {
    let mut map = SourceMap::new();
    map.insert(0, SourceLocation::new("test.atlas", 1, 1));
    let mut tracker = StepTracker::new();
    tracker.begin_step(StepRequest::Over, 1, Some(&loc(1)));
    tracker.should_pause(0, 2, &map); // deeper, same line
    tracker.should_pause(0, 2, &map);
    assert_eq!(tracker.instructions_executed(), 2);
}

#[test]
fn test_step_tracker_safety_limit() {
    let mut map = SourceMap::new();
    map.insert(0, SourceLocation::new("test.atlas", 1, 1));
    let mut tracker = StepTracker::new();
    tracker.set_max_instructions(3);
    tracker.begin_step(StepRequest::Over, 1, Some(&loc(1)));
    assert!(tracker.should_pause(0, 2, &map).is_none());
    assert!(tracker.should_pause(0, 2, &map).is_none());
    assert!(tracker.should_pause(0, 2, &map).is_none());
    assert!(tracker.should_pause(0, 2, &map).is_some()); // 4th call exceeds limit
}

// ══════════════════════════════════════════════════════════════════════════════
// DebuggerSession Integration Tests
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_session_breakpoint_set_and_hit() {
    let source = "let x = 1;\nlet y = 2;\nlet z = 3;";
    let bc = compile(source);
    let mut session = DebuggerSession::new(bc, source, "test.atlas");

    let resp = session.process_request(DebugRequest::SetBreakpoint {
        location: SourceLocation::new("test.atlas", 1, 1),
    });
    match resp {
        DebugResponse::BreakpointSet { breakpoint } => {
            if breakpoint.verified {
                let resp = session.run_until_pause(&security());
                if let DebugResponse::Paused { .. } = resp {
                    assert!(session.is_paused());
                }
            }
        }
        _ => panic!("expected BreakpointSet"),
    }
}

#[test]
fn test_session_step_into_pauses() {
    let source = "let x = 1;\nlet y = 2;";
    let bc = compile(source);
    let mut session = DebuggerSession::new(bc, source, "test.atlas");
    session.process_request(DebugRequest::StepInto);
    let resp = session.run_until_pause(&security());
    match resp {
        DebugResponse::Paused { .. } => {}
        DebugResponse::Error { .. } => {}
        r => panic!("expected Paused, got {:?}", r),
    }
}

#[test]
fn test_session_step_over_pauses() {
    let source = "let x = 1;\nlet y = 2;";
    let bc = compile(source);
    let mut session = DebuggerSession::new(bc, source, "test.atlas");
    session.process_request(DebugRequest::StepOver);
    let resp = session.run_until_pause(&security());
    match resp {
        DebugResponse::Paused { .. } => {}
        r => panic!("expected Paused, got {:?}", r),
    }
}

#[test]
fn test_session_step_out_at_top_level() {
    let source = "let x = 1;\nlet y = 2;";
    let bc = compile(source);
    let mut session = DebuggerSession::new(bc, source, "test.atlas");
    session.process_request(DebugRequest::StepOut);
    // At top level, step-out should run to completion
    let resp = session.run_until_pause(&security());
    match resp {
        DebugResponse::Paused { .. } => {} // completed
        r => panic!("unexpected: {:?}", r),
    }
}

#[test]
fn test_session_continue_runs_to_end() {
    let source = "let x = 1;\nlet y = 2;";
    let bc = compile(source);
    let mut session = DebuggerSession::new(bc, source, "test.atlas");
    session.process_request(DebugRequest::Continue);
    let resp = session.run_until_pause(&security());
    match resp {
        DebugResponse::Paused { .. } => assert!(session.is_stopped()),
        r => panic!("unexpected: {:?}", r),
    }
}

#[test]
fn test_session_multiple_breakpoints() {
    let source = "let a = 1;\nlet b = 2;\nlet c = 3;\nlet d = 4;";
    let bc = compile(source);
    let mut session = DebuggerSession::new(bc, source, "test.atlas");

    session.process_request(DebugRequest::SetBreakpoint {
        location: SourceLocation::new("test.atlas", 1, 1),
    });
    session.process_request(DebugRequest::SetBreakpoint {
        location: SourceLocation::new("test.atlas", 3, 1),
    });

    // Both breakpoints registered
    if let DebugResponse::Breakpoints { breakpoints } =
        session.process_request(DebugRequest::ListBreakpoints)
    {
        assert_eq!(breakpoints.len(), 2);
    }
}

#[test]
fn test_session_remove_breakpoint() {
    let source = "let x = 1;";
    let bc = compile(source);
    let mut session = DebuggerSession::new(bc, source, "test.atlas");
    session.process_request(DebugRequest::SetBreakpoint { location: loc(1) });
    let resp = session.process_request(DebugRequest::RemoveBreakpoint { id: 1 });
    match resp {
        DebugResponse::BreakpointRemoved { id } => assert_eq!(id, 1),
        r => panic!("unexpected: {:?}", r),
    }
}

#[test]
fn test_session_remove_nonexistent_breakpoint() {
    let source = "let x = 1;";
    let bc = compile(source);
    let mut session = DebuggerSession::new(bc, source, "test.atlas");
    let resp = session.process_request(DebugRequest::RemoveBreakpoint { id: 99 });
    match resp {
        DebugResponse::Error { .. } => {}
        r => panic!("expected Error, got {:?}", r),
    }
}

#[test]
fn test_session_clear_breakpoints() {
    let source = "let x = 1;\nlet y = 2;";
    let bc = compile(source);
    let mut session = DebuggerSession::new(bc, source, "test.atlas");
    session.process_request(DebugRequest::SetBreakpoint { location: loc(1) });
    session.process_request(DebugRequest::SetBreakpoint { location: loc(2) });
    session.process_request(DebugRequest::ClearBreakpoints);
    assert_eq!(session.debug_state().breakpoint_count(), 0);
}

#[test]
fn test_session_get_location() {
    let source = "let x = 1;";
    let bc = compile(source);
    let mut session = DebuggerSession::new(bc, source, "test.atlas");
    match session.process_request(DebugRequest::GetLocation) {
        DebugResponse::Location { ip, .. } => assert_eq!(ip, 0),
        r => panic!("unexpected: {:?}", r),
    }
}

#[test]
fn test_session_get_stack() {
    let source = "let x = 1;";
    let bc = compile(source);
    let mut session = DebuggerSession::new(bc, source, "test.atlas");
    match session.process_request(DebugRequest::GetStack) {
        DebugResponse::StackTrace { frames } => {
            assert!(!frames.is_empty());
            assert_eq!(frames[0].function_name, "<main>");
        }
        r => panic!("unexpected: {:?}", r),
    }
}

#[test]
fn test_session_get_variables() {
    let source = "let x = 1;";
    let bc = compile(source);
    let mut session = DebuggerSession::new(bc, source, "test.atlas");
    match session.process_request(DebugRequest::GetVariables { frame_index: 0 }) {
        DebugResponse::Variables { frame_index, .. } => assert_eq!(frame_index, 0),
        r => panic!("unexpected: {:?}", r),
    }
}

#[test]
fn test_session_pause_request() {
    let source = "let x = 1;";
    let bc = compile(source);
    let mut session = DebuggerSession::new(bc, source, "test.atlas");
    let resp = session.process_request(DebugRequest::Pause);
    match resp {
        DebugResponse::Resumed => {}
        r => panic!("unexpected: {:?}", r),
    }
}

#[test]
fn test_session_eval_arithmetic() {
    let source = "let x = 1;";
    let bc = compile(source);
    let mut session = DebuggerSession::new(bc, source, "test.atlas");
    match session.process_request(DebugRequest::Evaluate {
        expression: "2 + 3".into(),
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
fn test_session_eval_boolean() {
    let source = "let x = 1;";
    let bc = compile(source);
    let mut session = DebuggerSession::new(bc, source, "test.atlas");
    match session.process_request(DebugRequest::Evaluate {
        expression: "true && false".into(),
        frame_index: 0,
    }) {
        DebugResponse::EvalResult { type_name, .. } => {
            assert_eq!(type_name, "bool");
        }
        r => panic!("unexpected: {:?}", r),
    }
}

#[test]
fn test_session_eval_string_concat() {
    let source = "let x = 1;";
    let bc = compile(source);
    let mut session = DebuggerSession::new(bc, source, "test.atlas");
    match session.process_request(DebugRequest::Evaluate {
        expression: r#""hello" + " world""#.into(),
        frame_index: 0,
    }) {
        DebugResponse::EvalResult { value, type_name } => {
            assert_eq!(type_name, "string");
            assert!(value.contains("hello"));
        }
        r => panic!("unexpected: {:?}", r),
    }
}

#[test]
fn test_session_eval_invalid_returns_error() {
    let source = "let x = 1;";
    let bc = compile(source);
    let mut session = DebuggerSession::new(bc, source, "test.atlas");
    match session.process_request(DebugRequest::Evaluate {
        expression: "!!!invalid$$$".into(),
        frame_index: 0,
    }) {
        DebugResponse::EvalResult { .. } | DebugResponse::Error { .. } => {}
        r => panic!("unexpected: {:?}", r),
    }
}

#[test]
fn test_session_not_paused_initially() {
    let source = "let x = 1;";
    let bc = compile(source);
    let session = DebuggerSession::new(bc, source, "test.atlas");
    assert!(!session.is_paused());
    assert!(!session.is_stopped());
}

#[test]
fn test_session_current_ip_starts_at_zero() {
    let source = "let x = 1;";
    let bc = compile(source);
    let session = DebuggerSession::new(bc, source, "test.atlas");
    assert_eq!(session.current_ip(), 0);
}

#[test]
fn test_session_source_map_populated() {
    let source = "let x = 42;\nlet y = x + 1;";
    let bc = compile(source);
    let session = DebuggerSession::new(bc, source, "test.atlas");
    assert!(!session.source_map().is_empty());
}

#[test]
fn test_session_run_without_breakpoints_completes() {
    let source = "let x = 1;\nlet y = 2;";
    let bc = compile(source);
    let mut session = DebuggerSession::new(bc, source, "test.atlas");
    let resp = session.run_until_pause(&security());
    match resp {
        DebugResponse::Paused { .. } | DebugResponse::Error { .. } => {}
        r => panic!("unexpected: {:?}", r),
    }
    assert!(session.is_stopped());
}

#[test]
fn test_session_conditional_code_debug() {
    let source = "let x = 5;\nlet y = 0;\nif x > 3 {\n  y = x * 2;\n}";
    let bc = compile(source);
    let mut session = DebuggerSession::new(bc, source, "test.atlas");
    // Set step-into to go through conditional
    session.process_request(DebugRequest::StepInto);
    let resp = session.run_until_pause(&security());
    match resp {
        DebugResponse::Paused { .. } => {}
        r => panic!("unexpected: {:?}", r),
    }
}

#[test]
fn test_session_debug_state_accessible() {
    let source = "let x = 1;";
    let bc = compile(source);
    let session = DebuggerSession::new(bc, source, "test.atlas");
    let state = session.debug_state();
    assert!(state.is_running());
}

// ══════════════════════════════════════════════════════════════════════════════
// BreakpointCondition Edge Cases
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_condition_always_default() {
    let cond = BreakpointCondition::default();
    assert_eq!(cond, BreakpointCondition::Always);
}

#[test]
fn test_hit_count_zero_fires_immediately() {
    let mut mgr = BreakpointManager::new();
    let id = mgr.add_conditional(loc(1), BreakpointCondition::HitCount(0));
    mgr.verify(id, 10);
    // hit_count(0) means fire when hit_count >= 0, which is always true after increment
    assert_eq!(mgr.check_offset(10), ShouldFire::Pause);
}

#[test]
fn test_hit_count_multiple_zero_never_fires() {
    let mut mgr = BreakpointManager::new();
    let id = mgr.add_conditional(loc(1), BreakpointCondition::HitCountMultiple(0));
    mgr.verify(id, 10);
    assert_eq!(mgr.check_offset(10), ShouldFire::Skip);
    assert_eq!(mgr.check_offset(10), ShouldFire::Skip);
}

#[test]
fn test_bp_manager_drain_log_empty() {
    let mut mgr = BreakpointManager::new();
    let logs = mgr.drain_log_output();
    assert!(logs.is_empty());
}

#[test]
fn test_bp_manager_verify_nonexistent() {
    let mut mgr = BreakpointManager::new();
    assert!(!mgr.verify(99, 42));
}

#[test]
fn test_bp_manager_enable_nonexistent() {
    let mut mgr = BreakpointManager::new();
    assert!(!mgr.enable(99));
}

#[test]
fn test_bp_manager_disable_nonexistent() {
    let mut mgr = BreakpointManager::new();
    assert!(!mgr.disable(99));
}

#[test]
fn test_bp_manager_set_condition_nonexistent() {
    let mut mgr = BreakpointManager::new();
    assert!(!mgr.set_condition(99, BreakpointCondition::Always));
}

#[test]
fn test_bp_manager_all_breakpoints_protocol() {
    let mut mgr = BreakpointManager::new();
    mgr.add(loc(1));
    mgr.add(loc(2));
    let bps = mgr.all_breakpoints();
    assert_eq!(bps.len(), 2);
    assert_eq!(bps[0].id, 1);
    assert_eq!(bps[1].id, 2);
}

#[test]
fn test_bp_entry_reset_hit_count() {
    let bp = Breakpoint::verified_at(1, loc(1), 10);
    let mut entry = BreakpointEntry::new(bp);
    entry.check_and_increment();
    assert_eq!(entry.hit_count, 1);
    entry.reset_hit_count();
    assert_eq!(entry.hit_count, 0);
}

#[test]
fn test_bp_entry_with_condition() {
    let bp = Breakpoint::new(1, loc(1));
    let entry = BreakpointEntry::with_condition(bp, BreakpointCondition::HitCount(10));
    assert_eq!(entry.condition, BreakpointCondition::HitCount(10));
    assert!(entry.enabled);
}

#[test]
fn test_session_list_breakpoints_empty() {
    let source = "let x = 1;";
    let bc = compile(source);
    let mut session = DebuggerSession::new(bc, source, "test.atlas");
    match session.process_request(DebugRequest::ListBreakpoints) {
        DebugResponse::Breakpoints { breakpoints } => assert!(breakpoints.is_empty()),
        r => panic!("unexpected: {:?}", r),
    }
}

// --- Value inspection ---

// Debugger inspection tests — Phase 05.
//
// Tests variable inspection, expression evaluation, watch expressions,
// hover, and the Inspector API.

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
    assert_eq!(
        format_value_with_depth(&Value::Number(std::f64::consts::PI), 3),
        std::f64::consts::PI.to_string()
    );
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

// --- Debug protocol serialization ---

// Integration tests for the Atlas debugger infrastructure (phase-04).
//
// Covers:
// 1. Protocol request/response serialization
// 2. Source mapping (bidirectional accuracy)
// 3. Breakpoint management (set, remove, hit)
// 4. Step operations (into, over, out)
// 5. Variable inspection at breakpoints
// 6. Stack trace generation
// 7. Expression evaluation in context
// 8. Performance impact when debugging is disabled

// ── Helpers ───────────────────────────────────────────────────────────────────

fn new_session(source: &str) -> DebuggerSession {
    let bc = compile(source);
    DebuggerSession::new(bc, source, "test.atlas")
}

// ═════════════════════════════════════════════════════════════════════════════
// 1. Protocol serialization
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn proto_serialize_set_breakpoint() {
    let req = DebugRequest::SetBreakpoint { location: loc(5) };
    let json = serialize_request(&req).unwrap();
    let back: DebugRequest = deserialize_request(&json).unwrap();
    assert_eq!(req, back);
}

#[test]
fn proto_serialize_remove_breakpoint() {
    let req = DebugRequest::RemoveBreakpoint { id: 3 };
    let json = serialize_request(&req).unwrap();
    let back: DebugRequest = deserialize_request(&json).unwrap();
    assert_eq!(req, back);
}

#[test]
fn proto_serialize_continue() {
    let req = DebugRequest::Continue;
    let json = serialize_request(&req).unwrap();
    let back: DebugRequest = deserialize_request(&json).unwrap();
    assert_eq!(req, back);
}

#[test]
fn proto_serialize_step_over() {
    let req = DebugRequest::StepOver;
    let json = serialize_request(&req).unwrap();
    assert!(json.contains("StepOver"));
}

#[test]
fn proto_serialize_step_into() {
    let req = DebugRequest::StepInto;
    let json = serialize_request(&req).unwrap();
    assert!(json.contains("StepInto"));
}

#[test]
fn proto_serialize_step_out() {
    let req = DebugRequest::StepOut;
    let json = serialize_request(&req).unwrap();
    assert!(json.contains("StepOut"));
}

#[test]
fn proto_serialize_get_variables() {
    let req = DebugRequest::GetVariables { frame_index: 2 };
    let json = serialize_request(&req).unwrap();
    let back: DebugRequest = deserialize_request(&json).unwrap();
    assert_eq!(req, back);
}

#[test]
fn proto_serialize_evaluate() {
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
    let resp = DebugResponse::Error {
        message: "unknown error".to_string(),
    };
    let json = serialize_response(&resp).unwrap();
    let back: DebugResponse = deserialize_response(&json).unwrap();
    assert_eq!(resp, back);
}

#[test]
fn proto_serialize_debug_event_paused() {
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
    let source = "let x = 10; let y = 20; let z = x + y;";
    let bc = compile(source);
    let mut vm = VM::new(bc);
    let sec = security();
    let result = vm.run(&sec);
    assert!(result.is_ok(), "VM should run without errors");
}

#[test]
fn perf_debugger_disabled_by_default() {
    let bc = compile("let x = 1;");
    let vm = VM::new(bc);
    // Debugger should be None (disabled) on a plain VM
    assert!(vm.debugger().is_none());
}

#[test]
fn perf_debugger_disabled_after_run() {
    // Running without debugging doesn't accidentally enable debugging.
    let bc = compile("let x = 1;");
    let mut vm = VM::new(bc);
    let sec = security();
    vm.run(&sec).unwrap();
    assert!(vm.debugger().is_none() || !vm.debugger().unwrap().is_enabled());
}

#[test]
fn perf_run_completes_correctly_multiple_operations() {
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

// ═══════════════════════════════════════════════════════════════════════════════
// INTERPRETER DEBUGGER TESTS (Phase 01 - Parity with VM debugger)
// ═══════════════════════════════════════════════════════════════════════════════

use atlas_runtime::interpreter::debugger::InterpreterDebuggerSession;

fn interp_session(source: &str) -> InterpreterDebuggerSession {
    InterpreterDebuggerSession::new(source, "test.atlas")
}

// ── Interpreter Debugger: Session creation ────────────────────────────────────

#[test]
fn interp_session_creation() {
    let session = interp_session("let x = 1;");
    assert!(!session.is_paused());
    assert!(!session.is_stopped());
}

#[test]
fn interp_session_initial_frame_depth() {
    let session = interp_session("let x = 1;");
    assert_eq!(session.frame_depth(), 1); // Main frame
}

// ── Interpreter Debugger: Breakpoint management ───────────────────────────────

#[test]
fn interp_set_breakpoint_returns_id() {
    let mut session = interp_session("let x = 1;\nlet y = 2;");
    match session.process_request(DebugRequest::SetBreakpoint { location: loc(1) }) {
        DebugResponse::BreakpointSet { breakpoint } => assert_eq!(breakpoint.id, 1),
        r => panic!("expected BreakpointSet, got {:?}", r),
    }
}

#[test]
fn interp_set_multiple_breakpoints() {
    let mut session = interp_session("let a = 1;\nlet b = 2;\nlet c = 3;");
    session.process_request(DebugRequest::SetBreakpoint { location: loc(1) });
    session.process_request(DebugRequest::SetBreakpoint { location: loc(2) });
    session.process_request(DebugRequest::SetBreakpoint { location: loc(3) });

    match session.process_request(DebugRequest::ListBreakpoints) {
        DebugResponse::Breakpoints { breakpoints } => assert_eq!(breakpoints.len(), 3),
        r => panic!("unexpected: {:?}", r),
    }
}

#[test]
fn interp_remove_breakpoint() {
    let mut session = interp_session("let x = 1;");
    session.process_request(DebugRequest::SetBreakpoint { location: loc(1) });
    match session.process_request(DebugRequest::RemoveBreakpoint { id: 1 }) {
        DebugResponse::BreakpointRemoved { id } => assert_eq!(id, 1),
        r => panic!("unexpected: {:?}", r),
    }
}

#[test]
fn interp_remove_nonexistent_breakpoint_error() {
    let mut session = interp_session("let x = 1;");
    match session.process_request(DebugRequest::RemoveBreakpoint { id: 99 }) {
        DebugResponse::Error { .. } => {}
        r => panic!("expected Error, got {:?}", r),
    }
}

#[test]
fn interp_clear_breakpoints() {
    let mut session = interp_session("let x = 1;\nlet y = 2;");
    session.process_request(DebugRequest::SetBreakpoint { location: loc(1) });
    session.process_request(DebugRequest::SetBreakpoint { location: loc(2) });
    session.process_request(DebugRequest::ClearBreakpoints);
    assert_eq!(session.debug_state().breakpoint_count(), 0);
}

#[test]
fn interp_list_breakpoints_empty() {
    let mut session = interp_session("let x = 1;");
    match session.process_request(DebugRequest::ListBreakpoints) {
        DebugResponse::Breakpoints { breakpoints } => assert!(breakpoints.is_empty()),
        r => panic!("unexpected: {:?}", r),
    }
}

// ── Interpreter Debugger: Step modes ──────────────────────────────────────────

#[test]
fn interp_step_into_mode_set() {
    let mut session = interp_session("let x = 1;");
    session.process_request(DebugRequest::StepInto);
    assert_eq!(session.debug_state().step_mode, StepMode::Into);
}

#[test]
fn interp_step_over_mode_set() {
    let mut session = interp_session("let x = 1;");
    session.process_request(DebugRequest::StepOver);
    assert_eq!(session.debug_state().step_mode, StepMode::Over);
}

#[test]
fn interp_step_out_mode_set() {
    let mut session = interp_session("let x = 1;");
    session.process_request(DebugRequest::StepOut);
    assert_eq!(session.debug_state().step_mode, StepMode::Out);
}

#[test]
fn interp_step_into_pauses_execution() {
    let source = "let x = 1;\nlet y = 2;";
    let mut session = interp_session(source);
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
fn interp_continue_runs_to_end() {
    let source = "let x = 1;\nlet y = 2;";
    let mut session = interp_session(source);
    session.process_request(DebugRequest::Continue);
    session.run_until_pause(&security());
    assert!(session.is_stopped());
}

// ── Interpreter Debugger: Stack trace ─────────────────────────────────────────

#[test]
fn interp_get_stack_has_main() {
    let mut session = interp_session("let x = 1;");
    match session.process_request(DebugRequest::GetStack) {
        DebugResponse::StackTrace { frames } => {
            assert!(!frames.is_empty());
            assert_eq!(frames[0].function_name, "<main>");
        }
        r => panic!("unexpected: {:?}", r),
    }
}

#[test]
fn interp_stack_frame_index_zero() {
    let mut session = interp_session("let x = 1;");
    match session.process_request(DebugRequest::GetStack) {
        DebugResponse::StackTrace { frames } => {
            assert_eq!(frames[0].index, 0);
        }
        r => panic!("unexpected: {:?}", r),
    }
}

// ── Interpreter Debugger: Variable inspection ─────────────────────────────────

#[test]
fn interp_get_variables_frame_0() {
    let mut session = interp_session("let x = 1;");
    match session.process_request(DebugRequest::GetVariables { frame_index: 0 }) {
        DebugResponse::Variables { frame_index, .. } => assert_eq!(frame_index, 0),
        r => panic!("unexpected: {:?}", r),
    }
}

#[test]
fn interp_get_variables_includes_globals() {
    let source = "let x = 42;";
    let mut session = interp_session(source);
    // Run to completion so variable is defined
    session.process_request(DebugRequest::Continue);
    session.run_until_pause(&security());

    match session.process_request(DebugRequest::GetVariables { frame_index: 0 }) {
        DebugResponse::Variables { variables, .. } => {
            // Should have builtins at minimum
            assert!(!variables.is_empty());
        }
        r => panic!("unexpected: {:?}", r),
    }
}

// ── Interpreter Debugger: Expression evaluation ───────────────────────────────

#[test]
fn interp_eval_simple_arithmetic() {
    let mut session = interp_session("let x = 1;");
    match session.process_request(DebugRequest::Evaluate {
        expression: "2 + 3".into(),
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
fn interp_eval_string_expression() {
    let mut session = interp_session("let x = 1;");
    match session.process_request(DebugRequest::Evaluate {
        expression: r#""hello" + " world""#.into(),
        frame_index: 0,
    }) {
        DebugResponse::EvalResult { value, type_name } => {
            assert_eq!(type_name, "string");
            assert!(value.contains("hello"));
        }
        r => panic!("unexpected: {:?}", r),
    }
}

#[test]
fn interp_eval_boolean_expression() {
    let mut session = interp_session("let x = 1;");
    match session.process_request(DebugRequest::Evaluate {
        expression: "true && false".into(),
        frame_index: 0,
    }) {
        DebugResponse::EvalResult { type_name, .. } => {
            assert_eq!(type_name, "bool");
        }
        r => panic!("unexpected: {:?}", r),
    }
}

#[test]
fn interp_eval_null_literal() {
    let mut session = interp_session("let x = 1;");
    match session.process_request(DebugRequest::Evaluate {
        expression: "null".into(),
        frame_index: 0,
    }) {
        DebugResponse::EvalResult { value, type_name } => {
            assert_eq!(type_name, "null");
            assert!(value.contains("null"));
        }
        r => panic!("unexpected: {:?}", r),
    }
}

#[test]
fn interp_eval_invalid_syntax_error() {
    let mut session = interp_session("let x = 1;");
    match session.process_request(DebugRequest::Evaluate {
        expression: "!!!bad$$$".into(),
        frame_index: 0,
    }) {
        DebugResponse::EvalResult { .. } | DebugResponse::Error { .. } => {}
        r => panic!("unexpected: {:?}", r),
    }
}

// ── Interpreter Debugger: Location ────────────────────────────────────────────

#[test]
fn interp_get_location_initial() {
    let mut session = interp_session("let x = 1;");
    match session.process_request(DebugRequest::GetLocation) {
        DebugResponse::Location { ip, .. } => assert_eq!(ip, 0),
        r => panic!("unexpected: {:?}", r),
    }
}

// ── Interpreter Debugger: Pause request ───────────────────────────────────────

#[test]
fn interp_pause_request_sets_step_mode() {
    let mut session = interp_session("let x = 1;");
    session.process_request(DebugRequest::Pause);
    // Pause sets step-into mode
    assert_eq!(session.debug_state().step_mode, StepMode::Into);
}

// ── Interpreter Debugger: End-to-end ──────────────────────────────────────────

#[test]
fn interp_e2e_run_to_completion() {
    let source = "let x = 1 + 2;\nlet y = x * 3;";
    let mut session = interp_session(source);
    session.run_until_pause(&security());
    assert!(session.is_stopped());
}

#[test]
fn interp_e2e_breakpoint_ids_sequential() {
    let mut session = interp_session("let x = 1;\nlet y = 2;");
    let id1 = match session.process_request(DebugRequest::SetBreakpoint { location: loc(1) }) {
        DebugResponse::BreakpointSet { breakpoint } => breakpoint.id,
        r => panic!("{:?}", r),
    };
    let id2 = match session.process_request(DebugRequest::SetBreakpoint { location: loc(2) }) {
        DebugResponse::BreakpointSet { breakpoint } => breakpoint.id,
        r => panic!("{:?}", r),
    };
    assert_ne!(id1, id2);
    assert_eq!(id2, id1 + 1);
}

#[test]
fn interp_e2e_conditional_code() {
    let source = "let x = 5;\nif (x > 3) {\n  let y = x * 2;\n}";
    let mut session = interp_session(source);
    session.process_request(DebugRequest::StepInto);
    let resp = session.run_until_pause(&security());
    match resp {
        DebugResponse::Paused { .. } => {}
        r => panic!("unexpected: {:?}", r),
    }
}

#[test]
fn interp_debug_state_accessible() {
    let session = interp_session("let x = 1;");
    let state = session.debug_state();
    assert!(state.is_running());
}

// ── Interpreter-VM Parity Tests ───────────────────────────────────────────────

#[test]
fn parity_both_support_set_breakpoint() {
    let source = "let x = 1;\nlet y = 2;";

    let mut vm_session = new_session(source);
    let mut interp_session = interp_session(source);

    let vm_resp = vm_session.process_request(DebugRequest::SetBreakpoint { location: loc(1) });
    let interp_resp =
        interp_session.process_request(DebugRequest::SetBreakpoint { location: loc(1) });

    match (vm_resp, interp_resp) {
        (DebugResponse::BreakpointSet { .. }, DebugResponse::BreakpointSet { .. }) => {}
        r => panic!("expected both BreakpointSet, got {:?}", r),
    }
}

#[test]
fn parity_both_support_list_breakpoints() {
    let source = "let x = 1;";

    let mut vm_session = new_session(source);
    let mut interp_session = interp_session(source);

    let vm_resp = vm_session.process_request(DebugRequest::ListBreakpoints);
    let interp_resp = interp_session.process_request(DebugRequest::ListBreakpoints);

    match (vm_resp, interp_resp) {
        (DebugResponse::Breakpoints { .. }, DebugResponse::Breakpoints { .. }) => {}
        r => panic!("expected both Breakpoints, got {:?}", r),
    }
}

#[test]
fn parity_both_support_step_into() {
    let source = "let x = 1;";

    let mut vm_session = new_session(source);
    let mut interp_session = interp_session(source);

    vm_session.process_request(DebugRequest::StepInto);
    interp_session.process_request(DebugRequest::StepInto);

    assert_eq!(vm_session.debug_state().step_mode, StepMode::Into);
    assert_eq!(interp_session.debug_state().step_mode, StepMode::Into);
}

#[test]
fn parity_both_support_step_over() {
    let source = "let x = 1;";

    let mut vm_session = new_session(source);
    let mut interp_session = interp_session(source);

    vm_session.process_request(DebugRequest::StepOver);
    interp_session.process_request(DebugRequest::StepOver);

    assert_eq!(vm_session.debug_state().step_mode, StepMode::Over);
    assert_eq!(interp_session.debug_state().step_mode, StepMode::Over);
}

#[test]
fn parity_both_support_step_out() {
    let source = "let x = 1;";

    let mut vm_session = new_session(source);
    let mut interp_session = interp_session(source);

    vm_session.process_request(DebugRequest::StepOut);
    interp_session.process_request(DebugRequest::StepOut);

    assert_eq!(vm_session.debug_state().step_mode, StepMode::Out);
    assert_eq!(interp_session.debug_state().step_mode, StepMode::Out);
}

#[test]
fn parity_both_support_get_stack() {
    let source = "let x = 1;";

    let mut vm_session = new_session(source);
    let mut interp_session = interp_session(source);

    let vm_resp = vm_session.process_request(DebugRequest::GetStack);
    let interp_resp = interp_session.process_request(DebugRequest::GetStack);

    match (vm_resp, interp_resp) {
        (DebugResponse::StackTrace { frames: f1 }, DebugResponse::StackTrace { frames: f2 }) => {
            assert!(!f1.is_empty());
            assert!(!f2.is_empty());
            // Both should have <main> frame
            assert_eq!(f1[0].function_name, "<main>");
            assert_eq!(f2[0].function_name, "<main>");
        }
        r => panic!("expected both StackTrace, got {:?}", r),
    }
}

#[test]
fn parity_both_support_get_variables() {
    let source = "let x = 1;";

    let mut vm_session = new_session(source);
    let mut interp_session = interp_session(source);

    let vm_resp = vm_session.process_request(DebugRequest::GetVariables { frame_index: 0 });
    let interp_resp = interp_session.process_request(DebugRequest::GetVariables { frame_index: 0 });

    match (vm_resp, interp_resp) {
        (
            DebugResponse::Variables {
                frame_index: fi1, ..
            },
            DebugResponse::Variables {
                frame_index: fi2, ..
            },
        ) => {
            assert_eq!(fi1, 0);
            assert_eq!(fi2, 0);
        }
        r => panic!("expected both Variables, got {:?}", r),
    }
}

#[test]
fn parity_both_support_evaluate() {
    let source = "let x = 1;";

    let mut vm_session = new_session(source);
    let mut interp_session = interp_session(source);

    let vm_resp = vm_session.process_request(DebugRequest::Evaluate {
        expression: "1 + 2".into(),
        frame_index: 0,
    });
    let interp_resp = interp_session.process_request(DebugRequest::Evaluate {
        expression: "1 + 2".into(),
        frame_index: 0,
    });

    match (vm_resp, interp_resp) {
        (
            DebugResponse::EvalResult {
                value: v1,
                type_name: t1,
            },
            DebugResponse::EvalResult {
                value: v2,
                type_name: t2,
            },
        ) => {
            assert_eq!(t1, "number");
            assert_eq!(t2, "number");
            assert!(v1.contains('3'));
            assert!(v2.contains('3'));
        }
        r => panic!("expected both EvalResult, got {:?}", r),
    }
}

#[test]
fn parity_both_support_continue() {
    let source = "let x = 1;";

    let mut vm_session = new_session(source);
    let mut interp_session = interp_session(source);

    let vm_resp = vm_session.process_request(DebugRequest::Continue);
    let interp_resp = interp_session.process_request(DebugRequest::Continue);

    match (vm_resp, interp_resp) {
        (DebugResponse::Resumed, DebugResponse::Resumed) => {}
        r => panic!("expected both Resumed, got {:?}", r),
    }
}
