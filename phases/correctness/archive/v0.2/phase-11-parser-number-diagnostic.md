# Phase Correctness-11: Parser Number Literal Diagnostic

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Correctness-10 complete. Build passes. Suite green.

**Verification:**
```bash
cargo check -p atlas-runtime 2>&1 | grep -c "error"  # must be 0
cargo nextest run -p atlas-runtime 2>&1 | tail -3
```

---

## Objective

Two small but real correctness bugs in the parser that violate compiler standards:

**Bug 1:** `parser/expr.rs:97` and `parser/expr.rs:612` use `token.lexeme.parse().unwrap_or(0.0)` for number parsing. If the lexer produces a number token whose lexeme can't be parsed as `f64`, the parser silently converts it to `0.0`. A professional compiler emits a diagnostic. This matters for edge cases like `999999999999999999999999999999999999999999` (overflows to infinity) or future lexer changes that might produce malformed number tokens.

**Bug 2:** All parser errors use the same error code `"AT1000"`. Professional compilers (rustc, clang, tsc) use distinct error codes so users can search documentation, suppress specific warnings, and tooling can match on codes. This phase assigns distinct codes to the most common parse error categories.

Both are small, focused fixes that close the gap to professional standards.

---

## Files Changed

- `crates/atlas-runtime/src/parser/expr.rs` — replace `unwrap_or(0.0)` with diagnostic-emitting parse
- `crates/atlas-runtime/src/parser/mod.rs` — define error code constants, use distinct codes

---

## Dependencies

- Correctness-10 complete
- No other phases are prerequisites

---

## Implementation

### Step 1: Fix number literal parsing

In `parser/expr.rs`, replace both instances of:
```rust
let value: f64 = token.lexeme.parse().unwrap_or(0.0);
```
with:
```rust
let value: f64 = match token.lexeme.parse() {
    Ok(v) => v,
    Err(_) => {
        self.diagnostics.push(Diagnostic::error(
            format!("Invalid number literal: '{}'", token.lexeme),
            token.span,
        ).with_code("AT1001"));
        0.0 // recovery value — parsing continues
    }
};
```

The parser still recovers (returns 0.0), but the user gets a diagnostic. This is the standard approach — rustc, tsc, and clang all emit diagnostics for unparseable literals and continue.

### Step 2: Define error code constants

In `parser/mod.rs`, add a constants block:
```rust
// Parser error codes
const E_GENERIC: &str = "AT1000";       // Generic/uncategorized parse error
const E_BAD_NUMBER: &str = "AT1001";    // Invalid number literal
const E_MISSING_SEMI: &str = "AT1002";  // Missing semicolon
const E_MISSING_BRACE: &str = "AT1003"; // Missing closing brace/bracket/paren
const E_UNEXPECTED: &str = "AT1004";    // Unexpected token
const E_RESERVED: &str = "AT1005";      // Reserved keyword used as identifier
```

### Step 3: Apply distinct codes to existing error sites

Search for all `"AT1000"` usages in the parser. Categorize each by the error it represents and replace with the appropriate constant. Focus on the most common patterns:

- Missing semicolons → `E_MISSING_SEMI`
- Missing `}`, `)`, `]` → `E_MISSING_BRACE`
- "Expected X, got Y" → `E_UNEXPECTED`
- Reserved keyword as identifier → `E_RESERVED`
- Everything else stays as `E_GENERIC` (to be refined in future phases)

### Step 4: Verify parity

Parser changes affect both interpreter and VM paths equally (they share the parser). Verify a program with a parse error produces the same diagnostic in both paths.

---

## Tests

- `test_invalid_number_literal_diagnostic` — extremely large number produces diagnostic, not silent 0.0
- `test_distinct_error_codes` — missing semicolon produces `AT1002`, unexpected token produces `AT1004`
- `test_reserved_keyword_error_code` — using `let` as identifier produces `AT1005`
- All existing parser tests pass unchanged
- All existing tests pass: `cargo nextest run -p atlas-runtime`
- Zero clippy warnings

---

## Acceptance

- Zero `unwrap_or(0.0)` for number parsing in `parser/expr.rs`
- Invalid number literals produce a diagnostic with code `AT1001`
- At least 5 distinct error codes used across the parser (AT1000–AT1005)
- All existing tests pass unchanged
- Zero clippy warnings: `cargo clippy -p atlas-runtime -- -D warnings`
- Commit: `fix(parser): Emit diagnostic for invalid number literals, add distinct error codes`
