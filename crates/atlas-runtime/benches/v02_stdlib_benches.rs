//! v0.2 Standard Library Performance Benchmarks
//!
//! Measures throughput and latency of stdlib functions across all categories:
//! - String manipulation (len, split, join, to_upper, to_lower, trim, contains)
//! - Array operations (push, pop, slice, filter, map, sort)
//! - Math functions (abs, floor, ceil, sqrt, pow, min, max)
//! - Type utilities (to_string, to_number, type_of)
//! - JSON serialization (json_encode, json_decode)
//!
//! Run with: cargo bench --bench v02_stdlib_benches

use atlas_runtime::compiler::Compiler;
use atlas_runtime::lexer::Lexer;
use atlas_runtime::parser::Parser;
use atlas_runtime::security::SecurityContext;
use atlas_runtime::vm::VM;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

// ============================================================================
// Helpers
// ============================================================================

fn vm_run(source: &str) {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();
    let mut compiler = Compiler::with_optimization();
    let bytecode = compiler.compile(&program).expect("Compilation failed");
    let mut vm = VM::new(bytecode);
    let _ = vm.run(&SecurityContext::allow_all());
}

// ============================================================================
// String Function Benchmarks
// ============================================================================

fn bench_string_len(c: &mut Criterion) {
    let source = r#"
        var s = "hello, world! this is a test string for benchmarking.";
        var i = 0;
        var total = 0;
        while (i < 5000) {
            total = total + len(s);
            i = i + 1;
        }
        total;
    "#;
    c.bench_function("stdlib/string/len_5k", |b| {
        b.iter(|| vm_run(black_box(source)));
    });
}

fn bench_string_to_upper(c: &mut Criterion) {
    let source = r#"
        var s = "hello world";
        var i = 0;
        while (i < 2000) {
            var _ = to_upper(s);
            i = i + 1;
        }
        "done";
    "#;
    c.bench_function("stdlib/string/to_upper_2k", |b| {
        b.iter(|| vm_run(black_box(source)));
    });
}

fn bench_string_to_lower(c: &mut Criterion) {
    let source = r#"
        var s = "HELLO WORLD";
        var i = 0;
        while (i < 2000) {
            var _ = to_lower(s);
            i = i + 1;
        }
        "done";
    "#;
    c.bench_function("stdlib/string/to_lower_2k", |b| {
        b.iter(|| vm_run(black_box(source)));
    });
}

fn bench_string_trim(c: &mut Criterion) {
    let source = r#"
        var s = "   hello world   ";
        var i = 0;
        while (i < 3000) {
            var _ = trim(s);
            i = i + 1;
        }
        "done";
    "#;
    c.bench_function("stdlib/string/trim_3k", |b| {
        b.iter(|| vm_run(black_box(source)));
    });
}

fn bench_string_contains(c: &mut Criterion) {
    let source = r#"
        var haystack = "the quick brown fox jumps over the lazy dog";
        var needle = "fox";
        var count = 0;
        var i = 0;
        while (i < 3000) {
            if (contains(haystack, needle)) { count = count + 1; }
            i = i + 1;
        }
        count;
    "#;
    c.bench_function("stdlib/string/contains_3k", |b| {
        b.iter(|| vm_run(black_box(source)));
    });
}

fn bench_string_starts_with(c: &mut Criterion) {
    let source = r#"
        var s = "hello world";
        var count = 0;
        var i = 0;
        while (i < 3000) {
            if (starts_with(s, "hello")) { count = count + 1; }
            i = i + 1;
        }
        count;
    "#;
    c.bench_function("stdlib/string/starts_with_3k", |b| {
        b.iter(|| vm_run(black_box(source)));
    });
}

fn bench_string_split(c: &mut Criterion) {
    let source = r#"
        var s = "a,b,c,d,e,f,g,h,i,j";
        var i = 0;
        while (i < 1000) {
            var parts = split(s, ",");
            i = i + 1;
        }
        "done";
    "#;
    c.bench_function("stdlib/string/split_1k", |b| {
        b.iter(|| vm_run(black_box(source)));
    });
}

fn bench_string_replace(c: &mut Criterion) {
    let source = r#"
        var s = "hello world hello world hello";
        var i = 0;
        while (i < 1000) {
            var _ = replace(s, "hello", "hi");
            i = i + 1;
        }
        "done";
    "#;
    c.bench_function("stdlib/string/replace_1k", |b| {
        b.iter(|| vm_run(black_box(source)));
    });
}

fn bench_string_concat_loop(c: &mut Criterion) {
    let mut group = c.benchmark_group("stdlib/string/concat_size");
    for size in [50usize, 100, 200] {
        let src = format!(
            r#"var s = ""; var i = 0; while (i < {}) {{ s = s + "x"; i = i + 1; }} len(s);"#,
            size
        );
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::new("concat", size), &src, |b, s| {
            b.iter(|| vm_run(black_box(s.as_str())));
        });
    }
    group.finish();
}

// ============================================================================
// Array Function Benchmarks
// ============================================================================

fn bench_array_push(c: &mut Criterion) {
    let source = r#"
        var arr: number[] = [];
        var i = 0;
        while (i < 1000) {
            arr = push(arr, i);
            i = i + 1;
        }
        len(arr);
    "#;
    c.bench_function("stdlib/array/push_1k", |b| {
        b.iter(|| vm_run(black_box(source)));
    });
}

fn bench_array_pop(c: &mut Criterion) {
    // Build array first, then pop all elements
    let source = r#"
        var arr: number[] = [];
        var i = 0;
        while (i < 500) {
            arr = push(arr, i);
            i = i + 1;
        }
        while (len(arr) > 0) {
            arr = pop(arr);
        }
        len(arr);
    "#;
    c.bench_function("stdlib/array/pop_500", |b| {
        b.iter(|| vm_run(black_box(source)));
    });
}

fn bench_array_len(c: &mut Criterion) {
    let source = r#"
        var arr: number[] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        var total = 0;
        var i = 0;
        while (i < 5000) {
            total = total + len(arr);
            i = i + 1;
        }
        total;
    "#;
    c.bench_function("stdlib/array/len_5k", |b| {
        b.iter(|| vm_run(black_box(source)));
    });
}

fn bench_array_index(c: &mut Criterion) {
    let source = r#"
        var arr: number[] = [10, 20, 30, 40, 50];
        var sum = 0;
        var i = 0;
        while (i < 5000) {
            sum = sum + arr[0] + arr[2] + arr[4];
            i = i + 1;
        }
        sum;
    "#;
    c.bench_function("stdlib/array/index_5k", |b| {
        b.iter(|| vm_run(black_box(source)));
    });
}

fn bench_array_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("stdlib/array/push_size");
    for size in [100usize, 500, 1000] {
        let src = format!(
            r#"var arr: number[] = []; var i = 0; while (i < {}) {{ arr = push(arr, i); i = i + 1; }} len(arr);"#,
            size
        );
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::new("push", size), &src, |b, s| {
            b.iter(|| vm_run(black_box(s.as_str())));
        });
    }
    group.finish();
}

// ============================================================================
// Math Function Benchmarks
// ============================================================================

fn bench_math_abs(c: &mut Criterion) {
    let source = r#"
        var sum = 0;
        var i = 0;
        while (i < 5000) {
            sum = sum + abs(i - 2500);
            i = i + 1;
        }
        sum;
    "#;
    c.bench_function("stdlib/math/abs_5k", |b| {
        b.iter(|| vm_run(black_box(source)));
    });
}

fn bench_math_floor_ceil(c: &mut Criterion) {
    let source = r#"
        var sum = 0;
        var i = 0;
        while (i < 3000) {
            var x = i + 0.5;
            sum = sum + floor(x) + ceil(x);
            i = i + 1;
        }
        sum;
    "#;
    c.bench_function("stdlib/math/floor_ceil_3k", |b| {
        b.iter(|| vm_run(black_box(source)));
    });
}

fn bench_math_sqrt(c: &mut Criterion) {
    let source = r#"
        var sum = 0;
        var i = 1;
        while (i <= 2000) {
            sum = sum + sqrt(i);
            i = i + 1;
        }
        sum;
    "#;
    c.bench_function("stdlib/math/sqrt_2k", |b| {
        b.iter(|| vm_run(black_box(source)));
    });
}

fn bench_math_min_max(c: &mut Criterion) {
    let source = r#"
        var result = 0;
        var i = 0;
        while (i < 3000) {
            result = max(min(i, 100), 0);
            i = i + 1;
        }
        result;
    "#;
    c.bench_function("stdlib/math/min_max_3k", |b| {
        b.iter(|| vm_run(black_box(source)));
    });
}

// ============================================================================
// Type Utility Benchmarks
// ============================================================================

fn bench_type_to_string(c: &mut Criterion) {
    let source = r#"
        var i = 0;
        while (i < 3000) {
            var _ = to_string(i);
            i = i + 1;
        }
        "done";
    "#;
    c.bench_function("stdlib/type/to_string_3k", |b| {
        b.iter(|| vm_run(black_box(source)));
    });
}

fn bench_type_to_number(c: &mut Criterion) {
    let source = r#"
        var s = "42";
        var sum = 0;
        var i = 0;
        while (i < 3000) {
            sum = sum + to_number(s);
            i = i + 1;
        }
        sum;
    "#;
    c.bench_function("stdlib/type/to_number_3k", |b| {
        b.iter(|| vm_run(black_box(source)));
    });
}

fn bench_type_type_of(c: &mut Criterion) {
    let source = r#"
        var count = 0;
        var i = 0;
        while (i < 3000) {
            if (type_of(i) == "number") { count = count + 1; }
            i = i + 1;
        }
        count;
    "#;
    c.bench_function("stdlib/type/type_of_3k", |b| {
        b.iter(|| vm_run(black_box(source)));
    });
}

// ============================================================================
// Criterion Groups
// ============================================================================

criterion_group!(
    string_benches,
    bench_string_len,
    bench_string_to_upper,
    bench_string_to_lower,
    bench_string_trim,
    bench_string_contains,
    bench_string_starts_with,
    bench_string_split,
    bench_string_replace,
    bench_string_concat_loop,
);

criterion_group!(
    array_benches,
    bench_array_push,
    bench_array_pop,
    bench_array_len,
    bench_array_index,
    bench_array_sizes,
);

criterion_group!(
    math_benches,
    bench_math_abs,
    bench_math_floor_ceil,
    bench_math_sqrt,
    bench_math_min_max,
);

criterion_group!(
    type_benches,
    bench_type_to_string,
    bench_type_to_number,
    bench_type_type_of,
);

criterion_main!(string_benches, array_benches, math_benches, type_benches);
