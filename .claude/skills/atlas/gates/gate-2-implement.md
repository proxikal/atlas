# GATE 2: Implement + Test

**Condition:** Size estimated, plan ready

**⚠️ TESTING REMINDER:** Run TARGETED tests only. Full suite is GATE 4. See protocol below.

**IMPORTANT:** This gate differs based on workflow type.

---

## For Features (Structured Development, Enhancements)

**Approach:** Implementation-driven with comprehensive testing

**Action:**
1. **Implement feature** following Atlas standards:
   - Explicit types
   - Result<T, E> error handling
   - No unwrap() in production code
   - Clear naming
   - No TODOs or stubs

2. **Write tests alongside or after implementation**:
   - Basic functionality
   - Edge cases
   - Error handling
   - Both interpreter AND VM (if applicable)

3. **Iterate:** Implement → test → refine

**Why:** Compilers require exploratory implementation. You discover edge cases WHILE building. This mirrors rustc, Go compiler, TypeScript, Clang.

**Tests can come:**
- Alongside implementation (recommended)
- After implementation (acceptable)
- NOT required before implementation (unlike bugs)

---

## 🚨 TESTING PROTOCOL (READ BEFORE RUNNING TESTS)

**CRITICAL:** Do NOT run full test suite in GATE 2. That's GATE 4 only.

**In GATE 2, run TARGETED tests ONLY:**

```bash
# If you wrote new test files:
cargo test -p atlas-runtime --test your_new_test_file

# If you modified existing functionality:
cargo test -p atlas-runtime --test affected_test_file

# NEVER run:
cargo test -p atlas-runtime  # ❌ This is GATE 4 only
cargo test                    # ❌ This is GATE 4 only
```

**Run tests ONCE:**
- Write test → run ONCE → if it passes, STOP
- Don't "verify" by running again
- Don't run full suite to "make sure nothing broke"
- Trust your targeted tests

**Do NOT use background mode:**
- Tests finish in 1-5 seconds
- Just run them normally and wait
- Background mode is for operations >30 seconds

**Full test suite is GATE 4 ONLY** (one time at the end)

---

## For Bugs (Bug Fix Workflow ONLY)

**Approach:** Strict TDD (Test-Driven Development)

**Action:**
1. **Write failing test FIRST** (before any fix)
2. **Verify test fails** (RED phase - proves bug exists)
3. **Fix implementation** (minimal change to fix root cause)
4. **Verify test passes** (GREEN phase - bug fixed)

**BLOCKING:** For bugs, test MUST be written and failing before fix.

---

## Test Framework

```rust
use rstest::rstest;

#[rstest]
#[case("input", "expected")]
fn test_feature(#[case] input: &str, #[case] expected: &str) {
    // Test implementation
}
```

---

**Next:** GATE 3
