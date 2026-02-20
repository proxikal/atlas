# Atlas Language Semantics Reference

**Purpose:** Comprehensive reference for Atlas language semantics, type rules, and execution behavior

**For AI Agents:** This is your single source of truth for language semantics. Read this to understand string handling, array aliasing, numeric edge cases, operator rules, and execution order.

---

## Table of Contents
1. [String Semantics](#string-semantics)
2. [Array Aliasing & Mutation](#array-aliasing--mutation)
3. [Numeric Edge Cases](#numeric-edge-cases)
4. [Operator Type Rules](#operator-type-rules)
5. [Top-Level Execution](#top-level-execution)

---

## String Semantics

### Purpose
Lock string behavior for correctness and AI predictability. Strings in Atlas are designed to be explicit and deterministic.

### Core Rules

**UTF-8 Encoding:**
- All strings are UTF-8 encoded
- String literals support escape sequences: `\n \r \t \\ \"`
- Invalid UTF-8 sequences are lexer errors

**Length Semantics:**
- `len(string)` returns **Unicode scalar count**, NOT byte count
- Example: `len("hello")` → `5`
- Example: `len("🌟")` → `1` (one Unicode scalar, not 4 bytes)
- This ensures consistent behavior across ASCII and multi-byte characters

**Concatenation:**
- String concatenation uses `+` operator
- `+` is ONLY valid between two strings: `string + string → string`
- Mixing types is a compile-time error: `"hello" + 5` → `AT0001` (type mismatch)
- No implicit string coercion from other types

**Immutability:**
- Strings are immutable once created
- String operations (concatenation, substring, etc.) produce NEW strings
- No in-place string modification

### Test Coverage
- ASCII length: `len("hello")` → `5`
- Multi-byte length: `len("日本")` → `2` (two Unicode scalars)
- Emoji length: `len("🚀")` → `1`
- Concatenation: `"hello" + " " + "world"` → `"hello world"`
- Type error: `"hello" + 5` → `AT0001` type mismatch

---

## Array Aliasing & Mutation

### Purpose
Define mutation visibility and aliasing rules for arrays. Atlas arrays are reference-counted and mutable.

### Core Rules

**Reference Semantics:**
- Arrays are reference-counted (`Arc<Mutex<Vec<Value>>>`)
- Arrays are shared by reference, not deep-copied
- Assignment copies the reference: `let b = a` creates an alias, not a copy
- Function arguments pass references: arrays passed to functions share same underlying data

**Mutation Visibility:**
- Mutation through one reference is visible to ALL aliases
- Example:
  ```atlas
  let a = [1, 2, 3];
  let b = a;        // b is an alias to same array
  a[0] = 99;        // Mutate through a
  // b[0] is now 99  // Visible through b
  ```

**Assignment vs Index Assignment:**
- Reassignment creates NEW binding: `var a = [1]; a = [2];` (a now points to different array)
- Index assignment mutates: `var a = [1]; a[0] = 2;` (same array, different content)

**Mutability Requirements:**
- Index assignment requires `var` (mutable variable): `AT3003` if target is `let`
- Array content is always mutable (even if bound to `let`)
- Example:
  ```atlas
  let a = [1, 2, 3];
  a[0] = 99;        // OK: mutating array content
  a = [4, 5, 6];    // ERROR: cannot reassign let binding
  ```

### Test Coverage
- Aliasing: Assign array to another variable; mutate; verify both see changes
- Function arguments: Pass array to function; mutate inside function; verify caller sees changes
- Mutability: Index assignment on `let` binding works, reassignment fails

---

## Numeric Edge Cases

### Purpose
Lock behavior for numeric edge cases and avoid runtime surprises. Atlas rejects silent numeric errors.

### Core Rules

**Number Type:**
- All numbers are 64-bit floating-point (IEEE 754)
- Supports integers, decimals, scientific notation
- Example: `42`, `3.14`, `1.5e-3`, `6.022e23`

**Divide by Zero:**
- Division by zero is a **runtime error** (`AT0005`)
- Applies to `/` operator and `%` modulo operator
- Example: `1 / 0` → Runtime error `AT0005`
- Example: `0 / 0` → Runtime error `AT0005` (caught before NaN)

**NaN and Infinity:**
- ANY operation producing `NaN` is a **runtime error** (`AT0007`)
- ANY operation producing `Infinity` or `-Infinity` is a **runtime error** (`AT0007`)
- Example: `1e308 * 1e308` → Runtime error `AT0007` (overflow to Infinity)
- Example: `0.0 / 0.0` (if it ever bypassed division check) → `AT0007`

**Rationale:**
- Explicit errors better than silent NaN propagation
- AI agents can't predict NaN behavior reliably
- Fail fast and clearly, not silently corrupt computation

### Test Coverage
- `1 / 0` → `AT0005` divide by zero
- `0 / 0` → `AT0005` divide by zero (before NaN can occur)
- `5 % 0` → `AT0005` modulo by zero
- `1e308 * 1e308` → `AT0007` overflow to Infinity
- `-1e308 * 1e308` → `AT0007` overflow to -Infinity

---

## Operator Type Rules

### Purpose
Define operator validity and typing rules for consistency. Every operator has explicit type requirements.

### Arithmetic Operators

**Binary Arithmetic:** `+ - * / %`
- **Valid for:** `number` operands only
- **Returns:** `number`
- **Type error:** Using with non-number operands → `AT0001` type mismatch
- **Example:** `5 + 3` → `8`
- **Example:** `5 + "3"` → `AT0001` (type mismatch)

**Unary Minus:** `-expr`
- **Valid for:** `number` only
- **Returns:** `number`
- **Example:** `-5` → `-5`

### String Concatenation

**String Concatenation:** `+`
- **Valid for:** `string + string` ONLY
- **Returns:** `string`
- **Type error:** Mixing types → `AT0001`
- **Example:** `"hello" + " " + "world"` → `"hello world"`
- **Example:** `"hello" + 5` → `AT0001` (type mismatch)

**Note:** `+` is overloaded for `number + number` and `string + string`, but NEVER mixed types.

### Comparison Operators

**Numeric Comparison:** `< <= > >=`
- **Valid for:** `number` operands only
- **Returns:** `bool`
- **Type error:** Non-number operands → `AT0001`
- **Example:** `5 < 10` → `true`
- **Example:** `"a" < "b"` → `AT0001` (no string comparison)

**Equality:** `== !=`
- **Valid for:** Same-type operands only
- **Returns:** `bool`
- **Type error:** Different types → `AT0001`
- **Example:** `5 == 5` → `true`
- **Example:** `5 == "5"` → `AT0001` (type mismatch)
- **Example:** `null == null` → `true`

**Array Equality:**
- Arrays compare by **reference identity**
- Two arrays with same content but different identities are NOT equal
- Example: `[1, 2] == [1, 2]` → `false` (different array objects)
- Example: `let a = [1]; let b = a; a == b` → `true` (same reference)

### Logical Operators

**Binary Logical:** `&& ||`
- **Valid for:** `bool` operands only
- **Returns:** `bool`
- **Short-circuiting:** `&&` stops at first false, `||` stops at first true
- **Type error:** Non-bool operands → `AT0001`
- **Example:** `true && false` → `false`
- **Example:** `5 && true` → `AT0001` (no truthy/falsy coercion)

**Unary Logical:** `!expr`
- **Valid for:** `bool` only
- **Returns:** `bool`
- **Example:** `!true` → `false`

### Assignment Operators

**Compound Assignment:** `+= -= *= /= %=`
- **Valid for:** `number` targets and `number` values
- **Requires:** Mutable target (`var`, not `let`)
- **Type error:** Non-number → `AT0001`
- **Mutability error:** `let` target → `AT3003`
- **Example:** `var x = 5; x += 3;` → `x` is now `8`
- **Example:** `let x = 5; x += 3;` → `AT3003` (immutable)

**Increment/Decrement:** `++ --` (pre and post)
- **Valid for:** `number` targets only
- **Requires:** Mutable target (`var`, not `let`)
- **Type error:** Non-number → `AT0001`
- **Mutability error:** `let` target → `AT3003`
- **Example:** `var x = 5; x++;` → `x` is now `6`
- **Example:** `let x = 5; x++;` → `AT3003` (immutable)

**Simple Assignment:** `=`
- **Requires:** Mutable target (`var`) for reassignment
- **Immutability error:** Reassigning `let` binding → `AT3003`
- **Index assignment:** Always allowed on array (even if bound to `let`)

### Diagnostic Codes Summary

- `AT0001`: Type mismatch (wrong operand types)
- `AT0005`: Divide by zero (runtime)
- `AT0007`: NaN or Infinity result (runtime)
- `AT3003`: Immutability violation (assigning to `let`)

---

## Top-Level Execution

### Purpose
Lock rules for top-level execution order and function hoisting. Predictable execution order aids AI reasoning.

### Core Rules

**Execution Order:**
- Top-level statements execute in **source order**
- Each statement completes before the next begins
- Side effects (print, variable assignment) occur in order

**Function Hoisting:**
- Top-level function declarations are **hoisted**
- Functions can be called before their textual definition in source
- Example:
  ```atlas
  foo();              // OK: function hoisted
  fn foo() { ... }
  ```

**Variable Forward Reference:**
- Variables must be declared **before use**
- No forward reference for variables
- Using undeclared variable → `AT0002` (undefined symbol)
- Example:
  ```atlas
  let x = y;          // ERROR: y not yet declared
  let y = 5;
  ```

**Hoisting Scope:**
- Top-level functions are hoisted globally
- Nested functions are hoisted within their enclosing scope
- Variable declarations are NOT hoisted

**Declaration Order:**
- `let` and `var` declarations execute at their position in source
- Assignment happens at declaration site
- Example:
  ```atlas
  print(x);           // ERROR: x not declared yet
  let x = 5;
  ```

**Nested Function Hoisting:**
Nested functions are hoisted within their scope using two-pass binding:
```atlas
fn outer() -> number {
    return helper(21);  // ✅ OK: helper is hoisted in outer's scope

    fn helper(x: number) -> number {
        return x * 2;
    }
}
```

Sibling nested functions can call each other:
```atlas
fn outer() -> number {
    fn a() -> number { return b() + 10; }  // ✅ OK
    fn b() -> number { return 32; }
    return a();  // Returns 42
}
```

### Test Coverage
- Call function before declaration → works (hoisted)
- Use variable before declaration → `AT0002` undefined symbol
- Top-level statements with side effects execute in order
- Nested function forward references → works (hoisted within scope)
- Sibling nested function calls → works

---

## Semantic Consistency Principles

**Explicitness:**
- No implicit type coercion (no truthy/falsy, no auto-conversion)
- No silent failures (NaN/Infinity are errors)
- No automatic deep copies (explicit reference semantics)

**Predictability:**
- Same input always produces same output
- No platform-specific behavior
- No undefined behavior zones

**AI-Friendliness:**
- Clear error messages with diagnostic codes
- Deterministic execution (no non-determinism)
- Simple mental model (strings immutable, arrays mutable, numbers strict)

**Type Safety:**
- Strict type checking at compile time
- Minimal runtime type errors (only for edge cases like divide by zero)
- No `any` type or implicit nullability

---

## Quick Reference

**Strings:**
- UTF-8, immutable, `len()` returns scalar count
- `+` for concatenation, only `string + string`

**Arrays:**
- Reference-counted, mutable, shared by reference
- Index assignment allowed on `let`, reassignment needs `var`

**Numbers:**
- 64-bit float, division by zero is error, NaN/Infinity are errors

**Operators:**
- Arithmetic: `number` only
- String concat: `string` only
- Comparison: same types only
- Logical: `bool` only
- No implicit coercion, ever

**Execution:**
- Top-level statements in order
- Functions hoisted, variables not

---

**For implementation details, see:**
- `docs/implementation/09-value-model.md` - Rust value representation
- `docs/implementation/10-interpreter.md` - Interpreter semantics
- `docs/implementation/12-vm.md` - VM semantics
- `docs/implementation/07-typechecker.md` - Type checking rules
