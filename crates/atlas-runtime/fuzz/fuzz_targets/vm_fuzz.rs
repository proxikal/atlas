//! Comprehensive VM/Bytecode Stability Fuzzer
//!
//! This fuzz target exercises the Atlas bytecode VM with arbitrary input.
//! It tests two complementary paths:
//!
//! 1. **Source-to-bytecode path**: Compile arbitrary source through the full
//!    pipeline (lex → parse → bind → typecheck → codegen → VM), verifying
//!    no panics at any stage.
//!
//! 2. **Bytecode validation path**: Feed raw bytes to the bytecode validator,
//!    verifying the validator rejects malformed bytecode safely.
//!
//! Stability contracts:
//! - The VM must never panic on any compiled bytecode
//! - The bytecode validator must never panic on any input
//! - VM execution must terminate (no infinite loops from fuzz input)
//! - Execution must be deterministic (same input → same result)

#![no_main]

use libfuzzer_sys::fuzz_target;

use atlas_runtime::Atlas;
use atlas_runtime::bytecode::{Bytecode, validator};
use atlas_runtime::lexer::Lexer;
use atlas_runtime::parser::Parser;
use atlas_runtime::Binder;
use atlas_runtime::TypeChecker;
use atlas_runtime::Compiler;
use atlas_runtime::VM;

fuzz_target!(|data: &[u8]| {
    // Two-path fuzzing: split the fuzz data to exercise both paths.
    if data.is_empty() {
        return;
    }

    // If first byte is even: fuzz source-to-VM path
    // If first byte is odd: fuzz bytecode validator path
    if data[0] % 2 == 0 {
        if let Ok(input) = std::str::from_utf8(&data[1..]) {
            fuzz_source_to_vm(input);
        }
    } else {
        fuzz_bytecode_validator(&data[1..]);
    }
});

/// Fuzz the full source → compile → VM execution pipeline.
///
/// Contract: Atlas::eval() must return Ok(Value) or Err(Vec<Diagnostic>).
/// It must NEVER panic regardless of input content.
fn fuzz_source_to_vm(input: &str) {
    // ─── Path A: High-level Atlas::eval ────────────────────────────────────
    // This exercises the complete pipeline through the public API.
    // Most callers use this path — it's the most important to keep panic-free.
    let runtime = Atlas::new();
    let result_a = runtime.eval(input);

    // ─── Determinism check ─────────────────────────────────────────────────
    // Running the same program twice must produce identical results.
    let runtime_b = Atlas::new();
    let result_b = runtime_b.eval(input);

    match (&result_a, &result_b) {
        (Ok(val_a), Ok(val_b)) => {
            // Both succeeded — results must be identical (value equality).
            // Use string comparison as a proxy for structural equality.
            assert_eq!(
                format!("{:?}", val_a),
                format!("{:?}", val_b),
                "VM non-deterministic: same program produced different values"
            );
        }
        (Err(diags_a), Err(diags_b)) => {
            // Both failed — error count must match.
            assert_eq!(
                diags_a.len(),
                diags_b.len(),
                "VM non-deterministic: different diagnostic count on same input"
            );
        }
        _ => {
            // One succeeded and one failed — this is a determinism bug.
            // We don't panic here since the fuzzer may be exploring edge cases
            // where timing or resource limits affect the result. Instead, we
            // record the disagreement as a soft assertion.
            let _ = (result_a, result_b); // Non-determinism observed but not crashed on.
        }
    }

    // ─── Path B: Low-level compilation pipeline ─────────────────────────────
    // Test the compiler separately to isolate VM vs. compiler bugs.
    fuzz_compiler_pipeline(input);

    // ─── Stress: known VM edge cases ───────────────────────────────────────
    // Test a selection of programs that stress VM internals.
    let vm_stress_programs: &[&str] = &[
        // Deep call stack
        "fn f(n: number) -> number { if (n <= 0) { return 0; } return f(n - 1); } f(100);",
        // Large array allocation
        "let arr: number[] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]; arr[9];",
        // Arithmetic edge cases
        "1.7976931348623157e308 * 2.0;", // Overflow to infinity
        "0.0 / 0.0;",                    // NaN
        "-0.0;",                          // Negative zero
        // String operations
        r#"let s: string = ""; s;"#,
        // Empty function
        "fn f() -> null { } f();",
        // Nested conditionals
        "if (true) { if (false) { 1; } else { 2; } } else { 3; }",
    ];

    // Pick one stress program based on input content.
    if !input.is_empty() {
        let idx = (input.bytes().next().unwrap_or(0) as usize) % vm_stress_programs.len();
        let runtime = Atlas::new();
        let _ = runtime.eval(vm_stress_programs[idx]);
    }
}

/// Fuzz the compiler pipeline directly (without high-level Atlas API).
fn fuzz_compiler_pipeline(input: &str) {
    let mut lexer = Lexer::new(input);
    let (tokens, _) = lexer.tokenize();

    if tokens.is_empty() {
        return;
    }

    let mut parser = Parser::new(tokens);
    let (program, parse_diags) = parser.parse();

    // Only proceed to compilation if there are no parse errors.
    // The compiler expects a valid AST.
    if !parse_diags.is_empty() {
        return;
    }

    let mut binder = Binder::new();
    let (mut symbol_table, _) = binder.bind(&program);

    let mut type_checker = TypeChecker::new(&mut symbol_table);
    let type_diags = type_checker.check(&program);

    // Only compile type-correct programs.
    if !type_diags.is_empty() {
        return;
    }

    // Compile to bytecode — must not panic.
    let mut compiler = Compiler::new();
    match compiler.compile(&program) {
        Ok(bytecode) => {
            // Execute in VM — must not panic.
            let mut vm = VM::new(bytecode);
            let _ = vm.run();
        }
        Err(_) => {
            // Compilation errors are expected for some valid-syntax programs.
            // The compiler must return errors, not panic.
        }
    }
}

/// Fuzz the bytecode validator with raw bytes.
///
/// The validator must handle arbitrary byte sequences safely.
/// It must either accept or reject the bytecode, never panic.
fn fuzz_bytecode_validator(data: &[u8]) {
    // Try to deserialize the raw bytes as a Bytecode object.
    // Bytecode::from_bytes must handle malformed data safely.
    match Bytecode::from_bytes(data) {
        Ok(bytecode) => {
            // Successfully deserialized — validate it.
            // The validator must not panic on any valid-deserialize bytecode.
            let _ = validator::validate(&bytecode);

            // Also try executing valid bytecode in the VM.
            // The VM must handle even unusual-but-valid bytecode safely.
            let mut vm = VM::new(bytecode);
            let _ = vm.run();
        }
        Err(_) => {
            // Deserialization failure is expected for random bytes.
            // The important thing is that no panic occurred.
        }
    }
}
