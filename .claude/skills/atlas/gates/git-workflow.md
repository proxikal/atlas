# Git Workflow

**Full rules:** `.claude/rules/atlas-git.md` (auto-loaded)
**Single workspace:** `~/dev/projects/atlas/` — no other worktrees.

## Branch Naming
```
block/{name}                # e.g. block/trait-system — ONE branch per block (primary)
feat/{short-description}    # standalone features outside block plan
fix/{short-description}     # blocking fixes (may PR immediately)
ci/{short-description}      # CI/infra
docs/{short-description}    # docs-only
```

## Start of Block (Scaffold Session)
```bash
git checkout main && git pull origin main
git checkout -b block/{name}
# scaffold phase files → commit → NO push, NO PR
```

## Start of Phase (within a block)
```bash
# Already on block/{name} branch from scaffold — no branch switch needed
git pull origin main --rebase  # keep up to date if main has moved
```

## During Phase (multi-part)
```bash
cargo build --workspace
cargo nextest run -p atlas-runtime
git add <files> && git commit -m "feat(phase-XX): Part A"
```

## End of Phase — Commit Only (Batching)
```bash
# 1. Quality gates
cargo build --workspace
cargo nextest run -p atlas-runtime
cargo clippy -p atlas-runtime -- -D warnings
cargo fmt --check -p atlas-runtime

# 2. Commit (do NOT push or PR yet — batch multiple phases)
git add <files> && git commit -m "$(cat <<'EOF'
feat(block-XX/phase-YY): Description

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>
EOF
)"
```

## PR Workflow — Block Complete Flush
```bash
# ONLY when the block's final AC check phase is committed:

# 1. Rebase on latest main BEFORE pushing (strict CI policy requires up-to-date branch)
git fetch origin
git rebase origin/main   # resolve any conflicts; re-run tests if rebase had changes

# 2. Push and open PR
git push -u origin block/{name}
gh pr create --title "feat(block-XX): ..." --body "..."
gh pr merge --auto --squash

# After merge: sync and clean up
git checkout main && git pull origin main
git branch -d block/{name}
```

**Why rebase before push:** `strict_required_status_checks_policy=true` in the ruleset
means GitHub auto-merge will stall if any commit landed on main after the branch was
last rebased. Always rebase immediately before push to guarantee auto-merge proceeds.

**Exception:** Blocking fixes (`fix/`) or CI issues (`ci/`) may PR immediately — these are the ONLY valid early-PR cases.

## Banned
- `git push origin main` directly
- `--no-ff` merges
- `--force` on main
- `--no-verify`
