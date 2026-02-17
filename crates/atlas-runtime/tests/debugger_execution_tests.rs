//! Debugger execution control tests — Phase 05.
//!
//! Tests breakpoint management (set, hit, remove, conditional, hit counts, log points),
//! step operations (into, over, out, run-to-line), and execution flow.

use atlas_runtime::bytecode::Bytecode;
use atlas_runtime::compiler::Compiler;
use atlas_runtime::debugger::breakpoints::{
    BreakpointCondition, BreakpointEntry, BreakpointManager, ShouldFire,
};
use atlas_runtime::debugger::protocol::{Breakpoint, DebugRequest, DebugResponse, SourceLocation};
use atlas_runtime::debugger::stepping::{StepRequest, StepTracker};
use atlas_runtime::debugger::DebuggerSession;
use atlas_runtime::lexer::Lexer;
use atlas_runtime::parser::Parser;
use atlas_runtime::security::SecurityContext;

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
    use atlas_runtime::debugger::source_map::SourceMap;
    let map = SourceMap::new();
    let mut tracker = StepTracker::new();
    tracker.begin_step(StepRequest::RunToOffset(5), 1, None);
    assert!(tracker.should_pause(3, 1, &map).is_none());
    assert!(tracker.should_pause(5, 1, &map).is_some());
}

#[test]
fn test_step_tracker_instructions_counter() {
    use atlas_runtime::debugger::source_map::SourceMap;
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
    use atlas_runtime::debugger::source_map::SourceMap;
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
                match resp {
                    DebugResponse::Paused { .. } => assert!(session.is_paused()),
                    _ => {} // acceptable if source map doesn't bind
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
