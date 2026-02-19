# GATE 2: Implement + Test

**Condition:** Size estimated, plan ready

**⚠️ TESTING REMINDER:** Run TARGETED tests only. Full suite is GATE 6. See `memory/testing-patterns.md`.

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

## 🚨 TESTING PROTOCOL

**Do NOT run full test suite in GATE 2. That's GATE 6 only.**

```bash
# Single test (during dev)
cargo nextest run -p atlas-runtime -E 'test(exact_name)'

# Domain file (validate your work area)
cargo nextest run -p atlas-runtime --test <domain_file>

# Full suite — GATE 6 ONLY
cargo nextest run -p atlas-runtime
```

**Complete rules:** See `memory/testing-patterns.md` for domain file list, corpus workflow, parity helpers, and `#[ignore]` rules.

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
