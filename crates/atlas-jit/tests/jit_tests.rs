//! Comprehensive JIT integration tests
//!
//! Tests the full pipeline: bytecode → IR translation → native compilation → execution
//! Verifies JIT results match interpreter output for all supported operations.

use atlas_jit::backend::NativeBackend;
use atlas_jit::cache::CodeCache;
use atlas_jit::codegen::IrTranslator;
use atlas_jit::hotspot::HotspotTracker;
use atlas_jit::{JitConfig, JitEngine, JitError};
use atlas_runtime::bytecode::{Bytecode, Opcode};
use atlas_runtime::span::Span;
use atlas_runtime::value::Value;
use rstest::rstest;

fn dummy() -> Span {
    Span::dummy()
}

/// Build bytecode for a numeric expression and return the bytecode
fn num_bc(value: f64) -> Bytecode {
    let mut bc = Bytecode::new();
    let idx = bc.add_constant(Value::Number(value));
    bc.emit(Opcode::Constant, dummy());
    bc.emit_u16(idx);
    bc.emit(Opcode::Return, dummy());
    bc
}

/// Build bytecode for a binary operation on two constants
fn binop_bc(a: f64, b: f64, op: Opcode) -> Bytecode {
    let mut bc = Bytecode::new();
    let ia = bc.add_constant(Value::Number(a));
    let ib = bc.add_constant(Value::Number(b));
    bc.emit(Opcode::Constant, dummy());
    bc.emit_u16(ia);
    bc.emit(Opcode::Constant, dummy());
    bc.emit_u16(ib);
    bc.emit(op, dummy());
    bc.emit(Opcode::Return, dummy());
    bc
}

/// Full pipeline: translate → compile → execute → return f64
fn jit_eval(bc: &Bytecode) -> f64 {
    let translator = IrTranslator::new(0);
    let func = translator.translate(bc, 0, bc.instructions.len()).unwrap();
    let mut backend = NativeBackend::new(0).unwrap();
    let compiled = backend.compile(func).unwrap();
    unsafe { compiled.call_no_args() }
}

// =============================================================================
// Arithmetic tests
// =============================================================================

#[rstest]
#[case(10.0, 20.0, 30.0)]
#[case(0.0, 0.0, 0.0)]
#[case(-5.0, 5.0, 0.0)]
#[case(1.5, 2.5, 4.0)]
#[case(f64::MAX / 2.0, 1.0, f64::MAX / 2.0 + 1.0)]
fn test_jit_add(#[case] a: f64, #[case] b: f64, #[case] expected: f64) {
    let bc = binop_bc(a, b, Opcode::Add);
    assert_eq!(jit_eval(&bc), expected);
}

#[rstest]
#[case(20.0, 10.0, 10.0)]
#[case(0.0, 5.0, -5.0)]
#[case(-3.0, -7.0, 4.0)]
fn test_jit_sub(#[case] a: f64, #[case] b: f64, #[case] expected: f64) {
    let bc = binop_bc(a, b, Opcode::Sub);
    assert_eq!(jit_eval(&bc), expected);
}

#[rstest]
#[case(3.0, 4.0, 12.0)]
#[case(0.0, 100.0, 0.0)]
#[case(-2.0, 3.0, -6.0)]
#[case(1.5, 2.0, 3.0)]
fn test_jit_mul(#[case] a: f64, #[case] b: f64, #[case] expected: f64) {
    let bc = binop_bc(a, b, Opcode::Mul);
    assert_eq!(jit_eval(&bc), expected);
}

#[rstest]
#[case(10.0, 2.0, 5.0)]
#[case(7.0, 2.0, 3.5)]
#[case(-9.0, 3.0, -3.0)]
fn test_jit_div(#[case] a: f64, #[case] b: f64, #[case] expected: f64) {
    let bc = binop_bc(a, b, Opcode::Div);
    assert_eq!(jit_eval(&bc), expected);
}

#[rstest]
#[case(10.0, 3.0, 1.0)]
#[case(7.0, 2.0, 1.0)]
#[case(9.0, 4.0, 1.0)]
fn test_jit_mod(#[case] a: f64, #[case] b: f64, #[case] expected: f64) {
    let bc = binop_bc(a, b, Opcode::Mod);
    assert_eq!(jit_eval(&bc), expected);
}

#[test]
fn test_jit_negate() {
    let mut bc = Bytecode::new();
    let idx = bc.add_constant(Value::Number(42.0));
    bc.emit(Opcode::Constant, dummy());
    bc.emit_u16(idx);
    bc.emit(Opcode::Negate, dummy());
    bc.emit(Opcode::Return, dummy());
    assert_eq!(jit_eval(&bc), -42.0);
}

#[test]
fn test_jit_negate_zero() {
    let mut bc = Bytecode::new();
    let idx = bc.add_constant(Value::Number(0.0));
    bc.emit(Opcode::Constant, dummy());
    bc.emit_u16(idx);
    bc.emit(Opcode::Negate, dummy());
    bc.emit(Opcode::Return, dummy());
    // -0.0 == 0.0 in f64
    assert_eq!(jit_eval(&bc), 0.0);
}

// =============================================================================
// Comparison tests
// =============================================================================

#[rstest]
#[case(1.0, 2.0, Opcode::Less, 1.0)]
#[case(2.0, 1.0, Opcode::Less, 0.0)]
#[case(1.0, 1.0, Opcode::Less, 0.0)]
#[case(1.0, 2.0, Opcode::LessEqual, 1.0)]
#[case(1.0, 1.0, Opcode::LessEqual, 1.0)]
#[case(2.0, 1.0, Opcode::LessEqual, 0.0)]
#[case(2.0, 1.0, Opcode::Greater, 1.0)]
#[case(1.0, 2.0, Opcode::Greater, 0.0)]
#[case(2.0, 1.0, Opcode::GreaterEqual, 1.0)]
#[case(1.0, 1.0, Opcode::GreaterEqual, 1.0)]
#[case(1.0, 1.0, Opcode::Equal, 1.0)]
#[case(1.0, 2.0, Opcode::Equal, 0.0)]
#[case(1.0, 2.0, Opcode::NotEqual, 1.0)]
#[case(1.0, 1.0, Opcode::NotEqual, 0.0)]
fn test_jit_comparison(#[case] a: f64, #[case] b: f64, #[case] op: Opcode, #[case] expected: f64) {
    let bc = binop_bc(a, b, op);
    assert_eq!(jit_eval(&bc), expected);
}

// =============================================================================
// Logical tests
// =============================================================================

#[test]
fn test_jit_not_true() {
    let mut bc = Bytecode::new();
    bc.emit(Opcode::True, dummy());
    bc.emit(Opcode::Not, dummy());
    bc.emit(Opcode::Return, dummy());
    assert_eq!(jit_eval(&bc), 0.0);
}

#[test]
fn test_jit_not_false() {
    let mut bc = Bytecode::new();
    bc.emit(Opcode::False, dummy());
    bc.emit(Opcode::Not, dummy());
    bc.emit(Opcode::Return, dummy());
    assert_eq!(jit_eval(&bc), 1.0);
}

// =============================================================================
// Local variable tests
// =============================================================================

#[test]
fn test_jit_local_get_set() {
    let mut bc = Bytecode::new();
    let idx = bc.add_constant(Value::Number(99.0));
    bc.emit(Opcode::Constant, dummy());
    bc.emit_u16(idx);
    bc.emit(Opcode::SetLocal, dummy());
    bc.emit_u16(0);
    bc.emit(Opcode::GetLocal, dummy());
    bc.emit_u16(0);
    bc.emit(Opcode::Return, dummy());

    let translator = IrTranslator::new(0);
    let func = translator
        .translate_with_params(&bc, 0, bc.instructions.len(), 1)
        .unwrap();
    let mut backend = NativeBackend::new(0).unwrap();
    let compiled = backend.compile(func).unwrap();
    // The function takes 1 param but we set local 0 to 99.0
    let result = unsafe { compiled.call_1arg(0.0) };
    assert_eq!(result, 99.0);
}

// =============================================================================
// Stack manipulation tests
// =============================================================================

#[test]
fn test_jit_dup() {
    let mut bc = Bytecode::new();
    let idx = bc.add_constant(Value::Number(21.0));
    bc.emit(Opcode::Constant, dummy());
    bc.emit_u16(idx);
    bc.emit(Opcode::Dup, dummy());
    bc.emit(Opcode::Add, dummy());
    bc.emit(Opcode::Return, dummy());
    assert_eq!(jit_eval(&bc), 42.0);
}

#[test]
fn test_jit_pop() {
    let mut bc = Bytecode::new();
    let a = bc.add_constant(Value::Number(100.0));
    let b = bc.add_constant(Value::Number(42.0));
    bc.emit(Opcode::Constant, dummy());
    bc.emit_u16(a);
    bc.emit(Opcode::Constant, dummy());
    bc.emit_u16(b);
    bc.emit(Opcode::Pop, dummy()); // discard 42
    bc.emit(Opcode::Return, dummy());
    assert_eq!(jit_eval(&bc), 100.0);
}

// =============================================================================
// Complex expression tests
// =============================================================================

#[test]
fn test_jit_compound_arithmetic() {
    // (3 + 4) * (10 - 5) = 35
    let mut bc = Bytecode::new();
    let c3 = bc.add_constant(Value::Number(3.0));
    let c4 = bc.add_constant(Value::Number(4.0));
    let c10 = bc.add_constant(Value::Number(10.0));
    let c5 = bc.add_constant(Value::Number(5.0));
    bc.emit(Opcode::Constant, dummy());
    bc.emit_u16(c3);
    bc.emit(Opcode::Constant, dummy());
    bc.emit_u16(c4);
    bc.emit(Opcode::Add, dummy());
    bc.emit(Opcode::Constant, dummy());
    bc.emit_u16(c10);
    bc.emit(Opcode::Constant, dummy());
    bc.emit_u16(c5);
    bc.emit(Opcode::Sub, dummy());
    bc.emit(Opcode::Mul, dummy());
    bc.emit(Opcode::Return, dummy());
    assert_eq!(jit_eval(&bc), 35.0);
}

#[test]
fn test_jit_nested_negation() {
    // --42 = 42
    let mut bc = Bytecode::new();
    let idx = bc.add_constant(Value::Number(42.0));
    bc.emit(Opcode::Constant, dummy());
    bc.emit_u16(idx);
    bc.emit(Opcode::Negate, dummy());
    bc.emit(Opcode::Negate, dummy());
    bc.emit(Opcode::Return, dummy());
    assert_eq!(jit_eval(&bc), 42.0);
}

#[test]
fn test_jit_chain_operations() {
    // 1 + 2 + 3 + 4 + 5 = 15
    let mut bc = Bytecode::new();
    for i in 1..=5 {
        let idx = bc.add_constant(Value::Number(i as f64));
        bc.emit(Opcode::Constant, dummy());
        bc.emit_u16(idx);
        if i > 1 {
            bc.emit(Opcode::Add, dummy());
        }
    }
    bc.emit(Opcode::Return, dummy());
    assert_eq!(jit_eval(&bc), 15.0);
}

// =============================================================================
// Error handling tests
// =============================================================================

#[test]
fn test_jit_unsupported_opcode() {
    let mut bc = Bytecode::new();
    bc.emit(Opcode::GetGlobal, dummy());
    bc.emit_u16(0);

    let translator = IrTranslator::new(0);
    let result = translator.translate(&bc, 0, bc.instructions.len());
    assert!(result.is_err());
    match result.unwrap_err() {
        JitError::UnsupportedOpcode(Opcode::GetGlobal) => {}
        e => panic!("expected UnsupportedOpcode, got {:?}", e),
    }
}

#[test]
fn test_jit_stack_underflow() {
    let mut bc = Bytecode::new();
    bc.emit(Opcode::Add, dummy());

    let translator = IrTranslator::new(0);
    let result = translator.translate(&bc, 0, bc.instructions.len());
    assert!(result.is_err());
}

#[test]
fn test_jit_non_numeric_constant() {
    let mut bc = Bytecode::new();
    let idx = bc.add_constant(Value::String(std::sync::Arc::new("hello".to_string())));
    bc.emit(Opcode::Constant, dummy());
    bc.emit_u16(idx);

    let translator = IrTranslator::new(0);
    let result = translator.translate(&bc, 0, bc.instructions.len());
    assert!(result.is_err());
}

// =============================================================================
// Hotspot tracker integration tests
// =============================================================================

#[test]
fn test_hotspot_full_workflow() {
    let mut tracker = HotspotTracker::new(5);

    // Simulate function calls
    for _ in 0..10 {
        tracker.record_call(100);
    }
    for _ in 0..3 {
        tracker.record_call(200);
    }

    // Function at 100 is hot, 200 is not
    assert!(tracker.is_hot(100));
    assert!(!tracker.is_hot(200));

    let pending = tracker.pending_compilations();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].offset, 100);

    // Compile and mark
    tracker.mark_compiled(100);
    assert!(!tracker.is_hot(100));
    assert!(tracker.is_compiled(100));
    assert!(tracker.pending_compilations().is_empty());
}

#[test]
fn test_hotspot_threshold_update() {
    let mut tracker = HotspotTracker::new(10);
    for _ in 0..8 {
        tracker.record_call(50);
    }
    assert!(!tracker.is_hot(50));

    tracker.set_threshold(5);
    assert!(tracker.is_hot(50));
}

// =============================================================================
// Code cache integration tests
// =============================================================================

#[test]
fn test_cache_with_real_compiled_code() {
    let bc = num_bc(42.0);
    let translator = IrTranslator::new(0);
    let func = translator.translate(&bc, 0, bc.instructions.len()).unwrap();
    let mut backend = NativeBackend::new(0).unwrap();
    let compiled = backend.compile(func).unwrap();

    let mut cache = CodeCache::new(1024 * 1024);
    cache.insert(0, compiled.code_ptr, 64, 0).unwrap();

    assert!(cache.contains(0));
    let entry = cache.get(0).unwrap();
    let result = unsafe {
        let func: unsafe fn() -> f64 = std::mem::transmute(entry.code_ptr);
        func()
    };
    assert_eq!(result, 42.0);
}

#[test]
fn test_cache_eviction_preserves_hot() {
    let mut cache = CodeCache::new(192);
    let fake = 0x1000 as *const u8;

    cache.insert(1, fake, 64, 0).unwrap();
    cache.insert(2, fake, 64, 0).unwrap();
    cache.insert(3, fake, 64, 0).unwrap();

    // Make entry 2 hot
    cache.get(2);
    cache.get(2);
    cache.get(2);

    // Insert new entry — should evict 1 or 3, not 2
    cache.insert(4, fake, 64, 0).unwrap();
    assert!(cache.contains(2)); // preserved because hot
}

// =============================================================================
// Backend tests
// =============================================================================

#[test]
fn test_backend_multiple_functions() {
    let mut backend = NativeBackend::new(0).unwrap();

    // Compile several functions
    for val in [1.0, 2.0, 3.0, 4.0, 5.0] {
        let bc = num_bc(val);
        let translator = IrTranslator::new(0);
        let func = translator.translate(&bc, 0, bc.instructions.len()).unwrap();
        let compiled = backend.compile(func).unwrap();
        let result = unsafe { compiled.call_no_args() };
        assert_eq!(result, val);
    }
    assert_eq!(backend.compiled_count(), 5);
}

#[test]
fn test_backend_target_arch() {
    let backend = NativeBackend::new(0).unwrap();
    let arch = backend.target_arch();
    assert!(["x86_64", "aarch64"].contains(&arch));
}

#[test]
fn test_backend_optimization_levels() {
    for level in [0, 1, 2] {
        let backend = NativeBackend::new(level);
        assert!(backend.is_ok(), "opt_level {} should work", level);
    }
}

// =============================================================================
// Config tests
// =============================================================================

#[test]
fn test_config_default() {
    let config = JitConfig::default();
    assert!(config.enabled);
    assert_eq!(config.compilation_threshold, 100);
    assert_eq!(config.cache_size_limit, 64 * 1024 * 1024);
    assert_eq!(config.opt_level, 1);
}

#[test]
fn test_config_testing() {
    let config = JitConfig::for_testing();
    assert!(config.enabled);
    assert_eq!(config.compilation_threshold, 2);
}

// =============================================================================
// Parity tests: JIT vs expected values
// =============================================================================

#[rstest]
#[case(0.0)]
#[case(1.0)]
#[case(-1.0)]
#[case(42.0)]
#[case(std::f64::consts::PI)]
#[case(f64::MIN_POSITIVE)]
fn test_jit_constant_parity(#[case] value: f64) {
    let bc = num_bc(value);
    assert_eq!(jit_eval(&bc), value);
}

#[test]
fn test_jit_division_by_zero() {
    let bc = binop_bc(1.0, 0.0, Opcode::Div);
    let result = jit_eval(&bc);
    assert!(result.is_infinite());
}

#[test]
fn test_jit_null_as_zero() {
    let mut bc = Bytecode::new();
    bc.emit(Opcode::Null, dummy());
    bc.emit(Opcode::Return, dummy());
    assert_eq!(jit_eval(&bc), 0.0);
}

#[test]
fn test_jit_true_as_one() {
    let mut bc = Bytecode::new();
    bc.emit(Opcode::True, dummy());
    bc.emit(Opcode::Return, dummy());
    assert_eq!(jit_eval(&bc), 1.0);
}

#[test]
fn test_jit_false_as_zero() {
    let mut bc = Bytecode::new();
    bc.emit(Opcode::False, dummy());
    bc.emit(Opcode::Return, dummy());
    assert_eq!(jit_eval(&bc), 0.0);
}

// =============================================================================
// Performance measurement test
// =============================================================================

#[test]
fn test_jit_performance_improvement() {
    // Compile a simple arithmetic expression and measure execution time
    let bc = binop_bc(std::f64::consts::PI, std::f64::consts::E, Opcode::Mul);
    let translator = IrTranslator::new(1);
    let func = translator.translate(&bc, 0, bc.instructions.len()).unwrap();
    let mut backend = NativeBackend::new(1).unwrap();
    let compiled = backend.compile(func).unwrap();

    // Execute JIT-compiled function many times
    let start = std::time::Instant::now();
    let mut sum = 0.0;
    for _ in 0..1_000_000 {
        sum += unsafe { compiled.call_no_args() };
    }
    let jit_elapsed = start.elapsed();

    // Verify result is correct
    assert!((sum - std::f64::consts::PI * std::f64::consts::E * 1_000_000.0).abs() < 1.0);

    // JIT should be very fast for pure arithmetic
    assert!(
        jit_elapsed.as_millis() < 500,
        "JIT execution took too long: {:?}",
        jit_elapsed
    );
}

// =============================================================================
// End-to-end pipeline test
// =============================================================================

#[test]
fn test_full_jit_pipeline() {
    let config = JitConfig::for_testing();
    let mut tracker = HotspotTracker::new(config.compilation_threshold);
    let mut cache = CodeCache::new(config.cache_size_limit);

    // Simulate: function at offset 0 called many times
    let bc = binop_bc(6.0, 7.0, Opcode::Mul);

    for _ in 0..config.compilation_threshold {
        tracker.record_call(0);
    }

    // Check it's hot
    assert!(tracker.is_hot(0));

    // Compile
    let translator = IrTranslator::new(config.opt_level);
    let func = translator.translate(&bc, 0, bc.instructions.len()).unwrap();
    let mut backend = NativeBackend::new(config.opt_level).unwrap();
    let compiled = backend.compile(func).unwrap();

    // Cache
    cache.insert(0, compiled.code_ptr, 64, 0).unwrap();
    tracker.mark_compiled(0);

    // Execute from cache
    let entry = cache.get(0).unwrap();
    let result = unsafe {
        let func: unsafe fn() -> f64 = std::mem::transmute(entry.code_ptr);
        func()
    };
    assert_eq!(result, 42.0);

    // Verify tracking
    assert!(tracker.is_compiled(0));
    assert!(!tracker.is_hot(0));
    assert_eq!(cache.hits(), 1);
}

#[test]
fn test_halt_opcode() {
    let mut bc = Bytecode::new();
    let idx = bc.add_constant(Value::Number(77.0));
    bc.emit(Opcode::Constant, dummy());
    bc.emit_u16(idx);
    bc.emit(Opcode::Halt, dummy());
    // Extra instructions after halt should be ignored
    bc.emit(Opcode::Negate, dummy());
    assert_eq!(jit_eval(&bc), 77.0);
}

#[test]
fn test_empty_bytecode() {
    let bc = Bytecode::new();
    // Empty bytecode should return 0.0
    let translator = IrTranslator::new(0);
    let func = translator.translate(&bc, 0, 0).unwrap();
    let mut backend = NativeBackend::new(0).unwrap();
    let compiled = backend.compile(func).unwrap();
    assert_eq!(unsafe { compiled.call_no_args() }, 0.0);
}

// =============================================================================
// JitEngine integration tests
// =============================================================================

#[test]
fn test_engine_creation() {
    let engine = JitEngine::new(JitConfig::for_testing());
    assert!(engine.is_ok());
    let engine = engine.unwrap();
    assert!(engine.is_enabled());
    assert_eq!(engine.threshold(), 2);
}

#[test]
fn test_engine_enable_disable() {
    let mut engine = JitEngine::new(JitConfig::for_testing()).unwrap();
    assert!(engine.is_enabled());
    engine.disable();
    assert!(!engine.is_enabled());
    engine.enable();
    assert!(engine.is_enabled());
}

#[test]
fn test_engine_disabled_returns_none() {
    let mut engine = JitEngine::new(JitConfig::for_testing()).unwrap();
    engine.disable();
    let bc = num_bc(42.0);
    let result = engine.notify_call(0, &bc, bc.instructions.len());
    assert!(result.is_none());
}

#[test]
fn test_engine_below_threshold_returns_none() {
    let mut engine = JitEngine::new(JitConfig::for_testing()).unwrap();
    let bc = num_bc(42.0);
    // Threshold is 2, call only once
    let result = engine.notify_call(0, &bc, bc.instructions.len());
    assert!(result.is_none());
}

#[test]
fn test_engine_compiles_after_threshold() {
    let mut engine = JitEngine::new(JitConfig::for_testing()).unwrap();
    let bc = num_bc(42.0);
    let end = bc.instructions.len();

    // First call: below threshold
    assert!(engine.notify_call(0, &bc, end).is_none());
    // Second call: at threshold (2), should compile and execute
    let result = engine.notify_call(0, &bc, end);
    assert_eq!(result, Some(42.0));

    let stats = engine.stats();
    assert_eq!(stats.compilations, 1);
    assert_eq!(stats.jit_executions, 1);
}

#[test]
fn test_engine_cache_hit() {
    let mut engine = JitEngine::new(JitConfig::for_testing()).unwrap();
    let bc = num_bc(42.0);
    let end = bc.instructions.len();

    // Warm up
    engine.notify_call(0, &bc, end);
    engine.notify_call(0, &bc, end); // compiles

    // Third call should be a cache hit
    let result = engine.notify_call(0, &bc, end);
    assert_eq!(result, Some(42.0));

    let stats = engine.stats();
    assert_eq!(stats.compilations, 1); // only compiled once
    assert_eq!(stats.jit_executions, 2); // executed twice via JIT
}

#[test]
fn test_engine_stats() {
    let engine = JitEngine::new(JitConfig::for_testing()).unwrap();
    let stats = engine.stats();
    assert_eq!(stats.compilations, 0);
    assert_eq!(stats.jit_executions, 0);
    assert_eq!(stats.interpreter_fallbacks, 0);
    assert_eq!(stats.cached_functions, 0);
    assert_eq!(stats.tracked_functions, 0);
}

#[test]
fn test_engine_reset() {
    let mut engine = JitEngine::new(JitConfig::for_testing()).unwrap();
    let bc = num_bc(42.0);
    let end = bc.instructions.len();
    engine.notify_call(0, &bc, end);
    engine.notify_call(0, &bc, end);

    engine.reset();
    let stats = engine.stats();
    assert_eq!(stats.compilations, 0);
    assert_eq!(stats.jit_executions, 0);
    assert_eq!(stats.cached_functions, 0);
}

#[test]
fn test_engine_invalidate_cache() {
    let mut engine = JitEngine::new(JitConfig::for_testing()).unwrap();
    let bc = num_bc(42.0);
    let end = bc.instructions.len();
    engine.notify_call(0, &bc, end);
    engine.notify_call(0, &bc, end); // compile

    engine.invalidate_cache();
    // Cache invalidated but tracker still knows it was compiled,
    // so it won't recompile
    let stats = engine.stats();
    assert_eq!(stats.compilations, 1);
}

#[test]
fn test_engine_unsupported_fallback() {
    let mut engine = JitEngine::new(JitConfig::for_testing()).unwrap();
    // Bytecode with unsupported opcode
    let mut bc = Bytecode::new();
    bc.emit(Opcode::GetGlobal, dummy());
    bc.emit_u16(0);
    let end = bc.instructions.len();

    engine.notify_call(0, &bc, end);
    let result = engine.notify_call(0, &bc, end); // tries to compile, fails
    assert!(result.is_none());

    let stats = engine.stats();
    assert_eq!(stats.interpreter_fallbacks, 1);
}

#[test]
fn test_engine_multiple_functions() {
    // Build a single bytecode with two "functions" at different offsets
    let mut bc = Bytecode::new();
    // Function 1 at offset 0: returns 10.0
    let c10 = bc.add_constant(Value::Number(10.0));
    let fn1_start = bc.instructions.len();
    bc.emit(Opcode::Constant, dummy());
    bc.emit_u16(c10);
    bc.emit(Opcode::Return, dummy());
    let fn1_end = bc.instructions.len();

    // Function 2 at a later offset: returns 20.0
    let c20 = bc.add_constant(Value::Number(20.0));
    let fn2_start = bc.instructions.len();
    bc.emit(Opcode::Constant, dummy());
    bc.emit_u16(c20);
    bc.emit(Opcode::Return, dummy());
    let fn2_end = bc.instructions.len();

    let mut engine = JitEngine::new(JitConfig::for_testing()).unwrap();

    // Warm up both
    engine.notify_call(fn1_start, &bc, fn1_end);
    engine.notify_call(fn2_start, &bc, fn2_end);

    // Compile both
    assert_eq!(engine.notify_call(fn1_start, &bc, fn1_end), Some(10.0));
    assert_eq!(engine.notify_call(fn2_start, &bc, fn2_end), Some(20.0));

    let stats = engine.stats();
    assert_eq!(stats.compilations, 2);
}
