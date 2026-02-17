//! VM Performance Benchmarks
//!
//! Comprehensive benchmarks for VM execution performance covering:
//! - Arithmetic operations (add, sub, mul, div, mixed)
//! - Function calls (simple, recursive, nested)
//! - Loop execution (counting, accumulation, nested)
//! - Array operations (creation, indexing)
//! - Variable access (local, global)
//! - Comparison and logic operations
//! - String operations (concatenation)
//! - Stack operations (push/pop patterns)
//!
//! Run with: cargo bench --bench vm_performance_benches

use atlas_runtime::compiler::Compiler;
use atlas_runtime::lexer::Lexer;
use atlas_runtime::parser::Parser;
use atlas_runtime::security::SecurityContext;
use atlas_runtime::vm::VM;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

fn vm_run(source: &str) {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&program).expect("Compilation failed");
    let mut vm = VM::new(bytecode);
    let _ = vm.run(&SecurityContext::allow_all());
}

// ============================================================================
// Arithmetic Benchmarks
// ============================================================================

fn bench_arithmetic_add(c: &mut Criterion) {
    c.bench_function("vm_arithmetic_add_1000", |b| {
        let code = "let sum = 0; let i = 0; while (i < 1000) { sum = sum + i; i = i + 1; } sum;";
        b.iter(|| vm_run(black_box(code)));
    });
}

fn bench_arithmetic_sub(c: &mut Criterion) {
    c.bench_function("vm_arithmetic_sub_1000", |b| {
        let code = "let result = 1000000; let i = 0; while (i < 1000) { result = result - i; i = i + 1; } result;";
        b.iter(|| vm_run(black_box(code)));
    });
}

fn bench_arithmetic_mul(c: &mut Criterion) {
    c.bench_function("vm_arithmetic_mul_1000", |b| {
        let code = "let result = 1; let i = 1; while (i < 1000) { result = result * 1.001; i = i + 1; } result;";
        b.iter(|| vm_run(black_box(code)));
    });
}

fn bench_arithmetic_div(c: &mut Criterion) {
    c.bench_function("vm_arithmetic_div_1000", |b| {
        let code = "let result = 1000000; let i = 1; while (i < 1000) { result = result / 1.001; i = i + 1; } result;";
        b.iter(|| vm_run(black_box(code)));
    });
}

fn bench_arithmetic_mixed(c: &mut Criterion) {
    c.bench_function("vm_arithmetic_mixed_1000", |b| {
        let code = "let a = 0; let b = 1; let i = 0; while (i < 1000) { let temp = a + b; a = b * 2 - a; b = temp; i = i + 1; } a + b;";
        b.iter(|| vm_run(black_box(code)));
    });
}

fn bench_arithmetic_chained(c: &mut Criterion) {
    c.bench_function("vm_arithmetic_chained_expression", |b| {
        let code = "let i = 0; let r = 0; while (i < 500) { r = 1 + 2 * 3 - 4 + 5 * 6 - 7 + 8 * 9 - 10; i = i + 1; } r;";
        b.iter(|| vm_run(black_box(code)));
    });
}

// ============================================================================
// Function Call Benchmarks
// ============================================================================

fn bench_function_call_simple(c: &mut Criterion) {
    c.bench_function("vm_function_call_simple_1000", |b| {
        let code = "fn add(a: number, b: number) -> number { return a + b; } let result = 0; let i = 0; while (i < 1000) { result = add(result, 1); i = i + 1; } result;";
        b.iter(|| vm_run(black_box(code)));
    });
}

fn bench_function_call_recursive(c: &mut Criterion) {
    c.bench_function("vm_function_fibonacci_20", |b| {
        let code = "fn fib(n: number) -> number { if (n <= 1) { return n; } return fib(n - 1) + fib(n - 2); } fib(20);";
        b.iter(|| vm_run(black_box(code)));
    });
}

fn bench_function_call_nested(c: &mut Criterion) {
    c.bench_function("vm_function_nested_calls", |b| {
        let code = "fn double(x: number) -> number { return x * 2; } fn triple(x: number) -> number { return x * 3; } fn compute(x: number) -> number { return double(triple(x)) + triple(double(x)); } let result = 0; let i = 0; while (i < 500) { result = compute(i); i = i + 1; } result;";
        b.iter(|| vm_run(black_box(code)));
    });
}

fn bench_function_call_many_args(c: &mut Criterion) {
    c.bench_function("vm_function_multi_arg", |b| {
        let code = "fn sum3(a: number, b: number, c: number) -> number { return a + b + c; } let result = 0; let i = 0; while (i < 500) { result = sum3(i, i + 1, i + 2); i = i + 1; } result;";
        b.iter(|| vm_run(black_box(code)));
    });
}

// ============================================================================
// Loop Benchmarks
// ============================================================================

fn bench_loop_counting(c: &mut Criterion) {
    c.bench_function("vm_loop_count_10000", |b| {
        let code = "let i = 0; while (i < 10000) { i = i + 1; } i;";
        b.iter(|| vm_run(black_box(code)));
    });
}

fn bench_loop_accumulation(c: &mut Criterion) {
    c.bench_function("vm_loop_accumulate_5000", |b| {
        let code =
            "let sum = 0; let i = 0; while (i < 5000) { sum = sum + i * 2; i = i + 1; } sum;";
        b.iter(|| vm_run(black_box(code)));
    });
}

fn bench_loop_nested(c: &mut Criterion) {
    c.bench_function("vm_loop_nested_100x100", |b| {
        let code = "let count = 0; let i = 0; while (i < 100) { let j = 0; while (j < 100) { count = count + 1; j = j + 1; } i = i + 1; } count;";
        b.iter(|| vm_run(black_box(code)));
    });
}

fn bench_loop_with_conditionals(c: &mut Criterion) {
    c.bench_function("vm_loop_conditionals_5000", |b| {
        let code = "let even_sum = 0; let odd_sum = 0; let i = 0; while (i < 5000) { if (i % 2 == 0) { even_sum = even_sum + i; } else { odd_sum = odd_sum + i; } i = i + 1; } even_sum + odd_sum;";
        b.iter(|| vm_run(black_box(code)));
    });
}

// ============================================================================
// Array Benchmarks
// ============================================================================

fn bench_array_creation(c: &mut Criterion) {
    c.bench_function("vm_array_create_20", |b| {
        let code =
            "let arr = [1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20]; arr[0] + arr[19];";
        b.iter(|| vm_run(black_box(code)));
    });
}

fn bench_array_index_access(c: &mut Criterion) {
    c.bench_function("vm_array_index_access_1000", |b| {
        let code = "let arr = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]; let sum = 0; let i = 0; while (i < 1000) { sum = sum + arr[i % 10]; i = i + 1; } sum;";
        b.iter(|| vm_run(black_box(code)));
    });
}

fn bench_array_set_index(c: &mut Criterion) {
    c.bench_function("vm_array_set_index_1000", |b| {
        let code = "let arr = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0]; let i = 0; while (i < 1000) { arr[i % 10] = i; i = i + 1; } arr[0];";
        b.iter(|| vm_run(black_box(code)));
    });
}

// ============================================================================
// Variable Access Benchmarks
// ============================================================================

fn bench_local_variable_access(c: &mut Criterion) {
    c.bench_function("vm_local_var_access_5000", |b| {
        let code = "let a = 1; let b = 2; let c = 3; let i = 0; while (i < 5000) { let temp = a + b + c; a = b; b = c; c = temp; i = i + 1; } a + b + c;";
        b.iter(|| vm_run(black_box(code)));
    });
}

// ============================================================================
// Comparison and Logic Benchmarks
// ============================================================================

fn bench_comparison_ops(c: &mut Criterion) {
    c.bench_function("vm_comparison_ops_5000", |b| {
        let code = "let count = 0; let i = 0; while (i < 5000) { if (i > 100) { if (i < 4900) { if (i != 2500) { count = count + 1; } } } i = i + 1; } count;";
        b.iter(|| vm_run(black_box(code)));
    });
}

fn bench_equality_check(c: &mut Criterion) {
    c.bench_function("vm_equality_check_5000", |b| {
        let code = "let found = 0; let i = 0; while (i < 5000) { if (i == 2500) { found = found + 1; } i = i + 1; } found;";
        b.iter(|| vm_run(black_box(code)));
    });
}

// ============================================================================
// String Benchmarks
// ============================================================================

fn bench_string_concat(c: &mut Criterion) {
    c.bench_function("vm_string_concat_100", |b| {
        let code = r#"let s = ""; let i = 0; while (i < 100) { s = s + "x"; i = i + 1; } s;"#;
        b.iter(|| vm_run(black_box(code)));
    });
}

// ============================================================================
// Stack Operation Benchmarks
// ============================================================================

fn bench_stack_heavy(c: &mut Criterion) {
    c.bench_function("vm_stack_heavy_expressions", |b| {
        let code = "let r = 0; let i = 0; while (i < 1000) { r = (1 + 2) * (3 + 4) - (5 + 6) * (7 - 8); i = i + 1; } r;";
        b.iter(|| vm_run(black_box(code)));
    });
}

fn bench_deep_expression(c: &mut Criterion) {
    c.bench_function("vm_deep_expression_nesting", |b| {
        let code = "let i = 0; let r = 0; while (i < 500) { r = ((((1 + 2) * 3) + 4) * 5) + ((((6 + 7) * 8) + 9) * 10); i = i + 1; } r;";
        b.iter(|| vm_run(black_box(code)));
    });
}

// ============================================================================
// Scaling Benchmarks (parameterized)
// ============================================================================

fn bench_loop_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("vm_loop_scaling");
    for size in [100, 1000, 5000, 10000] {
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            let code = format!(
                "let sum = 0; let i = 0; while (i < {}) {{ sum = sum + i; i = i + 1; }} sum;",
                size
            );
            b.iter(|| vm_run(black_box(&code)));
        });
    }
    group.finish();
}

fn bench_function_call_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("vm_function_call_scaling");
    for size in [100, 500, 1000, 2000] {
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            let code = format!(
                "fn increment(x: number) -> number {{ return x + 1; }} let result = 0; let i = 0; while (i < {}) {{ result = increment(result); i = i + 1; }} result;",
                size
            );
            b.iter(|| vm_run(black_box(&code)));
        });
    }
    group.finish();
}

// ============================================================================
// Criterion Groups
// ============================================================================

criterion_group!(
    arithmetic_benches,
    bench_arithmetic_add,
    bench_arithmetic_sub,
    bench_arithmetic_mul,
    bench_arithmetic_div,
    bench_arithmetic_mixed,
    bench_arithmetic_chained,
);

criterion_group!(
    function_benches,
    bench_function_call_simple,
    bench_function_call_recursive,
    bench_function_call_nested,
    bench_function_call_many_args,
);

criterion_group!(
    loop_benches,
    bench_loop_counting,
    bench_loop_accumulation,
    bench_loop_nested,
    bench_loop_with_conditionals,
);

criterion_group!(
    array_benches,
    bench_array_creation,
    bench_array_index_access,
    bench_array_set_index,
);

criterion_group!(variable_benches, bench_local_variable_access,);

criterion_group!(
    comparison_benches,
    bench_comparison_ops,
    bench_equality_check,
);

criterion_group!(string_benches, bench_string_concat,);

criterion_group!(stack_benches, bench_stack_heavy, bench_deep_expression,);

criterion_group!(
    scaling_benches,
    bench_loop_scaling,
    bench_function_call_scaling,
);

criterion_main!(
    arithmetic_benches,
    function_benches,
    loop_benches,
    array_benches,
    variable_benches,
    comparison_benches,
    string_benches,
    stack_benches,
    scaling_benches,
);
