---
paths:
  - "**"
---

# Atlas Git Workflow

**Single workspace:** `~/dev/projects/atlas/` on `main`. No other worktrees.

## GitHub Branch Protection (main)

- **PRs required** — direct push to `main` is rejected
- **No merge commits** — linear history only (squash)
- **CI gate** — "CI Success" check must pass
- **Auto-merge** — use `gh pr merge --auto --squash`; merges when CI passes

## PR Workflow

```bash
# 1. Start from clean main
git checkout main && git pull origin main
git checkout -b feat/short-description

# 2. Do work, commit
git add <files> && git commit -m "feat: description"

# 3. Push + PR
git push -u origin feat/short-description
gh pr create --title "title" --body "body"
gh pr merge --auto --squash

# 4. After merge: sync and clean up
git checkout main && git pull origin main
git branch -d feat/short-description
```

## Branch Naming

```
phase/{category}-{number}   # e.g. phase/ownership-01
feat/{short-description}
fix/{short-description}
ci/{short-description}
```

## Commit Cadence

Batch multiple phases into one PR per week. Do NOT push every phase.
Exception: blocking fixes or large milestones.

## Banned

- `git push origin main` directly
- Merge commits (`--no-ff`)
- `--force` on main
- `--no-verify`
