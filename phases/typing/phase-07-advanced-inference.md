# Phase 07: Advanced Type Inference

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Type inference basics must exist with improvements from phase-01.

**Verification:**
```bash
ls crates/atlas-runtime/src/typechecker/inference.rs
cargo test type_improvements
cargo test type_inference
```

**What's needed:**
- Type inference from typing/phase-01
- Generic constraints from typing/phase-05
- Union types from typing/phase-04
- Module system from foundation/phase-06

**If missing:** Complete typing phases 01-05 first

---

## Objective
Implement advanced type inference including workspace-wide inference, higher-rank polymorphism, and flow-sensitive typing - enabling powerful type inference rivaling TypeScript and Rust reducing annotation burden while maintaining type safety.

## Files
**Update:** `crates/atlas-runtime/src/typechecker/inference.rs` (~600 lines)
**Create:** `crates/atlas-runtime/src/typechecker/unification.rs` (~500 lines)
**Create:** `crates/atlas-runtime/src/typechecker/flow_sensitive.rs` (~400 lines)
**Update:** `crates/atlas-runtime/src/typechecker/mod.rs` (~300 lines integration)
**Create:** `docs/type-inference.md` (~800 lines)
**Tests:** `crates/atlas-runtime/tests/advanced_inference_tests.rs` (~800 lines)

## Dependencies
- Type inference from phase-01
- Generic constraints from phase-05
- Union types from phase-04
- Module system for cross-module inference
- Control flow analysis

## Implementation

### Bidirectional Type Checking
Enhance bidirectional type checking combining synthesis and checking modes. Synthesis mode infers type from expression bottom-up. Checking mode validates expression against expected type top-down. Expected type guides inference reducing ambiguity. Propagate expected types through expressions. Synthesize types when no expectation. Switch modes at boundaries. Reduce need for explicit annotations.

### Higher-Rank Polymorphism
Support higher-rank types and inference. Rank-2 polymorphism for functions taking generic functions. Predicative vs impredicative instantiation. Infer higher-rank types where possible. Require annotations for rank-2 or higher. Unify types respecting rank restrictions. Enable powerful abstractions with callbacks. Desugar to explicit type applications.

### Let-Polymorphism Generalization
Generalize let-bindings with polymorphic types. Infer most general type for bindings. Quantify free type variables. Restrict generalization in mutable contexts. Monomorphism restriction for safety. Value restriction to prevent unsoundness. Generalize closure types carefully. Support recursive let bindings.

### Flow-Sensitive Typing
Track type changes through control flow. Refine types after assignments. Narrow types in branches. Widen types at merge points. Immutable tracking for precise types. Mutable tracking with conservative widening. Loop typing with fixpoint iteration. Detect impossible branches.

### Unification Algorithm
Implement advanced unification for type equations. Unify type variables with types. Occurs check prevents infinite types. Structural unification for compound types. Unify under constraints respecting bounds. Eager vs lazy unification. Unification error messages. Backtracking unification with multiple solutions.

### Constraint-Based Inference
Generate and solve type constraints. Collect constraints from expressions. Constraint variables for unknowns. Constraint solving via unification. Delayed constraint solving for complex cases. Simplify constraints before solving. Report unsolvable constraints clearly. Infer types satisfying all constraints.

### Cross-Module Inference
Infer types across module boundaries. Import type signatures from dependencies. Propagate inferred types to dependents. Incremental inference on module changes. Cache inferred module types. Type inference order from dependency graph. Avoid re-inference of unchanged modules. Export inferred types in module interface.

### Type Inference Heuristics
Apply heuristics for better inference. Prefer simple types over complex. Prefer primitive types when ambiguous. Use literal types when beneficial. Infer union types from branches. Prefer concrete over abstract types. Minimize type variables in solution. Heuristics improve without sacrificing soundness.

## Tests (TDD - Use rstest)

**Bidirectional checking tests:**
1. Synthesis mode infers type
2. Checking mode validates type
3. Expected type guides inference
4. Mode switch at boundaries
5. Reduce annotations with bidirectional
6. Complex expression inference
7. Propagate expected types

**Higher-rank polymorphism tests:**
1. Rank-1 polymorphism inferred
2. Rank-2 requires annotation
3. Function taking generic function
4. Infer with rank restrictions
5. Unify higher-rank types
6. Callback with generic parameter

**Let-polymorphism tests:**
1. Generalize let binding
2. Quantify free type variables
3. Monomorphism restriction applied
4. Value restriction prevents unsoundness
5. Recursive let binding
6. Closure generalization

**Flow-sensitive typing tests:**
1. Refine type after assignment
2. Narrow type in branch
3. Widen at merge point
4. Immutable precise tracking
5. Mutable conservative widening
6. Loop fixpoint iteration
7. Impossible branch detection

**Unification tests:**
1. Unify type variable with type
2. Occurs check prevents infinite
3. Structural unification
4. Unify under constraints
5. Unification error message
6. Backtracking unification

**Constraint solving tests:**
1. Generate constraints from expression
2. Solve constraints via unification
3. Delayed solving for complex
4. Simplify constraints
5. Report unsolvable constraints
6. Infer satisfying all constraints

**Cross-module inference tests:**
1. Import type signatures
2. Propagate inferred types
3. Incremental inference
4. Cache module types
5. Dependency order inference
6. Export inferred types

**Heuristics tests:**
1. Prefer simple types
2. Prefer primitives when ambiguous
3. Infer union from branches
4. Literal type inference
5. Minimize type variables
6. Heuristics improve results

**Integration tests:**
1. Complex program inference
2. Minimal annotations needed
3. Inference across modules
4. Real-world code patterns
5. Performance with large programs

**Minimum test count:** 90 tests

## Integration Points
- Uses: Type inference from phase-01
- Uses: Generic constraints from phase-05
- Uses: Union types from phase-04
- Uses: Module system from foundation/phase-06
- Updates: Type checker with advanced inference
- Creates: Powerful type inference
- Output: Reduced annotation burden

## Acceptance
- Bidirectional type checking works
- Higher-rank polymorphism supported
- Let-polymorphism generalizes bindings
- Flow-sensitive typing tracks changes
- Unification algorithm robust
- Constraint solving functional
- Cross-module inference works
- Inference heuristics improve results
- Annotation burden reduced significantly
- 90+ tests pass
- Documentation comprehensive
- No clippy warnings
- cargo test passes
