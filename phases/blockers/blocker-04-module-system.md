# BLOCKER 04: Module System

**Category:** Foundation - Language Feature (Multi-file Support)
**Blocks:** Package management, code organization, 15+ phases
**Estimated Effort:** 3-4 weeks
**Complexity:** Very High

---

## 🚨 BLOCKERS - CHECK BEFORE STARTING

**REQUIRED:** Core language features stable, no major changes planned.

**Verification:**
```bash
cargo test --all --no-fail-fast
grep -c "test result: ok" target/test-results.txt
ls docs/specification/*.md
```

**What's needed:**
- v0.1 complete and stable
- Parser, binder, type checker working well
- No pending refactors to core systems
- All existing tests passing

**If missing:** Stabilize core first. Modules touch everything.

---

## Objective

Add module system to Atlas for multi-file programs. Enables imports, exports, namespace management, and proper code organization. Foundation for package management.

**Core capability:** Split code across files, import dependencies.

---

## Background

Atlas currently single-file only. Need modules for:
- Large programs (split across files)
- Code reuse (import shared utilities)
- Package distribution (publish/consume packages)
- Namespace management (avoid global name conflicts)

**Design reference:** Rust modules (explicit, path-based) + TypeScript ES modules (import/export).

---

## Files

### Create
- `crates/atlas-runtime/src/modules/mod.rs` (~800 lines) - Module system core
- `crates/atlas-runtime/src/modules/resolver.rs` (~600 lines) - Module resolution
- `crates/atlas-runtime/src/modules/loader.rs` (~400 lines) - Module loading
- `crates/atlas-runtime/src/ast/modules.rs` (~200 lines) - Module AST nodes

### Modify (~25 files)
- `src/ast.rs` - Add import/export to AST
- `src/parser/*.rs` - Parse import/export syntax
- `src/binder.rs` - Bind across module boundaries
- `src/typechecker/mod.rs` - Type check imports/exports
- All runtime components - Module-aware execution

### Tests
- `tests/modules_basic_tests.rs` (~600 lines)
- `tests/modules_resolution_tests.rs` (~500 lines)
- `tests/vm_modules_tests.rs` (~600 lines)

**Minimum test count:** 100+ tests

---

## Implementation

### Step 1: Syntax Design
Define import/export syntax. Recommendation: ES module style for familiarity.

**Syntax:**
```atlas
// Export
export fn add(a: number, b: number) -> number { return a + b; }
export let PI = 3.14159;

// Import
import { add, PI } from "./math";
import * as math from "./math";

// Re-export
export { add } from "./math";
```

**Module path resolution:**
- Relative: `./file`, `../sibling`, `../../parent/file`
- Absolute: `/project/src/file` (from project root)
- Package: `"package-name"` (from dependencies - later phase)

### Step 2: AST Extension
Add ImportDecl and ExportDecl to Item enum. ImportDecl has source path and imported names. ExportDecl marks items as exported. Update Program to have imports and exports.

```rust
struct ImportDecl {
    source: String,              // "./math"
    imports: ImportKind,         // named or namespace
    span: Span,
}

enum ImportKind {
    Named(Vec<Identifier>),      // { add, PI }
    Namespace(Identifier),       // * as math
}

struct ExportDecl {
    item: Box<Item>,             // exported function/var
    span: Span,
}
```

### Step 3: Parser Implementation
Parse `import` and `export` keywords. Parse import source (string literal path). Parse named imports `{ x, y }` and namespace imports `* as name`. Parse export declarations (export keyword before item). Store in AST.

### Step 4: Module Resolution
Implement path resolution algorithm. Relative paths resolved from current file location. Absolute paths from project root. Circular dependency detection. Module cache to avoid reloading.

**Algorithm:**
1. Parse import statement
2. Resolve path to absolute file path
3. Check cache - if loaded, return cached module
4. If not loaded, recursively load and parse
5. Add to cache
6. Detect cycles via loading stack

### Step 5: Module Loader
Load and parse module files. Build module dependency graph. Topological sort for initialization order. Execute modules in dependency order. Cache module exports for imports.

**Module structure:**
- Path (absolute file path)
- AST (parsed program)
- Exports (names and types)
- Dependencies (imported modules)
- Initialized (execution state)

### Step 6: Binder Integration
Extend binder to track module boundaries. Imports create bindings in current scope pointing to exports from other modules. Exports mark symbols as externally visible. Symbol table per-module with import/export links.

**Cross-module binding:**
- Create symbol for each import
- Link to corresponding export in source module
- Type checker uses linked symbols

### Step 7: Type Checker Integration
Type check imports match exports (names exist, types compatible). Check no duplicate exports. Check circular type dependencies (if type A imports type B which imports A). Ensure imported types resolved correctly.

### Step 8: Interpreter Support
Interpreter module registry maps paths to loaded modules. Imports resolve through registry. Module initialization happens once per module. Exports stored in module's global scope.

### Step 9: VM Support
VM module linking at compile time. Compile each module separately. Link modules via export/import tables. Module initialization bytecode runs once per module. Function calls across modules use module-qualified names.

### Step 10: Comprehensive Testing
Test basic imports/exports. Test namespace imports. Test circular dependencies detected. Test module initialization order. Test cross-module function calls. Test cross-module type references. Test error cases (missing module, missing export). Full parity.

---

## Architecture Notes

**Module resolution order matters:** Ensure deterministic loading. Topological sort handles dependencies.

**Module caching critical:** Don't reload/re-execute modules. Cache by absolute path.

**Circular dependencies:** Detect and error. Don't attempt to resolve - too complex for v0.2.

**Namespace isolation:** Each module has own global scope. Imports explicitly bring names into scope.

**TypeScript vs Rust:** Take best of both - TypeScript's import syntax, Rust's explicitness.

---

## Acceptance Criteria

**Functionality:**
- ✅ Import/export syntax parses
- ✅ Named imports work ({ add, sub })
- ✅ Namespace imports work (* as math)
- ✅ Module resolution finds files
- ✅ Circular dependencies detected
- ✅ Cross-module calls work
- ✅ Module initialization order correct
- ✅ Module caching works

**Quality:**
- ✅ 100+ tests pass
- ✅ 100% interpreter/VM parity
- ✅ Zero clippy warnings
- ✅ All code formatted
- ✅ No module loading bugs
- ✅ Clear error messages

**Documentation:**
- ✅ Update Atlas-SPEC.md with module syntax
- ✅ Module system guide in docs/features/modules.md
- ✅ Resolution algorithm documented
- ✅ Examples for common patterns

---

## Dependencies

**Requires:**
- Stable core language (v0.1 complete)
- No pending major refactors

**Blocks:**
- Foundation Phase 6: Module System Core (this phase)
- Foundation Phase 7: Package Manifest (depends on modules)
- Foundation Phase 8: Package Manager (depends on modules)
- All multi-file programs
- Package distribution

---

## Rollout Plan

1. Design syntax (3 days)
2. AST extension (2 days)
3. Parser implementation (4 days)
4. Module resolution (5 days)
5. Module loader (4 days)
6. Binder integration (5 days)
7. Type checker integration (4 days)
8. Interpreter support (3 days)
9. VM support (5 days)
10. Testing and polish (5 days)

**Total: ~40 days (6 weeks with thorough testing)**

This is a major language feature. Allocate appropriate time.

---

## Known Limitations

**No package imports yet:** Only file paths, not package names. Package resolution comes in Phase 7/8.

**No re-exports yet:** Can't `export { x } from "./other"` (well, syntax allows, but not implemented).

**No dynamic imports:** All imports static, resolved at compile/load time. No `import()` function.

**No module types yet:** Can't export types (only values). Type exports come after user-defined types.

These are acceptable for initial module system. Can extend later.

---

## Examples

**math.atl:**
```atlas
export fn add(a: number, b: number) -> number {
    return a + b;
}

export fn subtract(a: number, b: number) -> number {
    return a - b;
}

export let PI = 3.14159;
```

**main.atl:**
```atlas
import { add, PI } from "./math";

let result = add(2, 3);
print(str(result));
print(str(PI));
```

**With namespace import:**
```atlas
import * as math from "./math";

let result = math.add(2, 3);
print(str(math.PI));
```

---

## Risk Assessment

**High risk:**
- Module loading/caching bugs
- Circular dependency edge cases
- Cross-module type resolution
- VM module linking complexity

**Mitigation:**
- Extensive testing (100+ tests)
- Reference TypeScript/Rust implementations
- Start simple (file imports only)
- Defer packages to later phases
- Thorough code review

**This touches every part of the compiler. Test exhaustively.**
