# Bug Fix Workflow

**When to use:** Fixing reported bugs, incorrect behavior, issues

**Approach:** TDD (Test first, then fix)

**Reference:** See `gates/README.md` for detailed gate definitions

---

## Workflow Gates

Bug fixes use **GATE 0, 1.5, 2, 3, 4, 5** with **TDD approach at GATE 2**.

| Gate | Action | Approach |
|------|--------|----------|
| 0 | **Read Docs** | Same as structured dev |
| 1.5 | **Foundation Check** | Check existing code before fix |
| 2 | **Implement + Test** | **TDD: Write FAILING test FIRST** |
| 3 | **Verify Parity** | Both engines work correctly |
| 4 | **Quality Gates** | cargo test, clippy, fmt |
| 5 | **Doc Update** | Update docs if needed |

**Key difference:** GATE 2 uses TDD (test FIRST, then fix). Structured dev uses implementation-driven (implement first, test alongside).

**Skipped:** GATE 0.5 (dependencies), GATE 1 (sizing), GATE 6 (status tracking)

**Reference:** See `gates/README.md` for detailed gate definitions

---

## Bug Fix Specifics

### Before GATE 2: Reproduce Bug

**Create minimal reproduction:**
```rust
#[test]
fn test_bug_ISSUE_NUMBER() {
    // Minimal Atlas code that triggers bug
    let input = "... buggy code ...";
    let result = run_interpreter(input);
    // This currently fails or produces wrong output
    assert_eq!(result, expected);
}
```

**If you can't reproduce:** Get more info from bug report. Can't fix what you can't reproduce.

**Run test - should FAIL (proves bug exists).**

---

### GATE 2: Write Failing Test

**TDD approach - RED first:**
- Test should FAIL before fix
- Proves bug exists
- Will pass after fix

**Test both engines if applicable:**
```rust
#[rstest]
fn test_bug_parity() {
    let input = "... buggy code ...";
    let interpreter_result = run_interpreter(input);
    let vm_result = run_vm(input);

    // Both should produce correct output (currently don't)
    assert_eq!(interpreter_result, expected);
    assert_eq!(vm_result, expected);
}
```

**BLOCKING:** Test must fail before proceeding to GATE 3.

---

### After GATE 2 Test: Locate Root Cause

**Investigation tools:**
- Read related code carefully
- Add temporary debug output if needed
- Check git history (`git log`, `git blame`)
- Look at related tests
- Review relevant docs (implementation guides)

**Common bug locations:**
- Parser: Grammar edge cases, error recovery
- Typechecker: Type inference, constraint solving
- Compiler: Bytecode generation, scope handling
- VM: Instruction execution, stack management
- Interpreter: Value operations, environment

**Don't fix until you understand WHY.**

---

### GATE 2 Continued: Fix Implementation

**Minimal fix principle:**
- Fix root cause, not symptoms
- Don't over-engineer
- Don't refactor unrelated code
- Clear, focused change

**If fix is complex:** Consider if bug reveals design issue. May need larger refactor (separate workflow).

**Test should now pass (GREEN):**
```bash
cargo test test_bug_ISSUE_NUMBER
```

---

### GATE 4: Full Test Suite & Quality

**After quality gates, run full suite:**
```bash
cargo test
```

**Ensure no regressions.** If other tests fail, your fix broke something - refine the fix.

---

## Common Bug Patterns

### Parser Bugs
- Missing grammar rules
- Incorrect precedence
- Error recovery issues
- Token lookahead problems

### Typechecker Bugs
- Inference failures
- Constraint propagation
- Generic instantiation
- Scope resolution

### VM Bugs
- Incorrect bytecode generation
- Stack corruption
- Jump target calculation
- Instruction implementation

### Interpreter Bugs
- Value conversions
- Environment lookup
- Scope handling
- Runtime errors

---

## Notes

- **TDD is mandatory** - Test first, then fix (both at GATE 2)
- **Foundation check** - GATE 1.5 before writing test
- **Parity matters** - Both engines must work correctly (GATE 3)
- **Minimal fixes** - Don't over-engineer
- **Full test suite** - Catch regressions (GATE 4)
- **Quality gates** - Must all pass (GATE 4)
- **Reference gates/README.md** for detailed gate definitions
