# BLOCKER 03-A: Pattern Matching Syntax & Type Checking

**Part:** 1 of 2 (Syntax & Type Checking)
**Category:** Foundation - Language Feature
**Estimated Effort:** 1 week
**Complexity:** High

---

## 🚨 BLOCKERS - CHECK BEFORE STARTING

**REQUIRED:** BLOCKER 02-D complete (need Option<T> and Result<T,E>).

**Verification:**
```bash
# Generics complete
cargo test --all --no-fail-fast | grep -c "test result: ok"
grep -n "Option" crates/atlas-runtime/src/stdlib/types.rs
grep -n "Result" crates/atlas-runtime/src/stdlib/types.rs
```

**What's needed:**
- ✅ Option<T> and Result<T,E> types exist
- ✅ Generics fully working

**If missing:** Complete BLOCKER 02-D first.

---

## Objective

**THIS PHASE:** Add match expression syntax, AST, parser, and type checking with exhaustiveness. Does NOT include runtime execution - just parsing and type checking.

**Success criteria:** Can parse and type check match expressions. Exhaustiveness checking works.

---

## Implementation

### Step 1-2: AST & Parser (Days 1-3)

Add Pattern enum and MatchExpr to AST. Parse match syntax. Handle all pattern types (literal, wildcard, variable, constructor).

### Step 3-4: Type Checking & Exhaustiveness (Days 4-7)

Type check patterns against scrutinee. Implement exhaustiveness algorithm. Ensure all arms return compatible types.

---

## Acceptance Criteria

- ✅ Parse match expressions
- ✅ All pattern types supported
- ✅ Type checking validates patterns
- ✅ Exhaustiveness checking works
- ✅ 60+ tests pass
- ✅ Clear error messages

**Next:** blocker-03-b-runtime-execution.md

---

**Pattern matching phase 1 of 2. Type checking only, no execution yet.**
