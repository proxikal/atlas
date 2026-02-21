# Atlas Keyword Policy

## Purpose
Define reserved keywords and how the lexer/parser should treat them.

## Reserved Keywords (v0.1)
- `let`, `var`, `fn`, `if`, `else`, `while`, `for`, `return`, `break`, `continue`
- `true`, `false`, `null`
- `match` (reserved for future use)
- `import` (reserved for future use)

## Rules
- Reserved keywords cannot be used as identifiers.
- `match` and `import` produce syntax errors in v0.1.

## Test Plan
- `let import = 1;` -> lexer error
- `import math` -> parser error `AT1000`
