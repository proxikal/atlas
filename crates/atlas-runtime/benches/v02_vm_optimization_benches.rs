//! v0.2 VM Optimization Benchmarks
//!
//! Measures the impact of the three-pass bytecode optimizer on real programs.
//! Each benchmark pair runs identical Atlas source through:
//!   - `Compiler::new()` — no optimizer (baseline)
//!   - `Compiler::with_optimization()` — constant folding + DCE + peephole
//!
//! Run with: cargo bench --bench v02_vm_optimization_benches

use atlas_runtime::compiler::Compiler;
use atlas_runtime::lexer::Lexer;
use atlas_runtime::parser::Parser;
use atlas_runtime::security::SecurityContext;
use atlas_runtime::vm::VM;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

// ============================================================================
// Helpers
// ============================================================================

fn compile_and_run(source: &str, optimized: bool) {
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

// ============================================================================
// Constant Folding Benchmarks
// ============================================================================

/// Programs that consist almost entirely of constant arithmetic expressions.
/// Constant folding collapses these at compile time — the VM only executes
/// a single constant load instead of a tree of arithmetic opcodes.
fn bench_constant_folding_arithmetic(c: &mut Criterion) {
    let source = r#"
        var a = 1 + 2 + 3 + 4 + 5;
        var b = 10 * 20 + 30 - 5;
        var c = 100 / 4 * 2;
        var d = a + b + c;
        d;
    "#;

    let mut group = c.benchmark_group("constant_folding/arithmetic");
    group.bench_with_input(BenchmarkId::new("unoptimized", "expr"), source, |b, src| {
        b.iter(|| compile_and_run(black_box(src), false));
    });
    group.bench_with_input(BenchmarkId::new("optimized", "expr"), source, |b, src| {
        b.iter(|| compile_and_run(black_box(src), true));
    });
    group.finish();
}

fn bench_constant_folding_chain(c: &mut Criterion) {
    // Long chain of constants that folds down to a single value
    let source = r#"
        var x = 2 * 2 * 2 * 2 * 2 * 2 * 2 * 2;
        var y = 1000 - 100 - 10 - 1;
        var z = x + y;
        z;
    "#;

    let mut group = c.benchmark_group("constant_folding/chain");
    group.bench_with_input(
        BenchmarkId::new("unoptimized", "chain"),
        source,
        |b, src| {
            b.iter(|| compile_and_run(black_box(src), false));
        },
    );
    group.bench_with_input(BenchmarkId::new("optimized", "chain"), source, |b, src| {
        b.iter(|| compile_and_run(black_box(src), true));
    });
    group.finish();
}

fn bench_constant_folding_comparisons(c: &mut Criterion) {
    // Constant boolean expressions — should fold to true/false
    let source = r#"
        var a = 5 > 3;
        var b = 10 == 10;
        var c = 7 != 8;
        var d = a && b && c;
        d;
    "#;

    let mut group = c.benchmark_group("constant_folding/comparisons");
    group.bench_with_input(BenchmarkId::new("unoptimized", "bool"), source, |b, src| {
        b.iter(|| compile_and_run(black_box(src), false));
    });
    group.bench_with_input(BenchmarkId::new("optimized", "bool"), source, |b, src| {
        b.iter(|| compile_and_run(black_box(src), true));
    });
    group.finish();
}

// ============================================================================
// Dead Code Elimination Benchmarks
// ============================================================================

/// Programs that contain code after `return` statements.
/// DCE removes these unreachable instructions, reducing the bytecode footprint.
fn bench_dce_after_return(c: &mut Criterion) {
    let source = r#"
        fn compute(x: number) -> number {
            return x * 2;
            var unused = 999;
            var also_unused = "never reached";
            return 0;
        }
        var sum = 0;
        var i = 0;
        while (i < 1000) {
            sum = sum + compute(i);
            i = i + 1;
        }
        sum;
    "#;

    let mut group = c.benchmark_group("dead_code_elimination/after_return");
    group.bench_with_input(BenchmarkId::new("unoptimized", "ret"), source, |b, src| {
        b.iter(|| compile_and_run(black_box(src), false));
    });
    group.bench_with_input(BenchmarkId::new("optimized", "ret"), source, |b, src| {
        b.iter(|| compile_and_run(black_box(src), true));
    });
    group.finish();
}

fn bench_dce_combined_with_constants(c: &mut Criterion) {
    // Dead code follows constant-folded branch
    let source = r#"
        fn classify(n: number) -> number {
            if (true) {
                return n * 2;
            }
            return n - 1;
        }
        var result = 0;
        var i = 0;
        while (i < 500) {
            result = classify(i);
            i = i + 1;
        }
        result;
    "#;

    let mut group = c.benchmark_group("dead_code_elimination/combined_constants");
    group.bench_with_input(
        BenchmarkId::new("unoptimized", "branch"),
        source,
        |b, src| {
            b.iter(|| compile_and_run(black_box(src), false));
        },
    );
    group.bench_with_input(BenchmarkId::new("optimized", "branch"), source, |b, src| {
        b.iter(|| compile_and_run(black_box(src), true));
    });
    group.finish();
}

// ============================================================================
// Peephole Optimization Benchmarks
// ============================================================================

/// Programs that trigger common peephole patterns (dup-pop, not-not, etc.)
fn bench_peephole_not_patterns(c: &mut Criterion) {
    let source = r#"
        fn check(x: number) -> bool {
            return !!( x > 0 );
        }
        var count = 0;
        var i = 0;
        while (i < 1000) {
            if (check(i)) { count = count + 1; }
            i = i + 1;
        }
        count;
    "#;

    let mut group = c.benchmark_group("peephole/not_patterns");
    group.bench_with_input(
        BenchmarkId::new("unoptimized", "not_not"),
        source,
        |b, src| {
            b.iter(|| compile_and_run(black_box(src), false));
        },
    );
    group.bench_with_input(
        BenchmarkId::new("optimized", "not_not"),
        source,
        |b, src| {
            b.iter(|| compile_and_run(black_box(src), true));
        },
    );
    group.finish();
}

fn bench_peephole_negation(c: &mut Criterion) {
    let source = r#"
        fn negate_twice(x: number) -> number {
            return -(-x);
        }
        var sum = 0;
        var i = 0;
        while (i < 2000) {
            sum = sum + negate_twice(i);
            i = i + 1;
        }
        sum;
    "#;

    let mut group = c.benchmark_group("peephole/negation");
    group.bench_with_input(BenchmarkId::new("unoptimized", "neg"), source, |b, src| {
        b.iter(|| compile_and_run(black_box(src), false));
    });
    group.bench_with_input(BenchmarkId::new("optimized", "neg"), source, |b, src| {
        b.iter(|| compile_and_run(black_box(src), true));
    });
    group.finish();
}

// ============================================================================
// Combined Optimization Benchmarks (Real-world programs)
// ============================================================================

fn bench_combined_fibonacci(c: &mut Criterion) {
    let source = r#"
        fn fib(n: number) -> number {
            if (n <= 1) { return n; }
            return fib(n - 1) + fib(n - 2);
        }
        fib(20);
    "#;

    let mut group = c.benchmark_group("combined/fibonacci");
    group.bench_with_input(
        BenchmarkId::new("unoptimized", "fib20"),
        source,
        |b, src| {
            b.iter(|| compile_and_run(black_box(src), false));
        },
    );
    group.bench_with_input(BenchmarkId::new("optimized", "fib20"), source, |b, src| {
        b.iter(|| compile_and_run(black_box(src), true));
    });
    group.finish();
}

fn bench_combined_loop_heavy(c: &mut Criterion) {
    let source = r#"
        var sum = 0;
        var i = 0;
        while (i < 5000) {
            var inner = 0;
            var j = 0;
            while (j < 10) {
                inner = inner + j;
                j = j + 1;
            }
            sum = sum + inner;
            i = i + 1;
        }
        sum;
    "#;

    let mut group = c.benchmark_group("combined/loop_heavy");
    group.bench_with_input(
        BenchmarkId::new("unoptimized", "nested"),
        source,
        |b, src| {
            b.iter(|| compile_and_run(black_box(src), false));
        },
    );
    group.bench_with_input(BenchmarkId::new("optimized", "nested"), source, |b, src| {
        b.iter(|| compile_and_run(black_box(src), true));
    });
    group.finish();
}

fn bench_combined_function_calls(c: &mut Criterion) {
    let source = r#"
        fn double(x: number) -> number { return x * 2; }
        fn triple(x: number) -> number { return x * 3; }
        fn compute(x: number) -> number { return double(x) + triple(x); }

        var result = 0;
        var i = 0;
        while (i < 1000) {
            result = compute(i);
            i = i + 1;
        }
        result;
    "#;

    let mut group = c.benchmark_group("combined/function_calls");
    group.bench_with_input(
        BenchmarkId::new("unoptimized", "multi_fn"),
        source,
        |b, src| {
            b.iter(|| compile_and_run(black_box(src), false));
        },
    );
    group.bench_with_input(
        BenchmarkId::new("optimized", "multi_fn"),
        source,
        |b, src| {
            b.iter(|| compile_and_run(black_box(src), true));
        },
    );
    group.finish();
}

fn bench_combined_string_ops(c: &mut Criterion) {
    let source = r#"
        var prefix = "hello_";
        var i = 0;
        var count = 0;
        while (i < 200) {
            var s = prefix + "world";
            count = count + len(s);
            i = i + 1;
        }
        count;
    "#;

    let mut group = c.benchmark_group("combined/string_ops");
    group.bench_with_input(
        BenchmarkId::new("unoptimized", "concat"),
        source,
        |b, src| {
            b.iter(|| compile_and_run(black_box(src), false));
        },
    );
    group.bench_with_input(BenchmarkId::new("optimized", "concat"), source, |b, src| {
        b.iter(|| compile_and_run(black_box(src), true));
    });
    group.finish();
}

fn bench_combined_arithmetic_heavy(c: &mut Criterion) {
    let source = r#"
        fn power(base: number, exp: number) -> number {
            var result = 1;
            var i = 0;
            while (i < exp) {
                result = result * base;
                i = i + 1;
            }
            return result;
        }

        var total = 0;
        var i = 0;
        while (i < 100) {
            total = total + power(2, 10);
            i = i + 1;
        }
        total;
    "#;

    let mut group = c.benchmark_group("combined/arithmetic_heavy");
    group.bench_with_input(
        BenchmarkId::new("unoptimized", "power"),
        source,
        |b, src| {
            b.iter(|| compile_and_run(black_box(src), false));
        },
    );
    group.bench_with_input(BenchmarkId::new("optimized", "power"), source, |b, src| {
        b.iter(|| compile_and_run(black_box(src), true));
    });
    group.finish();
}

fn bench_combined_real_world(c: &mut Criterion) {
    // Simulate a real-world-like program: prime sieve up to 100
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
        while (n <= 100) {
            if (is_prime(n)) { count = count + 1; }
            n = n + 1;
        }
        count;
    "#;

    let mut group = c.benchmark_group("combined/real_world");
    group.bench_with_input(
        BenchmarkId::new("unoptimized", "prime"),
        source,
        |b, src| {
            b.iter(|| compile_and_run(black_box(src), false));
        },
    );
    group.bench_with_input(BenchmarkId::new("optimized", "prime"), source, |b, src| {
        b.iter(|| compile_and_run(black_box(src), true));
    });
    group.finish();
}

// ============================================================================
// Optimization Level Benchmarks
// ============================================================================

fn vm_run_level(source: &str, level: u8) {
    use atlas_runtime::optimizer::Optimizer;

    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();
    let mut compiler = Compiler::new();
    compiler.set_optimizer(if level == 0 {
        None
    } else {
        Some(Optimizer::with_optimization_level(level))
    });
    let bytecode = compiler.compile(&program).expect("Compilation failed");
    let mut vm = VM::new(bytecode);
    let _ = vm.run(&SecurityContext::allow_all());
}

fn bench_optimization_levels(c: &mut Criterion) {
    let source = r#"
        fn fib(n: number) -> number {
            if (n <= 1) { return n; }
            return fib(n - 1) + fib(n - 2);
        }
        fib(18);
    "#;

    let mut group = c.benchmark_group("optimization_levels");
    for level in [0u8, 1, 2, 3] {
        group.bench_with_input(BenchmarkId::new("level", level), &level, |b, &lvl| {
            b.iter(|| vm_run_level(black_box(source), lvl));
        });
    }
    group.finish();
}

// ============================================================================
// Criterion Groups
// ============================================================================

criterion_group!(
    constant_folding,
    bench_constant_folding_arithmetic,
    bench_constant_folding_chain,
    bench_constant_folding_comparisons,
);

criterion_group!(
    dead_code_elimination,
    bench_dce_after_return,
    bench_dce_combined_with_constants,
);

criterion_group!(
    peephole,
    bench_peephole_not_patterns,
    bench_peephole_negation,
);

criterion_group!(
    combined,
    bench_combined_fibonacci,
    bench_combined_loop_heavy,
    bench_combined_function_calls,
    bench_combined_string_ops,
    bench_combined_arithmetic_heavy,
    bench_combined_real_world,
);

criterion_group!(levels, bench_optimization_levels,);

criterion_main!(
    constant_folding,
    dead_code_elimination,
    peephole,
    combined,
    levels,
);
