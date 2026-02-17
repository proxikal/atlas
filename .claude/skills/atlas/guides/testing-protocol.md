# Testing Protocol (CRITICAL - Cost/Time/Disk Space)

**🚫 STOP WASTING TIME AND DISK SPACE 🚫**

**THE PROBLEM:** Running `cargo test -p atlas-runtime` compiles ALL 124 test files into separate binaries with debug symbols. Multiple runs = 20GB+ bloat. NEVER AGAIN.

**THE FIX:** Use `cargo nextest run` — installed, faster, smarter parallelism.

---

## GATE -1 Verification: ABSOLUTELY NO TESTS

- Use `ls`, `grep`, file structure checks ONLY
- NEVER run `cargo test` for verification
- Tests are for GATE 4 only, not verification

---

## During Implementation: SURGICAL TESTING ONLY

**The ONLY allowed test command:**
```bash
cargo test -p atlas-runtime test_exact_function_name -- --exact
```

### Step-by-Step Protocol

1. Write code
2. `cargo clean && cargo check -p atlas-runtime` (clean + verify compilation)
3. Write ONE test
4. Run ONLY that test: `cargo test -p atlas-runtime test_exact_name -- --exact`
5. **PASSES?** → STOP. Done. Move to next feature.
6. **FAILS?** → Fix code, re-run ONLY that test (iterate until passes)
7. **Once passing:** STOP. Don't re-run for "verification"

---

## CRITICAL: Fix-Iterate-Stop Pattern

- ✅ **DO** iterate on failing tests (fix → re-run same test → repeat until passes)
- 🚫 **DON'T** re-run passing tests for "verification" (waste of time/disk)
- ✅ **DO** ensure every test passes before moving on (quality matters!)
- 🚫 **DON'T** carry on with broken tests (never leave failures!)

---

## ABSOLUTE BANS (Cause 20GB Bloat)

- 🚫 `cargo test` - Everything (compiles 124 binaries)
- 🚫 `cargo test -p atlas-runtime` - 124 test files = 20GB+ bloat
- 🚫 `cargo test -p atlas-runtime test_try` - Multiple tests (matches pattern)
- 🚫 Any `cargo test` WITHOUT `-- --exact` during development
- 🚫 Re-running passing tests for "verification" (trust your fix!)
- 🚫 Background test runs (tests finish in seconds)
- 🚫 Running unrelated tests (only test what you changed)

## Allowed Full-Suite Commands (nextest only)

```bash
cargo nextest run -p atlas-runtime                    # All tests (fast, no network)
cargo nextest run -p atlas-runtime --test <file>      # Per-file validation
cargo nextest run -p atlas-runtime --run-ignored all  # Include network tests
```

**Network tests** (`http_core_tests`, `http_advanced_tests`) are `#[ignore = "requires network"]` — excluded from all normal runs automatically.

---

## GATE 4 ONLY - End of Phase

- User will tell you when to run full suite
- Until then: ONE TEST FUNCTION AT A TIME
- Trust your code, don't verify with 1400 tests

---

## Examples

### ✅ CORRECT Approach

```bash
# Write code for feature X
cargo clean && cargo check -p atlas-runtime  # Clean + verify it compiles
# Write test_feature_x
cargo test -p atlas-runtime test_feature_x -- --exact  # Run ONLY this test

# Scenario 1: Test PASSES
# → STOP. Move to next feature. Don't re-run.

# Scenario 2: Test FAILS
# → Fix the code
# → Re-run: cargo test -p atlas-runtime test_feature_x -- --exact
# → Still fails? Fix again, re-run again
# → Iterate until it passes
# → Once passing: STOP. Don't re-run for "verification"
```

### ❌ WRONG Approach

```bash
cargo test -p atlas-runtime --test result_tests  # NO! Too many tests
cargo test -p atlas-runtime test_try             # NO! Runs all test_try_* tests
cargo test -p atlas-runtime                       # HELL NO! Full suite
```

---

## Remember

- Each test run costs time and money
- Run ONE test at a time
- If 1 test passes, assume the other 7 similar tests will too
- Only run full suite when user explicitly asks
