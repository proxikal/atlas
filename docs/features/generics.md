# Generics

**Status:** ✅ Implemented (v0.2 BLOCKER 02-03)
**Specification:** docs/specification/types.md (Generic Types section)
**Implementation:** crates/atlas-runtime/src/typechecker/generics.rs

---

## Overview

Atlas generics enable type-safe parametric polymorphism for functions and types.

**Key features:**
- Generic functions with type parameters
- Type inference for generic calls
- Built-in generic types (`Option<T>`, `Result<T, E>`, `Array<T>`)
- Pattern matching integration
- Type safety with no runtime overhead

---

## Generic Functions

### Syntax

```atlas
fn functionName<T>(param: T) -> T {
    return param;
}

fn functionName<T, U>(param1: T, param2: U) -> U {
    return param2;
}
```

**Type parameters:**
- Declared in angle brackets `<T, U, V>`
- Single uppercase letters by convention (T, E, K, V)
- Scoped to function body

### Type Inference

Atlas infers type arguments from call sites:

```atlas
fn identity<T>(value: T) -> T {
    return value;
}

let x = identity(42);        // T inferred as number
let y = identity("hello");   // T inferred as string
let z = identity(true);      // T inferred as bool
```

**No explicit type arguments needed** - inference is automatic.

### Multiple Type Parameters

```atlas
fn pair<T, U>(first: T, second: U) -> T[] {
    // Example: could return array of first type
    return [first];
}

let result = pair(42, "hello");  // T=number, U=string
```

### Generic Return Types

Type parameters in return types are inferred from parameters:

```atlas
fn wrap<T>(value: T) -> Option<T> {
    return Some(value);
}

let maybeNumber = wrap(42);      // Option<number>
let maybeString = wrap("hi");    // Option<string>
```

---

## Built-in Generic Types

### Option<T>

Represents optional values (value or absence).

**Constructors:**
- `Some(value)` - Contains a value of type `T`
- `None` - Represents absence

**Usage:**

```atlas
let some: Option<number> = Some(42);
let none: Option<number> = None;

match some {
    Some(value) => print(str(value)),
    None => print("no value")
}
```

**Type safety:**
```atlas
let x: Option<number> = Some("text");  // ❌ AT3001: Type mismatch
let y: Option<string> = Some("text");  // ✅ Correct
```

### Result<T, E>

Represents success (`Ok`) or failure (`Err`) with typed errors.

**Constructors:**
- `Ok(value)` - Success with value of type `T`
- `Err(error)` - Failure with error of type `E`

**Usage:**

```atlas
fn divide(a: number, b: number) -> Result<number, string> {
    if (b == 0) {
        return Err("division by zero");
    }
    return Ok(a / b);
}

let result = divide(10, 2);
match result {
    Ok(value) => print(str(value)),
    Err(error) => print(error)
}
```

**Type safety:**
```atlas
let r: Result<number, string> = Ok(42);      // ✅
let bad: Result<string, number> = Ok(42);    // ❌ AT3001: Type mismatch
```

### Array<T>

Generic array type (equivalent to `T[]` syntax).

```atlas
let numbers: Array<number> = [1, 2, 3];
let strings: Array<string> = ["a", "b"];
```

**Note:** Prefer `T[]` syntax (more concise). `Array<T>` provided for consistency with stdlib functions.

---

## Pattern Matching with Generics

### Option Patterns

```atlas
let value: Option<number> = Some(42);

match value {
    Some(n) => print(str(n)),     // n: number (inferred)
    None => print("none")
}
```

**Exhaustiveness checking:**
```atlas
match value {
    Some(n) => print(str(n))
    // ❌ AT3027: Non-exhaustive match on Option: missing None
}
```

### Result Patterns

```atlas
let result: Result<number, string> = Ok(42);

match result {
    Ok(value) => print(str(value)),   // value: number
    Err(error) => print(error)        // error: string
}
```

### Nested Generics

```atlas
let nested: Option<Result<number, string>> = Some(Ok(42));

match nested {
    Some(Ok(value)) => print(str(value)),
    Some(Err(error)) => print(error),
    None => print("none")
}
```

---

## Type Inference Rules

### Basic Inference

Type parameters are inferred from **argument types**:

```atlas
fn wrap<T>(value: T) -> Option<T> {
    return Some(value);
}

let x = wrap(42);  // T = number (inferred from 42)
```

### Unification

When multiple arguments constrain the same type parameter, they must unify:

```atlas
fn same<T>(a: T, b: T) -> bool {
    return a == b;
}

let x = same(1, 2);         // ✅ T = number
let y = same(1, "text");    // ❌ AT3001: Type inference failed
```

### Return Type Inference

Return type uses inferred type arguments:

```atlas
fn getFirst<T>(arr: T[]) -> Option<T> {
    if (len(arr) == 0) {
        return None;
    }
    return Some(arr[0]);
}

let nums = [1, 2, 3];
let first = getFirst(nums);  // first: Option<number>
```

---

## Error Codes

| Code | Error | Description |
|------|-------|-------------|
| AT3001 | Type mismatch | Generic type doesn't match expected type |
| AT3020 | Empty match | Match expression has no arms |
| AT3021 | Match arm type mismatch | Match arms return incompatible types |
| AT3022 | Pattern type mismatch | Pattern doesn't match scrutinee type |
| AT3023 | Constructor arity mismatch | Wrong number of arguments to Some/Ok/Err/None |
| AT3024 | Unknown constructor | Invalid constructor for Option/Result |
| AT3025 | Unsupported pattern type | Constructor patterns not supported for this type |
| AT3027 | Non-exhaustive match | Match doesn't cover all cases for Option/Result/bool |

---

## Limitations (v0.2)

**Not supported:**
- ❌ User-defined generic types (structs, enums)
- ❌ Generic constraints/bounds (`where T: Trait`)
- ❌ Generic type aliases
- ❌ Higher-kinded types
- ❌ Explicit type arguments at call site (`fn<number>(42)`)

**Supported:**
- ✅ Generic functions
- ✅ Built-in generic types (Option, Result, Array)
- ✅ Type inference
- ✅ Pattern matching

**Planned for future versions** (v0.3+)

---

## Best Practices

1. **Use inference** - Let the compiler infer types, don't annotate unnecessarily
2. **Descriptive type params** - Use `T` for generic type, `E` for error, `K`/`V` for key/value
3. **Prefer Option over null** - Use `Option<T>` instead of nullable types
4. **Use Result for fallible operations** - Better than throwing errors
5. **Match exhaustively** - Handle all cases in pattern matching

---

## Examples

### Safe Array Access

```atlas
fn getAt<T>(arr: T[], index: number) -> Option<T> {
    if (index < 0 or index >= len(arr)) {
        return None;
    }
    return Some(arr[index]);
}

let nums = [10, 20, 30];
let value = getAt(nums, 1);  // Option<number>

match value {
    Some(n) => print(str(n)),  // "20"
    None => print("out of bounds")
}
```

### Fallible Operations

```atlas
fn parseNumber(s: string) -> Result<number, string> {
    // Simplified example
    if (s == "42") {
        return Ok(42);
    }
    return Err("not a number");
}

let result = parseNumber("42");
match result {
    Ok(n) => print(str(n)),
    Err(e) => print(e)
}
```

### Generic Utility Functions

```atlas
fn getOrElse<T>(opt: Option<T>, default: T) -> T {
    match opt {
        Some(value) => return value,
        None => return default
    }
}

let x = Some(42);
let y = None;

print(str(getOrElse(x, 0)));  // "42"
print(str(getOrElse(y, 0)));  // "0"
```

### Chaining Operations

```atlas
fn mapOption<T, U>(opt: Option<T>, fn: (T) -> U) -> Option<U> {
    match opt {
        Some(value) => return Some(fn(value)),
        None => return None
    }
}

fn double(x: number) -> number {
    return x * 2;
}

let x = Some(21);
let doubled = mapOption(x, double);  // Some(42)
```

---

## Implementation Notes

**Type Inference Algorithm:**
- Uses unification-based inference
- Bidirectional type checking for better inference
- Error messages show inference failures with context

**Pattern Matching:**
- Constructor patterns desugar to type checks
- Exhaustiveness checking verifies all cases covered
- Non-exhaustive matches are compile-time errors

**Runtime:**
- Generics are fully erased at runtime (zero cost)
- Type parameters don't affect bytecode
- Pattern matching compiles to efficient tag checks

---

## See Also

- **Specification:** `docs/specification/types.md` (Generic Types section)
- **Implementation Phases:**
  - `phases/typing/BLOCKER-02-generic-type-system.md`
  - `phases/typing/BLOCKER-03-pattern-matching.md`
- **Related Features:**
  - Type system: `docs/specification/types.md`
  - Pattern matching: `docs/features/pattern-matching.md`
  - Stdlib generic functions: `docs/api/stdlib.md` (Array functions)
