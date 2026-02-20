# GATE 7: Memory Check (After Every Phase)

**Condition:** Phase complete, STATUS.md updated, ready to commit

**Purpose:** Keep AI memory accurate. Prevents drift that wastes tokens in future sessions.

---

## Quick Self-Check (10 seconds)

Ask yourself:

1. **Did I hit an API surprise?** (wrong signature, unexpected return type, missing method)
   → Update `memory/patterns.md`

2. **Did I discover a new codebase pattern?** (new helper, new module, new convention)
   → Update `memory/patterns.md`

3. **Did I make an architectural decision?** (chose between approaches, spec was silent)
   → Update `memory/decisions.md`

4. **Is anything in memory wrong?** (found that memory said X but codebase does Y)
   → Fix the stale entry

---

## Rules

- **Only update if something actually changed.** Most phases won't need memory updates.
- **Be surgical.** Update the specific line/section, don't rewrite whole files.
- **Be token-efficient.** One-liner patterns, not paragraphs of explanation.
- **Verify before writing.** Don't add patterns from a single observation — confirm against codebase.
- **Remove stale info** rather than flagging it. Flags become permanent noise.

---

## What NOT to Save

- Session-specific context (current task, temporary state)
- Anything already in the skill or gate files
- Obvious Rust patterns (how `Result` works, etc.)
- Speculative conclusions from reading one file

---

**Cost:** 0-30 seconds per phase. Prevents hours of confusion in future sessions.

---

## Git Finalization (After Memory Check)

1. **Commit all changes:**
   ```bash
   git add -A
   git commit -m "feat(category): Phase X - description"
   ```

2. **Push, create PR, enable auto-merge:**
   ```bash
   git push -u origin HEAD
   gh pr create --title "Phase X: Title" --body "## Summary
   - Key change 1
   - Key change 2

   🤖 Generated with Claude Code"
   gh pr merge --squash --auto
   ```

3. **Walk away** - automation handles everything:
   - CI runs (~3-4 min)
   - Auto-adds to merge queue
   - Auto-merges and auto-deletes branch
   - **Do NOT run `gh pr merge` again**

4. **Sync local** (after merge completes):
   ```bash
   git checkout main && git pull
   ```

**Next:** Report completion summary.
