# Phase Correctness-07b: Compiler Import Pre-pass + Parity Tests

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Correctness-07a complete. Interpreter imports working.

**Verification:**
```bash
cargo nextest run -p atlas-runtime -E 'test(import)' 2>&1 | tail -5
# Should show interpreter import tests passing
grep "Item::Import" crates/atlas-runtime/src/compiler/mod.rs
# Should show: "Imports don't generate code - they're resolved at compile time"
```

---

## Objective

The compiler stubs `Item::Import` with a misleading comment: "Imports don't generate code - they're resolved at compile time." But imports are NOT resolved at compile time today — the stub just skips them.

This phase adds an **import pre-pass** to `Runtime` that processes imports BEFORE compilation, populating the VM's globals with imported symbols. The compiler then correctly emits `GetGlobal` opcodes for imported names.

---

## Files Changed

- `crates/atlas-runtime/src/api/runtime.rs` — add `import_prepass()` method
- `crates/atlas-runtime/src/compiler/mod.rs` — update stub comment to reference pre-pass
- `crates/atlas-runtime/tests/interpreter.rs` or new `tests/imports.rs` — parity tests
- `tests/corpus/pass/modules/` — corpus test files
- `tests/corpus/fail/modules/` — error case corpus files

---

## Dependencies

- Correctness-07a complete (interpreter imports working)
- `ModuleExecutor` refactored to borrow interpreter

---

## Implementation

### Step 1: Add import pre-pass to Runtime

In `api/runtime.rs`, add a method that processes imports before compilation:
```rust
fn import_prepass(&mut self, ast: &Program, base_path: &Path) -> Result<(), EvalError> {
    for item in &ast.items {
        if let Item::Import(import_decl) = item {
            // Use the interpreter to execute the imported module
            // This populates self.interpreter.globals with exported symbols
            self.process_import(import_decl, base_path)?;
        }
    }
    Ok(())
}
```

### Step 2: Wire pre-pass into VM execution path

In `Runtime::eval_file()` for VM mode:
```rust
ExecutionMode::VM => {
    // Pre-pass: process imports first (populates globals)
    self.import_prepass(&ast, file_path.parent().unwrap())?;

    // Now compile - imported symbols are in globals
    let bytecode = compiler.compile(&ast)?;

    // Execute
    vm.execute(&bytecode)
}
```

### Step 3: Update compiler stub comment

In `compiler/mod.rs`, change:
```rust
Item::Import(_) => {
    // Imports don't generate code - they're resolved at compile time
    Ok(())
}
```
To:
```rust
Item::Import(_) => {
    // Imports are resolved in Runtime::import_prepass() BEFORE compilation.
    // By the time we reach here, imported symbols are already in globals.
    // The compiler emits GetGlobal for imported names — no special handling needed.
    Ok(())
}
```

### Step 4: Write parity tests

Ensure the same import program produces identical results in interpreter and VM:
```rust
#[test]
fn test_import_parity() {
    let code = r#"
        import { add } from "./math.atlas";
        add(1, 2)
    "#;
    assert_parity(code, create_math_module());
}
```

### Step 5: Create corpus test files

**Pass cases** (`tests/corpus/pass/modules/`):
- `basic_import.atlas` — imports a function, calls it
- `multi_import.atlas` — imports multiple names from one module
- `chained_import.atlas` — module A imports B, B imports C

**Fail cases** (`tests/corpus/fail/modules/`):
- `unknown_export.atlas` — imports a name that doesn't exist
- `circular_import.atlas` — A imports B, B imports A

Each corpus file needs a companion module file (e.g., `math.atlas` for `basic_import.atlas`).

---

## Tests

- `test_import_parity` — same program runs identically on interpreter and VM
- `test_vm_import_basic` — VM mode can call imported functions
- `test_vm_import_variable` — VM mode can access imported variables
- Corpus tests pass in corpus harness
- All existing tests pass

---

## Acceptance

- `Runtime` has `import_prepass()` that processes imports before compilation
- Compiler stub comment updated to explain pre-pass
- Import parity: same program runs identically in interpreter and VM
- Corpus tests for pass/fail modules exist and pass
- Namespace imports (`import * as`) return clear "not yet implemented" error
- All existing tests pass
- Zero clippy warnings
- Commit: `feat(compiler): Add import pre-pass for VM execution path`
