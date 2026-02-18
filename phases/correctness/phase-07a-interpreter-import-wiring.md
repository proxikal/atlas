# Phase Correctness-07a: Interpreter Import Wiring

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

The interpreter silently ignores all `import` statements. The module infrastructure (`ModuleLoader`, `ModuleExecutor`) exists and is complete. This phase **wires** the interpreter's import handling to that infrastructure.

**Key architectural issue**: `ModuleExecutor` owns its own `Interpreter` instance, but `Runtime` also has its own. Imports processed by `ModuleExecutor` populate ITS interpreter, not `Runtime`'s. This phase must resolve this architecture mismatch.

---

## Files Changed

- `crates/atlas-runtime/src/interpreter/mod.rs` — replace `Item::Import` stub with real execution
- `crates/atlas-runtime/src/module_executor.rs` — refactor to accept `&mut Interpreter` instead of owning one
- `crates/atlas-runtime/src/api/runtime.rs` — integrate `ModuleExecutor` with `Runtime`

---

## Dependencies

- Correctness-06 complete
- `module_executor.rs` and `module_loader.rs` exist and compile

---

## Implementation

### Step 1: Audit existing infrastructure

Before writing code, understand:
- `module_loader.rs` — `ModuleLoader::load_module()` returns `Vec<LoadedModule>` in topological order
- `module_executor.rs` — `ModuleExecutor::execute_module()` processes imports and extracts exports
- Circular detection already exists in `ModuleLoader.loading: HashSet<PathBuf>`

Document findings in `memory/decisions.md` as DR-007.

### Step 2: Resolve architecture — ModuleExecutor refactor

**Current problem** (line ~55 of module_executor.rs):
```rust
pub struct ModuleExecutor {
    interpreter: Interpreter,  // Owns its own interpreter
}
```

**Solution**: Refactor `ModuleExecutor` to accept `&mut Interpreter`:
```rust
pub struct ModuleExecutor<'a> {
    loader: ModuleLoader,
    resolver: ModuleResolver,
    cache: ModuleCache,
    interpreter: &'a mut Interpreter,  // Borrows interpreter
    security: &'a SecurityContext,
}
```

Update all methods to work with the borrowed interpreter. This ensures imports populate the SAME interpreter that `Runtime` uses.

### Step 3: Wire Item::Import in interpreter

Replace the stub in `interpreter/mod.rs`:
```rust
Item::Import(import_decl) => {
    // Imports are handled by ModuleExecutor before eval() is called.
    // If we reach here during standalone interpreter use (no ModuleExecutor),
    // we need to process the import inline.
    self.process_import(import_decl, current_file_path)?;
    Ok(Value::Null)
}
```

Add `process_import()` helper that:
1. Uses `ModuleLoader` to load the module
2. Evaluates the module (recursive call or fresh interpreter)
3. Extracts exports and binds them to `self.globals`

### Step 4: Update Runtime integration

In `api/runtime.rs`, ensure `Runtime::eval()` uses `ModuleExecutor` for file-based execution:
```rust
pub fn eval_file(&mut self, path: &Path) -> Result<Value, EvalError> {
    let mut executor = ModuleExecutor::new(
        &mut self.interpreter.borrow_mut(),
        &self.security,
        project_root,
    );
    executor.execute_module(path).map_err(EvalError::ParseError)
}
```

Keep `eval(&str)` for REPL/string evaluation (no import processing needed for inline code).

### Step 5: Verify circular detection

Circular import detection already exists in `ModuleLoader`. Write a quick test to verify it works:
```rust
#[test]
fn test_circular_import_detected() {
    // Create a.atlas importing b.atlas, b.atlas importing a.atlas
    // Verify error is returned, no stack overflow
}
```

---

## Tests

- `test_basic_named_import_interpreter` — `import { add } from "./math"` makes `add` callable
- `test_circular_import_error` — circular import returns clear error (no infinite loop)
- `test_import_unknown_export_error` — importing non-existent name returns clear error
- All existing tests pass

---

## Acceptance

- `Item::Import` stub (`// For now, just skip`) is GONE from interpreter/mod.rs
- `ModuleExecutor` borrows interpreter instead of owning one
- Named imports bind the imported name into the current scope
- Circular imports detected and reported as errors
- Unknown export imports produce clear error messages
- Architecture decision documented in `memory/decisions.md` (DR-007)
- All existing tests pass
- Zero clippy warnings
- Commit: `feat(interpreter): Wire import execution to module infrastructure`
