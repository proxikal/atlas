# Phase XX: [Feature Name]

## Dependencies

**Required:** [List required phases/components]

**Verification:**
```bash
# Check required files exist
ls [critical files]

# Verify functionality
cargo test -p atlas-runtime [relevant_tests] -- --nocapture

# Clean build check
cargo clean && cargo check -p atlas-runtime
```

**If missing:** [What to do if dependencies missing]

---

## Objective

[1-2 sentences: What this phase delivers and why it matters]

---

## Files

**Create:** [new files with ~line count]
**Update:** [existing files with ~line count additions]
**Tests:** [test files with ~line count]

**Total new code:** ~XXX lines
**Total tests:** ~XXX lines (XX+ test cases)

---

## Dependencies (Components)

- [Component A] (existing/new)
- [Component B] (existing/new)
- [External dependency] (Rust crate if applicable)

---

## Implementation Notes

**Key patterns to analyze:**
- Examine [similar feature] implementation in [file path]
- Follow [pattern name] from memory/patterns.md
- Reference [DR-XXX] decision for [context]

**Critical requirements:**
- [Requirement 1]
- [Requirement 2]
- [Requirement 3]

**Error handling:**
- Error code AT#### for [scenario]
- Use RuntimeError::new(ErrorCode, msg)

**Integration points:**
- Uses: [existing component]
- Creates: [new component]
- Updates: [modified component]

---

## Tests (TDD Approach)

### Test Categories

**[Category 1]:** (X tests)
1. [Test case description]
2. [Test case description]
3. [Test case description]

**[Category 2]:** (X tests)
1. [Test case description]
2. [Test case description]

**[Category 3]:** (X tests)
1. [Test case description]
2. [Test case description]

**Minimum test count:** XX tests

**Parity requirement:** All tests run in both interpreter and VM with identical results.

**Test approach:**
- Use rstest for parameterized tests
- Use insta for snapshot tests where applicable
- Test edge cases, error scenarios, and integration

---

## Acceptance Criteria

- ✅ [Specific deliverable 1]
- ✅ [Specific deliverable 2]
- ✅ [Specific deliverable 3]
- ✅ [Feature X] works correctly
- ✅ [Feature Y] works correctly
- ✅ XX+ tests pass (specific count)
- ✅ 100% interpreter/VM parity verified
- ✅ Documentation complete in [location]
- ✅ No clippy warnings
- ✅ cargo test -p atlas-runtime passes
- ✅ Decision logs [DR-XXX, DR-YYY] referenced/created

---

## References

**Decision Logs:** DR-XXX ([decision topic])
**Specifications:** docs/specification/[relevant-spec].md
**Related phases:** [Previous phase], [Next phase]
