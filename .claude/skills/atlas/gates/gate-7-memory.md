# GATE 7: Memory Check (After Every Phase)

**Condition:** Phase complete, STATUS.md updated, ready to commit

**Purpose:** Keep AI memory accurate and lean. Prevents drift and bloat.

---

## Quick Self-Check (10 seconds)

Ask these questions:

1. **Did I hit an API surprise?** (pattern not documented) → Update `patterns.md`
2. **Did I make an architectural decision?** (new constraint or approach) → Update `decisions/{domain}.md`
3. **Did I discover a crate-specific pattern?** (like LSP testing) → Update `testing-patterns.md` or `patterns.md`
4. **Is anything in memory wrong or stale?** → Fix or archive it
5. **Is any file approaching size limit?** → Split or archive

## When to Update Memory

### ✅ DO Update Memory

**patterns.md:**
- Hit an undocumented API quirk that cost time
- Discovered a crate-specific testing pattern
- Found a common error pattern with a fix
- Learned a Rust pattern that's Atlas-specific

**Example:** "LSP tests can't use helper functions due to lifetime issues"

**decisions/{domain}.md:**
- Made an architectural choice between alternatives
- Established a new constraint or rule
- Chose an approach that affects future work
- Resolved an ambiguity in the spec

**Example:** "DR-015: LSP testing uses inline pattern (no helpers due to tower-lsp lifetime constraints)"

### ❌ DON'T Update Memory

**Skip if:**
- Just following existing patterns (already documented)
- Phase-specific work (not reusable knowledge)
- Obvious or trivial changes
- Implementation details (not architectural)
- Temporary workarounds

**Example:** Don't document "Created 10 integration tests" (obvious, not reusable)

---

## File Size Limits (BLOCKING)

**Before committing, run this check:**
```bash
wc -l ~/.claude/projects/*/memory/*.md ~/.claude/projects/*/memory/decisions/*.md 2>/dev/null | grep -v total
```

| File | Max | If Exceeded → MUST DO |
|------|-----|----------------------|
| MEMORY.md | 50 | Split content to topic files |
| patterns.md | 150 | Archive old → `archive/YYYY-MM-patterns.md` |
| decisions/{x}.md | 100 | Split into sub-files |

**BLOCKING:** If ANY file exceeds limit, you MUST split/archive BEFORE committing.
**NO EXCEPTIONS.** This is not optional. Bloated memory = wasted tokens every message.

---

## Memory Structure

```
memory/
├── MEMORY.md           # Index ONLY (pointers, not content)
├── patterns.md         # Active patterns
├── decisions/          # Split by domain
│   ├── language.md
│   ├── runtime.md
│   ├── stdlib.md
│   ├── cli.md          # CRITICAL decisions here
│   ├── typechecker.md
│   ├── vm.md
│   └── {new-domain}.md # Add as needed
└── archive/            # Old stuff goes here
```

---

## How to Split patterns.md

When `patterns.md` exceeds 150 lines:
1. Create `archive/YYYY-MM-patterns.md` with old/stable patterns
2. Keep only actively-referenced patterns in `patterns.md`
3. Update MEMORY.md index if needed

**Example split:**
- `patterns.md` → Active patterns (runtime, stdlib, testing)
- `archive/2026-02-patterns.md` → Stable patterns (frontend API, error handling)

---

## Rules

- **Surgical updates.** One-liner patterns, not paragraphs.
- **Verify before writing.** Confirm against codebase.
- **Archive, don't delete.** Move to `archive/YYYY-MM-{file}.md`.
- **Split by domain.** New domain = new file in `decisions/`.

---

## Required Output (MANDATORY)

**In completion summary, include Memory section:**

```markdown
### Memory
- Updated: `patterns.md` (added X)
- Updated: `decisions/cli.md` (DR-003: reason)
- Archived: `patterns.md` → `archive/2026-02-patterns-v1.md`
```

OR if no updates:

```markdown
### Memory
- No updates needed
```

**This is NOT optional.** Visible accountability prevents drift.

---

## Git Finalization (After Memory Check)

1. `git add -A && git commit -m "feat(category): description"`
2. `git checkout main && git merge --no-ff <feature-branch>`
3. `git branch -d <feature-branch>`
4. `git checkout worktree/dev`

**Next:** Report completion summary with Memory section.
