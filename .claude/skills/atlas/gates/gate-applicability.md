# Gate Applicability Matrix

**Purpose:** Not all gates apply to all domains. Use this matrix to determine which gates to execute for your task.

---

## Quick Reference

| Gate | Runtime | LSP | CLI | VM | Frontend | Docs-Only |
|------|---------|-----|-----|-----|----------|-----------|
| **GATE -1** (Sanity) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **GATE 0** (Read Docs) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **GATE 1** (Sizing) | ✅ | ✅ | ✅ | ✅ | ✅ | ⚠️ Optional |
| **GATE 2** (Implement) | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ Skip |
| **GATE 3** (Parity) | ✅ | ❌ Skip | ❌ Skip | ✅ | ❌ Skip | ❌ Skip |
| **GATE 4** (Quality) | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ Skip |
| **GATE 5** (Docs) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **GATE 6** (Testing) | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ Skip |
| **GATE 7** (Memory) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **GATE V** (Versioning) | — event-driven, not part of phase sequence — see below | | | | | |

---

## Gate Descriptions

### GATE -1: Sanity Check
**Always run.** Verifies environment, dependencies, and security.

### GATE 0: Read Docs
**Always run.** Selective reading of specs and patterns.

### GATE 1: Sizing
**Usually run.** Estimate complexity and plan splits.
- **Skip for:** Simple doc updates, small fixes (< 50 lines)

### GATE 2: Implementation
**Run for code changes.** Write and test the actual code.
- **Skip for:** Documentation-only changes

### GATE 3: Parity Verification
**Only for dual-engine features.** Verify interpreter and VM produce identical output.

**When to run:**
- ✅ Runtime features (must work in both interpreter and VM)
- ✅ Stdlib functions (must have identical behavior)
- ✅ VM bytecode changes (must match interpreter)

**When to skip:**
- ❌ LSP (no dual-engine)
- ❌ CLI (no dual-engine)
- ❌ Frontend (no dual-engine)
- ❌ Documentation
- ❌ Tests themselves

**How to verify parity:**
```rust
// Use assert_parity helper
assert_parity(r#"len("hello")"#, "5");
```

### GATE 4: Quality Checks
**Run for all code changes.** Clippy, fmt, tests for the specific domain.

**Commands:**
```bash
cargo clippy -p <package> -- -D warnings
cargo fmt --check -p <package>
cargo nextest run -p <package> --test <domain_file>
```

**Skip for:** Documentation-only changes

### GATE 5: Documentation
**Run when docs are required.** Update relevant docs for code changes.

**When to skip:** Obvious (internal refactors, test-only changes)

### GATE 6: Final Testing
**Run before committing.** Full test suite for the package.

**Requirement: 100% pass rate. No exceptions.**

- ✅ 100% pass → Proceed to GATE 7
- ❌ Any failure → Fix it, do not proceed

### GATE 7: Memory Check
**Always run.** Update memory if you discovered new patterns or made decisions.

### GATE V: Versioning
**Event-driven — not part of the phase sequence.**

Run at exactly two moments:
1. **After a `fix/` PR merges to main** → patch tag check
2. **After a block AC check phase is committed** → minor version check

See `gates/gate-versioning.md` for the full decision process. Never ask the user — the exit criteria in the current version plan are the authority.

**When to update memory:**
- ✅ Hit an API surprise (pattern wasn't documented)
- ✅ Made an architectural decision (new constraint)
- ✅ Found a bug in existing patterns (fix the docs)
- ✅ Discovered crate-specific behavior (document it)

**When NOT to update:**
- ❌ Just following existing patterns (already documented)
- ❌ Phase-specific one-time work (not reusable)
- ❌ Obvious or trivial changes

---

## Domain-Specific Notes

### LSP Testing
- **Pattern:** Inline server creation (see `memory/testing-patterns.md`)
- **No helper functions** for server setup (lifetime issues)
- **Check existing tests first** before writing new ones

### Runtime Testing
- **Pattern:** Domain files (see `memory/testing-patterns.md`)
- **No new test files** without authorization
- **Parity required** for all features

### CLI Testing
- **Pattern:** Integration tests using `assert_cmd`
- **Use cargo_bin!** macro (not deprecated `cargo_bin()`)
- **Test cross-platform** paths (use `Path` APIs, not string manipulation)

### Documentation
- Commit directly to feature branch in `atlas-docs/` worktree
- Merge to local main when complete
- Push to GitHub on weekly cadence (no PR needed)

---

## Examples

### Example 1: LSP Feature
```
Task: Add new LSP hover feature
Gates: -1, 0, 1, 2, [skip 3], 4, 5, 6, 7
```

### Example 2: Stdlib Function
```
Task: Add new string method
Gates: -1, 0, 1, 2, 3 (parity!), 4, 5, 6, 7
```

### Example 3: Documentation Update
```
Task: Update README
Gates: -1, 0, [skip 1], [skip 2], [skip 3], [skip 4], 5, [skip 6], 7
```

### Example 4: Bug Fix
```
Task: Fix parser issue
Gates: -1, 0, [skip 1 - small fix], 2, 3 (if runtime), 4, 5, 6, 7
```

---

## Decision Tree

**Start here:** What are you working on?

1. **Documentation only?**
   → Run: -1, 0, 5, 7
   → Skip: 1, 2, 3, 4, 6

2. **LSP/CLI/Frontend code?**
   → Run: -1, 0, 1, 2, 4, 5, 6, 7
   → Skip: 3 (no parity)

3. **Runtime/Stdlib/VM code?**
   → Run: ALL gates (-1 through 7)
   → Parity is REQUIRED

4. **Small fix (< 50 lines)?**
   → Run: -1, 0, 2, 4, 6, 7
   → Skip: 1 (no sizing needed), maybe 5 (if obvious)

---

**Rule of thumb:** When in doubt, run the gate. Skipping gates is an optimization, not a requirement.
