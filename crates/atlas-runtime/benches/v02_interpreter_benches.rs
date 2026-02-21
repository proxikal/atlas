//! v0.2 Interpreter Performance Benchmarks
//!
//! Measures tree-walking interpreter performance across several workload types
//! and compares it against the unoptimized VM to establish relative baselines.
//!
//! Run with: cargo bench --bench v02_interpreter_benches

use atlas_runtime::compiler::Compiler;
use atlas_runtime::lexer::Lexer;
use atlas_runtime::parser::Parser;
use atlas_runtime::security::SecurityContext;
use atlas_runtime::vm::VM;
use atlas_runtime::Interpreter;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

// ============================================================================
// Helpers
// ============================================================================

fn interp_run(source: &str) {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();
    let security = SecurityContext::allow_all();
    let mut interp = Interpreter::new();
    let _ = interp.eval(&program, &security);
}

fn vm_run_unoptimized(source: &str) {
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
// Variable Lookup Benchmarks
// ============================================================================

/// Stress variable lookup across multiple scope levels.
/// The interpreter uses a scope chain — deeper scopes require more traversals.
fn bench_variable_lookup_flat(c: &mut Criterion) {
    let source = r#"
        var a = 1;
        var b = 2;
        var c = 3;
        var d = 4;
        var e = 5;
        var result = 0;
        var i = 0;
        while (i < 5000) {
            result = a + b + c + d + e;
            i = i + 1;
        }
        result;
    "#;

    let mut group = c.benchmark_group("variable_lookup/flat_scope");
    group.bench_with_input(
        BenchmarkId::new("interpreter", "5vars"),
        source,
        |b, src| {
            b.iter(|| interp_run(black_box(src)));
        },
    );
    group.bench_with_input(
        BenchmarkId::new("vm_baseline", "5vars"),
        source,
        |b, src| {
            b.iter(|| vm_run_unoptimized(black_box(src)));
        },
    );
    group.finish();
}

fn bench_variable_lookup_deep_scope(c: &mut Criterion) {
    let source = r#"
        var outer = 100;
        var i = 0;
        var result = 0;
        while (i < 1000) {
            var mid = outer + i;
            var j = 0;
            while (j < 5) {
                var inner = mid + j;
                result = result + inner + outer;
                j = j + 1;
            }
            i = i + 1;
        }
        result;
    "#;

    let mut group = c.benchmark_group("variable_lookup/deep_scope");
    group.bench_with_input(
        BenchmarkId::new("interpreter", "3-level"),
        source,
        |b, src| {
            b.iter(|| interp_run(black_box(src)));
        },
    );
    group.bench_with_input(
        BenchmarkId::new("vm_baseline", "3-level"),
        source,
        |b, src| {
            b.iter(|| vm_run_unoptimized(black_box(src)));
        },
    );
    group.finish();
}

// ============================================================================
// Function Call Benchmarks
// ============================================================================

fn bench_function_call_overhead(c: &mut Criterion) {
    let source = r#"
        fn identity(x: number) -> number { return x; }
        var result = 0;
        var i = 0;
        while (i < 5000) {
            result = identity(i);
            i = i + 1;
        }
        result;
    "#;

    let mut group = c.benchmark_group("function_calls/overhead");
    group.bench_with_input(BenchmarkId::new("interpreter", "id"), source, |b, src| {
        b.iter(|| interp_run(black_box(src)));
    });
    group.bench_with_input(BenchmarkId::new("vm_baseline", "id"), source, |b, src| {
        b.iter(|| vm_run_unoptimized(black_box(src)));
    });
    group.finish();
}

fn bench_function_call_recursive(c: &mut Criterion) {
    let source = r#"
        fn fib(n: number) -> number {
            if (n <= 1) { return n; }
            return fib(n - 1) + fib(n - 2);
        }
        fib(18);
    "#;

    let mut group = c.benchmark_group("function_calls/recursive");
    group.bench_with_input(
        BenchmarkId::new("interpreter", "fib18"),
        source,
        |b, src| {
            b.iter(|| interp_run(black_box(src)));
        },
    );
    group.bench_with_input(
        BenchmarkId::new("vm_baseline", "fib18"),
        source,
        |b, src| {
            b.iter(|| vm_run_unoptimized(black_box(src)));
        },
    );
    group.finish();
}

fn bench_function_call_mutual_recursion(c: &mut Criterion) {
    let source = r#"
        fn is_even(n: number) -> bool {
            if (n == 0) { return true; }
            return is_odd(n - 1);
        }
        fn is_odd(n: number) -> bool {
            if (n == 0) { return false; }
            return is_even(n - 1);
        }
        var i = 0;
        var count = 0;
        while (i < 100) {
            if (is_even(i)) { count = count + 1; }
            i = i + 1;
        }
        count;
    "#;

    let mut group = c.benchmark_group("function_calls/mutual_recursion");
    group.bench_with_input(
        BenchmarkId::new("interpreter", "mutual"),
        source,
        |b, src| {
            b.iter(|| interp_run(black_box(src)));
        },
    );
    group.bench_with_input(
        BenchmarkId::new("vm_baseline", "mutual"),
        source,
        |b, src| {
            b.iter(|| vm_run_unoptimized(black_box(src)));
        },
    );
    group.finish();
}

// ============================================================================
// AST Traversal Benchmarks
// ============================================================================

/// Deep expression trees stress AST traversal in the interpreter.
fn bench_ast_traversal_deep_expressions(c: &mut Criterion) {
    let source = r#"
        var i = 0;
        var result = 0;
        while (i < 2000) {
            result = ((i + 1) * (i + 2) - (i * i)) / (i + 1 + 1);
            i = i + 1;
        }
        result;
    "#;

    let mut group = c.benchmark_group("ast_traversal/deep_expressions");
    group.bench_with_input(BenchmarkId::new("interpreter", "expr"), source, |b, src| {
        b.iter(|| interp_run(black_box(src)));
    });
    group.bench_with_input(BenchmarkId::new("vm_baseline", "expr"), source, |b, src| {
        b.iter(|| vm_run_unoptimized(black_box(src)));
    });
    group.finish();
}

fn bench_ast_traversal_control_flow(c: &mut Criterion) {
    let source = r#"
        var i = 0;
        var fizzbuzz = 0;
        while (i < 1000) {
            if (i - (i / 15) * 15 == 0) {
                fizzbuzz = fizzbuzz + 1;
            } else {
                if (i - (i / 3) * 3 == 0) {
                    fizzbuzz = fizzbuzz + 1;
                } else {
                    if (i - (i / 5) * 5 == 0) {
                        fizzbuzz = fizzbuzz + 1;
                    }
                }
            }
            i = i + 1;
        }
        fizzbuzz;
    "#;

    let mut group = c.benchmark_group("ast_traversal/control_flow");
    group.bench_with_input(
        BenchmarkId::new("interpreter", "fizzbuzz"),
        source,
        |b, src| {
            b.iter(|| interp_run(black_box(src)));
        },
    );
    group.bench_with_input(
        BenchmarkId::new("vm_baseline", "fizzbuzz"),
        source,
        |b, src| {
            b.iter(|| vm_run_unoptimized(black_box(src)));
        },
    );
    group.finish();
}

// ============================================================================
// Loop Performance Benchmarks
// ============================================================================

fn bench_loop_counting(c: &mut Criterion) {
    let source = r#"
        var sum = 0;
        var i = 0;
        while (i < 10000) {
            sum = sum + i;
            i = i + 1;
        }
        sum;
    "#;

    let mut group = c.benchmark_group("loop/counting_10k");
    group.bench_with_input(BenchmarkId::new("interpreter", "sum"), source, |b, src| {
        b.iter(|| interp_run(black_box(src)));
    });
    group.bench_with_input(BenchmarkId::new("vm_baseline", "sum"), source, |b, src| {
        b.iter(|| vm_run_unoptimized(black_box(src)));
    });
    group.finish();
}

fn bench_loop_nested(c: &mut Criterion) {
    let source = r#"
        var total = 0;
        var i = 0;
        while (i < 100) {
            var j = 0;
            while (j < 100) {
                total = total + 1;
                j = j + 1;
            }
            i = i + 1;
        }
        total;
    "#;

    let mut group = c.benchmark_group("loop/nested_100x100");
    group.bench_with_input(
        BenchmarkId::new("interpreter", "nested"),
        source,
        |b, src| {
            b.iter(|| interp_run(black_box(src)));
        },
    );
    group.bench_with_input(
        BenchmarkId::new("vm_baseline", "nested"),
        source,
        |b, src| {
            b.iter(|| vm_run_unoptimized(black_box(src)));
        },
    );
    group.finish();
}

// ============================================================================
// Interpreter-Specific Workloads
// ============================================================================

fn bench_interp_scope_creation(c: &mut Criterion) {
    // Many short-lived scopes stress scope push/pop
    let source = r#"
        var result = 0;
        var i = 0;
        while (i < 3000) {
            {
                var tmp = i * 2;
                result = result + tmp;
            }
            i = i + 1;
        }
        result;
    "#;

    let mut group = c.benchmark_group("interpreter/scope_creation");
    group.bench_with_input(
        BenchmarkId::new("interpreter", "block_scopes"),
        source,
        |b, src| {
            b.iter(|| interp_run(black_box(src)));
        },
    );
    group.bench_with_input(
        BenchmarkId::new("vm_baseline", "block_scopes"),
        source,
        |b, src| {
            b.iter(|| vm_run_unoptimized(black_box(src)));
        },
    );
    group.finish();
}

fn bench_interp_conditional_heavy(c: &mut Criterion) {
    let source = r#"
        fn classify(n: number) -> number {
            if (n < 0) {
                return 0;
            } else {
                if (n < 10) {
                    return 1;
                } else {
                    if (n < 100) {
                        return 2;
                    } else {
                        return 3;
                    }
                }
            }
        }
        var sum = 0;
        var i = 0;
        while (i < 2000) {
            sum = sum + classify(i);
            i = i + 1;
        }
        sum;
    "#;

    let mut group = c.benchmark_group("interpreter/conditional_heavy");
    group.bench_with_input(
        BenchmarkId::new("interpreter", "classify"),
        source,
        |b, src| {
            b.iter(|| interp_run(black_box(src)));
        },
    );
    group.bench_with_input(
        BenchmarkId::new("vm_baseline", "classify"),
        source,
        |b, src| {
            b.iter(|| vm_run_unoptimized(black_box(src)));
        },
    );
    group.finish();
}

// ============================================================================
// Criterion Groups
// ============================================================================

criterion_group!(
    variable_lookup,
    bench_variable_lookup_flat,
    bench_variable_lookup_deep_scope,
);

criterion_group!(
    function_calls,
    bench_function_call_overhead,
    bench_function_call_recursive,
    bench_function_call_mutual_recursion,
);

criterion_group!(
    ast_traversal,
    bench_ast_traversal_deep_expressions,
    bench_ast_traversal_control_flow,
);

criterion_group!(loops, bench_loop_counting, bench_loop_nested,);

criterion_group!(
    interpreter_specific,
    bench_interp_scope_creation,
    bench_interp_conditional_heavy,
);

criterion_main!(
    variable_lookup,
    function_calls,
    ast_traversal,
    loops,
    interpreter_specific,
);
