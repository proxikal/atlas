# Refactoring Workflow

**When to use:** Code cleanup, optimization, restructuring WITHOUT behavior changes

**Key principle:** Existing tests must still pass (no behavior changes)

**Reference:** See `gates/README.md` for GATE 0 and GATE 5 definitions

---

## Workflow

Refactoring uses **GATE 0 and GATE 5** from central gate workflow, plus refactoring-specific steps.

| Step | Action | Reference |
|------|--------|-----------|
| 0 | **Read Docs** | See gates/gate-0-read-docs.md |
| 1 | **Verify Tests Pass** | Baseline - all tests pass before refactoring |
| 2 | **Identify Goal** | What are you improving? Why? |
| 3 | **Small Incremental Changes** | Refactor in small, verifiable steps |
| 4 | **Run Tests After Each Change** | Catch breaks immediately |
| 5 | **Quality Gates** | See gates/gate-5-docs.md |
| 6 | **Verify No Behavior Changes** | Tests pass identically |

---

## Step 1: Verify Tests Pass

**Before starting:**
```bash
cargo nextest run -p atlas-runtime
```

**All tests must pass.** This is your baseline.

**If tests fail before refactoring:** Fix them first. Can't refactor broken code.

---

## Step 2: Identify Refactoring Goal

**Common goals:**
- **Extract function** - Break large function into smaller focused functions
- **Rename for clarity** - Better variable/function names
- **Reorganize module** - Better file structure
- **Remove duplication** - DRY principle
- **Simplify logic** - Clearer control flow
- **Improve performance** - Optimize hot paths (measure first!)

**Be specific:** "Refactor parser.rs" is vague. "Extract expression parsing into separate functions" is clear.

---

## Step 3: Small Incremental Changes

**Refactor in small steps:**

**Bad:** Rewrite entire module at once
**Good:** Extract one function, test, commit. Repeat.

**Example sequence:**
1. Extract helper function `parse_primary` from `parse_expression`
2. Run tests → pass
3. Extract `parse_binary_op`
4. Run tests → pass
5. Rename `tmp_result` to `expression_node`
6. Run tests → pass
7. Commit

**Each step is small and verifiable.**

---

## Step 4: Run Tests After Each Change

**After EVERY change:**
```bash
cargo nextest run -p atlas-runtime
```

**Tests should pass at each step.**

**If tests fail:** You broke something. Undo or fix immediately. Don't stack changes on broken code.

---

## Step 5: Quality Gates

**After refactoring, run GATE 5 checks:**
```bash
cargo nextest run -p atlas-runtime           # 100% pass (same as before)
cargo clippy         # Zero warnings
cargo fmt -- --check # Formatted
```

**All quality gates must pass.** See `gates/gate-5-docs.md` for details.

---

## Step 6: Verify No Behavior Changes

**Critical check:**
- All existing tests still pass
- Same test coverage as before
- No new warnings
- No performance regressions (if applicable)
- Parity maintained (if interpreter/VM involved)

**Refactoring changes HOW code works, not WHAT it does.**

---

## When NOT to Refactor

**Don't refactor if:**
- You're also adding features (separate concerns)
- Tests are failing (fix tests first)
- You don't understand the code (investigate first)
- You're just hitting line limits (quality over metrics)
- It's working fine and code is clear (don't over-engineer)

**Refactoring is NOT:**
- Adding features
- Fixing bugs
- Changing behavior
- "Cleaning up" code you just wrote (write it right first)

---

## Large Refactorings

**If refactoring is large (e.g., splitting 2000-line VM into modules):**

1. **Plan the target structure** - What will the new organization be?
2. **Document the plan** - Write down the steps
3. **Refactor incrementally** - One module at a time
4. **Test at each step** - Never stack changes
5. **May take multiple sessions** - That's OK

**Example: Refactoring VM**
- Day 1: Extract instruction dispatch into separate module
- Day 2: Extract stack operations
- Day 3: Extract value operations
- Each day: tests pass, commit

---

## Performance Refactoring

**If optimizing for performance:**

1. **Measure first** - Profile to find actual bottlenecks
2. **Benchmark baseline** - Know current performance
3. **Optimize** - Make focused changes
4. **Measure after** - Verify improvement
5. **Tests still pass** - No behavior changes

**Don't optimize without measurements.** Premature optimization is real.

---

## Notes

- **Tests are your safety net** - They prove refactoring didn't break behavior
- **Small steps** - Easier to debug, easier to verify
- **Commit often** - Small commits are easy to revert if needed
- **No behavior changes** - That's what makes it a refactoring
- **Quality over line counts** - Don't refactor just to hit arbitrary limits
- **Reference gates/README.md** for GATE 0 and GATE 5 details

---

## After Refactoring: Dead Code Cleanup

**After completing a major refactoring, you may have dead code:**

```
✅ Refactoring complete

All tests pass. Code reorganized.

Noticed: 3 old functions no longer used after refactor

Want to run Dead Code Cleanup workflow to remove them?
```

**When to offer cleanup:**
- After large refactors (splitting modules, moving code)
- When old implementations replaced
- User explicitly asks

**See:** `workflows/dead-code-cleanup.md` for the systematic cleanup process
