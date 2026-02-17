# Type Aliases

Type aliases provide semantic names for type expressions. They improve readability,
reduce repetition, and keep complex signatures manageable without changing runtime
behavior.

## Syntax

```
type Name = TypeExpr;

type Name<T> = TypeExpr;

type Name<A, B> = TypeExpr;
```

Examples:

```
type UserId = string;

type Pair<T, U> = (T, U) -> T;

type ResultMap = HashMap<string, Result<number, string>>;
```

## Semantics

- Type aliases are compile-time only. They do not exist at runtime.
- Alias names can be used anywhere a type annotation is expected.
- Aliases resolve to their underlying type during type checking.
- Type equivalence is structural after alias expansion.

## Generic Aliases

Aliases can be parameterized with type parameters:

```
type Box<T> = T[];
let xs: Box<number> = [1, 2, 3];
```

If a generic alias is used without type arguments, Atlas will attempt to infer
arguments from context when possible (e.g., variable initializers). If inference
fails, a type error is reported.

## Circular Alias Detection

Atlas rejects circular alias definitions to prevent infinite expansion.

Invalid:

```
type A = B;
type B = A;
```

## Import/Export

Aliases are module-scoped and can be exported/imported like other declarations:

```
// types.atl
export type UserId = string;

// main.atl
import { UserId } from "./types";
let id: UserId = "abc";
```

## Documentation and Metadata

Doc comments (`///`) may be attached to a type alias declaration. The doc text
is stored in the AST and can be used by tooling.

The following tags are recognized:

- `@deprecated` marks the alias as deprecated and emits a warning on use.
- `@since <version>` records the alias introduction version for tooling.

Example:

```
/// @deprecated use UserIdV2 instead
/// @since 0.3
export type UserId = string;
```

## Notes

- Aliases are resolved before type checking; errors report the alias name when
  possible for clarity.
- Reflection reports alias information via the `TypeInfo` alias metadata.
