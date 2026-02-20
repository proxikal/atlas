# Phase 05C: Workspace Symbols & Performance Polish

## Dependencies

**Required:** Symbol index from Phase 05A and call hierarchy from Phase 05B

**Verification:**
```bash
# Check required files exist
ls crates/atlas-lsp/src/index.rs
ls crates/atlas-lsp/src/call_hierarchy.rs

# Verify functionality
cargo nextest run -p atlas-lsp --test references_tests
cargo nextest run -p atlas-lsp --test call_hierarchy_tests

# Clean build check
cargo clean && cargo check -p atlas-lsp
```

**If missing:** Complete Phase 05A and 05B first

---

## Objective

Implement workspace-wide symbol search with fuzzy matching, optimize indexing performance for large workspaces, and complete comprehensive documentation for all navigation features.

---

## Files

**Update:** `crates/atlas-lsp/src/index.rs` (~200 lines - add search and optimization)
**Update:** `crates/atlas-lsp/src/handlers.rs` (~100 lines - add workspace/symbol handler)
**Create:** `docs/lsp-navigation.md` (~500 lines)
**Tests:** `crates/atlas-lsp/tests/workspace_symbol_tests.rs` (~300 lines)
**Tests:** `crates/atlas-lsp/tests/performance_tests.rs` (~200 lines)

**Total new code:** ~800 lines (including docs)
**Total tests:** ~500 lines (15+ test cases)

---

## Dependencies (Components)

- Symbol index (from Phase 05A)
- References provider (from Phase 05A)
- Call hierarchy (from Phase 05B)
- Fuzzy matching library (add to dependencies if needed)

---

## Implementation Notes

**Key patterns to analyze:**
- Examine index implementation from Phase 05A
- Follow LSP testing patterns from auto-memory `testing-patterns.md`
- Reference performance optimization patterns

**Critical requirements:**
- Workspace-wide symbol search by name
- Fuzzy matching for convenience
- Filter results by symbol kind
- Rank results by relevance
- Parallel indexing for large workspaces
- Efficient data structures (hash maps, bloom filters)
- Cache query results
- Memory bounds enforcement

**Error handling:**
- Handle invalid search queries
- Handle workspace too large scenarios
- Handle out-of-memory gracefully

**Integration points:**
- Uses: Symbol index, fuzzy matcher
- Updates: Index with optimization
- Creates: Workspace symbol search

---

## Tests (TDD Approach)

### Test Categories

**Workspace Symbol Search:** (8 tests)
1. Search symbols by exact name
2. Search with fuzzy matching
3. Filter by symbol kind (function, variable, type)
4. Search across multiple files
5. Rank results by relevance
6. Search with partial query
7. No results for nonexistent symbol
8. Search in large workspace

**Performance Optimization:** (4 tests)
1. Parallel indexing speedup
2. Incremental update performance
3. Query cache effectiveness
4. Memory usage stays bounded

**Integration:** (3 tests)
1. Combined references + call hierarchy + search
2. Index consistency across operations
3. Full navigation workflow

**Minimum test count:** 15 tests

**Test approach:**
- Use rstest for parameterized tests
- Create large workspace for performance tests
- Benchmark indexing and search speed
- Verify memory bounds

---

## Acceptance Criteria

- ✅ workspace/symbol handler implemented
- ✅ Symbol search works with exact match
- ✅ Symbol search works with fuzzy match
- ✅ Filter by symbol kind works
- ✅ Results ranked by relevance
- ✅ Parallel indexing implemented
- ✅ Query caching implemented
- ✅ Memory usage bounded for large workspaces
- ✅ 15+ tests pass (specific count)
- ✅ Performance benchmarks show improvement
- ✅ Documentation complete in docs/lsp-navigation.md
- ✅ No clippy warnings
- ✅ cargo nextest run -p atlas-lsp passes
- ✅ All Phase 05 acceptance criteria met

---

## References

**Specifications:** LSP workspace/symbol specification
**Related phases:** Phase 05A (Symbol Indexing), Phase 05B (Call Hierarchy)
**Documentation:** docs/lsp-navigation.md (created in this phase)
