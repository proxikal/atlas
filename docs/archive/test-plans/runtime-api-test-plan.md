# Atlas Runtime API Test Plan

## Goals
Validate the runtime API behaves consistently for embedding use cases.

## Test Categories
- `eval` returns correct values
- `eval` returns diagnostics on errors
- `eval_file` handles missing files and parse errors
- stdout redirection behavior

## Examples
- `eval("1 + 2")` -> `Value::Number(3)`
- `eval("let x: number = \"a\";")` -> diagnostic `AT0001`

## Exit Criteria
- All runtime API tests pass without relying on CLI.
