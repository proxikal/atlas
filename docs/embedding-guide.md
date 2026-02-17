# Atlas Embedding Guide

**Status:** Implemented (Phases 01–02, 10a–10c, 12, 15)  
**Last Updated:** 2026-02-15

## Overview
Embedding Atlas lets host applications execute Atlas code, register native Rust functions, enforce security policies, and interoperate with C via FFI. This guide walks through setup, native function registration, security, callbacks, and examples.

## Quick Start (Rust)

### Using the `Atlas` struct (interpreter-only, simple embedding)
```rust
use atlas_runtime::{Atlas, Value};
use atlas_runtime::security::SecurityContext;

fn main() {
    // Default: deny-all security
    let runtime = Atlas::new();
    let result = runtime.eval("1 + 2").unwrap();
    println!("{result:?}"); // Number(3.0)

    // With permissive security
    let runtime = Atlas::new_with_security(SecurityContext::allow_all());
    let result = runtime.eval(r#"print("hello")"#).unwrap();
}
```

### Using the `Runtime` struct (interpreter or VM mode)
```rust
use atlas_runtime::api::{Runtime, ExecutionMode};

fn main() {
    let mut runtime = Runtime::new(ExecutionMode::Interpreter);
    runtime.eval("let x: number = 42;").unwrap();
    let result = runtime.eval("x").unwrap();
    println!("{result:?}"); // Number(42.0)
}
```

## Security & Sandbox
- Permissions configured via `SecurityContext` (filesystem, network, process, environment).
- `SecurityContext::new()` = deny-all (default, restrictive).
- `SecurityContext::allow_all()` = permit everything (development only).
- Build scripts and native functions run under the same policy.

## FFI (C Interop)
- Extern types: `CInt`, `CLong`, `CDouble`, `CCharPtr`, `CVoid`, `CBool`.
- Declare externs in Atlas: `extern "m" fn sqrt(x: CDouble) -> CDouble;`
- Dynamic loading handled by `ffi::loader` with marshaling in `ffi::marshal`.
- Callbacks: use `ffi::callbacks::CallbackManager::create_callback` to expose Atlas functions to C; keep handle alive for pointer validity.

## Reflection
- `TypeInfo` / `ValueInfo` available to hosts; mirror functions exposed in `stdlib.reflect`.
- Useful for serializers, test discovery, and debugging.

## Examples
- `examples/embedding/01_hello_world.rs` – minimal eval
- `examples/embedding/02_custom_functions.rs` – native functions
- `examples/embedding/03_value_conversion.rs` – Value interop
- `examples/embedding/04_persistent_state.rs` – stateful host data
- `examples/embedding/05_error_handling.rs` – diagnostics
- `examples/embedding/06_sandboxing.rs` – permissions
- `examples/ffi/call_c_library.atl` – calling C
- `examples/ffi/c_callback_example.c` – callbacks from C

## Diagnostics
- All APIs return `Vec<Diagnostic>`; includes codes, spans, JSON serialization.

## Best Practices
- Keep native surface small and capability-scoped.
- Prefer deterministic functions; avoid global mutable state.
- Validate inputs at the boundary; return typed errors with spans.

