# DR-001: Strict Type System with No Implicit Coercion

**Date:** 2024-01-15
**Status:** Accepted
**Component:** Language Core

## Context
Atlas needed to define its core type system philosophy - whether to follow dynamic/loose typing (JavaScript, Python) or strict typing (TypeScript strict mode, Rust).

## Decision
Strict typing with zero implicit coercion:
- No implicit `any` or nullable types
- No truthy/falsy coercion - conditionals require `bool`
- `+` supports only `number + number` and `string + string`
- Comparisons `< <= > >=` only for `number`
- Equality requires same-type operands
- Strings compare by value, arrays by reference identity

## Rationale
**AI-first principle:** Explicit behavior reduces ambiguity for AI code generation. LLMs can reason precisely about type behavior without guessing coercion rules.

**Developer ergonomics:** Catches errors at compile time rather than runtime. No surprising type conversions.

**Industry precedent:** Follows TypeScript strict mode, Rust, Go approach - proven for large-scale systems.

## Alternatives Considered
- **JavaScript-style coercion:** Rejected - unpredictable behavior (`[] + []`, `{} + []`), AI agents struggle with edge cases
- **Python-style duck typing:** Rejected - requires runtime type checks, slower execution, harder for AI to verify correctness
- **Partial coercion (numbers only):** Rejected - inconsistent mental model, still ambiguous for AI

## Consequences
- ✅ **Benefits:** Predictable compilation, AI can verify type correctness statically, faster execution (no runtime checks)
- ✅ **Benefits:** Clear error messages at compile time, not runtime surprises
- ⚠️  **Trade-offs:** More explicit code required (e.g., `str(num)` instead of implicit conversion)
- ❌ **Costs:** Slightly more verbose than dynamic languages for prototyping

## Implementation Notes
- Type checker enforces strict rules in `check_binary_op()` and `check_expr()`
- Diagnostic codes: `AT2008` (type mismatch), `AT2010` (invalid operand types)
- Both interpreter and VM enforce identical type rules

## References
- Spec: `docs/specification/language-semantics.md`
- Philosophy: `docs/philosophy/why-strict.md`
- Related: DR-003 (Diagnostic Format)
