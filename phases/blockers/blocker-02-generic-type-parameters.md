# BLOCKER 02: Generic Type Parameters

**Category:** Foundation - Type System Extension
**Blocks:** Result<T,E>, HashMap<K,V>, Option<T>, and 15+ phases
**Estimated Effort:** 4-6 weeks
**Complexity:** Very High

---

## 🚨 BLOCKERS - CHECK BEFORE STARTING

**REQUIRED:** Type system and type checker must be stable and well-tested.

**Verification:**
```bash
grep -n "enum Type" crates/atlas-runtime/src/types.rs
cargo test typechecker --no-fail-fast
cargo test binder --no-fail-fast
grep -c "test.*type" crates/atlas-runtime/tests/*.rs
```

**What's needed:**
- Stable Type enum with basic types
- Type checker with inference
- Binder with type resolution
- 200+ type system tests passing
- No existing type system bugs

**If missing:** Fix type system issues first. This builds on solid foundation.

---

## Objective

Extend Atlas type system to support generic type parameters, enabling parameterized types like `Result<T, E>`, `Option<T>`, `HashMap<K, V>`. This is a **fundamental type system change** affecting parser, binder, type checker, and runtime.

**NOT in scope:** Higher-kinded types, trait bounds, or variance. Just basic parameterized types.

---

## Background

Atlas currently supports:
- Primitives: number, string, bool, null, void
- Arrays: `number[]`, `string[]` (single type parameter, hard-coded)
- Functions: `(T1, T2) -> T3`

Need to generalize arrays into `Array<T>` and add other generic types.

**Design decision:** Monomorphization (like Rust) vs type erasure (like Java/TypeScript).
**Recommendation:** Monomorphization for performance, compile-time type safety.

---

## Files

### Create
- `crates/atlas-runtime/src/typechecker/generics.rs` (~600 lines)
  - Generic instantiation logic
  - Type parameter substitution
  - Constraint checking (basic - no trait bounds yet)
  - Monomorphization support

### Modify (~30 files)
- `src/types.rs` - Add Generic and TypeParameter variants
- `src/ast.rs` - Add generic syntax to AST
- `src/parser/types.rs` - Parse generic syntax
- `src/binder.rs` - Bind generic declarations
- `src/typechecker/*.rs` - Type check generics
- `src/interpreter/*.rs` - Runtime generic handling
- `src/compiler/*.rs` - Compile generics
- `src/vm/*.rs` - Execute generic code

### Tests
- `tests/generics_basic_tests.rs` (~800 lines)
- `tests/generics_inference_tests.rs` (~600 lines)
- `tests/vm_generics_tests.rs` (~800 lines)

**Minimum test count:** 150+ tests

---

## Implementation

### Step 1: Syntax Design
Define generic syntax for Atlas. Recommendation: `Type<T>` for type application, `fn name<T>(x: T) -> T` for generic functions.

Example syntax:
```atlas
// Generic type declaration (built-in for now, user-defined later)
// Result<T, E> - success T or error E
// Option<T> - value T or null

// Generic function
fn identity<T>(x: T) -> T {
    return x;
}

// Usage with explicit type
let x: number = identity<number>(42);

// Usage with inference
let y = identity(42);  // T inferred as number
```

### Step 2: AST Extension
Add TypeRef::Generic for generic type applications: `Result<number, string>`. Add generic type parameters to function declarations. Update Program/Item/FunctionDecl to support generic parameters.

**Example AST:**
```rust
enum TypeRef {
    Named(String, Span),
    Array(Box<TypeRef>, Span),
    Function { params, return_type, span },
    Generic {
        name: String,
        type_args: Vec<TypeRef>,
        span: Span,
    },
}
```

### Step 3: Type Representation
Add Type::Generic for instantiated generics and Type::TypeParameter for unresolved type variables.

```rust
enum Type {
    // existing variants...
    Generic {
        name: String,              // Result, Option, HashMap
        type_args: Vec<Type>,      // [Number, String]
    },
    TypeParameter {
        name: String,              // T, E, K, V
        constraints: Vec<Type>,    // empty for now
    },
}
```

### Step 4: Parser Implementation
Parse generic syntax: `Type<T1, T2>`. Handle nested generics: `Option<Result<T, E>>`. Parse function generic parameters: `fn<T, E>`. Error on malformed syntax (missing >, unbalanced brackets).

### Step 5: Binder Integration
Register type parameters in scope when entering generic function. Resolve type parameters to Type::TypeParameter. Resolve generic applications to Type::Generic with substituted arguments.

**Challenge:** Scoping - type parameters are lexically scoped to function/type declaration.

### Step 6: Type Inference
Implement type inference for generic functions. When calling `identity(42)`, infer `T = number` from argument type. Unification algorithm for constraint solving. Handle multiple type parameters with dependencies.

**Algorithm:** Hindley-Milner style unification with occurs check.

### Step 7: Monomorphization
Generate concrete versions of generic functions for each instantiation. `identity<number>` and `identity<string>` become separate function bodies. Happens at compile time (for VM) or lazily (for interpreter).

**Optimization:** Share monomorphized instances across call sites.

### Step 8: Type Checker Integration
Check generic type applications have correct arity. Substitute type parameters in function bodies. Ensure type parameters used consistently. Check that inferred types satisfy constraints (when constraints added later).

### Step 9: Interpreter Support
Interpreter can handle generics via dynamic typing (already does). May need to track type arguments for better error messages. Monomorphization optional for interpreter (can stay polymorphic).

### Step 10: VM Support
VM requires full monomorphization. Each generic instantiation becomes separate bytecode function. Function table needs to handle monomorphized names: `identity$number`, `identity$string`.

### Step 11: Built-in Generic Types
Once infrastructure works, add built-in generics:
- `Option<T>` - value or null wrapper
- `Result<T, E>` - success or error
- `Array<T>` - migrate from hard-coded array syntax

**Note:** Array migration is breaking change - need transition plan.

### Step 12: Comprehensive Testing
Test generic functions with primitives, arrays, functions. Test inference (explicit vs implicit type args). Test nested generics. Test error cases (arity mismatch, type errors). Test monomorphization generates correct code. Full parity between interpreter/VM.

---

## Architecture Notes

**Monomorphization vs Erasure:**
- Monomorphization: Generate concrete code per type (Rust, C++)
- Erasure: Single runtime code, types erased (Java, TypeScript)

**Recommendation:** Monomorphization
- Pros: Better performance, compile-time specialization, no runtime type info needed
- Cons: Code bloat, longer compile times
- For Atlas: Correctness > speed, and AI agents can handle complexity

**Type Parameter Scoping:**
Type parameters lexically scoped to declaration. `T` in one function is independent of `T` in another.

**Inference:**
Start simple - require explicit type args, add inference later if needed. Or implement basic inference for common cases (argument types → type params).

---

## Acceptance Criteria

**Functionality:**
- ✅ Generic type syntax parses: `Type<T1, T2>`
- ✅ Generic function syntax parses: `fn name<T>(x: T) -> T`
- ✅ Type parameters resolve correctly
- ✅ Generic instantiation works
- ✅ Type checking validates generic usage
- ✅ Monomorphization generates correct code
- ✅ Built-in Option<T> and Result<T,E> work

**Quality:**
- ✅ 150+ tests pass
- ✅ 100% interpreter/VM parity
- ✅ Zero clippy warnings
- ✅ All code formatted
- ✅ No type soundness holes
- ✅ Generics compose (nested generics work)

**Documentation:**
- ✅ Update Atlas-SPEC.md with generic syntax
- ✅ Document type system extensions
- ✅ Examples in docs/features/generics.md
- ✅ Decision log entry for monomorphization choice

---

## Dependencies

**Requires:**
- Stable type system from v0.1
- Type inference working
- No existing type checker bugs

**Blocks:**
- Foundation Phase 9: Error Handling (Result<T,E>)
- Stdlib Phase 7: Collections (HashMap<K,V>, HashSet<T>)
- Typing Phase 4: Union Types (may interact)
- Typing Phase 5: Generic Constraints
- All phases needing generic types

---

## Rollout Plan

1. Design syntax (1 day)
2. Extend AST (2 days)
3. Extend Type representation (1 day)
4. Parser implementation (3 days)
5. Binder integration (3 days)
6. Type checker integration (5 days)
7. Inference implementation (4 days)
8. Monomorphization (5 days)
9. Interpreter support (2 days)
10. VM support (4 days)
11. Built-in generics (3 days)
12. Testing and polish (5 days)

**Total: ~40 days (6 weeks)**

This is a major feature. No shortcuts. Get it right.

---

## Known Limitations

**No user-defined generic types yet:** Only built-in generics (Option, Result, Array, HashMap). User-defined generic structs/classes come later.

**No trait bounds:** Can't constrain `T` to types with specific operations. All type parameters unconstrained.

**No variance:** All type parameters invariant. Covariance/contravariance comes later if needed.

**No higher-kinded types:** Can't have `F<T>` where F itself is a type parameter.

These are acceptable for v0.2. Focus on getting basic generics right.

---

## Risk Assessment

**High risk areas:**
1. Type inference - complex algorithm, easy to get wrong
2. Monomorphization - code generation complexity
3. VM integration - function table management
4. Breaking change to array syntax (migration path needed)

**Mitigation:**
- Extensive testing (150+ tests minimum)
- Reference implementations (Rust, TypeScript, Swift)
- Incremental rollout (infrastructure first, then built-ins)
- Array syntax migration can be deferred (keep both syntaxes temporarily)

**This is the biggest foundation change in v0.2. Allocate appropriate time.**
