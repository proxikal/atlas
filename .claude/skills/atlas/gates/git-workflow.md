# Git Workflow

**Full rules:** `.claude/rules/atlas-git.md` (auto-loaded)
**Single workspace:** `~/dev/projects/atlas/` — no other worktrees.

## Branch Naming
```
phase/{category}-{number}   # e.g. phase/ownership-01
feat/{short-description}
fix/{short-description}
ci/{short-description}
```

## Start of Phase
```bash
git checkout main && git pull origin main
git checkout -b phase/{category}-{number}
```

## During Phase (multi-part)
```bash
cargo build --workspace
cargo nextest run -p atlas-runtime
git add <files> && git commit -m "feat(phase-XX): Part A"
```

## End of Phase — PR Workflow
```bash
# 1. Quality gates
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

# 3. PR with auto-merge
gh pr create --title "..." --body "..."
gh pr merge --auto --squash

# 4. After merge: sync
git checkout main && git pull origin main
git branch -d phase/{category}-{number}
```

## Banned
- `git push origin main` directly
- `--no-ff` merges
- `--force` on main
- `--no-verify`
