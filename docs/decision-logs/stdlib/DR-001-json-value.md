# DR-001: JsonValue - Controlled Dynamic Typing Exception

**Date:** 2024-10-15
**Status:** Accepted
**Component:** Standard Library

## Context
Atlas needs JSON support for AI agent workflows (APIs, config files, data interchange), but strict typing principle prohibits dynamic types.

## Decision
`JsonValue` is the **only** exception to "no dynamic types" principle:
- Follows Rust's `serde_json` pattern (ergonomic + type-safe)
- Natural indexing: `data["user"]["name"]` (like Python/JS)
- Explicit extraction: `.as_string()`, `.as_number()`, etc. (type safety)
- Returns `JsonValue::Null` for missing keys/indices (safe, no crashes)
- **Isolated** from regular type system - cannot be assigned to non-JsonValue variables without extraction

## Rationale
**AI-first necessity:** JSON is critical for AI agent workflows. Delaying this feature harms adoption.

**Controlled exception:** Isolation via type system prevents dynamic typing from leaking. Only `JsonValue <-> JsonValue` assignments allowed.

**Industry precedent:** Rust's `serde_json::Value` proves this pattern works at scale.

## Alternatives Considered
- **General-purpose `any` type:** Rejected - violates strict typing principle globally
- **Wait for union types:** Rejected - union types complex, delays critical feature for years
- **Schema-based only:** Rejected - too rigid for dynamic APIs, bad for AI agent flexibility

## Consequences
- ✅ **Benefits:** AI agents can work with JSON APIs immediately
- ✅ **Benefits:** Ergonomic API (natural indexing) with type safety (explicit extraction)
- ✅ **Benefits:** Strict isolation prevents dynamic typing from spreading
- ⚠️  **Trade-offs:** Accept controlled dynamic typing for JSON only
- ❌ **Costs:** Runtime type checks on extraction (minimal overhead)

## Implementation Notes
**v0.2 implementation:**
- `JsonValue` enum: 6 variants (Null, Bool, Number, String, Array, Object)
- `Value::JsonValue(Rc<JsonValue>)` variant
- `Type::JsonValue` in type system
- Isolation enforced via `Type::is_assignable_to()` - only json→json allowed
- Safe indexing: `index_str()` and `index_num()` return `JsonValue::Null` for invalid access
- Both interpreter and VM support `json[string|number]` indexing
- 21 integration tests verify behavior and isolation

## References
- Spec: `docs/specification/language-semantics.md` (JSON section)
- API: `docs/api/stdlib.md` (JSON module)
- Related: DR-001 (Type System), DR-003 (Method Syntax)
