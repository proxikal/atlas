# Phase 05: Find References and Call Hierarchy

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** LSP server and module system must exist.

**Verification:**
```bash
ls crates/atlas-lsp/src/server.rs
ls crates/atlas-runtime/src/modules/mod.rs
cargo test lsp
```

**What's needed:**
- LSP server from v0.1
- Module system from foundation/phase-06
- Symbol indexing
- Type checker for resolution

**If missing:** Complete foundation/phase-06 and lsp basics from v0.1

---

## Objective
Implement find-all-references and call hierarchy features enabling navigation of symbol usage across workspace - providing essential code navigation for understanding codebases and impact analysis.

## Files
**Create:** `crates/atlas-lsp/src/references.rs` (~600 lines)
**Create:** `crates/atlas-lsp/src/call_hierarchy.rs` (~500 lines)
**Create:** `crates/atlas-lsp/src/index.rs` (~700 lines)
**Update:** `crates/atlas-lsp/src/handlers.rs` (~300 lines)
**Create:** `docs/lsp-navigation.md` (~500 lines)
**Tests:** `crates/atlas-lsp/tests/references_tests.rs` (~600 lines)

## Dependencies
- LSP server infrastructure
- Module system with exports/imports
- Symbol resolution
- Workspace file management

## Implementation

### Symbol Indexing
Build workspace-wide symbol index. Index all definitions functions, variables, types. Index references to symbols. Track symbol locations. Update index on file changes. Incremental indexing for performance. Persist index to disk. Memory-efficient representation.

### Find All References
Implement textDocument/references handler. Find symbol at cursor position. Query index for all references. Include definition if requested. Filter by scope or file. Return location array. Cross-file reference finding. Performance optimization for large workspaces.

### Call Hierarchy Provider
Implement callHierarchy handlers. Find function at position. Incoming calls who calls this function. Outgoing calls what this function calls. Build call tree. Navigate hierarchy. Transitive call analysis. Recursive call handling.

### Type Hierarchy Provider
Implement typeHierarchy handlers (future). Supertypes and subtypes. Interface implementations. Type relationships. Hierarchy tree building. Navigate up and down.

### Workspace Symbols
Implement workspace/symbol handler. Search symbols by name across workspace. Fuzzy matching for convenience. Filter by symbol kind. Rank results by relevance. Quick open functionality. Support large workspaces efficiently.

### Performance Optimization
Index incrementally on changes. Parallel indexing for speed. Efficient data structures. Cache query results. Lazy loading for large workspaces. Background indexing. Memory bounds.

## Tests (TDD - Use rstest)
1. Find references local variable
2. Find references across files
3. Find references with definition
4. Call hierarchy incoming calls
5. Call hierarchy outgoing calls
6. Workspace symbol search
7. Index update on file change
8. Performance with large workspace
9. Recursive call handling
10. Memory efficiency

**Minimum test count:** 50 tests

## Acceptance
- Find references works cross-file
- Call hierarchy shows callers
- Workspace symbol search functional
- Index updates incrementally
- Performance acceptable
- 50+ tests pass
- Documentation complete
- cargo test passes
