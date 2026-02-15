# DR-002: Scientific Notation for Number Literals

**Date:** 2024-01-20
**Status:** Accepted
**Component:** Language Syntax

## Context
Number literal format needed definition - whether to support only basic integers/decimals or include scientific notation.

## Decision
Support scientific notation in number literals:
- Format: `digit { digit } [ "." digit { digit } ] [ ("e"|"E") ["+" | "-"] digit { digit } ]`
- Examples: `123`, `3.14`, `1e10`, `1.5e-3`, `2.5E+10`
- Lexer validates exponent has at least one digit (`1e` is error)

## Rationale
**AI-friendliness:** Scientific notation is far more readable and token-efficient than 300+ digit literals. For an AI-first language, this improves both human and AI code generation/understanding.

**Industry standard:** Every production language supports scientific notation (JavaScript, Python, Rust, Go, C#).

**Practical necessity:** Large numbers common in scientific computing, data analysis, AI/ML workloads.

## Alternatives Considered
- **Integer/decimal only:** Rejected - forces users to write `1000000000000` instead of `1e12`, poor developer experience
- **Underscore separators only:** Rejected - doesn't solve scale problem (`1_000_000_000_000` still verbose)
- **BigInt separate type:** Rejected - adds complexity, scientific notation simpler

## Consequences
- ✅ **Benefits:** Token-efficient for LLMs (fewer tokens for large numbers)
- ✅ **Benefits:** Readable code for humans and AI agents
- ✅ **Benefits:** Matches developer expectations from other languages
- ⚠️  **Trade-offs:** Slightly more complex lexer (already implemented)
- ❌ **Costs:** None significant

## Implementation Notes
- Lexer handles scientific notation in `scan_number()`
- Validation: Exponent must have digits after `e`/`E` (errors caught during lexing)
- Runtime: All numbers stored as IEEE 754 f64

## References
- Spec: `docs/specification/language-semantics.md` (Number Literals section)
- Related: DR-001 (Type System)
