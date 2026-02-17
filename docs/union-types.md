# Union and Intersection Types

**Version:** v0.2 (Draft)

This document describes Atlas union (`|`) and intersection (`&`) types, type narrowing, and how they interact with the existing type system.

---

## Overview

Union and intersection types allow composing types without introducing new nominal types:

- **Union (`A | B`)**: a value may be **either** `A` or `B`.
- **Intersection (`A & B`)**: a value must satisfy **both** `A` and `B`.

These types are primarily compile-time constructs used by the type checker.

---

## Syntax

```atlas
// Union
let x: number | string = 42;
let y: number | string = "hello";

// Intersection
let z: number & number = 10;

// Mixed
let u: (number | string)[] = [1, "two", 3];
let f: ((number) -> number) | ((string) -> string) = identity;
```

Notes:
- Union and intersection types are **right-associative** by parsing order.
- `&` has **higher precedence** than `|`.
- Parentheses can be used to group types when needed.

---

## Type Semantics

### Union (`A | B`)

- A value is valid if it matches **any** member type.
- Assignment to a union is allowed from any member type.
- Assignment **from** a union requires the target to accept **all** union members.
- When combining types during inference, unions are used as a safe least-upper-bound.

### Intersection (`A & B`)

- A value must satisfy **all** member types.
- Intersection is useful for composing constraints.
- Incompatible primitive intersections (e.g., `number & string`) resolve to `never`.

### `never`

- `never` represents an empty set of values.
- `never` is the result of an impossible intersection or an empty union.

---

## Type Narrowing

Type narrowing refines the type of a value inside control flow branches.

### Guards

```atlas
let value: number | string = getValue();

if (isString(value)) {
    // value: string
    print(value);
} else {
    // value: number
    print(value + 1);
}
```

Supported guards:
- `isString(x)`
- `isNumber(x)`
- `isBool(x)`
- `isNull(x)`
- `isArray(x)`
- `isFunction(x)`
- `typeof(x) == "string" | "number" | "bool" | "null" | "array" | "function" | "json"`
- Equality comparisons against literals: `x == 1`, `x == "hi"`, `x == true`, `x == null`

### Notes

- Narrowing only applies within the branch scope.
- Non-overlapping comparisons narrow to `never` in the false branch.

---

## Match Exhaustiveness

When matching over a union, each member type must be fully covered unless a wildcard (`_`) is present.

```atlas
let v: bool | Option<number> = ...;

match v {
  true => 1,
  false => 2,
  Some(x) => x,
  None => 0,
}
```

---

## Examples

### Union in Function Return

```atlas
fn parse(input: string) -> number | string {
    if (input == "42") {
        return 42;
    }
    return input;
}
```

### Intersection for Constraints

```atlas
// A value that must satisfy both constraints
let constrained: number & number = 10;
```

---

## Error Examples

```atlas
let x: number & string = 1; // error: incompatible intersection
let y: number | string = true; // error: bool not in union
```

---

## Implementation Notes

- Union and intersection types are **normalized** by flattening nested unions/intersections.
- Duplicate members are removed during normalization.
- Union/intersection algebra may distribute intersections over unions:
  - `(A | B) & C` becomes `(A & C) | (B & C)`

