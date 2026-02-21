# Phase Correctness-10: Stdlib Error Message Context

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Correctness-09 complete. Build passes. Suite green.

**Verification:**
```bash
cargo check -p atlas-runtime 2>&1 | grep -c "error"  # must be 0
cargo nextest run -p atlas-runtime 2>&1 | tail -3
```

---

## Objective

`RuntimeError::InvalidStdlibArgument { span }` is used ~294 times across 17 files. Every usage produces the same useless error message: "Invalid stdlib argument". The user has no idea which function was called, what argument was wrong, what the expected type was, or what they actually passed. This is unacceptable for a world-class compiler.

Every other major language provides rich stdlib error messages:
- Python: `TypeError: abs() argument must be real number, not 'str'`
- Rust: `expected f64, found &str`
- Go: `cannot use "hello" (untyped string) as int value`

This phase adds a `msg: String` field to `InvalidStdlibArgument` and updates every usage to provide context: function name, expected type, actual type, and argument position.

---

## Files Changed

- `crates/atlas-runtime/src/value.rs` — add `msg: String` field to `InvalidStdlibArgument` variant
- `crates/atlas-runtime/src/stdlib/mod.rs` — update all `InvalidStdlibArgument` constructions
- `crates/atlas-runtime/src/stdlib/*.rs` — update all 17 stdlib files that construct this error
- `crates/atlas-runtime/src/runtime.rs` — update if it constructs this error

---

## Dependencies

- Correctness-09 complete
- No other phases are prerequisites

---

## Implementation

### Step 1: Add msg field to the error variant

In `value.rs`, change:
```rust
#[error("Invalid stdlib argument")]
InvalidStdlibArgument { span: crate::span::Span },
```
to:
```rust
#[error("{msg}")]
InvalidStdlibArgument { msg: String, span: crate::span::Span },
```

This will cause a compile error at every construction site — the compiler tells you exactly what to fix.

### Step 2: Create a helper for consistent error construction

In `stdlib/mod.rs`, add a helper to eliminate boilerplate:

```rust
/// Construct an InvalidStdlibArgument error with context.
pub(crate) fn stdlib_arg_error(
    func_name: &str,
    expected: &str,
    actual: &Value,
    span: crate::span::Span,
) -> RuntimeError {
    RuntimeError::InvalidStdlibArgument {
        msg: format!(
            "{}(): expected {}, got {}",
            func_name, expected, actual.type_name()
        ),
        span,
    }
}

/// Construct an arity error for stdlib functions.
pub(crate) fn stdlib_arity_error(
    func_name: &str,
    expected: usize,
    actual: usize,
    span: crate::span::Span,
) -> RuntimeError {
    RuntimeError::InvalidStdlibArgument {
        msg: format!(
            "{}(): expected {} argument(s), got {}",
            func_name, expected, actual
        ),
        span,
    }
}
```

### Step 3: Update all construction sites

Work through each file systematically. The compile errors guide you. For each site:

**Before:**
```rust
return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
```

**After (type error):**
```rust
return Err(stdlib_arg_error("abs", "number", &args[0], call_span));
```

**After (arity error):**
```rust
return Err(stdlib_arity_error("abs", 1, args.len(), call_span));
```

Process files in this order (highest occurrence count first):
1. `stdlib/mod.rs` (124 occurrences)
2. `stdlib/io.rs` (22)
3. `stdlib/async_primitives.rs` (20)
4. `stdlib/collections/hashset.rs` (17)
5. `stdlib/types.rs` (17)
6. `stdlib/regex.rs` (14)
7. `stdlib/collections/hashmap.rs` (13)
8. `stdlib/reflect.rs` (12)
9. `stdlib/future.rs` (10)
10. `stdlib/json.rs` (9)
11. Remaining files (< 10 each)

### Step 4: Update the span() accessor

In `value.rs`, the `span()` method matches on `InvalidStdlibArgument { span }` — update to `InvalidStdlibArgument { span, .. }`.

### Step 5: Verify error messages are descriptive

Add a test that triggers a type error and checks the message contains the function name:

```rust
#[test]
fn test_stdlib_error_message_contains_function_name() {
    let result = eval_program(r#"abs("hello")"#);
    let err = result.unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("abs"), "Error should mention function name: {}", msg);
    assert!(msg.contains("number"), "Error should mention expected type: {}", msg);
    assert!(msg.contains("string"), "Error should mention actual type: {}", msg);
}
```

---

## Tests

- `test_stdlib_error_message_contains_function_name` — error mentions function, expected type, actual type
- `test_stdlib_arity_error_message` — wrong arg count mentions function and counts
- Spot-check 5+ different stdlib functions for message quality
- All existing tests pass (error variant shape changed but behavior identical)
- Zero clippy warnings

---

## Acceptance

- `InvalidStdlibArgument` has `msg: String` field
- `stdlib_arg_error()` and `stdlib_arity_error()` helpers exist and are used consistently
- Zero bare `InvalidStdlibArgument { span }` constructions remain (all carry context)
- Error messages follow the pattern: `func_name(): expected X, got Y`
- All existing tests pass: `cargo nextest run -p atlas-runtime`
- Zero clippy warnings: `cargo clippy -p atlas-runtime -- -D warnings`
- Commit: `fix(stdlib): Add function name and type context to all stdlib error messages`
