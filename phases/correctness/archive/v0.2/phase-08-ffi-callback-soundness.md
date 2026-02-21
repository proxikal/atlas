# Phase Correctness-08: FFI Callback Soundness

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Correctness-07 complete. Build passes. Suite green.

**Verification:**
```bash
cargo check -p atlas-runtime 2>&1 | grep -c "error"  # must be 0
cargo nextest run -p atlas-runtime 2>&1 | tail -3
```

---

## Objective

`ffi/callbacks.rs` has a fundamental soundness bug: it takes a Rust closure, boxes it, then casts `closure.as_ref() as *const _ as *const ()` to produce a "function pointer" for C code. This is **undefined behavior**. A Rust closure is a struct containing captured data — its address is the address of the data, not a callable function entry point. C code calling this pointer will jump to closure data bytes interpreted as machine instructions.

The existing tests only verify that `fn_ptr` is non-null and that `CallbackHandle` constructs — they never actually *call* the function pointer from C, which is why the bug hasn't manifested.

The correct approach for Rust→C callbacks uses `extern "C" fn` trampolines with the closure passed as an opaque `*mut c_void` context pointer (the standard pattern used by libffi, Lua C API, GLFW, etc.). Each trampoline is a real `extern "C"` function that receives a context pointer, casts it back to the closure, and calls it.

---

## Files Changed

- `crates/atlas-runtime/src/ffi/callbacks.rs` — rewrite trampoline generation using `extern "C" fn` stubs
- `crates/atlas-runtime/src/ffi/caller.rs` — update `ExternFunction::call` to pass context pointer for callbacks (if applicable)

---

## Dependencies

- Correctness-07 complete
- No other phases are prerequisites

---

## Implementation

### Step 1: Understand the C callback convention

The standard C callback pattern is:
```c
typedef int (*callback_t)(void* context, int arg);
void register_callback(callback_t cb, void* context);
```

The caller passes both a function pointer AND an opaque context pointer. The function pointer is a real `extern "C" fn`, and the context carries the closure data. This is how every professional C callback API works.

### Step 2: Define extern "C" trampoline functions

For each supported signature, define a static trampoline:

```rust
unsafe extern "C" fn trampoline_void_to_int(context: *mut std::ffi::c_void) -> c_int {
    let callback = &*(context as *const Box<dyn Fn(&[Value]) -> Result<Value, RuntimeError>>);
    match callback(&[]) {
        Ok(Value::Number(n)) => n as c_int,
        _ => 0,
    }
}
```

These are real function pointers — they have stable addresses and conform to the C calling convention.

### Step 3: Update CallbackHandle

```rust
pub struct CallbackHandle {
    /// Real extern "C" function pointer — safe to pass to C
    trampoline: *const (),
    /// Opaque context pointer — must be passed alongside trampoline
    context: *mut std::ffi::c_void,
    /// Prevent closure from being freed while callback is live
    _closure: Box<dyn std::any::Any>,
    param_types: Vec<ExternType>,
    return_type: ExternType,
}
```

The `trampoline` is the real callable function. The `context` is the address of the boxed closure. Both must be passed to C code — the C function receives the context as its first argument.

### Step 4: Update create_callback

Replace each match arm. Instead of:
```rust
let fn_ptr = closure.as_ref() as *const _ as *const ();  // UB!
```

Do:
```rust
let context = Box::into_raw(closure) as *mut std::ffi::c_void;
let trampoline = trampoline_void_to_int as *const ();
```

`Box::into_raw` transfers ownership — the closure is heap-allocated and its address is stable. The `CallbackHandle`'s `Drop` impl must call `Box::from_raw` to reclaim it.

### Step 5: Implement Drop for CallbackHandle

```rust
impl Drop for CallbackHandle {
    fn drop(&mut self) {
        // SAFETY: context was created by Box::into_raw in create_callback
        unsafe {
            let _ = Box::from_raw(self.context as *mut Box<dyn Fn(&[Value]) -> Result<Value, RuntimeError>>);
        }
    }
}
```

### Step 6: Update the public API

`CallbackHandle::fn_ptr()` becomes `CallbackHandle::trampoline()` and `CallbackHandle::context()`. Document that C code must call `trampoline(context, ...)` — the context is always the first argument.

If the current FFI caller code (`caller.rs`) invokes callbacks, update it to pass the context pointer.

### Step 7: Add real invocation tests

The existing tests only check construction. Add tests that actually call the trampoline through the context:

```rust
#[test]
fn test_callback_actually_callable() {
    let handle = create_callback(
        |_| Ok(Value::Number(42.0)),
        vec![],
        ExternType::CInt,
    ).unwrap();

    // Actually call the trampoline — this is what C code would do
    let trampoline: unsafe extern "C" fn(*mut std::ffi::c_void) -> c_int =
        unsafe { std::mem::transmute(handle.trampoline()) };
    let result = unsafe { trampoline(handle.context()) };
    assert_eq!(result, 42);
}
```

This test would have caught the original bug — it would crash or return garbage under the old implementation.

---

## Tests

- `test_callback_actually_callable` — void→int trampoline invoked through context, returns correct value
- `test_callback_double_callable` — double→double trampoline invoked, returns correct value
- `test_callback_binary_callable` — (double, double)→double trampoline invoked correctly
- `test_callback_int_callable` — int→int trampoline invoked correctly
- `test_callback_void_return_callable` — void→void trampoline invoked without crash
- `test_callback_error_handling` — callback that returns Err produces default value (not crash)
- `test_callback_drop_frees_closure` — verify no memory leak when CallbackHandle is dropped
- All existing FFI tests pass
- Zero `closure.as_ref() as *const _ as *const ()` patterns remain in codebase

---

## Acceptance

- Zero instances of `closure.as_ref() as *const _ as *const ()` in `ffi/callbacks.rs`
- All trampolines are `extern "C" fn` with proper calling convention
- `CallbackHandle` carries separate `trampoline` and `context` pointers
- `Drop` implementation reclaims the boxed closure
- Tests actually invoke trampolines and verify return values
- All existing tests pass: `cargo nextest run -p atlas-runtime`
- Zero clippy warnings: `cargo clippy -p atlas-runtime -- -D warnings`
- Commit: `fix(ffi): Replace unsound closure-as-fn-ptr with extern "C" trampolines`
