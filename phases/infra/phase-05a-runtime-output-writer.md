# Phase Infra-05a: Configurable Output Writer

## Blocker
**REQUIRED:** Infra-04 complete. Zero bare `#[ignore]`.

---

## Objective

Add a configurable output writer to the Atlas runtime so that `print()` writes to
any `dyn Write` target instead of hardcoding `println!()` to stdout. This is the
architectural prerequisite for the file-based corpus (Infra-05b), and the correct
design for a testable interpreter. Every serious interpreter does this: Python,
Ruby, Lua, V8 — all accept an injectable output stream.

**Contract being established:**
```rust
// Caller creates a capture buffer
let buf: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
let config = RuntimeConfig::new().with_output(buf.clone());
let runtime = Runtime::with_config(ExecutionMode::Interpreter, config);
runtime.eval(r#"print("hello")"#).unwrap();
let output = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
assert_eq!(output, "hello\n");
```

Default behavior (no `with_output` call) must still write to real stdout — no
behavior change for users.

---

## Files Changed

- `crates/atlas-runtime/src/api/config.rs` — add `output` field + `with_output()` builder
- `crates/atlas-runtime/src/api/runtime.rs` — store writer, thread to engines
- `crates/atlas-runtime/src/interpreter/mod.rs` — accept + store writer, pass to call_builtin
- `crates/atlas-runtime/src/vm/mod.rs` — same
- `crates/atlas-runtime/src/stdlib/mod.rs` — `call_builtin` + `print()` accept writer param

---

## Implementation

### Step 1: Define the writer type alias

In `stdlib/mod.rs` (or a new `src/output.rs`), define a shared type:

```rust
/// Shared, thread-safe output writer.
/// Default implementation writes to stdout.
pub type OutputWriter = Arc<Mutex<Box<dyn std::io::Write + Send>>>;

/// Construct a writer that goes to real stdout (the default).
pub fn stdout_writer() -> OutputWriter {
    Arc::new(Mutex::new(Box::new(std::io::stdout())))
}
```

### Step 2: Add to RuntimeConfig

```rust
pub struct RuntimeConfig {
    pub max_execution_time: Option<Duration>,
    pub max_memory_bytes: Option<usize>,
    pub allow_io: bool,
    pub allow_network: bool,
    /// Output destination for print(). Defaults to stdout.
    pub output: OutputWriter,
}

impl RuntimeConfig {
    pub fn new() -> Self {
        Self {
            // ... existing fields ...
            output: stdout_writer(),
        }
    }

    /// Redirect all print() output to a custom writer.
    ///
    /// # Example (capture output in tests)
    /// ```
    /// let buf = Arc::new(Mutex::new(Vec::<u8>::new()));
    /// let config = RuntimeConfig::new().with_output(buf.clone());
    /// ```
    pub fn with_output(mut self, output: OutputWriter) -> Self {
        self.output = output;
        self
    }
}
```

### Step 3: Thread through Runtime

`Runtime::new()` and `Runtime::new_with_security()` use `stdout_writer()`.
`Runtime::with_config()` uses `config.output`.

The `Runtime` struct stores the writer and passes it when constructing the
interpreter and VM.

### Step 4: Update Interpreter and VM

Both `Interpreter` and `VM` store an `OutputWriter`. They pass it to every
`call_builtin` invocation.

### Step 5: Update call_builtin and print()

```rust
pub fn call_builtin(
    name: &str,
    args: &[Value],
    call_span: Span,
    security: &SecurityContext,
    output: &OutputWriter,          // ← new parameter
) -> Result<Value, RuntimeError> {
    match name {
        "print" => {
            // ...
            print(&args[0], call_span, output)?;
            Ok(Value::Null)
        }
        // ... rest unchanged
    }
}

pub fn print(value: &Value, span: Span, output: &OutputWriter) -> Result<(), RuntimeError> {
    match value {
        Value::String(_) | Value::Number(_) | Value::Bool(_) | Value::Null => {
            let mut w = output.lock().unwrap();
            writeln!(w, "{}", value.to_display_string())
                .map_err(|_| RuntimeError::TypeError { msg: "write failed".into(), span })?;
            Ok(())
        }
        _ => Err(RuntimeError::InvalidStdlibArgument { span }),
    }
}
```

### Step 6: Update all call_builtin call sites

There are exactly two call sites in `vm/mod.rs` (lines 836, 2352) and at least
one in `interpreter/mod.rs`. Each gets `&self.output` passed as the final argument.

### Step 7: Verify no behavior change

All existing tests must pass unchanged. The default writer goes to stdout — nothing
the existing test suite observes changes. Only the corpus harness (Infra-05b) uses
the capture path.

---

## Tests

Add to `crates/atlas-runtime/src/stdlib/mod.rs` (unit tests, `#[cfg(test)]`):

```rust
#[test]
fn test_print_writes_to_custom_writer() {
    let buf: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
    let writer = Arc::new(Mutex::new(Box::new(/* wrap buf */) as Box<dyn Write + Send>));
    call_builtin("print", &[Value::string("hello")], Span::dummy(), &SecurityContext::allow_all(), &writer).unwrap();
    // assert buf contains "hello\n"
}
```

Add an integration test to `tests/api.rs` (// --- Core API --- section):

```rust
#[test]
fn test_runtime_captures_print_output() {
    use std::sync::{Arc, Mutex};
    let buf: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
    let output = Arc::new(Mutex::new(Box::new(/* VecWriter */ ) as Box<dyn std::io::Write + Send>));
    let config = RuntimeConfig::new().with_output(output.clone());
    let runtime = Runtime::with_config(ExecutionMode::Interpreter, config);
    runtime.eval(r#"print("captured")"#).unwrap();
    let s = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
    assert_eq!(s, "captured\n");
}
```

Note: the `Vec<u8>` buffer needs a thin newtype wrapper that implements `Write + Send`
(or use a crate like `std::io::Cursor<Vec<u8>>`).

---

## Acceptance

- `call_builtin` and `print()` accept `&OutputWriter` parameter
- `RuntimeConfig::with_output()` builder exists and is documented
- Default behavior unchanged: `Runtime::new()` still prints to real stdout
- All existing tests pass (`cargo nextest run -p atlas-runtime` — same pass/fail count)
- New unit test: `test_print_writes_to_custom_writer` passes
- New integration test: `test_runtime_captures_print_output` passes
- Zero clippy warnings
- Commit: `feat(runtime): Add configurable output writer to RuntimeConfig`
