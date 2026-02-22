# Phase 13 — Interpreter: Trait Method Dispatch

**Block:** 3 (Trait System)
**Depends on:** Phase 12 complete
**Estimated tests added:** 12–16

---

## Objective

Implement trait method dispatch in the tree-walking interpreter. When evaluating
`receiver.method(args)`, check the `impl_registry` to find the implementing method
and execute its body directly.

The interpreter does NOT use mangled function names — it looks up the `ImplMethod`
from the `impl_registry` and evaluates it inline. This is the "tree-walk" approach,
contrasting with the compiler's "mangled name" approach in Phase 12.

**Parity requirement:** Interpreter and VM must produce identical output for all
trait method calls.

---

## Current State (verified after Phase 12)

`crates/atlas-runtime/src/interpreter/`:
- `mod.rs` — main interpreter, handles Items and Stmts
- `expr.rs` — expression evaluation including member/method calls
- Method call evaluation: `eval_call_expr()` or similar — checks stdlib methods
- `Item::Trait` and `Item::Impl` not yet handled in interpreter

---

## Investigation Required (do this first)

```bash
grep -n "member\|method\|MemberExpr\|method_dispatch" \
  crates/atlas-runtime/src/interpreter/expr.rs | head -30
```

```bash
grep -n "eval_item\|Item::" crates/atlas-runtime/src/interpreter/mod.rs | head -20
```

Understand how `arr.push(x)` is evaluated in the interpreter before adding trait dispatch.
The new dispatch must slot into the same call chain.

---

## Changes

### `crates/atlas-runtime/src/interpreter/mod.rs`

**Handle `Item::Trait` and `Item::Impl` in `eval_item()`:**

```rust
match item {
    Item::Function(func) => {
        // existing: register function in scope
        self.register_function(func);
    }
    Item::Trait(_) => {
        // Trait declarations: no runtime action needed
        // Trait info lives in the typechecker's TraitRegistry
    }
    Item::Impl(impl_block) => {
        // Register impl methods for runtime dispatch
        self.register_impl(impl_block);
    }
    // ... existing variants
}
```

**Add `register_impl()`:**

```rust
fn register_impl(&mut self, impl_block: &ImplBlock) {
    let type_name = impl_block.type_name.name.clone();
    let trait_name = impl_block.trait_name.name.clone();

    for method in &impl_block.methods {
        let method_name = method.name.name.clone();
        // Store for runtime lookup using the same key format as the TypeChecker's ImplRegistry
        self.impl_registry.register(
            &type_name,
            &trait_name,
            // Convert ImplMethod to a stored representation
            // ...
        );
    }
}
```

**Note:** The interpreter needs access to `impl_registry` at runtime. Options:
A. The interpreter owns its own `ImplRegistry` (populated during item processing)
B. The interpreter receives the typechecker's `ImplRegistry` as input

**Recommendation:** Option A. The interpreter registers impls as it encounters them
during item evaluation (same as how it registers functions in the scope). Add an
`impl_methods: HashMap<(String, String, String), ImplMethod>` map to the interpreter
struct, keyed by `(type_name, trait_name, method_name)`.

### `crates/atlas-runtime/src/interpreter/expr.rs`

**Extend method call evaluation to try trait dispatch:**

```rust
// In eval_member_call() or equivalent:
fn eval_trait_method_call(
    &mut self,
    receiver: Value,
    receiver_type_name: &str,
    method_name: &str,
    args: Vec<Value>,
) -> Result<Value, RuntimeError> {
    // Search impl_methods for matching (type, *, method)
    // We don't know the trait_name at runtime — search by type+method
    for ((type_name, trait_name, meth_name), impl_method) in &self.impl_methods {
        if type_name == receiver_type_name && meth_name == method_name {
            // Found the impl method — evaluate its body
            return self.eval_impl_method_body(impl_method, receiver, args);
        }
    }
    Err(RuntimeError::MethodNotFound {
        type_name: receiver_type_name.to_string(),
        method: method_name.to_string(),
        span: /* call span */ Span::zero(),
    })
}

fn eval_impl_method_body(
    &mut self,
    method: &ImplMethod,
    self_value: Value,
    args: Vec<Value>,
) -> Result<Value, RuntimeError> {
    // Push new scope
    self.push_scope();

    // Bind self as first param
    if let Some(first_param) = method.params.first() {
        self.define_variable(&first_param.name.name, self_value);
    }

    // Bind remaining params
    for (param, arg) in method.params.iter().skip(1).zip(args.iter()) {
        self.define_variable(&param.name.name, arg.clone());
    }

    // Evaluate body
    let result = self.eval_block(&method.body);

    // Pop scope
    self.pop_scope();

    match result {
        Ok(val) => Ok(val),
        Err(RuntimeError::Return { value, .. }) => Ok(value),
        Err(e) => Err(e),
    }
}
```

**Receiver type name extraction:**

```rust
fn value_type_name(value: &Value) -> &str {
    match value {
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Bool(_) => "bool",
        Value::Null => "null",
        Value::Array(_) => "array",
        Value::HashMap(_) => "map",
        // etc.
        _ => "unknown",
    }
}
```

---

## Dispatch Priority in Interpreter

Method call resolution order:
1. Built-in stdlib methods (existing `method_dispatch.rs`)
2. Trait impl methods (new — this phase)
3. `MethodNotFound` error

This matches the typechecker's priority (Phase 08) and must match the VM's priority (Phase 14 parity check).

---

## Tests

Add to `crates/atlas-runtime/tests/interpreter.rs`:

```rust
#[test]
fn test_interpreter_trait_method_dispatch_basic() {
    let atlas = Atlas::new();
    let result = atlas.eval("
        trait Display { fn display(self: Display) -> string; }
        impl Display for number {
            fn display(self: number) -> string { return str(self); }
        }
        let x: number = 42;
        let s: string = x.display();
        s;
    ");
    assert_eq!(result.unwrap(), Value::String(Arc::new("42".to_string())));
}

#[test]
fn test_interpreter_multiple_type_impls() {
    let atlas = Atlas::new();
    let result = atlas.eval("
        trait Describe { fn describe(self: Describe) -> string; }
        impl Describe for number {
            fn describe(self: number) -> string { return \"num:\" + str(self); }
        }
        impl Describe for string {
            fn describe(self: string) -> string { return \"str:\" + self; }
        }
        let n: number = 7;
        let s: string = \"hi\";
        n.describe() + \" \" + s.describe();
    ");
    assert_eq!(result.unwrap().to_string(), "num:7 str:hi");
}

#[test]
fn test_interpreter_impl_method_accesses_self() {
    let atlas = Atlas::new();
    let result = atlas.eval("
        trait Doubler { fn double(self: Doubler) -> number; }
        impl Doubler for number {
            fn double(self: number) -> number { return self * 2; }
        }
        let x: number = 21;
        x.double();
    ");
    assert_eq!(result.unwrap().to_string(), "42");
}

#[test]
fn test_interpreter_impl_method_with_extra_args() {
    let atlas = Atlas::new();
    let result = atlas.eval("
        trait Adder { fn add(self: Adder, other: number) -> number; }
        impl Adder for number {
            fn add(self: number, other: number) -> number { return self + other; }
        }
        let x: number = 10;
        x.add(32);
    ");
    assert_eq!(result.unwrap().to_string(), "42");
}

#[test]
fn test_interpreter_impl_ordering_both_before_call() {
    let atlas = Atlas::new();
    let result = atlas.eval("
        trait Marker { fn tag(self: Marker) -> string; }
        impl Marker for bool {
            fn tag(self: bool) -> string { return \"bool\"; }
        }
        let b: bool = true;
        b.tag();
    ");
    assert_eq!(result.unwrap().to_string(), "bool");
}

#[test]
fn test_interpreter_trait_method_returns_correct_type() {
    let atlas = Atlas::new();
    let result = atlas.eval("
        trait HasLen { fn length(self: HasLen) -> number; }
        impl HasLen for string {
            fn length(self: string) -> number { return len(self); }
        }
        let s: string = \"hello\";
        s.length();
    ");
    // Note: string already has len() — this tests that the trait impl also works
    // If stdlib takes priority, this still returns 5 (correct)
    assert_eq!(result.unwrap().to_string(), "5");
}
```

---

## Acceptance Criteria

- [ ] `Item::Trait` is handled (no runtime action, no panic)
- [ ] `Item::Impl` registers impl methods for runtime dispatch
- [ ] `receiver.method(args)` dispatches to impl method body when receiver type matches
- [ ] Impl method `self` parameter is bound to the receiver value
- [ ] Additional parameters are bound correctly
- [ ] Return value from impl method body is the call expression value
- [ ] Multiple impls for different types coexist without collision
- [ ] Dispatch priority: stdlib first, then trait impls
- [ ] All existing interpreter tests pass (no regressions)
- [ ] `cargo clippy -- -D warnings` clean
- [ ] `cargo fmt` clean

---

## Notes

- The interpreter doesn't use mangled names — it stores `ImplMethod` structs directly
  in a HashMap keyed by `(type_name, method_name)`. The trait name is used for ambiguity
  resolution if two traits define a method with the same name for the same type — the
  first registered wins (same as typechecker priority).
- The interpreter's `impl_methods` map is separate from the typechecker's `impl_registry`.
  They serve different purposes: typechecker's is for type analysis, interpreter's is for
  runtime execution.
- Scope management in `eval_impl_method_body()` must mirror how `eval_function_body()`
  works. Copy that pattern exactly to avoid scope leak bugs.
