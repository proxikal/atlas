---
paths:
  - "**"
---

# Atlas Git & Worktree System

## Worktree Layout

| Directory | Branch | Purpose |
|-----------|--------|---------|
| `~/dev/projects/atlas-dev/` | `main` | **Primary dev worktree** — all code work happens here |
| `~/dev/projects/atlas-docs/` | `worktree/docs` | Docs-only worktree |
| `~/dev/projects/atlas/` | varies | Secondary worktree (check current branch before use) |

**Rule:** Never `git checkout` a branch that is checked out in another worktree. Use `git -C <path>` for cross-worktree operations.

## GitHub Branch Protection (main)

- **PRs required** — direct push to `main` is rejected
- **No merge commits** — main must be linear history (rebase/squash only)
- **CI gate** — "CI Success" check must pass before merge
- **Merge queue** — PRs auto-queue; do not merge manually

## Correct PR Workflow

```bash
# 1. Create feature branch from main
git checkout -b feat/short-description

# 2. Do work, commit locally
git add <files> && git commit -m "feat: description"

# 3. Push branch
git push -u origin feat/short-description

# 4. Create PR with auto-merge
gh pr create --title "title" --body "body"
gh pr merge --auto --squash

# 5. Wait for CI + merge queue — DO NOT force-merge
# 6. After merge: sync local main
git checkout main && git pull origin main
git branch -d feat/short-description
```

## After PR Merges — Sync All Worktrees

```bash
# Sync main worktree
git -C /Users/proxikal/dev/projects/atlas-dev pull origin main

# Sync other worktrees' home branches to new main
git -C /Users/proxikal/dev/projects/atlas-docs rebase main
git -C /Users/proxikal/dev/projects/atlas rebase main
```

## Commit Cadence

**Weekly push model** — batch multiple phases into one PR per week. Do NOT push on every phase. Exception: blocking fixes or large milestones.

## Branch Naming

```
phase/{category}-{number}   # e.g. phase/ownership-01
feat/{short-description}    # e.g. feat/array-slice
fix/{short-description}     # e.g. fix/parser-float
ci/{short-description}      # e.g. ci/optimize-cache
```

## Cross-Worktree Sync Commands (safe pattern)

```bash
# CORRECT — never leaves your current worktree
git -C /Users/proxikal/dev/projects/atlas-dev rebase main
git -C /Users/proxikal/dev/projects/atlas-docs rebase main

# WRONG — checking out a branch owned by another worktree will error
git checkout worktree/docs   # ERROR if atlas-docs has it checked out
```

## Banned

- `git push origin main` directly (branch protection rejects it)
- Merge commits on `main` (`--no-ff` merges)
- `--force` or `--force-with-lease` on `main`
- Skipping CI (`--no-verify`)
