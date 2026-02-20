# GATE 6: Final Testing

**Condition:** Implementation complete, ready for PR

**Source of truth:** auto-memory `testing-patterns.md` for crate-specific testing protocols

---

## Action

Run the full test suite for the package you modified:

```bash
cargo nextest run -p <package>  # e.g., atlas-lsp, atlas-runtime, atlas-cli
```

---

## Test Failure Triage

**Not all failures are equal.** Use this decision tree:

### Blocking Failures (MUST FIX)

Stop immediately and fix these:

- ❌ **Wrong results** - Code produces incorrect output
- ❌ **Panics/crashes** - Unexpected panics or segfaults
- ❌ **Data corruption** - Tests show data is corrupted
- ❌ **Security issues** - Security-related test failures
- ❌ **Parity breaks** - Interpreter and VM produce different results (for runtime code)
- ❌ **> 5% failure rate** - More than 5% of tests failing

**Action:** Fix the code, don't proceed until green.

### Non-Blocking Failures (NOTE IN PR)

You may proceed, but document in PR:

- ⚠️ **Flaky tests** - Test passes sometimes, fails others (< 5% failure rate)
- ⚠️ **Overly strict assertions** - Test assumes something that's not actually required
- ⚠️ **Cosmetic issues** - Formatting, whitespace, non-functional differences
- ⚠️ **Edge case assertions** - Feature works, but test is too strict about edge cases

**Example PR note:**
```markdown
## Known Issues
- 4/277 tests fail on edge case assertions (code actions, inlay hints)
- Features work correctly, test assertions are too strict
- Will address in follow-up
```

---

## Decision Matrix

| Pass Rate | Blocking Failures | Non-Blocking Failures | Action |
|-----------|-------------------|----------------------|--------|
| 100% | 0 | 0 | ✅ Proceed |
| 95-99% | 0 | Some | ✅ Proceed, note in PR |
| 95-99% | Any | Any | ❌ Fix blocking, then proceed |
| < 95% | Any | Any | ❌ Fix failures |
| < 95% | 0 | Many | ⚠️ Investigate, likely real issue |

---

## Examples

### ✅ Good: Proceed

```
277 tests: 273 passed, 4 failed
Failures: test_code_actions (assertion too strict)
```

**Analysis:** 98.6% pass rate, non-blocking failures, feature works.
**Action:** Proceed, note in PR.

### ❌ Bad: Stop and Fix

```
100 tests: 90 passed, 10 failed
Failures: test_calculation (wrong result: expected 42, got 43)
```

**Analysis:** 90% pass rate, wrong results = blocking.
**Action:** Fix the bug.

### ⚠️ Investigate

```
500 tests: 450 passed, 50 failed
All failures: similar assertion errors
```

**Analysis:** 90% pass rate, but no wrong results. Might be test quality issue.
**Action:** Investigate 2-3 failures, determine if blocking or test issue.

---

## Quality Checks (Also Run)

In addition to tests, run:

```bash
# Clippy (linting)
cargo clippy -p <package> -- -D warnings

# Formatting
cargo fmt --check -p <package>
```

**These must pass with 0 warnings/errors.**

---

## Pragmatic Workflow

**If you encounter test failures:**

1. **First 2 failures:** Investigate immediately
2. **Understand the pattern:** Are they all similar? Related to one feature?
3. **Classify:** Blocking or non-blocking?
4. **Decide:**
   - Blocking → Fix now
   - Non-blocking → Note in PR, proceed
5. **Time limit:** Don't spend > 15 minutes debugging flaky tests

**Remember:** The goal is working code, not perfect tests. If the feature works and tests are just overly strict, that's a test improvement task, not a blocker.

---

## What to Report in PR

Always include test results in PR body:

**Good example:**
```markdown
## Testing

cargo nextest run -p atlas-lsp: 273/277 passed (98.6%)

**Known Issues:**
- 4 integration tests fail on edge cases (strict assertions)
- Features verified working correctly
- Tests need assertion fixes (non-blocking)

All core LSP features validated ✅
```

**Bad example:**
```markdown
Tests mostly pass.
```

---

**Next:** GATE 7 (Memory Check)
