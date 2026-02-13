# Runtime Error Span Tracking - Implementation Audit

**Date:** 2026-02-12
**Purpose:** Full audit of RuntimeError creation sites and span tracking implementation

---

## Executive Summary

- **Total RuntimeError creation sites:** 75
- **Span availability:** ✅ All AST nodes have `span` field, VM has `current_span()` method
- **Current state:** ❌ No spans attached to any RuntimeError
- **Phases that required this:** phase-08-runtime-errors, phase-14-debug-info (both marked complete incorrectly)

---

## Error Creation Sites Breakdown

### Interpreter (50 sites)

**interpreter/expr.rs (30 sites):**
- Binary operators: &&, ||, +, -, *, /, % (15 sites)
- Comparison operators (3 sites)
- Unary operators: -, ! (2 sites)
- Array/string indexing (6 sites)
- Function calls (4 sites)

**interpreter/stmt.rs (8 sites):**
- Compound assignment: +=, -=, *=, /=, %= (5 sites)
- Increment: ++ (2 sites)
- Decrement: -- (2 sites)

**interpreter/mod.rs (12 sites):**
- Variable lookup (1 site)
- Array index get (5 sites)
- Array index set (6 sites)

### VM (40 sites)

**vm/mod.rs:**
- Opcode validation (5 sites)
- Arithmetic operations (8 sites)
- Variable operations (4 sites)
- Comparison operations (4 sites)
- Unary operations (2 sites)
- Function calls (3 sites)
- Array operations (8 sites)
- Stack operations (6 sites)

### Stdlib (7 sites)

**stdlib.rs:**
- Argument count validation (3 sites)
- Type validation (3 sites)
- Unknown function (1 site)

---

## Span Availability Analysis

### Interpreter ✅

Every AST node has a `span` field:
```rust
pub struct BinaryExpr {
    pub left: Box<Expr>,
    pub op: BinaryOp,
    pub right: Box<Expr>,
    pub span: Span,  // ← Available
}

pub struct UnaryExpr {
    pub op: UnaryOp,
    pub operand: Box<Expr>,
    pub span: Span,  // ← Available
}

// ... all other Expr and Stmt variants have span
```

**Implementation approach:**
- Pass current Expr/Stmt to error creation
- Extract `.span` field when creating RuntimeError

### VM ✅

VM has span lookup methods:
```rust
impl VM {
    pub fn current_span(&self) -> Option<Span> {
        self.bytecode.get_span_for_offset(self.ip)
    }

    pub fn span_for_offset(&self, offset: usize) -> Option<Span> {
        self.bytecode.get_span_for_offset(offset)
    }
}
```

Bytecode has debug info:
```rust
pub struct Bytecode {
    pub instructions: Vec<u8>,
    pub constants: Vec<Value>,
    pub debug_info: Vec<DebugSpan>,  // ← Available
}
```

**Implementation approach:**
- Call `self.current_span()` when creating RuntimeError
- Use `Span::dummy()` as fallback only if debug_info is empty

### Stdlib ⚠️

Stdlib functions don't have direct access to source:
```rust
pub fn call_builtin(name: &str, args: &[Value]) -> Result<Value, RuntimeError>
```

**Implementation approach:**
- Add `span: Span` parameter to `call_builtin`
- Caller (interpreter/VM) passes call expression span
- Stdlib functions use this span for all errors

---

## Required Changes

### 1. value.rs - RuntimeError enum

**Current:**
```rust
pub enum RuntimeError {
    TypeError(String),
    UndefinedVariable(String),
    DivideByZero,
    // ...
}
```

**Required:**
```rust
pub enum RuntimeError {
    TypeError { msg: String, span: Span },
    UndefinedVariable { name: String, span: Span },
    DivideByZero { span: Span },
    OutOfBounds { span: Span },
    InvalidNumericResult { span: Span },
    InvalidStdlibArgument { span: Span },
    UnknownFunction { name: String, span: Span },
    InvalidIndex { span: Span },
    UnknownOpcode { span: Span },
    StackUnderflow { span: Span },
}
```

### 2. stdlib.rs - call_builtin signature

**Current:**
```rust
pub fn call_builtin(name: &str, args: &[Value]) -> Result<Value, RuntimeError>
```

**Required:**
```rust
pub fn call_builtin(name: &str, args: &[Value], call_span: Span) -> Result<Value, RuntimeError>
```

### 3. interpreter/expr.rs - Error creation

**Current (example):**
```rust
return Err(RuntimeError::DivideByZero);
```

**Required:**
```rust
return Err(RuntimeError::DivideByZero { span: binary.span });
```

### 4. vm/mod.rs - Error creation

**Current (example):**
```rust
return Err(RuntimeError::DivideByZero);
```

**Required:**
```rust
return Err(RuntimeError::DivideByZero {
    span: self.current_span().unwrap_or_else(Span::dummy)
});
```

### 5. runtime.rs - runtime_error_to_diagnostic

**Current:**
```rust
fn runtime_error_to_diagnostic(error: RuntimeError) -> Diagnostic {
    let (code, message) = match error {
        RuntimeError::DivideByZero => ("AT0005", "Division by zero".to_string()),
        // ...
    };
    Diagnostic::error_with_code(code, message, Span::dummy())  // ← WRONG
}
```

**Required:**
```rust
fn runtime_error_to_diagnostic(error: RuntimeError) -> Diagnostic {
    let (code, message, span) = match error {
        RuntimeError::DivideByZero { span } => {
            ("AT0005", "Division by zero".to_string(), span)
        },
        // ...
    };
    Diagnostic::error_with_code(code, message, span)  // ← CORRECT
}
```

---

## Implementation Order

1. **Modify RuntimeError enum** (value.rs) - Foundation
2. **Update runtime_error_to_diagnostic** (runtime.rs) - Extract spans
3. **Update call_builtin signature** (stdlib.rs) - Add span parameter
4. **Update all interpreter error sites** (interpreter/*.rs) - 50 sites
5. **Update all VM error sites** (vm/mod.rs) - 40 sites
6. **Update all stdlib error sites** (stdlib.rs) - 7 sites
7. **Update all test assertions** - Match new error format
8. **Add span verification tests** - Ensure spans are correct

---

## Testing Requirements

### Unit Tests
- Verify each error type includes correct span
- Test span points to exact error location (not parent expression)
- Test edge cases (empty debug_info, missing spans)

### Integration Tests
- Test divide by zero shows correct line/column
- Test array out of bounds shows index expression span
- Test stdlib errors show function call span
- Test type errors show operation span

### Parity Tests
- Verify interpreter and VM produce identical spans
- Same source code → same error spans (per docs/e2e-parity.md)

---

## Estimated Impact

- **Files to modify:** 8 files
- **Error sites to update:** 75 locations
- **Test files to update:** ~15 test files
- **New tests to add:** ~20 span verification tests
- **Compilation:** Will break until all 75 sites are updated (cannot be done incrementally)

---

## Exit Criteria (Per Phase Requirements)

**Phase 08 - Runtime Errors:**
- ✅ Runtime error mapping to diagnostic codes
- ✅ Attach span and call stack info ← **CURRENTLY MISSING**
- ✅ Stack trace formatting aligned with spec

**Phase 14 - Debug Info:**
- ✅ Span table and instruction span mapping ← **INFRASTRUCTURE EXISTS**
- ✅ Wire VM error reporting to span table ← **CURRENTLY MISSING**
- ✅ VM errors show accurate source spans ← **CURRENTLY MISSING**

**Phase 03 - Stdlib Doc Sync:**
- ✅ All stdlib errors must include span info pointing to callsite ← **CURRENTLY MISSING**

---

## Notes

- This cannot be done incrementally - changing RuntimeError breaks all 75 call sites
- Must be completed in single comprehensive change
- All tests will need updating to match new error format
- This is foundational for LSP and CLI error reporting
