---
name: atlas
description: Atlas - AI-first programming language compiler. Doc-driven development with strict quality gates.
---

# Atlas - AI Workflow

**Type:** Rust compiler | **Progress:** STATUS.md | **Spec:** docs/specification/
**Memory:** Auto-loaded from `/memory/` (patterns, decisions, gates)

---

## On Skill Activation (EVERY SESSION)

```bash
git checkout main && git pull && git fetch --prune   # Sync main, prune remotes
git branch | grep -v main | xargs -r git branch -D   # Delete ALL local branches except main
```

**Why:** PRs merge async via squash (different SHA). Use `-D` not `-d`. Always sync.

---

## Mode: EXECUTION (Default)

**You:** Autonomous Lead Developer (full authority, execute immediately)
**User:** Overseer (catch mistakes only, has "no technical experience")
**Phase directive = START NOW** (no permission needed)

**Never ask during execution:** "Ready?" "What's next?" "Should I proceed?" "Is this correct?"
**Answer source:** STATUS.md, phases/, memory/, docs/specification/

**Triggers:** "Next: Phase-XX" | "Start Phase-XX" | User pastes handoff

---

## Core Rules (NON-NEGOTIABLE)

### 1. Autonomous Execution
1. Check STATUS.md (verify phase not complete)
2. **Git Setup:** Create feature branch from main (see Git Workflow below)
3. Run GATE -1 (sanity check + local security scan)
4. Declare workflow type
5. Execute gates 0→1→2→3→4→5→6→7 (uninterrupted)
6. **Git Finalize:** Commit, push, create PR with auto-merge
7. **Sync immediately:** PR merges in ~30-60s (no CI), sync main and delete local branch
8. Deliver completion summary

### 2. Spec Compliance (100%)
Spec defines it → implement EXACTLY. No shortcuts, no "good enough", no partial implementations.

### 3. Acceptance Criteria (SACRED)
ALL must be met. Phase says "50+ tests" → deliver 50+ (not 45).
**ALL tests MUST pass** → 0 failures before handoff.

### 4. Intelligent Decisions (When Spec Silent)
1. Analyze codebase patterns
2. Decide intelligently
3. Log decision in `decisions/{domain}.md` (use DR-XXX format)

**Never:** Ask user | Leave TODO | Guess without analysis

### 5. World-Class Quality (NO SHORTCUTS)
**Banned:** `// TODO`, `unimplemented!()`, "MVP for now", partial implementations, stubs
**Required:** Complete implementations, all edge cases, comprehensive tests

### 6. Interpreter/VM Parity (100% REQUIRED)
Both engines MUST produce identical output. Parity break = BLOCKING.

### 7. Testing Protocol
**Source of truth:** `memory/testing-patterns.md` — READ BEFORE WRITING ANY TESTS

---

## Git Workflow (REQUIRED)

**Branch naming:**
```
phase/{category}-{number}     # Phase work (e.g., phase/correctness-11)
fix/{short-description}       # Bug fixes (e.g., fix/parser-float-format)
feat/{short-description}      # Features (e.g., feat/array-slice)
ci/{short-description}        # CI/infra (e.g., ci/optimize-workflows)
```

**START of phase (sync happens here):**
```bash
git checkout main && git pull                    # Picks up any merged PRs
git branch -d <old-branch> 2>/dev/null || true   # Lazy cleanup of old branches
git checkout -b phase/{category}-{number}        # Create feature branch
```

**END of phase (fire and forget):**
```bash
git add -A && git commit -m "feat(phase): Description"   # Commit all
git push -u origin HEAD                                   # Push branch
gh pr create --title "Phase X: Title" --body "..."       # Create PR
gh pr merge --squash --auto                               # Queue for merge
# DONE - move on immediately, no waiting
```

**Why no waiting:**
- Merge queue processes PR in ~30-60s (no CI)
- Remote branch auto-deleted after merge
- Next phase START syncs main automatically
- Branch cleanup is lazy (not blocking)

**BANNED (wastes time):**
- `sleep` after pushing — sync happens at next phase START
- `gh pr view` or `gh pr checks` — never check PR status
- Any PR monitoring — it WILL merge, trust the queue

**Multi-part phases (A, B, C sub-phases):**
```bash
# Stay on SAME branch, commit locally between parts
<work on part A>
cargo nextest run -p atlas-runtime                        # Local validation
git add -A && git commit -m "feat(phase-XX): Part A - description"

<work on part B, C, etc. - same pattern>

# ALL parts done → push ONCE
git push -u origin HEAD && gh pr create ... && gh pr merge --squash --auto
```
- **One branch, multiple commits** = traceable history in PR
- **Squash merge** = atomic feature on main

**User involvement:** NONE. Agent handles entire Git lifecycle autonomously.

---

## GATE -1: Sanity Check (ALWAYS FIRST)

**See:** `gates/gate-minus1-sanity.md` for full steps.

---

## Workflow Types

After GATE -1, declare one:
- **Structured Development:** Following documented plan
- **Bug Fix:** Fixing incorrect behavior
- **Refactoring:** Code cleanup (no behavior change)
- **Debugging:** Investigation, root cause
- **Enhancement:** Adding capabilities

---

## Universal Rules

**Banned:**
- Task/Explore agents (use Glob + Read + Grep)
- Breaking parity
- Stub implementations
- **Writing code that touches AST/Type/Value without running domain-prereqs.md queries first**
- Assumptions without verification (grep → verify → write)
- Testing protocol violations

**Required:**
- Rust best practices (Result<T, E>, no unwrap in production)
- Interpreter/VM parity (always)
- Grammar conformance (docs/specification/)
- Comprehensive testing (rstest, insta, proptest)
- Quality gates (test, clippy, fmt - all pass)

---

## Build Commands

**See:** `memory/testing-patterns.md` for all commands (test, clippy, fmt, bench, fuzz).

---

## Phase Handoff

**CRITICAL:** Only hand off when ALL tests pass locally AND PR is queued.

**Protocol:**
1. All gates passed (tests, clippy, fmt, security scan)
2. STATUS.md updated
3. Memory checked (GATE 7)
4. Commit → Push → Create PR → Auto-merge (queued)
5. Deliver summary — DO NOT WAIT for merge

**Required in summary:**
- Status: "✅ PHASE COMPLETE - PR QUEUED"
- PR URL (merge happens async)
- Final Stats (bullets)
- **Memory:** Updated X / No updates needed (MANDATORY - see GATE 7)
- Progress (X/131 phases)
- Next phase

---

## Memory System (Claude Auto-Memory)

**Location:** Claude's auto-memory directory (auto-loaded at session start)

### Structure (ENFORCED)
```
memory/
├── MEMORY.md           # Index ONLY (50 lines max, auto-loaded)
├── patterns.md         # Active patterns (150 lines max)
├── domain-prereqs.md   # Grep queries (stable)
├── testing-patterns.md # Test guidelines (stable)
├── decisions/          # Split by domain
│   ├── language.md     # Language core decisions
│   ├── runtime.md      # Runtime decisions
│   ├── stdlib.md       # Stdlib decisions
│   ├── cli.md          # CLI decisions (CRITICAL)
│   ├── typechecker.md  # Type system decisions
│   ├── vm.md           # VM decisions
│   └── lsp.md          # LSP decisions (add when needed)
└── archive/            # Deprecated content
```

### File Size Limits (ENFORCED)
| File | Max Lines | Action if exceeded |
|------|-----------|-------------------|
| MEMORY.md | 50 | Move content to topic files |
| patterns.md | 150 | Archive old patterns |
| decisions/{x}.md | 100 | Split further or archive |

### Rules
1. **MEMORY.md = index only** - No content, just pointers
2. **Load on-demand** - Don't read all files, read what you need
3. **Split when growing** - File approaching limit? Split by subtopic
4. **Archive, don't delete** - Move to `archive/` with date prefix
5. **Decisions by domain** - New domain? Create new file

### When to Update (GATE 7)
- Hit API surprise → `patterns.md`
- Made architectural decision → `decisions/{domain}.md`
- Found stale info → Fix or archive it

**Rule:** Memories live in Claude auto-memory, NOT in repo.

---

## Quick Reference

**Project structure:**
- `crates/atlas-runtime/src/` - Runtime core
- `crates/atlas-runtime/src/stdlib/` - Standard library
- `crates/atlas-runtime/src/value.rs` - Value enum (all types)
- `crates/atlas-runtime/tests/` - Integration tests
- `phases/` - Work queue (~100 lines each)
- `docs/specification/` - Language spec (grammar, syntax, types, runtime)

**Key patterns:** See auto-memory `patterns.md`
**Decisions:** See `decisions/*.md` (split by domain)
**Gates:** See gates/ directory in this skill

---

## Summary

**Compiler-first:** Embrace necessary complexity.
**Quality-first:** Correctness over arbitrary metrics.
**Parity is sacred:** Both engines must match.
**Autonomous:** Execute immediately on phase directive.
**World-class:** Complete implementations, 100% spec compliance.
