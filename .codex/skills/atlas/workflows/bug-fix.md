# Bug Fix Workflow

**When to use:** Fixing reported bugs, incorrect behavior

**Approach:** TDD (test first, then fix)

---

## Gates Used

| Gate | Action |
|------|--------|
| 0 | Read docs + check dependencies |
| 1 | Foundation check (existing code before fix) |
| 2 | **TDD: Write FAILING test FIRST**, then fix |
| 3 | Verify parity (both engines correct) |
| 4 | Quality gates + full test suite (no regressions) |
| 5 | Doc update if needed |

**Key difference from structured dev:** GATE 2 uses strict TDD — test MUST fail before fix.

---

## TDD at GATE 2

1. **Write failing test** (RED) — proves bug exists, test both engines if applicable
2. **Verify test fails** — BLOCKING before proceeding
3. **Locate root cause** — understand WHY, not just WHERE
4. **Minimal fix** — fix root cause, don't over-engineer, don't refactor unrelated code
5. **Verify test passes** (GREEN)
6. **Run full suite** — ensure no regressions
