# Atlas Type System Specification

**Purpose:** Define Atlas type system, generics, and pattern matching.
**Status:** Living document — reflects current implementation.

---

## Type Categories

Atlas has a strict type system with the following categories:

- **Primitive:** `number`, `string`, `bool`, `void`, `null`
- **Arrays:** `T[]` or `Array<T>`
- **Function:** `(T1, T2) -> T3`
- **JSON:** `json` (isolated dynamic type)
- **Generic:** `Type<T1, T2, ...>`
- **Built-in generics:** `Option<T>`, `Result<T, E>`

---

## Primitive Types

### number
- 64-bit floating-point value (IEEE 754)
- `NaN` and `Infinity` results are runtime errors (`AT0007`)
- Examples: `42`, `3.14`, `-5.0`

### string
- UTF-8 encoded text
- Immutable
- Examples: `"hello"`, `"world\n"`, `""`

### bool
- Boolean values
- Only `true` or `false`
- No truthy/falsey coercion

### void
- Represents no value
- Used for function return types that don't return a value
- Cannot be stored in variables

### null
- Represents explicit absence of value
- Only assignable to `null` type (no implicit nullable)
- Example: `let x: null = null;`

---

## Function Types

Functions are first-class values that can be stored in variables, passed as arguments, and returned from functions.

### Syntax

```atlas
// Function type with one parameter
(number) -> bool

// Function type with multiple parameters
(number, string) -> number

// Function type with no parameters
() -> void

// Nested function types
((number) -> bool) -> string

// Function type with array parameters/returns
(number[]) -> string[]
```

### Examples

```atlas
// Store function in variable
fn double(x: number) -> number { return x * 2; }
let f = double;
f(5);  // 10

// Pass function as argument
fn apply(fn_param: (number) -> number, x: number) -> number {
    return fn_param(x);
}
apply(double, 5);  // 10

// Return function from function
fn getDouble() -> (number) -> number {
    return double;
}
let g = getDouble();
g(5);  // 10
```

### Current Limitations
- No anonymous function syntax (`fn(x) { ... }`) — planned Block 4
- All function values must be named functions
- **Let-bound variables at top-level scope** are accessible from any named function — works in
  both interpreter and VM.
- **Var-bound variables at top-level scope** are readable and mutable from any named function —
  works in both engines.
- **Inner functions referencing outer function locals:** Both engines support this via upvalue
  capture. The VM captures outer locals **by value at closure definition time**. The interpreter
  uses live dynamic scope lookup. For `let`-bound (immutable) outer variables, both engines
  produce identical results. For `var`-bound outer variables, the captured value reflects the
  state at the time the inner function was defined.
- **Returned closures:** A named inner function returned as a value and called after its defining
  scope has exited can only access top-level globals. Outer function locals captured at definition
  time (by value) are frozen in the upvalue slot.

**See:** `docs/internal/V03_PLAN.md` Block 4 for full closure semantics roadmap.

---

## JSON Type

The `json` type is an **isolated dynamic type** specifically for JSON interop. It is the **only exception** to Atlas's strict typing.

### Type Declaration

```atlas
let data: json = /* json value from API or parser */;
```

### Key Features

- **Natural indexing:** Supports both string keys (objects) and number indices (arrays)
- **Safe null semantics:** Missing keys/invalid indices return `null` instead of errors
- **Type isolation:** Cannot assign `json` to other types without explicit extraction
- **Structural equality:** JSON values compare by content, not reference

### Indexing

```atlas
// Object indexing with string keys
let name: json = data["user"]["name"];

// Array indexing with number indices
let first: json = items[0];

// Mixed indexing
let value: json = data["users"][0]["email"];
```

### Type Safety

- `json` values can only be assigned to `json`-typed variables
- Cannot use `json` in expressions: `data + 1` is a type error
- Extraction methods convert to typed values

### Current Limitations

- No JSON literal syntax in source code
- JSON values created via `json_parse()` or from Rust API

---

## Generic Types

Generic types enable parameterized types for reusable, type-safe code.

### Syntax

```atlas
// Generic type application
Type<T1, T2, ...>

// Generic function declaration
fn functionName<T, E>(param: T) -> Result<T, E> {
    // body
}
```

### Examples

```atlas
// Generic function with inference
fn identity<T>(x: T) -> T {
    return x;
}

let num = identity(42);        // T inferred as number
let str = identity("hello");   // T inferred as string

// Explicit type arguments
let result = identity<number>(42);

// Multiple type parameters
fn pair<A, B>(first: A, second: B) -> Result<A, B> {
    return Ok([first, second]);
}
```

### Built-in Generic Types

#### Option\<T\> - Represents optional values

```atlas
let some: Option<number> = Some(42);
let none: Option<number> = None;
```

#### Result\<T, E\> - Represents success or failure

```atlas
fn divide(a: number, b: number) -> Result<number, string> {
    if (b == 0) {
        return Err("division by zero");
    }
    return Ok(a / b);
}
```

#### Array\<T\> - Sugar for array types

```atlas
let arr1: number[] = [1, 2, 3];        // Sugar for Array<number>
let arr2: Array<number> = [1, 2, 3];  // Explicit generic form
```

### Semantics

- **Monomorphization:** Generates specialized code per type instantiation
- **Type inference:** Arguments infer type parameters when possible
- **Type parameters:** Lexically scoped to function declaration
- **No constraints:** All type parameters unbounded

### Current Limitations

- No user-defined generic types (only built-in: Option, Result, Array)
- No type parameter constraints/bounds — planned Block 3 (trait system)
- No variance (all type parameters invariant)
- No higher-kinded types

**See:** `docs/internal/V03_PLAN.md` Block 3 for trait system and generic constraints.

---

## Pattern Matching

Pattern matching enables destructuring and conditional logic based on value structure. Essential for ergonomic `Result<T,E>` and `Option<T>` handling.

### Syntax

```atlas
match expression {
    pattern1 => expression1,
    pattern2 => expression2,
    _ => default_expression
}
```

### Examples

```atlas
// Result handling
fn divide(a: number, b: number) -> Result<number, string> {
    if (b == 0) { return Err("division by zero"); }
    return Ok(a / b);
}

match divide(10, 2) {
    Ok(value) => print("Result: " + str(value)),
    Err(error) => print("Error: " + error)
}

// Option handling
match find([1, 2, 3], 2) {
    Some(index) => print("Found at " + str(index)),
    None => print("Not found")
}

// Literal patterns
match x {
    0 => "zero",
    1 => "one",
    _ => "many"
}

// Nested patterns
match result {
    Ok(Some(value)) => process(value),
    Ok(None) => use_default(),
    Err(error) => handle_error(error)
}
```

### Pattern Types

- **Literal:** `42`, `"hello"`, `true`, `false`, `null`
- **Wildcard:** `_` (matches anything, binds nothing)
- **Variable:** `value` (matches anything, binds to name)
- **Constructor:** `Ok(value)`, `Err(error)`, `Some(x)`, `None`
- **Array:** `[]`, `[x]`, `[x, y]` (fixed-length)

### Semantics

- Match is an expression (has a type/value)
- Exhaustiveness checking required (must handle all cases)
- All arms must return compatible types
- First-match-wins evaluation order
- Variable bindings scoped to arm expression

### Exhaustiveness

```atlas
// ✅ Exhaustive - all constructors covered
match result {
    Ok(value) => value,
    Err(error) => 0
}

// ❌ Non-exhaustive - compiler error
match result {
    Ok(value) => value
    // Missing: Err case
}

// ✅ Wildcard makes it exhaustive
match x {
    0 => "zero",
    _ => "non-zero"
}
```

### Current Limitations

- No guard clauses (`pattern if condition`) — planned Block 9
- No OR patterns (`0 | 1 | 2`) — planned Block 9
- No rest patterns in arrays (`[first, ...rest]`) — planned Block 9
- No struct patterns (no user-defined structs) — planned Block 3+

**See:** `docs/internal/V03_PLAN.md` Blocks 3 and 9.

---

## Module Types

Module system enables multi-file programs with explicit imports and exports.

**File extensions:** `.atl`

### Export Syntax

```atlas
// Export functions
export fn add(a: number, b: number) -> number {
    return a + b;
}

// Export variables
export let PI = 3.14159;
export var counter = 0;
```

### Import Syntax

```atlas
// Named imports
import { add, subtract } from "./math";

// Namespace import
import * as math from "./math";
// Usage: math.add(2, 3)
```

### Module Paths

```atlas
import { x } from "./sibling";        // Relative path
import { y } from "../parent";        // Parent directory
import { z } from "/src/utils";      // Absolute from project root
```

### Semantics

- **Single evaluation:** Each module executed exactly once
- **Caching:** Module exports cached by absolute path
- **Circular dependencies:** Detected and rejected (compile error)
- **Exports only:** Non-exported items are module-private
- **Initialization order:** Topological sort by dependencies

### Examples

```atlas
// math.atl
export fn add(a: number, b: number) -> number {
    return a + b;
}
export let PI = 3.14159;

// main.atl
import { add, PI } from "./math";
let result = add(2, 3);
print(str(PI));
```

### Namespace Imports

```atlas
import * as math from "./math";
let sum = math.add(10, 20);
print(str(math.PI));
```

### Current Limitations

- No default exports (`export default`)
- No re-exports (`export { x } from "./mod"`)
- No dynamic imports (all imports top-level)
- No type-only imports

See `docs/specification/modules.md` for full module documentation.

---

## Type Rules

### Assignment and Mutability

- `let` is immutable, `var` is mutable
- No implicit `any`
- Function params and return types must be explicit
- Local variables can be inferred from initializer

### Null Handling

- `null` is only assignable to `null` (no implicit nullable)
- No null coercion or automatic null checks

### Conditionals

- Conditionals require `bool` (no truthy/falsey coercion)
- Boolean operators (`&&`, `||`) short-circuit

### Type Compatibility

- `==` and `!=` require both operands have the same type; otherwise it's a type error
- No implicit type conversions

### Operators

- `+` is allowed for `number + number` and `string + string` only
- `<`, `<=`, `>`, `>=` are only valid for `number`

### Array Indexing

- Array indexing requires a `number` index
- Non-integer indices are runtime errors
- Out-of-bounds access is a runtime error

### JSON Indexing

- JSON indexing accepts `string` or `number`
- Missing keys/invalid indices return `null` (safe)
