# GATE 3: Verify Parity (CRITICAL - 100% REQUIRED)

**Condition:** Implementation and tests complete

---

## ⚠️ PARITY IS NON-NEGOTIABLE ⚠️

**BLOCKING REQUIREMENT:** Interpreter and VM MUST have:
1. **IDENTICAL test count** - Every interpreter test MUST have a matching VM test
2. **IDENTICAL behavior** - Same outputs, errors, edge cases
3. **IDENTICAL coverage** - Same scenarios tested in both engines

**Failure to achieve 100% parity is a BLOCKING ISSUE.**

---

## Mandatory Verification Steps

### Step 1: Count Tests (MUST BE EQUAL)

Run this command to verify test counts:

```bash
echo "=== INTERPRETER TESTS ===" && \
grep "#\[test\]" crates/atlas-runtime/tests/stdlib_*.rs tests/vm_stdlib_*.rs 2>/dev/null | \
grep -v vm_stdlib | wc -l && \
echo "=== VM TESTS ===" && \
grep "#\[test\]" crates/atlas-runtime/tests/vm_stdlib_*.rs 2>/dev/null | wc -l
```

Or for specific phases:
```bash
# Count interpreter tests for current phase
grep -c "^#\[test\]" crates/atlas-runtime/tests/stdlib_PHASE_tests.rs

# Count VM tests for current phase
grep -c "^#\[test\]" crates/atlas-runtime/tests/vm_stdlib_PHASE_tests.rs
```

**REQUIREMENT:** Counts MUST be identical.

### Step 2: Run All Tests

```bash
# Run interpreter tests
cargo test --test stdlib_PHASE_tests --quiet

# Run VM tests
cargo test --test vm_stdlib_PHASE_tests --quiet
```

**REQUIREMENT:** Both MUST pass with 100% success rate.

### Step 3: Verify Parity

For EVERY test scenario in interpreter tests, there MUST be:
- ✅ Matching VM test with identical setup
- ✅ Identical assertions and expectations
- ✅ Identical error handling checks
- ✅ Identical edge case coverage

**Common parity violations to avoid:**
- ❌ More interpreter tests than VM tests
- ❌ Testing different scenarios between engines
- ❌ Different assertion levels (e.g., basic in VM, comprehensive in interpreter)
- ❌ Missing error case tests in one engine
- ❌ Missing permission tests in one engine

---

## Parity Test Requirements

**Every phase MUST have:**
1. **Dedicated test files:**
   - Interpreter: `tests/stdlib_PHASE_tests.rs` or `tests/stdlib/PHASE_*.rs`
   - VM: `tests/vm_stdlib_PHASE_tests.rs` or `tests/vm/PHASE_*.rs`

2. **Matching test structure:**
   ```rust
   // Interpreter test
   #[test]
   fn test_function_basic() {
       let runtime = Atlas::new_with_security(...);
       let result = runtime.eval("code");
       assert!(result.is_ok());
   }

   // VM test (MUST EXIST)
   #[test]
   fn vm_test_function_basic() {
       let result = execute_with_vm("code", &temp_dir);
       assert!(result.is_ok());
   }
   ```

3. **Comprehensive coverage in BOTH:**
   - Basic functionality tests
   - Edge case tests
   - Error condition tests
   - Permission/security tests
   - Type validation tests
   - Boundary tests

---

## Verification Checklist

**Before proceeding to GATE 4, verify:**

- [ ] Test count is IDENTICAL (interpreter == VM)
- [ ] All interpreter tests pass
- [ ] All VM tests pass
- [ ] Every interpreter test has matching VM test
- [ ] Every VM test has matching interpreter test
- [ ] Same test scenarios covered in both engines
- [ ] Same assertions used in both engines
- [ ] Same error cases tested in both engines
- [ ] Edge cases tested in both engines

**If ANY checkbox is unchecked → FIX IMMEDIATELY → RETRY GATE 3**

---

## Decision

- ✅ All checks pass → GATE 4
- ❌ ANY check fails → **BLOCKING** → Fix parity gaps → Retry GATE 3
- ❓ N/A (no parity needed) → **IMPOSSIBLE** - parity is ALWAYS required

---

## Parity Enforcement

**This requirement is ABSOLUTE:**
- No exceptions
- No "close enough"
- No "will fix later"
- No "VM tests coming in next phase"

**100% parity or the phase is INCOMPLETE.**

---

**Next:** GATE 4 (Quality Gates)
