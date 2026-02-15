# DR-003: Method Call Syntax - Rust-Style Desugaring

**Date:** 2024-11-20
**Status:** Accepted
**Component:** Language Syntax

## Context
Atlas needed method call syntax for ergonomic API usage (e.g., `json.as_string()`, `array.map(fn)`).

## Decision
**Syntax:** `value.method(args)` desugars to `Type::method(value, args)`

**Design:** Methods are functions with special syntax. No runtime lookup, no prototype chains, no `this` binding complexity.

**Dual syntax:** Both `value.method()` and `Type::method(value)` valid - AI can use either form.

## Rationale
**AI-friendly:** Zero ambiguity - compile-time resolution, no runtime magic. AI knows exactly what code executes.

**Type-safe:** Methods resolved during type checking, not runtime lookup. Errors caught at compile time.

**Zero-cost abstraction:** Method syntax is pure syntactic sugar - compiles to direct function calls.

**Rust precedent:** Proven approach in production systems.

## Alternatives Considered
- **Python-style (everything-is-object):** Rejected - adds magic, runtime overhead, implicit behavior confuses AI
- **JavaScript prototype chains:** Rejected - `this` binding issues, runtime lookup complexity, not AI-friendly
- **Go interfaces:** Rejected - implicit satisfaction not AI-friendly (prefer explicit)

## Consequences
- ✅ **Benefits:** Ergonomic syntax without runtime cost
- ✅ **Benefits:** AI can reason about method calls statically
- ✅ **Benefits:** Dual syntax flexibility (both forms work)
- ✅ **Benefits:** Compile-time type safety
- ⚠️  **Trade-offs:** No dynamic dispatch (not needed in v0.2, traits later)
- ❌ **Costs:** None significant

## Implementation Notes
**v0.2 implementation:**
- Built-in methods for stdlib types: `json`, `string`, `array`
- Type checker resolves method calls during `check_method_call()`
- Both engines desugar to function calls
- Trait system for user-defined methods in v0.3+

**Examples:**
```atlas
// These are identical:
json.as_string()
JsonValue::as_string(json)

// AI can generate either form
arr.map(fn)
Array::map(arr, fn)
```

## References
- Spec: `docs/specification/language-semantics.md` (Method Calls section)
- Related: DR-001 (Type System)
