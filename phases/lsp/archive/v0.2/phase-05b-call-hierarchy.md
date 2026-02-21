# Phase 05B: Call Hierarchy

## Dependencies

**Required:** Symbol index and find references from Phase 05A

**Verification:**
```bash
# Check required files exist
ls crates/atlas-lsp/src/index.rs
ls crates/atlas-lsp/src/references.rs

# Verify functionality
cargo nextest run -p atlas-lsp --test references_tests

# Clean build check
cargo clean && cargo check -p atlas-lsp
```

**If missing:** Complete Phase 05A first

---

## Objective

Implement call hierarchy navigation enabling developers to understand function call relationships - who calls this function (incoming) and what does this function call (outgoing).

---

## Files

**Create:** `crates/atlas-lsp/src/call_hierarchy.rs` (~500 lines)
**Update:** `crates/atlas-lsp/src/handlers.rs` (~100 lines - add call hierarchy handlers)
**Update:** `crates/atlas-lsp/src/lib.rs` (~5 lines - module declaration)
**Tests:** `crates/atlas-lsp/tests/call_hierarchy_tests.rs` (~400 lines)

**Total new code:** ~605 lines
**Total tests:** ~400 lines (15+ test cases)

---

## Dependencies (Components)

- Symbol index (from Phase 05A)
- References provider (from Phase 05A)
- AST for function call analysis (existing)
- Type checker for resolution (existing)

---

## Implementation Notes

**Key patterns to analyze:**
- Examine references implementation from Phase 05A
- Follow LSP testing patterns from auto-memory `testing-patterns.md`
- Reference symbol index structure

**Critical requirements:**
- Prepare call hierarchy from function position
- Find incoming calls (who calls this function)
- Find outgoing calls (what this function calls)
- Build navigable call tree
- Handle recursive calls correctly
- Handle method calls and closures

**Error handling:**
- Handle non-function symbols gracefully
- Handle invalid positions
- Handle circular call chains

**Integration points:**
- Uses: Symbol index, references provider, AST walker
- Creates: Call hierarchy provider
- Updates: Handler registry for callHierarchy/* requests

---

## Tests (TDD Approach)

### Test Categories

**Incoming Calls:** (6 tests)
1. Find direct callers of function
2. Find callers across multiple files
3. Find callers of exported function
4. Find multiple callers
5. No incoming calls for unused function
6. Incoming calls for method

**Outgoing Calls:** (5 tests)
1. Find direct callees from function
2. Find callees across modules
3. Find multiple callees
4. No outgoing calls for leaf function
5. Outgoing calls including stdlib

**Recursive Calls:** (4 tests)
1. Handle direct recursion
2. Handle mutual recursion
3. Handle recursive call tree navigation
4. Limit recursion depth in results

**Minimum test count:** 15 tests

**Test approach:**
- Use rstest for parameterized tests
- Create helper for call hierarchy workspace
- Test both incoming and outgoing directions
- Verify recursive call handling

---

## Acceptance Criteria

- ✅ callHierarchy/prepare handler implemented
- ✅ callHierarchy/incomingCalls handler implemented
- ✅ callHierarchy/outgoingCalls handler implemented
- ✅ Incoming calls finds all callers
- ✅ Outgoing calls finds all callees
- ✅ Works cross-file
- ✅ Handles recursive calls correctly
- ✅ Handles method calls
- ✅ 15+ tests pass (specific count)
- ✅ No clippy warnings
- ✅ cargo nextest run -p atlas-lsp passes
- ✅ Integration tests verify call tree navigation

---

## References

**Specifications:** LSP callHierarchy/* specification
**Related phases:** Phase 05A (Symbol Indexing), Phase 05C (Workspace Symbols)
