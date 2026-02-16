# Language Specification

**Version:** 1.0.0
**Status:** Draft

---

## Introduction

This is a sample language specification for testing purposes.

## Syntax

The language syntax is defined using EBNF grammar.

### Expressions

Expressions are the basic building blocks.

```ebnf
expression = term { ("+" | "-") term }
term = factor { ("*" | "/") factor }
factor = number | identifier | "(" expression ")"
```

### Statements

Statements control program flow.

```ebnf
statement = assignment | if_statement | while_statement
assignment = identifier "=" expression ";"
if_statement = "if" expression "{" statement "}"
while_statement = "while" expression "{" statement "}"
```

## Semantics

### Type System

The language uses static typing.

```rust
// Example type annotation
let x: i32 = 42;
```

### References

See [expressions](#expressions) for more details.
Also check [external doc](../other.md#section).

## Examples

```rust
fn main() {
    let x = 10;
    let y = 20;
    println!("{}", x + y);
}
```

---
