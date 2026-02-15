# DR-004: Prelude with Shadowing Protection

**Date:** 2024-02-20
**Status:** Accepted
**Component:** Language - Prelude

## Context
Built-in functions (`print`, `len`, `str`) need to be available without imports, but shadowing behavior needed definition.

## Decision
Built-ins `print`, `len`, `str` always in scope (prelude).

**Global shadowing of prelude names is illegal** (`AT1012` diagnostic).

## Rationale
**Developer experience:** Core functions available immediately, no boilerplate imports.

**Safety:** Prevent accidental shadowing that breaks built-in behavior.

**AI-friendly:** Prelude is predictable - AI knows these names are always available and cannot be redefined.

## Alternatives Considered
- **Explicit imports:** Rejected - adds boilerplate, every file needs `import { print } from "prelude"`
- **Allow shadowing:** Rejected - confusing when built-ins stop working, hard to debug
- **No prelude, all explicit:** Rejected - poor developer experience for such common functions

## Consequences
- ✅ **Benefits:** Zero boilerplate for common functions
- ✅ **Benefits:** Prevents accidental shadowing bugs
- ✅ **Benefits:** AI knows prelude is always available
- ⚠️  **Trade-offs:** Cannot use `print` as variable name (acceptable - use `println` or other name)
- ❌ **Costs:** None significant

## Implementation Notes
**Prelude functions:**
- `print(value)` - Output to stdout
- `len(collection)` - Get length of string/array
- `str(value)` - Convert to string

**Diagnostic:** `AT1012` - "Cannot shadow prelude name 'X'"

**Enforcement:** Type checker validates no global declarations conflict with prelude.

## References
- Spec: `docs/specification/language-semantics.md` (Prelude section)
- Related: DR-001 (Type System)
