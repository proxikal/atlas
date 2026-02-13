# Phase 01: LSP Hover, Code Actions, Semantic Tokens

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** LSP server infrastructure must exist from v0.1.

**Verification:**
```bash
ls crates/atlas-lsp/src/server.rs
ls crates/atlas-lsp/src/handlers.rs
cargo test lsp
grep -n "textDocument" crates/atlas-lsp/src/server.rs
```

**What's needed:**
- LSP server from v0.1 with basic protocol handling
- Type checker for hover type information
- Enhanced diagnostics for code actions
- Parser for semantic token extraction

**If missing:** LSP server should exist from v0.1 - verify lsp crate

---

## Objective
Implement LSP hover information showing types and documentation, code actions providing quick fixes and refactorings, and semantic token provider for enhanced syntax highlighting enabling rich editor integration.

## Files
**Update:** `crates/atlas-lsp/src/handlers.rs` (~600 lines)
**Create:** `crates/atlas-lsp/src/hover.rs` (~300 lines)
**Create:** `crates/atlas-lsp/src/actions.rs` (~400 lines)
**Create:** `crates/atlas-lsp/src/semantic_tokens.rs` (~400 lines)
**Update:** `crates/atlas-lsp/src/server.rs` (~100 lines register handlers)
**Tests:** `crates/atlas-lsp/tests/lsp_hover_tests.rs` (~200 lines)
**Tests:** `crates/atlas-lsp/tests/lsp_actions_tests.rs` (~200 lines)
**Tests:** `crates/atlas-lsp/tests/lsp_tokens_tests.rs` (~200 lines)

## Dependencies
- LSP server from v0.1
- Type checker for type information
- Enhanced diagnostics from frontend/phase-01
- Parser for token extraction
- lsp-types crate for protocol types

## Implementation

### Hover Provider
Implement textDocument/hover handler. Accept hover params with document URI and position. Parse document getting AST. Find symbol at cursor position. Query type checker for symbol type. Extract documentation comments for symbol. Format hover content with type signature and documentation. Return hover response with markdown content. Handle variables showing inferred types. Handle functions showing signatures with parameter types and return type. Handle invalid positions gracefully.

### Type Information Display
Format type information for hover display. Show variable types with inferred or annotated types. Display function signatures with parameter names and types. Format complex types readably arrays objects unions. Include type aliases and definitions. Show generic type parameters. Format with markdown for rich display. Keep display concise but informative.

### Documentation Extraction
Extract and format documentation for hover. Find doc comments preceding symbol declaration. Parse markdown in doc comments. Include code examples from documentation. Format with proper markdown structure. Show function parameter documentation. Display return value documentation. Include usage examples when available.

### Code Action Provider
Implement textDocument/codeAction handler. Accept code action params with document range and context. Analyze diagnostics in range identifying fixable issues. Generate quick fix actions for each diagnostic. Implement refactoring actions for selected code. Return list of code actions with titles and edits. Support workspace edits for multi-file changes. Prioritize actions by relevance.

### Quick Fix Actions
Implement quick fixes for common errors. Add type annotation when type inference fails. Add missing import statements. Fix undefined variable by suggesting declaration. Convert between types with appropriate functions. Fix arity mismatches by adjusting call arguments. Implement diagnostic-specific fixes using error codes from frontend/phase-01. Generate appropriate text edits for each fix.

### Refactoring Actions
Implement code refactorings. Extract variable from selected expression. Extract function from selected statements. Rename symbol with workspace-wide updates. Inline variable replacing usages. Inline function expanding calls. Generate edits preserving code formatting. Validate refactoring safety before applying.

### Semantic Token Provider
Implement textDocument/semanticTokens/full handler. Accept semantic tokens params with document URI. Parse document extracting all tokens. Classify each token by type variable function parameter type keyword operator. Add token modifiers readonly mutable declaration definition. Encode tokens in LSP semantic tokens format with delta encoding. Return semantic tokens response. Support incremental updates for performance.

### Token Classification
Classify tokens for semantic highlighting. Identify variable references and declarations. Classify function names and calls. Identify type names and type parameters. Mark keywords with appropriate category. Classify operators and punctuation. Handle string literals and numeric literals. Identify comments for styling. Distinguish between declarations and references.

### Token Modifiers
Apply modifiers to semantic tokens. Mark readonly variables and constants. Mark mutable variables. Mark deprecated symbols. Mark static functions and variables. Mark async functions. Mark type definitions. Combine modifiers as bitmask per LSP spec.

## Tests (TDD - Use rstest)

**Hover tests:**
1. Hover on variable shows type
2. Hover on function shows signature
3. Hover includes documentation
4. Hover on invalid position returns none
5. Hover on complex types formatted well
6. Hover on generic functions
7. Hover performance acceptable
8. Markdown formatting correct

**Code action tests:**
1. Quick fix for type error
2. Quick fix for undefined variable
3. Quick fix for arity mismatch
4. Extract variable refactoring
5. Extract function refactoring
6. Rename symbol action
7. Inline variable action
8. Multiple actions for one diagnostic
9. Action edit generation
10. Workspace edits for imports

**Semantic token tests:**
1. Variable tokens classified
2. Function tokens classified
3. Type tokens classified
4. Keyword tokens classified
5. Modifiers applied correctly
6. Token encoding delta correct
7. Full document tokenization
8. Performance with large files
9. Incremental updates
10. Token position accuracy

**Minimum test count:** 80 tests (25 hover, 30 actions, 25 tokens)

## Integration Points
- Uses: LSP server from v0.1
- Uses: Type checker for hover information
- Uses: Diagnostics for code actions
- Uses: Parser for semantic tokens
- Updates: LSP handlers
- Creates: Hover provider
- Creates: Code action provider
- Creates: Semantic token provider
- Output: Rich editor integration features

## Acceptance
- Hover shows types accurately
- Hover includes documentation from doc comments
- Function signatures formatted clearly
- 5+ quick fix actions implemented
- 3+ refactoring actions implemented
- Code actions generate valid edits
- Semantic tokens classify all token types
- Token modifiers applied appropriately
- Semantic highlighting works in editors
- 80+ tests pass 25 hover 30 actions 25 tokens
- Performance acceptable for large files
- LSP protocol compliance verified
- No clippy warnings
- cargo test passes
