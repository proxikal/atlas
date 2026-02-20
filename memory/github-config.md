# GitHub Repository Configuration

**Repo:** `atl-lang/atlas` | **Updated:** 2026-02-20

**Architecture:** Rulesets only (legacy Branch Protection Rules deleted)

---

## Automation Settings

| Setting | Status | Effect |
|---------|--------|--------|
| Auto-merge | ✅ | PRs merge automatically when CI passes |
| Auto-delete branches | ✅ | Head branches deleted after merge |
| Auto-close issues | ✅ | Linked issues close on PR merge |

---

## Ruleset: `main-merge-queue` (Active)

**Target:** Default branch (`main`)
**Bypass:** None

### Enabled Rules

| Rule | Effect |
|------|--------|
| Restrict deletions | Only bypass users can delete main |
| Require linear history | No merge commits |
| Require merge queue | All merges go through queue |
| Require PR before merge | Direct pushes blocked |
| Require status checks | `CI Success` must pass |
| Require up-to-date | Branch must be current with main |
| Block force pushes | No `--force` to main |

### Merge Queue Settings

| Setting | Value | Rationale |
|---------|-------|-----------|
| Merge method | Squash | Linear history, clean commits |
| Build concurrency | 5 | Parallel CI runs |
| Min group size | 1 | Don't wait for batching |
| Max group size | 5 | Batch if multiple PRs |
| Wait time | 1 min | Fast for solo AI dev |
| All entries pass | ✅ | Each PR validated |
| Timeout | 60 min | Allow long CI |

### PR Settings

- Required approvals: **0** (AI-driven solo dev)
- Allowed merge: **Squash only**

---

## CI Pipeline (`ci.yml`)

**Required check:** `CI Success`

| Event | Jobs | Time |
|-------|------|------|
| PR (code) | fmt → clippy → check | ~1-1.5 min |
| PR (docs) | fmt only | ~15s |
| Merge queue | fmt → clippy → check | ~1-1.5 min |

**Philosophy:** AI runs full test suite locally before pushing. CI is just a safety net for lint/compile.

**Cross-platform testing:** Done locally (macOS dev machine, Windows laptop)

**Optimizations:**
- Path filtering (dorny/paths-filter)
- Rust caching (Swatinem/rust-cache)
- Concurrency control (cancel-in-progress)

---

## AI Workflow

```bash
# 1. Create PR and add to merge queue (run ONCE, immediately)
git push -u origin HEAD
gh pr create --title "..." --body "..."
gh pr merge --squash --auto              # Adds to queue when CI passes

# 2. Walk away - automation handles everything
# CI runs (~1-1.5 min) — fmt, clippy, check only
# Queue merges when CI passes
# Branch auto-deleted
# Main updated

# 3. Sync local
git checkout main && git pull
git branch -d <old-branch>
```

**Merge queue notes:**
- `--squash` flag ignored (queue controls merge method)
- Message "merge strategy is set by merge queue" = normal
- `gh pr view --json autoMergeRequest` shows `null` = normal (queue handles it)
- "already queued to merge" = working correctly

**No manual merge, no manual cleanup, no waiting.**
