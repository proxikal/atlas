# FFI (Foreign Function Interface) Guide

**Status:** Phase-10c Complete (Callbacks + Integration)
**Version:** v0.2
**Date:** 2026-02-15

## Overview

Atlas provides a complete Foreign Function Interface (FFI) system for interoperability with C libraries. FFI enables:

- **Calling C functions** from Atlas code (phase-10b)
- **Passing Atlas functions** to C code as callbacks (phase-10c)
- **Type marshaling** between Atlas and C types (phase-10a)
- **Dynamic library loading** at runtime (phase-10b)

## Table of Contents

1. [Quick Start](#quick-start)
2. [Extern Types](#extern-types)
3. [Extern Functions](#extern-functions)
4. [Type Marshaling](#type-marshaling)
5. [Callbacks](#callbacks)
6. [Memory Safety](#memory-safety)
7. [Platform Compatibility](#platform-compatibility)
8. [Common Patterns](#common-patterns)
9. [Error Handling](#error-handling)
10. [Performance](#performance)
11. [Limitations](#limitations)

---

## Quick Start

### Calling a C Function

```atlas
// Declare an extern function from the math library
extern "m" fn sqrt(x: CDouble) -> CDouble;

// Call it like a normal function
let result = sqrt(16.0);
print("sqrt(16) = " + str(result));  // prints: sqrt(16) = 4
```

### Using Multiple C Functions

```atlas
extern "m" fn pow(base: CDouble, exp: CDouble) -> CDouble;
extern "m" fn sin(x: CDouble) -> CDouble;
extern "m" fn cos(x: CDouble) -> CDouble;

fn hypotenuse(a: number, b: number) -> number {
    return sqrt(a * a + b * b);
}

let h = hypotenuse(3.0, 4.0);  // 5.0
```

---

## Extern Types

Atlas supports 6 C types for FFI:

| Atlas Type | C Type | Description | Size |
|-----------|--------|-------------|------|
| `CInt` | `int` | 32-bit signed integer | 4 bytes |
| `CLong` | `long` | Platform-dependent integer | 4-8 bytes |
| `CDouble` | `double` | 64-bit floating point | 8 bytes |
| `CCharPtr` | `char*` | C string pointer | platform |
| `CVoid` | `void` | No return value | 0 bytes |
| `CBool` | `bool` (_Bool) | Boolean value | 1 byte |

### Type Conversion Rules

**Atlas → C:**
- `number` → `CInt`, `CLong`, `CDouble`
- `string` → `CCharPtr` (null-terminated copy)
- `bool` → `CBool`
- `null` → not supported in FFI

**C → Atlas:**
- `CInt`, `CLong`, `CDouble` → `number`
- `CCharPtr` → `string` (copied, not owned)
- `CBool` → `bool`
- `CVoid` → `null`

### Examples

```atlas
// Integer operations
extern "c" fn abs(x: CInt) -> CInt;
let result = abs(-42);  // 42

// Floating point math
extern "m" fn floor(x: CDouble) -> CDouble;
let floored = floor(3.7);  // 3.0

// String functions
extern "c" fn strlen(s: CCharPtr) -> CLong;
let len = strlen("hello");  // 5

// Void return
extern "c" fn srand(seed: CInt) -> CVoid;
srand(42);  // initialize random number generator
```

---

## Extern Functions

### Declaration Syntax

```atlas
extern "library_name" fn function_name(params...) -> return_type;
```

**Components:**
- `library_name`: Dynamic library name (platform-specific resolution)
- `function_name`: Function name in Atlas (can differ from C symbol)
- `params`: Comma-separated list of `name: ExternType`
- `return_type`: One of the 6 extern types

### Symbol Renaming

Use `as "symbol"` to call a C function with a different name:

```atlas
// Call C's "strlen" but name it "string_length" in Atlas
extern "c" fn string_length as "strlen"(s: CCharPtr) -> CLong;

let len = string_length("test");  // calls strlen
```

### Library Name Resolution

Atlas automatically resolves library names to platform-specific files:

| Platform | Library Name | Resolves To |
|----------|--------------|-------------|
| Linux | `"m"` | `libm.so` |
| macOS | `"m"` | `libm.dylib` |
| Windows | `"msvcrt"` | `msvcrt.dll` |

Search paths:
1. System library directories
2. Current working directory
3. `LD_LIBRARY_PATH` (Linux) / `DYLD_LIBRARY_PATH` (macOS) / `PATH` (Windows)

### Examples

```atlas
// Math library (cross-platform)
extern "m" fn sqrt(x: CDouble) -> CDouble;
extern "m" fn pow(base: CDouble, exp: CDouble) -> CDouble;
extern "m" fn ceil(x: CDouble) -> CDouble;
extern "m" fn floor(x: CDouble) -> CDouble;

// C standard library
extern "c" fn abs(x: CInt) -> CInt;
extern "c" fn strlen(s: CCharPtr) -> CLong;

// Custom library
extern "mylib" fn custom_function(x: CInt) -> CDouble;
```

---

## Type Marshaling

Marshaling converts values between Atlas and C representations.

### Automatic Marshaling

Atlas automatically marshals arguments and return values:

```atlas
extern "m" fn pow(base: CDouble, exp: CDouble) -> CDouble;

// Atlas numbers are automatically converted to CDouble
let result = pow(2.0, 8.0);  // result is number (256.0)
```

### String Marshaling

Strings are copied and null-terminated:

```atlas
extern "c" fn strlen(s: CCharPtr) -> CLong;

// "hello" is converted to null-terminated C string
let len = strlen("hello");  // 5

// Atlas owns the original string, C gets a temporary copy
```

**Important:** C receives a *copy* of the string. Changes in C don't affect the Atlas string.

### Integer Conversion

```atlas
extern "c" fn abs(x: CInt) -> CInt;

// Atlas number (42.5) is truncated to CInt (42)
let result = abs(42.5);  // 42 (fractional part lost)
```

**Warning:** Precision may be lost when converting floating-point numbers to integers.

### Error Cases

```atlas
// These will fail at runtime:
extern "c" fn strlen(s: CCharPtr) -> CLong;

strlen(null);   // ERROR: Cannot marshal null to CCharPtr
strlen(42);     // ERROR: Cannot marshal number to CCharPtr
strlen([1,2]);  // ERROR: Cannot marshal array to CCharPtr
```

---

## Callbacks

**Phase-10c** adds support for C code calling Atlas functions.

### Creating Callbacks

Use the Runtime API to create C-callable function pointers:

```rust
// Rust embedding API
use atlas_runtime::interpreter::Interpreter;
use atlas_runtime::ffi::ExternType;

let mut interp = Interpreter::new();

// Define an Atlas function
interp.eval(source, &security)?;

// Create callback pointer
let callback_ptr = interp.create_callback(
    "my_atlas_function",
    vec![ExternType::CDouble],
    ExternType::CDouble,
)?;

// Pass to C code
unsafe {
    c_library_function(callback_ptr);
}
```

### Callback Flow

1. C code calls function pointer
2. Trampoline marshals C arguments to Atlas values
3. Atlas function executes
4. Return value is marshaled back to C type
5. C code receives result

### Supported Signatures

Currently supported callback signatures:

```atlas
fn callback_no_params() -> number;                        // () -> CInt
fn callback_one_double(x: number) -> number;             // (CDouble) -> CDouble
fn callback_two_doubles(x: number, y: number) -> number; // (CDouble, CDouble) -> CDouble
fn callback_int(x: number) -> number;                     // (CInt) -> CInt
fn callback_void(x: number) -> void;                      // (CInt) -> CVoid
```

### Example: C Callback

**C Library:**
```c
// mylib.c
typedef double (*callback_t)(double);

double call_with_callback(callback_t cb, double value) {
    return cb(value * 2.0);
}
```

**Atlas Code:**
```atlas
// atlas_program.atl
fn my_callback(x: number) -> number {
    return x + 10.0;
}

// In Rust embedding code:
// Create callback and pass to C
```

**Execution:**
1. C calls `my_callback` with value `20.0`
2. Atlas function executes: `20.0 + 10.0 = 30.0`
3. Returns `30.0` to C

### Callback Limitations

- Single-threaded execution only
- No array or object marshaling in callbacks
- Error handling returns default value (0 for numbers, null for void)
- Callback must exist for entire duration of use

---

## Memory Safety

### Safe Wrappers

Atlas provides safe wrappers for common FFI patterns:

```rust
use atlas_runtime::ffi::{
    SafeCString,
    check_null,
    BoundedBuffer,
    SafeMarshalContext,
};

// Safe C string (automatic cleanup)
let s = SafeCString::new("hello")?;
let ptr = s.as_ptr();  // Automatic cleanup on drop

// Null pointer checking
let ptr = unsafe { some_c_function() };
check_null(ptr)?;  // Returns error if null

// Bounded buffer
let buffer = BoundedBuffer::new(ptr, len)?;
let slice = buffer.as_slice();  // Safe access

// Safe marshaling
let mut ctx = SafeMarshalContext::new();
let c_value = ctx.safe_atlas_to_c(&value, &ExternType::CDouble)?;
```

### Memory Management

**Atlas manages:**
- String copies passed to C
- Callback trampolines
- Marshal context cleanup

**C manages:**
- Return values from extern functions (if pointers)
- Callback invocation

**Rules:**
1. Never store C pointers beyond function scope
2. Always use safe wrappers when possible
3. Check null pointers before dereferencing
4. Copy C strings to Atlas strings immediately

### Unsafe Operations

All FFI operations are `unsafe` in Rust:

```rust
// Loading libraries
unsafe { library_loader.load("mylib")? }

// Calling extern functions
unsafe { extern_fn.call(&args)? }

// Creating callbacks
unsafe { ExternFunction::new(fn_ptr, params, return_type) }
```

**Atlas runtime handles safety internally.** Users of the Atlas language don't see `unsafe`.

---

## Platform Compatibility

### Library Resolution

| Platform | Library Path | Extensions |
|----------|-------------|------------|
| Linux | `/lib`, `/usr/lib`, `/usr/local/lib` | `.so`, `.so.X` |
| macOS | `/usr/lib`, `/usr/local/lib`, `/opt/homebrew/lib` | `.dylib` |
| Windows | `C:\Windows\System32`, `PATH` | `.dll` |

### Calling Conventions

Atlas uses the C calling convention (`extern "C"`) by default.

### Platform-Specific Code

```atlas
// Cross-platform math
extern "m" fn sqrt(x: CDouble) -> CDouble;  // Works on Linux/macOS

// Platform-specific
#if WINDOWS
    extern "msvcrt" fn sqrt(x: CDouble) -> CDouble;
#endif
```

---

## Common Patterns

### Pattern 1: Math Operations

```atlas
extern "m" fn sqrt(x: CDouble) -> CDouble;
extern "m" fn pow(base: CDouble, exp: CDouble) -> CDouble;

fn distance(x1: number, y1: number, x2: number, y2: number) -> number {
    let dx = x2 - x1;
    let dy = y2 - y1;
    return sqrt(dx * dx + dy * dy);
}
```

### Pattern 2: String Functions

```atlas
extern "c" fn strlen(s: CCharPtr) -> CLong;
extern "c" fn strcmp(s1: CCharPtr, s2: CCharPtr) -> CInt;

fn strings_equal(a: string, b: string) -> bool {
    return strcmp(a, b) == 0;
}
```

### Pattern 3: Error Checking

```atlas
extern "c" fn some_c_function(x: CInt) -> CInt;

fn safe_call(x: number) -> Result<number, string> {
    let result = some_c_function(x);
    if (result < 0) {
        return Err("C function returned error");
    }
    return Ok(result);
}
```

### Pattern 4: Library Wrapper

```atlas
// Wrapper module for a C library
extern "mylib" fn init() -> CBool;
extern "mylib" fn process(data: CDouble) -> CDouble;
extern "mylib" fn cleanup() -> CVoid;

fn use_library() -> Result<number, string> {
    if (!init()) {
        return Err("Library initialization failed");
    }

    let result = process(42.0);
    cleanup();

    return Ok(result);
}
```

---

## Error Handling

### Library Not Found

```atlas
extern "nonexistent" fn foo() -> CInt;
foo();  // Runtime error: Failed to load library 'nonexistent'
```

### Symbol Not Found

```atlas
extern "m" fn nonexistent_function() -> CDouble;
nonexistent_function();  // Runtime error: Failed to find symbol 'nonexistent_function'
```

### Type Marshaling Errors

```atlas
extern "c" fn strlen(s: CCharPtr) -> CLong;
strlen(null);  // Runtime error: Cannot marshal null to CCharPtr
```

### Callback Errors

```rust
// Rust code
let result = interp.create_callback(
    "nonexistent",
    vec![ExternType::CDouble],
    ExternType::CDouble,
);
// Error: Function 'nonexistent' not found
```

---

## Performance

### Call Overhead

- **Extern call**: ~10-50ns (marshaling + FFI)
- **Callback**: ~50-200ns (trampoline + marshaling)
- **Native Atlas function**: ~5-10ns

### When to Use FFI

**Good for:**
- Complex math operations (sqrt, pow, trig functions)
- System calls
- Existing C libraries
- Performance-critical native code

**Bad for:**
- Tight loops (call overhead adds up)
- Simple operations (add, subtract, multiply)
- Frequent callbacks (trampoline overhead)

### Optimization Tips

1. **Batch operations:** Call C once with multiple values rather than multiple times
2. **Cache results:** Don't call the same C function repeatedly with the same arguments
3. **Use Atlas builtins first:** Built-in functions are faster than FFI
4. **Minimize marshaling:** Fewer arguments = less overhead

---

## Limitations

### v0.2 Limitations

**Not Supported:**
- Array marshaling (`number[]` ↔ `double*`)
- Struct marshaling (custom types)
- Function pointer parameters (except via callbacks)
- Variadic functions (`printf`, `scanf`)
- Thread-safe callbacks
- Callback closures with capture
- Out parameters (`int*` for output)

**Workarounds:**
- Use multiple scalar parameters instead of arrays
- Define wrapper C functions for complex types
- Single-threaded execution only

### Future Enhancements (v0.3+)

Planned features:
- Array and struct marshaling
- Multi-threaded callback execution
- Async FFI support
- Callback closures with capture
- Foreign struct definitions in Atlas

---

## Complete Examples

### Example 1: Math Library

```atlas
extern "m" fn sqrt(x: CDouble) -> CDouble;
extern "m" fn pow(base: CDouble, exp: CDouble) -> CDouble;
extern "m" fn sin(x: CDouble) -> CDouble;
extern "m" fn cos(x: CDouble) -> CDouble;

fn main() -> void {
    // Square root
    print("sqrt(16) = " + str(sqrt(16.0)));  // 4.0

    // Power
    print("2^10 = " + str(pow(2.0, 10.0)));  // 1024.0

    // Trigonometry
    let x = 0.5;
    let s = sin(x);
    let c = cos(x);
    print("sin^2 + cos^2 = " + str(s * s + c * c));  // 1.0
}
```

### Example 2: String Operations

```atlas
extern "c" fn strlen(s: CCharPtr) -> CLong;

fn string_info(text: string) -> void {
    let len = strlen(text);
    print("String: " + text);
    print("Length: " + str(len));
}

string_info("Hello, World!");  // Length: 13
```

### Example 3: Error Handling

```atlas
extern "c" fn some_function(x: CInt) -> CInt;

fn safe_wrapper(x: number) -> Result<number, string> {
    try {
        let result = some_function(x);
        if (result < 0) {
            return Err("Function failed with code: " + str(result));
        }
        return Ok(result);
    } catch (e) {
        return Err("FFI error: " + e);
    }
}
```

---

## See Also

- **Type System:** `docs/specification/types.md`
- **Runtime API:** `docs/api/runtime-api.md`
- **Security:** `docs/reference/security-model.md`
- **Examples:** `examples/ffi/`

---

## Changelog

**2026-02-15 (Phase-10c):**
- Added callback support
- Added memory safety wrappers
- Complete FFI system

**2026-02-15 (Phase-10b):**
- Added library loading
- Added extern function calls

**2026-02-15 (Phase-10a):**
- Initial FFI infrastructure
- Type marshaling
- Extern types
