# Phase 12 — Compiler: Static Trait Method Dispatch

**Block:** 3 (Trait System)
**Depends on:** Phase 11 complete
**Estimated tests added:** 12–16

---

## Objective

Compile trait method calls to direct function calls using static dispatch. When the
typechecker annotates a call site as "trait method dispatch," the compiler resolves the
impl method and emits a `Call` opcode pointing directly at the impl's function.

**No new opcodes.** Static dispatch = call a known function. The `Call` opcode is reused.

---

## Current State (verified after Phase 11)

`crates/atlas-runtime/src/compiler/`:
- `mod.rs` — main compilation entry point, processes `Item`s
- `stmt.rs` — compiles statements
- `expr.rs` — compiles expressions including function calls
- Function calls currently compile to: push args, push function value, `Call` opcode
- `Item::Trait` and `Item::Impl` not handled yet — will panic or be ignored

---

## Design: Static Dispatch Compilation

For trait method calls, the compiler needs to:
1. Know it's a trait method call (from Phase 08's `TraitDispatchInfo`)
2. Know the impl method's function name/location
3. Emit a direct `Call` to that function

### Impl method naming convention

Each impl method is compiled as a top-level function with a mangled name:
```
__impl__{TypeName}__{TraitName}__{MethodName}
```
Example: `impl Display for number` → `fn display(self: number) -> string` compiles to
function named `__impl__number__Display__display`.

This name is used as the function identifier in the function registry, allowing the
`Call` opcode to dispatch directly.

---

## Changes

### `crates/atlas-runtime/src/compiler/mod.rs`

**1. Handle `Item::Trait` and `Item::Impl` in the item compiler:**

```rust
match item {
    Item::Function(func) => self.compile_function(func)?,
    Item::Statement(stmt) => self.compile_statement(stmt)?,
    Item::Trait(_) => {
        // Trait declarations are compile-time only (no bytecode emitted)
        // The TraitRegistry in the typechecker holds all needed info
    }
    Item::Impl(impl_block) => self.compile_impl_block(impl_block)?,
    // ... existing variants
}
```

**2. Add `compile_impl_block()`:**

```rust
fn compile_impl_block(&mut self, impl_block: &ImplBlock) -> Result<(), CompileError> {
    let type_name = &impl_block.type_name.name;
    let trait_name = &impl_block.trait_name.name;

    for method in &impl_block.methods {
        let mangled_name = format!(
            "__impl__{}__{}__{}", type_name, trait_name, method.name.name
        );
        self.compile_impl_method(method, &mangled_name)?;
    }
    Ok(())
}

fn compile_impl_method(&mut self, method: &ImplMethod, mangled_name: &str) -> Result<(), CompileError> {
    // Begin function compilation with mangled name
    let func_idx = self.begin_function(mangled_name, method.params.len());

    // Compile params (same as regular function)
    for param in &method.params {
        self.declare_local(&param.name.name);
    }

    // Compile body
    for stmt in &method.body.statements {
        self.compile_statement(stmt)?;
    }

    // Implicit void return if no explicit return
    self.emit(Opcode::Null);
    self.emit(Opcode::Return);

    self.end_function(func_idx);
    Ok(())
}
```

**3. Compile trait method call expressions:**

In `compiler/expr.rs`, when compiling a `MemberExpr` call that has been annotated as
a trait dispatch (via `TraitDispatchInfo` from Phase 08):

```rust
// When compiling `receiver.method(args)` and it's a trait dispatch:
fn compile_trait_method_call(
    &mut self,
    receiver: &Expr,
    method_name: &str,
    args: &[Expr],
    dispatch_info: &TraitDispatchInfo,
) -> Result<(), CompileError> {
    // Push receiver as first argument (the `self` parameter)
    self.compile_expr(receiver)?;

    // Push remaining args
    for arg in args {
        self.compile_expr(arg)?;
    }

    // Look up the mangled function name
    let mangled = format!(
        "__impl__{}__{}__{}",
        dispatch_info.type_name, dispatch_info.trait_name, method_name
    );

    // Push the function reference and call it
    self.emit_get_global(&mangled);  // or equivalent for function lookup
    self.emit(Opcode::Call);
    self.emit_byte((args.len() + 1) as u8);  // +1 for self

    Ok(())
}
```

**How to pass dispatch info to the compiler:** Follow the same pattern as Block 2 for
ownership info. The typechecker produces the `fn_ownership_registry` which the compiler
reads. Add `pub trait_dispatch_annotations: HashMap<Span, TraitDispatchInfo>` to the
typechecker and read it in the compiler.

---

## Impl Method Pre-Pass

Trait declarations and impl blocks need to be compiled BEFORE the functions that call them.
Add an ordering pre-pass: sort `Item`s so `Trait` and `Impl` items are compiled first,
then regular functions and statements.

Alternatively: two-pass compilation where pass 1 registers all function names (including
mangled impl names) and pass 2 compiles bodies. Check if Atlas already has a two-pass
or forward-reference mechanism — if so, use it. If not, the pre-pass sort is simpler.

---

## Tests

Add to `crates/atlas-runtime/tests/vm.rs`:

```rust
#[test]
fn test_vm_trait_method_call_via_static_dispatch() {
    let result = common::run_program("
        trait Display { fn display(self: Display) -> string; }
        impl Display for number {
            fn display(self: number) -> string { return str(self); }
        }
        let x: number = 42;
        let s: string = x.display();
        print(s);
    ");
    assert_eq!(result.output.trim(), "42");
}

#[test]
fn test_vm_impl_method_compiles_to_callable_function() {
    // Verify the mangled function is in the bytecode function table
    let bytecode = common::compile_source("
        trait Greet { fn greet(self: Greet) -> string; }
        impl Greet for string {
            fn greet(self: string) -> string { return \"Hello, \" + self; }
        }
    ").unwrap();
    // Check that a function with impl-mangled name exists in the bytecode
    let has_impl_fn = bytecode.functions.iter().any(|f| {
        f.name.contains("__impl__") && f.name.contains("Greet") && f.name.contains("greet")
    });
    assert!(has_impl_fn, "Impl method should be compiled to a named function");
}

#[test]
fn test_vm_multiple_impl_methods_all_callable() {
    let result = common::run_program("
        trait Shape {
            fn area(self: Shape) -> number;
            fn perimeter(self: Shape) -> number;
        }
        impl Shape for number {
            fn area(self: number) -> number { return self * self; }
            fn perimeter(self: number) -> number { return self * 4; }
        }
        let side: number = 5;
        let a: number = side.area();
        let p: number = side.perimeter();
        print(str(a) + \" \" + str(p));
    ");
    assert_eq!(result.output.trim(), "25 20");
}

#[test]
fn test_vm_impl_for_different_types() {
    let result = common::run_program("
        trait Describe { fn describe(self: Describe) -> string; }
        impl Describe for number {
            fn describe(self: number) -> string { return \"number: \" + str(self); }
        }
        impl Describe for string {
            fn describe(self: string) -> string { return \"string: \" + self; }
        }
        let n: number = 42;
        let s: string = \"hello\";
        print(n.describe());
        print(s.describe());
    ");
    let lines: Vec<&str> = result.output.trim().split('\n').collect();
    assert_eq!(lines[0], "number: 42");
    assert_eq!(lines[1], "string: hello");
}

#[test]
fn test_vm_trait_impl_ordering_independent() {
    // impl can appear before the trait declaration without error
    // (two-pass or pre-pass compilation handles this)
    let result = common::run_program("
        impl Greeter for number {
            fn greet(self: number) -> string { return \"hi\"; }
        }
        trait Greeter { fn greet(self: Greeter) -> string; }
        let n: number = 1;
        print(n.greet());
    ");
    // Expected: either works (two-pass) or produces a clear error
    // Document which behavior Atlas has
    assert!(result.is_ok() || result.is_err()); // placeholder — document during execution
}
```

---

## Acceptance Criteria

- [ ] `Item::Trait` compiles without error (no bytecode emitted — trait is type-info only)
- [ ] `Item::Impl` compiles each method as a mangled `__impl__Type__Trait__method` function
- [ ] Trait method call site emits `Call` to the mangled function
- [ ] Mangled function receives `self` as first argument
- [ ] Multiple impl blocks for different types compile without collision
- [ ] All compiled impl methods appear in bytecode function table
- [ ] All existing compiler tests pass (no regressions)
- [ ] `cargo clippy -- -D warnings` clean
- [ ] `cargo fmt` clean

---

## Notes

- **Mangling format** `__impl__Type__Trait__Method` is the canonical format for Block 3.
  It must be consistent between the compiler (emit site) and the interpreter (lookup site
  in Phase 13). Document in auto-memory under `decisions/runtime.md` as DR-B03-01.
- **No vtable.** V0.4 adds vtable dispatch for trait objects. Block 3 is 100% static.
- **Impl ordering:** if Atlas already has forward-reference support in the compiler, no
  pre-pass is needed. Verify by checking if `get_global` can reference a function that
  hasn't been compiled yet. If not, implement a simple items-sort pre-pass.
