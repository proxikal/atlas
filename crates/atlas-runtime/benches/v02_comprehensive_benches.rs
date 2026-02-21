//! v0.2 Comprehensive Performance Benchmarks
//!
//! End-to-end benchmark suite covering:
//! - Full pipeline (lex → parse → typecheck → compile → run) throughput
//! - Engine comparison (interpreter vs VM unoptimized vs VM optimized)
//! - Profiler overhead measurement
//! - Real-world program performance
//! - Performance regression guards vs v0.1 baselines
//!
//! Run with: cargo bench --bench v02_comprehensive_benches

use atlas_runtime::compiler::Compiler;
use atlas_runtime::lexer::Lexer;
use atlas_runtime::parser::Parser;
use atlas_runtime::security::SecurityContext;
use atlas_runtime::vm::VM;
use atlas_runtime::Interpreter;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

// ============================================================================
// Helpers
// ============================================================================

fn lex_only(source: &str) {
    let mut lexer = Lexer::new(source.to_string());
    let _ = lexer.tokenize();
}

fn parse_only(source: &str) {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let _ = parser.parse();
}

fn compile_only(source: &str) {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();
    let mut compiler = Compiler::new();
    let _ = compiler.compile(&program);
}

fn compile_optimized_only(source: &str) {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();
    let mut compiler = Compiler::with_optimization();
    let _ = compiler.compile(&program);
}

fn vm_run(source: &str, optimized: bool) {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();
    let mut compiler = if optimized {
        Compiler::with_optimization()
    } else {
        Compiler::new()
    };
    let bytecode = compiler.compile(&program).expect("Compilation failed");
    let mut vm = VM::new(bytecode);
    let _ = vm.run(&SecurityContext::allow_all());
}

fn interp_run(source: &str) {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();
    let security = SecurityContext::allow_all();
    let mut interp = Interpreter::new();
    let _ = interp.eval(&program, &security);
}

// ============================================================================
// Pipeline Stage Benchmarks
// ============================================================================

fn bench_pipeline_stages(c: &mut Criterion) {
    let source = r#"
        fn factorial(n: number) -> number {
            if (n <= 1) { return 1; }
            return n * factorial(n - 1);
        }
        factorial(10);
    "#;

    let mut group = c.benchmark_group("pipeline/stages");
    group.bench_function("lex", |b| b.iter(|| lex_only(black_box(source))));
    group.bench_function("parse", |b| b.iter(|| parse_only(black_box(source))));
    group.bench_function("compile", |b| b.iter(|| compile_only(black_box(source))));
    group.bench_function("compile_optimized", |b| {
        b.iter(|| compile_optimized_only(black_box(source)))
    });
    group.bench_function("vm_unoptimized", |b| {
        b.iter(|| vm_run(black_box(source), false))
    });
    group.bench_function("vm_optimized", |b| {
        b.iter(|| vm_run(black_box(source), true))
    });
    group.bench_function("interpreter", |b| b.iter(|| interp_run(black_box(source))));
    group.finish();
}

// ============================================================================
// Engine Comparison Benchmarks
// ============================================================================

fn bench_engines_arithmetic(c: &mut Criterion) {
    let source = r#"
        var sum = 0;
        var i = 0;
        while (i < 5000) {
            sum = sum + i;
            i = i + 1;
        }
        sum;
    "#;

    let mut group = c.benchmark_group("engines/arithmetic_5k");
    group.bench_function("interpreter", |b| b.iter(|| interp_run(black_box(source))));
    group.bench_function("vm_unoptimized", |b| {
        b.iter(|| vm_run(black_box(source), false))
    });
    group.bench_function("vm_optimized", |b| {
        b.iter(|| vm_run(black_box(source), true))
    });
    group.finish();
}

fn bench_engines_fibonacci(c: &mut Criterion) {
    let source = r#"
        fn fib(n: number) -> number {
            if (n <= 1) { return n; }
            return fib(n - 1) + fib(n - 2);
        }
        fib(18);
    "#;

    let mut group = c.benchmark_group("engines/fibonacci_18");
    group.bench_function("interpreter", |b| b.iter(|| interp_run(black_box(source))));
    group.bench_function("vm_unoptimized", |b| {
        b.iter(|| vm_run(black_box(source), false))
    });
    group.bench_function("vm_optimized", |b| {
        b.iter(|| vm_run(black_box(source), true))
    });
    group.finish();
}

fn bench_engines_string_ops(c: &mut Criterion) {
    let source = r#"
        var s = "";
        var i = 0;
        while (i < 200) {
            s = s + "x";
            i = i + 1;
        }
        len(s);
    "#;

    let mut group = c.benchmark_group("engines/string_concat_200");
    group.bench_function("interpreter", |b| b.iter(|| interp_run(black_box(source))));
    group.bench_function("vm_unoptimized", |b| {
        b.iter(|| vm_run(black_box(source), false))
    });
    group.bench_function("vm_optimized", |b| {
        b.iter(|| vm_run(black_box(source), true))
    });
    group.finish();
}

fn bench_engines_recursion(c: &mut Criterion) {
    let source = r#"
        fn sum_to(n: number) -> number {
            if (n <= 0) { return 0; }
            return n + sum_to(n - 1);
        }
        sum_to(500);
    "#;

    let mut group = c.benchmark_group("engines/recursion_500");
    group.bench_function("interpreter", |b| b.iter(|| interp_run(black_box(source))));
    group.bench_function("vm_unoptimized", |b| {
        b.iter(|| vm_run(black_box(source), false))
    });
    group.bench_function("vm_optimized", |b| {
        b.iter(|| vm_run(black_box(source), true))
    });
    group.finish();
}

// ============================================================================
// Profiler Overhead Benchmarks
// ============================================================================

/// Measure the overhead of the VM profiler when enabled vs disabled.
/// Target: overhead < 10% for typical workloads.
fn bench_profiler_overhead(c: &mut Criterion) {
    let source = r#"
        var sum = 0;
        var i = 0;
        while (i < 2000) {
            sum = sum + i;
            i = i + 1;
        }
        sum;
    "#;

    let mut group = c.benchmark_group("profiler/overhead");

    // Baseline: VM without profiler
    group.bench_function("no_profiler", |b| {
        b.iter(|| vm_run(black_box(source), true));
    });

    // VM with profiler enabled (via direct profiler API)
    group.bench_function("with_profiler", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(source.to_string());
            let (tokens, _) = lexer.tokenize();
            let mut parser = Parser::new(tokens);
            let (program, _) = parser.parse();
            let mut compiler = Compiler::with_optimization();
            let bytecode = compiler.compile(&program).expect("Compilation failed");
            let mut vm = VM::with_profiling(bytecode);
            let _ = vm.run(&SecurityContext::allow_all());
        });
    });

    group.finish();
}

fn bench_profiler_overhead_fibonacci(c: &mut Criterion) {
    // Recursive workload — more function calls means more profiler events
    let source = r#"
        fn fib(n: number) -> number {
            if (n <= 1) { return n; }
            return fib(n - 1) + fib(n - 2);
        }
        fib(16);
    "#;

    let mut group = c.benchmark_group("profiler/overhead_recursive");

    group.bench_function("no_profiler", |b| {
        b.iter(|| vm_run(black_box(source), true));
    });

    group.bench_function("with_profiler", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(source.to_string());
            let (tokens, _) = lexer.tokenize();
            let mut parser = Parser::new(tokens);
            let (program, _) = parser.parse();
            let mut compiler = Compiler::with_optimization();
            let bytecode = compiler.compile(&program).expect("Compilation failed");
            let mut vm = VM::with_profiling(bytecode);
            let _ = vm.run(&SecurityContext::allow_all());
        });
    });

    group.finish();
}

// ============================================================================
// Real-World Program Benchmarks
// ============================================================================

fn bench_realworld_bubble_sort(c: &mut Criterion) {
    let source = r#"
        fn bubble_sort_step(arr: number[], n: number) -> number[] {
            var i = 0;
            while (i < n - 1) {
                if (arr[i] > arr[i + 1]) {
                    var tmp = arr[i];
                    arr[i] = arr[i + 1];
                    arr[i + 1] = tmp;
                }
                i = i + 1;
            }
            return arr;
        }

        var data: number[] = [64, 34, 25, 12, 22, 11, 90];
        var n = len(data);
        var pass = 0;
        while (pass < n) {
            data = bubble_sort_step(data, n);
            pass = pass + 1;
        }
        data[0];
    "#;

    let mut group = c.benchmark_group("realworld/bubble_sort");
    group.bench_function("interpreter", |b| b.iter(|| interp_run(black_box(source))));
    group.bench_function("vm_optimized", |b| {
        b.iter(|| vm_run(black_box(source), true))
    });
    group.finish();
}

fn bench_realworld_prime_sieve(c: &mut Criterion) {
    let source = r#"
        fn is_prime(n: number) -> bool {
            if (n < 2) { return false; }
            var i = 2;
            while (i * i <= n) {
                if (n - (n / i) * i == 0) { return false; }
                i = i + 1;
            }
            return true;
        }

        var count = 0;
        var n = 2;
        while (n <= 200) {
            if (is_prime(n)) { count = count + 1; }
            n = n + 1;
        }
        count;
    "#;

    let mut group = c.benchmark_group("realworld/prime_sieve_200");
    group.bench_function("interpreter", |b| b.iter(|| interp_run(black_box(source))));
    group.bench_function("vm_optimized", |b| {
        b.iter(|| vm_run(black_box(source), true))
    });
    group.finish();
}

fn bench_realworld_string_processing(c: &mut Criterion) {
    let source = r#"
        var words: string[] = ["atlas", "language", "compiler", "runtime", "benchmark"];
        var result = "";
        var i = 0;
        while (i < len(words)) {
            if (i > 0) { result = result + " "; }
            result = result + to_upper(words[i]);
            i = i + 1;
        }
        len(result);
    "#;

    let mut group = c.benchmark_group("realworld/string_processing");
    group.bench_function("interpreter", |b| b.iter(|| interp_run(black_box(source))));
    group.bench_function("vm_optimized", |b| {
        b.iter(|| vm_run(black_box(source), true))
    });
    group.finish();
}

fn bench_realworld_accumulator(c: &mut Criterion) {
    // Simulates a common pattern: accumulating values in a collection
    let source = r#"
        var results: number[] = [];
        var i = 0;
        while (i < 200) {
            var val = i * i;
            results = push(results, val);
            i = i + 1;
        }
        len(results);
    "#;

    let mut group = c.benchmark_group("realworld/accumulator");
    group.bench_function("interpreter", |b| b.iter(|| interp_run(black_box(source))));
    group.bench_function("vm_optimized", |b| {
        b.iter(|| vm_run(black_box(source), true))
    });
    group.finish();
}

fn bench_realworld_factorial_loop(c: &mut Criterion) {
    let source = r#"
        fn factorial(n: number) -> number {
            var result = 1;
            var i = 1;
            while (i <= n) {
                result = result * i;
                i = i + 1;
            }
            return result;
        }

        var total = 0;
        var i = 1;
        while (i <= 20) {
            total = total + factorial(i);
            i = i + 1;
        }
        total;
    "#;

    let mut group = c.benchmark_group("realworld/factorial_loop");
    group.bench_function("interpreter", |b| b.iter(|| interp_run(black_box(source))));
    group.bench_function("vm_optimized", |b| {
        b.iter(|| vm_run(black_box(source), true))
    });
    group.finish();
}

// ============================================================================
// Throughput / Input-Size Benchmarks
// ============================================================================

fn bench_throughput_arithmetic(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput/arithmetic");
    for iterations in [1000usize, 5000, 10000] {
        let source = format!(
            "var sum = 0; var i = 0; while (i < {}) {{ sum = sum + i; i = i + 1; }} sum;",
            iterations
        );
        group.throughput(Throughput::Elements(iterations as u64));
        group.bench_with_input(
            BenchmarkId::new("vm_optimized", iterations),
            &source,
            |b, s| {
                b.iter(|| vm_run(black_box(s.as_str()), true));
            },
        );
    }
    group.finish();
}

fn bench_throughput_function_calls(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput/function_calls");
    for calls in [500usize, 2000, 5000] {
        let source = format!(
            "fn inc(x: number) -> number {{ return x + 1; }} var r = 0; var i = 0; while (i < {}) {{ r = inc(r); i = i + 1; }} r;",
            calls
        );
        group.throughput(Throughput::Elements(calls as u64));
        group.bench_with_input(BenchmarkId::new("vm_optimized", calls), &source, |b, s| {
            b.iter(|| vm_run(black_box(s.as_str()), true));
        });
    }
    group.finish();
}

// ============================================================================
// Regression Guards (v0.1 Baselines)
// ============================================================================

/// These benchmarks correspond to the v0.1 baseline workloads.
/// They must not regress significantly from the recorded baseline.txt values.
fn bench_regression_lexer(c: &mut Criterion) {
    let source = r#"
        var x = 42;
        var y = 100;
        var z = x + y;
        z;
    "#;
    c.bench_function("regression/v01/lexer_simple", |b| {
        b.iter(|| lex_only(black_box(source)));
    });
}

fn bench_regression_parser(c: &mut Criterion) {
    let source = r#"
        fn factorial(n: number) -> number {
            if (n <= 1) { return 1; }
            return n * factorial(n - 1);
        }
        factorial(10);
    "#;
    c.bench_function("regression/v01/parser_function", |b| {
        b.iter(|| parse_only(black_box(source)));
    });
}

fn bench_regression_vm_loop(c: &mut Criterion) {
    let source = "var sum = 0; var i = 0; while (i < 1000) { sum = sum + i; i = i + 1; } sum;";
    c.bench_function("regression/v01/vm_loop_1k", |b| {
        b.iter(|| vm_run(black_box(source), false));
    });
}

fn bench_regression_interpreter_loop(c: &mut Criterion) {
    let source = "var sum = 0; var i = 0; while (i < 1000) { sum = sum + i; i = i + 1; } sum;";
    c.bench_function("regression/v01/interp_loop_1k", |b| {
        b.iter(|| interp_run(black_box(source)));
    });
}

fn bench_regression_fibonacci(c: &mut Criterion) {
    let source = r#"
        fn fib(n: number) -> number {
            if (n <= 1) { return n; }
            return fib(n - 1) + fib(n - 2);
        }
        fib(15);
    "#;
    let mut group = c.benchmark_group("regression/v01/fibonacci_15");
    group.bench_function("interpreter", |b| b.iter(|| interp_run(black_box(source))));
    group.bench_function("vm", |b| b.iter(|| vm_run(black_box(source), false)));
    group.finish();
}

// ============================================================================
// Criterion Groups
// ============================================================================

criterion_group!(pipeline, bench_pipeline_stages);

criterion_group!(
    engines,
    bench_engines_arithmetic,
    bench_engines_fibonacci,
    bench_engines_string_ops,
    bench_engines_recursion,
);

criterion_group!(
    profiler,
    bench_profiler_overhead,
    bench_profiler_overhead_fibonacci,
);

criterion_group!(
    realworld,
    bench_realworld_bubble_sort,
    bench_realworld_prime_sieve,
    bench_realworld_string_processing,
    bench_realworld_accumulator,
    bench_realworld_factorial_loop,
);

criterion_group!(
    throughput,
    bench_throughput_arithmetic,
    bench_throughput_function_calls,
);

criterion_group!(
    regression,
    bench_regression_lexer,
    bench_regression_parser,
    bench_regression_vm_loop,
    bench_regression_interpreter_loop,
    bench_regression_fibonacci,
);

criterion_main!(pipeline, engines, profiler, realworld, throughput, regression);
