# GitHub Repository Configuration

**Repo:** `atl-lang/atlas` | **Updated:** 2026-02-20

**Architecture:** Rulesets only (legacy Branch Protection Rules deleted)
**CI Status:** DISABLED (jobs skipped with `if: false`, `ci-success` always passes)

---

## Automation Settings

| Setting | Status | Effect |
|---------|--------|--------|
| Auto-merge | ✅ | PRs merge automatically |
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
| Require status checks | `CI Success` must pass (always does) |
| Require up-to-date | Branch must be current with main |
| Block force pushes | No `--force` to main |

### Merge Queue Settings

| Setting | Value | Rationale |
|---------|-------|-----------|
| Merge method | Squash | Linear history, clean commits |
| Build concurrency | 5 | Parallel processing |
| Min group size | 1 | Don't wait for batching |
| Max group size | 5 | Batch if multiple PRs |
| Wait time | 1 min | Fast for solo AI dev |
| All entries pass | ✅ | Each PR validated |
| Timeout | 60 min | Allow long operations |

---

## CI Pipeline (`ci.yml`) — DISABLED

**Status:** All jobs have `if: false` — nothing runs
**Required check:** `CI Success` (always passes, empty job)

**Why disabled:** Preserve GitHub Actions credits. AI validates locally before pushing.

**What still works:**
- Merge queue processes PRs
- Auto-merge enabled
- Branch auto-deletion
- Linear history enforcement

**What doesn't run:**
- fmt, clippy, check — AI runs these locally
- Tests — AI runs full suite locally before pushing
- Security scans — AI runs `cargo audit` locally

---

## AI Workflow (Optimized for No CI)

```bash
# 1. Complete phase work locally
cargo nextest run -p atlas-runtime          # Full test suite
cargo clippy -p atlas-runtime -- -D warnings # Zero warnings
cargo audit                                  # Security scan

# 2. Push and create PR (one operation)
git add -A && git commit -m "feat(phase): Description"
git push -u origin HEAD
gh pr create --title "..." --body "..."
gh pr merge --squash --auto                  # Add to queue

# 3. Wait briefly for merge (~30-60s)
sleep 30

# 4. Sync and cleanup
git checkout main && git pull
git branch -d <old-branch>                   # Clean local ref
```

**Key changes from CI-enabled workflow:**
- No waiting for CI (was ~1-1.5 min)
- No PR watching (was checking merge status repeatedly)
- No batch logic (was batching phases to save CI minutes)
- Push after EVERY phase (no batching needed)

---

## Security (Local Scanning)

Since CI security scans are disabled, AI runs these locally:

```bash
# Required in GATE -1
cargo audit

# Optional (stricter)
cargo deny check
```

---

## Merge Queue Notes

- `gh pr merge --squash --auto` adds PR to queue
- Message "merge strategy is set by merge queue" = normal
- Merges happen in ~30-60 seconds (no CI to run)
- Remote branch auto-deleted after merge
- Local branch must be manually deleted: `git branch -d <branch>`
