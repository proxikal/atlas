# Phase Correctness-05: Parity — Method Dispatch Unification

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Correctness-04 complete. All callback parity tests green.

**Verification:**
```bash
cargo nextest run -p atlas-runtime -E 'test(parity)' 2>&1 | tail -5
grep "method_to_function_name\|method_name_to_function" \
    crates/atlas-runtime/src/interpreter/expr.rs \
    crates/atlas-runtime/src/compiler/expr.rs
```

---

## Objective

Method dispatch is implemented differently in the two engines:

- **Interpreter:** `method_to_function_name(target: &Value, method: &str)` — dispatches based on the **runtime type** of the target value. For `JsonValue`, produces `"jsonXxx"`. For all other types, produces `"unknown_method"` (an error path).
- **Compiler:** `method_name_to_function(method: &str)` — ignores the target type entirely and **always** produces `"jsonXxx"`, because the comment says "all currently supported methods are JSON methods."

These two functions will produce different output the moment any type other than `JsonValue` gains method syntax (String, Array, DateTime, HashMap, etc. — all planned in stdlib phases 16+). The interpreter will correctly dispatch to the type-specific function; the compiler will emit the wrong `jsonXxx` name. This is a parity time bomb that gets harder to fix as each stdlib phase adds more methods.

This phase unifies method dispatch on a single, type-aware strategy that works correctly in both engines today and is extensible for all future types. The strategy: a shared **method dispatch table** in a new `method_dispatch.rs` module that maps `(TypeTag, method_name) → stdlib_function_name`. Both the interpreter and compiler consult this same table. The interpreter provides the runtime type tag; the compiler provides the compile-time type (from the symbol table annotation).

---

## Files Changed

- `crates/atlas-runtime/src/method_dispatch.rs` — new file: shared dispatch table, `TypeTag` enum, `resolve_method()` function
- `crates/atlas-runtime/src/lib.rs` — declare `pub(crate) mod method_dispatch`
- `crates/atlas-runtime/src/interpreter/expr.rs` — replace `method_to_function_name` with `method_dispatch::resolve_method(tag, name)`
- `crates/atlas-runtime/src/compiler/expr.rs` — replace `method_name_to_function` with type-aware lookup using `method_dispatch::resolve_method(tag, name)`
- `crates/atlas-runtime/src/ast.rs` — add `TypeTag` annotation to `MemberExpr` (populated by typechecker)
- `crates/atlas-runtime/src/typechecker/expr.rs` — annotate `MemberExpr` with the target's resolved type tag during type checking

---

## Dependencies

- Correctness-04 complete (parity fixes stable)
- Typechecker must have resolved the target's type at method call sites (already the case — typechecker validates method calls)

---

## Implementation

### Step 1: Define TypeTag and the dispatch table

Create `src/method_dispatch.rs`:

```rust
/// Runtime-stable type tag for method dispatch.
/// Mirrors the types that support method call syntax.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TypeTag {
    JsonValue,
    // Future types added here as stdlib phases add method support:
    // String, Array, HashMap, HashSet, DateTime, Regex, ...
}

/// Resolve a method call to its stdlib function name.
/// Returns None if the type/method combination is not registered.
pub fn resolve_method(type_tag: TypeTag, method_name: &str) -> Option<String> {
    match (type_tag, method_name) {
        (TypeTag::JsonValue, m) => Some(format!("json{}", capitalize_first(m))),
        // Future: add more arms as new types gain methods
    }
}
```

This is the single source of truth for method dispatch. Both engines call `resolve_method`. If a method is unknown, `None` is returned and the caller produces a `TypeError`.

### Step 2: Annotate MemberExpr with TypeTag

In `ast.rs`, add a `type_tag: Option<TypeTag>` field to `MemberExpr`. It is `None` after parsing (typechecker hasn't run yet) and `Some(tag)` after typechecking. This annotation carries the type information from the frontend to the compiler without requiring the compiler to have a symbol table.

### Step 3: Populate TypeTag in the typechecker

In `typechecker/expr.rs`, when validating a method call, the typechecker already knows the target type. After validation passes, write the `TypeTag` into `member_expr.type_tag`. For `JsonValue` targets, set `TypeTag::JsonValue`. This is a small change — one assignment in the existing method validation code.

### Step 4: Update interpreter to use resolve_method

In `interpreter/expr.rs`, replace `method_to_function_name(&target_value, &member.member.name)` with:

```rust
let type_tag = member.type_tag.expect("TypeTag not set — typechecker must run before eval");
let func_name = crate::method_dispatch::resolve_method(type_tag, &member.member.name)
    .ok_or_else(|| RuntimeError::TypeError {
        msg: format!("No method '{}' on type {:?}", member.member.name, type_tag),
        span: member.span,
    })?;
```

The `method_to_function_name` function is deleted.

### Step 5: Update compiler to use resolve_method

In `compiler/expr.rs`, `compile_member` uses `member.type_tag` (set by typechecker):

```rust
let type_tag = member.type_tag.expect("TypeTag not set — typechecker must run before compile");
let func_name = crate::method_dispatch::resolve_method(type_tag, &member.member.name)
    .ok_or_else(|| vec![Diagnostic::error(
        format!("No method '{}' on type {:?}", member.member.name, type_tag),
        member.span,
    )])?;
```

The `method_name_to_function` function is deleted.

### Step 6: Delete the old dispatch functions

Remove `method_to_function_name` from `interpreter/expr.rs` and `method_name_to_function` from `compiler/expr.rs`. Both `capitalize_first` helper functions can be moved into `method_dispatch.rs` and called from there.

### Step 7: Parity test for method calls

Write parity tests that call methods on every type with method support:

```rust
#[test]
fn test_json_method_call_parity() {
    assert_parity(r#"
        let j = parseJSON("{\"x\": 42}");
        print(j.as_number("x"));
    "#);
}
```

Both engines must produce identical output.

---

## Tests

- All existing method call tests pass (JsonValue methods work identically)
- Parity tests for all supported method calls pass
- Attempting a method call on an unsupported type produces the same error in both engines
- Typechecker-unannotated `MemberExpr` panics with clear message (not silent wrong behavior)
- `cargo nextest run -p atlas-runtime` green

---

## Integration Points

- `method_dispatch.rs` — new shared module
- `ast.rs` — `MemberExpr.type_tag` field added
- `typechecker/expr.rs` — populates TypeTag during validation
- `interpreter/expr.rs` — uses shared dispatch
- `compiler/expr.rs` — uses shared dispatch
- Future stdlib phases: add new `TypeTag` variants and new match arms in `resolve_method`

---

## Acceptance

- `method_dispatch.rs` exists with `TypeTag` enum and `resolve_method` function
- `MemberExpr` carries a `type_tag: Option<TypeTag>` field (set by typechecker)
- `method_to_function_name` in interpreter deleted
- `method_name_to_function` in compiler deleted
- Both engines call `method_dispatch::resolve_method` for method call dispatch
- Parity tests for all method calls pass
- All existing tests pass
- Zero clippy warnings
- Commit: `feat(compiler): Unified method dispatch via shared TypeTag table — eliminates parity divergence`
