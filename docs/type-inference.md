# Atlas Type Inference

This document describes the type inference system in the Atlas compiler.

---

## Overview

Atlas uses a bidirectional type inference algorithm that combines:

- **Synthesis mode** — infer the type of an expression bottom-up from its structure
- **Checking mode** — validate an expression against a known expected type top-down
- **Constraint solving** — accumulate type equations and solve them in batch
- **Flow-sensitive typing** — track how types change through control flow

The goal is to reduce annotation burden while maintaining full type safety.

---

## Bidirectional Type Checking

### Modes

| Mode | Direction | When Used |
|------|-----------|-----------|
| `Synthesis` | Bottom-up | No expected type available |
| `Checking(T)` | Top-down | Expected type known from context |

### How It Works

```
let x: number = 42;
         ^           ← checking mode: validate 42 against number
              ^^     ← synthesis mode: infer 42 → number
```

The type checker switches modes at **boundaries**:
- Variable declarations with annotations → checking mode
- Return statements → checking mode (against declared return type)
- Unannoted variable declarations → synthesis mode
- Call argument positions → synthesis mode (then validate against param type)

### Expected Type Propagation

When a type annotation is present, the expected type is propagated into sub-expressions to guide inference. This reduces the need for redundant annotations:

```atlas
// Without propagation — both sides need annotation
let xs: number[] = [1, 2, 3];   // ok - annotation present

// With propagation — annotation on one side informs the other
fn identity<T>(x: T) -> T { return x; }
let n: number = identity(42);   // T inferred as number from annotation
```

---

## Higher-Rank Polymorphism

Atlas supports rank-1 polymorphism (Hindley-Milner style) with inference, and rank-2 polymorphism when explicit annotations are provided.

### Rank-1 (Inferred)

Type parameters are inferred from call-site arguments:

```atlas
fn identity<T>(x: T) -> T { return x; }

let n = identity(42);      // T = number, inferred
let s = identity("hello"); // T = string, inferred
```

### Rank-2 (Requires Annotation)

A function that takes a *generic function* as a parameter requires an explicit annotation for the higher-rank type:

```atlas
fn apply<T>(f: (T) -> T, x: T) -> T {
    return f(x);
}
```

The parameter `f: (T) -> T` binds `T` in the function type — this is rank-2. Atlas requires the type to be written explicitly (no inference of higher-rank positions).

### Callbacks

Generic callbacks work naturally when the callback type is concrete:

```atlas
fn transform(f: (number) -> number, x: number) -> number {
    return f(x);
}
```

---

## Let-Polymorphism

Let bindings can be generalized to polymorphic type schemes, subject to two restrictions.

### Value Restriction

Only **syntactic values** (literals, array literals) can be generalized. Function calls and variable references are not generalized, to prevent unsoundness:

```atlas
let x = 42;        // SyntacticValue → can generalize
let y = identity(z); // NonValue → not generalized
```

### Monomorphism Restriction

**Mutable** bindings are never generalized:

```atlas
var counter = 0;   // mutable → not generalized
let answer = 42;   // immutable → generalized if type has free vars
```

### Recursive Let Bindings

Functions can call themselves recursively; the type checker hoists their signatures before checking the body:

```atlas
fn factorial(n: number) -> number {
    if (n == 0) { return 1; }
    return n * factorial(n - 1);  // self-reference valid
}
```

---

## Flow-Sensitive Typing

The type of a variable can be refined based on what the control flow reveals about it.

### Type Narrowing in Branches

After a `typeof` check, the type is narrowed in the matching branch:

```atlas
fn process(x: number | string) -> number {
    if (typeof(x) == "number") {
        return x;   // x is narrowed to number here
    }
    return 0;
}
```

### Type Widening at Merge Points

After an `if-else`, the types from both branches are joined (widened):

```atlas
fn get(flag: bool) -> number {
    var result = 0;
    if (flag) {
        result = 1;   // type: number
    } else {
        result = 2;   // type: number
    }
    return result;    // merged: number
}
```

### Immutable vs Mutable Tracking

| Binding Kind | Strategy |
|-------------|----------|
| `let` (immutable) | Precise update — type replaced exactly |
| `var` (mutable) | Conservative widening — LUB of old and new type |

### Loop Fixpoint Iteration

For loops, the type checker iterates until the type state stabilizes (fixpoint). Each iteration widens types that changed, ensuring soundness:

```
Pre-state:  i: number
Loop body:  i = i + 1   → post-state: i: number
Fixpoint:   stable (number == number) ✓
```

### Impossible Branch Detection

When a type is narrowed to `Never`, the branch is unreachable:

```atlas
fn check(x: number | string) -> bool {
    if (typeof(x) == "number") {
        // x: number
        return true;
    }
    // x: string (number excluded)
    return false;
}
```

---

## Unification Algorithm

The `UnificationEngine` in `typechecker/unification.rs` implements constraint-based unification.

### Constraints

| Constraint | Meaning |
|-----------|---------|
| `Equal(A, B)` | A and B must be the same type |
| `Assignable { from, to }` | `from` must be assignable to `to` |
| `Bound { ty, bound }` | `ty` must satisfy the bound |

### Occurs Check

Prevents infinite types. If type variable `T` appears in the type it would be unified with, the unification fails:

```
T = Option<T>  →  Error: InfiniteType
T = T[]        →  Error: InfiniteType
```

### Structural Unification

Two structural types unify when their shared member names have compatible types:

```
{ x: T, y: number } unifies with { x: string, y: number }
  → T = string
```

### Backtracking Unification

When unifying against a union type, the engine tries each member with backtracking:

```
number unified with (number | string)
  → tries number vs number → success
```

### Error Messages

All unification errors include human-readable messages via `.message()`:

| Error | Message Pattern |
|-------|----------------|
| `Mismatch` | `type mismatch: expected X, found Y` |
| `InfiniteType` | `infinite type: 'T' cannot equal ...` |
| `ConstraintViolation` | `'X' does not satisfy constraint 'Y': ...` |
| `Unsolvable` | `unsolvable constraint: ...` |

---

## Constraint-Based Inference

Constraints are accumulated during type-checking and solved in batch via `UnificationEngine::solve()`.

### Workflow

1. **Generate** — walk AST, emit constraints from expressions
2. **Simplify** — apply current substitutions, remove trivial constraints
3. **Solve** — unify remaining constraints, collecting errors
4. **Apply** — substitute solved type variables throughout

### Delayed Solving

Some constraints can only be solved after other constraints are solved. The `simplify()` step applies partial solutions before the main `solve()` pass, enabling delayed resolution.

---

## Cross-Module Inference

The type checker supports cross-module type information through `check_with_modules()`.

### Export Validation

Exported declarations are fully type-checked, including:
- Return types of exported functions
- Types of exported variables
- Generic type aliases

Duplicate exports of the same name are detected as errors (AT5008).

### Import Types

Imported symbols are resolved from the module registry. Their types are used in the importing module's type inference.

---

## Inference Heuristics

When type inference is ambiguous, heuristics guide the solver without sacrificing soundness.

### Prefer Simple

From a set of candidate types, prefer the simplest (primitive) type:

```
candidates: [Array<number>, number]  →  number
```

### Prefer Primitive

Priority order when ambiguous: `number > string > bool > null > compound`

### Union from Branches

When a variable receives different types in different branches, infer a union:

```atlas
fn get(flag: bool) -> number | string {
    if (flag) { return 42; }     // number
    return "hello";              // string
}
// inferred: number | string
```

### Minimize Type Variables

When inference cannot determine a concrete type, unresolved type parameters are replaced with `Unknown`:

```
Array<T>  →  Array<unknown>  (when T is unresolved)
```

### Literal Type Inference

Literal expressions directly determine their types:

| Literal | Inferred Type |
|---------|--------------|
| `42` | `number` |
| `"hello"` | `string` |
| `true` | `bool` |
| `null` | `null` |

---

## Implementation Files

| File | Role |
|------|------|
| `typechecker/inference.rs` | Bidirectional checking, let-polymorphism, heuristics |
| `typechecker/unification.rs` | Constraint accumulation and solving |
| `typechecker/flow_sensitive.rs` | Flow-sensitive type tracking |
| `typechecker/generics.rs` | Hindley-Milner unification (TypeInferer) |
| `typechecker/narrowing.rs` | Condition-based type narrowing |
| `typechecker/constraints.rs` | Generic constraint validation |

---

## Error Codes

| Code | Description |
|------|-------------|
| AT3001 | Type mismatch |
| AT3003 | Assignment to immutable variable |
| AT3004 | Not all code paths return a value |
| AT3010 | Break/continue outside loop |
| AT3011 | Return outside function |
| AT5008 | Duplicate export |
