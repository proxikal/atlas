# Debugging Workflow

**When to use:** Investigating issues, understanding behavior, root cause analysis

**Approach:** Systematic investigation, not random changes

**Reference:** See `gates/README.md` for GATE 0 (investigation starts with reading docs)

---

## Workflow

Debugging is investigation-focused. Uses **GATE 0** for context, then systematic analysis.

| Step | Action |
|------|--------|
| 0 | **Read Docs** - See gates/gate-0-read-docs.md (understand system) |
| 1 | **Reproduce Consistently** - Get reliable reproduction |
| 2 | **Gather Information** - Collect all relevant data |
| 3 | **Form Hypothesis** - What do you think is wrong? |
| 4 | **Test Hypothesis** - Verify or disprove |
| 5 | **Narrow Down** - Isolate the problem area |
| 6 | **Root Cause** - Find WHY it's happening |
| 7 | **Document Findings** - Write down what you learned |

**Note:** Debugging is research, not implementation. After finding root cause, switch to appropriate workflow (bug-fix, refactoring, etc.).

---

## Step 1: Reproduce Consistently

**Get reliable reproduction:**
- Minimal test case
- Clear steps to trigger issue
- Consistent results

**If can't reproduce reliably:** Investigate environmental factors, timing, state.

---

## Step 2: Gather Information

**Collect data:**
- Error messages (full text)
- Stack traces
- Input that triggers issue
- Expected vs actual output
- Recent changes (git log)
- Related tests

**Use debugging tools:**
```bash
# Dump AST to see parsing
cargo run -- --dump-ast test.atl

# Dump bytecode to see compilation
cargo run -- --dump-bytecode test.atl

# Run with specific test
cargo test test_name -- --nocapture

# Run clippy for hints
cargo clippy
```

---

## Step 3: Form Hypothesis

**Based on data, hypothesize:**
- "Parser might be mishandling operator precedence"
- "VM stack might be corrupted by function calls"
- "Typechecker might not be unifying constraints correctly"

**Good hypothesis:**
- Specific (not "something's broken")
- Testable (can verify or disprove)
- Based on evidence (not random guess)

---

## Step 4: Test Hypothesis

**Design experiment:**
- Add debug output
- Write targeted test
- Check intermediate state
- Trace execution flow

**Example:**
```rust
// Hypothesis: Stack is corrupted after function call
#[test]
fn test_stack_after_call() {
    let input = "fn f() { return 42; } f();";
    let vm = compile_and_run(input);
    // Check stack state
    assert_eq!(vm.stack.len(), 1); // Should have return value
}
```

**Verify or disprove hypothesis.**

---

## Step 5: Narrow Down

**Binary search approach:**
- Does it happen with simpler input? (Simplify)
- Does it happen in both interpreter and VM? (Isolation)
- Does it happen with specific operator? (Specificity)

**Goal:** Smallest possible test case that triggers issue.

**Example progression:**
```
"complex program with 50 lines" (issue happens)
  → "just the problematic function" (issue happens)
    → "just the problematic expression" (issue happens)
      → "minimal expression" (FOUND IT)
```

---

## Step 6: Root Cause

**Find WHY, not just WHERE:**
- Don't just find the crashing line
- Understand WHY that line crashes
- What assumptions were violated?
- What state led to this?

**Read the code carefully:**
- Understand control flow
- Check invariants
- Verify assumptions
- Look for edge cases

**Common root causes:**
- Off-by-one errors
- Unhandled edge cases
- Incorrect assumptions
- Missing null checks
- Stack/heap corruption
- Race conditions (rare in Atlas)

---

## Step 7: Document Findings

**Write down:**
- What the issue was
- Why it happened
- How you found it
- How to reproduce
- How to fix (if known)

**This helps:**
- Future debugging
- Bug fix workflow
- Team knowledge
- Similar issues

---

## Debugging Tools

### Atlas-specific
```bash
# AST dump
cargo run -- --dump-ast file.atl

# Bytecode dump
cargo run -- --dump-bytecode file.atl

# Typecheck dump
cargo run -- --dump-typecheck file.atl

# Run with test
cargo test test_name -- --nocapture
```

### Rust debugging
```bash
# Run with backtrace
RUST_BACKTRACE=1 cargo test

# Run with full backtrace
RUST_BACKTRACE=full cargo test

# Clippy for hints
cargo clippy

# Check specific test
cargo test --test specific_test
```

---

## When to Switch Workflows

**After debugging, you might:**
- **Bug Fix workflow** - Found a bug, time to fix it (use gates 0, 1.5, 2-5)
- **Refactoring workflow** - Found design issue, needs cleanup
- **Structured Development workflow** - Found missing feature
- **Enhancement workflow** - Found enhancement opportunity

**Debugging → Understanding → Action**

---

## Notes

- **Systematic, not random** - Don't just try things hoping they work
- **Hypothesis-driven** - Form theories, test them
- **Document findings** - Knowledge is valuable
- **Narrow down** - Smallest reproduction case
- **Understand WHY** - Root cause matters
- **Reference gates/README.md** for gate workflow after debugging
