# Phase 10: FFI - Foreign Function Interface

## 🚨 BLOCKERS - CHECK BEFORE STARTING

**REQUIRED:** Type system and runtime from v0.1 for FFI extension.

**Verification Steps:**
1. Check spec: `docs/specification/types.md` section 7 "Foreign Types" (if exists)
2. Verify type system exists:
   ```bash
   grep -n "pub enum Type" crates/atlas-runtime/src/typechecker/types.rs | head -3
   cargo test typechecker 2>&1 | grep "test result"
   ```
3. Verify runtime/value model exists:
   ```bash
   ls crates/atlas-runtime/src/value.rs
   ls crates/atlas-runtime/src/interpreter/mod.rs
   ```
4. Check if extern types already exist:
   ```bash
   grep -n "Extern\|Foreign\|FFI" crates/atlas-runtime/src/typechecker/types.rs
   ```

**Spec Check (types.md section 7):**
- Read `docs/specification/types.md` section 7 to see if extern types are defined
- If spec section exists: Implement exactly per spec
- If spec section doesn't exist: Extern types are NEW for v0.2 FFI phase

**Expected from v0.1 (sufficient for FFI):**
- Type enum with variants for all Atlas types
- Value enum with runtime representations
- Type checker can validate function signatures
- Runtime can call functions dynamically

**Decision Tree:**

a) If v0.1 type system exists (Type enum found):
   → Proceed with phase-10
   → Add Extern type variant to Type enum
   → Implement per spec if spec defines it

b) If spec defines extern types (section 7 exists):
   → Read spec section 7 completely
   → Implement extern types exactly per spec
   → Add Type::Extern variant matching spec
   → Log: "Implemented extern types per types.md section 7"

c) If spec doesn't define extern types:
   → Extern types are NEW for this phase
   → Design minimal FFI-compatible type representation
   → C-compatible primitives: int, double, char*, void
   → Document design decisions in implementation
   → Consider proposing spec addition after implementation

d) If type system missing entirely:
   → CRITICAL ERROR: v0.1 incomplete
   → STOP immediately
   → Verify v0.1 completion in STATUS.md

**No user questions needed:** Spec and v0.1 infrastructure are verifiable. If spec silent on extern types, implement minimal C-compatible design.

---

## Objective
Implement Foreign Function Interface enabling Atlas code to call C functions and vice versa - allowing integration with existing native libraries, system APIs, and embedding Atlas in other applications for maximum ecosystem compatibility.

## Files
**Create:** `crates/atlas-runtime/src/ffi/mod.rs` (~800 lines)
**Create:** `crates/atlas-runtime/src/ffi/types.rs` (~400 lines)
**Create:** `crates/atlas-runtime/src/ffi/loader.rs` (~300 lines)
**Create:** `crates/atlas-runtime/src/ffi/callbacks.rs` (~400 lines)
**Update:** `crates/atlas-runtime/src/typechecker/types.rs` (~200 lines extern types)
**Update:** `crates/atlas-runtime/src/interpreter/mod.rs` (~150 lines FFI calls)
**Update:** `crates/atlas-runtime/src/compiler/mod.rs` (~150 lines FFI codegen)
**Create:** `docs/ffi-guide.md` (~1000 lines)
**Create:** `examples/ffi/call_c_library.atl` (~100 lines)
**Tests:** `crates/atlas-runtime/tests/ffi_tests.rs` (~700 lines)

## Dependencies
- libffi or similar for dynamic calls
- libloading for dynamic library loading
- C compiler for test fixtures
- Type system extensibility

## Implementation

### Extern Function Declarations
Define extern syntax for foreign function declarations. Extern keyword marks functions as foreign. Function signature specifies C-compatible parameter and return types. Optional library name for loading shared libraries. Optional calling convention specification. Type checker validates extern signatures use only FFI-safe types. No function body for extern declarations. Link extern functions at runtime or compile time.

### FFI Type System
Map Atlas types to C types and vice versa. Number maps to c_double by default. Integers map to c_int, c_long sized variants. String maps to c_char pointer with null termination. Bool maps to c_bool. Arrays map to pointer and length pair. Structs map to C structs with compatible layout. Pointers for pass-by-reference. Opaque types for C handles. Type annotations specify exact C types. Validate type compatibility at FFI boundary.

### Dynamic Library Loading
Load shared libraries at runtime using libloading. Search library paths platform-specific defaults. Load library by name platform-specific extension handling. Lookup symbols by name in loaded library. Cache loaded libraries avoiding duplicates. Unload libraries on cleanup. Handle load errors missing library or symbol. Platform-specific library naming conventions.

### Function Call Marshaling
Marshal Atlas values to C representations. Convert Atlas strings to null-terminated C strings. Convert Atlas arrays to C pointers and lengths. Box heap values for pointer passing. Convert C return values to Atlas values. Handle null pointers as nullable values. Free allocated C memory appropriately. Error handling for marshaling failures.

### Callback Support
Enable C code calling Atlas functions. Wrap Atlas functions as C function pointers. Generate trampolines handling calling conventions. Marshal arguments from C to Atlas. Marshal return values from Atlas to C. Manage Atlas runtime state in callbacks. Handle exceptions in callbacks gracefully. Lifetime management for callback closures.

### Memory Safety Considerations
Document unsafe FFI operations clearly. Validate pointer arguments not null when required. Detect buffer overflows at boundaries. Prevent use-after-free with lifetime tracking. Mark FFI code as unsafe requiring explicit opt-in. Provide safe wrappers for common patterns. Runtime checks for memory safety. Guidelines for safe FFI usage.

### Platform Compatibility
Support FFI on all major platforms Linux, macOS, Windows. Handle platform-specific calling conventions. Map platform-specific types correctly. Conditional compilation for platform differences. Test FFI on all target platforms. Document platform-specific behaviors. Provide platform-specific examples.

## Tests (TDD - Use rstest)

**Extern declaration tests:**
1. Declare extern function
2. Call extern function from C library
3. Type check extern signature
4. Unsupported type in extern error
5. Library not found error
6. Symbol not found error
7. Multiple extern functions

**Type marshaling tests:**
1. Marshal number to c_double
2. Marshal string to c_char pointer
3. Marshal array to pointer and length
4. Marshal bool to c_bool
5. Unmarshal C return value
6. Null pointer handling
7. Type conversion errors
8. Struct marshaling

**Library loading tests:**
1. Load shared library by name
2. Lookup symbol in library
3. Cache loaded library
4. Unload library
5. Library not found error
6. Symbol not found error
7. Platform-specific naming

**Callback tests:**
1. Pass Atlas function to C as callback
2. C code calls Atlas callback
3. Callback marshals arguments
4. Callback returns value to C
5. Callback exception handling
6. Multiple callbacks
7. Callback lifetime management

**Memory safety tests:**
1. Null pointer check
2. Buffer bounds validation
3. String null termination
4. Memory leak detection
5. Use-after-free prevention
6. Safe wrapper patterns

**Integration tests:**
1. Call standard C library function
2. Use OS system API
3. Integrate third-party library
4. Bidirectional C-Atlas calls
5. Complex data structure passing
6. Performance overhead measurement

**Minimum test count:** 80 tests

## Integration Points
- Uses: Type system from typechecker
- Uses: Value model from value.rs
- Uses: Runtime from interpreter and VM
- Updates: Type system with extern types
- Creates: FFI infrastructure
- Creates: C interop layer
- Output: Native code integration capability

## Acceptance
- Extern function declarations work
- Call C library functions from Atlas
- Call Atlas functions from C via callbacks
- Type marshaling handles all supported types
- Load shared libraries dynamically
- Memory safety checks functional
- Platform compatibility verified
- 80+ tests pass on all platforms
- Documentation with safety guidelines
- Examples demonstrate common patterns
- Performance overhead acceptable
- No clippy warnings
- cargo test passes
