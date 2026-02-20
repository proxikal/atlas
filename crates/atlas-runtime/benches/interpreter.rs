//! Interpreter execution benchmarks
//!
//! Benchmarks the tree-walking interpreter on canonical programs
//! that stress different execution paths. Measures:
//! - Arithmetic and loop performance
//! - Function call overhead
//! - Variable lookup speed
//! - Collection operations
//! - Recursion depth
//! - Scope depth impact

use atlas_runtime::{Interpreter, Lexer, Parser, SecurityContext};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

/// Run interpreter on source code
fn interp_run(source: &str) {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();
    let security = SecurityContext::allow_all();
    let mut interp = Interpreter::new();
    let _ = interp.eval(&program, &security);
}

/// Parse source code (for measuring parse vs execution time)
fn parse_only(source: &str) {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let _ = parser.parse();
}

// ============================================================================
// Basic Execution Benchmarks
// ============================================================================

fn bench_interp_arithmetic_loop(c: &mut Criterion) {
    c.bench_function("interp_arithmetic_loop_10k", |b| {
        let code = "var sum = 0; var i = 0; while (i < 10000) { sum = sum + i; i = i + 1; } sum;";
        b.iter(|| interp_run(black_box(code)));
    });
}

fn bench_interp_fibonacci(c: &mut Criterion) {
    c.bench_function("interp_fibonacci_20", |b| {
        let code = "fn fib(n: number) -> number { if (n <= 1) { return n; } return fib(n - 1) + fib(n - 2); } fib(20);";
        b.iter(|| interp_run(black_box(code)));
    });
}

fn bench_interp_string_concat(c: &mut Criterion) {
    c.bench_function("interp_string_concat_500", |b| {
        let code = r#"var s = ""; var i = 0; while (i < 500) { s = s + "x"; i = i + 1; } len(s);"#;
        b.iter(|| interp_run(black_box(code)));
    });
}

fn bench_interp_collection_ops(c: &mut Criterion) {
    c.bench_function("interp_array_push_pop_1k", |b| {
        let code = r#"
            var arr: number[] = [];
            var i = 0;
            while (i < 1000) {
                arr = push(arr, i);
                i = i + 1;
            }
            len(arr);
        "#;
        b.iter(|| interp_run(black_box(code)));
    });
}

fn bench_interp_function_calls(c: &mut Criterion) {
    c.bench_function("interp_function_calls_10k", |b| {
        let code = "fn inc(x: number) -> number { return x + 1; } var r = 0; var i = 0; while (i < 10000) { r = inc(r); i = i + 1; } r;";
        b.iter(|| interp_run(black_box(code)));
    });
}

fn bench_interp_nested_loops(c: &mut Criterion) {
    c.bench_function("interp_nested_loops_100x100", |b| {
        let code = "var count = 0; var i = 0; while (i < 100) { var j = 0; while (j < 100) { count = count + 1; j = j + 1; } i = i + 1; } count;";
        b.iter(|| interp_run(black_box(code)));
    });
}

// ============================================================================
// Variable Lookup Benchmarks
// ============================================================================

fn bench_interp_variable_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("interp_variable_lookup");

    // Single variable access in tight loop
    group.bench_function("single_var_10k", |b| {
        let code = "let x = 42; var sum = 0; var i = 0; while (i < 10000) { sum = sum + x; i = i + 1; } sum;";
        b.iter(|| interp_run(black_box(code)));
    });

    // Multiple variable accesses per iteration
    group.bench_function("multi_var_10k", |b| {
        let code = "let a = 1; let b = 2; let c = 3; let d = 4; var sum = 0; var i = 0; while (i < 10000) { sum = sum + a + b + c + d; i = i + 1; } sum;";
        b.iter(|| interp_run(black_box(code)));
    });

    // Global vs local variable access
    group.bench_function("global_access_10k", |b| {
        let code = "var global = 0; fn inner() -> number { global = global + 1; return global; } var i = 0; while (i < 10000) { inner(); i = i + 1; } global;";
        b.iter(|| interp_run(black_box(code)));
    });

    group.finish();
}

// ============================================================================
// Scope Depth Benchmarks
// ============================================================================

fn bench_interp_scope_depth(c: &mut Criterion) {
    let mut group = c.benchmark_group("interp_scope_depth");

    // Shallow scope (1 level)
    group.bench_function("depth_1", |b| {
        let code = "var sum = 0; var i = 0; while (i < 1000) { let x = i; sum = sum + x; i = i + 1; } sum;";
        b.iter(|| interp_run(black_box(code)));
    });

    // Medium scope (3 levels)
    group.bench_function("depth_3", |b| {
        let code = r#"
            var sum = 0;
            var i = 0;
            while (i < 1000) {
                let a = i;
                if (true) {
                    let b = a;
                    if (true) {
                        let c = b;
                        sum = sum + c;
                    }
                }
                i = i + 1;
            }
            sum;
        "#;
        b.iter(|| interp_run(black_box(code)));
    });

    // Deep scope (5 levels via nested functions)
    group.bench_function("depth_5_functions", |b| {
        let code = r#"
            fn level1(x: number) -> number {
                fn level2(y: number) -> number {
                    fn level3(z: number) -> number {
                        return z + 1;
                    }
                    return level3(y) + 1;
                }
                return level2(x) + 1;
            }
            var sum = 0;
            var i = 0;
            while (i < 1000) {
                sum = sum + level1(i);
                i = i + 1;
            }
            sum;
        "#;
        b.iter(|| interp_run(black_box(code)));
    });

    group.finish();
}

// ============================================================================
// Recursion Depth Benchmarks
// ============================================================================

fn bench_interp_recursion(c: &mut Criterion) {
    let mut group = c.benchmark_group("interp_recursion");

    // Fibonacci at different depths
    for depth in [10, 15, 20].iter() {
        group.bench_with_input(BenchmarkId::new("fibonacci", depth), depth, |b, &d| {
            let code = format!(
                "fn fib(n: number) -> number {{ if (n <= 1) {{ return n; }} return fib(n - 1) + fib(n - 2); }} fib({});",
                d
            );
            b.iter(|| interp_run(black_box(&code)));
        });
    }

    // Tail recursion pattern (converted to loop)
    group.bench_function("tail_recursive_sum_100", |b| {
        let code = r#"
            fn sum_to(n: number) -> number {
                fn helper(n: number, acc: number) -> number {
                    if (n <= 0) { return acc; }
                    return helper(n - 1, acc + n);
                }
                return helper(n, 0);
            }
            sum_to(100);
        "#;
        b.iter(|| interp_run(black_box(code)));
    });

    group.finish();
}

// ============================================================================
// Function Call Overhead Benchmarks
// ============================================================================

fn bench_interp_call_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("interp_call_overhead");

    // Empty function calls
    group.bench_function("empty_fn_10k", |b| {
        let code =
            "fn noop() -> number { return 0; } var i = 0; while (i < 10000) { noop(); i = i + 1; }";
        b.iter(|| interp_run(black_box(code)));
    });

    // Function with 1 parameter
    group.bench_function("1_param_fn_10k", |b| {
        let code = "fn id(x: number) -> number { return x; } var i = 0; while (i < 10000) { id(i); i = i + 1; }";
        b.iter(|| interp_run(black_box(code)));
    });

    // Function with 3 parameters
    group.bench_function("3_param_fn_10k", |b| {
        let code = "fn add3(a: number, b: number, c: number) -> number { return a + b + c; } var i = 0; while (i < 10000) { add3(i, i, i); i = i + 1; }";
        b.iter(|| interp_run(black_box(code)));
    });

    // Nested function calls
    group.bench_function("nested_calls_5k", |b| {
        let code = "fn f(x: number) -> number { return x + 1; } fn g(x: number) -> number { return f(x) + 1; } fn h(x: number) -> number { return g(x) + 1; } var i = 0; var sum = 0; while (i < 5000) { sum = sum + h(i); i = i + 1; } sum;";
        b.iter(|| interp_run(black_box(code)));
    });

    group.finish();
}

// ============================================================================
// Array Operations Benchmarks
// ============================================================================

fn bench_interp_arrays(c: &mut Criterion) {
    let mut group = c.benchmark_group("interp_arrays");

    // Array creation
    group.bench_function("create_100", |b| {
        let code = "[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96, 97, 98, 99, 100];";
        b.iter(|| interp_run(black_box(code)));
    });

    // Array indexing
    group.bench_function("index_access_10k", |b| {
        let code = "let arr = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]; var sum = 0; var i = 0; while (i < 10000) { sum = sum + arr[i % 10]; i = i + 1; } sum;";
        b.iter(|| interp_run(black_box(code)));
    });

    // Array length
    group.bench_function("len_10k", |b| {
        let code = "let arr = [1, 2, 3, 4, 5]; var sum = 0; var i = 0; while (i < 10000) { sum = sum + len(arr); i = i + 1; } sum;";
        b.iter(|| interp_run(black_box(code)));
    });

    group.finish();
}

// ============================================================================
// Comparison Benchmarks (Parse vs Execution)
// ============================================================================

fn bench_interp_parse_vs_exec(c: &mut Criterion) {
    let mut group = c.benchmark_group("interp_parse_vs_exec");

    let code = "fn fib(n: number) -> number { if (n <= 1) { return n; } return fib(n - 1) + fib(n - 2); } fib(15);";

    group.bench_function("parse_only", |b| {
        b.iter(|| parse_only(black_box(code)));
    });

    group.bench_function("full_execution", |b| {
        b.iter(|| interp_run(black_box(code)));
    });

    group.finish();
}

// ============================================================================
// Throughput Benchmarks
// ============================================================================

fn bench_interp_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("interp_throughput");

    // Measure operations per second
    for iterations in [1000, 5000, 10000].iter() {
        group.throughput(Throughput::Elements(*iterations as u64));
        group.bench_with_input(
            BenchmarkId::new("additions", iterations),
            iterations,
            |b, &n| {
                let code = format!(
                    "var sum = 0; var i = 0; while (i < {}) {{ sum = sum + i; i = i + 1; }} sum;",
                    n
                );
                b.iter(|| interp_run(black_box(&code)));
            },
        );
    }

    group.finish();
}

// ============================================================================
// Builtin Function Benchmarks
// ============================================================================

fn bench_interp_builtins(c: &mut Criterion) {
    let mut group = c.benchmark_group("interp_builtins");

    // Print (to null writer) - tests builtin dispatch
    group.bench_function("builtin_len_10k", |b| {
        let code = r#"let s = "hello world"; var sum = 0; var i = 0; while (i < 10000) { sum = sum + len(s); i = i + 1; } sum;"#;
        b.iter(|| interp_run(black_box(code)));
    });

    // String operations
    group.bench_function("string_trim_1k", |b| {
        let code = r#"var i = 0; while (i < 1000) { trim("  hello  "); i = i + 1; }"#;
        b.iter(|| interp_run(black_box(code)));
    });

    // String upper/lower
    group.bench_function("string_case_1k", |b| {
        let code = r#"var i = 0; while (i < 1000) { toUpperCase("hello"); toLowerCase("WORLD"); i = i + 1; }"#;
        b.iter(|| interp_run(black_box(code)));
    });

    group.finish();
}

criterion_group!(
    basic_benches,
    bench_interp_arithmetic_loop,
    bench_interp_fibonacci,
    bench_interp_string_concat,
    bench_interp_collection_ops,
    bench_interp_function_calls,
    bench_interp_nested_loops
);

criterion_group!(
    advanced_benches,
    bench_interp_variable_lookup,
    bench_interp_scope_depth,
    bench_interp_recursion,
    bench_interp_call_overhead,
    bench_interp_arrays,
    bench_interp_parse_vs_exec,
    bench_interp_throughput,
    bench_interp_builtins
);

criterion_main!(basic_benches, advanced_benches);
