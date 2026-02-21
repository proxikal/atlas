# Atlas Warnings Test Plan

## Goals
Validate warning emission for unused variables and unreachable code.

## Test Categories
- Unused variable warnings in local scope
- Unused variable warnings at top level
- Unreachable code after `return`

## Examples
- `let x = 1;` in function -> `AT2001`
- `return 1; let y = 2;` -> `AT2002`
- `let _x = 1;` -> no warning

## Exit Criteria
- Warning tests pass with correct codes and spans.
