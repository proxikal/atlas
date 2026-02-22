# Git Workflow

**Full rules:** `.claude/rules/atlas-git.md` (loaded automatically)

## Branch Naming
```
phase/{category}-{number}   # e.g. phase/ownership-01
feat/{short-description}
fix/{short-description}
ci/{short-description}
```

## Start of Phase
```bash
git checkout main && git pull origin main   # Sync to remote main
git checkout -b phase/{category}-{number}   # Feature branch
```

## During Phase (multi-part)
```bash
cargo build --workspace                     # Must pass
cargo nextest run -p atlas-runtime          # Must be 100%
git add <files> && git commit -m "feat(phase-XX): Part A"
```

## End of Phase — PR Workflow
```bash
# 1. Final quality gates
cargo build --workspace
cargo nextest run -p atlas-runtime
cargo clippy -p atlas-runtime -- -D warnings
cargo fmt --check -p atlas-runtime

# 2. Commit + push
git add <files> && git commit -m "$(cat <<'EOF'
feat(phase-XX): Description

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>
EOF
)"
git push -u origin phase/{category}-{number}

# 3. PR with auto-merge (merge queue handles the rest)
gh pr create --title "..." --body "..."
gh pr merge --auto --squash

# 4. After merge: sync local main + other worktrees
git checkout main && git pull origin main
git branch -d phase/{category}-{number}
git -C /Users/proxikal/dev/projects/atlas-docs rebase main
git -C /Users/proxikal/dev/projects/atlas rebase main
```

## Weekly Push Cadence
Batch multiple phases into one PR per week. Do NOT push every phase.
Exception: blocking fixes or large milestones.

## Banned
- `git push origin main` directly (branch protection rejects it)
- Merge commits on main (`--no-ff`)
- `--force` on main
- `--no-verify` (skip CI)
- Leaving uncommitted changes at session end
