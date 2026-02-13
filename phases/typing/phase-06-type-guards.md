# Phase 06: Type Guards and Predicates

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Union types and control flow analysis must exist.

**Verification:**
```bash
ls crates/atlas-runtime/src/typechecker/narrowing.rs
cargo test union_type
grep -n "Union\|control.*flow" crates/atlas-runtime/src/typechecker/
```

**What's needed:**
- Union types from typing/phase-04
- Control flow analysis from type checker
- Type narrowing infrastructure

**If missing:** Complete typing/phase-04 first

---

## Objective
Implement user-defined type guard functions and type predicates enabling custom type narrowing - allowing developers to write is_string, is_user, has_field functions that narrow union types safely - providing TypeScript-like type guards for flexible type-safe code.

## Files
**Update:** `crates/atlas-runtime/src/typechecker/narrowing.rs` (~400 lines)
**Create:** `crates/atlas-runtime/src/typechecker/type_guards.rs` (~500 lines)
**Update:** `crates/atlas-runtime/src/parser/mod.rs` (~100 lines predicate syntax)
**Update:** `crates/atlas-runtime/src/stdlib/types.rs` (~300 lines built-in guards)
**Create:** `docs/type-guards.md` (~600 lines)
**Tests:** `crates/atlas-runtime/tests/type_guard_tests.rs` (~700 lines)

## Dependencies
- Union types from phase-04
- Type narrowing from phase-04
- Control flow analysis
- Boolean return type checking

## Implementation

### Type Predicate Syntax
Define type predicate return type syntax. Function returns boolean with type predicate annotation. Predicate syntax parameter is Type after return type. Parameter must be function parameter name. Type can be any valid type expression. Validates at compile time that narrowing is safe. Type predicate implies boolean return. Support in function signatures and lambdas.

### Built-in Type Guards
Implement standard type guard functions in stdlib. is_string function narrows to string type. is_number function narrows to number type. is_bool function narrows to boolean type. is_null function narrows to null type. is_array function narrows to array type. is_function function narrows to function type. is_object function narrows to object type. Generic is_type for runtime type checking.

### User-Defined Type Guards
Enable user-defined type guard functions. Function with boolean return and type predicate. Implementation validates type at runtime. Type checker trusts predicate in then branch. False branch narrows to complementary type. Predicate validation at definition optional. Support for complex type predicates. Compose type guards for compound checks.

### Type Narrowing Integration
Integrate type guards with narrowing system. Call to type guard narrows in then branch. Union type reduced to predicated type. Other branches exclude predicated type. Control flow tracking through guards. Narrowing persists within scope. Multiple guards combine narrowings. Exhaustiveness checking with guards.

### Discriminated Union Guards
Provide guards for discriminated unions. has_tag function checking tag field value. Narrows to specific union variant. Tag-based exhaustiveness checking. Generated guards for union types. Pattern matching alternative to guards. Efficient runtime implementation. Type-safe variant access after guard.

### Structural Type Guards
Support structural type predicates. has_field function checking field existence. Narrows to type with that field. has_method function checking method presence. Combine structural checks for interfaces. Duck typing with type safety. Structural guards with union types. Runtime field checking with type narrowing.

### Guard Composition
Compose type guards for complex predicates. Logical AND combines narrowings. Logical OR widens to union. Negation inverts narrowing. Chaining guards in sequence. Helper combinators for guards. Type-safe composition patterns. Efficient runtime evaluation.

### Performance Considerations
Optimize type guard implementation for speed. Inline simple guards when possible. Cache type check results. Avoid redundant checks. Compile-time elimination where feasible. Runtime type tag checking efficient. Benchmark guard overhead. Optimize hot path guards.

## Tests (TDD - Use rstest)

**Predicate syntax tests:**
1. Parse type predicate return type
2. Type predicate in function signature
3. Parameter is type syntax
4. Validate parameter name match
5. Type predicate with complex type
6. Predicate with lambda function
7. Invalid predicate error

**Built-in guards tests:**
1. is_string narrows to string
2. is_number narrows to number
3. is_bool narrows to boolean
4. is_null narrows to null
5. is_array narrows to array
6. is_function narrows to function
7. is_object narrows to object
8. Generic is_type function

**User-defined guards tests:**
1. Define custom type guard
2. Call guard narrows type
3. False branch excludes type
4. Complex type predicate
5. Compose type guards
6. Guard with runtime check
7. Type guard error handling

**Narrowing integration tests:**
1. Guard in if condition narrows
2. Multiple guards combine
3. Control flow tracks narrowing
4. Narrowing scope limits
5. Exhaustiveness with guards
6. Guard in while loop
7. Guard in ternary expression

**Discriminated union tests:**
1. has_tag checks tag field
2. Narrows to union variant
3. Tag exhaustiveness checking
4. Generated guards for unions
5. Type-safe variant access
6. Pattern match alternative

**Structural guards tests:**
1. has_field checks field existence
2. Narrows to type with field
3. has_method checks method
4. Combine structural checks
5. Duck typing with safety
6. Union with structural guards

**Guard composition tests:**
1. AND combines narrowings
2. OR widens to union
3. Negation inverts narrowing
4. Chain guards in sequence
5. Combinator helpers
6. Complex composition

**Performance tests:**
1. Inline simple guards
2. Cache type checks
3. Eliminate redundant checks
4. Benchmark guard overhead
5. Optimize hot paths

**Integration tests:**
1. Guards across functions
2. Guards in modules
3. Real-world validation
4. Complex type hierarchies
5. Guard-based APIs

**Minimum test count:** 80 tests

## Integration Points
- Uses: Union types from phase-04
- Uses: Narrowing from phase-04
- Uses: Control flow analysis
- Updates: Type narrowing system
- Creates: Type guard infrastructure
- Output: User-defined type narrowing

## Acceptance
- Type predicate syntax works
- Built-in type guards narrow correctly
- User-defined guards function
- Guards integrate with narrowing
- Discriminated union guards work
- Structural guards check fields
- Guard composition supported
- Performance overhead minimal
- 80+ tests pass
- Documentation with examples
- No clippy warnings
- cargo test passes
