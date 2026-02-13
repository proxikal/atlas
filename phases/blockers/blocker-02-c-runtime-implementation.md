# BLOCKER 02-C: Generic Runtime Implementation

**Part:** 3 of 4 (Runtime Implementation)
**Category:** Foundation - Type System Extension
**Estimated Effort:** 2 weeks
**Complexity:** Very High

---

## 🚨 BLOCKERS - CHECK BEFORE STARTING

**REQUIRED:** BLOCKER 02-B must be complete.

**Verification:**
```bash
# 02-B complete
cargo test generic_type_checking_tests --no-fail-fast
grep -n "TypeInferer" crates/atlas-runtime/src/typechecker/generics.rs

# All existing tests pass
cargo test --all --no-fail-fast
```

**What's needed:**
- ✅ Type checking validates generics
- ✅ Inference works
- ✅ 60+ type checking tests passing

**If missing:** Complete BLOCKER 02-B first.

---

## Objective

**THIS PHASE:** Implement monomorphization and runtime execution for generics. Enables interpreter and VM to execute generic functions by generating specialized versions for each type instantiation.

**Strategy:** Monomorphization (like Rust) - generate separate code for `identity<number>` and `identity<string>`.

**Success criteria:** Can execute `identity(42)`, `identity("hello")` with correct types in both engines.

---

## Implementation

### Step 1: Monomorphization Infrastructure (Days 1-3)

**File:** `crates/atlas-runtime/src/typechecker/generics.rs`

Create monomorphization engine:
```rust
pub struct Monomorphizer {
    // Cache of monomorphized functions
    instances: HashMap<MonomorphicKey, FunctionDef>,
}

#[derive(Hash, Eq, PartialEq)]
struct MonomorphicKey {
    function_name: String,
    type_args: Vec<Type>,  // e.g., ["number"], ["string"]
}

impl Monomorphizer {
    pub fn monomorphize(
        &mut self,
        func: &FunctionDecl,
        type_args: &[Type],
    ) -> Result<FunctionDef, MonomorphizeError> {
        let key = MonomorphicKey {
            function_name: func.name.clone(),
            type_args: type_args.to_vec(),
        };

        // Check cache
        if let Some(instance) = self.instances.get(&key) {
            return Ok(instance.clone());
        }

        // Generate new instance
        let instance = self.generate_instance(func, type_args)?;
        self.instances.insert(key, instance.clone());
        Ok(instance)
    }

    fn generate_instance(
        &self,
        func: &FunctionDecl,
        type_args: &[Type],
    ) -> Result<FunctionDef, MonomorphizeError> {
        // Build substitution map: T -> number, E -> string, etc.
        let mut subst = HashMap::new();
        for (param_name, arg_type) in func.type_params.iter().zip(type_args) {
            subst.insert(param_name.clone(), arg_type.clone());
        }

        // Substitute types in function signature and body
        let specialized_func = self.substitute_types(func, &subst)?;
        Ok(specialized_func)
    }

    fn substitute_types(
        &self,
        func: &FunctionDecl,
        subst: &HashMap<String, Type>,
    ) -> Result<FunctionDef, MonomorphizeError> {
        // Create new function with substituted types
        // Walk AST, replacing Type::TypeParameter with concrete types
        // ...
    }
}
```

### Step 2: Interpreter Integration (Days 4-7)

**File:** `crates/atlas-runtime/src/interpreter/mod.rs`

Add monomorphizer to interpreter:
```rust
pub struct Interpreter {
    // ... existing fields
    monomorphizer: Monomorphizer,
}
```

**Handle generic calls:**
```rust
fn eval_call(&mut self, call: &CallExpr) -> Result<Value, RuntimeError> {
    let callee_value = self.eval_expr(&call.callee)?;

    match callee_value {
        Value::Function(func_ref) => {
            // Check if function is generic
            if let Some(func_decl) = self.get_generic_function(&func_ref.name) {
                // Infer or get explicit type arguments
                let type_args = self.get_type_arguments(&call)?;

                // Monomorphize
                let specialized = self.monomorphizer.monomorphize(&func_decl, &type_args)?;

                // Call specialized version
                return self.call_specialized_function(specialized, args);
            }

            // Regular function call
            self.call_user_function(&func_ref, args)
        }
        _ => Err(RuntimeError::NotCallable),
    }
}
```

**Optional:** For interpreter, could stay polymorphic (track types at runtime). Monomorphization required for VM.

### Step 3: VM Integration (Days 8-11)

**File:** `crates/atlas-runtime/src/compiler/mod.rs`

**Compile generic functions:**
```rust
fn compile_function_decl(&mut self, func: &FunctionDecl) -> Result<(), CompileError> {
    if func.type_params.is_empty() {
        // Regular function - compile once
        return self.compile_regular_function(func);
    }

    // Generic function - store for later monomorphization
    self.generic_functions.insert(func.name.clone(), func.clone());
    Ok(())
}

fn compile_generic_call(&mut self, call: &CallExpr, type_args: &[Type]) -> Result<(), CompileError> {
    // Get generic function
    let func_decl = self.generic_functions.get(&call.callee_name)?;

    // Monomorphize
    let specialized = self.monomorphizer.monomorphize(func_decl, type_args)?;

    // Generate mangled name: identity$number, identity$string
    let mangled_name = self.mangle_generic_name(&func_decl.name, type_args);

    // Compile specialized version (if not already compiled)
    if !self.compiled_functions.contains(&mangled_name) {
        self.compile_specialized_function(&specialized, &mangled_name)?;
        self.compiled_functions.insert(mangled_name.clone());
    }

    // Emit call to mangled name
    self.emit(Opcode::Call(mangled_name));
    Ok(())
}

fn mangle_generic_name(&self, base: &str, type_args: &[Type]) -> String {
    let args_str = type_args.iter()
        .map(|t| t.display_name())
        .collect::<Vec<_>>()
        .join("$");
    format!("{}${}", base, args_str)
}
```

### Step 4: Parity Testing (Days 12-14)

**File:** `crates/atlas-runtime/tests/vm_generics_runtime_tests.rs`

Identical tests for VM as interpreter.

**Test execution:**
```rust
#[rstest]
fn test_generic_function_execution() {
    let code = r#"
        fn identity<T>(x: T) -> T {
            return x;
        }

        let a = identity(42);
        let b = identity("hello");
        let c = identity(true);
    "#;

    // Interpreter
    let result_interp = run_interpreter(code);
    
    // VM
    let result_vm = run_vm(code);

    // Must match
    assert_eq!(result_interp, result_vm);
}
```

---

## Files

### Create
- `crates/atlas-runtime/tests/generics_runtime_tests.rs` (~400 lines)
- `crates/atlas-runtime/tests/vm_generics_runtime_tests.rs` (~400 lines)

### Modify
- `crates/atlas-runtime/src/typechecker/generics.rs` (~400 lines) - Monomorphization
- `crates/atlas-runtime/src/interpreter/mod.rs` (~100 lines)
- `crates/atlas-runtime/src/compiler/mod.rs` (~200 lines)
- `crates/atlas-runtime/src/vm/mod.rs` (~50 lines)

**Total changes:** ~1550 lines

---

## Acceptance Criteria

**Functionality:**
- ✅ Monomorphization generates specialized functions
- ✅ Interpreter executes generic functions
- ✅ VM executes generic functions
- ✅ Name mangling prevents collisions
- ✅ Shared instances (same types reuse code)
- ✅ 100% parity between engines

**Quality:**
- ✅ 50+ runtime tests pass (both engines)
- ✅ Zero clippy warnings
- ✅ All code formatted
- ✅ No memory leaks

**Next phase:** blocker-02-d-builtin-types.md

---

## Dependencies

**Requires:**
- ✅ BLOCKER 02-B (Type Checker & Inference)

**Blocks:**
- BLOCKER 02-D (Built-in Types)
- BLOCKER 03 (Pattern Matching)

---

## Verification Commands

```bash
# Runtime tests pass
cargo test generics_runtime_tests
cargo test vm_generics_runtime_tests

# Parity verified
cargo test --test integration -- --test-threads=1

# Execute generics
echo 'fn id<T>(x: T) -> T { return x; } id(42);' | cargo run --
```

---

## Notes

**Monomorphization vs Erasure:** We chose monomorphization for performance and type safety.

**Code bloat:** Each instantiation generates code. Monitor binary size.

**Caching critical:** Don't regenerate `identity<number>` every call.

**This is phase 3 of 4. Almost done with generics!**
