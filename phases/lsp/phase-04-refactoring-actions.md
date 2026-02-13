# Phase 04: Refactoring Actions

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** LSP code actions must exist, AST manipulation support needed.

**Verification:**
```bash
ls crates/atlas-lsp/src/actions.rs
cargo test lsp_actions
grep -n "WorkspaceEdit" crates/atlas-lsp/src/
```

**What's needed:**
- LSP code actions from lsp/phase-01
- AST with transformation support
- Module system for find-all-references
- Formatter for code generation

**If missing:** Complete lsp/phase-01 and frontend/phase-02 first

---

## Objective
Implement comprehensive refactoring actions including extract variable, extract function, inline, rename with cross-file support - providing VS Code quality refactorings enabling safe large-scale code transformations.

## Files
**Create:** `crates/atlas-lsp/src/refactor/mod.rs` (~600 lines)
**Create:** `crates/atlas-lsp/src/refactor/extract.rs` (~500 lines)
**Create:** `crates/atlas-lsp/src/refactor/inline.rs` (~400 lines)
**Create:** `crates/atlas-lsp/src/refactor/rename.rs` (~500 lines)
**Update:** `crates/atlas-lsp/src/actions.rs` (~200 lines)
**Create:** `docs/lsp-refactoring.md` (~600 lines)
**Tests:** `crates/atlas-lsp/tests/refactor_tests.rs` (~700 lines)

## Dependencies
- LSP server infrastructure
- AST manipulation
- Module system for cross-file
- Formatter for code generation
- Find references capability

## Implementation

### Extract Variable Refactoring
Extract selected expression to variable. Generate unique variable name. Insert let binding before usage. Replace expression with variable reference. Prompt for variable name. Scope analysis for placement. Type inference for binding. Multiple occurrence replacement. Undo support.

### Extract Function Refactoring
Extract statements to new function. Analyze captured variables become parameters. Infer return type from extracted code. Generate function signature. Insert function definition. Replace with function call. Name generation and prompting. Scope and module considerations.

### Inline Variable Refactoring
Replace variable references with value. Find all variable usages. Substitute value at each usage. Remove variable declaration. Validate no side effects. Handle complex expressions carefully. Parenthesization for precedence.

### Inline Function Refactoring
Expand function calls inline. Find all call sites. Substitute function body. Map arguments to parameters. Handle return values. Remove function if unused. Multi-occurrence inlining. Preserve semantics exactly.

### Rename Symbol Refactoring
Rename variable, function, type across files. Find all references workspace-wide. Generate workspace edit. Update all occurrences. Preserve imports and exports. Handle shadowing correctly. Validate new name available. Preview changes before applying.

### Workspace Edits
Generate multi-file edits. TextEdit for each occurrence. Group by file in WorkspaceEdit. Apply all edits atomically. Rollback on failure. Preserve formatting where possible. Integration with formatter.

### Refactoring Safety
Validate refactorings preserve semantics. Check no name conflicts. Verify type safety maintained. Detect potential issues. Warn about edge cases. Preview before apply. Atomic application.

## Tests (TDD - Use rstest)
1. Extract variable simple expression
2. Extract variable multiple occurrences
3. Extract function statements
4. Inline variable single usage
5. Inline function call
6. Rename local variable
7. Rename across files
8. Workspace edit generation
9. Name conflict detection
10. Type safety preservation

**Minimum test count:** 60 tests

## Acceptance
- Extract variable works
- Extract function works
- Inline variable works
- Inline function works
- Rename cross-file works
- Workspace edits atomic
- Safety checks prevent errors
- 60+ tests pass
- Documentation complete
- cargo test passes
