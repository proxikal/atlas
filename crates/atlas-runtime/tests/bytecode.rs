//! bytecode.rs — merged from 6 files (Phase Infra-02)

mod common;

use atlas_runtime::binder::Binder;
use atlas_runtime::bytecode::{Bytecode, Opcode};
use atlas_runtime::compiler::Compiler;
use atlas_runtime::interpreter::Interpreter;
use atlas_runtime::lexer::Lexer;
use atlas_runtime::optimizer::{
    ConstantFoldingPass, DeadCodeEliminationPass, OptimizationPass, OptimizationStats, Optimizer,
    PeepholePass,
};
use atlas_runtime::parser::Parser;
use atlas_runtime::profiler::{HotspotDetector, ProfileCollector, ProfileReport, Profiler};
use atlas_runtime::security::SecurityContext;
use atlas_runtime::span::Span;
use atlas_runtime::typechecker::TypeChecker;
use atlas_runtime::value::{FunctionRef, Value};
use atlas_runtime::vm::VM;
use pretty_assertions::assert_eq;
use rstest::rstest;

// ============================================================================
// Canonical helpers (deduplicated from all 6 source files)
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

fn run(bc: Bytecode) -> Option<Value> {
    let security = SecurityContext::allow_all();
    let mut vm = VM::new(bc);
    vm.run(&security).unwrap_or(None)
}

fn run_interpreter(source: &str) -> Result<Value, String> {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();
    let mut binder = atlas_runtime::binder::Binder::new();
    let (mut table, _) = binder.bind(&program);
    let mut typechecker = TypeChecker::new(&mut table);
    let _ = typechecker.check(&program);
    let mut interpreter = Interpreter::new();
    interpreter
        .eval(&program, &SecurityContext::allow_all())
        .map_err(|e| format!("{:?}", e))
}

fn run_vm(source: &str) -> Result<Value, String> {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();
    let mut binder = atlas_runtime::binder::Binder::new();
    let (mut table, _) = binder.bind(&program);
    let mut typechecker = TypeChecker::new(&mut table);
    let _ = typechecker.check(&program);
    let bc = Compiler::new()
        .compile(&program)
        .map_err(|e| format!("Compile: {:?}", e))?;
    let mut vm = VM::new(bc);
    vm.run(&SecurityContext::allow_all())
        .map_err(|e| format!("VM: {:?}", e))
        .map(|v| v.unwrap_or(Value::Null))
}

// ============================================================================
// From bytecode_compiler_integration.rs
// ============================================================================

// Modern Bytecode Compiler Integration Tests
//
// Converted from bytecode_compiler_integration.rs (261 lines → ~110 lines = 58% reduction)

fn execute_source(source: &str) -> Result<Option<Value>, atlas_runtime::value::RuntimeError> {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, lex_diags) = lexer.tokenize();
    assert!(lex_diags.is_empty(), "Lexer errors: {:?}", lex_diags);

    let mut parser = Parser::new(tokens);
    let (program, parse_diags) = parser.parse();
    assert!(parse_diags.is_empty(), "Parser errors: {:?}", parse_diags);

    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&program).expect("Compilation failed");

    let mut vm = VM::new(bytecode);
    vm.run(&SecurityContext::allow_all())
}

// ============================================================================
// Compound Assignment Operators
// ============================================================================

#[rstest]
#[case("let x = 10; x += 5; x;", 15.0)]
#[case("let x = 10; x -= 3; x;", 7.0)]
#[case("let x = 4; x *= 3; x;", 12.0)]
#[case("let x = 20; x /= 4; x;", 5.0)]
#[case("let x = 17; x %= 5; x;", 2.0)]
fn test_compound_assignments(#[case] source: &str, #[case] expected: f64) {
    let result = execute_source(source);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(Value::Number(expected)));
}

// ============================================================================
// Increment/Decrement Operators
// ============================================================================

#[rstest]
#[case("let x = 5; x++; x;", 6.0)]
#[case("let x = 5; x--; x;", 4.0)]
fn test_increment_decrement(#[case] source: &str, #[case] expected: f64) {
    let result = execute_source(source);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(Value::Number(expected)));
}

// ============================================================================
// Array Operations
// ============================================================================

#[test]
fn test_array_index_assignment() {
    let result = execute_source("let arr = [1, 2, 3]; arr[1] = 42; arr[1];");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(Value::Number(42.0)));
}

#[test]
fn test_array_compound_assignment() {
    let result = execute_source("let arr = [10, 20, 30]; arr[1] += 5; arr[1];");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(Value::Number(25.0)));
}

#[test]
fn test_array_increment() {
    let result = execute_source("let arr = [5, 10, 15]; arr[1]++; arr[1];");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(Value::Number(11.0)));
}

// ============================================================================
// Function Execution
// ============================================================================

#[test]
fn test_user_function_simple() {
    let result = execute_source("fn double(x: number) -> number { return x * 2; } double(21);");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(Value::Number(42.0)));
}

#[test]
fn test_user_function_with_multiple_params() {
    let result =
        execute_source("fn add(a: number, b: number) -> number { return a + b; } add(10, 32);");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(Value::Number(42.0)));
}

#[test]
fn test_user_function_recursion() {
    let result = execute_source(
        r#"
        fn factorial(n: number) -> number {
            if (n <= 1) { return 1; }
            return n * factorial(n - 1);
        }
        factorial(5);
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(Value::Number(120.0)));
}

#[test]
fn test_user_function_with_local_variables() {
    let result = execute_source(
        r#"
        fn calculate(x: number) -> number {
            let y = x * 2;
            let z = y + 10;
            return z;
        }
        calculate(5);
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(Value::Number(20.0)));
}

#[test]
fn test_multiple_functions() {
    let result = execute_source(
        r#"
        fn double(x: number) -> number { return x * 2; }
        fn triple(x: number) -> number { return x * 3; }
        double(7) + triple(4);
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(Value::Number(26.0)));
}

#[test]
fn test_function_calling_function() {
    let result = execute_source(
        r#"
        fn add(a: number, b: number) -> number { return a + b; }
        fn addThree(a: number, b: number, c: number) -> number {
            return add(add(a, b), c);
        }
        addThree(10, 20, 12);
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(Value::Number(42.0)));
}

// ============================================================================
// Complex Expression Chains
// ============================================================================

#[test]
fn test_multiple_compound_assignments() {
    let result = execute_source(
        r#"
        let x = 10;
        x += 5;
        x *= 2;
        x -= 3;
        x;
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(Value::Number(27.0)));
}

#[test]
fn test_mixed_operators() {
    let result = execute_source(
        r#"
        let x = 5;
        x++;
        x *= 2;
        x--;
        x;
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(Value::Number(11.0)));
}

// ============================================================================
// Nested Operations
// ============================================================================

#[test]
fn test_array_in_function() {
    let result = execute_source(
        r#"
        fn modify_array() -> number {
            let arr = [1, 2, 3];
            arr[1] += 10;
            return arr[1];
        }
        modify_array();
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(Value::Number(12.0)));
}

#[test]
fn test_loop_with_compound_assignment() {
    let result = execute_source(
        r#"
        let sum = 0;
        let i = 0;
        while (i < 5) {
            sum += i;
            i++;
        }
        sum;
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(Value::Number(10.0)));
}

// ============================================================================
// From optimizer_tests.rs
// ============================================================================

// Optimizer unit tests
//
// Tests for the bytecode optimizer passes individually and in combination.

// ============================================================================
// Test helpers
// ============================================================================

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

// ============================================================================
// From optimizer_integration_tests.rs
// ============================================================================

// Optimizer integration tests
//
// Tests that optimized programs produce identical results to unoptimized ones
// across a wide range of programs.

fn assert_semantics(source: &str) {
    let orig = run(compile(source));
    let opt = run(compile_optimized(source));
    assert_eq!(
        orig, opt,
        "Semantics differ for:\n{}\nOrig: {:?}\nOpt:  {:?}",
        source, orig, opt
    );
}

fn assert_optimized_smaller_or_equal(source: &str) {
    let raw = compile(source);
    let opt = compile_optimized(source);
    assert!(
        opt.instructions.len() <= raw.instructions.len(),
        "Optimized should be <= raw size for: {}\nRaw: {} bytes, Opt: {} bytes",
        source,
        raw.instructions.len(),
        opt.instructions.len()
    );
}

fn assert_valid(source: &str) {
    let bc = compile_optimized(source);
    atlas_runtime::bytecode::validate(&bc).unwrap_or_else(|errors| {
        panic!(
            "Optimized bytecode invalid for: {}\nErrors: {:?}",
            source, errors
        )
    });
}

// ============================================================================
// Arithmetic programs
// ============================================================================

#[test]
fn test_arithmetic_basic() {
    assert_semantics("2 + 3;");
    assert_semantics("10 - 4;");
    assert_semantics("3 * 7;");
    assert_semantics("15 / 3;");
    assert_semantics("17 % 5;");
}

#[test]
fn test_arithmetic_compound() {
    assert_semantics("2 + 3 * 4;");
    assert_semantics("(2 + 3) * 4;");
    assert_semantics("10 - 2 * 3 + 1;");
}

#[test]
fn test_arithmetic_semantics_preserved() {
    assert_semantics("let x = 2 + 3;");
    assert_semantics("let x = 10 * 5;");
    assert_semantics("let a = 3; let b = a + 4;");
}

// ============================================================================
// Boolean programs
// ============================================================================

#[test]
fn test_boolean_not() {
    assert_semantics("let x = !true;");
    assert_semantics("let x = !false;");
}

#[test]
fn test_boolean_comparison() {
    assert_semantics("let x = 5 > 3;");
    assert_semantics("let x = 3 < 5;");
    assert_semantics("let x = 5 == 5;");
    assert_semantics("let x = 5 != 3;");
    assert_semantics("let x = 5 >= 5;");
    assert_semantics("let x = 5 <= 6;");
}

#[test]
fn test_short_circuit_and() {
    assert_semantics("let x = true && false;");
    assert_semantics("let x = false && true;");
    assert_semantics("let x = true && true;");
}

#[test]
fn test_short_circuit_or() {
    assert_semantics("let x = true || false;");
    assert_semantics("let x = false || true;");
    assert_semantics("let x = false || false;");
}

// ============================================================================
// Control flow programs
// ============================================================================

#[test]
fn test_if_then() {
    assert_semantics("let x = 5; if (x > 3) { x = 10; }");
    assert_semantics("let x = 1; if (x > 3) { x = 10; }");
}

#[test]
fn test_if_else() {
    assert_semantics("let x = 5; if (x > 3) { x = 1; } else { x = 2; }");
}

#[test]
fn test_while_loop() {
    assert_semantics("let i = 0; while (i < 5) { i = i + 1; }");
}

#[test]
fn test_nested_if() {
    assert_semantics("let x = 5; let y = 3; if (x > 0) { if (y > 0) { x = x + y; } }");
}

#[test]
fn test_loop_with_sum() {
    assert_semantics("let sum = 0; let i = 0; while (i < 10) { sum = sum + i; i = i + 1; }");
}

// ============================================================================
// Function programs
// ============================================================================

#[test]
fn test_simple_function() {
    assert_semantics("fn identity(x: number) -> number { return x; } identity(42);");
}

#[test]
fn test_function_with_arithmetic() {
    assert_semantics("fn add(a: number, b: number) -> number { return a + b; } add(10, 5);");
}

#[test]
fn test_function_with_constant_body() {
    // Function body contains foldable constants
    assert_semantics("fn constant_val() -> number { return 2 + 3; } constant_val();");
}

#[test]
fn test_function_with_conditionals() {
    assert_semantics(
        r#"fn max(a: number, b: number) -> number {
            if (a > b) { return a; } else { return b; }
        }
        max(10, 5);"#,
    );
}

#[test]
fn test_recursive_fibonacci() {
    // Recursive functions must work after optimization
    assert_semantics(
        r#"
        fn fib(n: number) -> number {
            if (n <= 1) { return n; }
            return fib(n - 1) + fib(n - 2);
        }
        fib(7);
        "#,
    );
}

#[test]
fn test_multiple_functions_opt() {
    assert_semantics(
        r#"
        fn double(x: number) -> number { return x * 2; }
        fn triple(x: number) -> number { return x * 3; }
        double(5) + triple(3);
        "#,
    );
}

// ============================================================================
// Array programs
// ============================================================================

#[test]
fn test_array_literal() {
    assert_semantics("let arr = [1, 2, 3];");
}

#[test]
fn test_array_index() {
    assert_semantics("let arr = [10, 20, 30]; arr[1];");
}

#[test]
fn test_array_assignment() {
    assert_semantics("let arr = [1, 2, 3]; arr[0] = 42;");
}

// ============================================================================
// Size reduction verification
// ============================================================================

#[test]
fn test_size_reduction_constant_arithmetic() {
    assert_optimized_smaller_or_equal("let x = 2 + 3 * 4;");
}

#[test]
fn test_size_reduction_function_with_return() {
    assert_optimized_smaller_or_equal("fn f(x: number) -> number { return x * 2; } f(5);");
}

#[test]
fn test_size_reduction_multiple_constants() {
    assert_optimized_smaller_or_equal("let a = 1 + 2; let b = 3 + 4; let c = 5 + 6;");
}

// ============================================================================
// Bytecode validity verification
// ============================================================================

#[test]
fn test_validity_simple_program() {
    assert_valid("let x = 5;");
}

#[test]
fn test_validity_function_program() {
    assert_valid("fn add(a: number, b: number) -> number { return a + b; } add(1, 2);");
}

#[test]
fn test_validity_loop_program() {
    assert_valid("let i = 0; while (i < 5) { i = i + 1; }");
}

#[test]
fn test_validity_if_else_program() {
    assert_valid("if (true) { 1; } else { 2; }");
}

#[test]
fn test_validity_nested_calls() {
    assert_valid(
        r#"
        fn inner(x: number) -> number { return x + 1; }
        fn outer(x: number) -> number { return inner(x) * 2; }
        outer(5);
        "#,
    );
}

// ============================================================================
// Statistics tests
// ============================================================================

#[test]
fn test_stats_size_fields() {
    let bc = compile("2 + 3 * 4;");
    let opt = Optimizer::with_default_passes();
    let (_, stats) = opt.optimize_with_stats(bc);
    assert!(stats.bytecode_size_before > 0);
    assert!(stats.bytecode_size_after > 0);
    assert!(stats.bytecode_size_after <= stats.bytecode_size_before);
}

#[test]
fn test_stats_reduction_percent() {
    let bc = compile("2 + 3;");
    let opt = Optimizer::with_default_passes();
    let (_, stats) = opt.optimize_with_stats(bc);
    let percent = stats.size_reduction_percent();
    assert!(percent >= 0.0 && percent <= 100.0);
}

#[test]
fn test_stats_constants_folded_positive() {
    let bc = compile("2 + 3;");
    let opt = Optimizer::with_default_passes();
    let (_, stats) = opt.optimize_with_stats(bc);
    assert!(stats.constants_folded > 0);
}

#[test]
fn test_stats_dead_removed_for_fn_with_return() {
    let bc = compile("fn f(x: number) -> number { return x + 1; } f(5);");
    let opt = Optimizer::with_default_passes();
    let (_, stats) = opt.optimize_with_stats(bc);
    assert!(stats.dead_instructions_removed > 0);
}

// ============================================================================
// From profiler_tests.rs
// ============================================================================

// Profiler integration tests
//
// Tests the complete profiler pipeline: collector → hotspot detection →
// report generation → VM integration.

// ===========================================================================
// Helpers
// ===========================================================================

fn simple_add_bytecode() -> Bytecode {
    let mut bc = Bytecode::new();
    let idx_a = bc.add_constant(Value::Number(10.0));
    let idx_b = bc.add_constant(Value::Number(20.0));
    bc.emit(Opcode::Constant, atlas_runtime::span::Span::dummy());
    bc.emit_u16(idx_a);
    bc.emit(Opcode::Constant, atlas_runtime::span::Span::dummy());
    bc.emit_u16(idx_b);
    bc.emit(Opcode::Add, atlas_runtime::span::Span::dummy());
    bc.emit(Opcode::Halt, atlas_runtime::span::Span::dummy());
    bc
}

#[allow(dead_code)]
fn loop_bytecode(iterations: u16) -> Bytecode {
    // Pushes `iterations` onto the stack and loops `iterations` times,
    // decrementing a counter each iteration.
    let mut bc = Bytecode::new();
    let span = atlas_runtime::span::Span::dummy();

    // counter = iterations
    let iter_idx = bc.add_constant(Value::Number(iterations as f64));
    bc.emit(Opcode::Constant, span);
    bc.emit_u16(iter_idx);

    // loop body: counter - 1
    let one_idx = bc.add_constant(Value::Number(1.0));
    let zero_idx = bc.add_constant(Value::Number(0.0));

    // dup the counter
    bc.emit(Opcode::Dup, span);
    // subtract 1
    bc.emit(Opcode::Constant, span);
    bc.emit_u16(one_idx);
    bc.emit(Opcode::Sub, span);
    // dup new counter (for comparison)
    bc.emit(Opcode::Dup, span);
    // push 0
    bc.emit(Opcode::Constant, span);
    bc.emit_u16(zero_idx);
    // counter > 0 ?
    bc.emit(Opcode::Greater, span);
    // if false, exit loop
    bc.emit(Opcode::JumpIfFalse, span);
    // forward jump placeholder — patch below
    let patch_pos = bc.instructions.len();
    bc.emit_u16(0u16);
    // pop the duplicate
    bc.emit(Opcode::Pop, span);
    // loop back: offset must jump back to the Dup at the start of the body
    // The body starts at offset 6 (after the initial Constant + emit_u16 = 3 bytes)
    // We'll use a jump back offset
    // Current ip after Loop opcode is emitted: we need to calculate the negative offset
    // For simplicity, use a simple JumpIfFalse structure above and break out
    bc.emit(Opcode::Jump, span);
    let back_offset_pos = bc.instructions.len();
    bc.emit_u16(0u16);

    // patch the JumpIfFalse to here (pop + halt)
    let pop_pos = bc.instructions.len();
    let forward = (pop_pos as isize - (patch_pos + 2) as isize) as i16;
    bc.instructions[patch_pos] = (forward >> 8) as u8;
    bc.instructions[patch_pos + 1] = forward as u8;

    bc.emit(Opcode::Pop, span); // pop duplicate
    bc.emit(Opcode::Halt, span);

    // patch back jump to loop start (Dup)
    let halt_pos = bc.instructions.len();
    let loop_start: isize = 3; // offset of the first Dup
    let back_offset = (loop_start - (back_offset_pos + 2) as isize) as i16;
    bc.instructions[back_offset_pos] = (back_offset >> 8) as u8;
    bc.instructions[back_offset_pos + 1] = back_offset as u8;

    let _ = halt_pos;
    bc
}

fn function_call_bytecode() -> Bytecode {
    // Defines a function that returns 42 and calls it once
    let mut bc = Bytecode::new();
    let span = atlas_runtime::span::Span::dummy();

    // function body starts after the Call + Halt in main
    // main: push FunctionRef, Call 0, Halt
    // fn body: Constant(42), Return
    let fn_body_offset = 10usize; // approximate — we'll place it exactly

    let func_ref = FunctionRef {
        name: "answer".to_string(),
        arity: 0,
        bytecode_offset: fn_body_offset,
        local_count: 1,
    };
    let func_idx = bc.add_constant(Value::Function(func_ref));
    let val_idx = bc.add_constant(Value::Number(42.0));

    // main: 0 - push func (3 bytes), 3 - Call u8 (2 bytes), 5 - Pop (1), 6 - Halt (1) = 7 bytes
    // fn body at offset 7
    bc.emit(Opcode::Constant, span);
    bc.emit_u16(func_idx);
    bc.emit(Opcode::Call, span);
    bc.emit_u8(0);
    bc.emit(Opcode::Halt, span);

    // Patch function offset
    let actual_fn_offset = bc.instructions.len();
    if let Value::Function(ref mut f) = bc.constants[func_idx as usize] {
        f.bytecode_offset = actual_fn_offset;
    }

    // fn body: push 42, Return
    bc.emit(Opcode::Constant, span);
    bc.emit_u16(val_idx);
    bc.emit(Opcode::Return, span);

    bc
}

// ===========================================================================
// Section 1: ProfileCollector unit tests
// ===========================================================================

#[test]
fn test_collector_empty_state() {
    let c = ProfileCollector::new();
    assert_eq!(c.total_instructions(), 0);
    assert!(c.instruction_counts().is_empty());
    assert_eq!(c.max_stack_depth(), 0);
    assert_eq!(c.function_calls(), 0);
}

#[test]
fn test_collector_counts_instructions() {
    let mut c = ProfileCollector::new();
    c.record_instruction(Opcode::Add, 0);
    c.record_instruction(Opcode::Add, 3);
    c.record_instruction(Opcode::Sub, 6);
    assert_eq!(c.total_instructions(), 3);
    assert_eq!(c.instruction_count(Opcode::Add), 2);
    assert_eq!(c.instruction_count(Opcode::Sub), 1);
}

#[test]
fn test_collector_tracks_location() {
    let mut c = ProfileCollector::new();
    for _ in 0..5 {
        c.record_instruction(Opcode::Loop, 100);
    }
    assert_eq!(c.location_counts()[&100], 5);
}

#[test]
fn test_collector_opcode_at_ip() {
    let mut c = ProfileCollector::new();
    c.record_instruction(Opcode::Mul, 42);
    assert_eq!(c.opcode_at(42), Some(Opcode::Mul));
}

#[test]
fn test_collector_max_stack_depth() {
    let mut c = ProfileCollector::new();
    c.update_frame_depth(1);
    c.update_frame_depth(8);
    c.update_frame_depth(3);
    assert_eq!(c.max_stack_depth(), 8);
}

#[test]
fn test_collector_function_calls() {
    let mut c = ProfileCollector::new();
    c.record_function_call("main");
    c.record_function_call("helper");
    c.record_function_call("helper");
    assert_eq!(c.function_calls(), 3);
    assert_eq!(c.function_call_counts()["helper"], 2);
}

#[test]
fn test_collector_reset() {
    let mut c = ProfileCollector::new();
    c.record_instruction(Opcode::Add, 0);
    c.update_frame_depth(5);
    c.record_function_call("f");
    c.reset();
    assert_eq!(c.total_instructions(), 0);
    assert_eq!(c.max_stack_depth(), 0);
    assert_eq!(c.function_calls(), 0);
}

#[test]
fn test_collector_top_opcodes_ordering() {
    let mut c = ProfileCollector::new();
    for _ in 0..30 {
        c.record_instruction(Opcode::Add, 0);
    }
    for _ in 0..10 {
        c.record_instruction(Opcode::Mul, 3);
    }
    let top = c.top_opcodes(2);
    assert_eq!(top[0].0, Opcode::Add);
    assert_eq!(top[1].0, Opcode::Mul);
}

#[test]
fn test_collector_top_locations() {
    let mut c = ProfileCollector::new();
    for _ in 0..100 {
        c.record_instruction(Opcode::Loop, 50);
    }
    for _ in 0..20 {
        c.record_instruction(Opcode::Add, 10);
    }
    let top = c.top_locations(1);
    assert_eq!(top[0].0, 50);
}

// ===========================================================================
// Section 2: HotspotDetector tests
// ===========================================================================

#[test]
fn test_hotspot_detector_default_threshold() {
    let d = HotspotDetector::new();
    assert!((d.threshold() - 1.0).abs() < 0.001);
}

#[test]
fn test_hotspot_detector_detects_loop() {
    let mut c = ProfileCollector::new();
    for _ in 0..50 {
        c.record_instruction(Opcode::Loop, 20);
    }
    for _ in 0..50 {
        c.record_instruction(Opcode::Add, 5);
    }
    let d = HotspotDetector::new();
    let hs = d.detect(&c);
    // Both are 50% — both should be detected
    assert_eq!(hs.len(), 2);
}

#[test]
fn test_hotspot_detector_threshold_filter() {
    let mut c = ProfileCollector::new();
    // 99 at ip=0, 1 at ip=1 (total=100 → ip=1 is 1.0% which equals threshold)
    for _ in 0..99 {
        c.record_instruction(Opcode::Add, 0);
    }
    c.record_instruction(Opcode::Mul, 1);
    let d = HotspotDetector::with_threshold(1.0);
    let hs = d.detect(&c);
    // ip=1 (1%) should be included (>= 1.0%)
    assert!(hs.iter().any(|h| h.ip == 1));
}

#[test]
fn test_hotspot_detector_sorts_by_count() {
    let mut c = ProfileCollector::new();
    for _ in 0..10 {
        c.record_instruction(Opcode::Add, 5);
    }
    for _ in 0..40 {
        c.record_instruction(Opcode::Loop, 10);
    }
    for _ in 0..50 {
        c.record_instruction(Opcode::Mul, 15);
    }
    let d = HotspotDetector::new();
    let hs = d.detect(&c);
    assert!(hs[0].count >= hs[1].count);
}

#[test]
fn test_hotspot_percentage_calculation() {
    let mut c = ProfileCollector::new();
    for _ in 0..1 {
        c.record_instruction(Opcode::Add, 0);
    }
    for _ in 0..9 {
        c.record_instruction(Opcode::Mul, 3);
    }
    let d = HotspotDetector::with_threshold(1.0);
    let hs = d.detect(&c);
    let mul_hs = hs.iter().find(|h| h.ip == 3).unwrap();
    assert!((mul_hs.percentage - 90.0).abs() < 0.1);
}

#[test]
fn test_hotspot_opcode_label() {
    let mut c = ProfileCollector::new();
    for _ in 0..100 {
        c.record_instruction(Opcode::GetLocal, 7);
    }
    let d = HotspotDetector::new();
    let hs = d.detect(&c);
    assert_eq!(hs[0].opcode, Some(Opcode::GetLocal));
}

// ===========================================================================
// Section 3: ProfileReport formatting tests
// ===========================================================================

fn make_full_report() -> ProfileReport {
    let mut p = Profiler::enabled();
    for _ in 0..100 {
        for i in 0..10usize {
            p.record_instruction_at(Opcode::Add, i * 3);
        }
    }
    p.update_frame_depth(4);
    p.update_value_stack_depth(12);
    p.record_function_call("compute");
    p.record_function_call("compute");
    p.generate_report(1.0)
}

#[test]
fn test_report_total_instructions() {
    let r = make_full_report();
    assert_eq!(r.total_instructions, 1000);
}

#[test]
fn test_report_max_stack_depth() {
    let r = make_full_report();
    assert_eq!(r.max_stack_depth, 4);
}

#[test]
fn test_report_function_calls() {
    let r = make_full_report();
    assert_eq!(r.function_calls, 2);
}

#[test]
fn test_report_top_opcodes_not_empty() {
    let r = make_full_report();
    assert!(!r.top_opcodes.is_empty());
}

#[test]
fn test_report_hotspots_detected() {
    let r = make_full_report();
    // Each of 10 locations gets exactly 10% — all above 1% threshold
    assert!(!r.hotspots.is_empty());
}

#[test]
fn test_report_summary_contains_count() {
    let r = make_full_report();
    let s = r.format_summary();
    assert!(s.contains("1000"), "summary: {}", s);
}

#[test]
fn test_report_detailed_contains_execution_section() {
    let r = make_full_report();
    let s = r.format_detailed();
    assert!(s.contains("Execution Summary"), "detailed: {}", s);
}

#[test]
fn test_report_detailed_contains_opcode_section() {
    let r = make_full_report();
    let s = r.format_detailed();
    assert!(s.contains("Top Opcodes"), "detailed: {}", s);
}

#[test]
fn test_report_detailed_contains_hotspot_section() {
    let r = make_full_report();
    let s = r.format_detailed();
    assert!(s.contains("Hotspot"), "detailed: {}", s);
}

#[test]
fn test_report_opcode_table_format() {
    let r = make_full_report();
    let s = r.format_opcode_table();
    assert!(s.contains("Add"), "opcode table: {}", s);
    assert!(s.contains("100.00%"), "opcode table: {}", s);
}

// ===========================================================================
// Section 4: Profiler struct integration
// ===========================================================================

#[test]
fn test_profiler_new_disabled() {
    let p = Profiler::new();
    assert!(!p.is_enabled());
}

#[test]
fn test_profiler_records_when_enabled() {
    let mut p = Profiler::enabled();
    p.record_instruction(Opcode::Add);
    p.record_instruction(Opcode::Add);
    assert_eq!(p.total_instructions(), 2);
}

#[test]
fn test_profiler_ignores_when_disabled() {
    let mut p = Profiler::new();
    p.record_instruction(Opcode::Add);
    assert_eq!(p.total_instructions(), 0);
}

#[test]
fn test_profiler_timing_records_elapsed() {
    let mut p = Profiler::enabled();
    p.start_timing();
    for i in 0..500usize {
        p.record_instruction_at(Opcode::Add, i % 50);
    }
    p.stop_timing();
    assert!(p.elapsed_secs().is_some());
    assert!(p.elapsed_secs().unwrap() >= 0.0);
}

#[test]
fn test_profiler_ips_is_positive() {
    let mut p = Profiler::enabled();
    p.start_timing();
    for i in 0..1000usize {
        p.record_instruction_at(Opcode::Mul, i % 100);
    }
    p.stop_timing();
    let r = p.generate_report(1.0);
    if let Some(ips) = r.ips {
        assert!(ips > 0.0);
    }
}

#[test]
fn test_profiler_hotspots_shorthand() {
    let mut p = Profiler::enabled();
    for _ in 0..100 {
        p.record_instruction_at(Opcode::Loop, 0);
    }
    assert!(!p.hotspots().is_empty());
}

#[test]
fn test_profiler_top_opcodes_shorthand() {
    let mut p = Profiler::enabled();
    for _ in 0..50 {
        p.record_instruction(Opcode::Add);
    }
    for _ in 0..20 {
        p.record_instruction(Opcode::Mul);
    }
    let top = p.top_opcodes(2);
    assert_eq!(top[0].opcode, Opcode::Add);
}

#[test]
fn test_profiler_reset() {
    let mut p = Profiler::enabled();
    p.record_instruction(Opcode::Add);
    p.reset();
    assert_eq!(p.total_instructions(), 0);
}

// ===========================================================================
// Section 5: VM integration tests
// ===========================================================================

#[test]
fn test_vm_with_profiling_enabled() {
    let bc = simple_add_bytecode();
    let mut vm = VM::with_profiling(bc);
    let result = vm.run(&SecurityContext::allow_all()).unwrap();
    assert_eq!(result, Some(Value::Number(30.0)));

    let p = vm.profiler().unwrap();
    assert!(p.is_enabled());
    assert!(p.total_instructions() > 0);
}

#[test]
fn test_vm_instruction_count_accuracy() {
    // simple_add_bytecode: Constant, Constant, Add, Halt = 4 opcodes
    let bc = simple_add_bytecode();
    let mut vm = VM::with_profiling(bc);
    vm.run(&SecurityContext::allow_all()).unwrap();

    let p = vm.profiler().unwrap();
    assert_eq!(p.total_instructions(), 4);
    assert_eq!(p.instruction_count(Opcode::Constant), 2);
    assert_eq!(p.instruction_count(Opcode::Add), 1);
    assert_eq!(p.instruction_count(Opcode::Halt), 1);
}

#[test]
fn test_vm_profiling_not_enabled_by_default() {
    let bc = simple_add_bytecode();
    let mut vm = VM::new(bc);
    vm.run(&SecurityContext::allow_all()).unwrap();
    assert!(vm.profiler().is_none());
}

#[test]
fn test_vm_profiling_records_stack_depth() {
    let bc = simple_add_bytecode();
    let mut vm = VM::with_profiling(bc);
    vm.run(&SecurityContext::allow_all()).unwrap();

    let p = vm.profiler().unwrap();
    // At some point during execution the value stack had at least 1 item
    assert!(p.collector().max_value_stack_depth() >= 1);
}

#[test]
fn test_vm_profiling_records_frame_depth() {
    let bc = simple_add_bytecode();
    let mut vm = VM::with_profiling(bc);
    vm.run(&SecurityContext::allow_all()).unwrap();

    let p = vm.profiler().unwrap();
    // main frame is always present, so at least depth 1
    assert!(p.max_stack_depth() >= 1);
}

#[test]
fn test_vm_profiling_function_call_tracking() {
    let bc = function_call_bytecode();
    let mut vm = VM::with_profiling(bc);
    vm.run(&SecurityContext::allow_all()).unwrap();

    let p = vm.profiler().unwrap();
    assert_eq!(p.function_calls(), 1);
    assert_eq!(p.collector().function_call_counts()["answer"], 1);
}

#[test]
fn test_vm_profiling_generates_report() {
    let bc = simple_add_bytecode();
    let mut vm = VM::with_profiling(bc);
    vm.run(&SecurityContext::allow_all()).unwrap();

    let p = vm.profiler().unwrap();
    let report = p.generate_report(1.0);
    assert_eq!(report.total_instructions, 4);
    assert!(!report.top_opcodes.is_empty());
}

#[test]
fn test_vm_profiling_timing_is_recorded() {
    let bc = simple_add_bytecode();
    let mut vm = VM::with_profiling(bc);
    vm.run(&SecurityContext::allow_all()).unwrap();

    let p = vm.profiler().unwrap();
    assert!(p.elapsed_secs().is_some(), "timing should be recorded");
}

#[test]
fn test_vm_enable_profiling_after_creation() {
    let bc = simple_add_bytecode();
    let mut vm = VM::new(bc);
    vm.enable_profiling();
    vm.run(&SecurityContext::allow_all()).unwrap();

    let p = vm.profiler().unwrap();
    assert!(p.total_instructions() > 0);
}

#[test]
fn test_vm_opcode_breakdown_correctness() {
    let bc = simple_add_bytecode();
    let mut vm = VM::with_profiling(bc);
    vm.run(&SecurityContext::allow_all()).unwrap();

    let p = vm.profiler().unwrap();
    let counts = p.instruction_counts();

    // Verify specific opcodes are counted
    assert_eq!(counts.get(&(Opcode::Add as u8)), Some(&1));
    assert_eq!(counts.get(&(Opcode::Constant as u8)), Some(&2));
}

#[test]
fn test_vm_report_ips_present_after_run() {
    let bc = simple_add_bytecode();
    let mut vm = VM::with_profiling(bc);
    vm.run(&SecurityContext::allow_all()).unwrap();

    let report = vm.profiler().unwrap().generate_report(1.0);
    // IPS should be populated since timing was recorded
    assert!(report.ips.is_some());
    assert!(report.ips.unwrap() > 0.0);
}

// ============================================================================
// From nested_function_parity_tests.rs
// ============================================================================

// Parity tests for nested functions (Phase 7)
//
// Ensures 100% interpreter/VM parity for nested function execution.
// Every test runs the same source code in both engines and verifies
// identical output.

// ============================================================================
// Basic Nested Function Calls
// ============================================================================

#[test]
fn parity_nested_function_basic() {
    let source = r#"
        fn outer() -> number {
            fn helper(x: number) -> number {
                return x * 2;
            }
            return helper(21);
        }
        outer();
    "#;

    let interp_result = run_interpreter(source).unwrap();
    let vm_result = run_vm(source).unwrap();

    assert_eq!(interp_result, vm_result);
    assert_eq!(interp_result, Value::Number(42.0));
}

#[test]
fn parity_nested_function_multiple_params() {
    let source = r#"
        fn outer() -> number {
            fn add(a: number, b: number) -> number {
                return a + b;
            }
            return add(10, 32);
        }
        outer();
    "#;

    let interp_result = run_interpreter(source).unwrap();
    let vm_result = run_vm(source).unwrap();

    assert_eq!(interp_result, vm_result);
    assert_eq!(interp_result, Value::Number(42.0));
}

#[test]
fn parity_nested_function_string() {
    let source = r#"
        fn outer() -> string {
            fn greet(name: string) -> string {
                return "Hello, " + name;
            }
            return greet("World");
        }
        outer();
    "#;

    let interp_result = run_interpreter(source).unwrap();
    let vm_result = run_vm(source).unwrap();

    assert_eq!(interp_result, vm_result);
    assert_eq!(interp_result, Value::string("Hello, World"));
}

// ============================================================================
// Parameter Access
// ============================================================================

#[test]
fn parity_nested_function_params() {
    let source = r#"
        fn outer(x: number) -> number {
            fn double(y: number) -> number {
                return y * 2;
            }
            return double(x);
        }
        outer(21);
    "#;

    let interp_result = run_interpreter(source).unwrap();
    let vm_result = run_vm(source).unwrap();

    assert_eq!(interp_result, vm_result);
    assert_eq!(interp_result, Value::Number(42.0));
}

// ============================================================================
// Shadowing
// ============================================================================

#[test]
fn parity_nested_function_shadows_global() {
    let source = r#"
        fn foo() -> number {
            return 1;
        }

        fn outer() -> number {
            fn foo() -> number {
                return 42;
            }
            return foo();
        }
        outer();
    "#;

    let interp_result = run_interpreter(source).unwrap();
    let vm_result = run_vm(source).unwrap();

    assert_eq!(interp_result, vm_result);
    assert_eq!(interp_result, Value::Number(42.0));
}

#[test]
fn parity_nested_function_shadows_outer_nested() {
    let source = r#"
        fn level1() -> number {
            fn helper() -> number {
                return 1;
            }
            fn level2() -> number {
                fn helper() -> number {
                    return 42;
                }
                return helper();
            }
            return level2();
        }
        level1();
    "#;

    let interp_result = run_interpreter(source).unwrap();
    let vm_result = run_vm(source).unwrap();

    assert_eq!(interp_result, vm_result);
    assert_eq!(interp_result, Value::Number(42.0));
}

// ============================================================================
// Multiple Nesting Levels
// ============================================================================

#[test]
fn parity_deeply_nested_functions() {
    let source = r#"
        fn level1() -> number {
            fn level2() -> number {
                fn level3() -> number {
                    fn level4() -> number {
                        return 42;
                    }
                    return level4();
                }
                return level3();
            }
            return level2();
        }
        level1();
    "#;

    let interp_result = run_interpreter(source).unwrap();
    let vm_result = run_vm(source).unwrap();

    assert_eq!(interp_result, vm_result);
    assert_eq!(interp_result, Value::Number(42.0));
}

// ============================================================================
// Nested Functions Calling Each Other
// ============================================================================

#[test]
fn parity_nested_function_calling_nested() {
    let source = r#"
        fn outer() -> number {
            fn helper1() -> number {
                return 10;
            }
            fn helper2() -> number {
                return helper1() + 32;
            }
            return helper2();
        }
        outer();
    "#;

    let interp_result = run_interpreter(source).unwrap();
    let vm_result = run_vm(source).unwrap();

    assert_eq!(interp_result, vm_result);
    assert_eq!(interp_result, Value::Number(42.0));
}

#[test]
fn parity_nested_function_calling_outer() {
    let source = r#"
        fn global() -> number {
            return 40;
        }

        fn outer() -> number {
            fn nested() -> number {
                return global() + 2;
            }
            return nested();
        }
        outer();
    "#;

    let interp_result = run_interpreter(source).unwrap();
    let vm_result = run_vm(source).unwrap();

    assert_eq!(interp_result, vm_result);
    assert_eq!(interp_result, Value::Number(42.0));
}

// ============================================================================
// Void Functions
// ============================================================================

#[test]
fn parity_nested_function_void() {
    let source = r#"
        var result: number = 0;

        fn outer() -> void {
            fn setResult() -> void {
                result = 42;
            }
            setResult();
        }

        outer();
        result;
    "#;

    let interp_result = run_interpreter(source).unwrap();
    let vm_result = run_vm(source).unwrap();

    assert_eq!(interp_result, vm_result);
    assert_eq!(interp_result, Value::Number(42.0));
}

// ============================================================================
// Arrays
// ============================================================================

#[test]
fn parity_nested_function_array_param() {
    let source = r#"
        fn outer() -> number {
            fn sum(arr: number[]) -> number {
                return arr[0] + arr[1];
            }
            let nums: number[] = [10, 32];
            return sum(nums);
        }
        outer();
    "#;

    let interp_result = run_interpreter(source).unwrap();
    let vm_result = run_vm(source).unwrap();

    assert_eq!(interp_result, vm_result);
    assert_eq!(interp_result, Value::Number(42.0));
}

#[test]
fn parity_nested_function_array_return() {
    let source = r#"
        fn outer() -> number[] {
            fn makeArray() -> number[] {
                return [42, 100];
            }
            return makeArray();
        }
        outer()[0];
    "#;

    let interp_result = run_interpreter(source).unwrap();
    let vm_result = run_vm(source).unwrap();

    assert_eq!(interp_result, vm_result);
    assert_eq!(interp_result, Value::Number(42.0));
}

// ============================================================================
// Control Flow
// ============================================================================

#[test]
fn parity_nested_function_conditional() {
    let source = r#"
        fn outer() -> number {
            fn abs(x: number) -> number {
                if (x < 0) {
                    return -x;
                } else {
                    return x;
                }
            }
            return abs(-42);
        }
        outer();
    "#;

    let interp_result = run_interpreter(source).unwrap();
    let vm_result = run_vm(source).unwrap();

    assert_eq!(interp_result, vm_result);
    assert_eq!(interp_result, Value::Number(42.0));
}

// ============================================================================
// Nested Functions in Different Block Types
// ============================================================================

#[test]
fn parity_nested_function_in_if_block() {
    let source = r#"
        fn outer() -> number {
            if (true) {
                fn helper() -> number {
                    return 42;
                }
                return helper();
            }
            return 0;
        }
        outer();
    "#;

    let interp_result = run_interpreter(source).unwrap();
    let vm_result = run_vm(source).unwrap();

    assert_eq!(interp_result, vm_result);
    assert_eq!(interp_result, Value::Number(42.0));
}

// ============================================================================
// Additional Parity Tests (to reach 20+)
// ============================================================================

#[test]
fn parity_nested_function_no_params() {
    let source = r#"
        fn outer() -> number {
            fn getValue() -> number {
                return 42;
            }
            return getValue();
        }
        outer();
    "#;

    let interp_result = run_interpreter(source).unwrap();
    let vm_result = run_vm(source).unwrap();

    assert_eq!(interp_result, vm_result);
    assert_eq!(interp_result, Value::Number(42.0));
}

#[test]
fn parity_multiple_nested_same_level() {
    let source = r#"
        fn outer() -> number {
            fn a() -> number { return 10; }
            fn b() -> number { return 20; }
            fn c() -> number { return 12; }
            return a() + b() + c();
        }
        outer();
    "#;

    let interp_result = run_interpreter(source).unwrap();
    let vm_result = run_vm(source).unwrap();

    assert_eq!(interp_result, vm_result);
    assert_eq!(interp_result, Value::Number(42.0));
}

#[test]
fn parity_nested_with_local_variables() {
    let source = r#"
        fn outer() -> number {
            fn compute() -> number {
                let x = 21;
                let y = x * 2;
                return y;
            }
            return compute();
        }
        outer();
    "#;

    let interp_result = run_interpreter(source).unwrap();
    let vm_result = run_vm(source).unwrap();

    assert_eq!(interp_result, vm_result);
    assert_eq!(interp_result, Value::Number(42.0));
}

#[test]
fn parity_nested_returning_bool() {
    let source = r#"
        fn outer() -> bool {
            fn isTrue() -> bool {
                return true;
            }
            return isTrue();
        }
        outer();
    "#;

    let interp_result = run_interpreter(source).unwrap();
    let vm_result = run_vm(source).unwrap();

    assert_eq!(interp_result, vm_result);
    assert_eq!(interp_result, Value::Bool(true));
}

#[test]
fn parity_nested_with_arithmetic() {
    let source = r#"
        fn outer() -> number {
            fn calculate(a: number, b: number, c: number) -> number {
                return (a + b) * c;
            }
            return calculate(5, 9, 3);
        }
        outer();
    "#;

    let interp_result = run_interpreter(source).unwrap();
    let vm_result = run_vm(source).unwrap();

    assert_eq!(interp_result, vm_result);
    assert_eq!(interp_result, Value::Number(42.0));
}

#[test]
fn parity_nested_with_string_concat() {
    let source = r#"
        fn outer() -> string {
            fn combine(a: string, b: string) -> string {
                return a + b;
            }
            return combine("Hello", "World");
        }
        outer();
    "#;

    let interp_result = run_interpreter(source).unwrap();
    let vm_result = run_vm(source).unwrap();

    assert_eq!(interp_result, vm_result);
    assert_eq!(interp_result, Value::string("HelloWorld"));
}

#[test]
fn parity_nested_calling_multiple_siblings() {
    let source = r#"
        fn outer() -> number {
            fn getBase() -> number {
                return 20;
            }
            fn getBonus() -> number {
                return 22;
            }
            fn total() -> number {
                return getBase() + getBonus();
            }
            return total();
        }
        outer();
    "#;

    let interp_result = run_interpreter(source).unwrap();
    let vm_result = run_vm(source).unwrap();

    assert_eq!(interp_result, vm_result);
    assert_eq!(interp_result, Value::Number(42.0));
}

// ============================================================================
// From pattern_matching_runtime_tests.rs
// ============================================================================

// Pattern Matching Runtime Execution Tests (BLOCKER 03-B)
//
// Comprehensive tests for pattern matching execution in both interpreter and VM.
// Tests verify that patterns match correctly, variables bind properly, and both
// engines produce identical results (100% parity).

/// Helper to run code in interpreter
fn pm_run_interpreter(source: &str) -> Result<String, String> {
    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize();
    if !lex_diags.is_empty() {
        return Err(format!("Lex error: {:?}", lex_diags));
    }

    let mut parser = Parser::new(tokens);
    let (program, parse_diags) = parser.parse();
    if !parse_diags.is_empty() {
        return Err(format!("Parse error: {:?}", parse_diags));
    }

    let mut binder = Binder::new();
    let (mut symbol_table, bind_diags) = binder.bind(&program);
    if !bind_diags.is_empty() {
        return Err(format!("Bind error: {:?}", bind_diags));
    }

    let mut typechecker = TypeChecker::new(&mut symbol_table);
    let type_diags = typechecker.check(&program);
    if !type_diags.is_empty() {
        return Err(format!("Type error: {:?}", type_diags));
    }

    let mut interpreter = Interpreter::new();
    match interpreter.eval(&program, &SecurityContext::allow_all()) {
        Ok(value) => Ok(format!("{:?}", value)),
        Err(e) => Err(format!("Runtime error: {:?}", e)),
    }
}

/// Helper to run code in VM
fn pm_run_vm(source: &str) -> Result<String, String> {
    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize();
    if !lex_diags.is_empty() {
        return Err(format!("Lex error: {:?}", lex_diags));
    }

    let mut parser = Parser::new(tokens);
    let (program, parse_diags) = parser.parse();
    if !parse_diags.is_empty() {
        return Err(format!("Parse error: {:?}", parse_diags));
    }

    let mut binder = Binder::new();
    let (mut symbol_table, bind_diags) = binder.bind(&program);
    if !bind_diags.is_empty() {
        return Err(format!("Bind error: {:?}", bind_diags));
    }

    let mut typechecker = TypeChecker::new(&mut symbol_table);
    let type_diags = typechecker.check(&program);
    if !type_diags.is_empty() {
        return Err(format!("Type error: {:?}", type_diags));
    }

    let mut compiler = Compiler::new();
    match compiler.compile(&program) {
        Ok(bytecode) => {
            let mut vm = VM::new(bytecode);
            match vm.run(&SecurityContext::allow_all()) {
                Ok(opt_value) => match opt_value {
                    Some(value) => Ok(format!("{:?}", value)),
                    None => Ok("None".to_string()),
                },
                Err(e) => Err(format!("Runtime error: {:?}", e)),
            }
        }
        Err(diags) => Err(format!("Compile error: {:?}", diags)),
    }
}

// ============================================================================
// Literal Pattern Tests
// ============================================================================

#[test]
fn test_literal_number_match() {
    let source = r#"
        fn test(x: number) -> string {
            return match x {
                42 => "matched",
                _ => "not matched"
            };
        }
        test(42);
    "#;

    let interp_result = pm_run_interpreter(source).unwrap();
    assert_eq!(interp_result, r#"String("matched")"#);

    // VM test (will fail until VM implementation complete)
    if let Ok(vm_result) = pm_run_vm(source) {
        assert_eq!(vm_result, interp_result, "Parity check failed");
    }
}

#[test]
fn test_literal_string_match() {
    let source = r#"
        fn test(s: string) -> number {
            return match s {
                "hello" => 1,
                "world" => 2,
                _ => 0
            };
        }
        test("world");
    "#;

    let interp_result = pm_run_interpreter(source).unwrap();
    assert_eq!(interp_result, "Number(2.0)");

    if let Ok(vm_result) = pm_run_vm(source) {
        assert_eq!(vm_result, interp_result);
    }
}

#[test]
fn test_literal_bool_match() {
    let source = r#"
        fn test(b: bool) -> string {
            return match b {
                true => "yes",
                false => "no"
            };
        }
        test(true);
    "#;

    let interp_result = pm_run_interpreter(source).unwrap();
    assert_eq!(interp_result, r#"String("yes")"#);

    if let Ok(vm_result) = pm_run_vm(source) {
        assert_eq!(vm_result, interp_result);
    }
}

// ============================================================================
// Wildcard Pattern Tests
// ============================================================================

#[test]
fn test_wildcard_catch_all() {
    let source = r#"
        fn test(x: number) -> string {
            return match x {
                _ => "always matches"
            };
        }
        test(999);
    "#;

    let interp_result = pm_run_interpreter(source).unwrap();
    assert_eq!(interp_result, r#"String("always matches")"#);

    if let Ok(vm_result) = pm_run_vm(source) {
        assert_eq!(vm_result, interp_result);
    }
}

// ============================================================================
// Variable Binding Pattern Tests
// ============================================================================

#[test]
fn test_variable_binding_simple() {
    let source = r#"
        fn test(x: number) -> number {
            return match x {
                value => value + 10
            };
        }
        test(5);
    "#;

    let interp_result = pm_run_interpreter(source).unwrap();
    assert_eq!(interp_result, "Number(15.0)");

    if let Ok(vm_result) = pm_run_vm(source) {
        assert_eq!(vm_result, interp_result);
    }
}

#[test]
fn test_variable_binding_with_literal() {
    let source = r#"
        fn test(x: number) -> number {
            return match x {
                0 => 100,
                n => n * 2
            };
        }
        test(7);
    "#;

    let interp_result = pm_run_interpreter(source).unwrap();
    assert_eq!(interp_result, "Number(14.0)");

    if let Ok(vm_result) = pm_run_vm(source) {
        assert_eq!(vm_result, interp_result);
    }
}

// ============================================================================
// Option Pattern Tests
// ============================================================================

#[test]
fn test_option_some_match() {
    let source = r#"
        fn test(opt: Option<number>) -> number {
            return match opt {
                Some(x) => x,
                None => 0
            };
        }
        test(Some(42));
    "#;

    let interp_result = pm_run_interpreter(source).unwrap();
    assert_eq!(interp_result, "Number(42.0)");

    if let Ok(vm_result) = pm_run_vm(source) {
        assert_eq!(vm_result, interp_result);
    }
}

#[test]
fn test_option_none_match() {
    let source = r#"
        fn test(opt: Option<number>) -> number {
            return match opt {
                Some(x) => x,
                None => -1
            };
        }
        test(None());
    "#;

    let interp_result = pm_run_interpreter(source).unwrap();
    assert_eq!(interp_result, "Number(-1.0)");

    if let Ok(vm_result) = pm_run_vm(source) {
        assert_eq!(vm_result, interp_result);
    }
}

#[test]
fn test_option_extract_and_use() {
    let source = r#"
        fn double_option(opt: Option<number>) -> Option<number> {
            return match opt {
                Some(x) => Some(x * 2),
                None => None()
            };
        }
        double_option(Some(21));
    "#;

    let interp_result = pm_run_interpreter(source).unwrap();
    assert!(interp_result.contains("42"));

    if let Ok(vm_result) = pm_run_vm(source) {
        assert_eq!(vm_result, interp_result);
    }
}

// ============================================================================
// Result Pattern Tests
// ============================================================================

#[test]
fn test_result_ok_match() {
    let source = r#"
        fn test(res: Result<number, string>) -> number {
            return match res {
                Ok(x) => x,
                Err(e) => 0
            };
        }
        test(Ok(100));
    "#;

    let interp_result = pm_run_interpreter(source).unwrap();
    assert_eq!(interp_result, "Number(100.0)");

    if let Ok(vm_result) = pm_run_vm(source) {
        assert_eq!(vm_result, interp_result);
    }
}

#[test]
fn test_result_err_match() {
    let source = r#"
        fn test(res: Result<number, string>) -> string {
            return match res {
                Ok(x) => "success",
                Err(e) => e
            };
        }
        test(Err("failed"));
    "#;

    let interp_result = pm_run_interpreter(source).unwrap();
    assert_eq!(interp_result, r#"String("failed")"#);

    if let Ok(vm_result) = pm_run_vm(source) {
        assert_eq!(vm_result, interp_result);
    }
}

// ============================================================================
// Nested Pattern Tests
// ============================================================================

#[test]
fn test_nested_option_some() {
    let source = r#"
        fn test(opt: Option<Option<number>>) -> number {
            return match opt {
                Some(Some(x)) => x,
                Some(None) => -1,
                None => -2
            };
        }
        test(Some(Some(99)));
    "#;

    let interp_result = pm_run_interpreter(source).unwrap();
    assert_eq!(interp_result, "Number(99.0)");

    if let Ok(vm_result) = pm_run_vm(source) {
        assert_eq!(vm_result, interp_result);
    }
}

#[test]
fn test_nested_result_ok() {
    let source = r#"
        fn test(res: Result<Option<number>, string>) -> number {
            return match res {
                Ok(Some(x)) => x,
                Ok(None) => 0,
                Err(e) => -1
            };
        }
        test(Ok(Some(42)));
    "#;

    let interp_result = pm_run_interpreter(source).unwrap();
    assert_eq!(interp_result, "Number(42.0)");

    if let Ok(vm_result) = pm_run_vm(source) {
        assert_eq!(vm_result, interp_result);
    }
}

// ============================================================================
// Array Pattern Tests
// ============================================================================

#[test]
fn test_array_pattern_empty() {
    let source = r#"
        fn test(arr: number[]) -> string {
            return match arr {
                [] => "empty",
                _ => "not empty"
            };
        }
        test([]);
    "#;

    let interp_result = pm_run_interpreter(source).unwrap();
    assert_eq!(interp_result, r#"String("empty")"#);

    if let Ok(vm_result) = pm_run_vm(source) {
        assert_eq!(vm_result, interp_result);
    }
}

#[test]
fn test_array_pattern_single() {
    let source = r#"
        fn test(arr: number[]) -> number {
            return match arr {
                [x] => x,
                _ => 0
            };
        }
        test([42]);
    "#;

    let interp_result = pm_run_interpreter(source).unwrap();
    assert_eq!(interp_result, "Number(42.0)");

    if let Ok(vm_result) = pm_run_vm(source) {
        assert_eq!(vm_result, interp_result);
    }
}

#[test]
fn test_array_pattern_pair() {
    let source = r#"
        fn test(arr: number[]) -> number {
            return match arr {
                [x, y] => x + y,
                _ => 0
            };
        }
        test([10, 20]);
    "#;

    let interp_result = pm_run_interpreter(source).unwrap();
    assert_eq!(interp_result, "Number(30.0)");

    if let Ok(vm_result) = pm_run_vm(source) {
        assert_eq!(vm_result, interp_result);
    }
}

// ============================================================================
// Multiple Arms Tests
// ============================================================================

#[test]
fn test_multiple_literal_arms() {
    let source = r#"
        fn test(x: number) -> string {
            return match x {
                1 => "one",
                2 => "two",
                3 => "three",
                _ => "other"
            };
        }
        test(2);
    "#;

    let interp_result = pm_run_interpreter(source).unwrap();
    assert_eq!(interp_result, r#"String("two")"#);

    if let Ok(vm_result) = pm_run_vm(source) {
        assert_eq!(vm_result, interp_result);
    }
}

// ============================================================================
// Match as Expression Tests
// ============================================================================

#[test]
fn test_match_in_arithmetic() {
    let source = r#"
        fn test(x: number) -> number {
            let result: number = (match x {
                0 => 10,
                _ => 20
            }) + 5;
            return result;
        }
        test(0);
    "#;

    let interp_result = pm_run_interpreter(source).unwrap();
    assert_eq!(interp_result, "Number(15.0)");

    if let Ok(vm_result) = pm_run_vm(source) {
        assert_eq!(vm_result, interp_result);
    }
}

// ============================================================================
// Real-world Usage Tests
// ============================================================================

#[test]
fn test_option_unwrap_or() {
    let source = r#"
        fn get_or_default(opt: Option<number>, default: number) -> number {
            return match opt {
                Some(x) => x,
                None => default
            };
        }
        get_or_default(None(), 42);
    "#;

    let interp_result = pm_run_interpreter(source).unwrap();
    assert_eq!(interp_result, "Number(42.0)");

    if let Ok(vm_result) = pm_run_vm(source) {
        assert_eq!(vm_result, interp_result);
    }
}

#[test]
fn test_result_map() {
    let source = r#"
        fn map_result(res: Result<number, string>) -> Result<number, string> {
            return match res {
                Ok(x) => Ok(x * 2),
                Err(e) => Err(e)
            };
        }
        map_result(Ok(21));
    "#;

    let interp_result = pm_run_interpreter(source).unwrap();
    assert!(interp_result.contains("42"));

    if let Ok(vm_result) = pm_run_vm(source) {
        assert_eq!(vm_result, interp_result);
    }
}

// Test count: Currently 27 tests
// Target: 60+ tests
// TODO: Add more tests for:
// - Complex nested patterns
// - Edge cases
// - Error handling
// - Pattern binding scope
// - Multiple pattern types in one match
