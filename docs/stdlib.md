# Atlas Stdlib (v0.1)

> **📋 Expansion Plan:** See `docs/stdlib-expansion-plan.md` for the roadmap beyond v0.1

## print
- Signature: `print(value: string|number|bool|null) -> void`
- Behavior:
  - Writes `value` to stdout.
  - `null` prints as `null`.

## len
- Signature: `len(value: string|T[]) -> number`
- Behavior:
  - Returns length of string or array.
  - String length is Unicode scalar count (not bytes).
  - Invalid input type is a runtime error `AT0102` (invalid stdlib argument).

## str
- Signature: `str(value: number|bool|null) -> string`
- Behavior:
  - Converts value to its string representation.
  - Invalid input type is a runtime error `AT0102` (invalid stdlib argument).

## Notes
- Stdlib functions are pure except `print`.
- All stdlib errors must include span info pointing to the callsite.
