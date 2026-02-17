# Atlas Type Guards and Predicates

This document defines type guards and type predicates in Atlas v0.2. It covers syntax, built-in guards, user-defined guards, structural guards, discriminated unions, guard composition, and best practices.

---

## Contents

- Overview
- Syntax
- Built-in Guards
- User-Defined Guards
- Guard Semantics
- Control Flow Narrowing
- Structural Guards
- Discriminated Union Guards
- Guard Composition
- Performance Notes
- Error Diagnostics
- Patterns and Examples
- FAQ

---

## Overview

Type guards let you narrow union types safely using boolean predicates. A predicate is a function that returns `bool` and declares which parameter it narrows. When a predicate is used in a condition, the type checker refines the parameter type in the `then` branch and excludes it from the `else` branch.

Atlas type guards are inspired by TypeScript but adapted to Atlas's syntax and type system:

- Guards are declared on the function return type.
- The predicate explicitly names the parameter being narrowed.
- The predicate type must be a subtype of the parameter's type.
- Built-in guards exist for common runtime checks.

---

## Syntax

### Function predicate syntax

```
fn isThing(value: SomeType) -> bool is value: NarrowType {
    // ... returns a boolean
}
```

Rules:

- Predicate must appear after the return type.
- Predicate must use a parameter name from the function signature.
- Predicate type must be assignable to the parameter type.
- Return type must be `bool`.

### Examples

```
fn isStringValue(x: number | string) -> bool is x: string {
    return isString(x);
}
```

```
fn isUser(data: { name: string } | { id: number }) -> bool is data: { name: string } {
    return hasField(data, "name");
}
```

### Invalid examples

```
// ERROR: return type must be bool
fn isStringValue(x: number | string) -> number is x: string {
    return 1;
}
```

```
// ERROR: predicate parameter must be a function parameter
fn isStringValue(x: number | string) -> bool is y: string {
    return true;
}
```

```
// ERROR: predicate type must be assignable to parameter type
fn isStringValue(x: number) -> bool is x: string {
    return true;
}
```

---

## Built-in Guards

Atlas includes built-in type guards for common runtime checks. These are standard library functions available without imports.

### `isString(value)`

Returns `true` if the value is a string.

```
if (isString(x)) {
    let y: string = x;
}
```

### `isNumber(value)`

Returns `true` if the value is a number.

```
if (isNumber(x)) {
    let y: number = x;
}
```

### `isBool(value)`

Returns `true` if the value is a boolean.

```
if (isBool(x)) {
    let y: bool = x;
}
```

### `isNull(value)`

Returns `true` if the value is `null`.

```
if (isNull(x)) {
    let y: null = x;
}
```

### `isArray(value)`

Returns `true` if the value is an array.

```
if (isArray(x)) {
    let y: number[] = x;
}
```

### `isFunction(value)`

Returns `true` if the value is a function.

```
if (isFunction(x)) {
    let y: (number) -> number = x;
}
```

### `isObject(value)`

Returns `true` if the value is a JSON object.

```
if (isObject(x)) {
    let y: json = x;
}
```

### `isType(value, typeName)`

Generic runtime type check using string type names. This is especially useful when type names are dynamic or when building higher-level guards.

```
if (isType(x, "string")) {
    let y: string = x;
}
```

Supported type names:

- `"string"`
- `"number"`
- `"bool"`
- `"null"`
- `"array"`
- `"function"`
- `"json"`
- `"object"`

---

## User-Defined Guards

User-defined guards let you encode domain-specific logic while keeping type safety.

### Example: basic guard

```
fn isText(x: number | string) -> bool is x: string {
    return isString(x);
}

fn demo(x: number | string) -> number {
    if (isText(x)) {
        let y: string = x;
        return 1;
    }
    let y: number = x;
    return 2;
}
```

### Example: structural guard

```
type WithName = { name: string };
type WithId = { id: number };

fn isNamed(x: WithName | WithId) -> bool is x: WithName {
    return hasField(x, "name");
}
```

### Example: discriminated guard

```
type Ok = { tag: string, value: number };
type Err = { tag: number, message: string };

fn isOk(x: Ok | Err) -> bool is x: Ok {
    return hasTag(x, "ok");
}
```

---

## Guard Semantics

Guards are only trusted when used in boolean control-flow positions. The type checker narrows in the `then` branch and removes the guarded type from the `else` branch.

### Basic narrowing

```
fn test(x: number | string) -> number {
    if (isString(x)) {
        let y: string = x;
        return 1;
    }
    let y: number = x;
    return 2;
}
```

### Negative narrowing

```
fn test(x: number | string) -> number {
    if (!isString(x)) {
        let y: number = x;
        return 1;
    }
    let y: string = x;
    return 2;
}
```

---

## Control Flow Narrowing

Guards integrate with control flow analysis:

- `if` conditions
- `while` conditions
- short-circuit logic (`&&`, `||`)
- nested guard usage

### `if` with guard

```
if (isNumber(x)) {
    let y: number = x;
}
```

### `while` with guard

```
while (isString(x)) {
    let y: string = x;
    break;
}
```

### Nested guards

```
if (isString(x)) {
    if (hasField(x, "length")) {
        let y: string = x;
    }
}
```

---

## Structural Guards

Structural guards allow safe duck-typing checks.

### `hasField(value, name)`

Checks whether the value has a field/key by name. Supports JSON objects and hash maps at runtime.

```
if (hasField(x, "name")) {
    let y: { name: string } = x;
}
```

### `hasMethod(value, name)`

Checks whether the value has a callable member by name. At runtime this is a field/key existence check for JSON objects and hash maps.

```
if (hasMethod(x, "len")) {
    let y: { len: () -> number } = x;
}
```

### Structural guard with unions

```
type WithName = { name: string };
type WithId = { id: number };

fn test(x: WithName | WithId) -> number {
    if (hasField(x, "name")) {
        let y: WithName = x;
        return 1;
    }
    let y: WithId = x;
    return 2;
}
```

---

## Discriminated Union Guards

Discriminated unions are a common pattern for enums and result types. Guards can be built around a `tag` field.

### `hasTag(value, tagValue)`

Checks whether the value has a `tag` field matching the given value.

```
type Ok = { tag: string, value: number };
type Err = { tag: number, message: string };

fn test(x: Ok | Err) -> number {
    if (hasTag(x, "ok")) {
        let y: Ok = x;
        return 1;
    }
    let y: Err = x;
    return 2;
}
```

### Pattern: custom discriminated guard

```
fn isOk(x: Ok | Err) -> bool is x: Ok {
    return hasTag(x, "ok");
}
```

---

## Guard Composition

Guards can be combined with boolean logic:

### `&&` combines narrowings

```
if (isString(x) && isType(x, "string")) {
    let y: string = x;
}
```

### `||` forms a wider union

```
if (isString(x) || isNumber(x)) {
    let y: number | string = x;
}
```

### `!` negates a guard

```
if (!isString(x)) {
    let y: number = x;
}
```

---

## Performance Notes

Type guard checks are optimized for the common case:

- Built-in guards map directly to runtime type checks.
- Structural checks use O(1) key lookup in JSON objects and hash maps.
- Composite guards short-circuit via normal boolean logic.

Guidelines:

- Prefer built-in guards for common primitives.
- Use `isType` when type names are dynamic.
- Keep user-defined guards focused and side-effect free.

---

## Error Diagnostics

The type checker emits clear diagnostics for invalid predicates:

### Return type mismatch

```
fn isText(x: string | number) -> number is x: string { ... }
```

Error:

```
Type predicate requires bool return type, found number
```

### Unknown predicate parameter

```
fn isText(x: string | number) -> bool is y: string { ... }
```

Error:

```
Type predicate refers to unknown parameter 'y'
```

### Unsafe predicate type

```
fn isText(x: number) -> bool is x: string { ... }
```

Error:

```
Predicate type string is not assignable to parameter type number
```

---

## Patterns and Examples

### Pattern: validation + narrowing

```
fn isEmail(x: string | null) -> bool is x: string {
    if (isNull(x)) {
        return false;
    }
    return contains(x, "@");
}
```

### Pattern: use built-ins directly

```
fn demo(x: number | string) -> number {
    if (isString(x)) {
        return len(x);
    }
    return x;
}
```

### Pattern: structural checks

```
type User = { name: string, id: number };

fn isUser(x: User | json) -> bool is x: User {
    return hasField(x, "name") && hasField(x, "id");
}
```

### Pattern: discriminated unions

```
type Ok = { tag: string, value: number };
type Err = { tag: number, message: string };

type Result = Ok | Err;

fn isOk(x: Result) -> bool is x: Ok {
    return hasTag(x, "ok");
}
```

### Pattern: guard-based APIs

```
fn ensureString(x: number | string) -> string {
    if (isString(x)) {
        return x;
    }
    return "";
}
```

---

## FAQ

### Do guards change runtime behavior?

No. Guards are just boolean functions. They inform the type checker when used in conditions.

### Do guards work in nested scopes?

Yes. Guards apply in the scope where they are evaluated.

### Can I create guards for generic types?

Yes, but the predicate type must be assignable to the parameter type. Prefer explicit unions rather than unconstrained type parameters.

### Are guards required for pattern matching?

No. Pattern matching and guards are complementary.

---

## Additional Examples

### Combining multiple guards

```
fn demo(x: number | string | bool) -> number {
    if (isString(x) || isNumber(x)) {
        return 1;
    }
    return 2;
}
```

### Guard usage in loops

```
fn demo(x: number | string) -> number {
    while (isString(x)) {
        return len(x);
    }
    return 0;
}
```

### Guard chaining

```
fn isJsonObject(x: json | string) -> bool is x: json {
    return isObject(x);
}

fn demo(x: json | string) -> number {
    if (isJsonObject(x) && hasField(x, "name")) {
        return 1;
    }
    return 2;
}
```

### Guard with type aliases

```
type Id = number | string;

fn isStringId(x: Id) -> bool is x: string {
    return isString(x);
}
```

### Guard for arrays

```
fn isNumberList(x: number[] | string) -> bool is x: number[] {
    return isArray(x);
}
```

### Guard for functions

```
fn isUnary(x: (number) -> number | string) -> bool is x: (number) -> number {
    return isFunction(x);
}
```

---

## Reference Summary

- Use `-> bool is param: Type` for predicates.
- Predicates narrow the named parameter.
- Built-in guards are available in `stdlib`.
- Structural guards are `hasField`, `hasMethod`, `hasTag`.
- Guard composition follows normal boolean semantics.

