# Atlas Runtime Specification

**Purpose:** Define Atlas runtime behavior and memory model.
**Status:** Living document — reflects current implementation.

---

## Overview

Atlas runtime model defines how values are represented in memory, how execution proceeds, and how the interpreter and VM maintain consistency.

**Key principle:** Interpreter/VM parity - both execution engines must produce identical output for all programs.

---

## Value Representation

### Primitive Values (Stack-allocated)
- `number` - 64-bit IEEE 754 floating-point
- `bool` - Boolean (true/false)
- `null` - Explicit absence

### Value Types (Copy-on-Write heap-allocated)
- `string` - Arc<String> (immutable, shared)
- `array` - `ValueArray(Arc<Vec<Value>>)` — CoW via `Arc::make_mut`
- `map` - `ValueHashMap(Arc<AtlasHashMap>)` — CoW via `Arc::make_mut`
- `set` - `ValueHashSet(Arc<AtlasHashSet>)` — CoW via `Arc::make_mut`
- `Queue` - `ValueQueue(Arc<VecDeque<Value>>)` — CoW via `Arc::make_mut`
- `Stack` - `ValueStack(Arc<Vec<Value>>)` — CoW via `Arc::make_mut`
- `Option` - `Option<Box<Value>>` (Some/None)
- `Result` - `Result<Box<Value>, Box<Value>>` (Ok/Err)
- `json` - `Arc<JsonValue>` (immutable, shared)
- `Regex` - `Arc<regex::Regex>` (compiled regex pattern)
- `DateTime` - `Arc<chrono::DateTime<Utc>>` (UTC timestamp)
- `HttpRequest` / `HttpResponse` - `Arc<T>` (immutable after construction)

### Reference Types (Explicit opt-in)
- `shared<T>` - `Arc<Mutex<Value>>` — explicit shared mutable state
- `function` - FunctionRef (name + arity + bytecode offset)

**See:** `docs/specification/memory-model.md` for the full CoW/ownership model.

---

## Memory Model

### Reference Counting
- Atomic reference counting (Arc), no GC
- Shared ownership for strings, arrays, JSON values, collections
- Interior mutability for mutable types via Mutex (Arc<Mutex<T>>)
- Deterministic cleanup on scope exit

### String Semantics
- UTF-8 encoded
- Immutable - string operations create new strings
- Shared via Arc (cheap cloning, thread-safe)
- `len(string)` returns Unicode scalar count

### Array Semantics
- Homogeneous elements (all same type)
- Mutable - element assignment supported
- **Value semantics (CoW):** mutation does not affect aliased copies
- **Structural equality:** arrays compare by content, not reference identity
- Indexing: whole numbers only (fractional = runtime error)
- Out-of-bounds: runtime error (`AT0006`)

### JSON Semantics
- Isolated from regular type system
- Immutable - indexing creates new values
- Safe indexing - returns `null` for missing keys/indices
- Structural equality (compares content)

---

## Execution Model

### Program Structure
- Top-level statements execute in order
- Function declarations hoisted (can call before definition)
- Variables must be declared before use (no forward reference)

### Function Calls
- Arguments evaluated left-to-right
- New scope created for function body
- Parameters bound to argument values (copy for primitives, reference for heap values)
- Return value passed to caller
- Scope destroyed on return

### Control Flow
- `if`/`while`/`for` evaluate condition, branch accordingly
- `break` exits innermost loop
- `continue` skips to next iteration
- `return` exits function with optional value

### Short-Circuit Evaluation
- `&&` - evaluates left; if false, returns false without evaluating right
- `||` - evaluates left; if true, returns true without evaluating right

---

## Error Handling

### Compile-Time Errors
- Syntax errors
- Type errors
- Invalid control flow (break/continue/return outside valid context)
- Redeclaration in same scope

### Runtime Errors
- Divide by zero (`AT0001`)
- Invalid numeric result - NaN, Infinity (`AT0007`)
- Out-of-bounds array access (`AT0006`)
- Invalid index - non-integer (`AT0103`)
- Type errors (if type system bypassed)
- Unknown function
- Stack overflow

### Error Propagation
- File mode: Errors terminate execution
- REPL mode: Errors reported, session continues
- Type-checking happens before execution (fail fast)

**See:** `docs/specification/diagnostic-system.md` for error codes and formats

---

## Scoping Rules

### Lexical Scoping
- Block scope for `let` and `var`
- Function parameters scoped to function body
- Shadowing allowed in nested scopes
- Redeclaration in same scope is error

### For Loop Scoping
- Init variable scoped to loop body
- Each iteration shares same binding

Example:
```atlas
for (let i = 0; i < 3; i = i + 1) {
    // i is scoped here
}
// i not accessible here
```

### For-In Loop Semantics

**Execution:**
1. Evaluate iterable expression (must be array)
2. Extract array elements
3. For each element:
   - Create new scope (if first iteration) or reuse existing scope
   - Bind loop variable to current element
   - Execute body
   - Check control flow (break/continue/return)
4. Exit loop when all elements processed or break encountered

**Variable Scoping:**
- Loop variable exists only within loop body
- Shadows outer variables with same name
- Outer variable unaffected after loop completes
- Each iteration rebinds same variable (not a new variable per iteration)

Example:
```atlas
let item: number = 100;  // outer variable

for item in [1, 2, 3] {
    // item here is loop variable (shadows outer)
    print(item);  // Prints 1, 2, 3
}

print(item);  // Prints 100 (outer variable unchanged)
```

**Break and Continue:**
- `break` exits the for-in loop immediately
- `continue` skips to next element
- Both work as expected in nested loops

**Performance:**
- For-in has same performance as manual index-based iteration
- Direct element access without index arithmetic
- No runtime overhead for iteration abstraction

---

## Variable Semantics

### Mutability
- `let` - immutable binding (cannot reassign)
- `var` - mutable binding (can reassign)
- Function parameters - immutable

### Initialization
- All variables must be initialized at declaration
- Type inference from initializer allowed
- No uninitialized variables

### Assignment
- Simple: `x = value`
- Array element: `arr[i] = value`
- Compound: `x += value` (var only)
- Increment/decrement: `++x`, `x++`, `--x`, `x--` (var only)

---

## Type Checking

### Static Type Checking
- All expressions have deterministic types at compile-time
- No implicit conversions
- `Unknown` type for error recovery only

### Type Rules
- `+` valid for `number + number` and `string + string`
- Arithmetic (`-`, `*`, `/`, `%`) valid for `number` only
- Comparisons (`<`, `<=`, `>`, `>=`) valid for `number` only
- Equality (`==`, `!=`) requires same type
- Logical operators require `bool`
- Conditionals require `bool`

**See:** `docs/specification/types.md` for complete type rules

---

## Numeric Semantics

### IEEE 754 Compliance
- All numbers are 64-bit floats
- Supports full range: `-1.7e308` to `1.7e308`
- Precision: ~15-17 decimal digits

### Invalid Results
- `NaN` is a runtime error
- `Infinity` is a runtime error
- Divide by zero is a runtime error
- Modulo by zero is a runtime error

**Rationale:** Fail fast on numeric edge cases. Atlas is not a numeric computing language.

---

## Function Semantics

### First-Class Functions
- Functions are values
- Can be stored in variables
- Can be passed as arguments
- Can be returned from functions

### Current Limitations
- No anonymous function syntax (`fn(x) { ... }`) — planned Block 4
- Named inner functions capture outer locals by value at definition time (see `types.md`)
- Full closure semantics (reference capture, mutation visibility) — planned Block 4

**See:** `docs/internal/V03_PLAN.md` Block 4.

### Calling Convention
- Callee-saves (function responsible for preserving state)
- Arguments passed by value (primitives) or reference (heap values)
- Return value passed back to caller

---

## Prelude

Built-in functions always in scope:

```atlas
print(value: string | number | bool | null) -> void
len(value: string | T[]) -> number
str(value: number | bool | null) -> string
```

**Note:** Prelude names cannot be shadowed - redeclaring `print`, `len`, or `str` is a compile error (`AT1012`)

**See:** `docs/specification/stdlib.md` for complete stdlib reference

---

## Interpreter vs VM

### Interpreter
- Tree-walking execution
- Direct AST evaluation
- Slower but simpler
- Used for REPL and debugging

### VM (Bytecode)
- Stack-based virtual machine
- Compiled to bytecode
- Faster execution
- Used for file mode and production

### Parity Requirement
- **CRITICAL:** Both must produce identical output
- Same error messages
- Same evaluation order
- Same memory semantics
- All tests must pass in both modes

**Note:** Parity is verified by running the same tests against both execution engines.

---

## Performance Characteristics

### Not Guaranteed
- Execution time
- Memory usage
- Stack depth limits

### Guaranteed
- Evaluation order (left-to-right for arguments)
- Short-circuit behavior (&&, ||)
- Deterministic cleanup (Arc drop on scope exit)
- Single evaluation of each expression

---

## v0.3 Roadmap (in progress)

The following are planned for v0.3 blocks:

- **Ownership annotations:** `own`, `borrow`, `shared` parameter syntax — Block 2
- **Closures + anonymous functions:** Block 4 (depends on Block 3 trait system)
- **async/await syntax:** Block 8 (runtime infrastructure exists, language syntax pending)

**See:** `docs/internal/V03_PLAN.md` for full block plan and dependencies.

---

## Related Documentation

- **Types:** `docs/specification/types.md`
- **Bytecode:** `docs/specification/bytecode.md`
- **Progress:** `STATUS.md`
- **Roadmap:** `ROADMAP.md`
