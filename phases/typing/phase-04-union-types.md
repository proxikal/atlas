# Phase 04: Union and Intersection Types

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Type system must support compound types.

**Verification:**
```bash
grep -n "Type::" crates/atlas-runtime/src/typechecker/types.rs
cargo test typechecker
ls crates/atlas-runtime/src/typechecker/mod.rs
```

**What's needed:**
- Type system with type representations
- Type checker with type operations
- Pattern matching or type guards

**If missing:** Core type system should support basics - enhancement needed

---

## Objective
Implement union and intersection types enabling flexible type composition - union types for values of multiple possible types (string | number) and intersection types for combining type constraints (Serializable & Comparable) - providing TypeScript-like type flexibility.

## Files
**Update:** `crates/atlas-runtime/src/typechecker/types.rs` (~400 lines union/intersection)
**Update:** `crates/atlas-runtime/src/typechecker/mod.rs` (~350 lines type checking)
**Update:** `crates/atlas-runtime/src/parser/mod.rs` (~150 lines parse union syntax)
**Create:** `crates/atlas-runtime/src/typechecker/narrowing.rs` (~500 lines)
**Create:** `docs/union-types.md` (~600 lines)
**Tests:** `crates/atlas-runtime/tests/union_type_tests.rs` (~600 lines)
**Tests:** `crates/atlas-runtime/tests/intersection_type_tests.rs` (~400 lines)

## Dependencies
- Type system with compound types
- Type checker with unification
- Pattern matching or type narrowing
- Control flow analysis

## Implementation

### Union Type Representation
Define union types representing values of multiple types. Union of two or more types using pipe syntax. Store member types in set avoiding duplicates. Normalize unions flattening nested unions. Simplify unions removing redundant members. Never type for empty union. Distinguish union from nullable which is union with null. Display unions with readable syntax.

### Union Type Construction
Parse union type syntax with pipe operator. Type annotation supports union types. Infer union types from control flow branches. Union of function return types from different paths. Union from conditional expressions. Flatten nested unions automatically. Validate union members are distinct types. Empty union not allowed in most contexts.

### Union Type Checking
Check value against union type succeeds if matches any member. Check operation on union requires support by all members. Access properties common to all union members. Discriminated union via type narrowing. Error messages show expected union members. Type inference with unions finds least upper bound. Assignment to union from any member type.

### Type Narrowing for Unions
Narrow union types in conditional contexts. Type guard narrows to specific member. typeof check narrows union to primitive type. Equality check with literal narrows to literal type. Pattern match narrows to matched variant. Control flow analysis tracks narrowing. Exhaustiveness checking for union coverage. Unreachable code detection after full narrowing.

### Intersection Type Representation
Define intersection types combining multiple type constraints. Intersection of two or more types using ampersand syntax. Store constituent types preserving all constraints. Simplify intersections removing redundant constraints. Any type for trivial intersection. Conflict detection for incompatible intersections like string & number. Display intersections clearly.

### Intersection Type Checking
Check value against intersection requires satisfying all types. Structural intersection merges object properties. Intersection with primitive types carefully defined. Method overload resolution with intersections. Type inference finds greatest lower bound. Assignment to intersection requires all constraints met.

### Union and Intersection Interaction
Distribute union over intersection when simplifying. Normalize complex type expressions. De Morgan's laws for type algebra. Subtyping with unions and intersections. Type equivalence considering normalization. Efficient representation avoiding explosion.

### Discriminated Unions
Support discriminated unions with tag fields. Tag field identifies union variant. Type narrowing on tag field. Exhaustiveness checking on tag values. Generate efficient code for discriminated unions. Pattern matching on discriminated unions. Error on missing union cases. Similar to Rust enums or TypeScript discriminated unions.

## Tests (TDD - Use rstest)

**Union construction tests:**
1. Parse simple union type
2. Union of primitives
3. Union of complex types
4. Flatten nested unions
5. Normalize union duplicates
6. Union type display
7. Empty union handling
8. Union with null nullable

**Union type checking tests:**
1. Value matches union member
2. Value type is union
3. Operation on union all members support
4. Access common properties
5. Error on incompatible operation
6. Type inference with unions
7. Assignment from union member
8. Assign to union from member

**Type narrowing tests:**
1. typeof narrows union
2. Equality check narrows
3. Pattern match narrows
4. Type guard function
5. Control flow narrowing
6. Exhaustiveness checking
7. Unreachable code after narrow
8. Multiple narrowing steps

**Intersection construction tests:**
1. Parse intersection type
2. Intersection of types
3. Simplify intersections
4. Intersection conflicts detected
5. Intersection display
6. Trivial intersection

**Intersection type checking tests:**
1. Value satisfies intersection
2. Structural intersection merges fields
3. Method resolution with intersection
4. Type inference with intersections
5. Assignment to intersection
6. Incompatible intersection error

**Union intersection interaction tests:**
1. Distribute union over intersection
2. Normalize complex expression
3. Subtyping with unions
4. Subtyping with intersections
5. Type equivalence

**Discriminated union tests:**
1. Define discriminated union
2. Tag field narrowing
3. Exhaustiveness on tag values
4. Pattern match on tag
5. Missing case error
6. Efficient codegen

**Integration tests:**
1. Union types across functions
2. Intersection types in interfaces
3. Complex nested types
4. Type inference with unions and intersections
5. Real-world use cases

**Minimum test count:** 80 tests

## Integration Points
- Uses: Type system from typechecker
- Uses: Parser from v0.1
- Updates: Type system with unions and intersections
- Updates: Type checker with narrowing
- Creates: Flexible type composition
- Output: TypeScript-like type flexibility

## Acceptance
- Union types work with pipe syntax
- Intersection types work with ampersand
- Type narrowing in conditionals
- Discriminated unions supported
- Exhaustiveness checking enforces coverage
- Type inference handles unions and intersections
- Error messages clear for union issues
- 80+ tests pass
- Documentation with examples
- No clippy warnings
- cargo test passes
