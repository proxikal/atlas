# Phase 02: LSP Symbols, Folding, Inlay Hints

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** LSP phase-01 must be complete.

**Verification:**
```bash
ls crates/atlas-lsp/src/hover.rs
ls crates/atlas-lsp/src/actions.rs
cargo test lsp_hover_tests
cargo test lsp_actions_tests
```

**What's needed:**
- LSP phase 01 complete with hover actions tokens
- Parser for symbol extraction
- Type checker for inlay type hints
- Configuration from foundation/phase-04 for hint preferences

**If missing:** Complete phase lsp/phase-01 first

---

## Objective
Implement LSP document and workspace symbol providers for navigation, folding range provider for code collapse, and inlay hint provider for type and parameter hints improving code readability and editor navigation.

## Files
**Update:** `crates/atlas-lsp/src/handlers.rs` (~400 lines)
**Create:** `crates/atlas-lsp/src/symbols.rs` (~500 lines)
**Create:** `crates/atlas-lsp/src/folding.rs` (~200 lines)
**Create:** `crates/atlas-lsp/src/inlay_hints.rs` (~300 lines)
**Update:** `crates/atlas-lsp/src/server.rs` (~80 lines register handlers)
**Tests:** `crates/atlas-lsp/tests/lsp_symbols_tests.rs` (~200 lines)
**Tests:** `crates/atlas-lsp/tests/lsp_folding_tests.rs` (~150 lines)
**Tests:** `crates/atlas-lsp/tests/lsp_inlay_tests.rs` (~150 lines)

## Dependencies
- LSP phase 01 complete
- Parser for symbol and structure extraction
- Type checker for type hints
- Configuration system for preferences
- lsp-types crate for protocol types

## Implementation

### Document Symbol Provider
Implement textDocument/documentSymbol handler. Accept document symbol params with document URI. Parse document extracting all symbols. Identify functions classes types variables constants. Build hierarchical symbol tree with nesting. Extract symbol names kinds ranges and selection ranges. Include detail information like function signatures. Support symbol icons for different kinds. Return document symbols with proper hierarchy. Enable outline view and breadcrumb navigation in editors.

### Symbol Extraction
Extract symbols from AST traversal. Visit all declarations in order. Identify function declarations with names parameters and return types. Extract variable declarations with scope information. Identify type definitions and aliases. Extract constants and enums. Determine symbol kinds using LSP symbol kind enum. Calculate accurate ranges for symbol locations. Determine selection ranges for cursor navigation. Build parent-child relationships for nesting.

### Workspace Symbol Provider
Implement workspace/symbol handler. Accept workspace symbol params with query string. Search all project files for symbols. Perform fuzzy matching on symbol names against query. Filter symbols by kind when specified. Rank results by relevance. Return symbol information with locations. Support fast incremental search. Cache symbols for performance. Update cache on file changes.

### Workspace Indexing
Maintain workspace-wide symbol index. Index all files in workspace on initialization. Track file changes updating index incrementally. Store symbols with file locations and metadata. Support efficient query operations. Handle large workspaces with thousands of files. Provide fast fuzzy search across all symbols. Limit results to prevent overwhelming UI.

### Folding Range Provider
Implement textDocument/foldingRange handler. Accept folding range params with document URI. Parse document identifying foldable structures. Find function bodies with braces. Identify block statements if-else loops. Detect comment blocks single-line and multi-line. Find array and object literals. Calculate start and end lines for each range. Specify folding kind region comment imports. Return folding ranges enabling code collapse in editors.

### Folding Structure Detection
Detect all foldable code structures. Identify function declarations with bodies. Find block statements with braces. Detect multi-line comments for folding. Identify large array literals. Find object literals with multiple fields. Detect import blocks. Calculate accurate line ranges. Ensure proper nesting of ranges. Support editors with different folding behaviors.

### Inlay Hint Provider
Implement textDocument/inlayHint handler. Accept inlay hint params with document range. Analyze code in range identifying hint opportunities. Generate type hints for inferred variable types. Generate parameter name hints for function calls. Format hints concisely. Return inlay hints with positions and labels. Support hint configuration from atlas.toml. Respect user preferences for hint display. Update hints incrementally on edits.

### Type Hints
Generate inlay type hints for variables. Identify variable declarations without explicit types. Query type checker for inferred types. Format type concisely for display. Position hint after variable name. Include colon prefix for clarity. Respect configuration for type hint display. Skip hints where type obvious. Show hints for complex inferred types.

### Parameter Name Hints
Generate parameter name hints for function calls. Identify function call expressions. Extract parameter names from function definition. Match arguments to parameters. Display parameter name before each argument. Use subtle formatting. Skip hints for obvious argument purposes. Support configuration to disable parameter hints. Show hints for calls with many parameters.

## Tests (TDD - Use rstest)

**Document symbol tests:**
1. Extract function symbols
2. Extract variable symbols
3. Extract type symbols
4. Symbol hierarchy correct
5. Symbol ranges accurate
6. Symbol kinds correct
7. Nested symbols structured
8. Selection ranges work
9. Detail information included
10. Empty document handling

**Workspace symbol tests:**
1. Search across multiple files
2. Fuzzy matching works
3. Filter by symbol kind
4. Ranking by relevance
5. Large workspace performance
6. Index updates on changes
7. Query performance acceptable
8. Result limit enforced
9. Symbol locations accurate
10. Cache invalidation correct

**Folding range tests:**
1. Function body folding
2. Block statement folding
3. Comment block folding
4. Array literal folding
5. Object literal folding
6. Nested folding ranges
7. Folding kind correct
8. Line range accuracy
9. Empty blocks handled
10. Performance with large files

**Inlay hint tests:**
1. Type hints for variables
2. Parameter name hints
3. Hint positioning correct
4. Configuration respected
5. Obvious types skipped
6. Complex types shown
7. Hint formatting clear
8. Incremental updates
9. Range filtering works
10. Performance acceptable

**Minimum test count:** 70 tests (20 symbols, 20 workspace, 15 folding, 15 inlay)

## Integration Points
- Uses: LSP server from phase 01
- Uses: Parser for symbol extraction
- Uses: Type checker for type hints
- Uses: Configuration for preferences
- Updates: LSP handlers
- Creates: Symbol providers
- Creates: Folding provider
- Creates: Inlay hint provider
- Output: Enhanced editor navigation and readability

## Acceptance
- Document symbols show all declarations
- Symbol hierarchy reflects nesting
- Workspace symbol search works across files
- Fuzzy matching finds relevant symbols
- Folding ranges identify all foldable structures
- Functions blocks comments foldable
- Inlay type hints show for inferred types
- Parameter name hints display for calls
- Configuration controls hint display
- 70+ tests pass 20 symbols 20 workspace 15 folding 15 inlay
- Performance acceptable for large files
- LSP protocol compliance verified
- Symbols work in outline view
- Folding works in all editors
- Hints subtle and helpful
- No clippy warnings
- cargo test passes
