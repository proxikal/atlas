# BLOCKER 03: Pattern Matching

**Category:** Foundation - Language Feature
**Blocks:** Result<T,E> usage, Option<T> handling, robust error handling
**Estimated Effort:** 2-3 weeks
**Complexity:** High

---

## 🚨 BLOCKERS - CHECK BEFORE STARTING

**REQUIRED:** Generic type system must be complete.

**Verification:**
```bash
grep -n "Type::Generic" crates/atlas-runtime/src/types.rs
cargo test generics --no-fail-fast
ls crates/atlas-runtime/tests/generics_*.rs
```

**What's needed:**
- BLOCKER 02: Generic Type Parameters complete
- Option<T> and Result<T,E> types working
- Type checker handles generics correctly

**If missing:** Complete BLOCKER 02 first. Pattern matching builds on generics.

---

## Objective

Add pattern matching to Atlas for destructuring and conditional logic. Essential for ergonomic Result<T,E> and Option<T> handling. Enables exhaustiveness checking and safer code.

**Core use case:**
```atlas
match result {
    Ok(value) => // handle success with value
    Err(error) => // handle error
}
```

---

## Background

Atlas has `match` keyword reserved but not implemented. Pattern matching needed for:
- Result type error handling
- Option type null checking
- Discriminating generic variants

**Design:**Match Rust/Swift style with exhaustiveness checking, not JavaScript/TypeScript style (no fallthrough, requires all cases).

---

## Files

### Create
- `crates/atlas-runtime/src/ast/patterns.rs` (~300 lines) - Pattern AST nodes
- `crates/atlas-runtime/src/parser/patterns.rs` (~400 lines) - Pattern parsing
- `crates/atlas-runtime/src/typechecker/patterns.rs` (~500 lines) - Pattern type checking + exhaustiveness
- `crates/atlas-runtime/src/interpreter/patterns.rs` (~300 lines) - Pattern matching execution

### Modify
- `src/ast.rs` - Add MatchExpr to Expr enum
- `src/parser/expr.rs` - Parse match expressions
- `src/typechecker/expr.rs` - Type check match expressions
- `src/interpreter/expr.rs` - Evaluate match expressions
- `src/compiler/expr.rs` - Compile match expressions
- `src/vm/mod.rs` - Execute match bytecode

### Tests
- `tests/pattern_matching_tests.rs` (~800 lines)
- `tests/vm_pattern_matching_tests.rs` (~800 lines)

**Minimum test count:** 120+ tests

---

## Implementation

### Step 1: Pattern Syntax Design
Define pattern syntax. Recommendation: Rust-style.

**Patterns:**
- Literal: `42`, `"hello"`, `true`, `null`
- Wildcard: `_` (matches anything)
- Variable binding: `x`, `value`, `error`
- Constructor: `Ok(value)`, `Err(error)`, `Some(x)`
- Array: `[first, second]`, `[head, ...tail]` (if rest patterns supported)

**Match syntax:**
```atlas
match expression {
    pattern1 => expression1,
    pattern2 => expression2,
    _ => default_expression
}
```

### Step 2: AST Representation
Add Pattern enum with variants for each pattern type. Add MatchExpr with scrutinee (expression being matched) and arms (pattern + expression pairs). Ensure all patterns have spans for error reporting.

```rust
enum Pattern {
    Literal(Literal, Span),
    Wildcard(Span),
    Variable(Identifier),
    Constructor { name: String, args: Vec<Pattern>, span: Span },
    // ... more patterns
}

struct MatchArm {
    pattern: Pattern,
    body: Expr,
    span: Span,
}

struct MatchExpr {
    scrutinee: Box<Expr>,
    arms: Vec<MatchArm>,
    span: Span,
}
```

### Step 3: Parser Implementation
Parse `match` keyword and expression. Parse match arms with patterns and `=>`. Require comma between arms (or newline). Error on missing closing brace. Store patterns in AST.

**Challenges:**
- Disambiguating constructor patterns from expressions
- Handling nested patterns
- Syntax error recovery

### Step 4: Type Checking
Check scrutinee expression type. For each pattern, check pattern is compatible with scrutinee type. Ensure pattern variables don't shadow incorrectly. Check arm bodies have compatible types (all arms must return same type). **Exhaustiveness checking:** Verify all possible values covered.

**Exhaustiveness algorithm:**
- For enums (Result, Option): Check all constructors present
- For literals: Check all values or wildcard present
- For nested patterns: Recursive exhaustiveness check

### Step 5: Pattern Matching Execution (Interpreter)
Evaluate scrutinee. For each arm in order, attempt to match pattern against value. If match succeeds, bind variables and evaluate arm body. If match fails, try next arm. Error if no arms match and no wildcard (should be caught by exhaustiveness).

**Pattern matching:**
- Literal: Compare values for equality
- Wildcard: Always matches
- Variable: Always matches, bind variable
- Constructor: Check constructor name, recursively match arguments

### Step 6: VM Compilation
Compile match into jump table or series of conditional jumps. For simple cases (few arms), use if-else chain. For complex cases (many arms), use jump table. Compile exhaustiveness checks into assertions.

### Step 7: Result/Option Integration
Ensure Ok/Err and Some/None constructors available. These are built-in constructors for Result<T,E> and Option<T>. Type checker knows their types. Pattern matching can destructure them.

### Step 8: Comprehensive Testing
Test literal patterns (number, string, bool, null). Test wildcard and variable patterns. Test constructor patterns (Ok/Err, Some/None). Test nested patterns. Test exhaustiveness checking (errors when non-exhaustive). Test all arms return compatible types. Test variable binding and shadowing. Full parity.

---

## Architecture Notes

**Exhaustiveness checking is critical:** Prevents bugs by forcing all cases to be handled. Better compile-time error than runtime panic.

**Pattern matching is an expression:** Match expression has a type (union of all arm types). Can use in assignments, returns, etc.

**Constructor patterns require generics:** Can't match on Ok/Err without generic support. This is why generics are prerequisite.

**Rust reference:** Atlas pattern matching should feel like Rust - safe, exhaustive, expressive.

---

## Acceptance Criteria

**Functionality:**
- ✅ Match expression syntax parses
- ✅ All pattern types supported (literal, wildcard, variable, constructor)
- ✅ Type checking validates pattern compatibility
- ✅ Exhaustiveness checking works
- ✅ Pattern execution binds variables correctly
- ✅ Match expressions compose (can nest matches)
- ✅ Result and Option patterns work

**Quality:**
- ✅ 120+ tests pass
- ✅ 100% interpreter/VM parity
- ✅ Zero clippy warnings
- ✅ All code formatted
- ✅ Exhaustiveness catches all gaps
- ✅ Clear error messages for non-exhaustive matches

**Documentation:**
- ✅ Update Atlas-SPEC.md with match syntax
- ✅ Examples in docs/features/pattern-matching.md
- ✅ Document exhaustiveness algorithm
- ✅ Result/Option usage examples

---

## Dependencies

**Requires:**
- BLOCKER 02: Generic Type Parameters
- Result<T,E> and Option<T> types defined
- Type system stable

**Blocks:**
- Foundation Phase 9: Error Handling (Result usage patterns)
- Any phase using Result or Option types
- Advanced error handling patterns

---

## Rollout Plan

1. Design syntax (2 days)
2. AST implementation (2 days)
3. Parser implementation (4 days)
4. Type checking (5 days)
5. Exhaustiveness checking (4 days)
6. Interpreter execution (3 days)
7. VM compilation (4 days)
8. Result/Option integration (2 days)
9. Testing and polish (4 days)

**Total: ~30 days (4-5 weeks with testing)**

---

## Known Limitations

**No guard clauses yet:** Can't have `pattern if condition =>`. Guards come later if needed.

**No OR patterns yet:** Can't do `Ok(x) | Some(x) =>` to match multiple patterns in one arm. Can add later.

**No slice patterns yet:** Array pattern matching basic (fixed length only), no `[first, ...rest]` syntax.

**No struct patterns yet:** No user-defined structs, so no struct patterns. Built-in types (Result, Option) only.

These can be added incrementally. Focus on core feature first.

---

## Examples

**Result error handling:**
```atlas
fn divide(a: number, b: number) -> Result<number, string> {
    if (b == 0) {
        return Err("division by zero");
    }
    return Ok(a / b);
}

let result = divide(10, 2);
match result {
    Ok(value) => print(str(value)),
    Err(error) => print(error)
}
```

**Option handling:**
```atlas
fn find(arr: number[], target: number) -> Option<number> {
    for (var i = 0; i < len(arr); i++) {
        if (arr[i] == target) {
            return Some(i);
        }
    }
    return None;
}

match find([1,2,3], 2) {
    Some(index) => print("found at " + str(index)),
    None => print("not found")
}
```

**Nested patterns:**
```atlas
match result {
    Ok(Some(value)) => // handle nested success
    Ok(None) => // handle success but no value
    Err(error) => // handle error
}
```
