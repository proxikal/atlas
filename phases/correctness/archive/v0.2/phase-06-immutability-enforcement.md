# Phase Correctness-06: Immutability Enforcement (let vs var)

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Correctness-05 complete. All parity tests green.

**Verification:**
```bash
cargo nextest run -p atlas-runtime -E 'test(parity)' 2>&1 | tail -3
grep "mutable.*bool\|pub.*mutable" crates/atlas-runtime/src/compiler/mod.rs
```

---

## Objective

Atlas has `let` (immutable) and `var` (mutable) declarations. The compiler tracks this distinction in `Local.mutable: bool` for every local variable. The interpreter has an equivalent distinction in the AST (`VarDecl.mutable: bool`). Neither engine uses these values to actually enforce anything. A program that reassigns a `let` variable compiles and runs without error, silently violating the language specification.

This is not a deferred feature — the data is already there. The enforcement is missing. A language that ignores its own immutability rules cannot be trusted for any serious use case. This phase activates the enforcement that the compiler's data model already anticipates.

The spec rule is simple: assignment to a `let`-declared variable is a **compile-time error** in the compiler and a **runtime error** in the interpreter (since the interpreter does not have a separate compilation pass). Both engines must enforce this consistently.

---

## Files Changed

- `crates/atlas-runtime/src/compiler/stmt.rs` — check `mutable` field on assignment targets; emit diagnostic on violation
- `crates/atlas-runtime/src/compiler/mod.rs` — remove `#[allow(dead_code)]` from `Local.depth` and `Local.mutable`
- `crates/atlas-runtime/src/interpreter/stmt.rs` — track mutability per scope binding; reject assignment to immutable variables
- `crates/atlas-runtime/src/interpreter/mod.rs` — change scope storage to `HashMap<String, (Value, bool)>` (value + mutable flag) for locals

---

## Dependencies

- Correctness-05 complete (stable codebase before adding semantic enforcement)

---

## Implementation

### Step 1: Enforce immutability in the compiler

In `compiler/stmt.rs`, when compiling an assignment (`Stmt::Assign`), look up the target variable in `self.locals`. The `Local` struct already has `mutable: bool`. If the target is found and `mutable == false`, emit a compile-time diagnostic:

```rust
Diagnostic::error(
    format!("Cannot assign to immutable variable '{}' — declared with 'let'", name),
    assign.span,
)
.with_label(assign.span, "assignment to immutable variable")
.with_note("Use 'var' instead of 'let' to declare a mutable variable")
```

For globals (not found in locals), there is currently no mutability tracking at the global level. Globals declared with `let` at the top level need tracking — add a `HashMap<String, bool>` for global mutability in the compiler.

Remove `#[allow(dead_code)]` from both `Local.depth` and `Local.mutable` — both are now used.

### Step 2: Enforce immutability in the interpreter

The interpreter stores locals as `HashMap<String, Value>`. This must change to carry mutability. Change the local scope type from:
```rust
Vec<HashMap<String, Value>>
```
to:
```rust
Vec<HashMap<String, (Value, bool)>>  // (value, is_mutable)
```

Where `is_mutable = true` means `var`, `is_mutable = false` means `let`.

Update `eval_var_decl` to store `(value, var_decl.mutable)`. Update all variable reads to extract `.0` from the tuple. Update `eval_assign` to check `.1` before updating — if `mutable == false`, return:

```rust
Err(RuntimeError::TypeError {
    msg: format!("Cannot assign to immutable variable '{}'", name),
    span: assign.span,
})
```

The error message must match the compiler's diagnostic message for the same violation (minus the "Use 'var'" note, which is a hint). For parity: both engines produce a clear error for `let` reassignment.

### Step 3: Handle shadowing correctly

`let x = 1; let x = 2;` (shadowing) is legal — the second `let` creates a new binding, not an assignment to the first. Only `x = 2;` (bare assignment without `let`/`var`) to an existing `let` binding is illegal. Ensure the enforcement only triggers on assignment, not on redeclaration.

### Step 4: Global immutability tracking

Top-level `let` and `var` declarations need the same treatment. In the compiler, add a `HashMap<String, bool>` for global mutability (analogous to `self.locals`). When emitting `SetGlobal`, check if the target was declared immutable. In the interpreter, globals live in `self.globals: HashMap<String, Value>` — change to `HashMap<String, (Value, bool)>` for consistency.

### Step 5: Write enforcement tests

Tests covering:
- `var x = 1; x = 2;` — succeeds in both engines
- `let x = 1; x = 2;` — fails with immutability error in both engines
- `let x = 1; let x = 2;` — succeeds (shadowing)
- `let x = [1,2,3]; x[0] = 9;` — index assignment to an array does NOT mutate `x` itself (the binding is immutable; the array contents are always mutable via `Arc<Mutex<>>`)
- Parity: same program produces same error in interpreter and VM

---

## Tests

- `test_let_immutable_local_compiler_error` — `let x = 1; x = 2;` produces compile error
- `test_let_immutable_local_interpreter_error` — same via interpreter
- `test_let_immutable_parity` — same error in both engines via `assert_parity`
- `test_var_mutable_local_allowed` — `var x = 1; x = 2;` succeeds
- `test_let_shadowing_allowed` — `let x = 1; let x = 2;` succeeds
- `test_let_immutable_global` — top-level `let` is also immutable
- `test_var_mutable_global` — top-level `var` is mutable
- All existing tests pass

---

## Integration Points

- `compiler/mod.rs` — `Local.mutable` dead_code annotation removed; global mutability tracking added
- `compiler/stmt.rs` — assignment checking uses `mutable` field
- `interpreter/mod.rs` — scope type updated to carry mutability flag
- `interpreter/stmt.rs` — var decl stores mutability; assignment checks it
- `tests/core.rs` or `tests/interpreter.rs` — enforcement tests added

---

## Acceptance

- `let x = 1; x = 2;` produces a compile error (compiler) and a runtime TypeError (interpreter)
- Error messages are clear and include the variable name
- `var x = 1; x = 2;` succeeds in both engines
- `let x = 1; let x = 2;` (shadowing) succeeds in both engines
- Parity tests pass — same error for same invalid program
- `#[allow(dead_code)]` removed from `Local.mutable` and `Local.depth`
- All existing tests pass
- Zero clippy warnings
- Commit: `feat(compiler): Enforce let immutability in compiler and interpreter`
