# GATE -1: Sanity Check

**Purpose:** Full environment audit before any work begins. The Lead Developer owns every decision here — the user is never asked about git state, branch state, or workflow recovery.

---

## Step 1: Workspace State Audit (ALWAYS FIRST)

```bash
git status --short                        # Uncommitted changes?
git branch --show-current                 # Which branch?
git log main..HEAD --oneline              # Commits not yet on main?
```

Classify and resolve autonomously:

### State: Clean, on main
→ Normal. Proceed to Step 2.

### State: Uncommitted changes present
→ Inspect every changed file: `git diff` + `git status`
→ **Valid WIP:** stage and commit before starting new work
→ **Stale/accidental:** `git restore .` to discard
→ Rule: changes relate to current task → commit. Unknown → inspect carefully.

### State: On a feature branch with unmerged commits
→ `git log main..HEAD --oneline` + `git diff main`
→ **Work complete** (build + tests pass): push → PR → auto-merge, then continue
→ **Work incomplete:** this is the resumption point — continue here, don't create new branch

### State: Detached HEAD
→ `git checkout main` to return, then reassess.

---

## Step 2: Sync from Remote

```bash
git fetch origin
git log HEAD..origin/main --oneline       # Is remote ahead of local main?
git log origin/main..HEAD --oneline       # Commits on this branch not yet in main
git log HEAD..origin/$(git branch --show-current) --oneline  # Is remote branch ahead?
```

→ **On main, remote ahead** (PR merged): `git pull origin main`
→ **On block branch, origin/main has new commits:** `git rebase origin/main` — keep the branch current
→ **Remote branch ahead of local** (another session pushed): `git pull --rebase`
→ **All equal:** nothing to do

**Why this matters:** `strict_required_status_checks_policy=true` means auto-merge stalls
if main advanced while the block branch was in progress. Catch it here, not at PR time.

---

## Step 3: Full Build Verification

```bash
cargo build --workspace
```

**BLOCKING.** If this fails, fix it before starting new work.

---

## Step 4: Security Scan

```bash
cargo audit
```

→ Vulnerabilities in **direct deps** → STOP, fix or escalate
→ Vulnerabilities in **transitive deps only** → note and continue

---

## Step 5: Branch Setup

```bash
git checkout -b block/{name}   # One branch per block — all phases committed here
```

If resuming an existing block branch (Step 1 State 2): skip, continue on existing branch.
See `git-workflow.md` for full branch naming convention (`feat/`, `fix/`, `ci/`, `docs/`).

---

## Step 6: Phase Evaluation

1. **Read phase blockers:** Check `🚨 BLOCKERS` section in phase file
2. **Verify dependencies:** Check spec → check codebase → decide autonomously
3. **Evaluate scope:** Version scope? Dependencies met? Parity impact?

---

## Decision Authority

**Lead Developer decides autonomously:**
- All git state resolution
- All build failures (fix them)
- Resume vs new branch decisions

**Architect is informed, not consulted:**
- Significant unexpected state → note once, handle it. Never block on user response.

---

**If concerns found:** Present with evidence, act. Don't ask.
**If no concerns:** Proceed to GATE 0.
