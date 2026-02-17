# Phase 21a: Cross-Module Symbol Resolution

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Module system, build system, binder, import/export AST nodes must exist.

**Verification:**
```bash
ls crates/atlas-runtime/src/module_executor.rs
ls crates/atlas-runtime/src/module_loader.rs
ls crates/atlas-build/src/builder.rs
cargo test -p atlas-runtime --lib binder -- --quiet
```

**What's needed:**
- Binder (`binder.rs`) with scope/symbol table support
- Parser producing `Item::Import` / `Item::Export` AST nodes
- Build system with dependency graph and build order

**If missing:** These are all complete from prior phases.

---

## Objective
Enable cross-module symbol resolution so that imported symbols are visible during binding and type checking. Currently each module compiles in isolation — the binder sees `import { add } from "math"` but cannot resolve `add` because it has no access to `math`'s exports. This phase wires module exports into dependent modules' symbol tables.

## Problem Statement

The build system (`builder.rs`) compiles modules via `compile_single_module()` / `compile_module()` which each create a fresh `Binder::new()`. When module A imports from module B:
1. B's exports are never registered in A's symbol table
2. The binder reports "Unknown symbol" for imported names
3. The type checker has no type information for imported values

**3 builder tests are `#[ignore]`** waiting on this:
- `test_build_multi_file_project_with_imports`
- `test_multiple_targets_library_and_binary`
- `test_build_order_respects_dependencies`

## Files
**Create:** `crates/atlas-build/src/module_resolver.rs` (~300 lines)
**Update:** `crates/atlas-build/src/builder.rs` (~150 lines)
**Update:** `crates/atlas-runtime/src/binder.rs` (~100 lines — add ability to pre-populate symbols)
**Tests:** Update `crates/atlas-build/tests/builder_tests.rs` — un-ignore 3 tests + add 15+ new tests

## Dependencies
- Build order (topological sort) — COMPLETE
- Import/Export AST nodes — COMPLETE
- Binder symbol table — COMPLETE

## Implementation

### 1. Export Collection
After compiling a module, collect its exported symbols (name, type, kind). Store in a `ModuleExports` map keyed by module name.

### 2. Symbol Pre-Population
Before binding a dependent module, scan its imports and inject the corresponding symbols from the `ModuleExports` of each dependency. The binder needs a method like `binder.register_external_symbol(name, type_info)`.

### 3. Build Pipeline Integration
Modify `compile_modules()` in `builder.rs` to:
1. Compile modules in topological order (already done)
2. After each module compiles, extract its exports
3. Before compiling a dependent, inject its dependencies' exports

### 4. Module Resolver
Create `module_resolver.rs`:
- `ModuleExports` struct: name → Vec<ExportedSymbol>
- `ExportedSymbol`: name, type, span
- `resolve_imports()`: given a module's import list and the exports map, produce the symbols to inject
- Handle: missing modules, missing symbols, name conflicts, re-exports

### 5. Type Information Propagation
Export type annotations so the type checker in dependent modules can validate usage of imported values.

## Tests (TDD)
1. Single import resolves symbol
2. Multiple imports from one module
3. Import from missing module → error
4. Import missing symbol from valid module → error
5. Diamond dependency (A→B, A→C, B→C) all resolve
6. Chain dependency (A→B→C) resolves transitively
7. Type mismatch on imported value caught
8. Re-export works
9. Circular import detected
10. Export function, variable, type alias
11. Wildcard import (`import * from "mod"`)
12. Selective import (`import { a, b } from "mod"`)
13. Import with alias (`import { a as b } from "mod"`)
14. Un-ignore existing 3 builder tests — they must pass
15. Incremental build with imports works

**Minimum test count:** 20 tests (15 new + 3 un-ignored + 2 existing)

## Acceptance
- All 3 previously-ignored builder tests pass
- Cross-module imports resolve correctly during binding
- Type information propagates across module boundaries
- Build order (topological sort) used for compilation
- Error messages for missing modules/symbols are clear
- 20+ tests pass
- cargo clippy -p atlas-build -- -D warnings clean
- cargo test -p atlas-build all pass, 0 ignored for this feature
