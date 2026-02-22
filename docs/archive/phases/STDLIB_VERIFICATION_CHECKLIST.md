# Stdlib Implementation Verification Checklist

**Phase:** stdlib/phase-03-stdlib-doc-sync.md
**Date:** 2026-02-12
**Status:** In Progress

## Summary

This checklist verifies that `crates/atlas-runtime/src/stdlib.rs` matches the specification in `docs/stdlib.md` and `Atlas-SPEC.md`.

---

## Function: print

### Documentation
- **Signature:** `print(value: string|number|bool|null) -> void`
- **Behavior:** Writes value to stdout. `null` prints as `null`.
- **Source:** `docs/stdlib.md` line 3-7, `Atlas-SPEC.md`

### Implementation Audit
- **File:** `crates/atlas-runtime/src/stdlib.rs` lines 38-41
- **Current Signature:** Accepts any `Value` type
- **Status:** ❌ **MISMATCH** - Implementation is more permissive than spec

### Issues Found
1. **Type Checking:** Function accepts ANY Value type (including arrays, functions, etc.)
   - **Expected:** Should only accept `string|number|bool|null`
   - **Required Fix:** Add type validation, return `InvalidStdlibArgument` for arrays and other types

### Test Coverage
- ✅ Test with string (line 137-140 in stdlib.rs)
- ✅ Test with wrong argument count (line 157-161 in stdlib.rs)
- ❌ **MISSING:** Test that arrays are rejected
- ❌ **MISSING:** Test that each valid type (number, bool, null) works

### Required Tests
```rust
#[test]
fn test_print_rejects_array() {
    let result = call_builtin("print", &[Value::array(vec![Value::Number(1.0)])]);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), RuntimeError::InvalidStdlibArgument));
}

#[test]
fn test_print_accepts_all_valid_types() {
    assert!(call_builtin("print", &[Value::string("test")]).is_ok());
    assert!(call_builtin("print", &[Value::Number(42.0)]).is_ok());
    assert!(call_builtin("print", &[Value::Bool(true)]).is_ok());
    assert!(call_builtin("print", &[Value::Null]).is_ok());
}
```

---

## Function: len

### Documentation
- **Signature:** `len(value: string|T[]) -> number`
- **Behavior:**
  - Returns length of string or array
  - String length is Unicode scalar count (not bytes)
  - Invalid input type is runtime error AT0102 (invalid stdlib argument)
- **Source:** `docs/stdlib.md` lines 9-14, `Atlas-SPEC.md`

### Implementation Audit
- **File:** `crates/atlas-runtime/src/stdlib.rs` lines 43-53
- **Current Behavior:**
  - Accepts String and Array
  - Returns Unicode scalar count for strings (line 49: `s.chars().count()`)
  - Returns element count for arrays
  - Returns `InvalidStdlibArgument` for other types
- **Status:** ✅ **MATCH** - Implementation matches specification

### Test Coverage
- ✅ Test with string (line 65-68)
- ✅ Test with array (line 71-74)
- ✅ Test with invalid type (line 111-115)
- ✅ Test Unicode scalar count (line 83-96)
- ✅ Test empty string (line 99-102)
- ✅ Test empty array (line 105-108)
- ✅ Integration test with string (interpreter_tests.rs:1426-1437)
- ✅ Integration test with array (interpreter_tests.rs:1440-1451)

### Verification
**All requirements satisfied** ✅

---

## Function: str

### Documentation
- **Signature:** `str(value: number|bool|null) -> string`
- **Behavior:**
  - Converts value to its string representation
  - Invalid input type is runtime error AT0102 (invalid stdlib argument)
- **Source:** `docs/stdlib.md` lines 16-20, `Atlas-SPEC.md`

### Implementation Audit
- **File:** `crates/atlas-runtime/src/stdlib.rs` lines 55-58
- **Current Signature:** Accepts any `Value` type
- **Status:** ❌ **MISMATCH** - Implementation is more permissive than spec

### Issues Found
1. **Type Checking:** Function accepts ANY Value type (including strings, arrays, etc.)
   - **Expected:** Should only accept `number|bool|null`
   - **Required Fix:** Add type validation, return `InvalidStdlibArgument` for strings, arrays, and other types

### Test Coverage
- ✅ Test with number (line 118-122, also line 77-80)
- ✅ Test with bool (line 125-128)
- ✅ Test with null (line 131-133)
- ✅ Integration test (interpreter_tests.rs:1454-1465)
- ❌ **MISSING:** Test that strings are rejected
- ❌ **MISSING:** Test that arrays are rejected

### Required Tests
```rust
#[test]
fn test_str_rejects_string() {
    let result = call_builtin("str", &[Value::string("already a string")]);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), RuntimeError::InvalidStdlibArgument));
}

#[test]
fn test_str_rejects_array() {
    let result = call_builtin("str", &[Value::array(vec![Value::Number(1.0)])]);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), RuntimeError::InvalidStdlibArgument));
}
```

---

## Cross-Cutting Concerns

### Error Code Mapping
- **Documentation:** Uses error code `AT0102` for invalid stdlib argument
- **Implementation:** Uses `RuntimeError::InvalidStdlibArgument`
- **Status:** ⚠️ **NEEDS VERIFICATION** - Need to confirm this maps to AT0102

**Verification Required:**
```bash
# Check error code mapping in value.rs or runtime error handling
grep -n "AT0102\|InvalidStdlibArgument" crates/atlas-runtime/src/value.rs
```

### Span Information
- **Documentation:** "All stdlib errors must include span info pointing to the callsite"
- **Implementation:** `stdlib.rs` returns `RuntimeError` without span info
- **Status:** ❌ **ARCHITECTURAL GAP** - Not currently implemented

**Investigation Results:**
- ✅ Bytecode HAS debug info: `bytecode/mod.rs:53` - `debug_info: Vec<DebugSpan>`
- ❌ VM doesn't use debug_info when reporting errors
- ❌ Interpreter doesn't track spans during execution
- ❌ All RuntimeErrors converted to Diagnostics with `Span::dummy()` (`runtime.rs:185`)

**Root Cause:**
This is a broader interpreter/VM architecture limitation, not a stdlib-specific issue. The stdlib functions correctly return errors - the calling context (interpreter/VM) should add span information but currently doesn't.

**Scope:**
This affects ALL runtime errors (divide by zero, out of bounds, null errors, etc.), not just stdlib errors. Fixing this requires:
1. Interpreter to track current execution span
2. VM to use bytecode debug_info for error reporting
3. RuntimeError to carry Span information
4. OR alternative architecture for propagating span info

**Recommendation:**
This should be addressed in a dedicated phase (e.g., "Runtime Error Span Tracking") after CLI/LSP phases, as it's a cross-cutting concern that affects the entire runtime error reporting system.

**For This Phase:**
✅ Documentation is CORRECT in stating the requirement
✅ Stdlib implementation is CORRECT in returning errors
❌ System doesn't yet meet the documented requirement (known gap)

### Purity Guarantees
- **Documentation:** "Stdlib functions are pure except `print`"
- **Implementation:**
  - `print` has side effect (stdout) ✅
  - `len` is pure (no side effects) ✅
  - `str` is pure (no side effects) ✅
- **Status:** ✅ **MATCH**

---

## Action Items

### High Priority (Breaks Spec Compliance)
1. ❌ **Fix `print()` type checking** - Add validation for string|number|bool|null only
2. ❌ **Fix `str()` type checking** - Add validation for number|bool|null only
3. ❌ **Add missing tests** - Test invalid type rejection for print() and str()

### Medium Priority (Documentation Verification)
4. ⚠️ **Verify error code mapping** - Confirm InvalidStdlibArgument = AT0102
5. ⚠️ **Verify span information** - Confirm errors include source location

### Low Priority (Already Compliant)
6. ✅ **len() implementation** - Already matches specification

---

## Test Mapping (Deliverable)

This section maps each documented behavior to its corresponding test(s).

### print(value: string|number|bool|null) -> void

| Behavior | Test(s) | Status |
|----------|---------|--------|
| Accepts string | `test_call_builtin_print` (line 136) | ✅ |
| Accepts number | **MISSING** | ❌ |
| Accepts bool | **MISSING** | ❌ |
| Accepts null | **MISSING** | ❌ |
| Rejects array | **MISSING** | ❌ |
| Rejects other types | **MISSING** | ❌ |
| Returns void (Null) | `test_call_builtin_print` (line 139) | ✅ |
| Prints to stdout | Manual verification needed | ⚠️ |
| null prints as "null" | **MISSING** | ❌ |

### len(value: string|T[]) -> number

| Behavior | Test(s) | Status |
|----------|---------|--------|
| Returns string length | `test_len_string`, `test_stdlib_len_string` | ✅ |
| Returns array length | `test_len_array`, `test_stdlib_len_array` | ✅ |
| Unicode scalar count | `test_len_unicode_string` (comprehensive) | ✅ |
| Empty string returns 0 | `test_len_empty_string` | ✅ |
| Empty array returns 0 | `test_len_empty_array` | ✅ |
| Rejects number | `test_len_invalid_type` | ✅ |
| Rejects bool | Covered by invalid_type test | ✅ |
| Rejects null | Covered by invalid_type test | ✅ |
| Error is InvalidStdlibArgument | `test_len_invalid_type` (line 114) | ✅ |

### str(value: number|bool|null) -> string

| Behavior | Test(s) | Status |
|----------|---------|--------|
| Converts number | `test_str_number`, `test_call_builtin_str` | ✅ |
| Converts bool | `test_str_bool` | ✅ |
| Converts null | `test_str_null` | ✅ |
| Number formatting | `test_str_number` (multiple cases) | ✅ |
| Rejects string | **MISSING** | ❌ |
| Rejects array | **MISSING** | ❌ |
| Error is InvalidStdlibArgument | **MISSING** | ❌ |

---

## Summary Statistics

- **Functions Audited:** 3
- **Fully Compliant:** 1 (len)
- **Type Check Issues:** 2 (print, str)
- **Missing Tests:** 9
- **Tests Passing:** All current tests pass ✅
- **Spec Compliance:** ❌ **BLOCKED** - Type checking violations

---

## Next Steps

1. Update `print()` implementation to add type checking
2. Update `str()` implementation to add type checking
3. Add missing tests for invalid type rejection
4. Add missing tests for valid type acceptance
5. Verify error code mapping (AT0102)
6. Verify span information in error messages
7. Run full test suite
8. Mark phase complete when all items are ✅

---

**Phase Exit Criteria:**
- ✅ Docs and implementation match with no gaps
- ✅ Tests map 1:1 to documented behaviors
- ❌ **Current Status:** NOT MET - Implementation has type checking gaps
