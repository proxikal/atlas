//! Optimizer integration tests
//!
//! Tests that optimized programs produce identical results to unoptimized ones
//! across a wide range of programs.

use atlas_runtime::{
    bytecode::Bytecode, compiler::Compiler, lexer::Lexer, optimizer::Optimizer, parser::Parser,
    security::SecurityContext, value::Value, vm::VM,
};

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
fn test_multiple_functions() {
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
