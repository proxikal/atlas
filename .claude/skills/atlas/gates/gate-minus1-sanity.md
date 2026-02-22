# GATE -1: Sanity Check

**Purpose:** Full environment audit before any work begins. The Lead Developer owns every decision here — the user is never asked about git state, branch state, or workflow recovery. Figure it out, fix it, proceed.

---

## Step 1: Worktree State Audit (ALWAYS FIRST)

Run this before anything else:

```bash
cat .worktree-id                          # Confirm which worktree we're in
git status --short                        # Uncommitted changes?
git branch --show-current                 # Which branch?
git log main..HEAD --oneline              # Commits not yet merged to main?
git worktree list                         # State of ALL three worktrees
```

Classify the current worktree state and resolve it autonomously:

### State: Clean, on home branch (`worktree/dev` or `worktree/docs`)
→ Normal. Proceed to Step 2.

### State: Uncommitted changes present
→ Inspect every changed file: `git diff` + `git status`
→ **If changes are valid work-in-progress:** stage and commit them with an appropriate message before starting new work
→ **If changes are stale/accidental:** `git restore .` to discard
→ Decision rule: if the changes relate to the current task → commit. If unrelated or unknown → inspect carefully, commit anything meaningful, discard noise. **Never leave uncommitted changes when starting new work.**

### State: On a feature branch with unmerged commits
→ Inspect what was done: `git log main..HEAD --oneline` + `git diff main`
→ **If work is complete** (passes build + tests): merge to main now, clean up branch, continue
→ **If work is incomplete** (partial implementation): this is the resumption point — continue from here, don't create a new branch
→ Decision rule: `cargo build --workspace` passes and tests pass → it's complete, merge it. Either way, no new branch until current branch is resolved.

### State: Detached HEAD or unknown branch
→ `git checkout worktree/dev` (or `worktree/docs`) to return to home branch, then reassess.

---

## Step 2: Memory Symlink Check

Worktrees each get their own Claude project folder. Without a symlink, each worktree maintains separate memories — defeating the shared memory system.

```bash
WORKTREE_ID=$(cat .worktree-id 2>/dev/null || echo "unknown")
MAIN_MEMORY="$HOME/.claude/projects/-Users-proxikal-dev-projects-atlas/memory"

# Determine this worktree's Claude project folder
case "$WORKTREE_ID" in
  dev)  PROJECT_DIR="$HOME/.claude/projects/-Users-proxikal-dev-projects-atlas-dev" ;;
  docs) PROJECT_DIR="$HOME/.claude/projects/-Users-proxikal-dev-projects-atlas-docs" ;;
  *)    PROJECT_DIR="" ;;  # main worktree — no symlink needed
esac

# Create symlink if this is a non-main worktree and symlink is missing
if [ -n "$PROJECT_DIR" ]; then
  mkdir -p "$PROJECT_DIR"
  if [ ! -L "$PROJECT_DIR/memory" ]; then
    ln -s "$MAIN_MEMORY" "$PROJECT_DIR/memory"
    echo "Memory symlink created for worktree/$WORKTREE_ID"
  fi
fi
```

→ **If symlink already exists:** Nothing to do, proceed.
→ **If symlink was just created:** Note it once, proceed. Memory is now unified.
→ **If worktree is `main`:** No action needed — main worktree owns the canonical memory.

---

## Step 3: Sync from Remote

```bash
git fetch origin                          # Download remote state (safe, touches nothing)
git log HEAD..origin/main --oneline       # Is remote ahead of local main?
```

→ **If remote is ahead** (PR merged since last session): `git checkout main && git pull origin main` — fast-forward local main, return to feature/home branch
→ **If remote is equal** (no new merges): nothing to do

**Sync model:** All work goes through PRs → merge queue → squash onto origin/main. Pull to sync, never rebase origin/main directly.

---

## Step 4: Other Worktrees (Awareness Only)

Read `git worktree list` output from Step 1. For each other worktree:

| Other worktree state | Action |
|---|---|
| Clean, on home branch | Nothing to do |
| On feature branch | Note it — that session has work in progress, don't interfere |
| Has `[modified]` indicator | Note it — that session has uncommitted changes, warn user once at session start |

**The agent never touches another worktree's files or branches.** Awareness only.

---

## Step 5: Branch Setup

```bash
git rebase main                           # Sync home branch to local main (NOT origin/main)
git checkout -b phase/{category}-{number} # Create feature branch for this session's work
```

If resuming an existing feature branch (from Step 1 State 2): skip branch creation, continue on existing branch.

---

## Step 6: Full Build Verification

```bash
cargo build --workspace
```

**BLOCKING.** If this fails, the codebase is broken. Fix it before proceeding — do not start new work on a broken foundation.

---

## Step 7: Security Scan

```bash
cargo audit
```

→ Vulnerabilities in **direct deps** → STOP, fix or escalate
→ Vulnerabilities in **transitive deps only** → note and continue

```bash
# Install if needed (one-time)
cargo install cargo-audit
```

---

## Step 8: Phase Evaluation

1. **Read phase blockers:** Check `🚨 BLOCKERS` section in phase file
2. **Verify dependencies:** Check spec → check codebase → decide autonomously
3. **Evaluate scope:** Version scope? Dependencies met? Parity impact? Workload reasonable?

---

## Decision Authority

**Lead Developer decides autonomously:**
- All git state resolution (uncommitted work, stale branches, merge decisions)
- All build failures (fix them)
- All dependency checks
- Resume vs new branch decisions

**Architect is informed, not consulted:**
- If a significant unexpected state is found (e.g., large unrecognised uncommitted change), note it once in the session opening, then handle it. Never block on user response.

---

**If concerns found:** Present with evidence, decide and act. Don't ask.
**If no concerns:** Proceed to GATE 0.

**Next:** GATE 0
