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
block/{name}                # e.g. block/trait-system — ONE branch per block
feat/{short-description}    # standalone features outside the block plan
fix/{short-description}     # blocking bug fixes (may PR immediately)
ci/{short-description}      # CI/infra changes
docs/{short-description}    # docs-only changes
```

## Commit Cadence — ONE PR PER BLOCK

All scaffold commits, phase execution commits, and spec/STATUS updates for a block
live on the **same branch** (`block/{name}`). The PR is opened only when the block's
final AC check phase is complete.

```
block/trait-system branch:
  scaffold commit
  phase-01 commit
  phase-02 commit
  ...
  phase-18 commit (spec + AC check)
  ← PR opened here, auto-merged
```

**Exception:** Blocking fixes or critical CI changes may PR immediately on a `fix/`
or `ci/` branch. These are the ONLY valid reasons to PR before block completion.

## Banned

- `git push origin main` directly
- Merge commits (`--no-ff`)
- `--force` on main
- `--no-verify`
