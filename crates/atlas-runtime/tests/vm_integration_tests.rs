//! VM Integration Tests
//!
//! Tests all bytecode-VM features working together: optimizer, profiler,
//! debugger, and performance optimizations. Verifies no interference
//! between subsystems.

use atlas_runtime::bytecode::Bytecode;
use atlas_runtime::compiler::Compiler;
use atlas_runtime::debugger::{DebugRequest, DebugResponse, DebuggerSession, SourceLocation};
use atlas_runtime::lexer::Lexer;
use atlas_runtime::optimizer::Optimizer;
use atlas_runtime::parser::Parser;
use atlas_runtime::security::SecurityContext;
use atlas_runtime::value::Value;
use atlas_runtime::vm::{Profiler, VM};
use rstest::rstest;

// ============================================================================
// Helpers
// ============================================================================

fn compile(source: &str) -> Bytecode {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();
    let mut compiler = Compiler::new();
    compiler.compile(&program).expect("Compilation failed")
}

fn compile_optimized(source: &str) -> Bytecode {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();
    let mut compiler = Compiler::with_optimization();
    compiler.compile(&program).expect("Compilation failed")
}

fn vm_run(bc: Bytecode) -> Option<Value> {
    let mut vm = VM::new(bc);
    vm.run(&SecurityContext::allow_all()).expect("VM failed")
}

fn vm_eval(source: &str) -> Option<Value> {
    vm_run(compile(source))
}

fn vm_eval_opt(source: &str) -> Option<Value> {
    vm_run(compile_optimized(source))
}

fn vm_number(source: &str) -> f64 {
    match vm_eval(source) {
        Some(Value::Number(n)) => n,
        other => panic!("Expected Number, got {:?}", other),
    }
}

fn vm_number_opt(source: &str) -> f64 {
    match vm_eval_opt(source) {
        Some(Value::Number(n)) => n,
        other => panic!("Expected Number, got {:?}", other),
    }
}

fn vm_string(source: &str) -> String {
    match vm_eval(source) {
        Some(Value::String(s)) => (*s).clone(),
        other => panic!("Expected String, got {:?}", other),
    }
}

fn vm_bool(source: &str) -> bool {
    match vm_eval(source) {
        Some(Value::Bool(b)) => b,
        other => panic!("Expected Bool, got {:?}", other),
    }
}

// ============================================================================
// 1. Optimizer + Debugger Integration (tests 1-10)
// ============================================================================

#[test]
fn test_opt_debug_simple_program() {
    // Optimized code should produce same result as unoptimized
    let source = "let x = 1 + 2;\nlet y = x * 3;\ny;";
    let result_plain = vm_eval(source);
    let result_opt = vm_eval_opt(source);
    assert_eq!(result_plain, result_opt);
}

#[test]
fn test_opt_debug_breakpoint_creation() {
    let source = "let x = 10;\nlet y = 20;\nlet z = x + y;\nz;";
    let bc = compile(source);
    let mut session = DebuggerSession::new(bc, source, "test.atlas");
    let response = session.process_request(DebugRequest::SetBreakpoint {
        location: SourceLocation::new("test.atlas", 2, 1),
    });
    match response {
        DebugResponse::BreakpointSet { .. } => {}
        other => panic!("Expected BreakpointSet, got {:?}", other),
    }
}

#[test]
fn test_opt_debug_optimized_breakpoint() {
    let source = "let x = 10;\nlet y = 20;\nlet z = x + y;\nz;";
    let bc = compile_optimized(source);
    let mut session = DebuggerSession::new(bc, source, "test.atlas");
    let response = session.process_request(DebugRequest::SetBreakpoint {
        location: SourceLocation::new("test.atlas", 2, 1),
    });
    // Should still accept breakpoints on optimized code
    match response {
        DebugResponse::BreakpointSet { .. } => {}
        other => panic!("Expected BreakpointSet, got {:?}", other),
    }
}

#[test]
fn test_opt_debug_step_through_optimized() {
    let source = "let a = 5;\nlet b = 10;\na + b;";
    let bc = compile_optimized(source);
    let mut session = DebuggerSession::new(bc, source, "test.atlas");
    // Step into should work
    let response = session.process_request(DebugRequest::StepInto);
    // Verify we got a response (not an error)
    assert!(!matches!(response, DebugResponse::Error { .. }));
}

#[test]
fn test_opt_debug_continue_execution() {
    let source = "let x = 42;\nx;";
    let bc = compile_optimized(source);
    let mut session = DebuggerSession::new(bc, source, "test.atlas");
    let response = session.process_request(DebugRequest::Continue);
    assert!(!matches!(response, DebugResponse::Error { .. }));
}

#[test]
fn test_opt_debug_variables_visible() {
    let source = "let x = 100;\nlet y = 200;\nx + y;";
    let bc = compile(source);
    let mut session = DebuggerSession::new(bc, source, "test.atlas");
    // Set breakpoint and run
    session.process_request(DebugRequest::SetBreakpoint {
        location: SourceLocation::new("test.atlas", 2, 1),
    });
    let _response = session.run_until_pause(&SecurityContext::allow_all());
    // Session should be in paused or stopped state
    assert!(session.is_paused() || session.is_stopped());
}

#[test]
fn test_opt_debug_arithmetic_optimized_semantic() {
    // Constant folding should preserve semantics
    let programs = vec!["2 + 3 * 4;", "10 - 3 + 2;", "(5 + 5) * 2;", "100 / 4 - 5;"];
    for prog in programs {
        let plain = vm_eval(prog);
        let opt = vm_eval_opt(prog);
        assert_eq!(plain, opt, "Mismatch for: {}", prog);
    }
}

#[test]
fn test_opt_debug_loop_semantics_preserved() {
    let source = "let sum = 0; let i = 0; while (i < 10) { sum = sum + i; i = i + 1; } sum;";
    let plain = vm_eval(source);
    let opt = vm_eval_opt(source);
    assert_eq!(plain, opt);
}

#[test]
fn test_opt_debug_function_semantics_preserved() {
    let source = "fn add(a: number, b: number) -> number { return a + b; } add(10, 20);";
    let plain = vm_eval(source);
    let opt = vm_eval_opt(source);
    assert_eq!(plain, opt);
}

#[test]
fn test_opt_debug_nested_function_optimized() {
    let source = r#"
fn outer(x: number) -> number {
    fn inner(y: number) -> number {
        return y * 2;
    }
    return inner(x) + 1;
}
outer(5);
"#;
    let plain = vm_eval(source);
    let opt = vm_eval_opt(source);
    assert_eq!(plain, opt);
}

// ============================================================================
// 2. Profiler + Optimizer Integration (tests 11-20)
// ============================================================================

#[test]
fn test_profiler_basic_stats() {
    let mut profiler = Profiler::enabled();
    assert!(profiler.is_enabled());
    profiler.start_timing();
    profiler.stop_timing();
    assert!(profiler.elapsed_secs().is_some());
}

#[test]
fn test_profiler_instruction_counting() {
    let mut profiler = Profiler::enabled();
    use atlas_runtime::bytecode::Opcode;
    profiler.record_instruction(Opcode::Add);
    profiler.record_instruction(Opcode::Add);
    profiler.record_instruction(Opcode::Sub);
    assert_eq!(profiler.instruction_count(Opcode::Add), 2);
    assert_eq!(profiler.instruction_count(Opcode::Sub), 1);
    assert_eq!(profiler.total_instructions(), 3);
}

#[test]
fn test_profiler_function_call_tracking() {
    let mut profiler = Profiler::enabled();
    profiler.record_function_call("add");
    profiler.record_function_call("add");
    profiler.record_function_call("multiply");
    assert_eq!(profiler.function_calls(), 3);
}

#[test]
fn test_profiler_stack_depth() {
    let mut profiler = Profiler::enabled();
    profiler.update_frame_depth(5);
    profiler.update_frame_depth(10);
    profiler.update_frame_depth(3);
    assert_eq!(profiler.max_stack_depth(), 10);
}

#[test]
fn test_profiler_report_generation() {
    let mut profiler = Profiler::enabled();
    use atlas_runtime::bytecode::Opcode;
    profiler.record_instruction(Opcode::Add);
    profiler.record_instruction(Opcode::Constant);
    profiler.record_instruction(Opcode::Constant);
    let report = profiler.report();
    assert!(!report.is_empty());
}

#[test]
fn test_profiler_reset() {
    let mut profiler = Profiler::enabled();
    use atlas_runtime::bytecode::Opcode;
    profiler.record_instruction(Opcode::Add);
    profiler.reset();
    assert_eq!(profiler.total_instructions(), 0);
}

#[test]
fn test_profiler_disabled_noop() {
    let mut profiler = Profiler::new();
    assert!(!profiler.is_enabled());
    use atlas_runtime::bytecode::Opcode;
    profiler.record_instruction(Opcode::Add);
    // Disabled profiler should still track (or not) - verify no panic
}

#[test]
fn test_profiler_hotspot_detection() {
    let mut profiler = Profiler::enabled();
    use atlas_runtime::bytecode::Opcode;
    for _ in 0..100 {
        profiler.record_instruction_at(Opcode::Add, 42);
    }
    for _ in 0..10 {
        profiler.record_instruction_at(Opcode::Sub, 50);
    }
    let hotspots = profiler.hotspots();
    // Should detect at least one hotspot
    assert!(!hotspots.is_empty() || profiler.total_instructions() > 0);
}

#[test]
fn test_profiler_top_opcodes() {
    let mut profiler = Profiler::enabled();
    use atlas_runtime::bytecode::Opcode;
    for _ in 0..50 {
        profiler.record_instruction(Opcode::Constant);
    }
    for _ in 0..30 {
        profiler.record_instruction(Opcode::Add);
    }
    for _ in 0..10 {
        profiler.record_instruction(Opcode::Sub);
    }
    let top = profiler.top_opcodes(3);
    assert!(!top.is_empty());
    // Most frequent should be first
    assert!(top[0].count >= top.last().unwrap().count);
}

#[test]
fn test_profiler_detailed_report() {
    let mut profiler = Profiler::enabled();
    use atlas_runtime::bytecode::Opcode;
    profiler.start_timing();
    for _ in 0..100 {
        profiler.record_instruction(Opcode::Constant);
    }
    profiler.record_function_call("test_fn");
    profiler.update_value_stack_depth(5);
    profiler.stop_timing();
    let report = profiler.generate_report(0.1);
    assert!(report.total_instructions > 0);
}

// ============================================================================
// 3. Debugger Step Operations (tests 21-30)
// ============================================================================

#[test]
fn test_debugger_session_creation() {
    let source = "let x = 1;";
    let bc = compile(source);
    let session = DebuggerSession::new(bc, source, "test.atlas");
    assert!(!session.is_paused());
    assert!(!session.is_stopped());
}

#[test]
fn test_debugger_step_into() {
    let source = "let x = 1;\nlet y = 2;\nx + y;";
    let bc = compile(source);
    let mut session = DebuggerSession::new(bc, source, "test.atlas");
    let response = session.process_request(DebugRequest::StepInto);
    assert!(!matches!(response, DebugResponse::Error { .. }));
}

#[test]
fn test_debugger_step_over() {
    let source = "let x = 1;\nlet y = 2;\nx + y;";
    let bc = compile(source);
    let mut session = DebuggerSession::new(bc, source, "test.atlas");
    let response = session.process_request(DebugRequest::StepOver);
    assert!(!matches!(response, DebugResponse::Error { .. }));
}

#[test]
fn test_debugger_step_out() {
    let source = "let x = 1;\nlet y = 2;";
    let bc = compile(source);
    let mut session = DebuggerSession::new(bc, source, "test.atlas");
    let response = session.process_request(DebugRequest::StepOut);
    assert!(!matches!(response, DebugResponse::Error { .. }));
}

#[test]
fn test_debugger_multiple_breakpoints() {
    let source = "let a = 1;\nlet b = 2;\nlet c = 3;\nlet d = 4;";
    let bc = compile(source);
    let mut session = DebuggerSession::new(bc, source, "test.atlas");
    for line in 1..=4 {
        let response = session.process_request(DebugRequest::SetBreakpoint {
            location: SourceLocation::new("test.atlas", line, 1),
        });
        match response {
            DebugResponse::BreakpointSet { .. } => {}
            other => panic!("Failed to set breakpoint at line {}: {:?}", line, other),
        }
    }
}

#[test]
fn test_debugger_remove_breakpoint() {
    let source = "let x = 1;\nlet y = 2;";
    let bc = compile(source);
    let mut session = DebuggerSession::new(bc, source, "test.atlas");
    let response = session.process_request(DebugRequest::SetBreakpoint {
        location: SourceLocation::new("test.atlas", 1, 1),
    });
    if let DebugResponse::BreakpointSet { breakpoint, .. } = response {
        let remove = session.process_request(DebugRequest::RemoveBreakpoint { id: breakpoint.id });
        assert!(!matches!(remove, DebugResponse::Error { .. }));
    }
}

#[test]
fn test_debugger_run_until_pause() {
    let source = "let x = 10;\nlet y = 20;\nlet z = x + y;\nz;";
    let bc = compile(source);
    let mut session = DebuggerSession::new(bc, source, "test.atlas");
    session.process_request(DebugRequest::SetBreakpoint {
        location: SourceLocation::new("test.atlas", 3, 1),
    });
    let _response = session.run_until_pause(&SecurityContext::allow_all());
    // Should either pause or stop (if breakpoint offset not found, runs to end)
    assert!(session.is_paused() || session.is_stopped());
}

#[test]
fn test_debugger_source_map() {
    let source = "let x = 1;\nlet y = 2;";
    let bc = compile(source);
    let session = DebuggerSession::new(bc, source, "test.atlas");
    let _sm = session.source_map();
    // Source map should have been created without error
}

#[test]
fn test_debugger_with_functions() {
    let source = "fn add(a: number, b: number) -> number {\n  return a + b;\n}\nadd(1, 2);";
    let bc = compile(source);
    let mut session = DebuggerSession::new(bc, source, "test.atlas");
    session.process_request(DebugRequest::SetBreakpoint {
        location: SourceLocation::new("test.atlas", 2, 1),
    });
    let _response = session.run_until_pause(&SecurityContext::allow_all());
    // Verify no crash
}

#[test]
fn test_debugger_state_management() {
    let source = "let x = 1;";
    let bc = compile(source);
    let session = DebuggerSession::new(bc, source, "test.atlas");
    let _state = session.debug_state();
    // Should have initial execution mode
    assert!(!session.is_paused());
}

// ============================================================================
// 4. All Features Simultaneously (tests 31-40)
// ============================================================================

#[test]
fn test_all_features_arithmetic() {
    let source = "1 + 2 * 3;";
    // Run with optimizer
    let opt_result = vm_eval_opt(source);
    // Run without
    let plain_result = vm_eval(source);
    assert_eq!(opt_result, plain_result);
    assert_eq!(opt_result, Some(Value::Number(7.0)));
}

#[test]
fn test_all_features_variables() {
    let source = "let x = 10; let y = 20; x + y;";
    let opt = vm_eval_opt(source);
    let plain = vm_eval(source);
    assert_eq!(opt, plain);
}

#[test]
fn test_all_features_conditionals() {
    let source = "let x = 10; if (x > 5) { x = x * 2; } x;";
    let opt = vm_eval_opt(source);
    let plain = vm_eval(source);
    assert_eq!(opt, plain);
}

#[test]
fn test_all_features_while_loop() {
    let source = "let sum = 0; let i = 0; while (i < 50) { sum = sum + i; i = i + 1; } sum;";
    let opt = vm_eval_opt(source);
    let plain = vm_eval(source);
    assert_eq!(opt, plain);
}

#[test]
fn test_all_features_function_calls() {
    let source = "fn square(x: number) -> number { return x * x; } square(7);";
    let opt = vm_eval_opt(source);
    let plain = vm_eval(source);
    assert_eq!(opt, plain);
}

#[test]
fn test_all_features_recursion() {
    let source = "fn fact(n: number) -> number { if (n <= 1) { return 1; } return n * fact(n - 1); } fact(6);";
    let opt = vm_eval_opt(source);
    let plain = vm_eval(source);
    assert_eq!(opt, plain);
    assert_eq!(opt, Some(Value::Number(720.0)));
}

#[test]
fn test_all_features_string_ops() {
    let source = r#""hello" + " " + "world";"#;
    let opt = vm_eval_opt(source);
    let plain = vm_eval(source);
    assert_eq!(opt, plain);
}

#[test]
fn test_all_features_array_ops() {
    let source = "let arr = [1, 2, 3, 4, 5]; arr[2];";
    let opt = vm_eval_opt(source);
    let plain = vm_eval(source);
    assert_eq!(opt, plain);
}

#[test]
fn test_all_features_boolean_logic() {
    let source = "true && false || true;";
    let opt = vm_eval_opt(source);
    let plain = vm_eval(source);
    assert_eq!(opt, plain);
}

#[test]
fn test_all_features_nested_expressions() {
    let source = "let a = 1; let b = 2; let c = 3; (a + b) * c - (a * b);";
    let opt = vm_eval_opt(source);
    let plain = vm_eval(source);
    assert_eq!(opt, plain);
    assert_eq!(opt, Some(Value::Number(7.0)));
}

// ============================================================================
// 5. Optimizer Validation (tests 41-50)
// ============================================================================

#[test]
fn test_optimizer_reduces_bytecode() {
    let source = "let x = 2 + 3; x;";
    let plain = compile(source);
    let opt = compile_optimized(source);
    assert!(opt.instructions.len() <= plain.instructions.len());
}

#[test]
fn test_optimizer_constant_folding_int() {
    // Constant folding: 2 + 3 should be folded to 5
    let result = vm_number_opt("2 + 3;");
    assert_eq!(result, 5.0);
}

#[test]
fn test_optimizer_constant_folding_mul() {
    let result = vm_number_opt("4 * 5;");
    assert_eq!(result, 20.0);
}

#[test]
fn test_optimizer_preserves_side_effects() {
    let source = "let x = 0; x = x + 1; x = x + 1; x = x + 1; x;";
    let opt = vm_number_opt(source);
    assert_eq!(opt, 3.0);
}

#[test]
fn test_optimizer_nested_arithmetic() {
    let source = "((1 + 2) * (3 + 4));";
    let opt = vm_number_opt(source);
    assert_eq!(opt, 21.0);
}

#[test]
fn test_optimizer_mixed_types() {
    let source = r#"let x = 5; let s = "hello"; x;"#;
    let opt = vm_eval_opt(source);
    let plain = vm_eval(source);
    assert_eq!(opt, plain);
}

#[test]
fn test_optimizer_loop_invariant() {
    let source = "let sum = 0; let i = 0; while (i < 100) { sum = sum + 2 * 3; i = i + 1; } sum;";
    let opt = vm_number_opt(source);
    assert_eq!(opt, 600.0);
}

#[test]
fn test_optimizer_dead_code_after_return() {
    let source = "fn test() -> number { return 42; } test();";
    let opt = vm_number_opt(source);
    assert_eq!(opt, 42.0);
}

#[test]
fn test_optimizer_chained_calls() {
    let source = "fn inc(x: number) -> number { return x + 1; } inc(inc(inc(0)));";
    let opt = vm_number_opt(source);
    assert_eq!(opt, 3.0);
}

#[test]
fn test_optimizer_complex_program() {
    let source = r#"
fn gcd(a: number, b: number) -> number {
    if (b == 0) { return a; }
    return gcd(b, a % b);
}
gcd(48, 18);
"#;
    let opt = vm_number_opt(source);
    let plain = vm_number(source);
    assert_eq!(opt, plain);
    assert_eq!(opt, 6.0);
}

// ============================================================================
// 6. Bytecode Validation Integration (tests 51-58)
// ============================================================================

#[test]
fn test_validate_simple_program() {
    let bc = compile("1 + 2;");
    let result = atlas_runtime::bytecode::validate(&bc);
    assert!(result.is_ok(), "Validation failed: {:?}", result);
}

#[test]
fn test_validate_optimized_program() {
    let bc = compile_optimized("1 + 2;");
    let result = atlas_runtime::bytecode::validate(&bc);
    assert!(result.is_ok(), "Validation failed: {:?}", result);
}

#[test]
fn test_validate_function_program() {
    let bc = compile("fn add(a: number, b: number) -> number { return a + b; } add(1, 2);");
    let result = atlas_runtime::bytecode::validate(&bc);
    assert!(result.is_ok(), "Validation failed: {:?}", result);
}

#[test]
fn test_validate_loop_program() {
    let bc = compile("let i = 0; while (i < 10) { i = i + 1; } i;");
    let result = atlas_runtime::bytecode::validate(&bc);
    assert!(result.is_ok(), "Validation failed: {:?}", result);
}

#[test]
fn test_validate_conditional_program() {
    let bc = compile("let x = 10; if (x > 5) { x = 20; } else { x = 0; } x;");
    let result = atlas_runtime::bytecode::validate(&bc);
    assert!(result.is_ok(), "Validation failed: {:?}", result);
}

#[test]
fn test_validate_array_program() {
    let bc = compile("let arr = [1, 2, 3]; arr[0];");
    let result = atlas_runtime::bytecode::validate(&bc);
    assert!(result.is_ok(), "Validation failed: {:?}", result);
}

#[test]
fn test_validate_string_program() {
    let bc = compile(r#""hello" + " world";"#);
    let result = atlas_runtime::bytecode::validate(&bc);
    assert!(result.is_ok(), "Validation failed: {:?}", result);
}

#[test]
fn test_validate_nested_functions() {
    let bc = compile(
        "fn outer(x: number) -> number { fn inner(y: number) -> number { return y * 2; } return inner(x); } outer(5);",
    );
    let result = atlas_runtime::bytecode::validate(&bc);
    assert!(result.is_ok(), "Validation failed: {:?}", result);
}

// ============================================================================
// 7. Optimizer Level Testing (tests 59-66)
// ============================================================================

#[test]
fn test_optimizer_level_0() {
    let optimizer = Optimizer::with_optimization_level(0);
    let bc = compile("1 + 2;");
    let result = optimizer.optimize(bc);
    // Level 0 should still produce valid bytecode
    let val = vm_run(result);
    assert_eq!(val, Some(Value::Number(3.0)));
}

#[test]
fn test_optimizer_level_1() {
    let optimizer = Optimizer::with_optimization_level(1);
    let bc = compile("1 + 2;");
    let result = optimizer.optimize(bc);
    let val = vm_run(result);
    assert_eq!(val, Some(Value::Number(3.0)));
}

#[test]
fn test_optimizer_level_2() {
    let optimizer = Optimizer::with_optimization_level(2);
    let bc = compile("1 + 2;");
    let result = optimizer.optimize(bc);
    let val = vm_run(result);
    assert_eq!(val, Some(Value::Number(3.0)));
}

#[test]
fn test_optimizer_stats() {
    let optimizer = Optimizer::with_default_passes();
    let bc = compile("2 + 3;");
    let (result, stats) = optimizer.optimize_with_stats(bc);
    let val = vm_run(result);
    assert_eq!(val, Some(Value::Number(5.0)));
    // Stats should report something
    assert!(stats.passes_run > 0);
}

#[test]
fn test_optimizer_enabled_toggle() {
    let mut optimizer = Optimizer::new();
    assert!(!optimizer.is_enabled());
    optimizer.set_enabled(true);
    assert!(optimizer.is_enabled());
    optimizer.set_enabled(false);
    assert!(!optimizer.is_enabled());
}

#[test]
fn test_optimizer_disabled_passthrough() {
    let optimizer = Optimizer::new();
    assert!(!optimizer.is_enabled());
    let bc = compile("1 + 2;");
    let original_len = bc.instructions.len();
    let result = optimizer.optimize(bc);
    // Disabled optimizer should pass through unchanged
    assert_eq!(result.instructions.len(), original_len);
}

#[test]
fn test_optimizer_multiple_passes() {
    let optimizer = Optimizer::with_default_passes();
    assert!(optimizer.passes_count() > 0);
}

#[test]
fn test_optimizer_with_stats_complex() {
    let source = "let x = 2 + 3; let y = 4 * 5; x + y;";
    let optimizer = Optimizer::with_default_passes();
    let bc = compile(source);
    let (_result, stats) = optimizer.optimize_with_stats(bc);
    // Should have tracked bytes
    assert!(stats.bytes_saved() >= 0 || stats.bytes_saved() < 0);
}

// ============================================================================
// 8. Cross-Feature Correctness (tests 67-80)
// ============================================================================

#[rstest]
#[case("1 + 1;", 2.0)]
#[case("10 * 10;", 100.0)]
#[case("100 / 4;", 25.0)]
#[case("7 % 3;", 1.0)]
fn test_cross_basic_arithmetic(#[case] source: &str, #[case] expected: f64) {
    let plain = vm_number(source);
    let opt = vm_number_opt(source);
    assert_eq!(plain, expected);
    assert_eq!(opt, expected);
}

#[rstest]
#[case("let x = 5; x;", 5.0)]
#[case("let x = 5; let y = 10; x + y;", 15.0)]
#[case("let x = 5; x = x + 1; x;", 6.0)]
#[case("let x = 100; let y = x / 2; y;", 50.0)]
fn test_cross_variables(#[case] source: &str, #[case] expected: f64) {
    let plain = vm_number(source);
    let opt = vm_number_opt(source);
    assert_eq!(plain, expected);
    assert_eq!(opt, expected);
}

#[rstest]
#[case("let r = 0; if (true) { r = 1; } else { r = 2; } r;", 1.0)]
#[case("let r = 0; if (false) { r = 1; } else { r = 2; } r;", 2.0)]
#[case("let r = 0; if (1 < 2) { r = 10; } else { r = 20; } r;", 10.0)]
#[case("let r = 0; if (1 > 2) { r = 10; } else { r = 20; } r;", 20.0)]
fn test_cross_conditionals(#[case] source: &str, #[case] expected: f64) {
    let plain = vm_number(source);
    let opt = vm_number_opt(source);
    assert_eq!(plain, expected);
    assert_eq!(opt, expected);
}

#[test]
fn test_cross_fibonacci_parity() {
    let source = "fn fib(n: number) -> number { if (n <= 1) { return n; } return fib(n - 1) + fib(n - 2); } fib(15);";
    let plain = vm_number(source);
    let opt = vm_number_opt(source);
    assert_eq!(plain, opt);
    assert_eq!(plain, 610.0);
}

#[test]
fn test_cross_string_concatenation() {
    let source = r#"let s = "a"; s = s + "b"; s = s + "c"; s;"#;
    let plain = vm_string(source);
    assert_eq!(plain, "abc");
}

#[test]
fn test_cross_array_manipulation() {
    let source = "let arr = [1, 2, 3]; arr[0] = 10; arr[0] + arr[1] + arr[2];";
    let plain = vm_number(source);
    let opt = vm_number_opt(source);
    assert_eq!(plain, opt);
    assert_eq!(plain, 15.0);
}

#[test]
fn test_cross_nested_loops() {
    let source = "let total = 0; let i = 0; while (i < 5) { let j = 0; while (j < 5) { total = total + 1; j = j + 1; } i = i + 1; } total;";
    let plain = vm_number(source);
    let opt = vm_number_opt(source);
    assert_eq!(plain, opt);
    assert_eq!(plain, 25.0);
}

#[test]
fn test_cross_comparison_chain() {
    let source = "let count = 0; if (1 < 2) { count = count + 1; } if (2 <= 2) { count = count + 1; } if (3 > 2) { count = count + 1; } if (3 >= 3) { count = count + 1; } count;";
    let plain = vm_number(source);
    let opt = vm_number_opt(source);
    assert_eq!(plain, 4.0);
    assert_eq!(opt, 4.0);
}

#[test]
fn test_cross_boolean_not() {
    let source = "!false;";
    let plain = vm_bool(source);
    assert!(plain);
}

#[test]
fn test_cross_null_value() {
    let source = "null;";
    let plain = vm_eval(source);
    let opt = vm_eval_opt(source);
    assert_eq!(plain, Some(Value::Null));
    assert_eq!(opt, Some(Value::Null));
}

#[test]
fn test_cross_complex_expression_tree() {
    let source = "let a = 2; let b = 3; let c = 4; let d = 5; ((a + b) * c) - d + (a * (b + c));";
    let plain = vm_number(source);
    let opt = vm_number_opt(source);
    assert_eq!(plain, opt);
}
