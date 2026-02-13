# Parser Error Recovery Policy

## Overview

Atlas uses a **minimal recovery** strategy that attempts to continue parsing after syntax errors to report multiple errors in a single pass. This provides better developer experience by showing all syntax issues at once rather than requiring multiple compile-fix cycles.

## Recovery Strategy

### Core Principle: Synchronize at Statement Boundaries

When the parser encounters an error, it synchronizes by advancing tokens until it finds a safe point to resume parsing. Safe points are:

1. **Statement boundaries**: After a semicolon (`;`)
2. **Statement keywords**: `fn`, `let`, `var`, `if`, `while`, `for`, `return`

This strategy balances error recovery with diagnostic quality:
- ✅ Reports multiple errors per parse
- ✅ Avoids cascading false errors
- ✅ Simple and predictable behavior

### Early-Exit vs Minimal Recovery

| Scenario | Strategy | Rationale |
|----------|----------|-----------|
| **Missing token** | Minimal recovery | Report error, synchronize, continue |
| **Mismatched brace** | Minimal recovery | Report error, synchronize, continue |
| **Invalid expression** | Minimal recovery | Report error, synchronize, continue |
| **End of file** | Early exit | Cannot recover, stop parsing |
| **Nested block errors** | Minimal recovery | Synchronize within block, continue outer context |

## Implementation Details

### Synchronization Algorithm

```rust
fn synchronize(&mut self) {
    self.advance();  // Move past error token

    while !self.is_at_end() {
        // Stop if we just passed a semicolon
        if self.tokens[self.current - 1].kind == TokenKind::Semicolon {
            return;
        }

        // Stop if we're at a statement keyword
        match self.peek().kind {
            TokenKind::Fn | TokenKind::Let | TokenKind::Var |
            TokenKind::If | TokenKind::While | TokenKind::For |
            TokenKind::Return => return,
            _ => { self.advance(); }
        }
    }
}
```

### Where Synchronization Occurs

1. **Top-level parsing** (`parse()` method)
   - When `parse_item()` fails
   - Synchronizes to next top-level item or statement

2. **Block parsing** (`parse_block()` method)
   - When `parse_statement()` fails within a block
   - Synchronizes to next statement within the block

## Specific Error Scenarios

### 1. Missing Semicolon

**Input:**
```atlas
let x = 42
let y = 10;
```

**Behavior:**
- Report error: "Expected ';' after variable declaration" at line 1
- Synchronize at `let` keyword
- Continue parsing line 2
- Result: One error, both declarations attempted

**Code Location:** `parser/mod.rs` (`parse_var_decl`)

### 2. Missing Closing Brace

**Input:**
```atlas
fn test() {
    let x = 42;
// Missing }
fn other() {
}
```

**Behavior:**
- Report error: "Expected '}'" at `fn` keyword
- Synchronize at `fn` keyword (statement boundary)
- Continue parsing `other()` function
- Result: One error for missing brace

**Code Location:** `parser/mod.rs` (`parse_block`)

### 3. Missing Closing Parenthesis

**Input:**
```atlas
if (x > 10 {
    let y = 5;
}
```

**Behavior:**
- Report error: "Expected ')' after if condition" at `{`
- Parser fails in `parse_if_stmt`, triggers synchronization
- Synchronize at block or next statement
- Result: Error reported, subsequent code may be skipped

**Code Location:** `parser/stmt.rs` (`parse_if_stmt`)

### 4. Invalid Expression

**Input:**
```atlas
let x = ;
let y = 42;
```

**Behavior:**
- Report error: "Expected expression" at `;`
- `parse_expression()` fails, propagates to `parse_var_decl`
- Synchronize at second `let` keyword
- Continue parsing line 2
- Result: One error, second declaration parsed

**Code Location:** `parser/expr.rs` (`parse_prefix`)

### 5. Mismatched Braces in Nested Blocks

**Input:**
```atlas
fn test() {
    if (true) {
        let x = 1;
    // Missing }
    let y = 2;
}
```

**Behavior:**
- Report error: "Expected '}'" in inner block
- Inner block synchronizes and continues
- Outer function may report secondary error
- Result: Reports primary error, attempts to continue

**Code Location:** `parser/mod.rs` (`parse_block`)

## Design Decisions

### Why Synchronize at Statement Boundaries?

1. **Predictable**: Easy for developers to understand where parser recovered
2. **Effective**: Most syntax errors occur within statements
3. **Clean separation**: Statement-level recovery prevents cross-statement confusion

### Why Not Expression-Level Recovery?

Expression-level recovery is complex and error-prone:
- Risk of misinterpreting subsequent code
- May produce confusing cascade errors
- Statement-level is simpler and more robust

### Why Not Panic Mode at Top Level Only?

Recovering only at top-level would:
- Report fewer errors per parse
- Require more compile cycles for developers
- Provide worse developer experience

## Testing Requirements

All error recovery scenarios must be tested to ensure:
1. Errors are reported with correct span information
2. Parser continues and reports subsequent errors
3. No crashes or infinite loops in recovery
4. Diagnostic quality remains high (no excessive cascading errors)

See `parser/mod.rs` test suite for comprehensive recovery tests.

## Future Enhancements

Potential improvements for later phases:

1. **Smarter brace matching**: Track brace depth to improve recovery
2. **Error productions**: Insert synthetic nodes for better AST quality
3. **Heuristic recovery**: Use indentation/structure hints
4. **Limit cascading errors**: Suppress likely false positives after recovery

These are explicitly **not** implemented in Phase 06 to maintain simplicity and predictability.

---

**Version:** 1.0
**Last Updated:** 2026-02-12
**Status:** Implemented in Phase 06
