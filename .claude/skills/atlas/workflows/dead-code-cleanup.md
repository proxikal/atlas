# Dead Code Cleanup Workflow

**When to use:** Removing unused code/files after refactoring (user-invoked only)

**Key principle:** Conservative deletion with verification - when in doubt, keep it

**Trigger:** User explicitly requests (e.g., "clean up dead code", "remove unused")

**NOT automatic:** NEVER run automatically during normal development

**Reference:** See `gates/README.md` for detailed gate definitions

---

## Workflow Gates

| Gate | Action | Critical Check |
|------|--------|----------------|
| -1 | **Sanity Check** | Right time for cleanup? |
| 0 | **Read Docs** | Understand protected code |
| 1 | **Identify Candidates** | Find unused code/files |
| 2 | **Verify Dead** | Prove truly unused |
| 3 | **Check Protection** | Not intentional stub? |
| 4 | **Present to User** | Get explicit approval |
| 5 | **Remove Code** | Delete approved items |
| 6 | **Verify Build** | cargo check passes |
| 7 | **Run Tests** | All tests pass |
| 8 | **Quality Gates** | clippy, fmt |

---

## GATE -1: Sanity Check

**Check if this is the right time:**

Good time:
- ✅ After major refactoring
- ✅ After feature completion
- ✅ All tests currently passing
- ✅ User explicitly requested

Bad time:
- ❌ During active development
- ❌ Tests failing
- ❌ Features in-progress

**If wrong time:** Discuss with user, recommend waiting until stable.

**If good time:** Proceed to GATE 0

---

## GATE 0: Read Docs

**Read protection lists BEFORE searching:**

1. **Check for intentional stubs:**
   - Optimizer hooks, profiler hooks, debugger hooks
   - Future expansion points
   - FFI stubs, runtime hooks

2. **`STATUS.md`:**
   - What's implemented vs planned
   - Current version scope

**Memorize protected patterns:**
- `optimizer::*` - v0.2 scope
- `profiler::*` - v0.2 scope
- `debugger::*` - v0.2 scope
- Anything in intentional-stubs.md

**BLOCKING:** Cannot proceed without reading intentional-stubs.md

---

## GATE 1: Identify Candidates

**Multiple search strategies:**

### Rust Analyzer Warnings:
```bash
cargo check 2>&1 | grep "never used"
```

### Grep-Based Search:
```bash
# Find function definitions
grep -r "pub fn\|fn " src/ --include="*.rs"

# Check if called
grep -r "function_name" src/ tests/ --include="*.rs"
```

### Dead Files:
```bash
# Find .rs files
find src/ -name "*.rs"

# Check if imported
grep -r "use.*filename" src/
```

### Orphaned Modules:
```bash
# Find mod declarations
grep -r "mod " src/ --include="*.rs"

# Check if used
grep -r "use.*module_name" src/
```

**Output:** Candidate list (NOT deletion list yet)

---

## GATE 2: Verify Dead

**For EACH candidate, prove it's truly unused:**

1. **Direct calls:**
   ```bash
   grep -r "function_name" src/ tests/ --include="*.rs"
   ```

2. **Trait implementations:**
   ```bash
   grep -r "impl.*Trait.*for" src/ | grep -A 10 "function_name"
   ```

3. **Dynamic dispatch checks:**
   - Trait objects?
   - Macros?
   - FFI exports?

4. **Public API:**
   - `pub` in library crate?
   - Part of public API?

**Decision per candidate:**
- ✅ **Dead:** No refs, not public, not trait
- ❌ **Keep:** Found refs, public, or trait
- ⚠️ **Unclear:** Might be dynamic, keep it

**Output:** Verified dead code list

---

## GATE 3: Check Protection

**CRITICAL:** Check against intentional-stubs.md

**For EACH verified dead item:**
```bash
grep -r "unimplemented!\|todo!" crates/atlas-runtime/src/
```

**If FOUND:**
- ❌ Remove from deletion list
- ⚠️ Protected code, keep it

**If NOT FOUND:**
- ✅ Safe to delete

**Output:** Approved deletion list (excluding protected)

**BLOCKING:** Must exclude anything in intentional-stubs.md

---

## GATE 4: Present to User

**MANDATORY:** Show EXACTLY what will be deleted

**Format:**
```
🗑️ DEAD CODE CLEANUP - APPROVAL REQUIRED

FILES (X):
1. src/path/file.rs (N lines)
   Reason: [why it's dead]
   Verified: [how confirmed]

FUNCTIONS (Y):
2. src/path/file.rs::func() (N lines)
   Reason: [why it's dead]
   Verified: [how confirmed]

PROTECTED (NOT deleted):
- optimizer::optimize_bytecode() - Intentional stub
- profiler::profile_run() - Intentional stub

VERIFICATION:
1. Delete items
2. cargo check
3. cargo nextest run -p atlas-runtime (all 1,391+ pass)
4. cargo clippy

Total lines: N

APPROVE? [yes/no]
```

**User must type "yes" to proceed.**

**BLOCKING:** Cannot delete without explicit approval

---

## GATE 5: Remove Code

**Delete systematically:**

### Files:
```bash
git rm src/path/file.rs
```

### Functions:
- Use Edit tool
- Don't leave empty files

### Modules:
- Remove `mod module_name;`
- Remove imports

**Order:**
1. Functions first
2. Then files
3. Then module declarations

**After EACH deletion:**
```bash
cargo check
```

**If fails:** Stop, investigate

---

## GATE 6: Verify Build

**After all deletions:**
```bash
cargo check --all-targets --all-features
```

**MUST pass without errors.**

**If errors:**
- False positive in deletion list
- Stop, investigate, restore

**BLOCKING:** Must build cleanly

---

## GATE 7: Run Tests

**Full test suite:**
```bash
cargo nextest run -p atlas-runtime
```

**ALL must pass:**
- Same count as baseline
- No regressions

**If ANY fail:**
- Code was NOT dead
- Stop immediately
- Restore: `git checkout HEAD -- <file>`
- Investigate why missed

**BLOCKING:** All tests pass, same baseline count

---

## GATE 8: Quality Gates

**Final verification:**

```bash
cargo clippy -- -D warnings  # Zero warnings
cargo fmt -- --check         # All formatted
```

**BLOCKING:** Both must pass

---

## Final Summary

**Present completion summary:**
```
✅ DEAD CODE CLEANUP COMPLETE

Removed:
- X files (N lines)
- Y functions (N lines)
Total: N lines deleted

Protected:
- Z intentional stubs (kept)

Verification:
✓ cargo check passed
✓ cargo nextest run -p atlas-runtime passed (1,391/1,391)
✓ cargo clippy passed
✓ cargo fmt passed

Commit ready.
```

---

## Conservative Principles

**Keep if:**
- Listed in intentional-stubs.md
- Public API
- Trait requirement
- FFI export
- Macro/dynamic dispatch
- Tests reference it
- Unclear usage

**Delete only if:**
- Zero refs (verified multiple ways)
- Not in intentional-stubs.md
- Not public
- Not trait
- User approved
- Tests pass after

**When in doubt, KEEP IT.**

---

## Integration Points

### After Refactoring:
```
✅ Refactoring complete
Tests pass.

Noticed: 3 old functions unused

Run Dead Code Cleanup? [yes/no]
```

### After Structured Development:
- Check at phase boundaries
- After large features
- Not every phase

---

## Notes

- **User-invoked only** - Never automatic
- **Conservative** - When unclear, keep
- **Verification-heavy** - Multiple checks
- **Test-backed** - Full suite must pass
- **Protection-aware** - Respects stubs
- **Explicit approval** - User approves first
- **Cost: ~$0.01** - cargo nextest run -p atlas-runtime is cheap

**Philosophy:** Measure twice, cut once.
