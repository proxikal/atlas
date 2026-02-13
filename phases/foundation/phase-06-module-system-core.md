# Phase 06: Module System - Core Infrastructure

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** AST and type checker must support namespacing concepts.

**Verification:**
```bash
grep -n "Scope\|Environment" crates/atlas-runtime/src/typechecker/mod.rs
grep -n "pub struct Program" crates/atlas-runtime/src/ast.rs
cargo test typechecker
```

**What's needed:**
- Type checker with scope management from v0.1
- AST structure supports top-level declarations
- Diagnostic system for module errors

**If missing:** Core systems should exist from v0.1 - verify workspace structure

---

## Objective
Implement core module system enabling code organization across files with explicit imports and exports, namespace isolation, and circular dependency detection - establishing foundation for package ecosystem and large-scale Atlas projects.

## Files
**Create:** `crates/atlas-runtime/src/modules/mod.rs` (~800 lines)
**Create:** `crates/atlas-runtime/src/modules/resolver.rs` (~600 lines)
**Create:** `crates/atlas-runtime/src/modules/graph.rs` (~400 lines)
**Create:** `crates/atlas-runtime/src/modules/loader.rs` (~500 lines)
**Update:** `crates/atlas-runtime/src/ast.rs` (~200 lines add module AST nodes)
**Update:** `crates/atlas-runtime/src/typechecker/mod.rs` (~300 lines module checking)
**Update:** `docs/modules.md` (~400 lines update from future to current)
**Tests:** `crates/atlas-runtime/tests/module_system_tests.rs` (~800 lines)
**Tests:** `crates/atlas-runtime/tests/module_circular_tests.rs` (~400 lines)

## Dependencies
- Type checker with scope management
- AST supports declarations
- File system operations for loading
- Graph algorithms for dependency resolution

## Implementation

### Module Declaration Syntax
Define module declaration in AST supporting export and import statements. Export statement marks declarations as public using export keyword before function, variable, or type declarations. Import statement brings symbols from other modules using import syntax with from clause specifying module path. Support named imports with specific symbol lists. Support wildcard imports for convenience. Module paths use dot notation for nested modules and slash notation for file paths. Default to file-based modules where each Atlas file is a module.

### Module Resolution Strategy
Implement module resolver finding modules by path. Resolve relative imports from current file location using dot-slash prefix. Resolve absolute imports from project root. Search for modules in standard locations src directory, lib directory. Support file extensions .atl for Atlas files. Resolve directory modules using index.atl convention. Cache resolved paths for performance. Report clear errors for missing modules. Support module aliases in configuration.

### Dependency Graph Construction
Build dependency graph tracking import relationships between modules. Add nodes for each module in project. Add directed edges for import dependencies. Detect circular dependencies using cycle detection algorithm. Report circular import errors with full cycle path. Support allowed circular dependencies for type-only imports. Topologically sort modules for compilation order. Enable parallel compilation of independent modules.

### Module Loading Pipeline
Create module loader managing file reading and parsing. Load module source from file system. Parse module into AST handling syntax errors. Extract exports and imports for graph construction. Cache parsed modules avoiding duplicate work. Track module modification times for invalidation. Support incremental reloading on file changes. Handle module load errors gracefully.

### Namespace Isolation
Ensure modules have isolated namespaces preventing naming conflicts. Each module has independent global scope. Imported symbols are explicitly declared. No implicit globals across modules. Module-local declarations stay private unless exported. Support re-exporting imported symbols. Validate no name collisions in imports. Clear error messages for ambiguous imports.

### Type Checker Integration
Extend type checker supporting cross-module type checking. Resolve imported symbol types from exporting module. Check imported symbols match export declarations. Validate import types are compatible with usage. Support type-only imports for type annotations. Enable incremental type checking per module. Propagate type information across module boundaries. Report module-aware type errors with file locations.

### Module Metadata
Track module metadata for tooling and diagnostics. Store module name, path, exports, imports. Record compilation timestamp. Track dependencies for invalidation. Include source locations for symbols. Enable IDE features like go-to-definition across files. Support documentation extraction from modules. Provide module introspection API.

## Tests (TDD - Use rstest)

**Module resolution tests:**
1. Resolve relative imports from current directory
2. Resolve absolute imports from project root
3. Module not found error with suggestions
4. Directory module using index.atl
5. File extension handling
6. Module path normalization
7. Circular dependency detection
8. Multiple circular dependencies reported
9. Valid dependency graph construction
10. Topological sort for compilation order

**Import/export tests:**
1. Export function and import in another module
2. Export variable and import
3. Export type and import
4. Named imports with specific symbols
5. Wildcard imports all symbols
6. Import non-existent symbol error
7. Re-export imported symbol
8. Export conflicts detected
9. Import shadowing warnings
10. Type-only imports

**Namespace tests:**
1. Module isolation no implicit globals
2. Private declarations stay private
3. Exported declarations accessible
4. Import does not pollute namespace
5. Name collision in imports error
6. Multiple modules with same local names
7. Nested module namespaces

**Integration tests:**
1. Multi-file project compilation
2. Cross-module function calls
3. Cross-module type usage
4. Incremental compilation after module change
5. Parallel module compilation
6. Module load error handling
7. Circular dependency error message quality

**Minimum test count:** 120 tests (60 unit, 60 integration)

## Integration Points
- Uses: AST from ast.rs
- Uses: Type checker from typechecker/mod.rs
- Uses: File system operations
- Updates: AST with module nodes
- Updates: Type checker with cross-module checking
- Creates: Complete module system
- Creates: Dependency graph infrastructure
- Output: Multi-file Atlas projects

## Acceptance
- Import and export statements work
- Relative and absolute imports resolve correctly
- Circular dependencies detected with clear errors
- Module namespaces isolated properly
- Cross-module function calls work
- Cross-module type usage works
- Type checker validates imports and exports
- 120+ tests pass 60 unit 60 integration
- Compilation order respects dependencies
- Incremental compilation works
- Module not found errors helpful with suggestions
- Documentation updated from future to current
- No clippy warnings
- cargo test passes
