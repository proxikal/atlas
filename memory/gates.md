# Atlas Quality Gates

**Purpose:** Quality gate definitions and validation procedures.

---

## Gate Philosophy

**Gates are BLOCKING checkpoints.** Failure at any gate = STOP and fix before proceeding.

**NO shortcuts allowed:**
- No `// TODO` in gate-passing code
- No `unimplemented!()` stubs
- No "good enough for now"
- 100% = 100%, not 95%

---

## GATE -1: Sanity Check (ALWAYS FIRST)

**Purpose:** Verify prerequisites and clean compile before starting work.

**Required:** Run BEFORE any implementation begins.

### Steps

#### 1. Verify Phase Dependencies

**Check phase file "Dependencies" section:**
```bash
# Example from phase file:
ls crates/atlas-runtime/src/stdlib/collections/{hashmap,hashset,queue,stack,hash}.rs

grep "HashMap\|HashSet\|Queue\|Stack" crates/atlas-runtime/src/value.rs | grep "Arc<Mutex"
```

**If missing:** STOP. Complete prerequisite phases first.

#### 2. Clean Build Check

```bash
cargo clean && cargo check -p atlas-runtime
```

**Expected:** No errors, no warnings.

**On failure:**
- Read error messages carefully
- Fix issues before proceeding
- Re-run until success

#### 3. Verify Referenced Files Exist

**Check all files referenced in phase:**
```bash
# Example validations:
ls crates/atlas-runtime/src/interpreter/   # Must exist (directory)
ls crates/atlas-runtime/src/vm/            # Must exist (directory)
ls crates/atlas-runtime/src/stdlib/mod.rs  # Must exist
```

**If missing:** Phase file is WRONG. Update phase file or inform user.

### Outcome

✅ **PASS:** Clean compile, all dependencies present → Proceed to GATE 0

❌ **FAIL:** Stop immediately, inform user with error details

---

## GATE 0: Declaration

**Purpose:** Declare workflow type and set expectations.

### Workflow Types

Declare ONE of:
- **Structured Development:** Following documented plan (phase file)
- **Bug Fix:** Fixing incorrect behavior
- **Refactoring:** Code cleanup (no behavior change)
- **Debugging:** Investigation, root cause analysis
- **Enhancement:** Adding capabilities beyond spec

### Example

```
GATE 0: PASS
Workflow: Structured Development
Phase: phase-07d-collection-integration.md
Objective: Implement HashMap/HashSet iteration intrinsics
```

---

## GATE 1: Implementation

**Purpose:** Feature is fully implemented with no shortcuts.

### Requirements

#### Code Quality
- ✅ No `// TODO` comments
- ✅ No `unimplemented!()` macros
- ✅ No placeholder functions
- ✅ All edge cases handled
- ✅ Error handling complete
- ✅ Clear, readable code

#### Completeness
- ✅ All acceptance criteria addressed
- ✅ Interpreter implementation complete
- ✅ VM implementation complete
- ✅ Registration in stdlib/mod.rs (if applicable)

#### Documentation
- ✅ Function-level doc comments (for complex functions)
- ✅ Inline comments for non-obvious logic
- ✅ Error messages are descriptive

### Validation

```bash
# Compiles without errors
cargo check -p atlas-runtime

# No grep hits for banned patterns
grep -r "TODO" crates/atlas-runtime/src/
grep -r "unimplemented!" crates/atlas-runtime/src/
```

### Outcome

✅ **PASS:** Implementation complete → Proceed to GATE 2

❌ **FAIL:** Finish implementation first

---

## GATE 2: Testing

**Purpose:** Comprehensive tests verify behavior.

### Requirements

#### Test Coverage
- ✅ Happy path tests (basic functionality)
- ✅ Edge case tests (empty, large, boundaries)
- ✅ Error tests (wrong types, invalid inputs)
- ✅ Integration tests (if cross-feature)
- ✅ Minimum test count met (from phase acceptance criteria)

#### Test Quality
- ✅ Clear test names (`test_{feature}_{scenario}`)
- ✅ Focused assertions (one concern per test)
- ✅ Tests are independent (no shared state)
- ✅ Good error messages

#### Test Execution During Development

**CRITICAL:** Use `-- --exact` flag to run ONE test at a time.

```bash
# CORRECT: Run specific test
cargo test -p atlas-runtime test_hashmap_foreach -- --exact

# WRONG: Run all tests (wastes time)
cargo test -p atlas-runtime
```

**Rationale:**
- Faster feedback (seconds vs minutes)
- Clear which test failed
- Don't re-run passing tests

### Validation

```bash
# Count tests (must meet phase minimum)
grep -c "^#\[test\]" crates/atlas-runtime/tests/<phase_test_file>.rs

# Run phase test files (NOT full suite)
cargo nextest run -p atlas-runtime --test <phase_test_file>
```

### Outcome

✅ **PASS:** All tests pass, coverage complete → Proceed to GATE 3

❌ **FAIL:** Fix failing tests or add missing tests

---

## GATE 3: Parity

**Purpose:** Interpreter and VM produce identical results.

### Requirements

- ✅ All integration tests pass in interpreter mode (default)
- ✅ All integration tests pass in VM mode
- ✅ Outputs are identical

### Validation

```bash
# All tests (excludes network tests by default)
cargo nextest run -p atlas-runtime

# VM tests (if VM test mode exists)
# NOTE: Currently tested via bytecode_compiler_integration tests
cargo nextest run -p atlas-runtime --test bytecode_compiler_integration
```

**Manual Verification:**
- Run same Atlas code in interpreter
- Run same Atlas code in VM
- Compare outputs (must be identical)

### Outcome

✅ **PASS:** Outputs identical → Proceed to GATE 4

❌ **FAIL:** Fix parity break (check implementation diff)

---

## GATE 4: Quality (Code Cleanliness)

**Purpose:** Code meets Atlas quality standards.

### Requirements

#### Formatting

```bash
cargo fmt -p atlas-runtime
```

**Expected:** No changes (already formatted)

#### Linting

```bash
cargo clippy -p atlas-runtime -- -D warnings
```

**Expected:** Zero warnings

**Common clippy fixes:**
- Unnecessary clones
- Unused variables
- Inefficient patterns
- Type simplifications

#### Build

```bash
cargo build -p atlas-runtime --release
```

**Expected:** Clean release build

### Outcome

✅ **PASS:** No warnings, clean build → Proceed to GATE 5

❌ **FAIL:** Fix clippy warnings and format code

---

## GATE 5: Documentation

**Purpose:** Changes are documented appropriately.

### Requirements

#### Code Documentation
- ✅ Complex functions have doc comments
- ✅ Public APIs documented
- ✅ Examples for non-obvious usage

#### Decision Log (If Applicable)
- ✅ Architectural decisions recorded in `memory/decisions.md`
- ✅ Use DR-XXX format

#### Update Memory (If Applicable)
- ✅ New patterns added to `memory/patterns.md`
- ✅ Testing patterns added to `memory/testing-patterns.md`

### Validation

**Check if documentation needed:**
- New architectural pattern? → Add to patterns.md
- Key decision made? → Add to decisions.md
- New testing approach? → Add to testing-patterns.md

### Outcome

✅ **PASS:** Documentation complete → Proceed to GATE 6

❌ **FAIL:** Add missing documentation

---

## GATE 6: Handoff

**Purpose:** Phase complete, ready for next phase.

### Requirements

#### All Acceptance Criteria Met

**From phase file:** Verify each checkbox

Example:
- ✅ 6 iteration functions implemented
- ✅ All intrinsics in interpreter and VM
- ✅ 33+ tests pass
- ✅ 100% parity verified
- ✅ No clippy warnings
- ✅ cargo test passes

#### STATUS.md Updated

**See `STATUS.md` → "Handoff Protocol" section**

Required updates:
1. Mark phase as ✅ complete in phase list
2. Update "Last Completed" field
3. Update "Next Phase" field
4. Update category progress percentage
5. Update "Last Updated" date

#### Git Commit

```bash
git add STATUS.md
git commit -m "Complete Phase-07d: Collection Integration

All acceptance criteria met:
- 6 intrinsics implemented (interpreter + VM)
- 22 tests passing (HashMap, HashSet, integration)
- 100% parity verified
- Zero clippy warnings

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

### Final Checks

```bash
# Run phase-specific test files (NOT full suite)
cargo nextest run -p atlas-runtime --test <relevant_test_file>

# Clippy clean
cargo clippy -p atlas-runtime -- -D warnings

# Verify clean status
git status
```

**Network tests** are `#[ignore = "requires network"]` — skipped by default.
Run `cargo nextest run -p atlas-runtime --run-ignored all` to include them.
Individual/per-file tests + cargo check + clippy catch regressions without wasting 15+ minutes.

### Outcome

✅ **PASS:** Phase complete! Deliver handoff summary to user.

❌ **FAIL:** Cannot handoff with incomplete criteria.

---

## Gate Summary Table

| Gate | Purpose | Key Commands | Blocking |
|------|---------|--------------|----------|
| **-1** | Sanity | `cargo clean && cargo check -p atlas-runtime` | ✅ Yes |
| **0** | Declare | (declare workflow type) | ❌ No |
| **1** | Implement | `cargo check -p atlas-runtime` | ✅ Yes |
| **2** | Test | `cargo nextest run -p atlas-runtime --test <file>` | ✅ Yes |
| **3** | Parity | (verify interpreter == VM) | ✅ Yes |
| **4** | Quality | `cargo clippy -p atlas-runtime -- -D warnings` | ✅ Yes |
| **5** | Document | (update memory/ if needed) | ✅ Yes |
| **6** | Handoff | `git commit` + STATUS.md update | ✅ Yes |

---

## Testing Protocol (Critical!)

### During Development

**ALWAYS use `-- --exact` for single test execution:**

```bash
# ✅ CORRECT
cargo test -p atlas-runtime test_hashmap_foreach -- --exact

# ❌ WRONG
cargo test -p atlas-runtime  # Full suite (slow)
cargo test -p atlas-runtime hashmap  # Pattern match (runs many)
```

### Before Handoff (GATE 6 Only)

**Run phase-specific test files:**

```bash
cargo test -p atlas-runtime --test <test_file>  # Per-file validation
cargo clippy -p atlas-runtime -- -D warnings    # Quality check
```

**Full suite is EMERGENCY ONLY** — use when something unexplainable is happening.

**Rationale:**
- During dev: single tests (seconds)
- Before handoff: per-file + clippy (seconds)
- Full suite: 15+ minutes, almost never catches anything cargo check + targeted tests don't

---

## Handling Gate Failures

### GATE -1 Failure (Sanity)

**Action:** STOP immediately.

**Common causes:**
- Missing prerequisite phases
- Codebase in broken state
- Wrong branch

**Fix:**
1. Check STATUS.md for phase dependencies
2. Complete prerequisite phases
3. Verify clean main/master branch
4. Re-run GATE -1

### GATE 2 Failure (Tests)

**Action:** Debug failing test.

**Process:**
1. Run failing test with `-- --exact --nocapture`
2. Read error message carefully
3. Check test expectations vs actual output
4. Fix implementation or test (depending on which is wrong)
5. Re-run test until pass

### GATE 3 Failure (Parity)

**Action:** Compare interpreter vs VM implementation.

**Common causes:**
- Forgot to implement in VM
- Different execution order
- Different error handling

**Fix:**
1. Read both implementations side-by-side
2. Identify difference
3. Align implementations (should be near-identical)
4. Re-run parity check

### GATE 4 Failure (Clippy)

**Action:** Fix all warnings.

**Process:**
```bash
cargo clippy -p atlas-runtime -- -D warnings
```

Read each warning, apply suggested fix, re-run until zero warnings.

---

## Gate Checklist (Quick Reference)

Use this checklist during execution:

```
Phase: _______________

⬜ GATE -1: Sanity check passed
⬜ GATE 0: Workflow declared
⬜ GATE 1: Implementation complete
⬜ GATE 2: All tests passing (used -- --exact during dev)
⬜ GATE 3: Parity verified
⬜ GATE 4: Clippy clean, formatted
⬜ GATE 5: Documentation updated
⬜ GATE 6: STATUS.md updated, committed

All acceptance criteria met: ⬜
Ready for handoff: ⬜
```

---

## References

- **Phase Files:** `phases/` directory (each phase defines acceptance criteria)
- **Patterns:** `memory/patterns.md` (implementation patterns)
- **Decisions:** `memory/decisions.md` (architectural decisions)
- **Testing:** `memory/testing-patterns.md` (how to test)
- **Status:** `STATUS.md` (progress tracking)
