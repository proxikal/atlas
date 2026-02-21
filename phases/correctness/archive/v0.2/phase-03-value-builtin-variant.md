# Phase Correctness-03: Value::Builtin Variant

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Correctness-02 complete. Builtin registry exists.

**Verification:**
```bash
cargo check -p atlas-runtime 2>&1 | grep -c "error"  # must be 0
grep "OnceLock\|builtin_registry" crates/atlas-runtime/src/stdlib/mod.rs | wc -l  # > 0
```

---

## Objective

Currently, builtin functions like `print`, `len`, and all 150 stdlib functions are stored in the interpreter's globals as `Value::Function(FunctionRef { bytecode_offset: 0, local_count: 0 })`. The fields `bytecode_offset` and `local_count` are meaningless for builtins — they exist only to satisfy the `FunctionRef` struct. The VM similarly creates dummy `FunctionRef` values for builtins at method call sites. There is no way to distinguish a builtin from a user-defined function by inspecting a `Value` alone.

This matters for correctness: the interpreter's call dispatch must check `is_builtin(name)` before looking up `function_bodies`, because a builtin has no body. If that check is reordered or omitted, the runtime fails with `UnknownFunction` instead of executing the builtin. This is a design that is one refactor away from a silent bug.

The correct representation is a dedicated `Value::Builtin(Arc<str>)` variant. A builtin is a distinct kind of callable, not a function with a garbage bytecode offset. After this phase, dispatch is type-driven: `Value::Function(_)` means a user-defined function with real bytecode, `Value::Builtin(_)` means a stdlib function dispatched through the registry.

---

## Files Changed

- `crates/atlas-runtime/src/value.rs` — add `Value::Builtin(Arc<str>)` variant; update `Display`, `Debug`, `PartialEq`, `type_name()`
- `crates/atlas-runtime/src/interpreter/mod.rs` — `register_builtin()` stores `Value::Builtin` instead of `Value::Function`
- `crates/atlas-runtime/src/interpreter/expr.rs` — `eval_call` dispatch updated; `call_value` updated
- `crates/atlas-runtime/src/interpreter/stmt.rs` — any match on `Value::Function` that must not match builtins updated
- `crates/atlas-runtime/src/vm/mod.rs` — `Call` opcode handler updated; `compile_member` emits `Value::Builtin`
- `crates/atlas-runtime/src/compiler/expr.rs` — `compile_member` uses `Value::Builtin` for the function constant
- `crates/atlas-runtime/src/reflect/value_info.rs` — reflect calls updated if they inspect function values
- `crates/atlas-runtime/src/api/conversion.rs` — conversion API updated if it exposes function values

---

## Dependencies

- Correctness-02 complete (registry provides the dispatch backend that `Value::Builtin` calls into)

---

## Implementation

### Step 1: Add the variant to Value

In `value.rs`, add to the `Value` enum:
```rust
/// Builtin stdlib function (dispatched through the registry by name)
Builtin(Arc<str>),
```

Place it immediately after `Function`. Update every exhaustive match on `Value`:
- `type_name()` → returns `"builtin"`
- `Display` → `write!(f, "<builtin {}>", name)`
- `Debug` → `write!(f, "Builtin({:?})", name)`
- `PartialEq` → `(Value::Builtin(a), Value::Builtin(b)) => a == b`

### Step 2: Update register_builtin in interpreter

In `interpreter/mod.rs`, `register_builtin()` currently creates:
```rust
Value::Function(FunctionRef { name: name.to_string(), arity, bytecode_offset: 0, local_count: 0 })
```
Change to:
```rust
Value::Builtin(Arc::from(name))
```
The `arity` field is no longer needed here because the registry dispatch functions validate argument counts themselves.

### Step 3: Update eval_call in interpreter

In `interpreter/expr.rs`, `eval_call` currently matches:
```rust
Value::Function(func_ref) => {
    if crate::stdlib::is_builtin(&func_ref.name) { ... }
    // user function path below
}
```
After this phase, the match gains a new arm:
```rust
Value::Builtin(name) => {
    let security = self.current_security.as_ref().expect("Security context not set");
    crate::stdlib::call_builtin(&name, &args, call.span, security, &self.output_writer)
}
Value::Function(func_ref) => {
    // This arm now exclusively handles user-defined functions
    // is_builtin check removed — builtins no longer arrive here
    ...
}
```

### Step 4: Update call_value in interpreter

Same pattern: `Value::Builtin(name)` arm dispatches through the registry. `Value::Function` arm handles only user-defined functions. The `is_builtin` call inside `call_value` is removed.

### Step 5: Update compiler/expr.rs compile_member

`compile_member` currently creates a `Value::Function(FunctionRef { ... })` constant for the method function. Change to:
```rust
let method_fn = Value::Builtin(Arc::from(func_name.as_str()));
let const_idx = self.bytecode.add_constant(method_fn);
```

### Step 6: Update VM Call opcode handler

In `vm/mod.rs`, the `Call` opcode handler currently checks `is_builtin(&func.name)`. After this phase, the loaded constant may be `Value::Builtin` or `Value::Function`. The match branches on the variant:

```rust
Value::Builtin(ref name) => {
    // dispatch through registry
}
Value::Function(ref func) => {
    // user-defined function: create call frame, jump to bytecode_offset
}
```

The `is_builtin` check inside the VM's Call handler is removed.

### Step 7: Verify is_builtin call sites are gone

```bash
grep -rn "is_builtin(" crates/atlas-runtime/src/interpreter/ crates/atlas-runtime/src/vm/
```
Must return no results. The function `is_builtin` still exists in `stdlib/mod.rs` (for API compatibility) but is no longer called from interpreter or VM dispatch paths.

---

## Tests

- All existing tests pass — behavior is identical; only internal representation changes
- Verify `Value::Builtin("print")` displays as `<builtin print>` (not `<fn print>`)
- Verify `Value::Builtin("unknown")` dispatched through registry returns `UnknownFunction` error
- Verify user-defined functions still represented as `Value::Function` and execute correctly
- Verify `is_builtin` call sites in interpreter/VM removed via grep
- `cargo nextest run -p atlas-runtime` green

---

## Integration Points

- `value.rs` — new variant; all Value match exhaustiveness enforced by compiler
- `interpreter/mod.rs`, `interpreter/expr.rs` — dispatch updated, `is_builtin` calls removed
- `vm/mod.rs` — Call handler updated
- `compiler/expr.rs` — compile_member updated
- `stdlib/mod.rs` — `is_builtin()` still present but no longer called by engines

---

## Acceptance

- `Value::Builtin(Arc<str>)` variant exists in value.rs
- Builtins stored as `Value::Builtin` in interpreter globals (not `Value::Function`)
- `Value::Function` arms in eval_call and VM Call handler contain ZERO `is_builtin` calls
- Compiler emits `Value::Builtin` constants for method calls (not `Value::Function`)
- `type_name()` for builtin returns `"builtin"` (not `"function"`)
- All existing tests pass
- Zero clippy warnings
- Commit: `feat(value): Add Value::Builtin variant — separate builtins from user functions`
