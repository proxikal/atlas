# Phase 05: Generic Constraints and Bounds

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Type system must support generic types.

**Verification:**
```bash
grep -n "Generic\|TypeParam" crates/atlas-runtime/src/typechecker/types.rs
cargo test typechecker
ls crates/atlas-runtime/src/typechecker/mod.rs
```

**What's needed:**
- Type system with generic support
- Type checker with type parameter handling
- Union types from typing/phase-04 helpful

**If missing:** Generics should exist from v0.1 - enhancement for constraints

---

## Objective
Implement generic type constraints and bounds restricting type parameters to specific capabilities - enabling bounded polymorphism with constraints like T extends Comparable, T implements Serializable - providing Rust-like trait bounds for type safety.

## Files
**Update:** `crates/atlas-runtime/src/typechecker/types.rs` (~300 lines constraints)
**Update:** `crates/atlas-runtime/src/typechecker/mod.rs` (~400 lines constraint checking)
**Update:** `crates/atlas-runtime/src/parser/mod.rs` (~150 lines extends syntax)
**Create:** `crates/atlas-runtime/src/typechecker/constraints.rs` (~500 lines)
**Create:** `docs/generic-constraints.md` (~700 lines)
**Tests:** `crates/atlas-runtime/tests/constraint_tests.rs` (~700 lines)

## Dependencies
- Type system with generics
- Type checker with subtyping
- Union types for constraint combinations
- Trait or interface system (future enhancement)

## Implementation

### Constraint Syntax
Define constraint syntax using extends keyword. Type parameter followed by extends and bound type. Multiple constraints with ampersand combining bounds. Primitive type bounds for value constraints. Structural bounds defining required fields or methods. Union bounds for alternatives. Intersection bounds for multiple requirements. Constraint on function return type. Constraint on generic alias.

### Constraint Representation
Store constraints with type parameters. Bound type or set of bounds per parameter. Normalize constraints simplifying expressions. Validate constraint well-formedness. Circular constraint detection. Constraint satisfaction checking. Default constraints for unconstrained parameters. Display constraints in type signatures.

### Constraint Checking at Call Site
Validate type arguments satisfy constraints at instantiation. Check concrete type against bounds. Structural subtyping for bounds. Required field presence verification. Method signature compatibility. Constraint propagation through nested generics. Clear error message showing violated constraint. Suggest fixing constraint violation.

### Multiple Constraints
Support multiple bounds on single type parameter. Intersection of constraints all must hold. Combining structural and nominal constraints. Constraint precedence and simplification. Conflicting constraints detected as error. Redundant constraints simplified. Efficient constraint representation.

### Constraint Inference
Infer type arguments respecting constraints. Constraint-guided type inference. Reject inferences violating bounds. Backtrack on constraint violation. Find type satisfying all constraints. Report inference failure due to constraints. Ambiguity resolution with constraints. Suggest explicit type arguments when needed.

### Higher-Kinded Constraints
Constrain type constructors not just types (future). Generic over generic types. Kind checking for type parameters. Higher-order type parameters. Constraint polymorphism. Enable powerful abstractions like functors or monads.

### Practical Constraint Patterns
Common constraint patterns as examples. Comparable for types supporting comparison operators. Serializable for types convertible to strings or bytes. Numeric for arithmetic types. Iterable for collection types. Equatable for equality comparison. Constraint composition for complex requirements.

### Error Messages for Constraints
Clear error messages when constraints fail. Show expected constraint and actual type. Explain why type doesn't satisfy bound. Suggest fixes adding methods or fields. Point to constraint definition. Multiple constraint failures listed. Constraint errors in generics helpful not overwhelming.

## Tests (TDD - Use rstest)

**Constraint syntax tests:**
1. Parse extends constraint
2. Multiple constraints with ampersand
3. Primitive type bound
4. Structural bound with fields
5. Union bound
6. Intersection bound
7. Constraint on alias
8. Constraint display

**Constraint checking tests:**
1. Type argument satisfies constraint
2. Type argument violates constraint error
3. Structural subtyping check
4. Required field presence
5. Method signature compatibility
6. Constraint propagation
7. Error message clarity
8. Suggest fix for violation

**Multiple constraints tests:**
1. Intersection of constraints
2. All constraints satisfied
3. One constraint violated
4. Conflicting constraints error
5. Redundant constraint simplified
6. Constraint normalization

**Constraint inference tests:**
1. Infer type with constraints
2. Reject invalid inference
3. Backtrack on violation
4. Find satisfying type
5. Inference failure reported
6. Ambiguity with constraints
7. Suggest explicit arguments

**Practical patterns tests:**
1. Comparable constraint
2. Serializable constraint
3. Numeric constraint
4. Iterable constraint
5. Equatable constraint
6. Constraint composition
7. Custom constraint

**Generic functions tests:**
1. Generic function with bound
2. Call with satisfying type
3. Call with violating type error
4. Infer type argument with bound
5. Multiple type params with bounds
6. Nested generic with constraints

**Generic types tests:**
1. Generic type with constraint
2. Instantiate with satisfying type
3. Instantiate with violating type error
4. Constraint on type alias
5. Recursive generic with constraint

**Error message tests:**
1. Clear constraint violation message
2. Expected vs actual shown
3. Suggestion to fix
4. Constraint definition location
5. Multiple failures listed
6. Helpful not overwhelming

**Integration tests:**
1. Constraints across modules
2. Exported constrained generics
3. Constraint-based overloading
4. Real-world constraint usage
5. Performance with many constraints

**Minimum test count:** 70 tests

## Integration Points
- Uses: Type system from typechecker
- Uses: Union types from phase-04
- Updates: Type system with constraints
- Updates: Type checker with bound checking
- Creates: Bounded polymorphism
- Output: Type-safe generic constraints

## Acceptance
- Constraint syntax parses correctly
- Type arguments checked against bounds
- Multiple constraints supported
- Constraint inference works
- Error messages clear and helpful
- Common constraint patterns work
- Structural bounds functional
- 70+ tests pass
- Documentation with patterns
- No clippy warnings
- cargo test passes
