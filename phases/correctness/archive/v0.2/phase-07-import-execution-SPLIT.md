# Phase Correctness-07: Import Statement Execution

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Correctness-06 complete. Foundation-21a (cross-module symbol resolution) verified complete.

**Verification:**
```bash
cargo check -p atlas-runtime 2>&1 | grep -c "error"  # must be 0
grep "Item::Import" crates/atlas-runtime/src/interpreter/mod.rs
# Must show the stub: "// For now, just skip"
ls crates/atlas-runtime/src/module_executor.rs
ls crates/atlas-runtime/src/module_loader.rs
```

---

## Objective

The interpreter silently ignores all `import` statements:
```rust
Item::Import(_) => {
    // Import execution handled in BLOCKER 04-D (module loading)
    // For now, just skip - imports are syntactically valid but not yet functional
}
```

Foundation-21a implemented cross-module symbol resolution. The `module_executor.rs` and `module_loader.rs` files exist. The infrastructure is built. The connection between them and the interpreter's `eval()` is missing — the import statement is parsed, typechecked, and then silently dropped.

This phase wires the interpreter's import handling to the existing module infrastructure. It also verifies that the VM path (via the compiler) handles imports correctly — or adds the same wiring if it doesn't. A language with a module system that doesn't execute imports is broken at a fundamental level.

---

## Files Changed

- `crates/atlas-runtime/src/interpreter/mod.rs` — replace `Item::Import` stub with real execution via `ModuleLoader` / `ModuleExecutor`
- `crates/atlas-runtime/src/interpreter/expr.rs` — ensure imported names are available in the interpreter's globals after import
- `crates/atlas-runtime/src/compiler/mod.rs` or `compiler/stmt.rs` — verify (or implement) that the compiler handles `Item::Import` correctly for the VM path
- `crates/atlas-runtime/src/api/runtime.rs` — ensure `Runtime` provides a `ModuleLoader` to the interpreter

---

## Dependencies

- Correctness-06 complete
- Foundation-21a complete (ModuleLoader, ModuleExecutor, ModuleRegistry exist)
- `module_executor.rs` and `module_loader.rs` must exist and compile

---

## Implementation

### Step 1: Audit the existing module infrastructure

Before writing any code, read and understand:
- `module_loader.rs` — how modules are located and loaded
- `module_executor.rs` — how a loaded module is evaluated and its exports extracted
- `resolver/mod.rs` — how symbol resolution works across modules

Document what the `ModuleExecutor::execute()` function returns. Understand the data contract: what does the caller receive that can be used to populate the interpreter's globals?

### Step 2: Add ModuleLoader to Interpreter

The interpreter currently has no reference to a `ModuleLoader`. Add a field:
```rust
pub(super) module_loader: crate::module_loader::ModuleLoader,
```
Initialize it in `Interpreter::new()` with a default loader (current directory resolution). The `Runtime` struct should pass its configured loader to the interpreter.

### Step 3: Implement import execution in eval()

Replace the stub:
```rust
Item::Import(import_decl) => {
    // execute the import
    let module = self.module_loader.load(&import_decl.source, &current_file_path)
        .map_err(|e| RuntimeError::IoError { message: e.to_string(), span: import_decl.span })?;

    let exports = self.execute_module(module)?;

    // Bind imported names into current scope
    for specifier in &import_decl.specifiers {
        match specifier {
            ImportSpecifier::Named { name, .. } => {
                let value = exports.get(&name.name)
                    .ok_or_else(|| RuntimeError::UndefinedVariable {
                        name: name.name.clone(),
                        span: name.span,
                    })?
                    .clone();
                self.globals.insert(name.name.clone(), (value, false));
            }
            ImportSpecifier::Namespace { alias, .. } => {
                // Bind all exports as a module object under the alias
                let module_obj = self.exports_to_value(exports, import_decl.span);
                self.globals.insert(alias.name.clone(), (module_obj, false));
            }
        }
    }
    Ok(Value::Null)
}
```

The `execute_module` helper evaluates the imported module's AST in a fresh interpreter scope and returns its exported bindings as a `HashMap<String, Value>`.

### Step 4: Fix compiler import handling

The compiler also stubs imports (confirmed: `compiler/mod.rs:129` returns `Ok(())` with comment "Imports don't generate code - they're resolved at compile time"). This is incorrect — imports are NOT resolved at compile time today.

Fix the compiler path: the `Runtime` must run a pre-pass over all `Item::Import` entries before compilation. This pre-pass uses `ModuleLoader` to load the imported module, evaluates it (via interpreter or a dedicated module evaluator), and populates the compiler's globals with the exported symbols. The compiler then sees these globals during compilation and can emit correct `GetGlobal` opcodes.

Implementation: add an `import_prepass()` method to `api/runtime.rs` that processes imports before calling the compiler. The compiler's `Item::Import` arm should then be a no-op (imports already resolved) but must NOT silently skip — add a comment explaining that imports are resolved in the pre-pass.

### Step 5: Circular import detection

Module loading must detect circular imports to avoid infinite loops. Add a `currently_loading: HashSet<PathBuf>` to the module loader (or the interpreter's execution context). If a module is requested while it is already being loaded, return an error:
```
Circular import detected: 'a.atlas' imports 'b.atlas' which imports 'a.atlas'
```

### Step 6: Test import execution end-to-end

Write `.atlas` corpus files that test the import system:
- `tests/corpus/pass/modules/basic_import.atlas` — imports a function from another file, calls it
- `tests/corpus/pass/modules/namespace_import.atlas` — `import * as math from "./math.atlas"`
- `tests/corpus/fail/modules/unknown_export.atlas` — imports a name that isn't exported
- `tests/corpus/fail/modules/circular.atlas` — circular import detected

---

## Tests

- `test_basic_named_import` — `import { add } from "./math"` makes `add` callable
- `test_namespace_import` — `import * as m from "./math"` makes `m.add` callable
- `test_import_unknown_export_error` — importing non-existent name returns clear error
- `test_circular_import_error` — circular import returns error (no infinite loop)
- `test_import_parity` — same import program runs identically on interpreter and VM
- Corpus files for pass/fail modules
- All existing tests pass (imports are not affected if no imports in program)

---

## Integration Points

- `interpreter/mod.rs` — import stub replaced with real execution
- `module_loader.rs` — used by interpreter; may need `Clone` or `Arc` wrapper
- `module_executor.rs` — used to evaluate imported modules
- `api/runtime.rs` — passes module loader configuration to interpreter
- `tests/corpus/pass/modules/` and `tests/corpus/fail/modules/` — new corpus files

---

## Acceptance

- `Item::Import` stub (`// For now, just skip`) is GONE from interpreter/mod.rs
- Named imports bind the imported name into the current scope
- Namespace imports bind all exports under the alias
- Circular imports detected and reported as errors (no stack overflow)
- Unknown export imports produce clear error messages
- Import parity: same program runs identically in interpreter and VM
- Corpus tests for imports pass in the corpus harness
- All existing tests pass
- Zero clippy warnings
- Commit: `feat(interpreter): Wire import statement execution to module loader`
