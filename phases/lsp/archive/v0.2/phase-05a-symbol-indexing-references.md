# Phase 05A: Symbol Indexing & Find References

## Dependencies

**Required:** LSP server and module system must exist.

**Verification:**
```bash
# Check required files exist
ls crates/atlas-lsp/src/server.rs
ls crates/atlas-runtime/src/module_loader.rs

# Verify functionality
cargo nextest run -p atlas-lsp

# Clean build check
cargo clean && cargo check -p atlas-lsp
```

**If missing:** Complete foundation/phase-06 and lsp basics from v0.1

---

## Objective

Implement workspace-wide symbol indexing and find-all-references functionality, enabling cross-file navigation of symbol usage and providing the foundation for advanced code navigation features.

---

## Files

**Create:** `crates/atlas-lsp/src/index.rs` (~700 lines)
**Create:** `crates/atlas-lsp/src/references.rs` (~600 lines)
**Update:** `crates/atlas-lsp/src/handlers.rs` (~100 lines - add references handler)
**Update:** `crates/atlas-lsp/src/lib.rs` (~10 lines - module declarations)
**Tests:** `crates/atlas-lsp/tests/references_tests.rs` (~400 lines)

**Total new code:** ~1,310 lines
**Total tests:** ~400 lines (20+ test cases)

---

## Dependencies (Components)

- LSP server infrastructure (existing)
- Module system with exports/imports (existing)
- Symbol resolution (existing)
- Type checker for resolution (existing)

---

## Implementation Notes

**Key patterns to analyze:**
- Examine hover/completion implementation in `crates/atlas-lsp/src/handlers.rs`
- Follow LSP testing patterns from auto-memory `testing-patterns.md`
- Reference existing symbol resolution in runtime

**Critical requirements:**
- Workspace-wide symbol index (all definitions and references)
- Incremental index updates on file changes
- Memory-efficient index representation
- Cross-file reference finding
- Include/exclude definition in results

**Error handling:**
- Handle missing symbols gracefully
- Handle invalid positions
- Handle file not found scenarios

**Integration points:**
- Uses: LSP server state, module loader, type checker
- Creates: Symbol index, references provider
- Updates: Handler registry for textDocument/references

---

## Tests (TDD Approach)

### Test Categories

**Local References:** (8 tests)
1. Find references to local variable in same function
2. Find references to function parameter
3. Find references to local let binding
4. Find references with definition included
5. Find references excluding definition
6. References to shadowed variables
7. No references found for undefined symbol
8. References in nested scopes

**Cross-File References:** (8 tests)
1. Find references across multiple files
2. Find references to imported function
3. Find references to exported variable
4. Find references in module hierarchy
5. References to re-exported symbols
6. References with circular imports
7. References in different modules
8. References to stdlib functions

**Index Management:** (6 tests)
1. Index updates on file change
2. Index updates on file add
3. Index updates on file delete
4. Incremental indexing performance
5. Index persistence and reload
6. Index memory efficiency with large workspace

**Minimum test count:** 22 tests

**Test approach:**
- Use rstest for parameterized tests
- Create helper to build test workspace
- Test both single-file and multi-file scenarios
- Verify index updates correctly

---

## Acceptance Criteria

- ✅ Symbol index tracks all definitions in workspace
- ✅ Symbol index tracks all references in workspace
- ✅ textDocument/references handler implemented
- ✅ Find references works for local variables
- ✅ Find references works cross-file
- ✅ Index updates incrementally on file changes
- ✅ Include/exclude definition option works
- ✅ 22+ tests pass (specific count)
- ✅ No clippy warnings
- ✅ cargo nextest run -p atlas-lsp passes
- ✅ Integration tests verify cross-file behavior

---

## References

**Specifications:** LSP textDocument/references specification
**Related phases:** Phase 05B (Call Hierarchy), Phase 05C (Workspace Symbols)
