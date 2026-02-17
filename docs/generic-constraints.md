# Generic Constraints and Bounds

This document describes the generic constraint system for Atlas.

## Overview

Generic constraints restrict type parameters to a subset of types. The syntax
uses `extends` to describe the bound:

```atlas
fn identity<T extends number>(value: T) -> T {
    return value;
}
```

When calling a generic function, the type checker infers type arguments and
verifies that the inferred types satisfy each constraint.

## Syntax

### Function Constraints

```atlas
fn compare<T extends Comparable>(left: T, right: T) -> bool {
    return left == right;
}
```

### Type Alias Constraints

```atlas
type NumericBox<T extends Numeric> = T;
```

### Multiple Constraints

Use intersection types to combine bounds:

```atlas
fn serialize<T extends Serializable & Equatable>(value: T) -> string {
    return str(value);
}
```

### Union Bounds

Use union bounds to permit alternatives:

```atlas
fn either<T extends number | string>(value: T) -> string {
    return str(value);
}
```

### Structural Bounds

Structural bounds describe required members with field or method signatures:

```atlas
fn stringify<T extends { as_string: () -> string }>(value: T) -> string {
    return value.as_string();
}
```

## Built-in Constraint Patterns

Atlas includes a set of conventional constraint names for common patterns:

- `Comparable` → types supporting comparison (currently `number`).
- `Numeric` → numeric types (`number`).
- `Equatable` → equality-compatible primitives (`number`, `string`, `bool`, `null`).
- `Serializable` → values convertible to string (`number`, `string`, `bool`, `null`, `json`).
- `Iterable` → array-like values (`Array<unknown>`).

These names are resolved to concrete type bounds during type checking.

## Constraint Checking Rules

- A type argument must be assignable to its bound.
- Intersection bounds require satisfying all member constraints.
- Union bounds require satisfying at least one member constraint.
- Structural bounds require all named members to be present and compatible.
- Conflicting constraints (e.g., `number & string`) are rejected.

## Error Messages

Constraint errors include:

- The type parameter name.
- The required constraint.
- The actual inferred type.

Example:

```
error[AT3001]: Type argument 'T' must satisfy constraint number, found string
```

## Examples

### Numeric Constraint

```atlas
fn add<T extends Numeric>(a: T, b: T) -> T {
    return a + b;
}
```

### Iterable Constraint

```atlas
fn first<T extends Iterable>(items: T) -> number {
    return items[0];
}
```

### Structural Constraint

```atlas
fn toText<T extends { as_string: () -> string }>(value: T) -> string {
    return value.as_string();
}
```

## Limitations

- Constraints apply to type parameters only.
- Higher-kinded constraints are reserved for future phases.
- Structural bounds are limited to member signatures known at type-check time.
