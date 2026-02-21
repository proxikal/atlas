# Phase Correctness-02: Unified Builtin Dispatch Registry

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Correctness-01 complete. Build clean.

**Verification:**
```bash
cargo check -p atlas-runtime 2>&1 | grep -c "error"  # must be 0
grep -c "is_builtin\|call_builtin" crates/atlas-runtime/src/stdlib/mod.rs
```

---

## Objective

The standard library dispatcher has a structural defect: `is_builtin(name)` and `call_builtin(name, ...)` each maintain independent lists of ~150 builtin function names as parallel string match arms. Every function call that hits a builtin incurs two full string scans — once to determine it is a builtin, once to dispatch it. Adding a new stdlib function requires updating both lists; forgetting one produces a silent runtime failure (`UnknownFunction` error at call time even though the function passes the `is_builtin` check).

Production compilers maintain a single registration point. This phase replaces the dual-match pattern with a `OnceLock`-initialized registry that maps function names to their dispatch functions. `is_builtin` becomes a hash lookup O(1). `call_builtin` becomes a single registry dispatch. New builtins are registered in one place. The duplication is structurally impossible after this phase.

---

## Files Changed

- `crates/atlas-runtime/src/stdlib/mod.rs` — add registry type, `OnceLock` initialization, rewrite `is_builtin` and `call_builtin` to use registry
- No other files change — the public API signatures of `is_builtin` and `call_builtin` remain identical

---

## Dependencies

- Correctness-01 complete (`SecurityContext` threading stable)
- No other correctness phases are prerequisites

---

## Implementation

### Step 1: Define the registry type

In `stdlib/mod.rs`, define:

```rust
use std::sync::OnceLock;
use std::collections::HashMap;

/// A builtin dispatch function: takes args, span, security, output → Result<Value, RuntimeError>
type BuiltinFn = fn(
    &[Value],
    crate::span::Span,
    &SecurityContext,
    &OutputWriter,
) -> Result<Value, RuntimeError>;

static BUILTIN_REGISTRY: OnceLock<HashMap<&'static str, BuiltinFn>> = OnceLock::new();
```

`OnceLock` guarantees the registry is initialized exactly once across all threads, with no runtime cost after the first call. `fn` pointers (not closures) are used because all builtins are pure functions — no captured state needed.

### Step 2: Write the registry initialization function

```rust
fn builtin_registry() -> &'static HashMap<&'static str, BuiltinFn> {
    BUILTIN_REGISTRY.get_or_init(|| {
        let mut m: HashMap<&'static str, BuiltinFn> = HashMap::new();
        // Core
        m.insert("print",  dispatch_print);
        m.insert("len",    dispatch_len);
        m.insert("str",    dispatch_str);
        // ... all builtins registered here ...
        m
    })
}
```

Each `dispatch_*` function is a free function with signature matching `BuiltinFn`. It extracts arguments, calls the existing implementation in the domain module (e.g., `string::split`, `math::abs`), and returns the result. The existing logic moves from the match arms into these thin dispatch functions.

### Step 3: Rewrite `is_builtin`

```rust
pub fn is_builtin(name: &str) -> bool {
    builtin_registry().contains_key(name)
}
```

One line. O(1). No duplication.

### Step 4: Rewrite `call_builtin`

```rust
pub fn call_builtin(
    name: &str,
    args: &[Value],
    call_span: crate::span::Span,
    security: &SecurityContext,
    output: &OutputWriter,
) -> Result<Value, RuntimeError> {
    match builtin_registry().get(name) {
        Some(dispatch_fn) => dispatch_fn(args, call_span, security, output),
        None => Err(RuntimeError::UnknownFunction {
            name: name.to_string(),
            span: call_span,
        }),
    }
}
```

The function signature is unchanged — all call sites in interpreter/expr.rs and vm/mod.rs remain untouched.

### Step 5: Write dispatch shims for every builtin

For each builtin, write a `dispatch_<name>` function that validates argument count and types, then delegates to the domain module. This is the same logic that was in the match arms — it moves, not changes. Example:

```rust
fn dispatch_len(
    args: &[Value],
    span: crate::span::Span,
    _security: &SecurityContext,
    _output: &OutputWriter,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }
    let length = len(&args[0], span)?;
    Ok(Value::Number(length))
}
```

The `_security` and `_output` parameters are present on all shims for uniformity; IO-performing builtins (`print`, file ops, network ops) use them.

### Step 6: Delete the old match arms

Once all dispatch shims are written and the registry is initialized, delete the original match arms from `is_builtin` and the body of `call_builtin`. The file will be shorter and cleaner.

### Step 7: Verify registry completeness

Add a test that exhaustively checks every known builtin name resolves in the registry:

```rust
#[test]
fn test_registry_completeness() {
    // Every name that was previously in is_builtin must resolve
    let known = ["print", "len", "str", "split", /* ... all names ... */];
    for name in &known {
        assert!(is_builtin(name), "Missing from registry: {}", name);
    }
}
```

This test is the replacement for the old `is_builtin` match — it serves as the exhaustive list and will fail if a name is accidentally removed from the registry.

---

## Tests

- All existing stdlib unit tests pass unchanged (behavior is identical — only dispatch mechanism changes)
- New `test_registry_completeness` test covering all ~150 builtin names
- `test_call_builtin_print`, `test_call_builtin_len`, `test_call_builtin_str` pass unchanged
- `cargo nextest run -p atlas-runtime` green

---

## Integration Points

- `stdlib/mod.rs` — registry replaces both match arms; dispatch shims replace inline logic
- `interpreter/expr.rs` and `vm/mod.rs` — unchanged: `call_builtin` signature identical
- All domain modules (`string`, `math`, `array`, etc.) — unchanged: logic stays in domain, shims delegate to it

---

## Acceptance

- `is_builtin` and `call_builtin` no longer contain match arms against string literals
- `builtin_registry()` is the single source of truth for all builtin names
- Adding a new builtin requires adding exactly one entry: the registry `m.insert()` + its dispatch shim
- `test_registry_completeness` passes and covers all ~150 builtins
- All existing tests pass unchanged
- Zero clippy warnings
- Commit: `refactor(stdlib): Replace dual match dispatch with unified OnceLock registry`
