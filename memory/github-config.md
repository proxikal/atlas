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
| PR (code) | fmt → clippy → test (ubuntu) | ~2 min |
| PR (docs) | fmt only | ~15s |
| Merge queue | fmt → clippy → test-matrix (3 platforms) | ~6 min |
| Main push | coverage only | ~5 min |
| Nightly | fuzz + MSRV | ~10 min |

**Optimizations:**
- Path filtering (dorny/paths-filter)
- Rust caching (Swatinem/rust-cache)
- Concurrency control (cancel-in-progress)
- Tiered testing (ubuntu PR → matrix merge)

---

## AI Workflow

```bash
# 1. Create PR with auto-merge
git push -u origin HEAD
gh pr create --title "..." --body "..."
gh pr merge --squash --auto

# 2. Done - automation handles rest
# CI runs (~2 min PR, ~6 min merge queue)
# Auto-merge when CI passes
# Branch auto-deleted
# Main updated

# 3. Sync local
git checkout main && git pull
git branch -d <old-branch>
```

**No manual merge, no manual cleanup, no waiting.**
