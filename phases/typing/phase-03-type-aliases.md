# Phase 03: Type Aliases

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Type system must support basic type definitions.

**Verification:**
```bash
grep -n "Type::" crates/atlas-runtime/src/typechecker/types.rs
cargo test typechecker
ls crates/atlas-runtime/src/typechecker/mod.rs
```

**What's needed:**
- Type system from v0.1
- Type checker with type resolution
- AST supports type annotations

**If missing:** Core type system should exist from v0.1

---

## Objective
Implement type aliases enabling named type definitions for documentation, abstraction, and code clarity - supporting complex type signatures and semantic type names like UserId, Timestamp, ConfigMap enhancing code readability.

## Files
**Update:** `crates/atlas-runtime/src/ast.rs` (~100 lines add TypeAlias node)
**Update:** `crates/atlas-runtime/src/typechecker/types.rs` (~200 lines alias support)
**Update:** `crates/atlas-runtime/src/typechecker/mod.rs` (~250 lines alias checking)
**Update:** `crates/atlas-runtime/src/parser/mod.rs` (~150 lines parse type keyword)
**Create:** `docs/type-aliases.md` (~400 lines)
**Tests:** `crates/atlas-runtime/tests/type_alias_tests.rs` (~500 lines)

## Dependencies
- Type system with type representations
- Type checker with resolution
- Parser for type keyword syntax

## Implementation

### Type Alias Syntax
Define type alias declaration syntax using type keyword. Type name follows type keyword using PascalCase convention. Equals sign separates name from definition. Right-hand side is any valid type expression. Support primitive type aliases. Support compound type aliases for functions, arrays. Support generic type aliases with type parameters. Allow documentation comments on aliases. Top-level and module-scoped aliases. Export and import aliases like other declarations.

### Type Alias Resolution
Resolve type alias references to underlying types. Lookup alias name in type environment. Substitute alias with defined type. Handle recursive alias detection preventing infinite expansion. Normalize aliases for type comparison. Track alias origin for error messages. Support nested alias resolution. Cache resolved aliases for performance. Fully expand aliases before type checking.

### Generic Type Aliases
Support parameterized type aliases with type parameters. Define type parameters in angle brackets. Use type parameters in alias definition. Instantiate generic aliases with concrete types. Validate type argument count matches parameter count. Propagate type parameter constraints. Substitute type arguments for parameters. Support nested generic aliases. Infer type arguments from usage context when possible.

### Type Equivalence with Aliases
Handle type equivalence considering aliases. Structural equivalence aliases resolve to same type are equivalent. Nominal equivalence option for distinct aliases (future). Compare underlying types after alias resolution. Alias name preserved in error messages for clarity. Support explicit alias unwrapping if needed. Type display shows alias name when beneficial.

### Circular Alias Detection
Detect and prevent circular type alias definitions. Track aliases being expanded in stack. Detect cycle when alias references itself. Report circular alias error with cycle chain. Allow mutually recursive aliases with indirection. Distinguish between infinite expansion and valid recursion. Clear error messages showing circular dependency.

### Alias in Type Annotations
Use aliases in function signatures and variable types. Alias names in parameter type annotations. Alias names in return type annotations. Alias names in variable type annotations. Type checker resolves aliases during checking. Inferred types can match alias types. Error messages show alias names for clarity. IDE tooling shows both alias and expanded type.

### Documentation and Metadata
Attach documentation to type aliases. Doc comments precede alias declaration. Aliases appear in generated documentation. Metadata tracks alias source location. Reflection API exposes alias information. Deprecated alias warnings. Since version tracking for aliases. Type alias exports in module interface.

## Tests (TDD - Use rstest)

**Alias declaration tests:**
1. Declare primitive type alias
2. Declare function type alias
3. Declare array type alias
4. Generic type alias with parameter
5. Multiple type parameters
6. Alias with complex type
7. Module-scoped alias
8. Exported alias

**Alias resolution tests:**
1. Resolve alias to underlying type
2. Nested alias resolution
3. Recursive alias error detection
4. Circular alias error with chain
5. Alias lookup in scope
6. Alias cache performance
7. Full alias expansion

**Generic alias tests:**
1. Define generic alias
2. Instantiate generic alias
3. Type parameter substitution
4. Type argument count validation
5. Infer type arguments
6. Nested generic aliases
7. Constraint propagation

**Type equivalence tests:**
1. Aliases to same type equivalent
2. Different aliases to same type
3. Structural equivalence
4. Type comparison with aliases
5. Alias in union type equivalence
6. Multiple alias resolutions

**Alias in annotations tests:**
1. Alias in function parameter
2. Alias in return type
3. Alias in variable declaration
4. Inferred type matches alias
5. Type error with alias name
6. Multiple aliases in signature

**Circular detection tests:**
1. Direct circular alias error
2. Indirect circular alias chain
3. Mutually recursive aliases
4. Valid recursive alias with indirection
5. Cycle detection performance

**Documentation tests:**
1. Doc comment on alias
2. Alias in generated docs
3. Deprecated alias warning
4. Reflection API alias info
5. Alias source location tracking

**Integration tests:**
1. Alias across modules
2. Import and use alias
3. Export alias from module
4. Alias with type inference
5. Alias in complex program
6. IDE tooling support

**Minimum test count:** 60 tests

## Integration Points
- Uses: Type system from typechecker
- Uses: Parser from v0.1
- Updates: AST with TypeAlias node
- Updates: Type checker with alias resolution
- Creates: Type alias feature
- Output: Named types for better code clarity

## Acceptance
- Type alias declarations parse correctly
- Aliases resolve to underlying types
- Generic type aliases work with parameters
- Circular aliases detected and reported
- Aliases improve error message clarity
- Type equivalence handles aliases
- Documentation on aliases generated
- 60+ tests pass
- Documentation with examples
- No clippy warnings
- cargo test passes
