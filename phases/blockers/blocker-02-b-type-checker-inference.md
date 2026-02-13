# BLOCKER 02-B: Generic Type Checking & Inference

**Part:** 2 of 4 (Type Checker & Inference)
**Category:** Foundation - Type System Extension
**Estimated Effort:** 2 weeks
**Complexity:** Very High

---

## 🚨 BLOCKERS - CHECK BEFORE STARTING

**REQUIRED:** BLOCKER 02-A must be complete.

**Verification:**
```bash
# 02-A complete
cargo test generic_syntax_tests --no-fail-fast
grep -n "TypeRef::Generic" crates/atlas-runtime/src/ast.rs
grep -n "Type::Generic" crates/atlas-runtime/src/types.rs

# All existing tests still pass
cargo test --all --no-fail-fast
```

**What's needed:**
- ✅ TypeRef::Generic in AST
- ✅ Type::Generic in type system
- ✅ Parser handles generic syntax
- ✅ 40+ syntax tests passing

**If missing:** Complete BLOCKER 02-A first.

---

## Objective

**THIS PHASE:** Implement type checking and inference for generic types. Enables binder to resolve generic types, type checker to validate usage, and inference to deduce type arguments from context.

**NOT in this phase:** Monomorphization, runtime execution, built-in types. Just compile-time type checking.

**Success criteria:** Can type check `identity<number>(42)`, infer `identity(42)` as `T=number`, validate type errors.

---

## Implementation

### Step 1: Binder Integration (Days 1-2)

**File:** `crates/atlas-runtime/src/binder.rs`

Update `resolve_type_ref()` to handle generics:
```rust
fn resolve_type_ref(&mut self, type_ref: &TypeRef) -> Result<Type, BindError> {
    match type_ref {
        // ... existing cases

        TypeRef::Generic { name, type_args, span } => {
            // Resolve each type argument recursively
            let resolved_args: Result<Vec<Type>, _> = type_args
                .iter()
                .map(|arg| self.resolve_type_ref(arg))
                .collect();
            let resolved_args = resolved_args?;

            // Validate generic type exists (for now, just allow)
            // Later: check against registered generic types

            Ok(Type::Generic {
                name: name.clone(),
                type_args: resolved_args,
            })
        }
    }
}
```

**Handle type parameters in function signatures:**
```rust
// When binding function with generic params:
fn bind_function_decl(&mut self, func: &FunctionDecl) -> Result<(), BindError> {
    // Enter new scope for type parameters
    self.enter_scope();

    // Register type parameters (T, E, K, V, etc.)
    for type_param in &func.type_params {
        self.register_type_parameter(type_param)?;
    }

    // Resolve parameter types (may reference type params)
    // ...

    self.exit_scope();
}
```

### Step 2: Type Parameter Scoping (Days 3-4)

**Extend binder to track type parameters:**
```rust
pub struct Binder {
    // ... existing fields
    type_param_scopes: Vec<HashMap<String, TypeParameter>>,
}

struct TypeParameter {
    name: String,
    // Constraints added later
}

impl Binder {
    fn register_type_parameter(&mut self, name: &str) -> Result<(), BindError> {
        // Add to current type param scope
        let scope = self.type_param_scopes.last_mut().unwrap();

        if scope.contains_key(name) {
            return Err(BindError::DuplicateTypeParameter { name: name.to_string() });
        }

        scope.insert(name.to_string(), TypeParameter { name: name.to_string() });
        Ok(())
    }

    fn resolve_type_parameter(&self, name: &str) -> Option<Type> {
        // Look up in type param scopes
        for scope in self.type_param_scopes.iter().rev() {
            if scope.contains_key(name) {
                return Some(Type::TypeParameter { name: name.to_string() });
            }
        }
        None
    }
}
```

### Step 3: Type Checker Validation (Days 5-7)

**File:** `crates/atlas-runtime/src/typechecker/expr.rs`

**Check generic type arity:**
```rust
fn check_generic_application(&mut self, name: &str, type_args: &[Type], span: Span) -> Type {
    // Get expected arity for this generic type
    let expected_arity = match name {
        "Result" => 2,
        "Option" => 1,
        "HashMap" => 2,
        // TODO: Load from registry
        _ => {
            self.diagnostics.push(Diagnostic::error(
                format!("Unknown generic type '{}'", name),
                span,
            ));
            return Type::Unknown;
        }
    };

    // Check arity
    if type_args.len() != expected_arity {
        self.diagnostics.push(Diagnostic::error(
            format!(
                "Generic type '{}' expects {} type argument(s), found {}",
                name, expected_arity, type_args.len()
            ),
            span,
        ));
        return Type::Unknown;
    }

    Type::Generic {
        name: name.to_string(),
        type_args: type_args.to_vec(),
    }
}
```

**Check generic function calls:**
```rust
fn check_generic_call(&mut self, func_type: &Type, args: &[Expr], span: Span) -> Type {
    match func_type {
        Type::Function { params, return_type, type_params } => {
            // If function has type params, try to infer them
            if !type_params.is_empty() {
                return self.check_call_with_inference(params, return_type, type_params, args, span);
            }

            // Regular call
            self.check_call_without_inference(params, return_type, args, span)
        }
        _ => {
            self.diagnostics.push(Diagnostic::error(
                "Cannot call non-function type",
                span,
            ));
            Type::Unknown
        }
    }
}
```

### Step 4: Type Inference (Days 8-11)

**Implement Hindley-Milner style unification:**
```rust
struct TypeInferer {
    substitutions: HashMap<String, Type>,  // T -> number
}

impl TypeInferer {
    // Unify two types, building substitution map
    fn unify(&mut self, expected: &Type, actual: &Type) -> Result<(), TypeError> {
        match (expected, actual) {
            // Type parameter can unify with anything
            (Type::TypeParameter { name }, actual_type) => {
                self.add_substitution(name, actual_type.clone())
            }

            // Concrete types must match
            (Type::Number, Type::Number) => Ok(()),
            (Type::String, Type::String) => Ok(()),
            (Type::Bool, Type::Bool) => Ok(()),

            // Arrays must have compatible element types
            (Type::Array(e1), Type::Array(e2)) => {
                self.unify(e1, e2)
            }

            // Generic types must have same name and compatible args
            (Type::Generic { name: n1, type_args: args1 },
             Type::Generic { name: n2, type_args: args2 }) => {
                if n1 != n2 {
                    return Err(TypeError::TypeMismatch);
                }
                for (a1, a2) in args1.iter().zip(args2.iter()) {
                    self.unify(a1, a2)?;
                }
                Ok(())
            }

            _ => Err(TypeError::TypeMismatch),
        }
    }

    fn add_substitution(&mut self, param: &str, ty: Type) -> Result<(), TypeError> {
        // Check for occurs check (T = Option<T>)
        if self.occurs_in(param, &ty) {
            return Err(TypeError::InfiniteType);
        }

        // Add to substitutions
        self.substitutions.insert(param.to_string(), ty);
        Ok(())
    }

    fn occurs_in(&self, param: &str, ty: &Type) -> bool {
        match ty {
            Type::TypeParameter { name } => name == param,
            Type::Array(elem) => self.occurs_in(param, elem),
            Type::Generic { type_args, .. } => {
                type_args.iter().any(|arg| self.occurs_in(param, arg))
            }
            _ => false,
        }
    }
}
```

**Use inference for generic calls:**
```rust
fn check_call_with_inference(
    &mut self,
    params: &[Type],
    return_type: &Type,
    type_params: &[String],
    args: &[Expr],
    span: Span,
) -> Type {
    let mut inferer = TypeInferer::new();

    // Check each argument against parameter, building substitutions
    for (param_ty, arg) in params.iter().zip(args.iter()) {
        let arg_ty = self.check_expr(arg);
        if let Err(e) = inferer.unify(param_ty, &arg_ty) {
            self.diagnostics.push(Diagnostic::error(
                format!("Type inference failed: {:?}", e),
                span,
            ));
            return Type::Unknown;
        }
    }

    // Apply substitutions to return type
    inferer.apply_substitutions(return_type)
}
```

### Step 5: Testing (Days 12-14)

**File:** `crates/atlas-runtime/tests/generic_type_checking_tests.rs` (~600 lines)

Test categories:
```rust
// Basic type checking
"identity<number>(42)"          // Explicit type arg
"identity(42)"                  // Inferred T=number
"identity<string>(\"hello\")"   // Explicit string

// Arity errors
"Result<number>()"              // Too few args
"Option<T, E>()"                // Too many args

// Type mismatches
"identity<number>(\"string\")"  // Wrong arg type

// Nested generics
"Option<Result<number, string>>"

// Inference with nested
"let x = identity(Some(42))"    // T=Option<number>

// Multiple type params
"fn pair<A, B>(a: A, b: B) -> Result<A, B>"

// Occurs check
"fn bad<T>(x: T) -> Option<T> where T = Option<T>"  // Error
```

---

## Files

### Create
- `crates/atlas-runtime/src/typechecker/generics.rs` (~600 lines) - Inference engine
- `crates/atlas-runtime/tests/generic_type_checking_tests.rs` (~600 lines)

### Modify
- `crates/atlas-runtime/src/binder.rs` (~100 lines) - Type param scoping
- `crates/atlas-runtime/src/typechecker/expr.rs` (~150 lines) - Generic validation

**Total changes:** ~1450 lines

---

## Acceptance Criteria

**Functionality:**
- ✅ Binder resolves generic types
- ✅ Type parameters scoped correctly
- ✅ Type checker validates arity
- ✅ Inference works for simple cases
- ✅ Inference works for nested generics
- ✅ Occurs check prevents infinite types
- ✅ Clear error messages

**Quality:**
- ✅ 60+ type checking tests pass
- ✅ Zero clippy warnings
- ✅ All code formatted
- ✅ No type soundness holes

**Documentation:**
- ✅ Inference algorithm documented
- ✅ Examples in tests

**Next phase:** blocker-02-c-runtime-implementation.md

---

## Dependencies

**Requires:**
- ✅ BLOCKER 02-A (Type System Foundation)

**Blocks:**
- BLOCKER 02-C (Runtime Implementation)
- BLOCKER 02-D (Built-in Types)

**This phase MUST be complete before 02-C.**

---

## Verification Commands

```bash
# Type checking tests pass
cargo test generic_type_checking_tests

# Inference works
echo 'fn identity<T>(x: T) -> T { return x; } identity(42);' | cargo run -- check -

# All tests still pass
cargo test --all --no-fail-fast
```

---

## Notes

**Complex phase:** Type inference is hard. Reference Hindley-Milner algorithm.

**Occurs check critical:** Without it, infinite types like `T = Option<T>` crash the compiler.

**Test thoroughly:** Type soundness bugs are catastrophic.

**This is phase 2 of 4. No runtime yet - just compile-time checking.**
