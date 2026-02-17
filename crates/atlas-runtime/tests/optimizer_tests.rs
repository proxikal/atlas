//! Optimizer unit tests
//!
//! Tests for the bytecode optimizer passes individually and in combination.

use atlas_runtime::{
    bytecode::{Bytecode, Opcode},
    compiler::Compiler,
    lexer::Lexer,
    optimizer::{
        ConstantFoldingPass, DeadCodeEliminationPass, OptimizationPass, OptimizationStats,
        Optimizer, PeepholePass,
    },
    parser::Parser,
    security::SecurityContext,
    span::Span,
    value::Value,
    vm::VM,
};

// ============================================================================
// Test helpers
// ============================================================================

fn compile(source: &str) -> Bytecode {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, lex_diags) = lexer.tokenize();
    assert!(lex_diags.is_empty(), "Lexer errors: {:?}", lex_diags);
    let mut parser = Parser::new(tokens);
    let (program, parse_diags) = parser.parse();
    assert!(parse_diags.is_empty(), "Parser errors: {:?}", parse_diags);
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

fn run(bc: Bytecode) -> Option<Value> {
    let security = SecurityContext::allow_all();
    let mut vm = VM::new(bc);
    vm.run(&security).unwrap_or(None)
}

fn assert_same_result(source: &str) {
    let bc_orig = compile(source);
    let bc_opt = compile_optimized(source);
    let orig = run(bc_orig);
    let opt = run(bc_opt);
    assert_eq!(orig, opt, "Optimized result differs for: {}", source);
}

fn run_cf(bc: Bytecode) -> (Bytecode, OptimizationStats) {
    ConstantFoldingPass.optimize(bc)
}

fn run_dce(bc: Bytecode) -> (Bytecode, OptimizationStats) {
    DeadCodeEliminationPass.optimize(bc)
}

fn run_peep(bc: Bytecode) -> (Bytecode, OptimizationStats) {
    PeepholePass.optimize(bc)
}

fn run_all(bc: Bytecode) -> (Bytecode, OptimizationStats) {
    Optimizer::with_default_passes().optimize_with_stats(bc)
}

// ============================================================================
// Optimizer configuration tests
// ============================================================================

#[test]
fn test_optimizer_new_disabled() {
    let opt = Optimizer::new();
    assert!(!opt.is_enabled());
    assert_eq!(opt.passes_count(), 0);
}

#[test]
fn test_optimizer_with_default_passes_has_three() {
    let opt = Optimizer::with_default_passes();
    assert!(opt.is_enabled());
    assert_eq!(opt.passes_count(), 3);
}

#[test]
fn test_optimizer_passthrough_when_disabled() {
    let opt = Optimizer::new();
    let mut bc = Bytecode::new();
    bc.emit(Opcode::Null, Span::dummy());
    bc.emit(Opcode::Halt, Span::dummy());
    let size = bc.instructions.len();
    let result = opt.optimize(bc);
    assert_eq!(result.instructions.len(), size);
}

#[test]
fn test_optimizer_with_stats_disabled() {
    let opt = Optimizer::new();
    let mut bc = Bytecode::new();
    bc.emit(Opcode::Halt, Span::dummy());
    let (_, stats) = opt.optimize_with_stats(bc);
    assert_eq!(stats.total_optimizations(), 0);
    assert_eq!(stats.bytes_saved(), 0);
}

#[test]
fn test_optimizer_level_0_is_disabled() {
    let opt = Optimizer::with_optimization_level(0);
    assert!(!opt.is_enabled());
}

#[test]
fn test_optimizer_level_1_peephole_only() {
    let opt = Optimizer::with_optimization_level(1);
    assert!(opt.is_enabled());
    assert_eq!(opt.passes_count(), 1);
}

#[test]
fn test_optimizer_level_2() {
    let opt = Optimizer::with_optimization_level(2);
    assert!(opt.is_enabled());
    assert_eq!(opt.passes_count(), 2);
}

#[test]
fn test_optimizer_level_3_all_passes() {
    let opt = Optimizer::with_optimization_level(3);
    assert_eq!(opt.passes_count(), 3);
}

// ============================================================================
// Constant folding tests
// ============================================================================

#[test]
fn test_cf_add() {
    let mut bc = Bytecode::new();
    let a = bc.add_constant(Value::Number(2.0));
    let b = bc.add_constant(Value::Number(3.0));
    bc.emit(Opcode::Constant, Span::dummy());
    bc.emit_u16(a);
    bc.emit(Opcode::Constant, Span::dummy());
    bc.emit_u16(b);
    bc.emit(Opcode::Add, Span::dummy());
    bc.emit(Opcode::Halt, Span::dummy());

    let (result, stats) = run_cf(bc);
    assert_eq!(stats.constants_folded, 1);
    assert!(result.instructions.len() < 8);
    assert_eq!(*result.constants.last().unwrap(), Value::Number(5.0));
}

#[test]
fn test_cf_sub() {
    let mut bc = Bytecode::new();
    let a = bc.add_constant(Value::Number(10.0));
    let b = bc.add_constant(Value::Number(4.0));
    bc.emit(Opcode::Constant, Span::dummy());
    bc.emit_u16(a);
    bc.emit(Opcode::Constant, Span::dummy());
    bc.emit_u16(b);
    bc.emit(Opcode::Sub, Span::dummy());
    bc.emit(Opcode::Halt, Span::dummy());

    let (result, stats) = run_cf(bc);
    assert_eq!(stats.constants_folded, 1);
    assert_eq!(*result.constants.last().unwrap(), Value::Number(6.0));
}

#[test]
fn test_cf_mul() {
    let mut bc = Bytecode::new();
    let a = bc.add_constant(Value::Number(4.0));
    let b = bc.add_constant(Value::Number(7.0));
    bc.emit(Opcode::Constant, Span::dummy());
    bc.emit_u16(a);
    bc.emit(Opcode::Constant, Span::dummy());
    bc.emit_u16(b);
    bc.emit(Opcode::Mul, Span::dummy());
    bc.emit(Opcode::Halt, Span::dummy());

    let (result, stats) = run_cf(bc);
    assert_eq!(stats.constants_folded, 1);
    assert_eq!(*result.constants.last().unwrap(), Value::Number(28.0));
}

#[test]
fn test_cf_div() {
    let mut bc = Bytecode::new();
    let a = bc.add_constant(Value::Number(12.0));
    let b = bc.add_constant(Value::Number(4.0));
    bc.emit(Opcode::Constant, Span::dummy());
    bc.emit_u16(a);
    bc.emit(Opcode::Constant, Span::dummy());
    bc.emit_u16(b);
    bc.emit(Opcode::Div, Span::dummy());
    bc.emit(Opcode::Halt, Span::dummy());

    let (result, stats) = run_cf(bc);
    assert_eq!(stats.constants_folded, 1);
    assert_eq!(*result.constants.last().unwrap(), Value::Number(3.0));
}

#[test]
fn test_cf_mod() {
    let mut bc = Bytecode::new();
    let a = bc.add_constant(Value::Number(17.0));
    let b = bc.add_constant(Value::Number(5.0));
    bc.emit(Opcode::Constant, Span::dummy());
    bc.emit_u16(a);
    bc.emit(Opcode::Constant, Span::dummy());
    bc.emit_u16(b);
    bc.emit(Opcode::Mod, Span::dummy());
    bc.emit(Opcode::Halt, Span::dummy());

    let (result, stats) = run_cf(bc);
    assert_eq!(stats.constants_folded, 1);
    assert_eq!(*result.constants.last().unwrap(), Value::Number(2.0));
}

#[test]
fn test_cf_no_fold_div_zero() {
    let mut bc = Bytecode::new();
    let a = bc.add_constant(Value::Number(5.0));
    let b = bc.add_constant(Value::Number(0.0));
    bc.emit(Opcode::Constant, Span::dummy());
    bc.emit_u16(a);
    bc.emit(Opcode::Constant, Span::dummy());
    bc.emit_u16(b);
    bc.emit(Opcode::Div, Span::dummy());
    bc.emit(Opcode::Halt, Span::dummy());

    let size = bc.instructions.len();
    let (result, stats) = run_cf(bc);
    assert_eq!(stats.constants_folded, 0);
    assert_eq!(result.instructions.len(), size);
}

#[test]
fn test_cf_negate() {
    let mut bc = Bytecode::new();
    let a = bc.add_constant(Value::Number(42.0));
    bc.emit(Opcode::Constant, Span::dummy());
    bc.emit_u16(a);
    bc.emit(Opcode::Negate, Span::dummy());
    bc.emit(Opcode::Halt, Span::dummy());

    let (result, stats) = run_cf(bc);
    assert_eq!(stats.constants_folded, 1);
    assert_eq!(*result.constants.last().unwrap(), Value::Number(-42.0));
}

#[test]
fn test_cf_comparison_less() {
    let mut bc = Bytecode::new();
    let a = bc.add_constant(Value::Number(3.0));
    let b = bc.add_constant(Value::Number(5.0));
    bc.emit(Opcode::Constant, Span::dummy());
    bc.emit_u16(a);
    bc.emit(Opcode::Constant, Span::dummy());
    bc.emit_u16(b);
    bc.emit(Opcode::Less, Span::dummy());
    bc.emit(Opcode::Halt, Span::dummy());

    let (result, stats) = run_cf(bc);
    assert_eq!(stats.constants_folded, 1);
    assert_eq!(*result.constants.last().unwrap(), Value::Bool(true));
}

#[test]
fn test_cf_comparison_greater() {
    let mut bc = Bytecode::new();
    let a = bc.add_constant(Value::Number(5.0));
    let b = bc.add_constant(Value::Number(3.0));
    bc.emit(Opcode::Constant, Span::dummy());
    bc.emit_u16(a);
    bc.emit(Opcode::Constant, Span::dummy());
    bc.emit_u16(b);
    bc.emit(Opcode::Greater, Span::dummy());
    bc.emit(Opcode::Halt, Span::dummy());

    let (result, stats) = run_cf(bc);
    assert_eq!(stats.constants_folded, 1);
    assert_eq!(*result.constants.last().unwrap(), Value::Bool(true));
}

#[test]
fn test_cf_true_not() {
    let mut bc = Bytecode::new();
    bc.emit(Opcode::True, Span::dummy());
    bc.emit(Opcode::Not, Span::dummy());
    bc.emit(Opcode::Halt, Span::dummy());

    let (result, stats) = run_cf(bc);
    assert_eq!(stats.constants_folded, 1);
    assert!(result.instructions.contains(&(Opcode::False as u8)));
    assert!(!result.instructions.contains(&(Opcode::True as u8)));
}

#[test]
fn test_cf_false_not() {
    let mut bc = Bytecode::new();
    bc.emit(Opcode::False, Span::dummy());
    bc.emit(Opcode::Not, Span::dummy());
    bc.emit(Opcode::Halt, Span::dummy());

    let (result, stats) = run_cf(bc);
    assert_eq!(stats.constants_folded, 1);
    assert!(result.instructions.contains(&(Opcode::True as u8)));
}

#[test]
fn test_cf_nested_arithmetic() {
    // (2 + 3) * 4 should fold to 20
    let (result, stats) = {
        let bc = compile("(2 + 3) * 4;");
        run_cf(bc)
    };
    assert!(stats.constants_folded >= 2);
    let has_20 = result
        .constants
        .iter()
        .any(|c| matches!(c, Value::Number(n) if (n - 20.0).abs() < 1e-9));
    assert!(has_20);
}

#[test]
fn test_cf_size_reduction() {
    let bc_orig = compile("2 + 3;");
    let orig_size = bc_orig.instructions.len();
    let (bc_opt, stats) = run_cf(bc_orig);
    assert!(stats.constants_folded > 0);
    assert!(bc_opt.instructions.len() < orig_size);
}

#[test]
fn test_cf_preserves_semantics() {
    assert_same_result("2 + 3;");
}

#[test]
fn test_cf_preserves_semantics_complex() {
    assert_same_result("10 * 2 + 5 - 3;");
}

#[test]
fn test_cf_preserves_semantics_comparison() {
    assert_same_result("let x = 5 > 3;");
}

// ============================================================================
// Dead code elimination tests
// ============================================================================

#[test]
fn test_dce_removes_after_halt() {
    let mut bc = Bytecode::new();
    bc.emit(Opcode::Halt, Span::dummy());
    bc.emit(Opcode::Null, Span::dummy()); // dead
    bc.emit(Opcode::Pop, Span::dummy()); // dead

    let (result, stats) = run_dce(bc);
    assert_eq!(stats.dead_instructions_removed, 2);
    assert_eq!(result.instructions.len(), 1);
}

#[test]
fn test_dce_removes_after_return_in_fn() {
    let source = r#"
        fn add(a: number, b: number) -> number {
            return a + b;
        }
        add(2, 3);
    "#;
    let bc = compile(source);
    let size_before = bc.instructions.len();
    let (result, stats) = run_dce(bc);
    assert!(
        stats.dead_instructions_removed >= 2,
        "Should remove implicit Null+Return"
    );
    assert!(result.instructions.len() < size_before);
}

#[test]
fn test_dce_all_reachable_unchanged() {
    let mut bc = Bytecode::new();
    bc.emit(Opcode::Null, Span::dummy());
    bc.emit(Opcode::Pop, Span::dummy());
    bc.emit(Opcode::Halt, Span::dummy());

    let size = bc.instructions.len();
    let (result, stats) = run_dce(bc);
    assert_eq!(stats.dead_instructions_removed, 0);
    assert_eq!(result.instructions.len(), size);
}

#[test]
fn test_dce_preserves_while_loop() {
    assert_same_result("let x = 0; while (x < 5) { x = x + 1; }");
}

#[test]
fn test_dce_preserves_if_else() {
    assert_same_result("let x = 5; if (x > 3) { x = 1; } else { x = 2; }");
}

#[test]
fn test_dce_preserves_function_call() {
    assert_same_result("fn sq(x: number) -> number { return x * x; } let r = sq(5);");
}

#[test]
fn test_dce_size_reduction() {
    let mut bc = Bytecode::new();
    bc.emit(Opcode::Halt, Span::dummy());
    for _ in 0..20 {
        bc.emit(Opcode::Null, Span::dummy()); // all dead
    }
    let (result, stats) = run_dce(bc);
    assert_eq!(stats.dead_instructions_removed, 20);
    assert!(stats.bytes_saved() >= 20);
    assert_eq!(result.instructions.len(), 1);
}

#[test]
fn test_dce_empty_unchanged() {
    let bc = Bytecode::new();
    let (result, stats) = run_dce(bc);
    assert_eq!(stats.dead_instructions_removed, 0);
    assert!(result.instructions.is_empty());
}

// ============================================================================
// Peephole tests
// ============================================================================

#[test]
fn test_peep_dup_pop_eliminated() {
    let mut bc = Bytecode::new();
    bc.emit(Opcode::Null, Span::dummy());
    bc.emit(Opcode::Dup, Span::dummy());
    bc.emit(Opcode::Pop, Span::dummy());
    bc.emit(Opcode::Halt, Span::dummy());

    let (result, stats) = run_peep(bc);
    assert_eq!(stats.peephole_patterns_applied, 1);
    assert!(!result.instructions.contains(&(Opcode::Dup as u8)));
    assert!(!result.instructions.contains(&(Opcode::Pop as u8)));
}

#[test]
fn test_peep_not_not_eliminated() {
    let mut bc = Bytecode::new();
    bc.emit(Opcode::True, Span::dummy());
    bc.emit(Opcode::Not, Span::dummy());
    bc.emit(Opcode::Not, Span::dummy());
    bc.emit(Opcode::Halt, Span::dummy());

    let (result, stats) = run_peep(bc);
    assert_eq!(stats.peephole_patterns_applied, 1);
    let not_count = result
        .instructions
        .iter()
        .filter(|&&b| b == Opcode::Not as u8)
        .count();
    assert_eq!(not_count, 0);
}

#[test]
fn test_peep_jump_zero_eliminated() {
    let mut bc = Bytecode::new();
    bc.emit(Opcode::Jump, Span::dummy());
    bc.emit_i16(0); // no-op jump
    bc.emit(Opcode::Halt, Span::dummy());

    let (result, stats) = run_peep(bc);
    assert_eq!(stats.peephole_patterns_applied, 1);
    assert!(!result.instructions.contains(&(Opcode::Jump as u8)));
}

#[test]
fn test_peep_preserves_if_else() {
    assert_same_result("let x = 3; if (x > 2) { x = 10; }");
}

#[test]
fn test_peep_preserves_while() {
    assert_same_result("let i = 0; while (i < 3) { i = i + 1; }");
}

#[test]
fn test_peep_size_reduced_with_dup_pop() {
    let bc = compile("let x = 5;");
    let size_before = bc.instructions.len();
    let (result, _) = run_peep(bc);
    // Either same or smaller (if any dup-pop patterns were there)
    assert!(result.instructions.len() <= size_before);
}

// ============================================================================
// Full pipeline tests
// ============================================================================

#[test]
fn test_full_pipeline_size_reduced() {
    let bc = compile("let x = 2 + 3;");
    let size_before = bc.instructions.len();
    let (optimized, stats) = run_all(bc);
    assert!(
        optimized.instructions.len() <= size_before,
        "Optimized should be smaller or equal"
    );
    assert!(stats.total_optimizations() > 0);
}

#[test]
fn test_full_pipeline_semantics_simple() {
    assert_same_result("2 + 3;");
}

#[test]
fn test_full_pipeline_semantics_variable() {
    assert_same_result("let x = 5; let y = x + 3;");
}

#[test]
fn test_full_pipeline_semantics_function() {
    assert_same_result("fn double(x: number) -> number { return x * 2; } double(21);");
}

#[test]
fn test_full_pipeline_semantics_if() {
    assert_same_result("let x = 10; if (x > 5) { x = x - 1; }");
}

#[test]
fn test_full_pipeline_semantics_while() {
    assert_same_result("let sum = 0; let i = 0; while (i < 5) { sum = sum + i; i = i + 1; }");
}

#[test]
fn test_full_pipeline_semantics_nested_functions() {
    assert_same_result(
        r#"
        fn add(a: number, b: number) -> number { return a + b; }
        fn mul(a: number, b: number) -> number { return a * b; }
        let result = add(mul(2, 3), 4);
        "#,
    );
}

#[test]
fn test_full_pipeline_semantics_comparison_chain() {
    assert_same_result("let x = 5; let y = x > 3 && x < 10;");
}

#[test]
fn test_full_pipeline_semantics_array() {
    assert_same_result("let arr = [1, 2, 3]; arr[1];");
}

#[test]
fn test_full_pipeline_stats_populated() {
    let bc = compile("2 + 3;");
    let (_, stats) = run_all(bc);
    assert!(stats.bytecode_size_before > 0);
    assert!(stats.bytecode_size_after > 0);
    assert!(stats.passes_run > 0);
}

// ============================================================================
// Bytecode validity tests (optimized bytecode passes validator)
// ============================================================================

#[test]
fn test_optimized_bytecode_valid_simple() {
    let bc = compile("let x = 2 + 3;");
    let (optimized, _) = run_all(bc);
    atlas_runtime::bytecode::validate(&optimized)
        .expect("Optimized bytecode should be valid (simple)");
}

#[test]
fn test_optimized_bytecode_valid_function() {
    let bc = compile("fn add(a: number, b: number) -> number { return a + b; } add(1, 2);");
    let (optimized, _) = run_all(bc);
    atlas_runtime::bytecode::validate(&optimized)
        .expect("Optimized bytecode should be valid (function)");
}

#[test]
fn test_optimized_bytecode_valid_if_else() {
    let bc = compile("if (true) { 1; } else { 2; }");
    let (optimized, _) = run_all(bc);
    atlas_runtime::bytecode::validate(&optimized)
        .expect("Optimized bytecode should be valid (if-else)");
}

#[test]
fn test_optimized_bytecode_valid_while() {
    let bc = compile("let i = 0; while (i < 3) { i = i + 1; }");
    let (optimized, _) = run_all(bc);
    atlas_runtime::bytecode::validate(&optimized)
        .expect("Optimized bytecode should be valid (while)");
}

#[test]
fn test_cf_preserves_validator() {
    let bc = compile("5 * 6 - 2;");
    let (optimized, _) = run_cf(bc);
    atlas_runtime::bytecode::validate(&optimized).expect("CF should produce valid bytecode");
}

#[test]
fn test_dce_preserves_validator() {
    let bc = compile("fn f(x: number) -> number { return x + 1; } f(5);");
    let (optimized, _) = run_dce(bc);
    atlas_runtime::bytecode::validate(&optimized).expect("DCE should produce valid bytecode");
}

#[test]
fn test_peep_preserves_validator() {
    let bc = compile("let x = 5; if (x > 3) { x = 1; }");
    let (optimized, _) = run_peep(bc);
    atlas_runtime::bytecode::validate(&optimized).expect("Peephole should produce valid bytecode");
}

// ============================================================================
// Compiler integration tests
// ============================================================================

#[test]
fn test_compiler_with_optimization_enabled() {
    let compiler = Compiler::with_optimization();
    let _ = compiler; // just verify it constructs
}

#[test]
fn test_compiler_optimized_output_valid() {
    let bc = compile_optimized("let x = 2 + 3; let y = x * 4;");
    atlas_runtime::bytecode::validate(&bc).expect("Compiler-optimized bytecode should be valid");
}

#[test]
fn test_compiler_optimized_smaller() {
    let bc_raw = compile("let x = 2 + 3;");
    let bc_opt = compile_optimized("let x = 2 + 3;");
    assert!(
        bc_opt.instructions.len() <= bc_raw.instructions.len(),
        "Optimized compiler should produce smaller code"
    );
}

#[test]
fn test_compiler_optimized_correct_result() {
    let bc_raw = compile("let x = 2 + 3;");
    let bc_opt = compile_optimized("let x = 2 + 3;");
    let result_raw = run(bc_raw);
    let result_opt = run(bc_opt);
    assert_eq!(result_raw, result_opt);
}

// ============================================================================
// Edge cases
// ============================================================================

#[test]
fn test_empty_program() {
    let bc = compile("");
    let (optimized, _) = run_all(bc);
    atlas_runtime::bytecode::validate(&optimized)
        .expect("Empty program optimized bytecode should be valid");
}

#[test]
fn test_single_number() {
    assert_same_result("42;");
}

#[test]
fn test_single_string() {
    assert_same_result("\"hello\";");
}

#[test]
fn test_boolean_operations() {
    assert_same_result("true && false;");
}

#[test]
fn test_already_optimal_unchanged() {
    // Single constant + halt is already optimal
    let mut bc = Bytecode::new();
    let idx = bc.add_constant(Value::Number(42.0));
    bc.emit(Opcode::Constant, Span::dummy());
    bc.emit_u16(idx);
    bc.emit(Opcode::Pop, Span::dummy());
    bc.emit(Opcode::Halt, Span::dummy());
    let size = bc.instructions.len();
    let (result, stats) = run_all(bc);
    // No optimizations possible (constant is not combined with anything)
    assert_eq!(stats.constants_folded, 0);
    assert_eq!(result.instructions.len(), size);
}
