# Atlas Decision Log

This log captures irreversible or high-impact design decisions. Update when a new foundational decision is made.

## Language Semantics
- Strict typing, no implicit any or nullable. (TypeScript-style)
- No truthy/falsey coercion; conditionals require `bool`.
- `number` is IEEE 754 f64, but invalid numeric results are runtime errors.
- `+` supports `number + number` and `string + string` only.
- Comparisons `< <= > >=` only for `number`.
- Equality requires same-type operands; strings compare by value, arrays by reference identity.
- Top-level statements execute in order.
- Function declarations are top-level only (v0.1) and hoisted.
- Variables must be declared before use (no forward reference).
- `break`/`continue` only inside loops; `return` only inside functions.

## Number Literals
- Format: `digit { digit } [ "." digit { digit } ] [ ("e"|"E") ["+" | "-"] digit { digit } ]`
- Supports: Integer (`123`), decimal (`3.14`), and scientific notation (`1e10`, `1.5e-3`, `2.5E+10`)
- Rationale: AI-friendliness. Scientific notation is far more readable and token-efficient than 300+ digit literals. As an AI-first language, this improves both human and AI code generation/understanding.
- Lexer validates: Exponent must have at least one digit (e.g., `1e` or `1e+` are errors)

## Runtime Model
- Single shared `Value` enum across interpreter and VM.
- Reference counting (`Rc/Arc`), no GC in v0.1.
- Strings immutable; arrays mutable and shared by reference.
- Function arguments are passed by value with shared references for heap types.

## Diagnostics
- Unified `Diagnostic` type with human + JSON formats.
- Errors emitted before warnings.
- Diagnostics include `diag_version` and optional related spans.

## Warning Implementation (Unused Variables/Code)
- TypeChecker maintains internal `declared_symbols` and `used_symbols` tracking per function.
- Rationale: Binder creates/destroys scopes during binding phase, before type checking runs. Scopes are gone by the time TypeChecker needs to check for unused symbols.
- Symbol table remains immutable during type checking; usage tracking is TypeChecker's responsibility.
- Symbol struct has no `used` field - keeps Symbol focused on type information, TypeChecker focused on analysis.
- Warnings emitted at end of each function scope, not globally.
- AI-friendly: Clear separation of concerns, no unused fields to cause confusion.

## Prelude
- Built-ins `print`, `len`, `str` always in scope.
- Global shadowing of prelude names is illegal (`AT1012`).

## Bytecode
- `.atb` format defined in `docs/bytecode-format.md`.
- Debug info is emitted by default.
