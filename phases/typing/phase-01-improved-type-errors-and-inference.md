# Phase 01: Improved Type Errors & Inference

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Type checker must exist from v0.1 with basic type checking.

**Verification:**
```bash
grep -n "TypeChecker\|type_check" crates/atlas-runtime/src/typechecker/mod.rs
cargo test typechecker
ls crates/atlas-runtime/src/typechecker/mod.rs
```

**What's needed:**
- Type checker from v0.1 with basic type checking
- Type system with Type enum Number String Bool etc
- Error reporting with TypeError
- Enhanced diagnostics from frontend/phase-01

**If missing:** Type checker from v0.1 should exist - verify typechecker module

---

## Objective
Improve type error messages with clear expected versus actual comparisons and helpful fix suggestions while enhancing type inference to reduce annotation requirements through bidirectional checking and return type inference.

## Files
**Update:** `crates/atlas-runtime/src/typechecker/mod.rs` (~400 lines improvements)
**Create:** `crates/atlas-runtime/src/typechecker/inference.rs` (~600 lines)
**Create:** `crates/atlas-runtime/src/typechecker/suggestions.rs` (~300 lines)
**Update:** `crates/atlas-runtime/src/typechecker/types.rs` (~200 lines add type display)
**Tests:** `crates/atlas-runtime/tests/type_improvements_tests.rs` (~500 lines)
**Tests:** `crates/atlas-runtime/tests/type_inference_tests.rs` (~400 lines)

## Dependencies
- Type checker from v0.1
- Enhanced diagnostics from frontend/phase-01 for error codes and help text
- AST with Span information

## Implementation

### Improved Type Error Messages
Enhance type error messages with clear comparisons. Format type mismatch errors showing expected type versus found type with type display names. Add contextual help suggestions for common type errors. Suggest conversion functions when applicable toNumber toString toArray. Suggest fixes for common mistakes like missing return statement or wrong operator. Include span information pointing to exact error location. Use diagnostic formatter from frontend for consistent presentation with error codes. Format complex types clearly showing function signatures, array element types, object shapes.

### Type Fix Suggestions
Create suggestion system for common type errors. Detect number-string mismatches and suggest toNumber or toString conversion. Detect missing return statements suggesting explicit return. Detect wrong operator usage suggesting correct operator. Detect undefined variables suggesting similar named variables. Detect arity mismatches showing function signature. Build suggestion database for common patterns. Integrate suggestions with diagnostic help text system.

### Enhanced Type Inference
Improve type inference reducing annotation requirements. Implement bidirectional type checking where expected type guides inference of expression. Infer function return types from all return statements finding common type. Infer variable types from initializer expressions. Infer generic type parameters from usage context. Propagate type information through expressions downward and upward. Handle recursive functions with placeholder types. Unify inferred types resolving to most specific common type. Track inference failures and emit clear errors when annotation needed.

### Return Type Inference
Implement automatic return type inference for functions. Collect all return statements in function body. Infer type of each return expression. Find least upper bound common type of all returns. Handle void functions with no explicit returns. Handle early returns and conditional returns. Report error if returns have incompatible types. Allow explicit return type to override and validate. Support mutual recursion through forward declarations.

### Generic Type Inference
Infer generic type parameters from call site usage. Match generic parameter constraints. Substitute inferred types for generic parameters. Validate substitution satisfies constraints. Report ambiguous generics when inference impossible. Support partial inference where some parameters inferred, others annotated.

### Type Display Improvements
Enhance type display for better error messages. Format function types showing parameter names and return type. Format array types showing element type. Format object types showing field names and types. Format union types showing alternatives. Format generic types showing parameters. Keep display concise but informative. Use syntax familiar to users.

## Tests (TDD - Use rstest)

**Type error message tests:**
1. Type mismatch clear expected vs actual
2. Suggestions for number-string mismatch
3. Suggestions for missing return
4. Suggestions for wrong operator
5. Suggestions for undefined variables
6. Complex type display function signatures
7. Array and object type display
8. Error location accuracy

**Type inference tests:**
1. Infer variable type from initializer
2. Infer return type from returns
3. Infer generic parameters from usage
4. Bidirectional checking expected type guides inference
5. Multiple return statements common type
6. Recursive function inference
7. Mutual recursion handling
8. Ambiguous inference error messages

**Regression tests:**
1. Previously working code still types
2. Annotations still work when provided
3. No false positives on valid code

**Minimum test count:** 80 tests (40 errors, 40 inference)

## Integration Points
- Uses: Type checker from v0.1
- Uses: Enhanced diagnostics from frontend/phase-01
- Updates: Type error formatting
- Creates: Type inference improvements
- Creates: Fix suggestion system
- Output: Better type checking user experience

## Acceptance
- Type errors show expected vs actual clearly
- Suggestions provided for 10+ common mistakes
- Help text integrated with diagnostic system
- More expressions infer without annotations
- Function return types inferred when unambiguous
- Generic parameters inferred from usage
- Bidirectional checking reduces annotations
- Complex types display clearly
- 80+ tests pass 40 errors 40 inference
- No regressions on previously valid code
- Error messages more helpful than v0.1
- Annotation requirements reduced by 30%+
- No clippy warnings
- cargo test passes
