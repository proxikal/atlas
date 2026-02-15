# GATE 4: Quality Gates

**Condition:** Implementation complete, parity verified

---

## 🚨 THIS IS THE ONLY TIME YOU RUN FULL TEST SUITE

**In previous gates (GATE 2), you ran TARGETED tests only.**
**Now in GATE 4, you run the FULL suite for the FIRST TIME.**

---

## Action

1. **Run all tests (FULL SUITE - first time):**
   ```bash
   cargo test -p atlas-runtime
   ```
   **MUST:** 100% pass rate

   **If tests fail:**
   - Fix the ONE failing test
   - Re-run ONLY that test: `cargo test -p atlas-runtime --test failing_test`
   - Don't re-run full suite unless you broke multiple things

2. **Run clippy:**
   ```bash
   cargo clippy -- -D warnings
   ```
   **MUST:** Zero warnings

3. **Run formatter check:**
   ```bash
   cargo fmt -- --check
   ```
   **MUST:** All files formatted

---

**BLOCKING:** All three must pass. No exceptions.

---

## Decision

- All pass → GATE 5
- Any fail → Fix → Retry
- Max 2 retry attempts → Escalate

---

**Next:** GATE 5
