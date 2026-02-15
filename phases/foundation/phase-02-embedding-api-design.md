# Phase 02: Embedding API - Custom Functions & Examples

## 🚨 BLOCKERS - CHECK BEFORE STARTING

**REQUIRED:** Foundation phase-01 (Runtime API Expansion) must be complete.

**Verification Steps:**
1. Check STATUS.md: Foundation section, phase-01 should be ✅
2. Verify API files exist:
   ```bash
   ls crates/atlas-runtime/src/api/runtime.rs
   ls crates/atlas-runtime/src/api/conversion.rs
   ls crates/atlas-runtime/src/api/mod.rs
   ```
3. Verify Runtime struct exists:
   ```bash
   grep -n "pub struct Runtime" crates/atlas-runtime/src/api/runtime.rs
   grep -n "impl Runtime" crates/atlas-runtime/src/api/runtime.rs
   ```
4. Verify conversion traits exist:
   ```bash
   grep -n "pub trait FromAtlas" crates/atlas-runtime/src/api/conversion.rs
   grep -n "pub trait ToAtlas" crates/atlas-runtime/src/api/conversion.rs
   ```
5. Run phase-01 tests:
   ```bash
   cargo test api_tests
   cargo test api_conversion_tests
   ```

**Expected from phase-01 (per acceptance criteria):**
- Runtime struct with eval() and call() methods
- FromAtlas and ToAtlas traits for type conversion
- Bidirectional conversion for primitives, Vec, HashMap, Option
- 80+ tests passing (40 API, 40 conversion)
- Both interpreter and VM execution modes supported

**Decision Tree:**

a) If phase-01 complete (STATUS.md ✅, all files exist, tests pass):
   → Proceed with phase-02

b) If phase-01 incomplete (STATUS.md ⬜ or missing files):
   → STOP immediately
   → Report: "Foundation phase-01 required before phase-02"
   → Update STATUS.md next phase to foundation/phase-01
   → Do NOT proceed

c) If phase-01 marked complete but tests failing:
   → Phase-01 is not actually complete
   → Fix phase-01 issues first
   → Verify 80+ tests pass
   → Mark phase-01 complete in STATUS.md
   → Then proceed with phase-02

**No user questions needed:** Phase-01 completion is verifiable via STATUS.md, file existence, and cargo test.

---

## Objective
Extend embedding API with custom native functions, sandboxing capabilities, and comprehensive examples demonstrating all embedding scenarios from simple scripts to complex integrations.

## Files
**Create:** `crates/atlas-runtime/src/api/native.rs` (~500 lines)
**Update:** `crates/atlas-runtime/src/api/runtime.rs` (~200 lines added)
**Update:** `crates/atlas-runtime/src/api/mod.rs` (~50 lines)
**Create:** `examples/embedding/01_hello_world.rs` (~50 lines)
**Create:** `examples/embedding/02_custom_functions.rs` (~100 lines)
**Create:** `examples/embedding/03_value_conversion.rs` (~150 lines)
**Create:** `examples/embedding/04_persistent_state.rs` (~120 lines)
**Create:** `examples/embedding/05_error_handling.rs` (~100 lines)
**Create:** `examples/embedding/06_sandboxing.rs` (~150 lines)
**Create:** `docs/embedding-guide.md` (~600 lines)
**Tests:** `crates/atlas-runtime/tests/api_native_functions_tests.rs` (~400 lines)

## Dependencies
- Phase foundation/phase-01 complete Runtime API
- Value model with Function and NativeFunction variants
- Interpreter and VM support custom natives

## Implementation

### Native Function Infrastructure
Create NativeFn type alias for native function pointers using Arc for thread safety. Implement NativeFunctionBuilder for constructing native functions with fluent API. Support fixed arity checking automatically validating argument count. Support variadic functions accepting any argument count. Wrap native functions with automatic arity validation when specified. Convert builder to Value NativeFunction variant. Handle builder validation errors.

### Runtime Native Registration
Add register_function method to Runtime for fixed arity natives. Add register_variadic method for variable argument natives. Store natives in global scope making them callable from Atlas code. Integrate with existing set_global infrastructure. Enable calling natives through both eval and call methods. Support closures capturing Rust state in native functions.

### Sandboxing Configuration
Create RuntimeConfig struct with execution limits. Include max_execution_time field for timeout enforcement. Include max_memory field for allocation limits. Include allow_io and allow_network flags for capability control. Add with_config constructor for custom configurations. Implement sandboxed constructor with restrictive defaults no IO, 5 second timeout, 10MB memory limit. Integrate config into Runtime execution flow.

### Embedding Examples
Create hello_world example showing minimal embedding. Create custom_functions example demonstrating native registration and calling. Create value_conversion example showing Rust-Atlas bidirectional conversion with collections. Create persistent_state example demonstrating global state across multiple eval calls. Create error_handling example showing all error type handling and recovery. Create sandboxing example demonstrating untrusted code execution with limits. Each example should be runnable with cargo run --example.

### Embedding Guide Documentation
Write comprehensive embedding guide covering all scenarios. Document Runtime creation with different modes and configurations. Explain eval and call methods with examples. Document native function registration fixed and variadic. Show value conversion patterns Rust to Atlas and back. Explain error handling for all error types. Include security best practices for sandboxing untrusted code. Provide performance tips VM mode, reuse runtimes, pre-register natives. Document all configuration options with use cases.

## Tests (TDD - Use rstest)

**Native function tests:**
1. Register fixed arity function
2. Register variadic function
3. Call native from eval
4. Call native from call method
5. Arity checking too few arguments
6. Arity checking too many arguments
7. Native returning error
8. Native with closure capture
9. Native with complex argument types
10. Native with complex return types

**Sandboxing tests:**
1. Time limit terminates long loop
2. IO operations fail when disabled
3. Network operations fail when disabled
4. Memory limit enforced
5. Sandboxed runtime configuration
6. Safe execution no host crash

**Example tests:**
1. All examples compile successfully
2. All examples run without errors

**Minimum test count:** 60 tests (40 natives, 20 sandboxing)

## Integration Points
- Uses: Runtime from phase 01
- Uses: Value conversion traits
- Updates: Runtime with native function support
- Updates: Runtime with sandboxing config
- Creates: 6 complete embedding examples
- Creates: Comprehensive embedding guide
- Output: Production-ready embedding API

## Acceptance
- Custom native functions work fixed arity and variadic
- Arity checking enforced automatically
- Sandboxed runtime blocks IO operations
- Time limits terminate long-running code
- All 6 examples compile and run correctly
- Embedding guide complete with all scenarios
- 60+ tests pass
- Native functions callable from eval and call
- Error handling comprehensive
- Documentation includes best practices and security
- cargo run --example works for all examples
- No clippy warnings
- cargo test passes
