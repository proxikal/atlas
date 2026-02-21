# Atlas Prelude Test Plan

## Goals
Validate prelude built-ins are always available and shadowing rules are correct.

## Test Categories
- Prelude availability in global scope
- Shadowing in nested scopes allowed
- Shadowing in global scope disallowed

## Examples
- `print(1)` works without imports
- `fn f() { let print = 1; }` allowed
- `let print = 1;` at top-level produces diagnostic `AT1012`

## Exit Criteria
- Prelude tests pass with correct diagnostics.
