# Phase 09: Error Handling Primitives - Result Types

## 🚨 BLOCKERS - CHECK BEFORE STARTING

**REQUIRED:** Generic type system from v0.1 must be complete.

**Verification Steps:**
1. Check STATUS.md: v0.1 completion should confirm generic types complete
2. Check spec: `docs/specification/types.md` section 5 "Generic Types"
3. Verify generic type support exists:
   ```bash
   grep -n "Generic\|TypeParam" crates/atlas-runtime/src/typechecker/types.rs | head -10
   ```
4. Verify Option<T> exists (proves generics work):
   ```bash
   grep -n "Option" crates/atlas-runtime/src/typechecker/types.rs
   cargo test | grep -i option
   ```
5. Verify pattern matching exists:
   ```bash
   grep -n "match\|Match" crates/atlas-runtime/src/ast.rs
   cargo test | grep -i pattern
   ```

**Expected from v0.1 (per STATUS.md prerequisites):**
- Generic type system: Type<T> syntax and checking
- Built-in generic types: Option<T> implemented
- Pattern matching: match statements work
- Value model: Can represent generic type instances
- All type system tests passing

**Spec Requirements (from types.md section 5.3):**
- Result<T, E> is a generic enum type
- Two variants: Ok(T) and Err(E)
- Pattern matching must work on Result values
- Type parameters T and E can be any type

**Decision Tree:**

a) If v0.1 generics complete (Option<T> exists and works):
   → Proceed with phase-09
   → Result<T,E> uses same generic infrastructure as Option<T>

b) If generics exist but incomplete (no Option<T>):
   → ERROR: v0.1 should have Option<T> per STATUS.md
   → Verify v0.1 truly complete
   → Do NOT proceed until Option<T> works

c) If no generics at all:
   → CRITICAL ERROR: v0.1 prerequisites not met
   → Check STATUS.md - was v0.1 actually completed?
   → Must complete v0.1 generic types first
   → STOP immediately

d) If pattern matching missing:
   → ERROR: Pattern matching listed in v0.1 prerequisites
   → Verify v0.1 completion status
   → Cannot implement Result<T,E> without pattern matching
   → Fix v0.1 first

**No user questions needed:** Generic type system existence is verifiable via spec, code, and tests.

---

## Objective
Implement Result<T, E> type and error handling primitives enabling explicit error propagation, error recovery, and type-safe error handling - eliminating runtime exceptions and providing Rust-like error handling for reliability.

## Files
**Create:** `crates/atlas-runtime/src/result_type.rs` (~400 lines)
**Create:** `crates/atlas-runtime/src/stdlib/result.rs` (~600 lines)
**Update:** `crates/atlas-runtime/src/value.rs` (~200 lines add Result variant)
**Update:** `crates/atlas-runtime/src/typechecker/types.rs` (~150 lines Result type)
**Update:** `crates/atlas-runtime/src/interpreter/mod.rs` (~200 lines Result handling)
**Update:** `crates/atlas-runtime/src/compiler/mod.rs` (~200 lines Result codegen)
**Create:** `docs/error-handling.md` (~800 lines)
**Update:** `docs/api/stdlib.md` (~200 lines Result API)
**Tests:** `crates/atlas-runtime/tests/result_type_tests.rs` (~600 lines)
**Tests:** `crates/atlas-runtime/tests/error_handling_tests.rs` (~500 lines)

## Dependencies
- Type system with generic support
- Pattern matching or equivalent
- Value model extensibility
- Interpreter and VM support new types

## Implementation

### Result Type Definition
Define Result<T, E> as built-in generic type with two variants Ok and Err. Ok variant contains success value of type T. Err variant contains error value of type E. Result is an enum type in type system. Values of Result type carry variant tag and wrapped value. Type checker validates Result usage with correct type parameters. Pattern matching on Result for control flow. No implicit unwrapping preventing runtime errors.

### Result Value Representation
Add Result variant to Value enum wrapping Ok or Err values. Store variant tag indicating Ok or Err. Store boxed inner value of appropriate type. Implement Display showing variant and value. Support equality comparison for Results. Enable serialization for diagnostics. Clone support for value operations. Memory-efficient representation avoiding duplication.

### Result Constructor Functions
Provide Ok and Err constructor functions in prelude. ok function takes value returns Ok Result. err function takes error returns Err Result. Type inference determines Result type parameters from usage. Functions available without import. Constructors work with any value types. Type checking validates constructor usage.

### Result Methods in Standard Library
Implement comprehensive Result API methods. is_ok method returns true if Ok variant. is_err method returns true if Err variant. unwrap method extracts Ok value or panics on Err. unwrap_or method extracts Ok value or returns default. unwrap_or_else method extracts Ok value or calls function. expect method unwraps with custom panic message. map method transforms Ok value preserving Err. map_err method transforms Err value preserving Ok. and_then method chains Results short-circuiting on Err. or_else method recovers from Err. ok method converts Result to nullable dropping Err. err method extracts Err as nullable dropping Ok.

### Error Propagation Operator
Implement question mark operator for error propagation. Operator unwraps Ok value or returns Err from function. Works only in functions returning Result. Type checks Err type compatibility. Syntactic sugar for match and early return pattern. Reduces boilerplate in error handling. Clear desugaring for debugging. Compiler or interpreter support.

### Type Checking Integration
Extend type checker for Result types. Infer Result type parameters from constructor usage. Check unwrap calls with warnings for unsafe unwrapping. Validate error propagation operator usage in Result-returning functions. Ensure Err types compatible on propagation. Suggest using Result for fallible operations. Warn on ignored Result values. Track Result flow for exhaustiveness.

### Pattern Matching Support
Enable pattern matching on Result values. Match on Ok variant binding value. Match on Err variant binding error. Exhaustiveness checking requires both variants. Type system extracts Ok and Err types in patterns. Support nested Result matching. Wildcard patterns for either variant. Guard clauses with Result patterns.

### Integration with Existing Code
Convert runtime errors to Result types where appropriate. Stdlib functions returning Result for fallible ops. File I/O returns Result with I/O errors. Network operations return Result. Parsing functions return Result. Division by zero returns Result instead of panic. Out of bounds access returns Result. Type conversion returns Result. Maintain backward compatibility where needed.

## Tests (TDD - Use rstest)

**Result type tests:**
1. Create Ok Result with value
2. Create Err Result with error
3. Type checking Ok constructor
4. Type checking Err constructor
5. Pattern match on Ok
6. Pattern match on Err
7. Exhaustiveness checking
8. Nested Result types
9. Generic type inference
10. Type parameter constraints

**Result methods tests:**
1. is_ok returns true for Ok
2. is_err returns true for Err
3. unwrap extracts Ok value
4. unwrap panics on Err
5. unwrap_or returns default on Err
6. unwrap_or_else calls function on Err
7. expect panics with message
8. map transforms Ok value
9. map preserves Err
10. map_err transforms Err value
11. and_then chains Results
12. and_then short-circuits on Err
13. or_else recovers from Err

**Error propagation tests:**
1. Question mark unwraps Ok
2. Question mark returns Err early
3. Type checking propagation
4. Err type compatibility
5. Only in Result-returning functions
6. Multiple propagations in sequence
7. Propagation in nested calls

**Integration tests:**
1. File I/O with Result
2. Division by zero Result
3. Array access Result
4. Parse number Result
5. Type conversion Result
6. Network operation Result
7. Multiple errors in sequence
8. Error recovery pattern
9. Combining Results
10. Result in loops

**Pattern matching tests:**
1. Match both variants
2. Extract Ok value in pattern
3. Extract Err value in pattern
4. Nested Result match
5. Guard clauses with Result
6. Wildcard patterns
7. Non-exhaustive match error

**Type checking tests:**
1. Infer Result type parameters
2. Unwrap safety warnings
3. Ignored Result warnings
4. Err type compatibility
5. Result flow analysis

**Minimum test count:** 100 tests

## Integration Points
- Uses: Type system from typechecker
- Uses: Value model from value.rs
- Uses: Pattern matching if exists
- Updates: Value with Result variant
- Updates: Type system with Result type
- Creates: Result stdlib module
- Creates: Error handling primitives
- Output: Type-safe error handling

## Acceptance
- Result<T, E> type works correctly
- Ok and Err constructors functional
- Pattern matching on Results works
- All Result methods implemented
- Error propagation operator functional
- Type checking validates Result usage
- Warnings for unsafe unwrapping
- Stdlib functions use Result appropriately
- 100+ tests pass
- Documentation comprehensive with examples
- Error handling guide complete
- No clippy warnings
- cargo test passes
