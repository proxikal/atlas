# Phase 10c: FFI Callbacks + Integration

## 🚨 DEPENDENCIES - CHECK BEFORE STARTING

**REQUIRED:** Phase-10b complete (library loading + extern calls)

**Verification Steps:**
1. Verify phase-10b complete:
   ```bash
   ls crates/atlas-runtime/src/ffi/loader.rs
   ls crates/atlas-runtime/src/ffi/caller.rs
   grep -n "libffi\|libloading" crates/atlas-runtime/Cargo.toml
   ```
2. Verify extern calls work:
   ```bash
   cargo test -p atlas-runtime test_call_c_sqrt -- --exact
   cargo test -p atlas-runtime test_parity_extern -- --exact
   ```
3. Check callback infrastructure doesn't exist yet:
   ```bash
   ls crates/atlas-runtime/src/ffi/callbacks.rs 2>/dev/null || echo "Not created yet"
   ```

**Expected State:**
- ✅ Phase-10a complete (type marshaling)
- ✅ Phase-10b complete (library loading + extern calls)
- ✅ 70+ FFI tests passing
- ❌ No callback support yet (we're creating it)

---

## Objective

Implement bidirectional FFI by enabling C code to call Atlas functions via callbacks. Adds trampoline generation, callback registration, memory safety wrappers, comprehensive documentation, and complete integration tests. Completes the FFI infrastructure for v0.2.

This phase makes FFI fully bidirectional: Atlas↔C in both directions.

---

## Files

**Create:** `crates/atlas-runtime/src/ffi/callbacks.rs` (~500 lines)
**Create:** `crates/atlas-runtime/src/ffi/safety.rs` (~300 lines)
**Update:** `crates/atlas-runtime/src/ffi/mod.rs` (~30 lines)
**Update:** `crates/atlas-runtime/src/interpreter/mod.rs` (~80 lines)
**Update:** `crates/atlas-runtime/src/compiler/mod.rs` (~80 lines)
**Create:** `docs/features/ffi-guide.md` (~1200 lines)
**Create:** `examples/ffi/call_c_library.atl` (~150 lines)
**Create:** `examples/ffi/c_callback_example.c` (~100 lines)
**Tests:** `crates/atlas-runtime/tests/ffi_callback_tests.rs` (~600 lines)
**Tests:** `crates/atlas-runtime/tests/ffi_integration_complete_tests.rs` (~400 lines)

**Total:** ~3440 lines (callbacks + safety + docs + examples)

---

## Implementation

### 1. Callback Infrastructure

**File:** `crates/atlas-runtime/src/ffi/callbacks.rs`

```rust
use crate::ffi::marshal::{MarshalContext, MarshalError};
use crate::ffi::types::CType;
use crate::types::{ExternType, Type};
use crate::value::Value;
use crate::interpreter::Interpreter;
use libffi::high::Closure;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub enum CallbackError {
    MarshalError(MarshalError),
    ExecutionError(String),
    InvalidSignature(String),
}

impl From<MarshalError> for CallbackError {
    fn from(e: MarshalError) -> Self {
        CallbackError::MarshalError(e)
    }
}

/// Represents an Atlas function wrapped for C calling
pub struct CallbackHandle {
    /// Function pointer that C code can call
    fn_ptr: *const (),
    /// Keep closure alive (Drop will invalidate fn_ptr)
    _closure: Box<dyn std::any::Any>,
    /// Signature for validation
    param_types: Vec<ExternType>,
    return_type: ExternType,
}

impl CallbackHandle {
    pub fn fn_ptr(&self) -> *const () {
        self.fn_ptr
    }

    pub fn signature(&self) -> (&[ExternType], &ExternType) {
        (&self.param_types, &self.return_type)
    }
}

/// Manages Atlas→C callbacks
pub struct CallbackManager {
    /// Active callbacks (kept alive for lifetime)
    callbacks: Vec<CallbackHandle>,
    /// Shared interpreter reference for execution
    interpreter: Arc<Mutex<Interpreter>>,
}

impl CallbackManager {
    pub fn new(interpreter: Arc<Mutex<Interpreter>>) -> Self {
        Self {
            callbacks: Vec::new(),
            interpreter,
        }
    }

    /// Create a C-callable function pointer for an Atlas function
    pub fn create_callback(
        &mut self,
        atlas_function_name: String,
        param_types: Vec<ExternType>,
        return_type: ExternType,
    ) -> Result<CallbackHandle, CallbackError> {
        // Validate Atlas function exists and has compatible signature
        let atlas_fn_type = {
            let interp = self.interpreter.lock().unwrap();
            interp.get_function_type(&atlas_function_name)
                .ok_or_else(|| CallbackError::ExecutionError(
                    format!("Function '{}' not found", atlas_function_name)
                ))?
        };

        self.validate_signature(&atlas_fn_type, &param_types, &return_type)?;

        // Create trampoline closure
        let interp_clone = Arc::clone(&self.interpreter);
        let fn_name = atlas_function_name.clone();
        let param_types_clone = param_types.clone();
        let return_type_clone = return_type.clone();

        // Build libffi closure
        let closure = self.build_trampoline(
            interp_clone,
            fn_name,
            param_types_clone,
            return_type_clone,
        )?;

        let fn_ptr = closure.code_ptr() as *const ();

        let handle = CallbackHandle {
            fn_ptr,
            _closure: Box::new(closure),
            param_types,
            return_type,
        };

        self.callbacks.push(handle);
        Ok(self.callbacks.last().unwrap())
    }

    fn build_trampoline(
        &self,
        interpreter: Arc<Mutex<Interpreter>>,
        function_name: String,
        param_types: Vec<ExternType>,
        return_type: ExternType,
    ) -> Result<Closure<'static>, CallbackError> {
        // Trampoline: C args → Marshal to Atlas → Call Atlas function → Marshal result → C return

        let closure = Closure::new(
            move |c_args: &[*const std::ffi::c_void]| -> *mut std::ffi::c_void {
                // Marshal C args to Atlas values
                let mut marshal_ctx = MarshalContext::new();
                let atlas_args: Vec<Value> = c_args.iter()
                    .zip(param_types.iter())
                    .map(|(c_arg, ty)| {
                        let c_value = Self::c_void_ptr_to_ctype(c_arg, ty);
                        marshal_ctx.c_to_atlas(&c_value).unwrap()
                    })
                    .collect();

                // Call Atlas function
                let result = {
                    let mut interp = interpreter.lock().unwrap();
                    interp.call_function(&function_name, &atlas_args)
                        .expect("Atlas function call failed")
                };

                // Marshal result back to C
                let c_result = marshal_ctx.atlas_to_c(&result, &return_type).unwrap();
                Self::ctype_to_c_void_ptr(&c_result)
            }
        );

        Ok(closure)
    }

    fn validate_signature(
        &self,
        atlas_type: &Type,
        param_types: &[ExternType],
        return_type: &ExternType,
    ) -> Result<(), CallbackError> {
        match atlas_type {
            Type::Function { params, return_type: ret, .. } => {
                // Check param count
                if params.len() != param_types.len() {
                    return Err(CallbackError::InvalidSignature(format!(
                        "Atlas function has {} params, extern signature has {}",
                        params.len(),
                        param_types.len()
                    )));
                }

                // Check param types are compatible
                for (atlas_param, extern_param) in params.iter().zip(param_types.iter()) {
                    if !extern_param.accepts_atlas_type(atlas_param) {
                        return Err(CallbackError::InvalidSignature(format!(
                            "Parameter type mismatch: Atlas {:?} not compatible with extern {:?}",
                            atlas_param,
                            extern_param
                        )));
                    }
                }

                // Check return type
                if !return_type.accepts_atlas_type(ret) {
                    return Err(CallbackError::InvalidSignature(format!(
                        "Return type mismatch: Atlas {:?} not compatible with extern {:?}",
                        ret,
                        return_type
                    )));
                }

                Ok(())
            }
            _ => Err(CallbackError::InvalidSignature(
                "Expected function type".to_string()
            )),
        }
    }

    // Helper methods for C void pointer conversions
    // ... (implementation details)
}
```

---

### 2. Memory Safety Wrappers

**File:** `crates/atlas-runtime/src/ffi/safety.rs`

```rust
//! Safe wrappers for common FFI patterns

use crate::ffi::marshal::MarshalContext;
use crate::ffi::types::CType;
use crate::types::ExternType;
use crate::value::Value;
use std::ffi::CString;

/// RAII wrapper for C strings ensuring cleanup
pub struct SafeCString {
    inner: CString,
}

impl SafeCString {
    pub fn new(s: &str) -> Result<Self, std::ffi::NulError> {
        Ok(Self {
            inner: CString::new(s)?,
        })
    }

    pub fn as_ptr(&self) -> *const i8 {
        self.inner.as_ptr()
    }
}

/// Safe wrapper for null pointer checks
pub fn check_null<T>(ptr: *const T) -> Result<*const T, &'static str> {
    if ptr.is_null() {
        Err("Null pointer")
    } else {
        Ok(ptr)
    }
}

/// Safe wrapper for buffer bounds checking
pub struct BoundedBuffer {
    ptr: *const u8,
    len: usize,
}

impl BoundedBuffer {
    pub fn new(ptr: *const u8, len: usize) -> Result<Self, &'static str> {
        check_null(ptr)?;
        Ok(Self { ptr, len })
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }
}

/// Safe marshaling with automatic cleanup
pub struct SafeMarshalContext {
    context: MarshalContext,
}

impl SafeMarshalContext {
    pub fn new() -> Self {
        Self {
            context: MarshalContext::new(),
        }
    }

    /// Marshal with automatic error handling
    pub fn safe_atlas_to_c(
        &mut self,
        value: &Value,
        target: &ExternType,
    ) -> Result<CType, String> {
        self.context.atlas_to_c(value, target)
            .map_err(|e| format!("Marshal error: {:?}", e))
    }

    pub fn safe_c_to_atlas(&self, c_value: &CType) -> Result<Value, String> {
        self.context.c_to_atlas(c_value)
            .map_err(|e| format!("Unmarshal error: {:?}", e))
    }
}

// Automatic Drop cleanup
impl Drop for SafeMarshalContext {
    fn drop(&mut self) {
        // MarshalContext handles cleanup automatically
    }
}
```

---

### 3. Callback Syntax Support

**Update:** `crates/atlas-runtime/src/interpreter/mod.rs`

```rust
impl Interpreter {
    /// Get a C-callable function pointer for an Atlas function
    pub fn get_callback_ptr(
        &mut self,
        function_name: &str,
        param_types: &[ExternType],
        return_type: &ExternType,
    ) -> Result<*const (), RuntimeError> {
        let handle = self.callback_manager.create_callback(
            function_name.to_string(),
            param_types.to_vec(),
            return_type.clone(),
        )?;

        Ok(handle.fn_ptr())
    }
}
```

**Example Atlas usage:**
```atlas
// Atlas function to be called from C
fn my_callback(x: number) -> number {
    return x * 2;
}

// Pass callback to C function that expects a callback
extern fn register_callback(cb: c_callback) -> void from "mylib";

// Get callback pointer (runtime provides this)
// C code will call my_callback via the pointer
```

---

### 4. Documentation

**File:** `docs/features/ffi-guide.md`

Complete FFI documentation including:
- **Overview:** What is FFI, why use it
- **Extern Types:** All 6 types with examples
- **Extern Functions:** Declaring and calling C functions
- **Type Marshaling:** Atlas↔C conversion rules
- **Callbacks:** C→Atlas function calls
- **Memory Safety:** Unsafe operations, best practices
- **Platform Compatibility:** Linux/macOS/Windows differences
- **Common Patterns:** Examples for common use cases
- **Error Handling:** FFI errors and debugging
- **Performance:** Overhead and optimization tips
- **Limitations:** What's not supported in v0.2

---

### 5. Examples

**File:** `examples/ffi/call_c_library.atl`

```atlas
// Example 1: Call C math library
extern fn sqrt(x: c_double) -> c_double from "m";
extern fn pow(base: c_double, exp: c_double) -> c_double from "m";

fn main() -> void {
    let result = sqrt(16.0);
    print("sqrt(16) = " + str(result));  // 4.0

    let power = pow(2.0, 10.0);
    print("2^10 = " + str(power));  // 1024.0
}

// Example 2: Call C string functions
extern fn strlen(s: c_char_ptr) -> c_int from "c";

fn string_length_via_c(s: string) -> number {
    return strlen(s);
}

// Example 3: Callback from C
fn atlas_callback(x: number) -> number {
    print("C called Atlas with: " + str(x));
    return x * 3;
}

extern fn call_with_callback(cb: c_callback, value: c_double) -> c_double from "mylib";

fn use_callback() -> void {
    // C library will call atlas_callback
    let result = call_with_callback(atlas_callback, 10.0);
    print("Result: " + str(result));  // 30.0
}
```

**File:** `examples/ffi/c_callback_example.c`

```c
// Companion C library for callback example

#include <stdio.h>

typedef double (*callback_t)(double);

double call_with_callback(callback_t cb, double value) {
    printf("C: Calling callback with %f\n", value);
    double result = cb(value);
    printf("C: Callback returned %f\n", result);
    return result;
}
```

---

## Tests (Use rstest + insta)

**File:** `crates/atlas-runtime/tests/ffi_callback_tests.rs`

### Callback Creation Tests (8 tests)
1. `test_create_callback_simple` - Create callback for Atlas function
2. `test_create_callback_validates_signature` - Type checking
3. `test_create_callback_missing_function` - Error on undefined function
4. `test_create_callback_signature_mismatch` - Param count error
5. `test_create_callback_multiple` - Multiple callbacks
6. `test_callback_lifetime_management` - Callbacks stay valid
7. `test_callback_function_pointer_valid` - Pointer is callable
8. `test_callback_manager_cleanup` - Proper cleanup on drop

### Callback Execution Tests (10 tests)
1. `test_c_calls_atlas_simple` - C calls Atlas, gets result
2. `test_c_calls_atlas_with_params` - Multiple parameters
3. `test_c_calls_atlas_string_param` - String marshaling in callback
4. `test_c_calls_atlas_number_return` - Number return marshaling
5. `test_c_calls_atlas_void_return` - Void return handling
6. `test_c_calls_atlas_bool_param` - Bool marshaling
7. `test_callback_exception_handling` - Atlas error in callback
8. `test_callback_nested_calls` - Callback calls another function
9. `test_callback_state_access` - Callback accesses interpreter state
10. `test_callback_concurrent_safety` - Thread-safety (basic)

### Memory Safety Tests (7 tests)
1. `test_safe_cstring_wrapper` - SafeCString RAII
2. `test_null_pointer_check` - check_null catches null
3. `test_bounded_buffer_validation` - Buffer bounds checked
4. `test_safe_marshal_error_handling` - Errors wrapped safely
5. `test_callback_memory_leak_detection` - No leaks
6. `test_string_allocation_cleanup` - CString cleanup
7. `test_callback_drop_invalidates_pointer` - Lifetime safety

**File:** `crates/atlas-runtime/tests/ffi_integration_complete_tests.rs`

### Complete Integration Tests (15 tests)
1. `test_full_ffi_flow_interpreter` - Declare → Call → Callback (interpreter)
2. `test_full_ffi_flow_vm` - Declare → Call → Callback (VM)
3. `test_parity_callback_execution` - Interpreter == VM callbacks
4. `test_real_c_library_integration` - Actual libm usage
5. `test_complex_callback_chain` - C→Atlas→C→Atlas
6. `test_error_propagation_through_ffi` - Errors surface correctly
7. `test_performance_ffi_overhead` - Measure call cost
8. `test_ffi_with_module_system` - Extern in imported modules
9. `test_ffi_with_generics` - Extern with Result<T,E> returns
10. `test_ffi_platform_compatibility` - Works on current platform
11. `test_multiple_libraries_callbacks` - Multiple C libs + callbacks
12. `test_ffi_safety_wrappers_prevent_crashes` - Safety wrappers work
13. `test_ffi_documentation_examples` - All doc examples run
14. `test_extern_in_repl` - REPL supports extern declarations
15. `test_ffi_stress_test` - 1000 calls, no leaks/crashes

**Minimum test count:** 40 tests (20 interpreter + 20 VM for parity)

---

## Integration Points

- **Uses:** Phase-10a marshaling
- **Uses:** Phase-10b library loading
- **Creates:** Bidirectional FFI (Atlas↔C)
- **Creates:** Safety wrappers
- **Creates:** Complete FFI documentation
- **Creates:** FFI examples
- **Completes:** Foundation phase-10 (FFI infrastructure)
- **Output:** Production-ready FFI system

---

## Acceptance Criteria

- [ ] `CallbackManager` creates C-callable function pointers
- [ ] Callbacks validate Atlas function signatures
- [ ] Trampoline marshals C→Atlas→C correctly
- [ ] Callback lifetime management prevents use-after-free
- [ ] Memory safety wrappers (SafeCString, check_null, BoundedBuffer)
- [ ] SafeMarshalContext provides RAII cleanup
- [ ] Interpreter supports callback registration
- [ ] VM supports callback registration
- [ ] C code can call Atlas functions via callbacks
- [ ] Callbacks handle exceptions gracefully
- [ ] Complete FFI documentation (ffi-guide.md)
- [ ] Working examples (call_c_library.atl + c_callback_example.c)
- [ ] All documentation examples tested
- [ ] 40+ tests pass (20 interpreter + 20 VM)
- [ ] 100% interpreter/VM parity for callbacks
- [ ] No memory leaks detected
- [ ] No clippy warnings
- [ ] `cargo fmt` applied
- [ ] Platform compatibility verified

---

## Final Phase-10 Verification

After completing phase-10c, verify complete FFI system:

```bash
# All FFI tests pass
cargo test -p atlas-runtime ffi -- --exact

# Type marshaling works (phase-10a)
cargo test -p atlas-runtime test_marshal -- --exact

# Extern calls work (phase-10b)
cargo test -p atlas-runtime test_call_c -- --exact

# Callbacks work (phase-10c)
cargo test -p atlas-runtime test_c_calls_atlas -- --exact

# Parity verified
cargo test -p atlas-runtime test_parity_callback -- --exact

# Examples run
atlas examples/ffi/call_c_library.atl
```

**Total phase-10 tests:** 110+ (30 + 40 + 40)
**Total phase-10 lines:** ~6900 (1355 + 2130 + 3440)

---

## Notes

**Callback Safety:**
- Callbacks keep interpreter lock during execution
- Exception handling prevents C code crashes
- Lifetime management via CallbackHandle
- Automatic cleanup on Drop

**Performance:**
- Callback overhead: trampoline + 2x marshaling
- Acceptable for infrequent calls
- Not suitable for tight loops (use direct extern calls)

**Limitations (v0.2):**
- No array marshaling in callbacks
- No struct marshaling
- Single-threaded callback execution (interpreter lock)
- No variadic callbacks

**Future Enhancements (v0.3+):**
- Array/struct marshaling
- Multi-threaded callback execution
- Async FFI support
- Callback closures with capture

**Phase-10 Complete:**
This completes the FFI infrastructure for v0.2, enabling full Atlas↔C interoperability. Update STATUS.md to mark foundation/phase-10 (all three sub-phases) as complete.
