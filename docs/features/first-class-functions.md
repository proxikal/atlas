# First-Class Functions

**Status:** ✅ Implemented in v0.2
**Since:** v0.2.0
**Spec:** `Atlas-SPEC.md` - Types section

---

## Overview

Atlas supports first-class functions - functions can be stored in variables, passed as arguments to other functions, and returned from functions. This enables functional programming patterns like map, filter, reduce, and function composition.

**Key principle:** Functions are values, just like numbers and strings.

---

## Features

### 1. Store Functions in Variables

Functions can be assigned to variables and called through those variables.

```atlas
fn double(x: number) -> number {
    return x * 2;
}

let f = double;      // Store function in variable
let result = f(5);   // Call through variable
print(result);       // 10
```

**Works with:**
- `let` bindings (immutable)
- `var` bindings (mutable, can be reassigned)
- Builtin functions (`print`, `len`, `str`, etc.)

---

### 2. Pass Functions as Arguments

Functions can be parameters to other functions, enabling callbacks and higher-order functions.

```atlas
fn apply(func: (number) -> number, x: number) -> number {
    return func(x);
}

fn triple(n: number) -> number {
    return n * 3;
}

apply(triple, 7);  // 21
```

**Common patterns:**
- Map: Apply function to each element
- Filter: Keep elements matching predicate
- Reduce: Accumulate values with reducer function
- Callbacks: Execute function after operation

---

### 3. Return Functions from Functions

Functions can create and return other functions.

```atlas
fn getMultiplier(factor: number) -> (number) -> number {
    fn multiply(x: number) -> number {
        return x * factor;  // Note: 'factor' must be global in v0.2
    }
    return multiply;
}

let double = getMultiplier(2);
double(5);  // 10
```

**Use cases:**
- Factory functions
- Function composition
- Strategy pattern
- Conditional function selection

---

## Type Syntax

Function types use arrow syntax: `(param_types) -> return_type`

### Examples

```atlas
// Single parameter
(number) -> bool

// Multiple parameters
(number, string) -> number

// No parameters
() -> void

// Nested function types
((number) -> bool) -> string

// Array parameters/returns
(number[]) -> string[]

// Function type in variable declaration
let transform: (number) -> number = double;

// Function type in parameter
fn apply(f: (string) -> number, s: string) -> number {
    return f(s);
}

// Function type in return type
fn getFunc() -> (number) -> number {
    return double;
}
```

---

## Current Limitations (v0.2)

### Named Functions Only

v0.2 supports **named functions only**. Anonymous function syntax is planned for v0.3+.

```atlas
// ✅ Supported: Named function
fn isEven(x: number) -> bool {
    return x % 2 == 0;
}
filter([1, 2, 3, 4], isEven);

// ❌ Not yet: Anonymous function
filter([1, 2, 3, 4], fn(x) { return x % 2 == 0; });  // v0.3+
```

### No Closure Capture

Functions can only reference:
- Their own parameters
- Global variables
- Other functions

Functions **cannot** capture variables from outer scopes (closure capture).

```atlas
// ✅ Supported: Reference global
let multiplier = 2;
fn double(x: number) -> number {
    return x * multiplier;  // OK: multiplier is global
}

// ❌ Not yet: Closure capture
fn makeMultiplier(factor: number) -> (number) -> number {
    // 'factor' is not global - cannot be captured
    fn multiply(x: number) -> number {
        return x * factor;  // Error: 'factor' not accessible
    }
    return multiply;
}  // v0.3+
```

**Workaround:** Use global variables for shared state.

---

## Common Patterns

### Map Pattern

Apply a transformation to each element:

```atlas
fn applyToArray(arr: number[], f: (number) -> number) -> number[] {
    var result: number[] = [];
    for (var i = 0; i < len(arr); i++) {
        result = result + [f(arr[i])];
    }
    return result;
}

fn double(x: number) -> number { return x * 2; }

let arr = [1, 2, 3];
let doubled = applyToArray(arr, double);  // [2, 4, 6]
```

### Filter Pattern

Keep only elements matching a predicate:

```atlas
fn filterArray(arr: number[], predicate: (number) -> bool) -> number[] {
    var result: number[] = [];
    for (var i = 0; i < len(arr); i++) {
        if (predicate(arr[i])) {
            result = result + [arr[i]];
        }
    }
    return result;
}

fn isPositive(x: number) -> bool { return x > 0; }

let nums = [-2, -1, 0, 1, 2];
let positive = filterArray(nums, isPositive);  // [1, 2]
```

### Reduce Pattern

Accumulate values into a single result:

```atlas
fn reduceArray(
    arr: number[],
    reducer: (number, number) -> number,
    initial: number
) -> number {
    var acc = initial;
    for (var i = 0; i < len(arr); i++) {
        acc = reducer(acc, arr[i]);
    }
    return acc;
}

fn add(a: number, b: number) -> number { return a + b; }

let nums = [1, 2, 3, 4, 5];
let sum = reduceArray(nums, add, 0);  // 15
```

### Function Composition

Combine functions to create new functions:

```atlas
fn compose(
    f: (number) -> number,
    g: (number) -> number
) -> (number) -> number {
    fn composed(x: number) -> number {
        return f(g(x));
    }
    return composed;
}

fn double(x: number) -> number { return x * 2; }
fn inc(x: number) -> number { return x + 1; }

let doubleAndInc = compose(inc, double);
doubleAndInc(5);  // 11 (double(5) = 10, then inc(10) = 11)
```

---

## Type Checking

Function types are fully type-checked:

```atlas
// ✅ Type matches
fn double(x: number) -> number { return x * 2; }
let f: (number) -> number = double;

// ❌ Type error: wrong parameter count
fn add(a: number, b: number) -> number { return a + b; }
let g: (number) -> number = add;  // Error AT3001

// ❌ Type error: wrong return type
fn getStr() -> string { return "test"; }
let h: (number) -> number = getStr;  // Error AT3001

// ❌ Type error: calling non-function
let x: number = 5;
x();  // Error AT3006: Cannot call non-function type number
```

---

## Implementation Notes

### Interpreter

- Functions stored as `Value::Function(FunctionRef)` in globals
- Function bodies stored separately in `function_bodies` HashMap
- `eval_call()` evaluates callee as expression, extracts function value

### VM

- Functions loaded onto stack as `Value::Function`
- `Opcode::Call` handles function values from any source
- SetLocal/GetLocal/SetGlobal/GetGlobal work transparently with functions

### Parity

Both interpreter and VM produce identical results for all first-class function operations. Over 500 tests verify parity.

---

## Future Enhancements (v0.3+)

Planned for future versions:

1. **Anonymous Functions (Lambdas)**
   ```atlas
   let double = fn(x: number) -> number { return x * 2; };
   ```

2. **Closure Capture**
   ```atlas
   fn makeAdder(x: number) -> (number) -> number {
       return fn(y: number) -> number { return x + y; };
   }
   ```

3. **Partial Application**
   ```atlas
   fn add(a: number, b: number) -> number { return a + b; }
   let add5 = add(5);  // Returns (number) -> number
   ```

---

## Error Codes

| Code | Message | Example |
|------|---------|---------|
| AT3001 | Type mismatch in assignment | `let f: (number) -> bool = add;` where add has wrong signature |
| AT3006 | Cannot call non-function type | `let x = 5; x();` |
| AT0002 | Unknown function | Calling non-existent function |

---

## See Also

- **Atlas-SPEC.md** - Language specification
- **docs/api/stdlib.md** - Standard library reference
- **phases/stdlib/phase-02-complete-array-api.md** - Array API using callbacks
