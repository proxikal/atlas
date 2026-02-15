# DR-002: TypeChecker-Owned Usage Tracking

**Date:** 2024-09-10
**Status:** Accepted
**Component:** Type Checker - Warnings

## Context
Unused variable/code warnings need tracking of symbol declarations and usage. Question: Where does this tracking live?

## Decision
TypeChecker maintains internal `declared_symbols` and `used_symbols` tracking per function.

**No `used` field on Symbol struct** - keeps Symbol focused on type information, TypeChecker focused on analysis.

Symbol table remains immutable during type checking; usage tracking is TypeChecker's responsibility.

Warnings emitted at end of each function scope, not globally.

## Rationale
**Architectural clarity:** Binder creates/destroys scopes during binding phase, before type checking runs. Scopes are gone by the time TypeChecker needs to check for unused symbols.

**Separation of concerns:**
- Symbol = type information (immutable)
- TypeChecker = usage analysis (mutable tracking)

**AI-friendly:** Clear separation, no unused fields to cause confusion.

## Alternatives Considered
- **Add `used: bool` to Symbol:** Rejected - Symbol table is immutable during type checking, would require `RefCell` or similar, breaks clean separation
- **Track at Binder level:** Rejected - Binder destroys scopes before TypeChecker runs, tracking would be lost
- **Global usage tracking:** Rejected - loses function scope context, harder to implement and debug

## Consequences
- ✅ **Benefits:** Clean separation of concerns (Symbol = types, TypeChecker = analysis)
- ✅ **Benefits:** No mutable state in Symbol table
- ✅ **Benefits:** Scoped warnings (per function, accurate)
- ⚠️  **Trade-offs:** TypeChecker slightly more complex (maintains tracking maps)
- ❌ **Costs:** None significant

## Implementation Notes
**TypeChecker structure:**
```rust
struct TypeChecker {
    declared_symbols: HashMap<FunctionId, HashSet<String>>,
    used_symbols: HashMap<FunctionId, HashSet<String>>,
    // ...
}
```

**Workflow:**
1. Enter function scope → initialize tracking sets
2. Visit declarations → add to `declared_symbols`
3. Visit references → add to `used_symbols`
4. Exit function scope → emit warnings for `declared - used`

## References
- Implementation: `crates/atlas-runtime/src/typechecker.rs`
- Related: DR-001 (Type System)
