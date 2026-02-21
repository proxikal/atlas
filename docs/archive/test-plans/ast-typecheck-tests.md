# AST & Typecheck Dump Tests

## Goals
Ensure AST and typecheck JSON dumps are deterministic and schema-compliant.

## Test Categories
- AST dump for simple expressions
- AST dump for functions and blocks
- Typecheck dump for variable bindings
- Typecheck dump for function signatures

## Conventions
- Expected outputs stored as `.json` next to `.atl` inputs.
- JSON outputs must include version fields.

## Exit Criteria
- Dumps match expected JSON across machines.
