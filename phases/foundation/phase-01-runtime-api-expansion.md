# Phase 01: Runtime API Expansion

## 🚨 BLOCKERS - CHECK BEFORE STARTING

**REQUIRED:** v0.1 runtime must be complete with Value model and execution engines.

**Verification Steps:**
1. Check STATUS.md: "v0.1.0: COMPLETE" section should show 93/93 phases
2. Verify Value enum exists:
   ```bash
   grep -n "pub enum Value" crates/atlas-runtime/src/value.rs
   ```
3. Verify execution engines exist:
   ```bash
   ls crates/atlas-runtime/src/interpreter/mod.rs
   ls crates/atlas-runtime/src/vm/mod.rs
   ```
4. Run v0.1 test suite:
   ```bash
   cargo test --lib 2>&1 | grep "test result"
   ```

**Expected from v0.1 (per STATUS.md v0.1 completion):**
- Value enum with types: Number, String, Bool, Null, Array, Object, Function
- Interpreter execution engine (100% working)
- VM execution engine (100% parity with interpreter)
- 1,391 tests passing (zero flaky tests)
- Basic compilation pipeline: lexer → parser → compiler → bytecode

**Decision Tree:**

a) If v0.1 complete (STATUS.md confirms, tests pass):
   → Proceed with phase-01

b) If v0.1 incomplete (tests failing, missing files):
   → STOP immediately
   → Report: "v0.1 must be 100% complete before v0.2 foundation"
   → Fix v0.1 issues first
   → Do NOT proceed until cargo test --lib passes

c) If STATUS.md says complete but tests fail:
   → Investigate which v0.1 component is broken
   → Fix broken component
   → Verify all 1,391 tests pass
   → Then proceed with phase-01

**No user questions needed:** v0.1 status is verifiable via STATUS.md and cargo test.

---

## Objective
Create comprehensive public API for embedding Atlas in Rust applications enabling external programs to create runtimes, execute code, call functions, and convert values between Rust and Atlas seamlessly.

## Files
**Create:** `crates/atlas-runtime/src/api/mod.rs` (~600 lines)
**Create:** `crates/atlas-runtime/src/api/runtime.rs` (~400 lines)
**Create:** `crates/atlas-runtime/src/api/conversion.rs` (~300 lines)
**Update:** `crates/atlas-runtime/src/lib.rs` (~20 lines export API)
**Tests:** `crates/atlas-runtime/tests/api_tests.rs` (~400 lines)
**Tests:** `crates/atlas-runtime/tests/api_conversion_tests.rs` (~300 lines)

## Dependencies
- v0.1 complete Value model interpreter VM compiler
- Existing compilation pipeline lexer parser compiler
- Existing error handling RuntimeError CompileError

## Implementation

### Runtime API Structure
Create Runtime struct managing execution state across eval calls. Support two execution modes Interpreter and VM with identical semantics. Maintain global variable state persisting between eval calls. Provide eval method accepting source string returning result Value. Implement call method for invoking Atlas functions from Rust with argument list. Include set_global and get_global for variable management. Define EvalError enum encompassing parse compile and runtime errors.

### Evaluation Flow
Implement eval method orchestrating full execution pipeline. Parse source string into AST handling syntax errors. Compile AST to appropriate representation bytecode for VM or AST for interpreter. Execute using selected mode interpreter or VM. Return resulting Value or error. Maintain state across invocations functions and globals persist.

### Function Calling
Implement call method for invoking Atlas functions from Rust. Look up function by name in global scope. Convert Rust arguments to Atlas Values. Execute function with provided arguments. Return result as Value. Handle missing function errors. Support variable argument counts.

### Value Conversion Traits
Define FromAtlas trait for converting Atlas Values to Rust types. Define ToAtlas trait for converting Rust types to Atlas Values. Implement for primitives f64 String bool unit. Implement for Option mapping None to null and Some to value. Implement for Vec as Atlas arrays with element conversion. Implement for HashMap as Atlas objects with string keys. Enable automatic composition nested types like Vec of Option of String.

### Conversion Error Handling
Define ConversionError for type mismatches. Provide clear messages showing expected versus found types. Handle null to Option conversion gracefully. Validate array element types during Vec conversion. Validate object value types during HashMap conversion. Return descriptive errors for failed conversions.

### Public API Export
Update lib.rs to export API module publicly. Export Runtime struct and ExecutionMode enum. Export FromAtlas and ToAtlas traits. Make conversion functions available. Ensure API is top-level accessible. Provide ergonomic imports for library users.

## Tests (TDD - Use rstest)

**Runtime API tests:**
1. Runtime creation with default mode
2. Runtime with specific execution mode
3. Basic eval expressions and statements
4. Global state persistence across evals
5. Function definition and calling
6. Error handling parse compile runtime
7. Mode parity interpreter VM identical results
8. Complex programs with multiple evals

**Value conversion tests:**
1. Primitive f64 bidirectional conversion
2. String conversion both directions
3. Bool conversion both directions
4. Unit and null conversion
5. Option Some and None conversion
6. Vec to Array conversion
7. HashMap to Object conversion
8. Nested Vec Option String conversion
9. Conversion error cases type mismatches
10. Error messages clarity

**Minimum test count:** 80 tests (40 API, 40 conversion)

## Integration Points
- Uses: Value enum from value.rs
- Uses: Interpreter from interpreter/mod.rs
- Uses: VM from vm/mod.rs
- Uses: compile from compiler/mod.rs
- Creates: Public embedding API
- Output: Rust crate usable as library dependency

## Acceptance
- Runtime API complete new eval call globals
- Value conversion traits work FromAtlas ToAtlas
- Bidirectional conversion for primitives Vec HashMap Option
- Nested conversions work automatically
- Error handling comprehensive with clear messages
- 80+ tests pass
- Both execution modes supported interpreter VM
- Mode parity verified identical results
- Public API ergonomic for Rust developers
- No clippy warnings
- cargo test passes
