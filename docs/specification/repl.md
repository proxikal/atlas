# Atlas REPL Specification

**Version:** v0.2 (Draft)
**Status:** Living document

---

## Overview

The Atlas REPL (Read-Eval-Print Loop) provides an interactive environment for rapid prototyping, exploration, and learning. It differs from file mode in several key ways to optimize for interactive use.

**Key principle:** REPL is first-class, not an afterthought. Atlas is designed for REPL-first development.

---

## REPL vs File Mode Differences

| Feature | REPL Mode | File Mode |
|---------|-----------|-----------|
| **Semicolons** | Optional for single expressions | Required for statements |
| **Expression results** | Auto-printed | Discarded unless printed |
| **Error handling** | Continue session after error | Terminate on error |
| **Global scope** | Persistent across inputs | Fresh per execution |
| **Multi-line input** | Balance braces to end | Not applicable |

---

## Input Rules

### Single Expressions

REPL accepts single expressions without semicolons:

```atlas
>>> 2 + 2
4

>>> "hello" + " world"
"hello world"

>>> len([1, 2, 3])
3
```

### Statements

Statements still require semicolons:

```atlas
>>> let x = 42;
>>> var y = 10;
>>> y = y + 1;
```

### Multi-line Input

Input ends when braces are balanced:

```atlas
>>> fn factorial(n: number) -> number {
...   if (n <= 1) {
...     return 1;
...   }
...   return n * factorial(n - 1);
... }

>>> factorial(5)
120
```

**Indicators:**
- `>>>` - Primary prompt (ready for new input)
- `...` - Continuation prompt (waiting for balanced braces)

---

## State Persistence

### Global Scope

REPL maintains global scope across inputs:

```atlas
>>> let x = 10;
>>> let y = 20;
>>> x + y
30

>>> fn double(n: number) -> number {
...   return n * 2;
... }
>>> double(x)
20
```

### Function Declarations

Functions are available immediately after declaration:

```atlas
>>> fn greet(name: string) -> void {
...   print("Hello, " + name);
... }

>>> greet("Alice")
Hello, Alice
```

---

## Expression Evaluation

### Automatic Printing

Expressions are evaluated and results printed:

```atlas
>>> 42
42

>>> true && false
false

>>> [1, 2, 3]
[1, 2, 3]
```

### Type Display

Results show with their type representation:

```atlas
>>> 3.14
3.14  // number

>>> "text"
"text"  // string (quotes shown)

>>> null
null
```

### Void Results

Void expressions print nothing:

```atlas
>>> print("hello")
hello
// No result printed (void return)
```

---

## Type Checking

### Before Execution

REPL type-checks input before evaluation:

```atlas
>>> let x: number = "wrong";
error[AT3001]: Type mismatch
  expected number, found string

>>> x  // x was never created
error[AT2002]: Unknown symbol 'x'
```

### Incremental Checking

Each input is type-checked against current global scope:

```atlas
>>> let x = 42;
>>> x + "text"  // Type error
error[AT3002]: '+' requires both operands to be number or both to be string
```

---

## Error Handling

### Compile-Time Errors

Type errors, syntax errors reported without execution:

```atlas
>>> 1 + "2"
error[AT3002]: '+' requires both operands to be number or both to be string
// Session continues
```

### Runtime Errors

Runtime errors reported, session continues:

```atlas
>>> 10 / 0
error[AT0001]: Divide by zero
// Session continues

>>> let arr = [1, 2, 3];
>>> arr[10]
error[AT0006]: Array index out of bounds
// Session continues
```

### Error Recovery

REPL state remains valid after errors:

```atlas
>>> let x = 10;
>>> x / 0  // Runtime error
error[AT0001]: Divide by zero

>>> x  // x still exists
10
```

---

## Variable Declaration

### Immutable Bindings

`let` creates immutable binding:

```atlas
>>> let x = 42;
>>> x = 43;
error[AT3003]: Cannot reassign to immutable variable 'x'
```

### Mutable Bindings

`var` creates mutable binding:

```atlas
>>> var count = 0;
>>> count = count + 1;
>>> count
1

>>> count += 5;
>>> count
6
```

### Type Inference

Types can be inferred:

```atlas
>>> let x = 42;  // Inferred as number
>>> let s = "hello";  // Inferred as string
>>> let arr = [1, 2, 3];  // Inferred as number[]
```

---

## Function Declarations

### Top-Level Only

Functions can only be declared at top level:

```atlas
>>> fn outer() -> void {
...   fn inner() -> void {  // Error in v0.2
...     print("nested");
...   }
... }
error[AT1000]: Function declarations are only allowed at top level
```

### Hoisting

Function declarations are hoisted (can call before defining):

```atlas
>>> greet("World");  // Call before definition
Hello, World

>>> fn greet(name: string) -> void {
...   print("Hello, " + name);
... }
```

---

## Import/Export (v0.2+)

### Not Supported in REPL

Module imports/exports only work in file mode:

```atlas
>>> import { add } from "./math";
error[AT1001]: import/export only supported in file mode
```

**Rationale:** REPL is for single-file exploration, not multi-file programs.

---

## Built-in Functions

### Always Available

Prelude functions available in every session:

```atlas
>>> print("Hello")
Hello

>>> len("Atlas")
5

>>> str(42)
"42"
```

### Cannot Shadow

Redeclaring prelude names is an error:

```atlas
>>> let print = 42;
error[AT1012]: Cannot shadow prelude function 'print'
```

---

## Special Commands

### Not Implemented in v0.1/v0.2

Future REPL commands (planned):

- `.help` - Show help
- `.clear` - Clear session
- `.type <expr>` - Show type of expression
- `.ast <expr>` - Show AST
- `.exit` - Exit REPL

**Current:** Use Ctrl+C to exit

---

## Implementation Notes

### Core/UI Split

REPL implementation splits concerns:

- **Core:** Lexer, parser, typechecker, interpreter (pure logic)
- **UI:** Input handling, prompts, formatting (terminal interaction)

**See:** `docs/repl.md` for REPL architecture overview

### Execution Modes

REPL can use either execution engine:

- **Interpreter:** Default (simpler, better errors)
- **VM:** Optional (faster, production-like)

Both modes must produce identical output (parity requirement).

---

## Usage Examples

### Quick Calculation

```atlas
>>> 2 + 2 * 10
22

>>> (2 + 2) * 10
40
```

### Prototyping Function

```atlas
>>> fn fibonacci(n: number) -> number {
...   if (n <= 1) { return n; }
...   return fibonacci(n - 1) + fibonacci(n - 2);
... }

>>> fibonacci(10)
55
```

### Testing Array Operations

```atlas
>>> let nums = [1, 2, 3, 4, 5];
>>> nums[0]
1

>>> nums[2] = 10;
>>> nums
[1, 2, 10, 4, 5]
```

### String Manipulation

```atlas
>>> let name = "Atlas";
>>> "Hello, " + name
"Hello, Atlas"

>>> len(name)
5
```

---

## Limitations

### v0.2 Constraints

- No multi-file programs (file mode only)
- No import/export
- No persistent history across restarts
- No command history navigation (basic terminal only)
- No syntax highlighting

### Performance

- REPL prioritizes feedback speed over execution speed
- Use file mode + VM for performance-critical code
- Each input re-type-checks against global scope (not optimized)

---

## Future Enhancements (v0.3+)

- Command history (up/down arrows)
- Tab completion
- Syntax highlighting
- Multi-line editing
- `.type` and `.ast` inspection commands
- Session save/load
- Persistent history file

---

## Design Philosophy

**REPL-first:** Atlas is designed for interactive exploration. The REPL is not a toy - it's a first-class development environment suitable for real work.

**Fast feedback:** Type errors caught before execution. No waiting for runtime to discover type issues.

**Forgiving:** Errors don't kill the session. State preserved across errors.

**Discoverable:** Built-ins always available. No imports needed for core functionality.

**See:** `docs/specification/language-semantics.md` for design rationale
